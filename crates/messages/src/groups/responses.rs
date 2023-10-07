use serde::{Deserialize, Serialize};

use self::errors::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllResponses {
    // TODO:
    // Client(AllClientResponses),
    // Server(AllServerResponses),
    DecodeError(DecodeErrorResponse),
    InternalError(InternalErrorResponse),
}

pub mod errors {
    use derive_more::From;
    use lsp_types::request::ShowMessageRequest;
    use serde::{Deserialize, Serialize};

    use crate::{
        codec::DecodeError,
        core::{
            request::RequestId,
            response::{
                response_error::{ReservedResponseErrorCodes, ResponseError, ResponseErrorCode},
                ResponseId, ResponseMessage,
            },
        },
        groups::AllMessages,
    };

    use super::AllResponses;

    #[derive(From)]
    pub enum ResponseErrors {
        Decode(DecodeErrorResponse),
        Internal(InternalErrorResponse),
    }

    impl ResponseErrors {
        pub fn is_recoverable(&self) -> bool {
            match self {
                ResponseErrors::Decode(_) => true,
                ResponseErrors::Internal(_) => false,
            }
        }
    }

    impl From<ResponseErrors> for AllMessages {
        fn from(errors: ResponseErrors) -> Self {
            match errors {
                ResponseErrors::Decode(err) => err.into(),
                ResponseErrors::Internal(err) => err.into(),
            }
        }
    }

    // Arbitrary generic parameter since incoming couldn't be concluded.
    type UnknownRequest = ShowMessageRequest;

    #[derive(Debug, Serialize, Deserialize)]
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

    impl From<DecodeErrorResponse> for AllMessages {
        fn from(decode_error_response: DecodeErrorResponse) -> Self {
            AllMessages::Responses(AllResponses::DecodeError(decode_error_response))
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
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

    impl From<InternalErrorResponse> for AllMessages {
        fn from(internal_error_response: InternalErrorResponse) -> Self {
            AllMessages::Responses(AllResponses::InternalError(internal_error_response))
        }
    }
}
