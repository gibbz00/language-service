use std::marker::PhantomData;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    messages::groups::{
        notifications::AllNotifications,
        requests::AllRequests,
        responses::{
            errors::{InvalidMessageResponse, ResponseErrors},
            AllResponses,
        },
        AllMessages,
    },
    service::error::BACKEND_OUTPUT_CLOSED,
};

use super::error::{BACKEND_INPUT_CLOSED, FRONTEND_INPUT_CLOSED, FRONTEND_OUTPUT_CLOSED};

pub(crate) struct ServiceMessageFilter<F: MessageFilter> {
    frontend_rx: UnboundedReceiver<AllMessages>,
    frontend_tx: UnboundedSender<AllMessages>,
    backend_rx: UnboundedReceiver<OutgoingMessage<F>>,
    backend_tx: UnboundedSender<IncomingMessage<F>>,
    filter_marker: PhantomData<F>,
}

impl<F: MessageFilter> ServiceMessageFilter<F> {
    pub fn new(
        frontend_rx: UnboundedReceiver<AllMessages>,
        frontend_tx: UnboundedSender<AllMessages>,
        backend_rx: UnboundedReceiver<OutgoingMessage<F>>,
        backend_tx: UnboundedSender<IncomingMessage<F>>,
    ) -> Self {
        Self {
            frontend_rx,
            frontend_tx,
            backend_rx,
            backend_tx,
            filter_marker: PhantomData,
        }
    }

    pub fn tick(&mut self) {
        self.try_forward_to_backend();
        self.forward_to_frontend();
    }

    pub fn forward_to_frontend(&mut self) {
        if let Ok(message_result) = self.backend_rx.try_next() {
            let message = message_result.expect(BACKEND_OUTPUT_CLOSED);
            self.frontend_tx
                .unbounded_send(message.into())
                .expect(FRONTEND_INPUT_CLOSED)
        }
    }

    pub fn try_forward_to_backend(&mut self) {
        if let Ok(message_result) = self.frontend_rx.try_next() {
            let message = message_result.expect(FRONTEND_OUTPUT_CLOSED);

            match IncomingMessage::<F>::try_from(message) {
                Ok(incoming_message) => self
                    .backend_tx
                    .unbounded_send(incoming_message)
                    .expect(BACKEND_INPUT_CLOSED),
                Err(try_from_err) => {
                    self.frontend_tx
                        .unbounded_send(ResponseErrors::from(try_from_err).into())
                        .expect(FRONTEND_INPUT_CLOSED);
                }
            }
        }
    }
}

pub trait MessageFilter {
    type OutgoingNotifications: Into<AllNotifications>;
    type OutgoingRequests: Into<AllRequests>;
    type OutgoingResponses: Into<AllResponses>;
    type IncomingNotifications: TryFrom<AllNotifications, Error = InvalidMessageResponse>;
    type IncomingRequests: TryFrom<AllRequests, Error = InvalidMessageResponse>;
    type IncomingResponses: TryFrom<AllResponses, Error = InvalidMessageResponse>;
}

#[derive(Debug, PartialEq)]
pub enum IncomingMessage<F: MessageFilter> {
    Notification(F::IncomingNotifications),
    Request(F::IncomingRequests),
    Response(F::IncomingResponses),
}

impl<F: MessageFilter> TryFrom<AllMessages> for IncomingMessage<F> {
    type Error = InvalidMessageResponse;

    fn try_from(all_messages: AllMessages) -> Result<Self, Self::Error> {
        match all_messages {
            AllMessages::Requests(message) => message.try_into().map(IncomingMessage::Request),
            AllMessages::Responses(message) => message.try_into().map(IncomingMessage::Response),
            AllMessages::Notifications(message) => {
                message.try_into().map(IncomingMessage::Notification)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum OutgoingMessage<F: MessageFilter> {
    Notification(F::OutgoingNotifications),
    Request(F::OutgoingRequests),
    Response(F::OutgoingResponses),
}

impl<F: MessageFilter> From<OutgoingMessage<F>> for AllMessages {
    fn from(outgoing_message: OutgoingMessage<F>) -> Self {
        match outgoing_message {
            OutgoingMessage::Notification(message) => AllMessages::Notifications(message.into()),
            OutgoingMessage::Request(message) => AllMessages::Requests(message.into()),
            OutgoingMessage::Response(message) => AllMessages::Responses(message.into()),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use lsp_types::NumberOrString::Number;
    use once_cell::sync::Lazy;

    use crate::messages::{
        core::request::{tests::WILL_RENAME_FILES_REQUEST_MOCK, RequestMessage},
        groups::{
            notifications::tests::SomeNotificationsMock,
            requests::{tests::SomeRequestsMock, AllClientRequests::ShowDocument},
            responses::tests::SomeResponsesMock,
        },
    };

    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct FilterMock;

    impl MessageFilter for FilterMock {
        type OutgoingNotifications = SomeNotificationsMock;
        type OutgoingRequests = SomeRequestsMock;
        type OutgoingResponses = SomeResponsesMock;
        type IncomingNotifications = SomeNotificationsMock;
        type IncomingRequests = SomeRequestsMock;
        type IncomingResponses = SomeResponsesMock;
    }

    pub const OUTGOING_MESSAGE_MOCK: OutgoingMessage<FilterMock> =
        OutgoingMessage::<FilterMock>::Request(SomeRequestsMock::WillRenameFiles(
            WILL_RENAME_FILES_REQUEST_MOCK,
        ));

    pub const INCOMING_MESSAGE_MOCK: IncomingMessage<FilterMock> =
        IncomingMessage::<FilterMock>::Request(SomeRequestsMock::WillRenameFiles(
            WILL_RENAME_FILES_REQUEST_MOCK,
        ));

    pub static INVALID_INCOMING_MOCK: Lazy<AllMessages> = Lazy::new(|| {
        AllMessages::Requests(AllRequests::Client(ShowDocument(RequestMessage {
            id: Number(1).into(),
            params: Some(lsp_types::ShowDocumentParams {
                uri: "www.google.com".parse().unwrap(),
                external: None,
                take_focus: None,
                selection: None,
            }),
        })))
    });
}
