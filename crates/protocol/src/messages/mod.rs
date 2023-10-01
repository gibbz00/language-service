pub(crate) mod notification;
pub(crate) mod request;
pub(crate) mod response;
mod version;

use serde::{de::DeserializeOwned, Serialize};

pub trait Message: Serialize + DeserializeOwned {}
