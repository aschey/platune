use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use super::{
    queries::{clean_query, combine_spellfix_results, get_search_query, replace_ampersand},
    search_options::SearchOptions,
    search_result::SearchResult,
};
use crate::{
    consts::{END_MATCH_TEXT, START_MATCH_TEXT},
    db_error::DbError,
    entry_type::EntryType,
    search::{
        queries::get_full_spellfix_query, search_entry::SearchEntry,
        spellfix_result::SpellfixResult,
    },
};
use concread::arcache::{ARCache, ARCacheBuilder};
use itertools::Itertools;
use regex::Regex;
use sqlx::{pool::PoolConnection, Pool, Row, Sqlite};
use tap::Tap;
use tracing::{info, warn};

#[derive(Clone)]
pub(crate) struct SearchEngine {
    pool: Pool<Sqlite>,
    cache: Arc<ARCache<String, Vec<SearchResult>>>,
}

const MAX_TERMS: usize = 20;

impl SearchEngine {
    pub(crate) fn new(pool: Pool<Sqlite>) -> Self {
        let builder = ARCacheBuilder::new().set_size(25, 25);
        let cache = Arc::new(builder.build().unwrap());
        Self { pool, cache }
    }

    pub(crate) async fn search(
        &self,
        query: &str,
        options: SearchOptions<'_>,
    ) -> Result<Vec<SearchResult>, DbError> {
        let query = query.trim().to_lowercase();
        let res = match self.cache.read().get(&query) {
            Some(val) => {
                info!("Using cache for search {}", query);
                val.to_owned()
            }
            None => {
                let start = Instant::now();
                // Parse out artist filter if it was supplied
                let (adj_query, artist_filter) = self.split_artist_filter(&query).await?;

                let res = self
                    .search_helper(&adj_query, &adj_query, options, artist_filter)
                    .await?;
                let time_taken = start.elapsed();
                if time_taken > Duration::from_millis(50) {
                    warn!("Search for {query} was slow: {time_taken:?}. Caching result");
                    let mut write_tx = self.cache.write();
                    write_tx.insert(query.to_owned(), res.clone());
                    write_tx.commit();
                } else {
                    info!("Search for {query} finished in {time_taken:?}");
                }

                return Ok(res);
            }
        };

        Ok(res)
    }

    pub(crate) fn clear_cache(&self) {
        let mut write_tx = self.cache.write();
        write_tx.clear();
        write_tx.commit();
    }

    async fn split_artist_filter(&self, query: &str) -> Result<(String, Vec<String>), DbError> {
        let artist_split = query.split("artist:").collect_vec();

        match *artist_split.as_slice() {
            [] => Ok(("".to_owned(), vec![])),
            [query] => Ok((query.to_owned(), vec![])),
            [query, artist, ..] => {
                let artist_filter = self
                    .search_helper(
                        artist,
                        query,
                        SearchOptions {
                            valid_entry_types: vec!["artist", "album_artist"],
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await?
                    .into_iter()
                    .map(|r| r.entry)
                    .collect_vec();
                Ok((query.to_owned(), artist_filter))
            }
        }
    }

    fn restrict_num_terms(&self, spellfix_results: Vec<SpellfixResult>) -> Vec<SpellfixResult> {
        spellfix_results
            .into_iter()
            .sorted_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .take(MAX_TERMS)
            .sorted_by(|a, b| a.search.cmp(&b.search))
            .collect_vec()
    }

    async fn run_spellfix_query(
        &self,
        spellfix_query: &str,
        terms: &[&str],
        conn: &mut PoolConnection<Sqlite>,
    ) -> Result<Vec<SpellfixResult>, DbError> {
        let mut corrected = sqlx::query_as::<_, SpellfixResult>(spellfix_query);
        for term in terms {
            corrected = corrected.bind(term);
        }
        let mut spellfix_results = corrected
            .fetch_all(&mut **conn)
            .await
            .map_err(|e| DbError::DbError(format!("{e:?}")))?;
        if let Some(last) = terms.last() {
            let last = last.to_string();
            spellfix_results.push(SpellfixResult {
                word: last.to_owned(),
                search: last,
                score: 0.,
            });
        }

        // Searching for an excessive amount of terms can be a big performance hit
        // If there are a lot of results, we can ignore lower-scored suggestions
        if spellfix_results.len() > MAX_TERMS {
            return Ok(self.restrict_num_terms(spellfix_results));
        }
        Ok(spellfix_results)
    }

    async fn search_helper(
        &self,
        query: &str,
        original_query: &str,
        options: SearchOptions<'_>,
        artist_filter: Vec<String>,
    ) -> Result<Vec<SearchResult>, DbError> {
        let query = clean_query(query);
        if query.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| DbError::DbError(format!("{e:?}")))?;
        let mut search_entries = self
            .run_search(
                &replace_ampersand(&query),
                original_query,
                HashMap::new(),
                &options,
                &artist_filter,
                &mut conn,
            )
            .await?;

        // Already generated enough results, don't need to attempt spellfix
        if search_entries.len() == options.limit as usize {
            return Ok(self.convert_entries(search_entries));
        }

        let re = Regex::new(r"\s+").unwrap();
        let terms = re.split(&query).collect_vec();
        let spellfix_query = get_full_spellfix_query(&terms);

        let spellfix_results = self
            .run_spellfix_query(&spellfix_query, &terms, &mut conn)
            .await?;

        let weights = spellfix_results
            .iter()
            .map(|s| (s.word.to_owned(), s.score))
            .collect::<HashMap<_, _>>();
        let combined_results = combine_spellfix_results(spellfix_results);

        if combined_results.is_empty() {
            return Ok(vec![]);
        }

        let rest = self
            .run_search(
                &combined_results,
                original_query,
                weights.clone(),
                &options,
                &artist_filter,
                &mut conn,
            )
            .await?;

        for r in &mut search_entries {
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

        Ok(self.convert_entries(search_entries))
    }

    async fn run_search(
        &self,
        query: &str,
        original_query: &str,
        weights: HashMap<String, f32>,
        options: &SearchOptions<'_>,
        artist_filter: &[String],
        con: &mut PoolConnection<Sqlite>,
    ) -> Result<Vec<SearchEntry>, DbError> {
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

        sql_query
            .map(|row: <sqlx::Sqlite as sqlx::Database>::Row| SearchEntry {
                entry: row.try_get("entry").unwrap_or_default(),
                entry_type: row.try_get("entry_type").unwrap_or_default(),
                artist: row.try_get("artist").unwrap_or_default(),
                album: row.try_get("album").unwrap_or_default(),
                original_query: original_query.to_owned(),
                correlation_id: row.try_get("correlation_id").unwrap_or_default(),
                start_highlight: row.try_get("start_highlight").unwrap_or_default(),
                end_highlight: row.try_get("end_highlight").unwrap_or_default(),
                weights: weights.clone(),
            })
            .fetch_all(&mut **con)
            .await
            .map_err(|e| DbError::DbError(format!("{e:?}")))
    }

    fn convert_entries(&self, search_entries: Vec<SearchEntry>) -> Vec<SearchResult> {
        search_entries
            .tap_mut(|s| s.sort())
            .into_iter()
            .group_by(|key| (key.get_formatted_entry(), key.get_description()))
            .into_iter()
            .map(|(key, group)| {
                let group = group.collect_vec();
                let first = &group[0];
                SearchResult {
                    entry: key.0,
                    entry_type: EntryType::from_str(&first.entry_type).unwrap(),
                    artist: first.artist.to_owned(),
                    description: key.1,
                    correlation_ids: group.iter().map(|v| v.correlation_id).collect(),
                }
            })
            .collect_vec()
    }
}
