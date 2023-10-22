use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    messages::{
        core::{
            response::{
                response_error::{
                    ReservedResponseErrorCodes, ResponseError, ResponseErrorCode::Reserved,
                },
                LspResponse, ResponseId, ResponseMessage, UntypedResponseMessage,
            },
            LspRequest,
        },
        groups::{
            notifications::AllNotifications,
            requests::AllRequests,
            responses::errors::{DecodeErrorResponse, ErrorResponse, InvalidMessageResponse},
            AllMessages,
        },
    },
    service::error::BACKEND_OUTPUT_CLOSED,
};

use super::error::{BACKEND_INPUT_CLOSED, FRONTEND_INPUT_CLOSED, FRONTEND_OUTPUT_CLOSED};

pub(crate) struct ServiceMessageFilter<F: MessageFilter> {
    frontend_rx: UnboundedReceiver<AllMessages>,
    frontend_tx: UnboundedSender<AllMessages>,
    backend_rx: UnboundedReceiver<OutgoingMessage<F>>,
    backend_tx: UnboundedSender<IncomingMessage<F>>,
    type_store: F::TypeStore,
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
            type_store: F::TypeStore::new(),
        }
    }

    pub fn tick(&mut self) {
        self.try_forward_to_backend();
        self.forward_to_frontend();
    }

    pub fn forward_to_frontend(&mut self) {
        if let Ok(message_result) = self.backend_rx.try_next() {
            let message = message_result.expect(BACKEND_OUTPUT_CLOSED);
            if let OutgoingMessage::Request(outgoing_request) = &message {
                self.type_store.store_request_type(outgoing_request)
            }

            self.frontend_tx
                .unbounded_send(message.into())
                .expect(FRONTEND_INPUT_CLOSED)
        }
    }

    pub fn try_forward_to_backend(&mut self) {
        if let Ok(message_result) = self.frontend_rx.try_next() {
            let message = message_result.expect(FRONTEND_OUTPUT_CLOSED);

            match self.typeset_incoming(message) {
                Ok(incoming_message) => self
                    .backend_tx
                    .unbounded_send(incoming_message)
                    .expect(BACKEND_INPUT_CLOSED),
                Err(try_from_err) => {
                    self.frontend_tx
                        .unbounded_send(try_from_err.into())
                        .expect(FRONTEND_INPUT_CLOSED);
                }
            }
        }
    }

    fn typeset_incoming(
        &mut self,
        all_messages: AllMessages,
    ) -> Result<IncomingMessage<F>, ResponseMessage<ErrorResponse>> {
        return match all_messages {
            AllMessages::Requests(message) => message
                .try_into()
                .map(IncomingMessage::Request)
                .map_err(|request: AllRequests| {
                    invalid_message::<F>(AllMessages::Requests(request))
                }),
            AllMessages::UntypedResponse(untyped_response) => self
                .type_store
                .load_response_type(untyped_response)
                .map(IncomingMessage::Response)
                .map_err(|parse_error: serde_json::Error| {
                    DecodeErrorResponse::create(parse_error.into())
                }),
            AllMessages::Notifications(message) => message
                .try_into()
                .map(IncomingMessage::Notification)
                .map_err(|notification| {
                    invalid_message::<F>(AllMessages::Notifications(notification))
                }),
        };

        fn invalid_message<F: MessageFilter>(
            message: AllMessages,
        ) -> ResponseMessage<ErrorResponse> {
            InvalidMessageResponse::create(
                match &message {
                    AllMessages::Requests(request) => request.request_id().clone().into(),
                    AllMessages::UntypedResponse(untyped_response) => untyped_response.id.clone(),
                    AllMessages::Notifications(_) => ResponseId::Null,
                },
                ResponseError {
                    code: Reserved(ReservedResponseErrorCodes::InternalError),
                    message: format!("invalid message for {:#?}", std::any::type_name::<F>()),
                    data: Some(serde_json::to_value(message).expect("message not serializable")),
                },
            )
        }
    }
}

pub trait TypeStore<F: MessageFilter> {
    fn new() -> Self;
    fn store_request_type(&mut self, outgoing_request: &F::OutgoingRequests);
    fn load_response_type(
        &mut self,
        untyped_response: UntypedResponseMessage,
    ) -> Result<F::IncomingResponses, serde_json::Error>;
}

pub trait ResponseTypingFn<F: MessageFilter> {
    fn typing_fn(
        &self,
    ) -> fn(UntypedResponseMessage) -> Result<F::IncomingResponses, serde_json::Error>;
}

pub trait MessageFilter: Sized {
    type OutgoingNotifications: Into<AllNotifications>;
    type OutgoingRequests: Into<AllRequests> + LspRequest + ResponseTypingFn<Self>;
    type OutgoingResponses: LspResponse;
    type IncomingNotifications: TryFrom<AllNotifications, Error = AllNotifications>;
    type IncomingRequests: TryFrom<AllRequests, Error = AllRequests>;
    type IncomingResponses;
    type TypeStore: TypeStore<Self>;
}

#[derive(Debug, PartialEq)]
pub enum IncomingMessage<F: MessageFilter> {
    Notification(F::IncomingNotifications),
    Request(F::IncomingRequests),
    Response(F::IncomingResponses),
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
            OutgoingMessage::Response(message) => AllMessages::UntypedResponse(message.untyped()),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use lsp_types::NumberOrString::Number;
    use once_cell::sync::Lazy;

    use crate::messages::{
        core::{
            request::{tests::SHUTDOWN_REQUEST_MOCK, RequestMessage},
            RequestId,
        },
        groups::{
            notifications::tests::SomeNotificationsMock,
            requests::{tests::SomeRequestsMock, AllClientRequests::ShowDocument},
            responses::tests::SomeResponsesMock,
        },
    };

    use super::*;

    pub struct TypeStoreMock {
        store: HashMap<
            RequestId,
            fn(UntypedResponseMessage) -> Result<SomeResponsesMock, serde_json::Error>,
        >,
    }

    impl TypeStore<FilterMock> for TypeStoreMock {
        fn new() -> Self {
            Self {
                store: HashMap::new(),
            }
        }

        fn store_request_type(
            &mut self,
            outgoing_request: &<FilterMock as MessageFilter>::OutgoingRequests,
        ) {
            self.store.insert(
                outgoing_request.request_id().clone(),
                outgoing_request.typing_fn(),
            );
        }

        fn load_response_type(
            &mut self,
            untyped_response: UntypedResponseMessage,
        ) -> Result<SomeResponsesMock, serde_json::Error> {
            match &untyped_response.id {
                ResponseId::NumberOrString(request_id) => {
                    let request_id = RequestId::from(request_id.clone());
                    // TEMP: unwrap
                    self.store.get(&request_id).unwrap()(untyped_response)
                }
                ResponseId::Null => {
                    // This is an error response not based on a notification
                    todo!()
                }
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct FilterMock;

    impl MessageFilter for FilterMock {
        type OutgoingNotifications = SomeNotificationsMock;
        type OutgoingRequests = SomeRequestsMock;
        type OutgoingResponses = SomeResponsesMock;
        type IncomingNotifications = SomeNotificationsMock;
        type IncomingRequests = SomeRequestsMock;
        type IncomingResponses = SomeResponsesMock;
        type TypeStore = TypeStoreMock;
    }

    pub const OUTGOING_MESSAGE_MOCK: OutgoingMessage<FilterMock> =
        OutgoingMessage::<FilterMock>::Request(SomeRequestsMock::ShutDown(SHUTDOWN_REQUEST_MOCK));

    pub const INCOMING_MESSAGE_MOCK: IncomingMessage<FilterMock> =
        IncomingMessage::<FilterMock>::Request(SomeRequestsMock::ShutDown(SHUTDOWN_REQUEST_MOCK));

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
