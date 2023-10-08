pub(crate) mod headers;

use std::fmt::Display;

use crate::messages::groups::MessageGroup;

use self::headers::JsonRpcHeaders;

pub struct Payload {
    pub header: JsonRpcHeaders,
    pub body: String,
}

impl Payload {
    pub fn try_new(message: impl MessageGroup) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_string(&message)?;
        Ok(Self {
            header: JsonRpcHeaders {
                content_length: body.len(),
            },
            body,
        })
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

    use crate::messages::groups::tests::AGENT_MESSAGE_MOCK;

    use super::*;

    pub static PROTOCOL_MESSAGE: Lazy<String> =
        Lazy::new(|| Payload::try_new(AGENT_MESSAGE_MOCK).unwrap().to_string());

    #[test]
    fn displays_protocol_message() {
        let body_string = serde_json::to_string(&AGENT_MESSAGE_MOCK).unwrap();
        let expected_string = format!(
            "{}\r\n{}",
            JsonRpcHeaders {
                content_length: body_string.len()
            },
            body_string
        );

        assert_eq!(expected_string, *PROTOCOL_MESSAGE);
    }
}
