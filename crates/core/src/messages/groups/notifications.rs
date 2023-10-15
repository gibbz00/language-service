use lsp_types::notification::*;
use serde::{Deserialize, Serialize};

use crate::messages::core::notification::NotificationMessage;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllNotifications {
    Client(AllClientNotifications),
    Server(AllServerNotifications),
    ImplementationDependent(AllImplementationNotifications),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllClientNotifications {
    LogTrace(NotificationMessage<LogTrace>),
    LogMessage(NotificationMessage<LogMessage>),
    PublishDiagnostics(NotificationMessage<PublishDiagnostics>),
    ShowMessage(NotificationMessage<ShowMessage>),
    Telemetry(NotificationMessage<TelemetryEvent>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum AllServerNotifications {
    Initialized(NotificationMessage<Initialized>),
    SetTrace(NotificationMessage<SetTrace>),
    Exit(NotificationMessage<Exit>),
    WillSaveTextDocument(NotificationMessage<WillSaveTextDocument>),
    WorkDoneProgressCancel(NotificationMessage<WorkDoneProgressCancel>),
    DidOpenText(NotificationMessage<DidOpenTextDocument>),
    DidChangeText(NotificationMessage<DidChangeTextDocument>),
    DidSaveTextDocument(NotificationMessage<DidSaveTextDocument>),
    DidCloseTextDocument(NotificationMessage<DidCloseTextDocument>),
    DidOpenNotebook(NotificationMessage<DidOpenNotebookDocument>),
    DidChangeNotebook(NotificationMessage<DidChangeNotebookDocument>),
    DidSaveNotebookDocument(NotificationMessage<DidSaveNotebookDocument>),
    DidCloseNotebookDocument(NotificationMessage<DidCloseNotebookDocument>),
    DidChangeNotification(NotificationMessage<DidChangeConfiguration>),
    DidChangeWorkspaceFolders(NotificationMessage<DidChangeWorkspaceFolders>),
    DidCreateFiles(NotificationMessage<DidCreateFiles>),
    DidRenameFiles(NotificationMessage<DidRenameFiles>),
    DidDeleteFiles(NotificationMessage<DidDeleteFiles>),
    DidChangeWatcheFiles(NotificationMessage<DidChangeWatchedFiles>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllImplementationNotifications {
    CancelRequest(NotificationMessage<Cancel>),
    Progress(NotificationMessage<Progress>),
}

#[cfg(test)]
pub mod tests {
    use crate::messages::{
        core::response::response_error::{
            ReservedResponseErrorCodes, ResponseError, ResponseErrorCode::Reserved,
        },
        groups::responses::errors::InvalidMessageResponse,
    };

    use super::*;

    #[derive(Debug, PartialEq)]
    pub enum SomeNotificationsMock {
        Exit(NotificationMessage<Exit>),
    }

    impl From<SomeNotificationsMock> for AllNotifications {
        fn from(some_notifications: SomeNotificationsMock) -> Self {
            match some_notifications {
                SomeNotificationsMock::Exit(notification) => {
                    AllNotifications::Server(AllServerNotifications::Exit(notification))
                }
            }
        }
    }

    impl TryFrom<AllNotifications> for SomeNotificationsMock {
        type Error = InvalidMessageResponse;

        fn try_from(all_notifications: AllNotifications) -> Result<Self, Self::Error> {
            match all_notifications {
                AllNotifications::Server(AllServerNotifications::Exit(notification)) => {
                    Ok(SomeNotificationsMock::Exit(notification))
                }
                notification => Err(InvalidMessageResponse::new(
                    None,
                    ResponseError {
                        code: Reserved(ReservedResponseErrorCodes::InternalError),
                        message: "Invalid notification.".to_string(),
                        data: Some(
                            serde_json::to_value(notification)
                                .expect("notification not serializable"),
                        ),
                    },
                )),
            }
        }
    }
}
