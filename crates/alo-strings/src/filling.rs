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
//!
//! # And a gap can hold more than one word
//!
//! A chord is the case: `Super+Bild ↑` is the notation every desktop writes a
//! shortcut in, holding a name for each key, and no translator is ever handed
//! it whole. There is no single place its words came from, so
//! [`Filling::and_composed`] takes the list — and *where did the words in this
//! gap come from* has a list for an answer everywhere, which is what lets a
//! clause with a clause inside it carry both.
//!
//! The gap that is **sometimes** a word needs nothing new. A destination that
//! is a host somebody's verb named, a key that prints `Q`, a window known only
//! by its identifier: each is a word or it is data, and a caller that knows
//! which puts it in through [`Filling::and_said`] or through [`Filling::and`].

use crate::said::{CameFrom, Said};

/// One gap's value, and where it came from when it came from the vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Given {
    /// The gap this fills.
    name: String,
    /// What goes in it.
    text: String,
    /// Where the words in the text came from, one entry for each piece of it
    /// that is itself a string somebody translates. Empty for data, which is
    /// the ordinary case; one entry for a clause the vocabulary said; more than
    /// one for a clause composed out of several.
    came_from: Vec<CameFrom>,
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
        self.put(name.into(), value.into(), Vec::new());
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
        self.put(name.into(), said.text().to_owned(), everywhere_from(said));
        self
    }

    /// Another value, composed out of the things this crate said to make it.
    ///
    /// The text is the composer's — a `+` between the names of the keys in a
    /// chord is notation and not a string — and what is kept beside it is where
    /// **every** piece's words came from. So a sentence with a chord in it is
    /// only as translated as the least translated key in that chord, which is
    /// the rule [`Filling::and_said`] carries for one piece, carried for
    /// several.
    ///
    /// **Passing no pieces says there were no words in it**, which is
    /// [`Filling::and`] said a longer way. A caller composing out of a list
    /// that turned out to be empty gets the honest answer rather than a
    /// provenance invented to fill the argument.
    #[must_use]
    pub fn and_composed(
        mut self,
        name: impl Into<String>,
        text: impl Into<String>,
        of: &[Said],
    ) -> Self {
        let came_from = of.iter().flat_map(everywhere_from).collect();
        self.put(name.into(), text.into(), came_from);
        self
    }

    /// The value for this gap, if there is one.
    #[must_use]
    pub fn value(&self, name: &str) -> Option<&str> {
        self.given(name).map(|given| given.text.as_str())
    }

    /// Where the words in this gap's value came from, one entry for each piece
    /// of it that came from the vocabulary rather than from a caller holding a
    /// piece of data.
    ///
    /// Empty when the gap holds data, and empty for a gap nothing was given
    /// for — in both cases for the same reason, which is that there are no
    /// words in it whose language anybody could be wrong about.
    #[must_use]
    pub fn came_from(&self, name: &str) -> &[CameFrom] {
        match self.given(name) {
            Some(given) => &given.came_from,
            None => &[],
        }
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
    fn put(&mut self, name: String, text: String, came_from: Vec<CameFrom>) {
        self.values.retain(|already| already.name != name);
        self.values.push(Given {
            name,
            text,
            came_from,
        });
    }
}

/// Everywhere the words of one answer came from: the sentence itself, and every
/// word already put into one of its gaps.
///
/// The second half is what makes the rule hold at any depth. A clause with a
/// clause inside it stopped being hypothetical as soon as two crates worded
/// something between them — `alo-egress` puts a place inside a refusal, and a
/// shell will one day put that refusal inside a line of its own — and keeping
/// only the outer provenance would report the innermost English as translated.
/// That is the failure this whole file exists to prevent, one level further in.
fn everywhere_from(said: &Said) -> Vec<CameFrom> {
    let mut all = Vec::with_capacity(1 + said.gaps_came_from().len());
    all.push(said.came_from().clone());
    all.extend(said.gaps_came_from().iter().cloned());
    all
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
        assert!(filling.came_from("path").is_empty());
        assert!(filling.came_from("nothing-of-that-name").is_empty());
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
            [CameFrom::Translation(_)]
        ));
        assert!(filling.came_from("application").is_empty());
    }

    /// A name given twice keeps the last value **and the last provenance**. A
    /// filling that kept the old one would answer that a sentence was
    /// translated because a value nobody is looking at was.
    #[test]
    fn a_word_replaced_by_data_stops_carrying_a_language() {
        let said = Said::new("the left half".to_owned(), CameFrom::TheSource, Vec::new());
        let filling = Filling::of("where", "left_half").and_said("where", &said);
        assert_eq!(filling.came_from("where"), [CameFrom::TheSource]);

        let back = filling.and("where", "left_half");
        assert_eq!(back.value("where"), Some("left_half"));
        assert!(back.came_from("where").is_empty());
        assert_eq!(back.names(), ["where"]);
    }

    /// **A clause with a clause inside it carries both.** Keeping only the
    /// outer one would say a German refusal was German while the place named in
    /// the middle of it was still English — the same failure as filling the gap
    /// with a bare `String`, one level further in.
    #[test]
    fn a_word_put_into_a_gap_brings_the_words_that_were_put_into_it() {
        let german = CameFrom::Translation(crate::Language::written("de").unwrap());
        let clause = Said::new(
            "von jemandem, der nicht gesagt hat, wo er läuft".to_owned(),
            german.clone(),
            Vec::new(),
        );
        let refusal = Said::new(
            "dieser Rechner lässt nichts hinaus, und von jemandem, der nicht gesagt hat, wo er \
             läuft ist anderswo"
                .to_owned(),
            german.clone(),
            Vec::new(),
        )
        .filled_with(vec![CameFrom::TheSource]);

        // The clause alone brings one provenance; the refusal brings its own
        // and the untranslated place already inside it.
        let one = Filling::nothing().and_said("what", &clause);
        assert_eq!(one.came_from("what"), std::slice::from_ref(&german));
        let both = Filling::nothing().and_said("what", &refusal);
        assert_eq!(both.came_from("what"), [german, CameFrom::TheSource]);
    }

    /// **A composed value is as translated as its least translated piece**, and
    /// a chord is the case it exists for: the `+` is notation, and each name
    /// beside it is a string of its own.
    #[test]
    fn a_composed_value_carries_where_every_piece_came_from() {
        let german = CameFrom::Translation(crate::Language::written("de").unwrap());
        let held = Said::new("Super".to_owned(), german.clone(), Vec::new());
        let pressed = Said::new("Page Up".to_owned(), CameFrom::TheSource, Vec::new());
        let filling = Filling::nothing().and_composed("chord", "Super+Page Up", &[held, pressed]);
        assert_eq!(filling.value("chord"), Some("Super+Page Up"));
        assert_eq!(filling.came_from("chord"), [german, CameFrom::TheSource]);
    }

    /// **A composed value made of no words is data.** A caller whose list of
    /// pieces turned out empty is told the truth about it rather than handed a
    /// provenance invented to fill the argument.
    #[test]
    fn a_composed_value_with_no_words_in_it_is_data() {
        let filling = Filling::nothing().and_composed("chord", "Super+Q", &[]);
        assert_eq!(filling.value("chord"), Some("Super+Q"));
        assert!(filling.came_from("chord").is_empty());
    }
}
