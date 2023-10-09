pub(crate) mod headers;

use std::fmt::Display;

use crate::messages::groups::MessageGroup;

use self::headers::JsonRpcHeaders;

pub struct Payload {
    header: JsonRpcHeaders,
    body: String,
}

impl Payload {
    pub fn new(message: impl MessageGroup) -> Self {
        let body = serde_json::to_string(&message).expect("unexpected serialization failure");
        Self {
            header: JsonRpcHeaders {
                content_length: body.len(),
            },
            body,
        }
    }
}

impl Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\r\n{}", self.header, self.body)
    }
}

#[cfg(test)]
pub mod tests {
    use once_cell::sync::Lazy;

    use crate::messages::groups::tests::MESSAGE_MOCK;

    use super::*;

    pub static PAYLOAD_STR_MOCK: Lazy<String> =
        Lazy::new(|| Payload::new(MESSAGE_MOCK).to_string());

    pub static INVALID_PAYLOAD_STR_MOCK: Lazy<String> = Lazy::new(|| {
        let body = serde_json::json!(
            {
                "name": 10
            }
        )
        .to_string();

        Payload {
            header: JsonRpcHeaders {
                content_length: body.len(),
            },
            body,
        }
        .to_string()
    });

    #[test]
    fn displays_protocol_message() {
        let body_string = serde_json::to_string(&MESSAGE_MOCK).unwrap();
        let expected_string = format!(
            "{}\r\n{}",
            JsonRpcHeaders {
                content_length: body_string.len()
            },
            body_string
        );

        assert_eq!(expected_string, *PAYLOAD_STR_MOCK);
    }
}
