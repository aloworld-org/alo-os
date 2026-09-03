//! What one invocation offered: the moment, and the three things that reach an
//! agent at it.
//!
//! # It can only be made, never read back
//!
//! This type has one constructor, it takes the moment the person pressed the
//! key, and **nothing in this crate deserialises** — there is no serde
//! dependency at all, which is why. A context that could be read back off a
//! disk would be a context that exists without an invocation having made one,
//! and *context is offered, never watched* would become a convention rather
//! than a shape. It is `alo_capability::Call`'s rule met one step earlier: the
//! deciding side of alo OS does not read things back.
//!
//! It is not `Copy` and not `Clone` either, and [`crate::Turn`] takes it by
//! value. One invocation, one turn — the compiler's version of ADR 0001 §4's
//! *and only for that turn*, and there is a `compile_fail` doctest on
//! [`crate::Turn::beginning`] asserting it.
//!
//! # An invocation that offered nothing is still an invocation
//!
//! [`Context::at_invocation`] is a whole context with nothing in it, and every
//! part is added deliberately. A person who presses the key on an empty desktop
//! has offered nothing, and that is a fact worth being able to state — *nothing
//! from your screen was offered* is a row a person reads, rather than an empty
//! space they have to interpret.
//!
//! # What a person can see
//!
//! [`Context::shown`] is the visible half of the promise. A rule nobody can
//! check is a promise, and a person who cannot see what they are offering
//! cannot tell the difference between a system that offers three things at
//! invocation and one that watches everything all day.

use std::time::SystemTime;

use alo_strings::{Filling, Said, Strings};

use crate::document::Document;
use crate::focused::Focused;
use crate::selection::Selection;
use crate::words;

/// What reached an agent at the moment it was invoked.
///
/// Deliberately not `Clone`: a context that can be copied is a context that can
/// serve a second turn.
#[derive(Debug, PartialEq, Eq)]
pub struct Context {
    /// The moment the person invoked the agent. Everything a turn asks about
    /// time is asked at this moment rather than at a fresh reading of a clock.
    at: SystemTime,
    /// The window that was in front of them.
    window: Option<Focused>,
    /// What they had selected.
    selection: Option<Selection>,
    /// The document they had open.
    document: Option<Document>,
}

impl Context {
    /// An invocation, with nothing offered yet.
    ///
    /// The moment is passed in rather than read, which is item 1's rule
    /// reaching this crate: nothing here reads the clock, so a turn's expiry is
    /// arithmetic a test can do rather than a wait, and the daemon and the
    /// shell cannot disagree about when the person pressed the key.
    #[must_use]
    pub fn at_invocation(at: SystemTime) -> Self {
        Self {
            at,
            window: None,
            selection: None,
            document: None,
        }
    }

    /// And the window that was in front of them.
    #[must_use]
    pub fn and_window(mut self, window: Focused) -> Self {
        self.window = Some(window);
        self
    }

    /// And what they had selected.
    #[must_use]
    pub fn and_selection(mut self, selection: Selection) -> Self {
        self.selection = Some(selection);
        self
    }

    /// And the document they had open.
    #[must_use]
    pub fn and_document(mut self, document: Document) -> Self {
        self.document = Some(document);
        self
    }

    /// The moment the person invoked the agent.
    #[must_use]
    pub fn at(&self) -> SystemTime {
        self.at
    }

    /// The window that was in front of them.
    #[must_use]
    pub fn window(&self) -> Option<&Focused> {
        self.window.as_ref()
    }

    /// What they had selected.
    #[must_use]
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    /// The document they had open — the only part of this that grants
    /// anything.
    #[must_use]
    pub fn document(&self) -> Option<&Document> {
        self.document.as_ref()
    }

    /// Whether the invocation offered nothing at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.window.is_none() && self.selection.is_none() && self.document.is_none()
    }

    /// What a person is shown of what they offered: one row per part, and one
    /// row saying nothing when there was nothing.
    ///
    /// Rows rather than one sentence with a list in it, which is
    /// `alo-shortcuts`' rule: the separator and the conjunction are not
    /// punctuation a program can pick, and a language that writes them
    /// differently would be handed a sentence assembled by a machine that does
    /// not know its grammar.
    ///
    /// The document comes first because it is the one that grants something.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> Vec<Said> {
        if self.is_empty() {
            return vec![strings.say(&words::NOTHING_OFFERED.key(), &Filling::nothing())];
        }
        let mut rows = Vec::new();
        if let Some(document) = &self.document {
            rows.push(strings.say(
                &words::THE_DOCUMENT.key(),
                &Filling::of("document", document.path().to_string_lossy().into_owned()),
            ));
        }
        if self.selection.is_some() {
            rows.push(strings.say(&words::THE_SELECTION.key(), &Filling::nothing()));
        }
        if let Some(window) = &self.window {
            rows.push(strings.say(
                &words::THE_WINDOW.key(),
                &window.fills("window", Filling::nothing(), strings),
            ));
        }
        rows
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{everything_offered, in_english, march, noon, translated};

    #[test]
    fn a_context_is_the_moment_and_what_was_offered_at_it() {
        let context = everything_offered();
        assert_eq!(context.at(), noon());
        assert_eq!(context.document().map(Document::path), Some(march()));
        assert_eq!(
            context.window().map(Focused::application),
            Some("org.blender.Blender")
        );
        assert_eq!(
            context.selection().map(Selection::text),
            Some("the invoice from March")
        );
        assert!(!context.is_empty());
    }

    /// **An invocation that offered nothing is still an invocation**, and it
    /// says so in a row a person reads rather than in an empty space they have
    /// to interpret.
    #[test]
    fn an_invocation_that_offered_nothing_says_so() {
        let context = Context::at_invocation(noon());
        assert!(context.is_empty());
        assert!(context.document().is_none());
        assert!(context.window().is_none());
        assert!(context.selection().is_none());

        let rows = context.shown(&in_english());
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0].text().contains("nothing from your screen"),
            "{}",
            rows[0]
        );
    }

    /// **A person can see what they offered**, which is the half of *offered,
    /// never watched* that they can check for themselves.
    #[test]
    fn what_was_offered_is_shown_as_one_row_for_each_part() {
        let rows = everything_offered().shown(&in_english());
        assert_eq!(rows.len(), 3);
        assert!(rows[0].text().contains("/home/anna/Invoices/march.pdf"));
        assert!(rows[1].text().contains("the text you had selected"));
        assert!(
            rows[2]
                .text()
                .contains("untitled.blend (org.blender.Blender)")
        );
    }

    /// **The row about the selection never holds the selection.** It can be
    /// pages long and it is the person's own text; the row says that it went
    /// with the question, and no more.
    #[test]
    fn the_row_about_a_selection_does_not_repeat_it_back() {
        let context = Context::at_invocation(noon())
            .and_selection(Selection::of("the account number is 12345678").unwrap());
        let rows = context.shown(&in_english());
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].text().contains("12345678"), "{}", rows[0]);
    }

    /// Only the parts that were offered are shown, and the document comes
    /// first because it is the one that grants something.
    #[test]
    fn only_what_was_offered_is_shown_and_the_document_is_first() {
        let rows = Context::at_invocation(noon())
            .and_window(crate::testing::blender())
            .and_document(Document::open(march()).unwrap())
            .shown(&in_english());
        assert_eq!(rows.len(), 2);
        assert!(rows[0].text().contains("the document you have open"));
        assert!(rows[1].text().contains("the window in front of you"));
    }

    /// The rows are the reader's language and the path and identifier in them
    /// are the machine's.
    #[test]
    fn the_rows_are_read_in_the_readers_own_language() {
        let strings = translated(&[
            (words::THE_DOCUMENT, "das geöffnete Dokument: {document}"),
            (
                words::NOTHING_OFFERED,
                "von Ihrem Bildschirm wurde nichts übergeben",
            ),
        ]);
        let rows = Context::at_invocation(noon())
            .and_document(Document::open(march()).unwrap())
            .shown(&strings);
        assert!(rows[0].is_translated(), "{}", rows[0]);
        assert_eq!(
            rows[0].text(),
            "das geöffnete Dokument: /home/anna/Invoices/march.pdf"
        );

        let nothing = Context::at_invocation(noon()).shown(&strings);
        assert!(nothing[0].is_translated());
        assert_eq!(
            nothing[0].text(),
            "von Ihrem Bildschirm wurde nichts übergeben"
        );
    }

    /// **A row is only as translated as the window named inside it.** What a
    /// window is called is introduced by a word of this crate's, so a German
    /// row with that word still in English says so rather than being counted as
    /// done — and the identifier and the title inside it stay as they are,
    /// because neither is anybody's to translate.
    #[test]
    fn a_row_naming_an_untranslated_window_does_not_claim_to_be_translated() {
        let half = translated(&[(words::THE_WINDOW, "das Fenster vor Ihnen: {window}")]);
        let rows = Context::at_invocation(noon())
            .and_window(Focused::titled("org.blender.Blender", "untitled.blend").unwrap())
            .shown(&half);
        assert!(!rows[0].is_translated(), "{}", rows[0]);
        assert!(rows[0].text().starts_with("das Fenster"), "{}", rows[0]);
        assert!(rows[0].text().contains("untitled.blend"), "{}", rows[0]);

        // A window with no title to show is its identifier, which is data: a
        // row naming one is as translated as it reads.
        let bare = Context::at_invocation(noon())
            .and_window(Focused::window("org.gimp.GIMP").unwrap())
            .shown(&half);
        assert!(bare[0].is_translated(), "{}", bare[0]);
        assert_eq!(bare[0].text(), "das Fenster vor Ihnen: org.gimp.GIMP");
    }

    /// A shell that never declared this crate's words shows the key and says it
    /// is a bug, rather than being handed an English sentence kept for the
    /// purpose.
    #[test]
    fn a_shell_that_forgot_to_declare_these_words_shows_that_it_forgot() {
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let rows = Context::at_invocation(noon()).shown(&strings);
        assert!(rows[0].is_a_bug());
        assert_eq!(rows[0].text(), "«context.nothing-offered»");
    }
}
