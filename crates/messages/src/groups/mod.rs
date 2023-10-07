use lsp_types::{notification::*, request::*};
use serde::{de::DeserializeOwned, Serialize};

pub trait Message: Serialize + DeserializeOwned {}

pub enum AllMessages {
    Requests(AllRequests),
    Notifications(AllNotifications),
}

pub enum AllRequests {
    Client(AllClientRequest),
    Server(AllServerRequest),
}

pub enum AllClientRequest {
    RegisterCapability(RegisterCapability),
    UnregisterCapability(UnregisterCapability),
    InlayHintRefresh(InlayHintRefreshRequest),
    SemanticTokensRefresh(SemanticTokensRefresh),
    InlineValueRefresh(InlineValueRefreshRequest),
    WorkspaceDiagnosticsRefresh(WorkspaceDiagnosticRefresh),
    Configuration(WorkspaceConfiguration),
    WorkspaceFolders(WorkspaceFoldersRequest),
    ApplyWorkspaceEdit(ApplyWorkspaceEdit),
    ShowMessageRequest(ShowMessageRequest),
    ShowDocument(ShowDocument),
    WorkDoneProgressCreate(WorkDoneProgressCreate),
    WorkDoneProgressCancel(WorkDoneProgressCancel),
}

pub enum AllServerRequest {
    Initialize(Initialize),
    ShutDown(Shutdown),
    WillSaveWaitUntilTextDocument(WillSaveWaitUntil),

    GotoDeclaration(GotoDeclaration),
    GotoDefinition(GotoDefinition),
    GotoTypeDefinition(GotoTypeDefinition),
    GotoImplementation(GotoImplementation),
    References(References),

    CallHierarchyPrepare(CallHierarchyPrepare),
    CallHierarchyIncoming(CallHierarchyIncomingCalls),
    CallHierarchyOutgoing(CallHierarchyOutgoingCalls),

    TypeHierarchyPrepare(TypeHierarchyPrepare),
    TypeHierarchySuper(TypeHierarchySupertypes),
    TypeHierarchySub(TypeHierarchySubtypes),

    DocumentHighlights(DocumentHighlightRequest),
    DocumentLink(DocumentLinkRequest),
    DocumentLinkResolve(DocumentLinkResolve),
    Hover(HoverRequest),
    CodeLens(CodeLensRequest),
    CodeLensResolve(CodeLensResolve),
    CodeLensRefresh(CodeLensRefresh),
    FoldingRange(FoldingRangeRequest),
    SelectionRange(SelectionRangeRequest),
    DocumentSymbols(DocumentSymbolRequest),

    SemanticTokensFull(SemanticTokensFullRequest),
    SemanticTokensFullDelta(SemanticTokensFullDeltaRequest),
    SemanticTokensRange(SemanticTokensRangeRequest),

    InlayHint(InlayHintRequest),
    InlayHindResolve(InlayHintResolveRequest),
    InlineValue(InlineValueRequest),

    Moniker(MonikerRequest),
    Completion(Completion),
    ResolveCompletionItem(ResolveCompletionItem),
    DocumentDiagnostics(DocumentDiagnosticRequest),
    WorkspaceDiagnostics(WorkspaceDiagnosticRequest),
    SignatureHelp(SignatureHelpRequest),
    CodeAction(CodeActionRequest),
    CodeActionResolve(CodeActionResolveRequest),
    DocumentColor(DocumentColor),
    ColorPresentation(ColorPresentationRequest),
    DocumentFormatting(Formatting),
    DocumentRangeFormatting(RangeFormatting),
    DocumentOnTypeFormatting(OnTypeFormatting),
    Rename(Rename),
    PrepareRename(PrepareRenameRequest),
    LinkedEditingRange(LinkedEditingRange),

    WorkspaceSymbols(WorkspaceSymbolRequest),
    WorkspaceSymbolsResolve(WorkspaceSymbolResolve),
    WillCreateFiles(WillCreateFiles),
    WillRenameFiles(WillRenameFiles),
    WillDeleteFiles(WillDeleteFiles),
    ExecuteCommand(ExecuteCommand),
}

pub enum AllNotifications {
    Client(AllClientNotifications),
    Server(AllServerNotifications),
    ImplementationDependent(AllImplementationNotifications),
}

pub enum AllClientNotifications {
    LogTrace(LogTrace),
    LogMessage(LogMessage),
    PublishDiagnostics(PublishDiagnostics),
    ShowMessage(ShowMessage),
    Telemetry(TelemetryEvent),
}

pub enum AllServerNotifications {
    Initialized(Initialized),
    SetTrace(SetTrace),
    Exit(Exit),
    WillSaveTextDocument(WillSaveTextDocument),
    DidOpenText(DidOpenTextDocument),
    DidChangeText(DidChangeTextDocument),
    DidSaveTextDocument(DidSaveTextDocument),
    DidCloseTextDocument(DidCloseTextDocument),
    DidOpenNotebook(DidOpenNotebookDocument),
    DidChangeNotebook(DidChangeNotebookDocument),
    DidSaveNotebookDocument(DidSaveNotebookDocument),
    DidCloseNotebookDocument(DidCloseNotebookDocument),
    DidChangeNotification(DidChangeConfiguration),
    DidChangeWorkspaceFolders(DidChangeWorkspaceFolders),
    DidCreateFiles(DidCreateFiles),
    DidRenameFiles(DidRenameFiles),
    DidDeleteFiles(DidDeleteFiles),
    DidChangeWatcheFiles(DidChangeWatchedFiles),
}

pub enum AllImplementationNotifications {
    CancelRequest(Cancel),
    Progress(Progress),
}

#[cfg(test)]
pub mod tests {
    use lsp_types::request::Shutdown;
    use serde::{Deserialize, Serialize};

    use crate::core::response::{tests::SHUTDOWN_RESPONSE_MOCK, ResponseMessage};

    use super::Message;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum MockAgentMessage {
        Shutdown(ResponseMessage<Shutdown>),
    }

    impl Message for MockAgentMessage {}

    pub const AGENT_MESSAGE_MOCK: MockAgentMessage =
        MockAgentMessage::Shutdown(SHUTDOWN_RESPONSE_MOCK);
}
