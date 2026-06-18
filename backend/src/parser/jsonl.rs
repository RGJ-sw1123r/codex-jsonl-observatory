use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use indexmap::IndexMap;
use serde_json::Value;

use crate::domain::{ParsedChatLog, RenderedEntry, RenderedEntryKind};

pub fn parse_str(input: &str) -> ParsedChatLog {
    parse_lossy_lines(input)
}

pub fn parse_reader<R: Read>(mut reader: R) -> io::Result<ParsedChatLog> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(parse_lossy_lines(&String::from_utf8_lossy(&bytes)))
}

pub fn parse_file(path: impl AsRef<Path>) -> io::Result<ParsedChatLog> {
    let path = path.as_ref();
    if !path.is_file() {
        return Ok(ParsedChatLog::empty());
    }

    parse_reader(File::open(path)?)
}

fn parse_lossy_lines(input: &str) -> ParsedChatLog {
    let mut entries = Vec::new();
    let mut ignored_lines = 0;
    let mut malformed_lines = 0;
    let mut observed_event_counts = IndexMap::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let value = match serde_json::from_str::<Value>(line) {
            Ok(value) => value,
            Err(_) => {
                malformed_lines += 1;
                continue;
            }
        };

        increment_observed_event_count(&mut observed_event_counts, &value);

        match extract_entry(&value) {
            Some(entry) => entries.push(entry),
            None => ignored_lines += 1,
        }
    }

    ParsedChatLog {
        parsed_candidates: entries.len(),
        entries,
        ignored_lines,
        malformed_lines,
        observed_event_counts,
    }
}

fn increment_observed_event_count(counts: &mut IndexMap<String, usize>, value: &Value) {
    let top_type = string_field(value, "type");
    let payload_type = value
        .get("payload")
        .and_then(|payload| string_field(payload, "type"));

    let key = match (top_type, payload_type) {
        (Some(top_type), Some(payload_type)) => format!("{top_type}/{payload_type}"),
        (None, Some(payload_type)) => payload_type.to_owned(),
        (Some(top_type), None) => top_type.to_owned(),
        (None, None) => "unknown".to_owned(),
    };

    *counts.entry(key).or_insert(0) += 1;
}

fn extract_entry(value: &Value) -> Option<RenderedEntry> {
    let top_type = string_field(value, "type")?;
    let payload = value.get("payload")?;
    let payload_type = string_field(payload, "type");

    match (top_type, payload_type) {
        ("event_msg", Some("user_message")) => {
            let content = string_field(payload, "message")?.trim();
            if content.is_empty() {
                None
            } else {
                Some(classify_user_message(content))
            }
        }
        ("event_msg", Some("agent_message")) => {
            let content = string_field(payload, "message")?.trim();
            if content.is_empty() {
                None
            } else {
                Some(RenderedEntry {
                    kind: RenderedEntryKind::Codex,
                    content: content.to_owned(),
                })
            }
        }
        ("response_item", Some("message")) => extract_response_message(payload),
        ("response_item", Some("function_call")) => extract_tool_call(payload, "Function call"),
        ("response_item", Some("custom_tool_call")) => {
            extract_tool_call(payload, "Custom tool call")
        }
        ("response_item", Some("function_call_output")) => {
            extract_tool_result(payload, "Function call output")
        }
        ("response_item", Some("custom_tool_call_output")) => {
            extract_tool_result(payload, "Custom tool call output")
        }
        _ => None,
    }
}

fn extract_response_message(payload: &Value) -> Option<RenderedEntry> {
    let role = string_field(payload, "role")?;
    let content = extract_text(payload)?.trim().to_owned();
    if content.is_empty() {
        return None;
    }

    if role == "user" {
        return Some(classify_user_message(&content));
    }

    let kind = match role {
        "assistant" | "model" => RenderedEntryKind::Codex,
        "tool" => RenderedEntryKind::ToolResult,
        "system" => RenderedEntryKind::System,
        _ => return None,
    };

    Some(RenderedEntry { kind, content })
}

fn extract_tool_call(payload: &Value, label: &str) -> Option<RenderedEntry> {
    let name = string_field(payload, "name")
        .or_else(|| string_field(payload, "call_id"))
        .or_else(|| string_field(payload, "id"))?;

    let mut content = format!("{label}: {name}");
    if let Some(arguments) = extract_tool_arguments(payload) {
        content.push('\n');
        content.push_str(&arguments);
    }

    Some(RenderedEntry {
        kind: RenderedEntryKind::ToolCall,
        content,
    })
}

fn extract_tool_result(payload: &Value, label: &str) -> Option<RenderedEntry> {
    let content = extract_text(payload)
        .or_else(|| string_field(payload, "call_id").map(|call_id| format!("{label}: {call_id}")))?
        .trim()
        .to_owned();

    if content.is_empty() {
        None
    } else {
        Some(RenderedEntry {
            kind: RenderedEntryKind::ToolResult,
            content,
        })
    }
}

fn extract_tool_arguments(payload: &Value) -> Option<String> {
    if let Some(arguments) = payload.get("arguments") {
        return match arguments {
            Value::String(arguments) => {
                let arguments = arguments.trim();
                (!arguments.is_empty()).then(|| arguments.to_owned())
            }
            Value::Object(_) | Value::Array(_) => Some(arguments.to_string()),
            _ => None,
        };
    }

    string_field(payload, "input")
        .map(str::trim)
        .filter(|input| !input.is_empty())
        .map(str::to_owned)
        .or_else(|| extract_text(payload))
}

fn classify_user_message(content: &str) -> RenderedEntry {
    let trimmed = content.trim();
    let kind = classify_user_message_kind(trimmed);

    RenderedEntry {
        kind,
        content: match kind {
            RenderedEntryKind::Context => "AGENTS.md project instructions loaded".to_owned(),
            RenderedEntryKind::Task => "Task or prompt instructions loaded".to_owned(),
            _ => trimmed.to_owned(),
        },
    }
}

fn classify_user_message_kind(content: &str) -> RenderedEntryKind {
    if is_agents_instructions(content) {
        RenderedEntryKind::Context
    } else if is_structured_task_or_prompt(content) {
        RenderedEntryKind::Task
    } else {
        RenderedEntryKind::You
    }
}

fn is_agents_instructions(content: &str) -> bool {
    content.contains("# AGENTS.md")
        || content.contains("AGENTS.md instructions")
        || content.contains("<INSTRUCTIONS>")
}

fn is_structured_task_or_prompt(content: &str) -> bool {
    [
        "<environment_context>",
        "<user_instructions>",
        "<developer_context>",
        "<task>",
        "<prompt>",
    ]
    .iter()
    .any(|marker| content.contains(marker))
}

fn extract_text(value: &Value) -> Option<String> {
    for field in ["content", "text", "message", "output", "result", "summary"] {
        if let Some(raw_value) = value.get(field) {
            if let Some(text) = text_from_value(raw_value) {
                return Some(text);
            }
        }
    }

    None
}

fn text_from_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(values) => {
            let text = values
                .iter()
                .filter_map(text_from_value)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_owned();

            (!text.is_empty()).then_some(text)
        }
        Value::Object(_) => {
            if let Some(nested) = value.get("text").and_then(text_from_value) {
                return Some(nested);
            }

            for field in ["value", "content", "message"] {
                if let Some(nested) = value.get(field).and_then(text_from_value) {
                    return Some(nested);
                }
            }

            None
        }
        _ => None,
    }
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field)?.as_str()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn event_user_message_line(message: &str) -> String {
        serde_json::json!({
            "type": "event_msg",
            "payload": {
                "type": "user_message",
                "message": message
            }
        })
        .to_string()
    }

    fn response_user_message_line(content: &str) -> String {
        serde_json::json!({
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": content
            }
        })
        .to_string()
    }

    fn event_agent_message_line(message: &str) -> String {
        serde_json::json!({
            "type": "event_msg",
            "payload": {
                "type": "agent_message",
                "message": message
            }
        })
        .to_string()
    }

    fn response_message_line(role: &str, content: &str) -> String {
        serde_json::json!({
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": role,
                "content": content
            }
        })
        .to_string()
    }

    fn response_item_line(payload_type: &str, fields: serde_json::Value) -> String {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "type".to_owned(),
            serde_json::Value::String(payload_type.to_owned()),
        );

        let serde_json::Value::Object(fields) = fields else {
            panic!("test fields must be a JSON object");
        };

        payload.extend(fields);

        serde_json::json!({
            "type": "response_item",
            "payload": payload
        })
        .to_string()
    }

    #[test]
    fn parses_event_msg_user_message_from_string() {
        let parsed = parse_str(
            r#"{"type":"event_msg","payload":{"type":"user_message","message":"hello"}}"#,
        );

        assert_eq!(parsed.malformed_lines, 0);
        assert_eq!(parsed.ignored_lines, 0);
        assert_eq!(parsed.parsed_candidates, 1);
        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::You,
                content: "hello".to_owned()
            }]
        );
    }

    #[test]
    fn parses_response_item_assistant_message_from_reader() {
        let input = Cursor::new(
            r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"hi from codex"}]}}"#,
        );

        let parsed = parse_reader(input).expect("reader parses");

        assert_eq!(parsed.parsed_candidates, 1);
        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::Codex,
                content: "hi from codex".to_owned()
            }]
        );
    }

    #[test]
    fn event_msg_agent_message_is_codex() {
        let parsed = parse_str(&event_agent_message_line("I can help with that."));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::Codex,
                content: "I can help with that.".to_owned()
            }]
        );
    }

    #[test]
    fn response_item_assistant_message_is_codex() {
        let parsed = parse_str(&response_message_line("assistant", "Assistant response"));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::Codex,
                content: "Assistant response".to_owned()
            }]
        );
    }

    #[test]
    fn response_item_model_message_is_codex() {
        let parsed = parse_str(&response_message_line("model", "Model response"));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::Codex,
                content: "Model response".to_owned()
            }]
        );
    }

    #[test]
    fn response_item_system_message_is_system() {
        let parsed = parse_str(&response_message_line("system", "System message"));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::System,
                content: "System message".to_owned()
            }]
        );
    }

    #[test]
    fn developer_messages_remain_ignored() {
        let parsed = parse_str(&response_message_line("developer", "Developer instruction"));

        assert_eq!(parsed.ignored_lines, 1);
        assert_eq!(parsed.parsed_candidates, 0);
        assert!(parsed.entries.is_empty());
    }

    #[test]
    fn response_item_function_call_is_tool_call() {
        let parsed = parse_str(&response_item_line(
            "function_call",
            serde_json::json!({
                "name": "read_file",
                "arguments": "{\"path\":\"README.md\"}"
            }),
        ));

        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].kind, RenderedEntryKind::ToolCall);
        assert_eq!(
            parsed.entries[0].content,
            "Function call: read_file\n{\"path\":\"README.md\"}"
        );
    }

    #[test]
    fn response_item_custom_tool_call_is_tool_call() {
        let parsed = parse_str(&response_item_line(
            "custom_tool_call",
            serde_json::json!({
                "name": "shell_command",
                "input": "cargo test"
            }),
        ));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::ToolCall,
                content: "Custom tool call: shell_command\ncargo test".to_owned()
            }]
        );
    }

    #[test]
    fn response_item_function_call_output_is_tool_result() {
        let parsed = parse_str(&response_item_line(
            "function_call_output",
            serde_json::json!({
                "call_id": "call_1",
                "output": "file contents"
            }),
        ));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::ToolResult,
                content: "file contents".to_owned()
            }]
        );
    }

    #[test]
    fn response_item_custom_tool_call_output_is_tool_result() {
        let parsed = parse_str(&response_item_line(
            "custom_tool_call_output",
            serde_json::json!({
                "call_id": "call_2",
                "content": [{"text": "command output"}]
            }),
        ));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::ToolResult,
                content: "command output".to_owned()
            }]
        );
    }

    #[test]
    fn observed_event_counts_include_tool_lines() {
        let parsed = parse_str(
            &[
                response_item_line("function_call", serde_json::json!({"name": "read_file"})),
                response_item_line(
                    "custom_tool_call_output",
                    serde_json::json!({"output": "done"}),
                ),
            ]
            .join("\n"),
        );

        assert_eq!(
            parsed.observed_event_counts["response_item/function_call"],
            1
        );
        assert_eq!(
            parsed.observed_event_counts["response_item/custom_tool_call_output"],
            1
        );
    }

    #[test]
    fn incomplete_or_unsupported_tool_payloads_are_ignored_safely() {
        let parsed = parse_str(
            &[
                response_item_line("function_call", serde_json::json!({"arguments": "{}"})),
                response_item_line("custom_tool_call_output", serde_json::json!({})),
                response_item_line("unsupported_tool", serde_json::json!({"name": "noop"})),
            ]
            .join("\n"),
        );

        assert_eq!(parsed.malformed_lines, 0);
        assert_eq!(parsed.ignored_lines, 3);
        assert_eq!(parsed.parsed_candidates, 0);
        assert!(parsed.entries.is_empty());
    }

    #[test]
    fn counts_malformed_non_empty_lines_and_continues() {
        let parsed = parse_str(
            "\nnot json\n{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"still parsed\"}}\n{",
        );

        assert_eq!(parsed.malformed_lines, 2);
        assert_eq!(parsed.ignored_lines, 0);
        assert_eq!(parsed.parsed_candidates, 1);
        assert_eq!(parsed.entries[0].content, "still parsed");
    }

    #[test]
    fn counts_ignored_valid_json_lines_once() {
        let parsed = parse_str(
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"unknown\",\"message\":\"ignored\"}}\n{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"developer\",\"content\":\"ignored\"}}",
        );

        assert_eq!(parsed.malformed_lines, 0);
        assert_eq!(parsed.ignored_lines, 2);
        assert_eq!(parsed.parsed_candidates, 0);
        assert!(parsed.entries.is_empty());
    }

    #[test]
    fn collects_observed_event_counts_for_valid_json_only() {
        let parsed = parse_str(
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"one\"}}\n{\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"two\"}}\n{\"payload\":{\"type\":\"payload_only\"}}\n{\"type\":\"top_only\",\"payload\":{}}\n{\"payload\":{}}\nnot json",
        );

        assert_eq!(parsed.malformed_lines, 1);
        assert_eq!(parsed.observed_event_counts["event_msg/user_message"], 2);
        assert_eq!(parsed.observed_event_counts["payload_only"], 1);
        assert_eq!(parsed.observed_event_counts["top_only"], 1);
        assert_eq!(parsed.observed_event_counts["unknown"], 1);
    }

    #[test]
    fn empty_trimmed_lines_are_not_ignored_or_malformed() {
        let parsed = parse_str("\n  \n\t\n");

        assert_eq!(parsed.malformed_lines, 0);
        assert_eq!(parsed.ignored_lines, 0);
        assert_eq!(parsed.parsed_candidates, 0);
        assert!(parsed.entries.is_empty());
        assert!(parsed.observed_event_counts.is_empty());
    }

    #[test]
    fn classifies_agents_instructions_as_context_summary() {
        let parsed = parse_str(&event_user_message_line(
            "# AGENTS.md instructions\n<INSTRUCTIONS>observe</INSTRUCTIONS>",
        ));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::Context,
                content: "AGENTS.md project instructions loaded".to_owned()
            }]
        );
    }

    #[test]
    fn classifies_structured_task_body_as_task_summary() {
        let parsed = parse_str(&response_user_message_line(
            "<environment_context>local</environment_context>\nBuild this",
        ));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::Task,
                content: "Task or prompt instructions loaded".to_owned()
            }]
        );
    }

    #[test]
    fn classifies_ordinary_human_prompt_as_you() {
        let parsed = parse_str(&event_user_message_line(
            "Can you explain how this parser handles malformed JSONL?",
        ));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::You,
                content: "Can you explain how this parser handles malformed JSONL?".to_owned()
            }]
        );
    }

    #[test]
    fn long_ordinary_human_prompt_stays_you() {
        let prompt = "Please review this parser behavior carefully. ".repeat(80);
        let parsed = parse_str(&response_user_message_line(&prompt));

        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].kind, RenderedEntryKind::You);
        assert_eq!(parsed.entries[0].content, prompt.trim());
    }

    #[test]
    fn korean_utf8_user_message_stays_intact() {
        let message = "\u{c548}\u{b155}\u{d558}\u{c138}\u{c694}. JSONL \u{d30c}\u{c11c}\u{b97c} \u{d655}\u{c778}\u{d574} \u{c8fc}\u{c138}\u{c694}.";
        let parsed = parse_str(&event_user_message_line(message));

        assert_eq!(
            parsed.entries,
            vec![RenderedEntry {
                kind: RenderedEntryKind::You,
                content: message.to_owned()
            }]
        );
    }

    #[test]
    fn preserves_unicode_message_content() {
        let parsed = parse_str(&response_message_line(
            "assistant",
            "\u{c548}\u{b155}\u{d558}\u{c138}\u{c694}",
        ));

        assert_eq!(
            parsed.entries[0],
            RenderedEntry {
                kind: RenderedEntryKind::Codex,
                content: "\u{c548}\u{b155}\u{d558}\u{c138}\u{c694}".to_owned()
            }
        );
    }

    #[test]
    fn lossy_reader_replaces_invalid_utf8_without_counting_as_io_error() {
        let input = Cursor::new(vec![
            b'{', b'"', b't', b'y', b'p', b'e', b'"', b':', b'"', b'e', b'v', b'e', b'n', b't',
            b'_', b'm', b's', b'g', b'"', b',', b'"', b'p', b'a', b'y', b'l', b'o', b'a', b'd',
            b'"', b':', b'{', b'"', b't', b'y', b'p', b'e', b'"', b':', b'"', b'u', b's', b'e',
            b'r', b'_', b'm', b'e', b's', b's', b'a', b'g', b'e', b'"', b',', b'"', b'm', b'e',
            b's', b's', b'a', b'g', b'e', b'"', b':', b'"', 0xff, b'"', b'}', b'}',
        ]);

        let parsed = parse_reader(input).expect("invalid utf8 is replaced");

        assert_eq!(parsed.malformed_lines, 0);
        assert_eq!(parsed.entries[0].content, "\u{fffd}");
    }
}
