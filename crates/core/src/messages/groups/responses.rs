pub mod errors {
    use crate::messages::{
        codec::DecodeError,
        core::response::{
            response_error::{ReservedResponseErrorCodes, ResponseError, ResponseErrorCode},
            ResponseId, ResponseMessage,
        },
        groups::AllMessages,
    };

    // Arbitrary generic parameter since incoming doesn't have/couldn't be concluded.
    #[derive(Debug)]
    pub struct ErrorResponse;

    impl lsp_types::request::Request for ErrorResponse {
        type Params = ();
        type Result = ();
        const METHOD: &'static str = "";
    }

    impl From<ResponseMessage<ErrorResponse>> for AllMessages {
        fn from(error_response: ResponseMessage<ErrorResponse>) -> Self {
            AllMessages::UntypedResponse(error_response.into())
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
    use lsp_types::request::Shutdown;

    use crate::messages::core::response::{
        LspResponse, ResponseId, ResponseMessage, UntypedResponseMessage,
    };

    #[derive(Debug, PartialEq)]
    pub enum SomeResponsesMock {
        Shutdown(ResponseMessage<Shutdown>),
    }

    impl LspResponse for SomeResponsesMock {
        fn response_id(&self) -> &ResponseId {
            match self {
                SomeResponsesMock::Shutdown(response) => response.response_id(),
            }
        }

        fn untyped(self) -> UntypedResponseMessage {
            match self {
                SomeResponsesMock::Shutdown(response) => response.untyped(),
            }
        }
    }
}
