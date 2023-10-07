use std::io::Write;

use bytes::{BufMut, BytesMut};
use derive_more::From;
use tokio_util::codec::Encoder;

use crate::groups::MessageGroup;

use self::protocol_message::ProtocolMessage;

use super::LanguageServerCodec;

#[derive(Debug, From)]
pub enum EncodeError {
    Serialize(serde_json::Error),
    Io(std::io::Error),
}

impl<M: MessageGroup> Encoder<M> for LanguageServerCodec<M> {
    type Error = EncodeError;

    fn encode(&mut self, item: M, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let protocol_message_str = ProtocolMessage::try_new(item)?.to_string();
        dst.reserve(protocol_message_str.len());
        let mut writer = dst.writer();
        write!(writer, "{}", protocol_message_str)?;
        writer.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::groups::tests::{MockAgentMessage, AGENT_MESSAGE_MOCK};

    use super::*;

    #[test]
    fn encodes_message() {
        let mut language_server_codec = LanguageServerCodec::<MockAgentMessage>::default();
        let mut message_buffer = BytesMut::new();
        language_server_codec
            .encode(AGENT_MESSAGE_MOCK, &mut message_buffer)
            .unwrap();

        assert_eq!(
            &ProtocolMessage::try_new(AGENT_MESSAGE_MOCK)
                .unwrap()
                .to_string(),
            std::str::from_utf8(&message_buffer).unwrap()
        )
    }
}

pub(crate) mod protocol_message {
    use std::fmt::Display;

    use crate::{codec::headers::JsonRpcHeaders, groups::MessageGroup};

    pub struct ProtocolMessage {
        pub header: JsonRpcHeaders,
        pub body: String,
    }

    impl ProtocolMessage {
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

    impl Display for ProtocolMessage {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}\r\n{}", self.header, self.body)
        }
    }

    #[cfg(test)]
    pub mod tests {
        use once_cell::sync::Lazy;

        use crate::groups::tests::AGENT_MESSAGE_MOCK;

        use super::*;

        pub static PROTOCOL_MESSAGE: Lazy<String> = Lazy::new(|| {
            ProtocolMessage::try_new(AGENT_MESSAGE_MOCK)
                .unwrap()
                .to_string()
        });

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
}
