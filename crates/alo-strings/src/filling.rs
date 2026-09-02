//! What goes into the gaps of a sentence.
//!
//! A filling is names to values, and the values are text that the caller has
//! already made. **This crate does not format numbers, dates or sizes**, and
//! that is a decision rather than an omission: how a number is written is the
//! region's business and not the language's, which `alo-appearance` settled
//! first — a person reading Swedish in Finland writes a time the Finnish way.
//! A filling that formatted for you would decide it in the wrong place, once,
//! for everybody.
//!
//! So `1 024` or `1,024` is chosen by whoever knows the region, and this crate
//! puts it where the sentence says it goes.

/// The values that go into a sentence's gaps.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filling {
    /// Name and value, one entry per name, in the order they were given.
    values: Vec<(String, String)>,
}

impl Filling {
    /// A sentence with nothing to fill in.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// One value, which is the common case.
    #[must_use]
    pub fn of(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self::nothing().and(name, value)
    }

    /// Another value. A name given twice keeps the last one, because a filling
    /// that held two values for one gap would fill it differently depending on
    /// which the reader looked at first.
    #[must_use]
    pub fn and(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        self.values.retain(|(already, _)| already != &name);
        self.values.push((name, value.into()));
        self
    }

    /// The value for this gap, if there is one.
    #[must_use]
    pub fn value(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(given, _)| given == name)
            .map(|(_, value)| value.as_str())
    }

    /// Every name a value was given for.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.values.iter().map(|(name, _)| name.as_str()).collect()
    }

    /// Whether nothing was given.
    #[must_use]
    pub fn is_nothing(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_nothing() {
        let filling = Filling::nothing();
        assert!(filling.is_nothing());
        assert_eq!(filling.value("path"), None);
    }

    #[test]
    fn values_are_kept_by_name() {
        let filling = Filling::of("path", "/home/ada/notes").and("bytes", "1 024");
        assert_eq!(filling.value("path"), Some("/home/ada/notes"));
        assert_eq!(filling.value("bytes"), Some("1 024"));
        assert_eq!(filling.names(), ["path", "bytes"]);
    }

    /// One gap, one value. A filling holding two would fill the same sentence
    /// differently depending on which entry was reached first.
    #[test]
    fn a_name_given_twice_keeps_the_last_value() {
        let filling = Filling::of("path", "/tmp/one").and("path", "/tmp/two");
        assert_eq!(filling.value("path"), Some("/tmp/two"));
        assert_eq!(filling.names(), ["path"]);
    }

    /// The value is text the caller made. Nothing here turns a number into
    /// text, because how a number is written belongs to the region rather than
    /// to the language.
    #[test]
    fn a_value_is_text_the_caller_already_made() {
        let swedish_in_finland = Filling::of("bytes", "1 024");
        let english = Filling::of("bytes", "1,024");
        assert_ne!(swedish_in_finland, english);
    }
}
