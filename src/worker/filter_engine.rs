use crate::worker::index::filter::ActiveFilter;

/// Filter engine that manages filtering state and logic
#[allow(dead_code)]
pub struct FilterEngine {
    active_filter: Option<ActiveFilter>,
    is_enabled: bool,
}

#[allow(dead_code)]

impl FilterEngine {
    pub fn new() -> Self {
        Self {
            active_filter: None,
            is_enabled: false,
        }
    }

    /// Sets the active filter
    pub fn set_filter(&mut self, filter: Option<ActiveFilter>) {
        self.active_filter = filter;
        self.is_enabled = self.active_filter.is_some();
    }

    /// Clears the filter
    pub fn clear(&mut self) {
        self.active_filter = None;
        self.is_enabled = false;
    }

    /// Checks if filtering is currently enabled
    pub fn is_filtering(&self) -> bool {
        self.is_enabled
    }

    /// Returns a reference to the active filter
    pub fn active_filter(&self) -> Option<&ActiveFilter> {
        self.active_filter.as_ref()
    }

    /// Checks if the given text should be included based on the active filter
    pub fn should_include(&self, text: &str) -> bool {
        if !self.is_enabled {
            return false;
        }
        self.active_filter
            .as_ref()
            .map(|f| f.matches(text))
            .unwrap_or(false)
    }
}

impl Default for FilterEngine {
    fn default() -> Self {
        Self::new()
    }
}
