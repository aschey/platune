#[derive(Debug, sqlx::FromRow)]
pub(crate) struct SpellfixResult {
    pub(crate) word: String,
    pub(crate) search: String,
    pub(crate) score: f32,
}
