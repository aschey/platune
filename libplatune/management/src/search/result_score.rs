use std::{cmp::Ordering, iter::Sum};

#[derive(Debug)]
pub(crate) struct ResultScore {
    pub(crate) match_len_score: usize,
    pub(crate) full_len_score: usize,
    pub(crate) weighted_score: f32,
    pub(crate) full_entry: String,
}

impl Default for ResultScore {
    fn default() -> Self {
        Self {
            match_len_score: 0,
            full_len_score: 0,
            weighted_score: 0.0,
            full_entry: "".to_owned(),
        }
    }
}

impl Sum for ResultScore {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| ResultScore {
            weighted_score: a.weighted_score + b.weighted_score,
            match_len_score: a.match_len_score + b.match_len_score,
            full_len_score: a.full_len_score,
            full_entry: a.full_entry,
        })
        .unwrap_or_default()
    }
}

impl PartialEq for ResultScore {
    fn eq(&self, other: &Self) -> bool {
        self.weighted_score.partial_cmp(&other.weighted_score) == Some(Ordering::Equal)
            && self.full_len_score.cmp(&other.full_len_score) == Ordering::Equal
    }
}

impl PartialOrd for ResultScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let weighted_ord = self.weighted_score.partial_cmp(&other.weighted_score);
        if weighted_ord != Some(Ordering::Equal) && weighted_ord != None {
            return weighted_ord;
        }
        let len_ord = self.match_len_score.cmp(&other.match_len_score);
        if len_ord != Ordering::Equal {
            return Some(len_ord);
        }
        let full_len_ord = self.full_len_score.cmp(&other.full_len_score);
        if full_len_ord != Ordering::Equal {
            return Some(full_len_ord);
        }
        self.full_entry.partial_cmp(&other.full_entry)
    }
}