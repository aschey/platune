pub struct SearchOptions<'a> {
    pub start_highlight: &'a str,
    pub end_highlight: &'a str,
    pub limit: i32,
    pub restrict_entry_type: Vec<&'a str>,
}

impl<'a> Default for SearchOptions<'a> {
    fn default() -> Self {
        Self {
            start_highlight: "",
            end_highlight: "",
            limit: 10,
            restrict_entry_type: vec![],
        }
    }
}
