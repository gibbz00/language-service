// IMPROVEMENT: ?
#![allow(clippy::large_enum_variant)]

pub mod notifications;
pub mod requests;
pub mod responses;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::{notifications::AllNotifications, requests::AllRequests, responses::AllResponses};

pub trait MessageGroup: Serialize + DeserializeOwned {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllMessages {
    Requests(AllRequests),
    Responses(AllResponses),
    Notifications(AllNotifications),
}

impl MessageGroup for AllMessages {}

#[cfg(test)]
pub mod tests {
    use lsp_types::request::Shutdown;
    use serde::{Deserialize, Serialize};

    use crate::messages::core::response::{tests::SHUTDOWN_RESPONSE_MOCK, ResponseMessage};

    use super::MessageGroup;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum MockAgentMessage {
        Shutdown(ResponseMessage<Shutdown>),
    }

    impl MessageGroup for MockAgentMessage {}

    pub const AGENT_MESSAGE_MOCK: MockAgentMessage =
        MockAgentMessage::Shutdown(SHUTDOWN_RESPONSE_MOCK);
}
