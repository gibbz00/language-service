use lsp_types::notification::*;
use serde::{Deserialize, Serialize};

use crate::core::notification::NotificationMessage;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllNotifications {
    Client(AllClientNotifications),
    Server(AllServerNotifications),
    ImplementationDependent(AllImplementationNotifications),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllClientNotifications {
    LogTrace(NotificationMessage<LogTrace>),
    LogMessage(NotificationMessage<LogMessage>),
    PublishDiagnostics(NotificationMessage<PublishDiagnostics>),
    ShowMessage(NotificationMessage<ShowMessage>),
    Telemetry(NotificationMessage<TelemetryEvent>),
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllImplementationNotifications {
    CancelRequest(NotificationMessage<Cancel>),
    Progress(NotificationMessage<Progress>),
}
