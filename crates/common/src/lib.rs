pub enum ClientToServerMessages {
    ClientRequest(ClientRequest),
    ClientNotification(ClientNotification),
    ServerResponse(ServerResponse),
}

pub enum ServerToClientMessages {
    ServerRequest(ServerRequest),
    ServerNotification(ServerNotification),
    ClientResponse(ClientResponse),
}

pub enum ClientRequest {}

pub enum ClientNotification {}

pub enum ServerResponse {}

pub enum ServerRequest {}

pub enum ServerNotification {}

pub enum ClientResponse {}
