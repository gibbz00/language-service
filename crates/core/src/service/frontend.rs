use std::sync::Arc;

use crate::{
    messages::{
        codec::LanguageServerCodec,
        groups::{
            responses::errors::{DecodeErrorResponse, ResponseErrors},
            AllMessages,
        },
    },
    service::error::{MESSAGE_FILTER_INPUT_CLOSED, MESSAGE_FILTER_OUTPUT_CLOSED},
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

use crate::service::error::OUTPUT_CLOSED;

type FramedInput<I> = FramedRead<I, LanguageServerCodec<AllMessages>>;
type FramedOutputLock<O> = Arc<Mutex<FramedWrite<O, LanguageServerCodec<AllMessages>>>>;

pub(crate) struct ServiceFrontend<I: AsyncRead + Unpin, O: AsyncWrite + Unpin> {
    framed_input: FramedInput<I>,
    framed_output: FramedOutputLock<O>,
    framed_output_clone: FramedOutputLock<O>,
    message_filter_tx: UnboundedSender<AllMessages>,
    message_filter_rx: UnboundedReceiver<AllMessages>,
}

impl<I: AsyncRead + Unpin, O: AsyncWrite + Unpin> ServiceFrontend<I, O> {
    pub fn new(
        read_input: I,
        write_output: O,
        message_filter_tx: UnboundedSender<AllMessages>,
        message_filter_rx: UnboundedReceiver<AllMessages>,
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
            message_filter_tx,
            message_filter_rx,
        }
    }

    pub async fn tick(&mut self) {
        join!(
            Self::try_forward_to_message_filter(
                &self.framed_output_clone,
                &mut self.framed_input,
                &self.message_filter_tx
            ),
            Self::forward_from_message_filter(&self.framed_output, &mut self.message_filter_rx),
        );
    }

    async fn forward_from_message_filter(
        framed_write_lock: &FramedOutputLock<O>,
        from_backend_rx: &mut UnboundedReceiver<AllMessages>,
    ) {
        if let Ok(channel_result) = from_backend_rx.try_next() {
            let mut output_guard = framed_write_lock.lock().await;
            let message = channel_result.expect(MESSAGE_FILTER_OUTPUT_CLOSED);

            tracing::debug!(
                ?message,
                "Forwarding message from message_filter to writer."
            );

            output_guard.send(message).await.expect(OUTPUT_CLOSED)
        }
    }

    async fn try_forward_to_message_filter(
        framed_write_lock: &FramedOutputLock<O>,
        framed_read_input: &mut FramedRead<I, LanguageServerCodec<AllMessages>>,
        backend_tx: &UnboundedSender<AllMessages>,
    ) {
        if let Some(Some(message_decode_attempt)) = framed_read_input.next().now_or_never() {
            match message_decode_attempt {
                Ok(message) => {
                    tracing::debug!(
                        ?message,
                        "Forwarding message from reader to message filter."
                    );
                    backend_tx
                        .unbounded_send(message)
                        .expect(MESSAGE_FILTER_INPUT_CLOSED)
                }

                Err(err) => {
                    framed_write_lock
                        .lock()
                        .await
                        .send(ResponseErrors::Decode(DecodeErrorResponse::new(err)).into())
                        .await
                        .expect(OUTPUT_CLOSED);
                }
            }
        }
    }
}
