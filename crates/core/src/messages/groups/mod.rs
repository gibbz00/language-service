// IMPROVEMENT: ?
#![allow(clippy::large_enum_variant)]

pub mod notifications;
pub mod requests;
pub mod responses;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::{notifications::AllNotifications, requests::AllRequests, responses::AllResponses};

pub trait MessageGroup: Serialize + DeserializeOwned {}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllMessages {
    Requests(AllRequests),
    Responses(AllResponses),
    Notifications(AllNotifications),
}

impl MessageGroup for AllMessages {}

#[cfg(test)]
pub mod tests {
    use crate::messages::core::response::tests::SHUTDOWN_RESPONSE_MOCK;

    use super::{
        responses::{AllResponses, AllServerResponses},
        AllMessages,
    };

    pub const MESSAGE_MOCK: AllMessages = AllMessages::Responses(AllResponses::Server(
        AllServerResponses::Shutdown(SHUTDOWN_RESPONSE_MOCK),
    ));
}
