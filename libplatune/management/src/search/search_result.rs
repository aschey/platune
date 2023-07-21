use crate::entry_type::EntryType;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct SearchResult {
    pub entry: String,
    pub entry_type: EntryType,
    pub description: String,
    pub artist: Option<String>,
    pub correlation_ids: Vec<i64>,
}
