mod decode;
mod encode;
mod headers;

use std::marker::PhantomData;

use crate::messages::Message;

pub struct LanguageServerCodec<M: Message> {
    known_content_length: Option<usize>,
    marker: PhantomData<M>,
}
