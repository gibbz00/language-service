pub(crate) mod error;
pub(crate) mod notification;
pub(crate) mod request;
pub(crate) mod response;
pub(crate) mod version;

use serde::{de::DeserializeOwned, Serialize};

pub trait Message: Serialize + DeserializeOwned {}
