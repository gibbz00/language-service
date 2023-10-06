use bytes::{Buf, BytesMut};
use derive_more::From;
use tokio_util::codec::Decoder;

use crate::{codec::headers::JsonRpcHeaders, core::Message};

use super::{headers::HeadersParseError, LanguageServerCodec};

#[derive(Debug, From)]
pub enum DecodeError {
    Io(std::io::Error),
    Httparse(httparse::Error),
    HeadersParseError(HeadersParseError),
    Deserialize(serde_json::Error),
}

impl<M: Message> Decoder for LanguageServerCodec<M> {
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

                                let missing_capacity = content_length - src.capacity();
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
    use lsp_types::request::Shutdown;
    use once_cell::sync::Lazy;

    use crate::{
        codec::{
            encode::protocol_message::ProtocolMessage,
            headers::{CONTENT_TYPE_HEADER_NAME, JSON_RPC_CONTENT_TYPE},
        },
        core::response::{tests::SHUTDOWN_RESPONSE_MOCK, ResponseMessage},
    };

    use super::*;

    static PROTOCOL_MESSAGE: Lazy<String> = Lazy::new(|| {
        ProtocolMessage::try_new(SHUTDOWN_RESPONSE_MOCK)
            .unwrap()
            .to_string()
    });

    #[test]
    fn decodes_messages() {
        let mut message_bytes = BytesMut::new();
        let mut codec = LanguageServerCodec::<ResponseMessage<Shutdown>>::default();
        decode_message(&mut message_bytes, &mut codec);
        decode_message(&mut message_bytes, &mut codec);

        fn decode_message(
            message_bytes: &mut BytesMut,
            codec: &mut LanguageServerCodec<ResponseMessage<Shutdown>>,
        ) {
            message_bytes.put(PROTOCOL_MESSAGE.as_bytes());
            assert_eq!(
                SHUTDOWN_RESPONSE_MOCK,
                codec.decode(message_bytes).unwrap().unwrap()
            )
        }
    }

    #[test]
    fn ok_on_missing_content_length_header() {
        let mut message_bytes = BytesMut::from(
            format!("{}: {}", CONTENT_TYPE_HEADER_NAME, JSON_RPC_CONTENT_TYPE).as_str(),
        );

        let mut codec = LanguageServerCodec::<ResponseMessage<Shutdown>>::default();
        assert!(codec.decode(&mut message_bytes).unwrap().is_none())
    }

    #[test]
    fn ok_on_partial_headers() {
        let mut message_bytes = BytesMut::from("cont");

        let mut codec = LanguageServerCodec::<ResponseMessage<Shutdown>>::default();
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

        let mut codec = LanguageServerCodec::<ResponseMessage<Shutdown>>::default();
        assert!(codec.decode(&mut message_bytes).unwrap().is_none())
    }

    #[test]
    #[should_panic]
    fn to_many_bytes_are_unimplemented() {
        let mut message_bytes = BytesMut::from(
            format!("{}\r\nsomething", JsonRpcHeaders { content_length: 1 }).as_str(),
        );

        let mut codec = LanguageServerCodec::<ResponseMessage<Shutdown>>::default();
        let _ = codec.decode(&mut message_bytes);
    }
}
