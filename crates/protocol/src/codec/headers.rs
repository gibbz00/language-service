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
