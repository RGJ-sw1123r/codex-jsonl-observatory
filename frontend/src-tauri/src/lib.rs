use std::path::{Path, PathBuf};

use backend::api::{self, ParseBoundaryRequest};
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
struct TauriFilterDto {
    show_you: bool,
    show_codex: bool,
    show_tool_call: bool,
    show_tool_result: bool,
    show_meta: bool,
}

impl From<TauriFilterDto> for api::FilterDto {
    fn from(filter: TauriFilterDto) -> Self {
        Self {
            show_you: filter.show_you,
            show_codex: filter.show_codex,
            show_tool_call: filter.show_tool_call,
            show_tool_result: filter.show_tool_result,
            show_meta: filter.show_meta,
        }
    }
}

#[tauri::command]
fn parse_selected_jsonl(path: String, filter: TauriFilterDto) -> Result<Value, Value> {
    api::parse_for_transport(ParseBoundaryRequest {
        path,
        filter: Some(filter.into()),
    })
    .map(|response| response.to_json())
    .map_err(|error| error.to_json())
}

#[tauri::command]
fn resolve_jsonl_initial_directory(remembered_directory: Option<String>) -> String {
    if let Some(remembered_directory) = remembered_directory
        .as_deref()
        .filter(|path| !path.trim().is_empty())
        .map(Path::new)
        .filter(|path| path.is_dir())
    {
        return remembered_directory.to_string_lossy().into_owned();
    }

    let codex_home = std::env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let user_home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    let candidates = [
        codex_home.as_ref().map(|path| path.join("sessions")),
        user_home
            .as_ref()
            .map(|path| path.join(".codex").join("sessions")),
        user_home,
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|path| path.is_dir())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from("."))
        })
        .to_string_lossy()
        .into_owned()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            parse_selected_jsonl,
            resolve_jsonl_initial_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
