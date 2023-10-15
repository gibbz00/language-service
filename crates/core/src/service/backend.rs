use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::service::error::BACKEND_INPUT_CLOSED;

use super::{
    error::BACKEND_OUTPUT_CLOSED,
    filter::{IncomingMessage, MessageFilter, OutgoingMessage},
};

pub(crate) struct ServiceBackend<F: MessageFilter> {
    backend_rx: UnboundedReceiver<IncomingMessage<F>>,
    backend_tx: UnboundedSender<OutgoingMessage<F>>,
}

impl<F: MessageFilter> ServiceBackend<F> {
    pub fn new(
        backend_rx: UnboundedReceiver<IncomingMessage<F>>,
        backend_tx: UnboundedSender<OutgoingMessage<F>>,
    ) -> Self {
        Self {
            backend_rx,
            backend_tx,
        }
    }

    pub fn get_incoming(&mut self) -> Option<IncomingMessage<F>> {
        match self.backend_rx.try_next() {
            Ok(message_result) => Some(message_result.expect(BACKEND_INPUT_CLOSED)),
            Err(_) => None,
        }
    }

    pub fn send_outgoing(&self, message: OutgoingMessage<F>) {
        self.backend_tx
            .unbounded_send(message)
            .expect(BACKEND_OUTPUT_CLOSED)
    }
}
