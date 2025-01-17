mod decode;
mod encode;

pub use decode::DecodeError;
pub use encode::EncodeError;

use std::marker::PhantomData;

use crate::messages::groups::MessageGroup;

pub struct LanguageServerCodec<M: MessageGroup> {
    known_content_length: Option<usize>,
    marker: PhantomData<M>,
}

impl<M: MessageGroup> Default for LanguageServerCodec<M> {
    fn default() -> Self {
        Self {
            known_content_length: None,
            marker: PhantomData,
        }
    }
}
