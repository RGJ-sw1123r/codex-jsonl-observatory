#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedEntry {
    pub kind: RenderedEntryKind,
    pub content: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderedEntryKind {
    Context,
    Task,
    You,
    Codex,
    ToolCall,
    ToolResult,
    System,
}

impl RenderedEntryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Context => "[CONTEXT]",
            Self::Task => "[TASK]",
            Self::You => "[YOU]",
            Self::Codex => "[CODEX]",
            Self::ToolCall => "[TOOL CALL]",
            Self::ToolResult => "[TOOL RESULT]",
            Self::System => "[SYSTEM]",
        }
    }
}
