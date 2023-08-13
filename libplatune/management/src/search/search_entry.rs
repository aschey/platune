use std::cmp::Ordering;
use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

use super::result_score::ResultScore;
use crate::consts::{END_MATCH_TEXT, MIN_LEN, MIN_WORDS, START_MATCH_TEXT};

#[derive(Debug)]
pub(crate) struct SearchEntry {
    pub(crate) entry: String,
    pub entry_type: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub correlation_id: i64,
    pub(crate) original_query: String,
    pub(crate) start_highlight: String,
    pub(crate) end_highlight: String,
    pub(crate) weights: HashMap<String, f32>,
}
lazy_static! {
    static ref MATCH_REGEX: String = format!(
        r"(?:{}(.*?){}[^\s]*).*",
        START_MATCH_TEXT.replace('{', r"\{").replace('}', r"\}"),
        END_MATCH_TEXT.replace('{', r"\{").replace('}', r"\}")
    );
}

impl SearchEntry {
    fn score_match(&self) -> ResultScore {
        let entry = self.entry.to_lowercase();
        let count = entry.matches(START_MATCH_TEXT).count();

        let re = Regex::new(&MATCH_REGEX.repeat(count)).unwrap();
        let caps = re.captures(&entry).unwrap();

        let mut score: ResultScore = caps
            .iter()
            .skip(1)
            .map(|c| match c.map(|c| c.as_str()) {
                Some(cap) => match self.weights.get(&cap.to_lowercase()) {
                    Some(weight) => ResultScore {
                        weighted_score: *weight,
                        match_len_score: cap.len(),
                        full_len_score: entry.len(),
                        full_entry: entry.to_owned(),
                    },
                    None => ResultScore {
                        weighted_score: 0.,
                        match_len_score: cap.len(),
                        full_len_score: entry.len(),
                        full_entry: entry.to_owned(),
                    },
                },
                None => ResultScore {
                    weighted_score: 0.,
                    match_len_score: 0,
                    full_len_score: entry.len(),
                    full_entry: entry.to_owned(),
                },
            })
            .sum();

        if score.weighted_score == 0.
            && score.match_len_score + count >= MIN_LEN
            && count >= MIN_WORDS
            && self.original_query.len() == count
        {
            score.match_len_score = self.original_query.len();
        }

        score
    }

    pub fn get_description(&self) -> String {
        match &self.entry_type[..] {
            "song" => format!(
                "Song from {} by {}",
                self.album.to_owned().unwrap_or_default(),
                self.artist.to_owned().unwrap_or_default(),
            ),
            "album" => format!("Album by {}", self.artist.to_owned().unwrap_or_default()),
            "artist" => "Artist".to_owned(),
            "album_artist" => "Album Artist".to_owned(),
            _ => "".to_owned(),
        }
    }

    pub fn get_formatted_entry(&self) -> String {
        self.entry
            .replace("{startmatch}", &self.start_highlight)
            .replace("{endmatch}", &self.end_highlight)
    }
}

impl Ord for SearchEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for SearchEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_score = self.score_match();
        let other_score = other.score_match();

        self_score.partial_cmp(&other_score)
    }
}

impl Eq for SearchEntry {}

impl PartialEq for SearchEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
