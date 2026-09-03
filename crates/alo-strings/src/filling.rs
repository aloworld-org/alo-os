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
//!
//! # A gap can hold a word rather than a value
//!
//! Almost every gap holds data — a path, a size, an identifier — and data is
//! not translated. One kind does not: an option a verb offers, which is a
//! string somebody translates dropped into the middle of another one
//! (`alo_capability::Offered`). [`Filling::and_said`] is the door for those,
//! and it exists so that a sentence cannot claim to be translated while a piece
//! of it is still English.
//!
//! Without it the failure is silent and is exactly the one [`crate::Said`] was
//! built to prevent: a German approval sentence with `on the left half of the
//! screen` in the middle of it would answer [`crate::Said::is_translated`] with
//! `true`, so nothing would mark it, nothing would count it, and the first
//! person to find out would be the person reading it.

use crate::said::{CameFrom, Said};

/// One gap's value, and where it came from when it came from the vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Given {
    /// The gap this fills.
    name: String,
    /// What goes in it.
    text: String,
    /// Where the text came from, when the text is itself a string somebody
    /// translates. `None` for data, which is the ordinary case.
    came_from: Option<CameFrom>,
}

/// The values that go into a sentence's gaps.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filling {
    /// One entry per name, in the order they were given.
    values: Vec<Given>,
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
        self.put(name.into(), value.into(), None);
        self
    }

    /// Another value, which is itself something this crate said.
    ///
    /// The text goes in as it would either way; what is kept beside it is where
    /// that text came from, so the answer built around it can say whether the
    /// *whole* sentence was translated. See this module's documentation for why
    /// a sentence is only as translated as its least translated piece.
    #[must_use]
    pub fn and_said(mut self, name: impl Into<String>, said: &Said) -> Self {
        self.put(
            name.into(),
            said.text().to_owned(),
            Some(said.came_from().clone()),
        );
        self
    }

    /// The value for this gap, if there is one.
    #[must_use]
    pub fn value(&self, name: &str) -> Option<&str> {
        self.given(name).map(|given| given.text.as_str())
    }

    /// Where the value for this gap came from, when it came from the vocabulary
    /// rather than from a caller holding a piece of data.
    #[must_use]
    pub fn came_from(&self, name: &str) -> Option<&CameFrom> {
        self.given(name).and_then(|given| given.came_from.as_ref())
    }

    /// Every name a value was given for.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.values
            .iter()
            .map(|given| given.name.as_str())
            .collect()
    }

    /// Whether nothing was given.
    #[must_use]
    pub fn is_nothing(&self) -> bool {
        self.values.is_empty()
    }

    /// One entry, by the name of the gap it fills.
    fn given(&self, name: &str) -> Option<&Given> {
        self.values.iter().find(|given| given.name == name)
    }

    /// Put a value in, replacing whatever was under that name.
    fn put(&mut self, name: String, text: String, came_from: Option<CameFrom>) {
        self.values.retain(|already| already.name != name);
        self.values.push(Given {
            name,
            text,
            came_from,
        });
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
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

    /// **Data carries no language and says so.** A path and a size came from
    /// somebody's machine, so there is nothing to have translated and nothing
    /// about them can make a sentence less translated than it is.
    #[test]
    fn a_value_that_is_data_came_from_nowhere() {
        let filling = Filling::of("path", "/home/ada/notes");
        assert_eq!(filling.value("path"), Some("/home/ada/notes"));
        assert_eq!(filling.came_from("path"), None);
        assert_eq!(filling.came_from("nothing-of-that-name"), None);
    }

    /// A gap filled with something this crate said keeps where that came from,
    /// which is what stops a sentence claiming to be translated while a piece
    /// of it is English.
    #[test]
    fn a_value_that_is_a_word_remembers_where_it_came_from() {
        let translated = Said::new(
            "auf der linken Bildschirmhälfte".to_owned(),
            CameFrom::Translation(crate::Language::written("de").unwrap()),
            Vec::new(),
        );
        let filling =
            Filling::of("application", "org.blender.Blender").and_said("where", &translated);
        assert_eq!(
            filling.value("where"),
            Some("auf der linken Bildschirmhälfte")
        );
        assert!(matches!(
            filling.came_from("where"),
            Some(&CameFrom::Translation(_))
        ));
        assert_eq!(filling.came_from("application"), None);
    }

    /// A name given twice keeps the last value **and the last provenance**. A
    /// filling that kept the old one would answer that a sentence was
    /// translated because a value nobody is looking at was.
    #[test]
    fn a_word_replaced_by_data_stops_carrying_a_language() {
        let said = Said::new("the left half".to_owned(), CameFrom::TheSource, Vec::new());
        let filling = Filling::of("where", "left_half").and_said("where", &said);
        assert_eq!(filling.came_from("where"), Some(&CameFrom::TheSource));

        let back = filling.and("where", "left_half");
        assert_eq!(back.value("where"), Some("left_half"));
        assert_eq!(back.came_from("where"), None);
        assert_eq!(back.names(), ["where"]);
    }
}
