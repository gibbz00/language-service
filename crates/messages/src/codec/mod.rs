mod decode;
mod encode;
mod headers;

use std::marker::PhantomData;

use crate::groups::Message;

pub struct LanguageServerCodec<M: Message> {
    known_content_length: Option<usize>,
    marker: PhantomData<M>,
}

impl<M: Message> Default for LanguageServerCodec<M> {
    fn default() -> Self {
        Self {
            known_content_length: None,
            marker: PhantomData,
        }
    }
}
