use lsp_types::request::Request;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use self::response_error::ResponseError;

use super::{version::Version, Message};

#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResponseId {
    Number(i32),
    String(String),
    /// While `null` is considered a valid request ID by the JSON-RPC 2.0 specification, its use is
    /// _strongly_ discouraged because the specification also uses a `null` value to indicate an
    /// unknown ID in the [`Response`] object.
    Null,
}

pub struct ResponseMessage<R: Request> {
    id: ResponseId,
    kind: Result<R::Result, ResponseError>,
}

impl<R: Request> Message for ResponseMessage<R> {}

impl<R: Request> Serialize for ResponseMessage<R> {
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

impl<'de, R: Request> Deserialize<'de> for ResponseMessage<R> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ResponseMessageDom<R: Request> {
            #[serde(rename = "jsonrpc")]
            _jsonrpc: Version,
            id: ResponseId,
            #[serde(flatten, bound = "R: Request")]
            kind: ResultDom<R>,
        }

        #[derive(Deserialize)]
        enum ResultDom<R: Request> {
            #[serde(rename = "result")]
            Ok(R::Result),
            #[serde(rename = "error")]
            Error(ResponseError),
        }

        let response_messarge_dom = ResponseMessageDom::<R>::deserialize(deserializer)?;

        Ok(ResponseMessage {
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

    impl<R: Request> std::fmt::Debug for ResponseMessage<R> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut debug_struct = f.debug_struct("ResponseMessage");
            debug_struct
                .field("jsonrpc", &Version)
                .field("id", &self.id);
            match &self.kind {
                Ok(value) => debug_struct.field("result", &serde_json::to_string(value).unwrap()),
                Err(err) => debug_struct.field("error", err),
            };

            debug_struct.finish()
        }
    }

    impl<R: Request> PartialEq for ResponseMessage<R> {
        fn eq(&self, other: &Self) -> bool {
            const SERIALIZE_ERROR_MESSAGE: &str = "Kind should be serializable into Value.";
            serde_json::to_value(&self.kind).expect(SERIALIZE_ERROR_MESSAGE)
                == serde_json::to_value(&other.kind).expect(SERIALIZE_ERROR_MESSAGE)
                && self.id == other.id
        }
    }

    pub const SHUTDOWN_RESPONSE_MOCK: ResponseMessage<Shutdown> = ResponseMessage {
        id: ResponseId::Number(0),
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
    fn serializes_request_message() {
        assert_eq!(
            *SHUTDOWN_RESPONSE_JSON,
            serde_json::to_value(SHUTDOWN_RESPONSE_MOCK).unwrap()
        )
    }

    #[test]
    fn deserializes_request_message() {
        assert_eq!(
            SHUTDOWN_RESPONSE_MOCK,
            serde_json::from_value::<ResponseMessage<Shutdown>>(SHUTDOWN_RESPONSE_JSON.clone(),)
                .unwrap()
        )
    }
}

mod response_error {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use serde_repr::{Deserialize_repr, Serialize_repr};
    use strum::FromRepr;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ResponseError {
        pub code: ResponseErrorCode,
        pub message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Value>,
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

    #[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, FromRepr)]
    #[repr(i64)]
    pub enum ReservedResponseErrorCodes {
        ParseError = json_rpc_error_codes::PARSE_ERROR,
        InvalidRequest = json_rpc_error_codes::INVALID_REQUEST,
        MethodNotFound = json_rpc_error_codes::METHOD_NOT_FOUND,
        InvalidParams = json_rpc_error_codes::INVALID_PARAMS,
        InternalError = json_rpc_error_codes::INTERNAL_ERROR,
        RequestFailed = lsp_types::error_codes::REQUEST_FAILED,
        RequestCancelled = lsp_types::error_codes::REQUEST_CANCELLED,
        ContentModified = lsp_types::error_codes::SERVER_CANCELLED,
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
