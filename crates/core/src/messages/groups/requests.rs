use lsp_types::request::*;
use serde::{Deserialize, Serialize};

use crate::messages::core::request::{LspRequest, RequestId, RequestMessage};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllRequests {
    Client(AllClientRequests),
    Server(AllServerRequests),
}

impl LspRequest for AllRequests {
    fn request_id(&self) -> &RequestId {
        match self {
            AllRequests::Client(request) => request.request_id(),
            AllRequests::Server(request) => request.request_id(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllClientRequests {
    RegisterCapability(RequestMessage<RegisterCapability>),
    UnregisterCapability(RequestMessage<UnregisterCapability>),
    InlayHintRefresh(RequestMessage<InlayHintRefreshRequest>),
    SemanticTokensRefresh(RequestMessage<SemanticTokensRefresh>),
    InlineValueRefresh(RequestMessage<InlineValueRefreshRequest>),
    WorkspaceDiagnosticsRefresh(RequestMessage<WorkspaceDiagnosticRefresh>),
    Configuration(RequestMessage<WorkspaceConfiguration>),
    WorkspaceFolders(RequestMessage<WorkspaceFoldersRequest>),
    ApplyWorkspaceEdit(RequestMessage<ApplyWorkspaceEdit>),
    ShowMessageRequest(RequestMessage<ShowMessageRequest>),
    ShowDocument(RequestMessage<ShowDocument>),
    WorkDoneProgressCreate(RequestMessage<WorkDoneProgressCreate>),
}

impl LspRequest for AllClientRequests {
    fn request_id(&self) -> &RequestId {
        match self {
            AllClientRequests::RegisterCapability(request) => request.request_id(),
            AllClientRequests::UnregisterCapability(request) => request.request_id(),
            AllClientRequests::InlayHintRefresh(request) => request.request_id(),
            AllClientRequests::SemanticTokensRefresh(request) => request.request_id(),
            AllClientRequests::InlineValueRefresh(request) => request.request_id(),
            AllClientRequests::WorkspaceDiagnosticsRefresh(request) => request.request_id(),
            AllClientRequests::Configuration(request) => request.request_id(),
            AllClientRequests::WorkspaceFolders(request) => request.request_id(),
            AllClientRequests::ApplyWorkspaceEdit(request) => request.request_id(),
            AllClientRequests::ShowMessageRequest(request) => request.request_id(),
            AllClientRequests::ShowDocument(request) => request.request_id(),
            AllClientRequests::WorkDoneProgressCreate(request) => request.request_id(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllServerRequests {
    Initialize(RequestMessage<Initialize>),
    Shutdown(RequestMessage<Shutdown>),
    WillSaveWaitUntilTextDocument(RequestMessage<WillSaveWaitUntil>),
    GotoDeclaration(RequestMessage<GotoDeclaration>),
    GotoDefinition(RequestMessage<GotoDefinition>),
    GotoTypeDefinition(RequestMessage<GotoTypeDefinition>),
    GotoImplementation(RequestMessage<GotoImplementation>),
    References(RequestMessage<References>),
    CallHierarchyPrepare(RequestMessage<CallHierarchyPrepare>),
    CallHierarchyIncoming(RequestMessage<CallHierarchyIncomingCalls>),
    CallHierarchyOutgoing(RequestMessage<CallHierarchyOutgoingCalls>),
    TypeHierarchyPrepare(RequestMessage<TypeHierarchyPrepare>),
    TypeHierarchySuper(RequestMessage<TypeHierarchySupertypes>),
    TypeHierarchySub(RequestMessage<TypeHierarchySubtypes>),
    DocumentHighlights(RequestMessage<DocumentHighlightRequest>),
    DocumentLink(RequestMessage<DocumentLinkRequest>),
    DocumentLinkResolve(RequestMessage<DocumentLinkResolve>),
    Hover(RequestMessage<HoverRequest>),
    CodeLens(RequestMessage<CodeLensRequest>),
    CodeLensResolve(RequestMessage<CodeLensResolve>),
    CodeLensRefresh(RequestMessage<CodeLensRefresh>),
    FoldingRange(RequestMessage<FoldingRangeRequest>),
    SelectionRange(RequestMessage<SelectionRangeRequest>),
    DocumentSymbols(RequestMessage<DocumentSymbolRequest>),
    SemanticTokensFull(RequestMessage<SemanticTokensFullRequest>),
    SemanticTokensFullDelta(RequestMessage<SemanticTokensFullDeltaRequest>),
    SemanticTokensRange(RequestMessage<SemanticTokensRangeRequest>),
    InlayHint(RequestMessage<InlayHintRequest>),
    InlayHindResolve(RequestMessage<InlayHintResolveRequest>),
    InlineValue(RequestMessage<InlineValueRequest>),
    Moniker(RequestMessage<MonikerRequest>),
    Completion(RequestMessage<Completion>),
    ResolveCompletionItem(RequestMessage<ResolveCompletionItem>),
    DocumentDiagnostics(RequestMessage<DocumentDiagnosticRequest>),
    WorkspaceDiagnostics(RequestMessage<WorkspaceDiagnosticRequest>),
    SignatureHelp(RequestMessage<SignatureHelpRequest>),
    CodeAction(RequestMessage<CodeActionRequest>),
    CodeActionResolve(RequestMessage<CodeActionResolveRequest>),
    DocumentColor(RequestMessage<DocumentColor>),
    ColorPresentation(RequestMessage<ColorPresentationRequest>),
    DocumentFormatting(RequestMessage<Formatting>),
    DocumentRangeFormatting(RequestMessage<RangeFormatting>),
    DocumentOnTypeFormatting(RequestMessage<OnTypeFormatting>),
    Rename(RequestMessage<Rename>),
    PrepareRename(RequestMessage<PrepareRenameRequest>),
    LinkedEditingRange(RequestMessage<LinkedEditingRange>),
    WorkspaceSymbols(RequestMessage<WorkspaceSymbolRequest>),
    WorkspaceSymbolsResolve(RequestMessage<WorkspaceSymbolResolve>),
    WillCreateFiles(RequestMessage<WillCreateFiles>),
    WillRenameFiles(RequestMessage<WillRenameFiles>),
    WillDeleteFiles(RequestMessage<WillDeleteFiles>),
    ExecuteCommand(RequestMessage<ExecuteCommand>),
}

impl LspRequest for AllServerRequests {
    fn request_id(&self) -> &RequestId {
        match self {
            AllServerRequests::Initialize(request) => request.request_id(),
            AllServerRequests::Shutdown(request) => request.request_id(),
            AllServerRequests::WillSaveWaitUntilTextDocument(request) => request.request_id(),
            AllServerRequests::GotoDeclaration(request) => request.request_id(),
            AllServerRequests::GotoDefinition(request) => request.request_id(),
            AllServerRequests::GotoTypeDefinition(request) => request.request_id(),
            AllServerRequests::GotoImplementation(request) => request.request_id(),
            AllServerRequests::References(request) => request.request_id(),
            AllServerRequests::CallHierarchyPrepare(request) => request.request_id(),
            AllServerRequests::CallHierarchyIncoming(request) => request.request_id(),
            AllServerRequests::CallHierarchyOutgoing(request) => request.request_id(),
            AllServerRequests::TypeHierarchyPrepare(request) => request.request_id(),
            AllServerRequests::TypeHierarchySuper(request) => request.request_id(),
            AllServerRequests::TypeHierarchySub(request) => request.request_id(),
            AllServerRequests::DocumentHighlights(request) => request.request_id(),
            AllServerRequests::DocumentLink(request) => request.request_id(),
            AllServerRequests::DocumentLinkResolve(request) => request.request_id(),
            AllServerRequests::Hover(request) => request.request_id(),
            AllServerRequests::CodeLens(request) => request.request_id(),
            AllServerRequests::CodeLensResolve(request) => request.request_id(),
            AllServerRequests::CodeLensRefresh(request) => request.request_id(),
            AllServerRequests::FoldingRange(request) => request.request_id(),
            AllServerRequests::SelectionRange(request) => request.request_id(),
            AllServerRequests::DocumentSymbols(request) => request.request_id(),
            AllServerRequests::SemanticTokensFull(request) => request.request_id(),
            AllServerRequests::SemanticTokensFullDelta(request) => request.request_id(),
            AllServerRequests::SemanticTokensRange(request) => request.request_id(),
            AllServerRequests::InlayHint(request) => request.request_id(),
            AllServerRequests::InlayHindResolve(request) => request.request_id(),
            AllServerRequests::InlineValue(request) => request.request_id(),
            AllServerRequests::Moniker(request) => request.request_id(),
            AllServerRequests::Completion(request) => request.request_id(),
            AllServerRequests::ResolveCompletionItem(request) => request.request_id(),
            AllServerRequests::DocumentDiagnostics(request) => request.request_id(),
            AllServerRequests::WorkspaceDiagnostics(request) => request.request_id(),
            AllServerRequests::SignatureHelp(request) => request.request_id(),
            AllServerRequests::CodeAction(request) => request.request_id(),
            AllServerRequests::CodeActionResolve(request) => request.request_id(),
            AllServerRequests::DocumentColor(request) => request.request_id(),
            AllServerRequests::ColorPresentation(request) => request.request_id(),
            AllServerRequests::DocumentFormatting(request) => request.request_id(),
            AllServerRequests::DocumentRangeFormatting(request) => request.request_id(),
            AllServerRequests::DocumentOnTypeFormatting(request) => request.request_id(),
            AllServerRequests::Rename(request) => request.request_id(),
            AllServerRequests::PrepareRename(request) => request.request_id(),
            AllServerRequests::LinkedEditingRange(request) => request.request_id(),
            AllServerRequests::WorkspaceSymbols(request) => request.request_id(),
            AllServerRequests::WorkspaceSymbolsResolve(request) => request.request_id(),
            AllServerRequests::WillCreateFiles(request) => request.request_id(),
            AllServerRequests::WillRenameFiles(request) => request.request_id(),
            AllServerRequests::WillDeleteFiles(request) => request.request_id(),
            AllServerRequests::ExecuteCommand(request) => request.request_id(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub enum SomeRequestsMock {
        WillRenameFiles(RequestMessage<WillRenameFiles>),
    }

    impl From<SomeRequestsMock> for AllRequests {
        fn from(some_requests: SomeRequestsMock) -> Self {
            match some_requests {
                SomeRequestsMock::WillRenameFiles(request) => {
                    AllRequests::Server(AllServerRequests::WillRenameFiles(request))
                }
            }
        }
    }

    impl TryFrom<AllRequests> for SomeRequestsMock {
        type Error = AllRequests;

        fn try_from(all_requests: AllRequests) -> Result<Self, Self::Error> {
            match all_requests {
                AllRequests::Server(AllServerRequests::WillRenameFiles(request)) => {
                    Ok(SomeRequestsMock::WillRenameFiles(request))
                }
                request => Err(request),
            }
        }
    }
}
