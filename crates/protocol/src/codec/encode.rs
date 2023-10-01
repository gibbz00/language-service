use std::io::Write;

use bytes::{BufMut, BytesMut};
use derive_more::From;
use tokio_util::codec::Encoder;

use crate::messages::Message;

use self::protocol_message::ProtocolMessage;

use super::LanguageServerCodec;

#[derive(From)]
pub enum EncodeError {
    Serialize(serde_json::Error),
    Io(std::io::Error),
}

impl<M: Message> Encoder<M> for LanguageServerCodec<M> {
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

mod protocol_message {
    use std::fmt::Display;

    use crate::{codec::headers::JsonRpcHeaders, messages::Message};

    pub struct ProtocolMessage {
        pub header: JsonRpcHeaders,
        pub body: String,
    }

    impl ProtocolMessage {
        pub fn try_new(message: impl Message) -> Result<Self, serde_json::Error> {
            let body = serde_json::to_string(&message)?;
            Ok(Self {
                header: JsonRpcHeaders::new(body.len()),
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
    mod tests {
        use crate::messages::response::tests::SHUTDOWN_RESPONSE_MOCK;

        use super::*;

        #[test]
        fn displays_protocol_message() {
            let body_string = serde_json::to_string(&SHUTDOWN_RESPONSE_MOCK).unwrap();
            let expected_string = format!(
                "{}\r\n{}",
                JsonRpcHeaders::new(body_string.len()),
                body_string
            );

            assert_eq!(
                expected_string,
                ProtocolMessage::try_new(SHUTDOWN_RESPONSE_MOCK)
                    .unwrap()
                    .to_string()
            );
        }
    }
}
