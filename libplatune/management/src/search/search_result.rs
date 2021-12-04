use crate::entry_type::EntryType;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: String,
    pub entry_type: EntryType,
    pub description: String,
    pub artist: Option<String>,
    pub correlation_ids: Vec<i32>,
}
