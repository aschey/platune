use std::{collections::HashMap, str::FromStr};

use super::{
    queries::{clean_query, correct_search, get_search_query, replace_ampersand},
    search_options::SearchOptions,
    search_result::SearchResult,
};
use crate::{
    entry_type::EntryType,
    search::{
        queries::{get_full_spellfix_query, END_MATCH_TEXT, START_MATCH_TEXT},
        search_entry::SearchEntry,
        spellfix_result::SpellfixResult,
    },
    spellfix::acquire_with_spellfix,
};
use itertools::Itertools;
use regex::Regex;
use sqlx::{pool::PoolConnection, Pool, Row, Sqlite};

#[derive(Clone)]
pub(crate) struct SearchEngine {
    pool: Pool<Sqlite>,
}

const MAX_TERMS: usize = 20;

impl SearchEngine {
    pub(crate) fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub(crate) async fn search(
        &self,
        query: &str,
        options: SearchOptions<'_>,
    ) -> Vec<SearchResult> {
        // Parse out artist filter if it was supplied
        let (adj_query, artist_filter) = self.split_artist_filter(query).await;

        return self
            .search_helper(&adj_query, &adj_query, options, artist_filter)
            .await;
    }

    async fn split_artist_filter(&self, query: &str) -> (String, Vec<String>) {
        let artist_split = query.split("artist:").collect_vec();

        match *artist_split.as_slice() {
            [] => return ("".to_owned(), vec![]),
            [query] => return (query.to_owned(), vec![]),
            [query, artist, ..] => {
                let artist_filter = self
                    .search_helper(
                        &artist,
                        &query,
                        SearchOptions {
                            valid_entry_types: vec!["artist", "album_artist"],
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await
                    .into_iter()
                    .map(|r| r.entry)
                    .collect_vec();
                return (query.to_owned(), artist_filter);
            }
        }
    }

    fn restrict_num_terms(&self, spellfix_results: Vec<SpellfixResult>) -> Vec<SpellfixResult> {
        let updated_results = spellfix_results
            .into_iter()
            .sorted_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            .take(MAX_TERMS)
            .sorted_by(|a, b| a.search.cmp(&b.search))
            .collect_vec();
        updated_results
    }

    async fn run_spellfix_query(
        &self,
        spellfix_query: &str,
        terms: &Vec<&str>,
        conn: &mut PoolConnection<Sqlite>,
    ) -> Vec<SpellfixResult> {
        let mut corrected = sqlx::query_as::<_, SpellfixResult>(&spellfix_query);
        for term in terms {
            corrected = corrected.bind(term);
        }
        let mut spellfix_results = corrected.fetch_all(conn).await.unwrap();
        let last = terms.last().unwrap().to_string();
        spellfix_results.push(SpellfixResult {
            word: last.to_owned(),
            search: last,
            score: 0.,
        });

        // Searching for an excessive amount of terms can be a big performance hit
        // If there are a lot of results, we can ignore lower-scored suggestions
        if spellfix_results.len() > MAX_TERMS {
            return self.restrict_num_terms(spellfix_results);
        }
        spellfix_results
    }

    async fn search_helper(
        &self,
        query: &str,
        original_query: &str,
        options: SearchOptions<'_>,
        artist_filter: Vec<String>,
    ) -> Vec<SearchResult> {
        let query = clean_query(query);
        if query.is_empty() {
            return vec![];
        }

        let mut conn = acquire_with_spellfix(&self.pool).await;
        let mut search_entries = self
            .run_search(
                &replace_ampersand(&query),
                original_query,
                HashMap::new(),
                &options,
                &artist_filter,
                &mut conn,
            )
            .await;

        if search_entries.len() == options.limit as usize {
            return self.convert_entries(search_entries);
        }
        let re = Regex::new(r"\s+").unwrap();
        let terms = re.split(&query).collect_vec();
        let spellfix_query = get_full_spellfix_query(&terms);

        let spellfix_results = self
            .run_spellfix_query(&spellfix_query, &terms, &mut conn)
            .await;

        let weights = spellfix_results
            .iter()
            .map(|s| (s.word.to_owned(), s.score))
            .collect::<HashMap<_, _>>();
        let corrected_search = correct_search(spellfix_results);

        if corrected_search.is_empty() {
            return vec![];
        }

        let rest = self
            .run_search(
                &corrected_search,
                original_query,
                weights.clone(),
                &options,
                &artist_filter,
                &mut conn,
            )
            .await;

        for mut r in &mut search_entries {
            r.weights = weights.clone();
        }
        search_entries.extend(rest);
        let search_entries = search_entries
            .into_iter()
            .unique_by(|r| {
                r.entry
                    .clone()
                    .replace(START_MATCH_TEXT, "")
                    .replace(END_MATCH_TEXT, "")
                    + "-"
                    + &r.entry_type
                    + &r.correlation_id.to_string()
            })
            .take(options.limit as usize)
            .collect_vec();

        return self.convert_entries(search_entries);
    }

    async fn run_search(
        &self,
        query: &str,
        original_query: &str,
        weights: HashMap<String, f32>,
        options: &SearchOptions<'_>,
        artist_filter: &Vec<String>,
        con: &mut PoolConnection<Sqlite>,
    ) -> Vec<SearchEntry> {
        let full_query = get_search_query(artist_filter, &options.valid_entry_types);

        let mut sql_query = sqlx::query(&full_query)
            .bind(options.start_highlight)
            .bind(options.end_highlight)
            .bind(query.to_owned())
            .bind(options.limit * 2)
            .bind(options.limit);

        for artist in artist_filter {
            sql_query = sql_query.bind(artist.to_owned());
        }
        for entry_type in &options.valid_entry_types {
            sql_query = sql_query.bind(entry_type.to_owned());
        }
        let res = sql_query
            .map(|row| SearchEntry {
                entry: row.try_get("entry").unwrap(),
                entry_type: row.try_get("entry_type").unwrap(),
                artist: row.try_get("artist").unwrap(),
                album: row.try_get("album").unwrap(),
                original_query: original_query.to_owned(),
                correlation_id: row.try_get("correlation_id").unwrap(),
                start_highlight: row.try_get("start_highlight").unwrap(),
                end_highlight: row.try_get("end_highlight").unwrap(),
                weights: weights.clone(),
            })
            .fetch_all(con)
            .await
            .unwrap();

        return res;
    }

    fn convert_entries(&self, mut search_entries: Vec<SearchEntry>) -> Vec<SearchResult> {
        search_entries.sort();
        let grouped = search_entries
            .into_iter()
            .group_by(|key| (key.get_formatted_entry(), key.get_description()))
            .into_iter()
            .map(|(key, group)| {
                let group = group.collect_vec();
                let first = group.get(0).unwrap();
                SearchResult {
                    entry: key.0,
                    entry_type: EntryType::from_str(&first.entry_type).unwrap(),
                    artist: first.artist.to_owned(),
                    description: key.1,
                    correlation_ids: group.iter().map(|v| v.correlation_id).collect(),
                }
            })
            .collect_vec();

        return grouped;
    }
}
