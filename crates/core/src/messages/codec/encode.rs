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
        let payload = Payload::new(&item).to_string();
        if dst.capacity() < payload.len() {
            dst.reserve(payload.len() - dst.capacity());
        }
        let mut writer = dst.writer();
        write!(writer, "{}", payload)?;
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
        let mut payload_buffer = BytesMut::new();
        language_server_codec
            .encode(MESSAGE_MOCK, &mut payload_buffer)
            .unwrap();

        assert_eq!(
            &Payload::new(&MESSAGE_MOCK).to_string(),
            std::str::from_utf8(&payload_buffer).unwrap()
        )
    }
}
