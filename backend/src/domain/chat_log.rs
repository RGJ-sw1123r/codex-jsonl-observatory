use indexmap::IndexMap;

use super::RenderedEntry;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedChatLog {
    pub entries: Vec<RenderedEntry>,
    pub parsed_candidates: usize,
    pub ignored_lines: usize,
    pub malformed_lines: usize,
    pub observed_event_counts: IndexMap<String, usize>,
}

impl ParsedChatLog {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            parsed_candidates: 0,
            ignored_lines: 0,
            malformed_lines: 0,
            observed_event_counts: IndexMap::new(),
        }
    }
}
