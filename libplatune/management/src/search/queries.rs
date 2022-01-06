use itertools::Itertools;
use regex::Regex;

use crate::consts::{END_MATCH_TEXT, START_MATCH_TEXT};

use super::spellfix_result::SpellfixResult;

pub(crate) fn get_search_query(artist_filter: &[String], allowed_entry_types: &[&str]) -> String {
    let num_base_args = 5;
    let num_artists = artist_filter.len();
    let artist_select =
        "CASE entry_type WHEN 'song' THEN ar.artist_name WHEN 'album' THEN aa.album_artist_name ELSE NULL END";

    let artist_filter_clause = if artist_filter.is_empty() {
        "".to_owned()
    } else {
        //  WHERE clause with parameterized bindings for each artist in the list
        let start = num_base_args + 1;
        let artist_list = generate_parameterized_bindings(start, num_artists);

        format!("WHERE {} in ({})", artist_select, artist_list)
    };

    let type_filter = if allowed_entry_types.is_empty() {
        "".to_owned()
    } else {
        // AND clause for the search_index search if allowed_entry_types was supplied
        let start = num_base_args + num_artists + 1;
        let in_list = generate_parameterized_bindings(start, allowed_entry_types.len());

        format!("AND entry_type in ({})", &in_list)
    };

    let full_query = format!("
    WITH CTE AS (
        SELECT DISTINCT entry, entry_type, rank, $1 start_highlight, $2 end_highlight, assoc_id correlation_id,
        {0} artist,
        al2.album_name album,
        -- Partition results to prevent returning the same value for artist and album artist
        -- Only return the album artist if there is no equivalent artist entry
        -- Also ensure multiple songs on different albums by the same artist are returned and 
        -- if there are multiple results only differing by and/&, both are returned
        ROW_NUMBER() OVER (PARTITION BY 
            entry_value, 
            {0}, 
            CASE entry_type WHEN 'song' THEN 1 WHEN 'album' THEN 2 WHEN 'tag' THEN 3 ELSE 4 END,
            CASE entry_type WHEN 'song' THEN s.song_title + s.album_id WHEN 'album' THEN al.album_name WHEN 'artist' THEN ar2.artist_name WHEN 'album_artist' THEN aa2.album_artist_name END
            ORDER BY entry_type DESC) row_num
        FROM (SELECT entry_type, assoc_id, entry_value, highlight(search_index, 0, '{3}', '{4}') entry, rank FROM search_index WHERE entry_value match $3 {2}) search_query
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
    LIMIT $5;", artist_select, artist_filter_clause, type_filter, START_MATCH_TEXT, END_MATCH_TEXT);

    full_query
}

pub(crate) fn get_full_spellfix_query(terms: &[&str]) -> String {
    // Union all queries together to avoid multiple trips to the database
    let full_query = terms
        .iter()
        .enumerate()
        .map(|(i, _)| get_spellfix_query(i + 1))
        .collect_vec()
        .join(" UNION ALL ");

    full_query
}

pub(crate) fn combine_spellfix_results(spellfix_results: Vec<SpellfixResult>) -> String {
    // Group spellfix results by their correpsonding search input
    let spellfix_groups = spellfix_results
        .into_iter()
        .group_by(|row| row.search.to_owned())
        .into_iter()
        .map(|(_, val)| val.map(|v| v.word + " ").collect_vec())
        .collect_vec();

    // Generate all combinations of search terms, preserving the order of the inputs
    let mut search_combinations = spellfix_groups
        .into_iter()
        .fold(vec!["".to_owned()], |a, b| {
            a.into_iter()
                .flat_map(|x| b.iter().map(move |y| x.clone() + y))
                .collect_vec()
        })
        .into_iter()
        .map(|s| replace_ampersand(&s))
        .unique();

    // Join all combinations togeter into one search string
    let mut combined_query = search_combinations.join("OR ").trim().to_owned();
    combined_query = replace_special_chars(&combined_query);

    combined_query
}

pub(crate) fn clean_query(query: &str) -> String {
    let query = replace_special_chars(query);
    if query.is_empty() || query.ends_with('*') {
        return query;
    }
    // Add wildcard to the end to do a prefix search
    query + "*"
}

pub(crate) fn replace_ampersand(string: &str) -> String {
    string.replace(" & ", " and ").replace("&", " ")
}

fn replace_special_chars(query: &str) -> String {
    // Replace all special characters with whitespace because they cause sqlite to error
    let special_chars = Regex::new(r"[^A-Za-z0-9&\*\s]").unwrap();
    return special_chars.replace_all(query, " ").trim().to_owned();
}

fn generate_parameterized_bindings(start: usize, count: usize) -> String {
    (start..start + count)
        .map(|i| "$".to_owned() + &i.to_string())
        .collect_vec()
        .join(",")
}

fn get_spellfix_query(index: usize) -> String {
    // These constants were obtained very unscientifically through trial and error
    let word_normalization_factor = 3.5;
    let max_score = 50;

    // If results appear more often, rate them higher
    let score_clause = "score * CASE WHEN cnt IS NULL THEN 1.0 ELSE 5.0 / MIN(cnt, 5) END";

    // If the word contains whitespace, it must be a special entry we added explicitly like an abbreviation
    // Need to calculate these two cases differently and attempt to normalize them
    // Divide by word length to normalize total error over number of letters
    // Note: multiplying by 1.0 is a way to coerce an int to a float
    format!(
        "
        --beginsql
        SELECT * FROM (
            SELECT DISTINCT word, ${0} search, {3} score FROM (
                SELECT * FROM (
                    SELECT word, CASE 
                        WHEN word like '% %' then (distance * 1.0 / (LENGTH(word) - LENGTH(REPLACE(word, ' ', '')))) * {1}
                        ELSE EDITDIST3(${0}, word) * 1.0 / LENGTH(word) END score
                    FROM search_spellfix
                    WHERE word match REPLACE(${0}, '*', '')
                ) 
                UNION ALL
                -- Sometimes the match function is too conservative for our purposes
                -- so we need to include results based on the raw distance as well
                SELECT * FROM (
                    SELECT word, EDITDIST3(${0}, word) * 1.0 / LENGTH(word) score 
                    FROM search_spellfix
                    WHERE EDITDIST3(REPLACE(${0}, '*', ''), word) * 1.0 / LENGTH(word) <= {2}
                )
                
            )
            LEFT OUTER JOIN search_vocab sv ON sv.term = word
            WHERE score <= {2}
            ORDER BY {3}
            LIMIT 5
        )
        --endsql
        ",
        index,
        word_normalization_factor,
        max_score,
        score_clause
    )
}
