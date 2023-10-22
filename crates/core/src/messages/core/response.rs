use lsp_types::{request::Request, NumberOrString};
use serde::{ser::SerializeMap, Deserialize, Serialize};

use self::response_error::ResponseError;

use super::{request::RequestId, version::Version};

const SERIALIZE_ERROR: &str = "unable to serialize type into serde_json::Value.";

pub trait LspResponse {
    fn response_id(&self) -> &ResponseId;
    fn untyped(self) -> UntypedResponseMessage;
}

impl<R: Request> LspResponse for ResponseMessage<R> {
    fn response_id(&self) -> &ResponseId {
        &self.id
    }

    fn untyped(self) -> UntypedResponseMessage {
        self.into()
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResponseId {
    NumberOrString(NumberOrString),
    /// While `null` is considered a valid request ID by the JSON-RPC 2.0 specification, its use is
    /// _strongly_ discouraged because the specification also uses a `null` value to indicate an
    /// unknown ID in the [`Response`] object.
    Null,
}

impl From<RequestId> for ResponseId {
    fn from(request_id: RequestId) -> Self {
        ResponseId::NumberOrString(request_id.into())
    }
}

#[derive(Debug)]
pub struct ResponseMessage<R: Request> {
    pub id: ResponseId,
    pub kind: Result<R::Result, ResponseError>,
}

impl<R: Request> PartialEq for ResponseMessage<R> {
    fn eq(&self, other: &Self) -> bool {
        serde_json::to_value(&self.kind).expect(SERIALIZE_ERROR)
            == serde_json::to_value(&other.kind).expect(SERIALIZE_ERROR)
            && self.id == other.id
    }
}

#[derive(Debug, PartialEq)]
pub struct UntypedResponseMessage {
    pub id: ResponseId,
    pub kind: Result<serde_json::Value, ResponseError>,
}

impl<R: Request> From<ResponseMessage<R>> for UntypedResponseMessage {
    fn from(response_message: ResponseMessage<R>) -> Self {
        Self {
            id: response_message.id,
            kind: response_message
                .kind
                .map(|ok| serde_json::to_value(ok).expect(SERIALIZE_ERROR)),
        }
    }
}

impl<R: Request> TryFrom<UntypedResponseMessage> for ResponseMessage<R> {
    type Error = serde_json::Error;

    fn try_from(untyped: UntypedResponseMessage) -> Result<Self, Self::Error> {
        Ok(ResponseMessage::<R> {
            id: untyped.id,
            kind: match untyped.kind {
                Ok(value) => serde_json::from_value(value)?,
                Err(err) => Err(err),
            },
        })
    }
}

impl Serialize for UntypedResponseMessage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut message_map = serializer.serialize_map(Some(3))?;
        message_map.serialize_entry("jsonrpc", &Version)?;
        message_map.serialize_entry("id", &self.id)?;
        match &self.kind {
            Ok(value) => message_map.serialize_entry("result", value)?,
            Err(err) => message_map.serialize_entry("error", err)?,
        }
        message_map.end()
    }
}

impl<'de> Deserialize<'de> for UntypedResponseMessage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ResponseMessageDom {
            #[serde(rename = "jsonrpc")]
            _jsonrpc: Version,
            id: ResponseId,
            kind: ResultDom,
        }

        #[derive(Deserialize)]
        enum ResultDom {
            #[serde(rename = "result")]
            Ok(serde_json::Value),
            #[serde(rename = "error")]
            Error(ResponseError),
        }

        let response_messarge_dom = ResponseMessageDom::deserialize(deserializer)?;

        Ok(UntypedResponseMessage {
            id: response_messarge_dom.id,
            kind: match response_messarge_dom.kind {
                ResultDom::Ok(value) => Ok(value),
                ResultDom::Error(err) => Err(err),
            },
        })
    }
}

#[cfg(test)]
pub mod tests {
    use lsp_types::request::Shutdown;
    use once_cell::sync::Lazy;
    use serde_json::json;

    use super::*;

    pub const SHUTDOWN_RESPONSE_MOCK: ResponseMessage<Shutdown> = ResponseMessage {
        id: ResponseId::NumberOrString(NumberOrString::Number(0)),
        kind: Ok(()),
    };

    static SHUTDOWN_RESPONSE_JSON: Lazy<serde_json::Value> = Lazy::new(|| {
        json!({
            "jsonrpc": "2.0",
            "id": 0,
            "result": null
        })
    });

    #[test]
    fn serializes_response_message() {
        assert_eq!(
            *SHUTDOWN_RESPONSE_JSON,
            serde_json::to_value(UntypedResponseMessage::from(SHUTDOWN_RESPONSE_MOCK)).unwrap()
        )
    }

    #[test]
    fn deserializes_response_message() {
        assert_eq!(
            SHUTDOWN_RESPONSE_MOCK,
            serde_json::from_value::<UntypedResponseMessage>(SHUTDOWN_RESPONSE_JSON.clone(),)
                .unwrap()
                .try_into()
                .unwrap()
        )
    }
}

pub mod response_error {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use serde_repr::{Deserialize_repr, Serialize_repr};
    use strum::FromRepr;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ResponseError {
        pub code: ResponseErrorCode,
        pub message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Value>,
    }

    #[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, FromRepr)]
    #[repr(i64)]
    pub enum ReservedResponseErrorCodes {
        ParseError = json_rpc_error_codes::PARSE_ERROR,
        InvalidRequest = json_rpc_error_codes::INVALID_REQUEST,
        MethodNotFound = json_rpc_error_codes::METHOD_NOT_FOUND,
        InvalidParams = json_rpc_error_codes::INVALID_PARAMS,
        InternalError = json_rpc_error_codes::INTERNAL_ERROR,
        ServerNotInitialized = lsp_types::error_codes::SERVER_NOT_INITIALIZED,
        UnknownErrorCode = lsp_types::error_codes::UNKNOWN_ERROR_CODE,
        RequestFailed = lsp_types::error_codes::REQUEST_FAILED,
        RequestCancelled = lsp_types::error_codes::REQUEST_CANCELLED,
        ContentModified = lsp_types::error_codes::SERVER_CANCELLED,
    }

    #[derive(Debug, PartialEq)]
    pub enum ResponseErrorCode {
        Reserved(ReservedResponseErrorCodes),
        Other(i64),
    }

    impl Serialize for ResponseErrorCode {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                ResponseErrorCode::Reserved(reserved_code) => reserved_code.serialize(serializer),
                ResponseErrorCode::Other(other) => serializer.serialize_i64(*other),
            }
        }
    }

    impl<'de> Deserialize<'de> for ResponseErrorCode {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            i64::deserialize(deserializer).map(|code| {
                ReservedResponseErrorCodes::from_repr(code)
                    .map(Self::Reserved)
                    .unwrap_or(Self::Other(code))
            })
        }
    }

    mod json_rpc_error_codes {
        pub const PARSE_ERROR: i64 = -32700;
        pub const INVALID_REQUEST: i64 = -32600;
        pub const METHOD_NOT_FOUND: i64 = -32601;
        pub const INVALID_PARAMS: i64 = -32602;
        pub const INTERNAL_ERROR: i64 = -32603;
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn serializes_reserved_response_error_code_as_i64() {
            assert_eq!(
                json_rpc_error_codes::PARSE_ERROR.to_string(),
                serde_json::to_string(&ResponseErrorCode::Reserved(
                    ReservedResponseErrorCodes::ParseError
                ))
                .unwrap()
            )
        }

        #[test]
        fn serializes_other_response_error_code_as_i64() {
            const OTHER_CODE: i64 = -123;
            assert_eq!(
                OTHER_CODE.to_string(),
                serde_json::to_string(&ResponseErrorCode::Other(OTHER_CODE)).unwrap()
            )
        }

        #[test]
        fn deserializes_reserved_response_error_code_from_i64() {
            assert_eq!(
                ResponseErrorCode::Reserved(ReservedResponseErrorCodes::ParseError),
                serde_json::from_str(&json_rpc_error_codes::PARSE_ERROR.to_string()).unwrap()
            )
        }

        #[test]
        fn deserializes_other_response_error_code_from_i64() {
            const OTHER_CODE: i64 = -123;
            assert_eq!(
                ResponseErrorCode::Other(OTHER_CODE),
                serde_json::from_str(&OTHER_CODE.to_string()).unwrap()
            )
        }
    }
}
