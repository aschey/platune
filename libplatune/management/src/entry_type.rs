use strum::EnumString;

#[derive(Debug, EnumString, Clone)]
#[strum(ascii_case_insensitive)]
pub enum EntryType {
    Song,
    Artist,
    Album,
}
