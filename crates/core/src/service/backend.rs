use crate::messages::groups::{
    notifications::{AllClientNotifications, AllServerNotifications},
    requests::{AllClientRequests, AllServerRequests},
};

trait ServiceBackend {
    type OutgoingRequests;
    type OutgoingNotifications;
    type IncomingRequests;
    type IncomingNotifications;
}

// TODO: move into a spique-client crate
struct ClientServiceBackend;

impl ServiceBackend for ClientServiceBackend {
    type OutgoingRequests = AllServerRequests;
    type OutgoingNotifications = AllServerNotifications;
    type IncomingRequests = AllClientRequests;
    type IncomingNotifications = AllClientNotifications;
}
