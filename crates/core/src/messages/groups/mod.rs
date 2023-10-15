// IMPROVEMENT: ?
#![allow(clippy::large_enum_variant)]

pub mod notifications;
pub mod requests;
pub mod responses;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::{notifications::AllNotifications, requests::AllRequests, responses::AllResponses};

use super::core::{LspRequest, RequestId};

pub trait MessageGroup: Serialize + DeserializeOwned {}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllMessages {
    Requests(AllRequests),
    Responses(AllResponses),
    Notifications(AllNotifications),
}

impl AllMessages {
    pub fn request_id(&self) -> Option<&RequestId> {
        match self {
            AllMessages::Requests(request) => Some(request.request_id()),
            AllMessages::Responses(_) | AllMessages::Notifications(_) => None,
        }
    }
}

impl MessageGroup for AllMessages {}

#[cfg(test)]
pub mod tests {
    use crate::messages::core::request::tests::WILL_RENAME_FILES_REQUEST_MOCK;

    use super::{
        requests::{AllRequests::Server, AllServerRequests::WillRenameFiles},
        AllMessages,
    };

    pub const MESSAGE_MOCK: AllMessages =
        AllMessages::Requests(Server(WillRenameFiles(WILL_RENAME_FILES_REQUEST_MOCK)));
}
