use regex::Regex;

/// Active filter for log searching
#[derive(Clone)]
pub struct ActiveFilter {
    pub query: String,
    pub query_lower: String,
    pub match_case: bool,
    pub regex: Option<Regex>,
    pub invert: bool,
}

impl ActiveFilter {
    pub fn matches(&self, text: &str) -> bool {
        let matched = if let Some(re) = &self.regex {
            re.is_match(text)
        } else if self.match_case {
            text.contains(&self.query)
        } else {
            // Optimization: Use pre-calculated lowercased query
            text.to_lowercase().contains(&self.query_lower)
        };
        if self.invert {
            !matched
        } else {
            matched
        }
    }
}

/// Builder for ActiveFilter
pub struct ActiveFilterBuilder {
    query: String,
    match_case: bool,
    use_regex: bool,
    invert: bool,
}

impl ActiveFilterBuilder {
    pub fn new(query: String) -> Self {
        Self {
            query,
            match_case: true,
            use_regex: false,
            invert: false,
        }
    }

    pub fn case_sensitive(mut self, yes: bool) -> Self {
        self.match_case = yes;
        self
    }

    pub fn regex(mut self, yes: bool) -> Self {
        self.use_regex = yes;
        self
    }

    pub fn invert(mut self, yes: bool) -> Self {
        self.invert = yes;
        self
    }

    pub fn build(self) -> Result<ActiveFilter, String> {
        let regex = if self.use_regex {
            Some(
                regex::RegexBuilder::new(&self.query)
                    .case_insensitive(!self.match_case)
                    .build()
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };

        let query_lower = if !self.match_case {
            self.query.to_lowercase()
        } else {
            String::new()
        };

        Ok(ActiveFilter {
            query: self.query,
            query_lower,
            match_case: self.match_case,
            regex,
            invert: self.invert,
        })
    }
}
