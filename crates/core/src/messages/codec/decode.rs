use bytes::{Buf, BytesMut};
use derive_more::{Display, From};
use tokio_util::codec::Decoder;

use crate::messages::{
    groups::MessageGroup,
    payload::headers::{HeadersParseError, JsonRpcHeaders},
};

use super::LanguageServerCodec;

#[derive(Debug, Display, From)]
pub enum DecodeError {
    Io(std::io::Error),
    Httparse(httparse::Error),
    HeadersParseError(HeadersParseError),
    Deserialize(serde_json::Error),
}

impl<M: MessageGroup> Decoder for LanguageServerCodec<M> {
    type Item = M;
    type Error = DecodeError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.known_content_length {
            None => {
                let mut headers_buffer = [httparse::EMPTY_HEADER; 2];
                match httparse::parse_headers(src, &mut headers_buffer)? {
                    httparse::Status::Partial => Ok(None),
                    httparse::Status::Complete((parsed_src_index, headers)) => {
                        match JsonRpcHeaders::try_from(headers) {
                            Ok(json_rpc_headers) => {
                                let content_length = json_rpc_headers.content_length;
                                self.known_content_length = Some(content_length);
                                src.advance(parsed_src_index);

                                let missing_capacity =
                                    content_length.saturating_sub(src.capacity());
                                if missing_capacity > 0 {
                                    src.reserve(missing_capacity)
                                }

                                self.decode(src)
                            }
                            Err(err) => match err {
                                HeadersParseError::MissingContentLength => Ok(None),
                                _ => Err(DecodeError::HeadersParseError(err)),
                            },
                        }
                    }
                }
            }
            Some(content_length) => {
                if src.len() < content_length {
                    return Ok(None);
                }

                if src.len() > content_length {
                    // TODO: handle situation when buffer contains more than content length?
                    unimplemented!()
                }

                let message = serde_json::de::from_slice::<M>(src)?;
                src.clear();
                self.known_content_length = None;
                Ok(Some(message))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use crate::messages::{
        groups::{tests::MESSAGE_MOCK, AllMessages},
        payload::{
            headers::{CONTENT_TYPE_HEADER_NAME, JSON_RPC_CONTENT_TYPE},
            tests::PAYLOAD_STR_MOCK,
        },
    };

    use super::*;

    #[test]
    fn decodes_messages() {
        let mut message_bytes = BytesMut::new();
        let mut codec = LanguageServerCodec::<AllMessages>::default();
        decode_message(&mut message_bytes, &mut codec);
        decode_message(&mut message_bytes, &mut codec);

        fn decode_message(
            message_bytes: &mut BytesMut,
            codec: &mut LanguageServerCodec<AllMessages>,
        ) {
            message_bytes.put(PAYLOAD_STR_MOCK.as_bytes());
            assert_eq!(MESSAGE_MOCK, codec.decode(message_bytes).unwrap().unwrap())
        }
    }

    #[test]
    fn ok_on_missing_content_length_header() {
        let mut message_bytes = BytesMut::from(
            format!("{}: {}", CONTENT_TYPE_HEADER_NAME, JSON_RPC_CONTENT_TYPE).as_str(),
        );

        let mut codec = LanguageServerCodec::<AllMessages>::default();
        assert!(codec.decode(&mut message_bytes).unwrap().is_none())
    }

    #[test]
    fn ok_on_partial_headers() {
        let mut message_bytes = BytesMut::from("cont");

        let mut codec = LanguageServerCodec::<AllMessages>::default();
        assert!(codec.decode(&mut message_bytes).unwrap().is_none())
    }

    #[test]
    fn ok_on_partial_content() {
        let mut message_bytes = BytesMut::from(
            JsonRpcHeaders {
                content_length: 100,
            }
            .to_string()
            .as_str(),
        );

        let mut codec = LanguageServerCodec::<AllMessages>::default();
        assert!(codec.decode(&mut message_bytes).unwrap().is_none())
    }

    #[test]
    #[should_panic]
    fn to_many_bytes_are_unimplemented() {
        let mut message_bytes = BytesMut::from(
            format!("{}\r\nsomething", JsonRpcHeaders { content_length: 1 }).as_str(),
        );

        let mut codec = LanguageServerCodec::<AllMessages>::default();
        let _ = codec.decode(&mut message_bytes);
    }
}
