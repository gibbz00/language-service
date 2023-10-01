use std::marker::PhantomData;

use crate::json_rpc::Message;

mod headers {
    use std::fmt::Display;

    // Not an official IANA Media Type:
    // https://www.iana.org/assignments/media-types/media-types.xhtml
    const JSON_RPC_CONTENT_TYPE: &str = "application/vscode-jsonrpc; charset=utf-8";

    const CONTENT_TYPE_HEADER_NAME: &str = "Content-Length";
    const CONTENT_LENGTH_HEADER_NAME: &str = "Content-Type";

    pub struct JsonRpcHeaders {
        content_length: usize,
    }

    impl JsonRpcHeaders {
        pub fn new(content_length: usize) -> Self {
            Self { content_length }
        }

        pub fn content_length(&self) -> usize {
            self.content_length
        }
    }

    impl Display for JsonRpcHeaders {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}: {}\r\n",
                CONTENT_LENGTH_HEADER_NAME, JSON_RPC_CONTENT_TYPE
            )?;
            write!(
                f,
                "{}: {}\r\n",
                CONTENT_TYPE_HEADER_NAME, self.content_length
            )
        }
    }

    pub enum HeadersParseError {
        InvalidHeader(String),
        DuplicateOfValidHeader(String),
        Value(std::str::Utf8Error),
        InvalidContentType(String),
        ContentLength(std::num::ParseIntError),
        MissingContentLength,
    }

    impl TryFrom<&[httparse::Header<'_>]> for JsonRpcHeaders {
        type Error = HeadersParseError;

        fn try_from(headers: &[httparse::Header]) -> Result<Self, Self::Error> {
            let mut content_length_header_index: Option<usize> = None;
            let mut content_type_header_index: Option<usize> = None;
            for (header_index, header) in headers.iter().enumerate() {
                match header.name {
                    CONTENT_LENGTH_HEADER_NAME => match content_length_header_index.is_some() {
                        true => {
                            return Err(HeadersParseError::DuplicateOfValidHeader(
                                CONTENT_LENGTH_HEADER_NAME.to_owned(),
                            ))
                        }
                        false => content_length_header_index = Some(header_index),
                    },
                    CONTENT_TYPE_HEADER_NAME => match content_type_header_index.is_some() {
                        true => {
                            return Err(HeadersParseError::DuplicateOfValidHeader(
                                CONTENT_TYPE_HEADER_NAME.to_owned(),
                            ))
                        }
                        false => content_type_header_index = Some(header_index),
                    },
                    invalid_header_name => {
                        return Err(HeadersParseError::InvalidHeader(
                            invalid_header_name.to_owned(),
                        ))
                    }
                }
            }

            if let Some(content_type_header_index) = content_type_header_index {
                let content_type = std::str::from_utf8(headers[content_type_header_index].value)
                    .map_err(HeadersParseError::Value)?;

                if content_type != JSON_RPC_CONTENT_TYPE {
                    return Err(HeadersParseError::InvalidContentType(
                        content_type.to_owned(),
                    ));
                }
            }

            let Some(content_length_header_index) = content_length_header_index else {
                return Err(HeadersParseError::MissingContentLength);
            };

            Ok(JsonRpcHeaders {
                content_length: std::str::from_utf8(headers[content_length_header_index].value)
                    .map_err(HeadersParseError::Value)
                    .and_then(|content_length_str| {
                        content_length_str
                            .parse()
                            .map_err(HeadersParseError::ContentLength)
                    })?,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn replace_with_crlf(str: &str) -> String {
            str.replace('\n', "\r\n")
        }

        #[test]
        fn displays_header() {
            let header = JsonRpcHeaders::new(10);
            let expected_string = replace_with_crlf(indoc::indoc! {"
                    Content-Type: application/vscode-jsonrpc; charset=utf-8
                    Content-Length: 10
                "});
            assert_eq!(expected_string, header.to_string());
        }

        #[test]
        fn fails_on_unknown_headers() {
            todo!()
        }

        #[test]
        fn fails_on_duplicate_headers() {
            todo!()
        }

        #[test]
        fn fails_on_content_type_mismatch() {
            todo!()
        }

        #[test]
        fn allows_missing_content_type() {
            todo!()
        }

        #[test]
        fn fails_on_missing_content_length() {
            todo!()
        }
    }
}

pub struct LanguageServerCodec<M: Message> {
    known_content_length: Option<usize>,
    marker: PhantomData<M>,
}

mod encode {
    use std::io::Write;

    use bytes::{BufMut, BytesMut};
    use derive_more::From;
    use tokio_util::codec::Encoder;

    use crate::json_rpc::Message;

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

        use crate::{codec::headers::JsonRpcHeaders, json_rpc::Message};

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
            use crate::json_rpc::response::tests::SHUTDOWN_RESPONSE_MOCK;

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
}

mod decode {
    use bytes::{Buf, BytesMut};
    use derive_more::From;
    use tokio_util::codec::Decoder;

    use crate::{codec::headers::JsonRpcHeaders, json_rpc::Message};

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
                                    let content_length = json_rpc_headers.content_length();
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
}
