use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};

use crate::{
    domain::{ChatEntryFilter, ParsedChatLog, RenderedEntryKind},
    parser,
};

const SESSION_ID_HEX_GROUPS: [usize; 5] = [8, 4, 4, 4, 12];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseRequestDto {
    pub path: String,
    pub filter: FilterDto,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterDto {
    pub show_you: bool,
    pub show_codex: bool,
    pub show_tool_call: bool,
    pub show_tool_result: bool,
    pub show_meta: bool,
}

impl Default for FilterDto {
    fn default() -> Self {
        Self::from(ChatEntryFilter::all())
    }
}

impl From<FilterDto> for ChatEntryFilter {
    fn from(filter: FilterDto) -> Self {
        Self {
            show_you: filter.show_you,
            show_codex: filter.show_codex,
            show_tool_call: filter.show_tool_call,
            show_tool_result: filter.show_tool_result,
            show_meta: filter.show_meta,
        }
    }
}

impl From<ChatEntryFilter> for FilterDto {
    fn from(filter: ChatEntryFilter) -> Self {
        Self {
            show_you: filter.show_you,
            show_codex: filter.show_codex,
            show_tool_call: filter.show_tool_call,
            show_tool_result: filter.show_tool_result,
            show_meta: filter.show_meta,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseResponseDto {
    pub source: LoadedFileMetadataDto,
    pub parsed_chat_log: ParsedChatLogDto,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedFileMetadataDto {
    pub file_name: Option<String>,
    pub absolute_path: String,
    pub session_id: Option<String>,
    pub resume_command: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedChatLogDto {
    pub entries: Vec<RenderedEntryDto>,
    pub transcript_blocks: Vec<TranscriptBlockDto>,
    pub counters: ParseCountersDto,
    pub observed_event_counts: Vec<ObservedEventCountDto>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedEntryDto {
    pub kind: EntryKindDto,
    pub label: &'static str,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptBlockDto {
    pub entry_type: EntryKindDto,
    pub label: &'static str,
    pub title: &'static str,
    pub content: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKindDto {
    Context,
    Task,
    You,
    Codex,
    ToolCall,
    ToolResult,
    System,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseCountersDto {
    pub parsed_candidates: usize,
    pub total_entries: usize,
    pub visible_entries: usize,
    pub ignored_lines: usize,
    pub malformed_lines: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedEventCountDto {
    pub event: String,
    pub count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorResponseDto {
    pub error: ApiErrorDto,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiErrorDto {
    pub code: String,
    pub message: String,
}

pub type ApiResult<T> = Result<T, ErrorResponseDto>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseBoundaryRequest {
    pub path: String,
    pub filter: Option<FilterDto>,
}

pub fn parse_for_transport(request: ParseBoundaryRequest) -> ApiResult<ParseResponseDto> {
    parse_selected_file(ParseRequestDto {
        path: request.path,
        filter: request.filter.unwrap_or_default(),
    })
}

pub fn parse_selected_file(request: ParseRequestDto) -> ApiResult<ParseResponseDto> {
    let path = PathBuf::from(&request.path);
    let source = LoadedFileMetadataDto::from_path(&path).map_err(ErrorResponseDto::from_io)?;
    let parsed = parser::parse_file(&path).map_err(ErrorResponseDto::from_io)?;
    let filter = ChatEntryFilter::from(request.filter);

    Ok(ParseResponseDto {
        source,
        parsed_chat_log: ParsedChatLogDto::from_domain(&parsed, &filter),
    })
}

pub fn project_parsed_chat_log(
    parsed: &ParsedChatLog,
    filter: Option<FilterDto>,
) -> ParsedChatLogDto {
    let filter = filter.unwrap_or_default();
    ParsedChatLogDto::from_domain(parsed, &ChatEntryFilter::from(filter))
}

impl LoadedFileMetadataDto {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let absolute_path = absolute_path(path)?;
        let session_id = detect_session_id(path);
        let resume_command = session_id
            .as_ref()
            .map(|session_id| format!("codex resume {session_id}"));

        Ok(Self {
            file_name: path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            absolute_path: absolute_path.to_string_lossy().into_owned(),
            session_id,
            resume_command,
        })
    }
}

impl ParseResponseDto {
    pub fn to_json(&self) -> Value {
        json!({
            "source": self.source.to_json(),
            "parsed_chat_log": self.parsed_chat_log.to_json(),
        })
    }
}

impl LoadedFileMetadataDto {
    pub fn to_json(&self) -> Value {
        json!({
            "file_name": self.file_name,
            "absolute_path": self.absolute_path,
            "session_id": self.session_id,
            "resume_command": self.resume_command,
        })
    }
}

impl ParsedChatLogDto {
    pub fn from_domain(parsed: &ParsedChatLog, filter: &ChatEntryFilter) -> Self {
        let visible_entries = parsed
            .entries
            .iter()
            .filter(|entry| api_filter_allows_kind(filter, entry.kind))
            .collect::<Vec<_>>();
        let entries = visible_entries
            .iter()
            .map(|entry| RenderedEntryDto {
                kind: EntryKindDto::from(entry.kind),
                label: entry.kind.label(),
                content: entry.content.clone(),
            })
            .collect();
        let transcript_blocks = visible_entries
            .iter()
            .map(|entry| {
                let label = entry.kind.label();
                TranscriptBlockDto {
                    entry_type: EntryKindDto::from(entry.kind),
                    label,
                    title: label,
                    content: entry.content.clone(),
                }
            })
            .collect();
        let observed_event_counts = parsed
            .observed_event_counts
            .iter()
            .map(|(event, count)| ObservedEventCountDto {
                event: event.clone(),
                count: *count,
            })
            .collect();

        Self {
            entries,
            transcript_blocks,
            counters: ParseCountersDto {
                parsed_candidates: parsed.parsed_candidates,
                total_entries: parsed.entries.len(),
                visible_entries: visible_entries.len(),
                ignored_lines: parsed.ignored_lines,
                malformed_lines: parsed.malformed_lines,
            },
            observed_event_counts,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "entries": self.entries.iter().map(RenderedEntryDto::to_json).collect::<Vec<_>>(),
            "transcript_blocks": self.transcript_blocks.iter().map(TranscriptBlockDto::to_json).collect::<Vec<_>>(),
            "counters": self.counters.to_json(),
            "observed_event_counts": self.observed_event_counts.iter().map(ObservedEventCountDto::to_json).collect::<Vec<_>>(),
        })
    }
}

fn api_filter_allows_kind(filter: &ChatEntryFilter, kind: RenderedEntryKind) -> bool {
    match kind {
        RenderedEntryKind::Context | RenderedEntryKind::Task | RenderedEntryKind::System => {
            filter.show_meta
        }
        RenderedEntryKind::You => filter.show_you,
        RenderedEntryKind::Codex => filter.show_codex,
        RenderedEntryKind::ToolCall => filter.show_tool_call,
        RenderedEntryKind::ToolResult => filter.show_tool_result,
    }
}

impl RenderedEntryDto {
    pub fn to_json(&self) -> Value {
        json!({
            "kind": self.kind.as_str(),
            "label": self.label,
            "content": self.content,
        })
    }
}

impl TranscriptBlockDto {
    pub fn to_json(&self) -> Value {
        json!({
            "entry_type": self.entry_type.as_str(),
            "label": self.label,
            "title": self.title,
            "content": self.content,
        })
    }
}

impl From<RenderedEntryKind> for EntryKindDto {
    fn from(kind: RenderedEntryKind) -> Self {
        match kind {
            RenderedEntryKind::Context => Self::Context,
            RenderedEntryKind::Task => Self::Task,
            RenderedEntryKind::You => Self::You,
            RenderedEntryKind::Codex => Self::Codex,
            RenderedEntryKind::ToolCall => Self::ToolCall,
            RenderedEntryKind::ToolResult => Self::ToolResult,
            RenderedEntryKind::System => Self::System,
        }
    }
}

impl EntryKindDto {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Task => "task",
            Self::You => "you",
            Self::Codex => "codex",
            Self::ToolCall => "tool_call",
            Self::ToolResult => "tool_result",
            Self::System => "system",
        }
    }
}

impl ParseCountersDto {
    pub fn to_json(&self) -> Value {
        json!({
            "parsed_candidates": self.parsed_candidates,
            "total_entries": self.total_entries,
            "visible_entries": self.visible_entries,
            "ignored_lines": self.ignored_lines,
            "malformed_lines": self.malformed_lines,
        })
    }
}

impl ObservedEventCountDto {
    pub fn to_json(&self) -> Value {
        json!({
            "event": self.event,
            "count": self.count,
        })
    }
}

impl ErrorResponseDto {
    pub fn from_io(error: io::Error) -> Self {
        Self {
            error: ApiErrorDto {
                code: "parse_file_failed".to_owned(),
                message: error.to_string(),
            },
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "error": {
                "code": self.error.code,
                "message": self.error.message,
            }
        })
    }
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(_) if path.is_absolute() => Ok(path.to_path_buf()),
        Err(_) => Ok(std::env::current_dir()?.join(path)),
    }
}

fn detect_session_id(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(first_uuid)
        .or_else(|| {
            path.ancestors()
                .skip(1)
                .filter_map(|ancestor| ancestor.file_name()?.to_str())
                .find_map(first_uuid)
        })
}

fn first_uuid(value: &str) -> Option<String> {
    value
        .char_indices()
        .filter_map(|(start, character)| character.is_ascii_hexdigit().then_some(start))
        .find_map(|start| uuid_at(value, start))
}

fn uuid_at(value: &str, start: usize) -> Option<String> {
    let bytes = value.as_bytes();
    let mut index = start;

    if start > 0 && is_word_byte(bytes[start - 1]) {
        return None;
    }

    for (group_index, group_len) in SESSION_ID_HEX_GROUPS.iter().enumerate() {
        for _ in 0..*group_len {
            if index >= bytes.len() || !bytes[index].is_ascii_hexdigit() {
                return None;
            }
            index += 1;
        }

        if group_index < SESSION_ID_HEX_GROUPS.len() - 1 {
            if index >= bytes.len() || bytes[index] != b'-' {
                return None;
            }
            index += 1;
        }
    }

    if index < bytes.len() && is_word_byte(bytes[index]) {
        return None;
    }

    Some(value[start..index].to_owned())
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{RenderedEntry, RenderedEntryKind};
    use indexmap::IndexMap;

    fn parsed_log() -> ParsedChatLog {
        let mut observed_event_counts = IndexMap::new();
        observed_event_counts.insert("event_msg/user_message".to_owned(), 2);
        observed_event_counts.insert("response_item/message".to_owned(), 1);

        ParsedChatLog {
            entries: vec![
                RenderedEntry {
                    kind: RenderedEntryKind::You,
                    content: "hello".to_owned(),
                },
                RenderedEntry {
                    kind: RenderedEntryKind::Codex,
                    content: "hi".to_owned(),
                },
                RenderedEntry {
                    kind: RenderedEntryKind::ToolCall,
                    content: "run tests".to_owned(),
                },
                RenderedEntry {
                    kind: RenderedEntryKind::Task,
                    content: "task".to_owned(),
                },
                RenderedEntry {
                    kind: RenderedEntryKind::System,
                    content: "session".to_owned(),
                },
            ],
            parsed_candidates: 5,
            ignored_lines: 3,
            malformed_lines: 1,
            observed_event_counts,
        }
    }

    #[test]
    fn parsed_chat_log_dto_preserves_domain_counts_and_visible_count() {
        let dto = ParsedChatLogDto::from_domain(
            &parsed_log(),
            &ChatEntryFilter {
                show_tool_call: false,
                show_meta: false,
                ..ChatEntryFilter::all()
            },
        );

        assert_eq!(dto.counters.parsed_candidates, 5);
        assert_eq!(dto.counters.total_entries, 5);
        assert_eq!(dto.counters.visible_entries, 2);
        assert_eq!(dto.counters.ignored_lines, 3);
        assert_eq!(dto.counters.malformed_lines, 1);
        assert_eq!(
            dto.entries
                .iter()
                .map(|entry| entry.kind)
                .collect::<Vec<_>>(),
            vec![EntryKindDto::You, EntryKindDto::Codex]
        );
        assert_eq!(dto.entries[0].label, "[YOU]");
        assert_eq!(dto.transcript_blocks[1].title, "[CODEX]");
        assert_eq!(
            dto.observed_event_counts,
            vec![
                ObservedEventCountDto {
                    event: "event_msg/user_message".to_owned(),
                    count: 2,
                },
                ObservedEventCountDto {
                    event: "response_item/message".to_owned(),
                    count: 1,
                }
            ]
        );
    }

    #[test]
    fn api_filter_treats_task_as_meta_for_source_app_parity() {
        let dto = ParsedChatLogDto::from_domain(
            &parsed_log(),
            &ChatEntryFilter {
                show_you: false,
                show_meta: true,
                ..ChatEntryFilter::all()
            },
        );

        assert_eq!(
            dto.entries
                .iter()
                .map(|entry| entry.kind)
                .collect::<Vec<_>>(),
            vec![
                EntryKindDto::Codex,
                EntryKindDto::ToolCall,
                EntryKindDto::Task,
                EntryKindDto::System,
            ]
        );
    }

    #[test]
    fn serialized_contract_uses_stable_field_names_and_entry_kinds() {
        let response = ParseResponseDto {
            source: LoadedFileMetadataDto {
                file_name: Some("session.jsonl".to_owned()),
                absolute_path: "C:\\sessions\\session.jsonl".to_owned(),
                session_id: Some("11111111-2222-3333-4444-555555555555".to_owned()),
                resume_command: Some(
                    "codex resume 11111111-2222-3333-4444-555555555555".to_owned(),
                ),
            },
            parsed_chat_log: ParsedChatLogDto::from_domain(&parsed_log(), &ChatEntryFilter::all()),
        };

        let json = response.to_json();

        assert_eq!(json["source"]["file_name"], "session.jsonl");
        assert_eq!(
            json["source"]["absolute_path"],
            "C:\\sessions\\session.jsonl"
        );
        assert_eq!(
            json["source"]["session_id"],
            "11111111-2222-3333-4444-555555555555"
        );
        assert_eq!(
            json["source"]["resume_command"],
            "codex resume 11111111-2222-3333-4444-555555555555"
        );
        assert_eq!(json["parsed_chat_log"]["entries"][0]["kind"], "you");
        assert_eq!(json["parsed_chat_log"]["entries"][0]["label"], "[YOU]");
        assert_eq!(
            json["parsed_chat_log"]["counters"]["visible_entries"],
            json["parsed_chat_log"]["entries"]
                .as_array()
                .expect("entries is an array")
                .len()
        );
        assert_eq!(
            json["parsed_chat_log"]["observed_event_counts"][0]["event"],
            "event_msg/user_message"
        );
    }

    #[test]
    fn loaded_file_metadata_detects_session_id_from_file_stem_first() {
        let path = Path::new(
            "E:\\sessions\\aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee\\11111111-2222-3333-4444-555555555555.jsonl",
        );

        let metadata = LoadedFileMetadataDto::from_path(path).expect("metadata builds");

        assert_eq!(
            metadata.file_name,
            Some("11111111-2222-3333-4444-555555555555.jsonl".to_owned())
        );
        assert_eq!(
            metadata.session_id,
            Some("11111111-2222-3333-4444-555555555555".to_owned())
        );
        assert_eq!(
            metadata.resume_command,
            Some("codex resume 11111111-2222-3333-4444-555555555555".to_owned())
        );
    }

    #[test]
    fn loaded_file_metadata_detects_session_id_from_parent_when_file_has_none() {
        let path = Path::new("E:\\sessions\\aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee\\codex.jsonl");

        let metadata = LoadedFileMetadataDto::from_path(path).expect("metadata builds");

        assert_eq!(
            metadata.session_id,
            Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_owned())
        );
        assert_eq!(
            metadata.resume_command,
            Some("codex resume aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_owned())
        );
    }

    #[test]
    fn loaded_file_metadata_omits_resume_command_without_session_id() {
        let path = Path::new("E:\\sessions\\codex.jsonl");

        let metadata = LoadedFileMetadataDto::from_path(path).expect("metadata builds");

        assert_eq!(metadata.session_id, None);
        assert_eq!(metadata.resume_command, None);
    }

    #[test]
    fn error_response_shape_is_serializable_for_frontend_display() {
        let response = ErrorResponseDto::from_io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "permission denied",
        ));

        let json = response.to_json();

        assert_eq!(json["error"]["code"], "parse_file_failed");
        assert_eq!(json["error"]["message"], "permission denied");
    }

    #[test]
    fn transport_boundary_defaults_to_all_filters() {
        let dto = project_parsed_chat_log(&parsed_log(), None);

        assert_eq!(dto.counters.total_entries, 5);
        assert_eq!(dto.counters.visible_entries, 5);
        assert_eq!(
            dto.entries
                .iter()
                .map(|entry| entry.kind)
                .collect::<Vec<_>>(),
            vec![
                EntryKindDto::You,
                EntryKindDto::Codex,
                EntryKindDto::ToolCall,
                EntryKindDto::Task,
                EntryKindDto::System,
            ]
        );
    }

    #[test]
    fn transport_boundary_projects_explicit_filter_without_reparse() {
        let dto = project_parsed_chat_log(
            &parsed_log(),
            Some(FilterDto {
                show_you: false,
                show_codex: true,
                show_tool_call: false,
                show_tool_result: true,
                show_meta: true,
            }),
        );

        assert_eq!(dto.counters.parsed_candidates, 5);
        assert_eq!(dto.counters.ignored_lines, 3);
        assert_eq!(dto.counters.malformed_lines, 1);
        assert_eq!(dto.counters.total_entries, 5);
        assert_eq!(dto.counters.visible_entries, 3);
        assert_eq!(
            dto.entries
                .iter()
                .map(|entry| entry.kind)
                .collect::<Vec<_>>(),
            vec![
                EntryKindDto::Codex,
                EntryKindDto::Task,
                EntryKindDto::System
            ]
        );
    }

    #[test]
    fn transport_boundary_request_defaults_filter_before_parse() {
        let request = ParseBoundaryRequest {
            path: "E:\\sessions\\codex.jsonl".to_owned(),
            filter: None,
        };

        let selected = ParseRequestDto {
            path: request.path.clone(),
            filter: request.filter.unwrap_or_default(),
        };

        assert_eq!(selected.filter, FilterDto::default());
        assert_eq!(selected.filter, FilterDto::from(ChatEntryFilter::all()));
    }

    #[test]
    fn transport_boundary_json_shape_stays_dto_compatible() {
        let response = ParseResponseDto {
            source: LoadedFileMetadataDto {
                file_name: Some("codex.jsonl".to_owned()),
                absolute_path: "E:\\sessions\\codex.jsonl".to_owned(),
                session_id: None,
                resume_command: None,
            },
            parsed_chat_log: project_parsed_chat_log(&parsed_log(), None),
        };

        let json = response.to_json();

        assert_eq!(json["source"]["file_name"], "codex.jsonl");
        assert!(json["source"]["session_id"].is_null());
        assert!(json["source"]["resume_command"].is_null());
        assert_eq!(json["parsed_chat_log"]["counters"]["total_entries"], 5);
        assert_eq!(json["parsed_chat_log"]["counters"]["visible_entries"], 5);
        assert_eq!(json["parsed_chat_log"]["entries"][3]["kind"], "task");
    }
}
