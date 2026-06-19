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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![parse_selected_jsonl])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
