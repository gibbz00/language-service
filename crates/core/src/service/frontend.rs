use std::sync::Arc;

use crate::{
    messages::{
        codec::LanguageServerCodec,
        core::LspRequest,
        groups::{
            responses::errors::{DecodeErrorResponse, InternalErrorResponse, ResponseErrors},
            AllMessages,
        },
    },
    service::error::BACKEND_OUTPUT_CLOSED,
};
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    join, FutureExt, SinkExt, StreamExt,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::service::error::{BACKEND_INPUT_CLOSED, OUTPUT_CLOSED};

type FramedInput<I> = FramedRead<I, LanguageServerCodec<AllMessages>>;
type FramedOutputLock<O> = Arc<Mutex<FramedWrite<O, LanguageServerCodec<AllMessages>>>>;

pub struct ServiceFrontend<I: AsyncRead + Unpin, O: AsyncWrite + Unpin> {
    framed_input: FramedInput<I>,
    framed_output: FramedOutputLock<O>,
    framed_output_clone: FramedOutputLock<O>,
    to_backend_tx: UnboundedSender<AllMessages>,
    from_backend_rx: UnboundedReceiver<AllMessages>,
}

#[allow(unused)]
impl<I: AsyncRead + Unpin, O: AsyncWrite + Unpin> ServiceFrontend<I, O> {
    pub fn new(
        read_input: I,
        write_output: O,
        to_backend_tx: UnboundedSender<AllMessages>,
        from_backend_rx: UnboundedReceiver<AllMessages>,
    ) -> Self {
        let framed_output = Arc::new(Mutex::new(FramedWrite::new(
            write_output,
            LanguageServerCodec::<AllMessages>::default(),
        )));

        Self {
            framed_input: FramedRead::new(
                read_input,
                LanguageServerCodec::<AllMessages>::default(),
            ),
            framed_output_clone: framed_output.clone(),
            framed_output,
            to_backend_tx,
            from_backend_rx,
        }
    }

    pub async fn run(mut self) -> Result<(), ()> {
        loop {
            self.tick().await
        }
    }

    pub async fn tick(&mut self) {
        join!(
            Self::forward_from_backend(&self.framed_output, &mut self.from_backend_rx),
            Self::forward_to_backend(
                &self.framed_output_clone,
                &mut self.framed_input,
                &self.to_backend_tx
            )
        );
    }

    async fn forward_from_backend(
        framed_write_lock: &FramedOutputLock<O>,
        from_backend_rx: &mut UnboundedReceiver<AllMessages>,
    ) {
        if let Ok(channel_result) = from_backend_rx.try_next() {
            let mut output_guard = framed_write_lock.lock().await;
            match channel_result {
                Some(message) => {
                    tracing::debug!(?message, "Forwarding message from backend to writer.");
                    output_guard.send(message).await.expect(OUTPUT_CLOSED)
                }
                None => output_guard
                    .send(InternalErrorResponse::new(None, BACKEND_OUTPUT_CLOSED).into())
                    .await
                    .expect(OUTPUT_CLOSED),
            }
        }
    }

    async fn forward_to_backend(
        framed_write_lock: &FramedOutputLock<O>,
        framed_read_input: &mut FramedRead<I, LanguageServerCodec<AllMessages>>,
        to_backend_tx: &UnboundedSender<AllMessages>,
    ) {
        if let Err(response_error) = try_forward_read_input(framed_read_input, to_backend_tx).await
        {
            let is_recoverable = response_error.is_recoverable();
            framed_write_lock
                .lock()
                .await
                .send(response_error.into())
                .await
                .expect(OUTPUT_CLOSED);

            if !is_recoverable {
                std::process::exit(1)
            }
        }

        async fn try_forward_read_input<I: AsyncRead + Unpin>(
            framed_read_input: &mut FramedRead<I, LanguageServerCodec<AllMessages>>,
            to_backend_tx: &UnboundedSender<AllMessages>,
        ) -> Result<(), ResponseErrors> {
            if let Some(Some(message_decode_attempt)) = framed_read_input.next().now_or_never() {
                match message_decode_attempt {
                    Ok(message) => {
                        tracing::debug!(?message, "Forwarding message from reader to backend.");
                        let request_id = match &message {
                            AllMessages::Requests(request) => Some(request.request_id().clone()),
                            AllMessages::Responses(_) | AllMessages::Notifications(_) => None,
                        };

                        if to_backend_tx.unbounded_send(message).is_err() {
                            return Err(InternalErrorResponse::new(
                                request_id,
                                BACKEND_INPUT_CLOSED,
                            )
                            .into());
                        }
                    }

                    Err(err) => return Err(DecodeErrorResponse::new(err).into()),
                }
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncWriteExt;

    use crate::messages::{groups::tests::MESSAGE_MOCK, payload::tests::PAYLOAD_MOCK};

    use super::*;

    #[test_log::test(tokio::test)]
    async fn forwards_payload_to_backend() {
        let (_to_frontend_tx, from_backend_rx) = futures::channel::mpsc::unbounded::<AllMessages>();
        let (to_backend_tx, mut from_frontend_rx) =
            futures::channel::mpsc::unbounded::<AllMessages>();

        let (service_input, mut input_handle) = tokio::io::duplex(2000);
        let (service_output, _output_handle) = tokio::io::duplex(2000);

        let mut service_frontend = ServiceFrontend::new(
            service_input,
            service_output,
            to_backend_tx,
            from_backend_rx,
        );

        let bytes_written = input_handle.write(PAYLOAD_MOCK.as_bytes()).await.unwrap();
        // TODO: assert that bytes written == payload.length
        println!("{:?}", bytes_written);

        service_frontend.tick().await;

        assert_eq!(MESSAGE_MOCK, from_frontend_rx.next().await.unwrap());
    }
}
