use std::sync::Arc;

use crate::messages::{
    codec::LanguageServerCodec,
    core::LspRequest,
    groups::{
        responses::errors::{DecodeErrorResponse, InternalErrorResponse, ResponseErrors},
        AllMessages,
    },
};
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    join, SinkExt, StreamExt,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::service::error::{OUTPUT_SINK_CLOSED, SERVICE_BACKEND_CLOSED};

type FramedOutputLock<O> = Arc<Mutex<FramedWrite<O, LanguageServerCodec<AllMessages>>>>;

pub struct ServiceFrontend;

impl ServiceFrontend {
    pub async fn run<I: AsyncRead + Unpin, O: AsyncWrite + Unpin>(
        read_input: I,
        write_output: O,
        agent_incoming_tx: UnboundedSender<AllMessages>,
        mut agent_outgoing_rx: UnboundedReceiver<AllMessages>,
    ) -> Result<(), ()> {
        let mut framed_read_input =
            FramedRead::new(read_input, LanguageServerCodec::<AllMessages>::default());
        let mut framed_write_lock = Arc::new(Mutex::new(FramedWrite::new(
            write_output,
            LanguageServerCodec::<AllMessages>::default(),
        )));

        let framed_write_lock_clone = framed_write_lock.clone();

        loop {
            join!(
                Self::forward_write_output(&framed_write_lock, &mut agent_outgoing_rx),
                Self::forward_read_input(
                    &framed_write_lock_clone,
                    &mut framed_read_input,
                    &agent_incoming_tx
                )
            );
        }
    }

    async fn forward_write_output<O: AsyncWrite + Unpin>(
        framed_write_lock: &FramedOutputLock<O>,
        outgoing_rx: &mut UnboundedReceiver<AllMessages>,
    ) {
        while let Some(message) = outgoing_rx.next().await {
            framed_write_lock
                .lock()
                .await
                .send(message)
                .await
                .expect(OUTPUT_SINK_CLOSED)
        }
    }

    async fn forward_read_input<I: AsyncRead + Unpin, O: AsyncWrite + Unpin>(
        framed_write_lock: &FramedOutputLock<O>,
        framed_read_input: &mut FramedRead<I, LanguageServerCodec<AllMessages>>,
        agent_incoming_tx: &UnboundedSender<AllMessages>,
    ) {
        if let Err(response_error) =
            try_forward_read_input(framed_read_input, agent_incoming_tx).await
        {
            let is_recoverable = response_error.is_recoverable();
            framed_write_lock
                .lock()
                .await
                .send(response_error.into())
                .await
                .expect(OUTPUT_SINK_CLOSED);

            if !is_recoverable {
                std::process::exit(1)
            }
        }

        async fn try_forward_read_input<I: AsyncRead + Unpin>(
            framed_read_input: &mut FramedRead<I, LanguageServerCodec<AllMessages>>,
            agent_incoming_tx: &UnboundedSender<AllMessages>,
        ) -> Result<(), ResponseErrors> {
            while let Some(incoming_decode) = framed_read_input.next().await {
                match incoming_decode {
                    Ok(message) => {
                        let request_id = match &message {
                            AllMessages::Requests(request) => Some(request.request_id().clone()),
                            AllMessages::Responses(_) | AllMessages::Notifications(_) => None,
                        };

                        if agent_incoming_tx.unbounded_send(message).is_err() {
                            return Err(InternalErrorResponse::new(
                                request_id,
                                SERVICE_BACKEND_CLOSED,
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
    use std::io::Cursor;

    use tokio::io::DuplexStream;
    use tokio_util::bytes::BytesMut;

    use super::*;

    // TEMP:
    async fn formwards_message() {
        let (agent_outgoing_tx, agent_outgoing_rx) =
            futures::channel::mpsc::unbounded::<AllMessages>();
        let (agent_incoming_tx, agent_incoming_rx) =
            futures::channel::mpsc::unbounded::<AllMessages>();

        let (service_input, input_handle) = tokio_test::io::Builder::new().build_with_handle();
        let (service_output, output_handle) = tokio_test::io::Builder::new().build_with_handle();

        let service_handle = tokio::spawn(ServiceFrontend::run(
            service_input,
            service_output,
            agent_incoming_tx,
            agent_outgoing_rx,
        ));
    }
}
