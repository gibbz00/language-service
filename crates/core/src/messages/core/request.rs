use derive_more::{Deref, From, Into};
use lsp_types::request::Request;
use lsp_types::NumberOrString;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use super::{
    response::{ResponseMessage, UntypedResponseMessage},
    version::Version,
};

pub trait LspRequest {
    fn request_id(&self) -> &RequestId;
}

impl<R: Request> LspRequest for RequestMessage<R> {
    fn request_id(&self) -> &RequestId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Deref, From, Into)]
#[serde(transparent)]
pub struct RequestId(NumberOrString);

pub struct RequestMessage<R: Request> {
    pub id: RequestId,
    pub params: Option<R::Params>,
}

impl<R: Request> RequestMessage<R> {
    pub fn response_typing_fn(
    ) -> fn(UntypedResponseMessage) -> Result<ResponseMessage<R>, serde_json::Error> {
        |untyped_response_message: UntypedResponseMessage| untyped_response_message.try_into()
    }
}

impl<R: Request> Serialize for RequestMessage<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut message_map = serializer.serialize_map(Some(4))?;
        message_map.serialize_entry("jsonrpc", &Version)?;
        message_map.serialize_entry("id", &self.id)?;
        message_map.serialize_entry("method", R::METHOD)?;
        if self.params.is_some() {
            message_map.serialize_entry("params", &self.params)?;
        }
        message_map.end()
    }
}

impl<'de, R: Request> Deserialize<'de> for RequestMessage<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RequestMessageDom<R: Request> {
            #[serde(rename = "jsonrpc")]
            _jsonrpc: Version,
            id: RequestId,
            method: String,
            params: Option<R::Params>,
        }

        let request_messarge_dom = RequestMessageDom::<R>::deserialize(deserializer)?;
        if request_messarge_dom.method != R::METHOD {
            return Err(serde::de::Error::unknown_variant(
                &request_messarge_dom.method,
                &[R::METHOD],
            ));
        }

        Ok(RequestMessage {
            id: request_messarge_dom.id,
            params: request_messarge_dom.params,
        })
    }
}

impl<R: Request> std::fmt::Debug for RequestMessage<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestMessage")
            .field("jsonrpc", &Version)
            .field("id", &self.id)
            .field("method", &R::METHOD)
            .field("params", &serde_json::to_string(&self.params).unwrap())
            .finish()
    }
}

impl<R: Request> PartialEq for RequestMessage<R> {
    fn eq(&self, other: &Self) -> bool {
        const SERIALIZE_ERROR_MESSAGE: &str = "Params should be serializable into Value.";
        serde_json::to_value(&self.params).expect(SERIALIZE_ERROR_MESSAGE)
            == serde_json::to_value(&other.params).expect(SERIALIZE_ERROR_MESSAGE)
            && self.id == other.id
    }
}

#[cfg(test)]
pub mod tests {
    use lsp_types::request::Shutdown;
    use once_cell::sync::Lazy;
    use serde_json::json;

    use super::*;

    pub const SHUTDOWN_REQUEST_MOCK: RequestMessage<Shutdown> = RequestMessage {
        id: RequestId(lsp_types::NumberOrString::Number(0)),
        params: None,
    };

    static SHUTDOWN_REQUEST_JSON: Lazy<serde_json::Value> = Lazy::new(|| {
        json!({
            "jsonrpc": "2.0",
            "id": 0,
            "method": "shutdown",
        })
    });

    #[test]
    fn serializes_request_message() {
        assert_eq!(
            *SHUTDOWN_REQUEST_JSON,
            serde_json::to_value(SHUTDOWN_REQUEST_MOCK).unwrap()
        )
    }

    #[test]
    fn deserializes_request_message() {
        assert_eq!(
            SHUTDOWN_REQUEST_MOCK,
            serde_json::from_value::<RequestMessage<Shutdown>>(SHUTDOWN_REQUEST_JSON.clone(),)
                .unwrap()
        )
    }
}
