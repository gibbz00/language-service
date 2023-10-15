use lsp_types::request::*;
use serde::{Deserialize, Serialize};

use crate::messages::core::response::{LspResponse, ResponseId, ResponseMessage};

use self::errors::ErrorResponse;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllResponses {
    ResponseError(ResponseMessage<ErrorResponse>),
    // TODO:
    // Client(AllClientResponses),
    Server(AllServerResponses),
}

impl LspResponse for AllResponses {
    fn response_id(&self) -> &ResponseId {
        match self {
            AllResponses::ResponseError(response) => response.response_id(),
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
    use crate::messages::{
        codec::DecodeError,
        core::response::{
            response_error::{ReservedResponseErrorCodes, ResponseError, ResponseErrorCode},
            ResponseId, ResponseMessage,
        },
        groups::AllMessages,
    };

    use super::AllResponses;

    // Arbitrary generic parameter since incoming doesn't have/couldn't be concluded.
    pub struct ErrorResponse;

    impl lsp_types::request::Request for ErrorResponse {
        type Params = ();
        type Result = ();
        const METHOD: &'static str = "";
    }

    impl From<ResponseMessage<ErrorResponse>> for AllMessages {
        fn from(error_response: ResponseMessage<ErrorResponse>) -> Self {
            AllMessages::Responses(AllResponses::ResponseError(error_response))
        }
    }

    pub struct DecodeErrorResponse;
    impl DecodeErrorResponse {
        pub fn create(decode_error: DecodeError) -> ResponseMessage<ErrorResponse> {
            ResponseMessage {
                id: ResponseId::Null,
                kind: Err(ResponseError {
                    code: ResponseErrorCode::Reserved(ReservedResponseErrorCodes::ParseError),
                    message: decode_error.to_string(),
                    data: None,
                }),
            }
        }
    }

    pub struct InvalidMessageResponse;
    impl InvalidMessageResponse {
        pub fn create(
            id: ResponseId,
            response_error: ResponseError,
        ) -> ResponseMessage<ErrorResponse> {
            ResponseMessage {
                id,
                kind: Err(response_error),
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
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
        type Error = AllResponses;

        fn try_from(all_responses: AllResponses) -> Result<Self, Self::Error> {
            match all_responses {
                AllResponses::Server(AllServerResponses::Shutdown(response)) => {
                    Ok(SomeResponsesMock::Shutdown(response))
                }
                response => Err(response),
            }
        }
    }
}
