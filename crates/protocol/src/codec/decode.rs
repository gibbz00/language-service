use bytes::{Buf, BytesMut};
use derive_more::From;
use tokio_util::codec::Decoder;

use crate::{codec::headers::JsonRpcHeaders, messages::Message};

use super::{headers::HeadersParseError, LanguageServerCodec};

#[derive(From)]
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

                                Ok(None)
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
