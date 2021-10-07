use std::{collections::HashMap, str::FromStr};

use super::{search_options::SearchOptions, search_result::SearchResult};
use crate::{
    entry_type::EntryType,
    search::{search_entry::SearchEntry, spellfix_result::SpellfixResult},
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
const START_MATCH_TEXT: &str = "{startmatch}";
const END_MATCH_TEXT: &str = "{endmatch}";

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
                            restrict_entry_type: vec!["artist", "album_artist"],
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

    fn replace_ampersand(&self, string: &str) -> String {
        string.replace(" & ", " and ").replace("&", " ")
    }

    fn replace_special_chars(&self, query: &str) -> String {
        // Replace all special characters with whitespace because they cause sqlite to error
        let special_chars = Regex::new(r"[^A-Za-z0-9&\*\s]").unwrap();
        return special_chars.replace_all(query, " ").trim().to_owned();
    }

    fn clean_query(&self, query: &str) -> String {
        let query = self.replace_special_chars(query);
        if query.is_empty() || query.ends_with("*") {
            return query;
        }
        // Add wildcard to the end to do a prefix search
        return query + "*";
    }

    async fn search_helper(
        &self,
        query: &str,
        original_query: &str,
        options: SearchOptions<'_>,
        artist_filter: Vec<String>,
    ) -> Vec<SearchResult> {
        let query = self.clean_query(query);
        if query.is_empty() {
            return vec![];
        }

        let mut conn = acquire_with_spellfix(&self.pool).await;
        let mut search_entries = self
            .run_search(
                &(self.replace_ampersand(&query)),
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
        let last = terms.last().unwrap().to_owned().to_owned();
        let spellfix_query = terms
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let score_clause = format!("case 
            when word like '% %' then (distance * 1.0 / (length(word) - length(replace(word, ' ', '')))) * 3.5 
            else editdist3(${0}, word) * 1.0 / length(word) end", i + 1);
            format!(
                "
                select * from (
                    select distinct word, ${0} search, {1} score
                    from search_spellfix 
                    where word match replace(${0}, '*', '')
                    and ({1}) <= 50
                    order by {1}
                    limit 5
                )
                ",
                i + 1, score_clause
            )
        })
        .collect_vec()
        .join(" union all ");

        let mut corrected = sqlx::query_as::<_, SpellfixResult>(&spellfix_query);
        for term in terms {
            corrected = corrected.bind(term);
        }
        let mut spellfix_res = corrected.fetch_all(&mut conn).await.unwrap();

        spellfix_res.push(SpellfixResult {
            word: last.to_owned(),
            search: last,
            score: 0.,
        });

        // Searching for an excessive amount of terms can be a big performance hit
        // If there are a lot of results, we can ignore lower-scored suggestions
        if spellfix_res.len() > MAX_TERMS {
            spellfix_res = spellfix_res
                .into_iter()
                .sorted_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
                .take(20)
                .sorted_by(|a, b| a.search.cmp(&b.search))
                .collect_vec();
        }

        let weights = spellfix_res
            .iter()
            .map(|s| (s.word.to_owned(), s.score))
            .collect::<HashMap<_, _>>();
        let mut corrected_search = spellfix_res
            .into_iter()
            .group_by(|row| row.search.to_owned())
            .into_iter()
            .map(|(_, val)| val.map(|v| v.word + " ").collect_vec())
            .fold(vec!["".to_owned()], |a, b| {
                a.into_iter()
                    .flat_map(|x| b.iter().map(move |y| x.clone() + &y))
                    .collect_vec()
            })
            .iter()
            .map(|s| self.replace_ampersand(s))
            .unique()
            .join("OR ")
            .trim()
            .to_owned();
        if corrected_search.is_empty() {
            return vec![];
        }
        corrected_search = self.replace_special_chars(&corrected_search);

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
        let artist_select = "CASE entry_type WHEN 'song' THEN ar.artist_name WHEN 'album' THEN aa.album_artist_name ELSE NULL END";
        let mut artist_filter_clause = "".to_owned();
        let num_base_args = 5;
        let mut num_extra_args = 0;
        if artist_filter.len() > 0 {
            let start = num_base_args + num_extra_args + 1;
            let artist_list = (start..start + artist_filter.len())
                .map(|i| "$".to_owned() + &i.to_string())
                .collect_vec()
                .join(",");
            artist_filter_clause = format!("WHERE {} in ({})", artist_select, artist_list);
            num_extra_args += artist_filter.len();
        }
        let mut type_filter = "".to_owned();
        if !options.restrict_entry_type.is_empty() {
            let start = num_base_args + num_extra_args + 1;
            let in_list = (start..start + options.restrict_entry_type.len())
                .map(|i| "$".to_owned() + &i.to_string())
                .collect_vec()
                .join(",");

            type_filter = format!("AND entry_type in ({})", &in_list);
        }
        let full_query = format!("
    WITH CTE AS (
        SELECT DISTINCT entry, entry_type, rank, $1 start_highlight, $2 end_highlight, assoc_id correlation_id,
        {0} artist,
        al2.album_name album,
        ROW_NUMBER() OVER (PARTITION BY 
            entry_value, 
            {0}, 
            CASE entry_type WHEN 'song' THEN 1 WHEN 'album' THEN 2 WHEN 'tag' THEN 3 ELSE 4 END,
            CASE entry_type WHEN 'song' THEN s.song_title + s.album_id WHEN 'album' THEN al.album_name WHEN 'artist' THEN ar2.artist_name WHEN 'album_artist' THEN aa2.album_artist_name END
            ORDER BY entry_type DESC) row_num
        FROM (select entry_type, assoc_id, entry_value, highlight(search_index, 0, '{3}', '{4}') entry, rank from search_index where entry_value match $3 {2}) a
        LEFT OUTER JOIN song s on s.song_id = assoc_id
        LEFT OUTER JOIN artist ar on ar.artist_id = s.artist_id
        LEFT OUTER JOIN album al on al.album_id = assoc_id
        LEFT OUTER JOIN album al2 on al2.album_id = s.album_id
        LEFT OUTER JOIN album_artist aa on aa.album_artist_id = al.album_artist_id
        LEFT OUTER JOIN artist ar2 on ar2.artist_id = assoc_id
        LEFT OUTER JOIN album_artist aa2 on aa2.album_artist_id = assoc_id
        {1}
        ORDER BY rank
        LIMIT $4
    )
    SELECT entry, entry_type, artist, album, correlation_id, start_highlight, end_highlight FROM cte
    WHERE row_num = 1
    ORDER BY rank
    LIMIT $5", artist_select, artist_filter_clause, type_filter, START_MATCH_TEXT, END_MATCH_TEXT);
        let mut sql_query = sqlx::query(&full_query)
            .bind(options.start_highlight)
            .bind(options.end_highlight)
            .bind(query.to_owned())
            .bind(options.limit * 2)
            .bind(options.limit);

        for artist in artist_filter {
            sql_query = sql_query.bind(artist.to_owned());
        }
        for entry_type in &options.restrict_entry_type {
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

    fn convert_entries(&self, mut res: Vec<SearchEntry>) -> Vec<SearchResult> {
        res.sort();
        let grouped = res
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
