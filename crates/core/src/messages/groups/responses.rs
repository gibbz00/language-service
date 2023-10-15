use lsp_types::request::*;
use serde::{Deserialize, Serialize};

use crate::messages::core::response::{LspResponse, ResponseId, ResponseMessage};

use self::errors::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllResponses {
    ResponseErrors(ResponseErrors),
    // TODO:
    // Client(AllClientResponses),
    Server(AllServerResponses),
}

impl LspResponse for AllResponses {
    fn response_id(&self) -> &ResponseId {
        match self {
            AllResponses::ResponseErrors(response) => response.response_id(),
            AllResponses::Server(response) => response.response_id(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllServerResponses {
    Shutdown(ResponseMessage<Shutdown>),
}

impl LspResponse for AllServerResponses {
    fn response_id(&self) -> &ResponseId {
        match self {
            AllServerResponses::Shutdown(response) => response.response_id(),
        }
    }
}

pub mod errors {
    use derive_more::{Deref, From};
    use lsp_types::request::ShowMessageRequest;
    use serde::{Deserialize, Serialize};

    use crate::messages::{
        codec::DecodeError,
        core::{
            request::RequestId,
            response::{
                response_error::{ReservedResponseErrorCodes, ResponseError, ResponseErrorCode},
                LspResponse, ResponseId, ResponseMessage,
            },
        },
        groups::AllMessages,
    };

    use super::AllResponses;

    #[derive(Debug, PartialEq, Serialize, Deserialize, From)]
    #[serde(untagged)]
    pub enum ResponseErrors {
        Decode(DecodeErrorResponse),
        Internal(InternalErrorResponse),
        InvalidMessage(InvalidMessageResponse),
    }

    impl LspResponse for ResponseErrors {
        fn response_id(&self) -> &ResponseId {
            match self {
                ResponseErrors::Decode(response) => response.response_id(),
                ResponseErrors::Internal(response) => response.response_id(),
                ResponseErrors::InvalidMessage(response) => response.response_id(),
            }
        }
    }

    impl From<ResponseErrors> for AllMessages {
        fn from(response_errors: ResponseErrors) -> Self {
            AllMessages::Responses(AllResponses::ResponseErrors(response_errors))
        }
    }

    // Arbitrary generic parameter since incoming couldn't be concluded.
    type UnknownRequest = ShowMessageRequest;

    #[derive(Debug, PartialEq, Serialize, Deserialize, Deref)]
    #[serde(transparent)]
    pub struct DecodeErrorResponse(ResponseMessage<UnknownRequest>);
    impl DecodeErrorResponse {
        pub fn new(decode_error: DecodeError) -> Self {
            Self(ResponseMessage {
                id: ResponseId::Null,
                kind: Err(ResponseError {
                    code: ResponseErrorCode::Reserved(ReservedResponseErrorCodes::ParseError),
                    message: decode_error.to_string(),
                    data: None,
                }),
            })
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Deref)]
    #[serde(transparent)]
    pub struct InternalErrorResponse(ResponseMessage<UnknownRequest>);
    impl InternalErrorResponse {
        pub fn new(request_id: Option<RequestId>, message: impl Into<String>) -> Self {
            Self(ResponseMessage {
                id: request_id.map(ResponseId::from).unwrap_or(ResponseId::Null),
                kind: Err(ResponseError {
                    code: ResponseErrorCode::Reserved(ReservedResponseErrorCodes::InternalError),
                    message: message.into(),
                    data: None,
                }),
            })
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Deref)]
    #[serde(transparent)]
    pub struct InvalidMessageResponse(ResponseMessage<UnknownRequest>);
    impl InvalidMessageResponse {
        pub fn new(response_id: Option<ResponseId>, response_error: ResponseError) -> Self {
            Self(ResponseMessage {
                id: response_id.unwrap_or(ResponseId::Null),
                kind: Err(response_error),
            })
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::messages::{
        core::response::response_error::{
            ReservedResponseErrorCodes, ResponseError, ResponseErrorCode::Reserved,
        },
        groups::responses::errors::InvalidMessageResponse,
    };

    use super::*;

    #[derive(Debug, PartialEq)]
    pub enum SomeResponsesMock {
        Shutdown(ResponseMessage<Shutdown>),
    }

    impl From<SomeResponsesMock> for AllResponses {
        fn from(some_responses: SomeResponsesMock) -> Self {
            match some_responses {
                SomeResponsesMock::Shutdown(response) => {
                    AllResponses::Server(AllServerResponses::Shutdown(response))
                }
            }
        }
    }

    impl TryFrom<AllResponses> for SomeResponsesMock {
        type Error = InvalidMessageResponse;

        fn try_from(all_responses: AllResponses) -> Result<Self, Self::Error> {
            match all_responses {
                AllResponses::Server(AllServerResponses::Shutdown(response)) => {
                    Ok(SomeResponsesMock::Shutdown(response))
                }
                response => Err(InvalidMessageResponse::new(
                    Some(response.response_id().clone()),
                    ResponseError {
                        code: Reserved(ReservedResponseErrorCodes::InternalError),
                        message: "Invalid response.".to_string(),
                        data: Some(
                            serde_json::to_value(response).expect("response not serializable"),
                        ),
                    },
                )),
            }
        }
    }
}
