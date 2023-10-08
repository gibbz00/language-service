use std::io::Write;

use bytes::{BufMut, BytesMut};
use derive_more::From;
use tokio_util::codec::Encoder;

use crate::messages::{groups::MessageGroup, payload::Payload};

use super::LanguageServerCodec;

#[derive(Debug, From)]
pub enum EncodeError {
    Serialize(serde_json::Error),
    Io(std::io::Error),
}

impl<M: MessageGroup> Encoder<M> for LanguageServerCodec<M> {
    type Error = EncodeError;

    fn encode(&mut self, item: M, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let protocol_message_str = Payload::try_new(item)?.to_string();
        dst.reserve(protocol_message_str.len());
        let mut writer = dst.writer();
        write!(writer, "{}", protocol_message_str)?;
        writer.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::groups::{tests::MESSAGE_MOCK, AllMessages};

    use super::*;

    #[test]
    fn encodes_message() {
        let mut language_server_codec = LanguageServerCodec::<AllMessages>::default();
        let mut message_buffer = BytesMut::new();
        language_server_codec
            .encode(MESSAGE_MOCK, &mut message_buffer)
            .unwrap();

        assert_eq!(
            &Payload::try_new(MESSAGE_MOCK).unwrap().to_string(),
            std::str::from_utf8(&message_buffer).unwrap()
        )
    }
}
