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
    backend_tx: UnboundedSender<AllMessages>,
    backend_rx: UnboundedReceiver<AllMessages>,
}

// TEMP:
#[allow(dead_code)]
impl<I: AsyncRead + Unpin, O: AsyncWrite + Unpin> ServiceFrontend<I, O> {
    pub fn new(
        read_input: I,
        write_output: O,
        backend_tx: UnboundedSender<AllMessages>,
        backend_rx: UnboundedReceiver<AllMessages>,
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
            backend_tx,
            backend_rx,
        }
    }

    pub async fn run(mut self) -> Result<(), ()> {
        loop {
            self.tick().await
        }
    }

    pub async fn tick(&mut self) {
        join!(
            Self::forward_from_backend(&self.framed_output, &mut self.backend_rx),
            Self::forward_to_backend(
                &self.framed_output_clone,
                &mut self.framed_input,
                &self.backend_tx
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
        backend_tx: &UnboundedSender<AllMessages>,
    ) {
        if let Err(response_error) = try_forward_read_input(framed_read_input, backend_tx).await {
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
            backend_tx: &UnboundedSender<AllMessages>,
        ) -> Result<(), ResponseErrors> {
            if let Some(Some(message_decode_attempt)) = framed_read_input.next().now_or_never() {
                match message_decode_attempt {
                    Ok(message) => {
                        tracing::debug!(?message, "Forwarding message from reader to backend.");
                        let request_id = match &message {
                            AllMessages::Requests(request) => Some(request.request_id().clone()),
                            AllMessages::Responses(_) | AllMessages::Notifications(_) => None,
                        };

                        if backend_tx.unbounded_send(message).is_err() {
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
    use bytes::BytesMut;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
    use tokio_util::codec::Decoder;

    use crate::{
        messages::{groups::tests::MESSAGE_MOCK, payload::Payload},
        service::error::FRONTEND_INPUT_CLOSED,
    };

    use super::*;

    struct ServiceFrontendDriver {
        service_frontend: ServiceFrontend<DuplexStream, DuplexStream>,
        backend_tx: UnboundedSender<AllMessages>,
        backend_rx: UnboundedReceiver<AllMessages>,
        input_handle: DuplexStream,
        output_handle: DuplexStream,
    }

    impl ServiceFrontendDriver {
        const MAX_PAYLOAD_BYTES: usize = 1_000_000;

        pub async fn send_input_message(&mut self, message: AllMessages) {
            let payload = Payload::new(message).to_string();

            let bytes_written = self.input_handle.write(payload.as_bytes()).await.unwrap();
            let payload_size = payload.as_bytes().len();

            if bytes_written < payload_size {
                panic!(
                    "unable to send payload in one write, payload size: {}, bytes_written: {}",
                    payload_size, bytes_written
                )
            }
        }

        pub async fn get_output_message(&mut self) -> Option<AllMessages> {
            let mut buffer = vec![0u8; Self::MAX_PAYLOAD_BYTES];
            let bytes_read = self
                .output_handle
                .read(&mut buffer)
                .await
                .expect(OUTPUT_CLOSED);

            LanguageServerCodec::default()
                .decode(&mut BytesMut::from(&buffer[..bytes_read]))
                .expect("invalid payload encoding")
        }

        pub async fn get_message_at_backend(&mut self) -> Option<AllMessages> {
            self.backend_rx.next().await
        }

        pub fn send_message_from_backend(&mut self, message: AllMessages) {
            self.backend_tx
                .unbounded_send(message)
                .expect(FRONTEND_INPUT_CLOSED)
        }

        pub async fn tick(&mut self) {
            self.service_frontend.tick().await
        }
    }

    impl Default for ServiceFrontendDriver {
        fn default() -> Self {
            let (frontend_tx, backend_rx) = futures::channel::mpsc::unbounded::<AllMessages>();
            let (backend_tx, frontend_rx) = futures::channel::mpsc::unbounded::<AllMessages>();
            let (service_input, input_handle) = tokio::io::duplex(Self::MAX_PAYLOAD_BYTES);
            let (service_output, output_handle) = tokio::io::duplex(Self::MAX_PAYLOAD_BYTES);

            Self {
                service_frontend: ServiceFrontend::new(
                    service_input,
                    service_output,
                    frontend_tx,
                    frontend_rx,
                ),
                backend_tx,
                backend_rx,
                input_handle,
                output_handle,
            }
        }
    }

    #[test_log::test(tokio::test)]
    async fn forwards_payload_to_backend() {
        let mut service_frontend_harness = ServiceFrontendDriver::default();
        service_frontend_harness
            .send_input_message(MESSAGE_MOCK)
            .await;
        service_frontend_harness.tick().await;
        assert!(service_frontend_harness
            .get_message_at_backend()
            .await
            .is_some_and(|message| message == MESSAGE_MOCK))
    }

    #[test_log::test(tokio::test)]
    async fn outputs_payload_from_backend() {
        let mut service_frontend_harness = ServiceFrontendDriver::default();
        service_frontend_harness.send_message_from_backend(MESSAGE_MOCK);
        service_frontend_harness.tick().await;
        assert!(service_frontend_harness
            .get_output_message()
            .await
            .is_some_and(|message| message == MESSAGE_MOCK))
    }
}
