pub(crate) mod queries;
mod result_score;
pub(crate) mod search_engine;
mod search_entry;
pub mod search_options;
pub(crate) mod search_result;
mod spellfix_result;

#[cfg(test)]
#[path = "./search_test.rs"]
mod search_test;
