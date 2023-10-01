use std::fmt::Display;

// Not an official IANA Media Type:
// https://www.iana.org/assignments/media-types/media-types.xhtml
pub(crate) const JSON_RPC_CONTENT_TYPE: &str = "application/vscode-jsonrpc; charset=utf-8";
// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#contentPart
const DEPRECATED_CONTENT_TYPE: &str = "application/vscode-jsonrpc; charset=utf8";

const CONTENT_LENGTH_HEADER_NAME: &str = "Content-Length";
pub(crate) const CONTENT_TYPE_HEADER_NAME: &str = "Content-Type";

pub struct JsonRpcHeaders {
    pub content_length: usize,
}

impl Display for JsonRpcHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}\r\n",
            CONTENT_LENGTH_HEADER_NAME, self.content_length
        )?;
        write!(
            f,
            "{}: {}\r\n",
            CONTENT_TYPE_HEADER_NAME, JSON_RPC_CONTENT_TYPE
        )
    }
}

#[derive(Debug)]
pub enum HeadersParseError {
    InvalidHeader(String),
    DuplicateOfValidHeader(String),
    Utf8(std::str::Utf8Error),
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
            let content_type_bytes = headers[content_type_header_index].value;

            if content_type_bytes != JSON_RPC_CONTENT_TYPE.as_bytes()
                && content_type_bytes != DEPRECATED_CONTENT_TYPE.as_bytes()
            {
                let content_type = std::str::from_utf8(content_type_bytes)
                    .map_err(HeadersParseError::Utf8)?
                    .to_owned();
                return Err(HeadersParseError::InvalidContentType(content_type));
            }
        }

        let Some(content_length_header_index) = content_length_header_index else {
            return Err(HeadersParseError::MissingContentLength);
        };

        Ok(JsonRpcHeaders {
            content_length: std::str::from_utf8(headers[content_length_header_index].value)
                .map_err(HeadersParseError::Utf8)
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
        let header = JsonRpcHeaders { content_length: 10 };
        let expected_string = replace_with_crlf(indoc::indoc! {"
                    Content-Length: 10
                    Content-Type: application/vscode-jsonrpc; charset=utf-8
                "});
        assert_eq!(expected_string, header.to_string());
    }

    #[test]
    fn fails_on_unknown_headers() {
        let headers = [httparse::Header {
            name: "x",
            value: b"X",
        }];
        assert!(matches!(
            JsonRpcHeaders::try_from(&headers[..]),
            Err(HeadersParseError::InvalidHeader(_)),
        ))
    }

    #[test]
    fn fails_on_duplicate_headers() {
        let headers = [
            httparse::Header {
                name: CONTENT_TYPE_HEADER_NAME,
                value: b"x",
            },
            httparse::Header {
                name: CONTENT_TYPE_HEADER_NAME,
                value: b"x",
            },
        ];
        assert!(matches!(
            JsonRpcHeaders::try_from(&headers[..]),
            Err(HeadersParseError::DuplicateOfValidHeader(_)),
        ))
    }

    #[test]
    fn fails_on_content_type_mismatch() {
        let headers = [httparse::Header {
            name: CONTENT_TYPE_HEADER_NAME,
            value: b"X",
        }];
        assert!(matches!(
            JsonRpcHeaders::try_from(&headers[..]),
            Err(HeadersParseError::InvalidContentType(_)),
        ))
    }

    #[test]
    fn allows_missing_content_type() {
        let headers = [httparse::Header {
            name: CONTENT_LENGTH_HEADER_NAME,
            value: b"10",
        }];
        assert!(JsonRpcHeaders::try_from(&headers[..]).is_ok())
    }

    #[test]
    fn fails_on_missing_content_length() {
        let headers = [httparse::Header {
            name: CONTENT_TYPE_HEADER_NAME,
            value: JSON_RPC_CONTENT_TYPE.as_bytes(),
        }];

        assert!(matches!(
            JsonRpcHeaders::try_from(&headers[..]),
            Err(HeadersParseError::MissingContentLength),
        ))
    }

    #[test]
    fn backwards_compatible_utf8_content_type_header() {
        let headers = [
            httparse::Header {
                name: CONTENT_LENGTH_HEADER_NAME,
                value: b"10",
            },
            httparse::Header {
                name: CONTENT_TYPE_HEADER_NAME,
                value: DEPRECATED_CONTENT_TYPE.as_bytes(),
            },
        ];
        JsonRpcHeaders::try_from(&headers[..]).unwrap();
        assert!(JsonRpcHeaders::try_from(&headers[..]).is_ok())
    }
}
