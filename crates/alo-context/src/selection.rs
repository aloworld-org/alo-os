//! The text a person had selected when they invoked an agent.
//!
//! # It is the person's text, and it grants nothing
//!
//! A selection is words a person had highlighted. It is not a path even when it
//! reads like one, not an application even when it names one, and it does not
//! widen what an agent may reach by so much as a byte: [`crate::Turn`] makes a
//! grant from the open document and from nothing else, and there is a test
//! there that offers `/etc/shadow` as a selection and asserts nothing is
//! granted.
//!
//! It is also the part of a context most likely to have been written by
//! somebody else. ADR 0001's design assumption is that the model is already
//! saying whatever an attacker wants it to say, so nothing here pretends to
//! sanitise a selection into something safe to believe. What it does is
//! narrower and worth stating exactly:
//!
//! # Two things are done to it, and only one of them is announced
//!
//! **Characters nobody can see come out, silently.** A carriage return alone
//! puts the cursor back at the start of a line and lets what follows overwrite
//! what came before; an escape sequence can repaint a terminal. A character
//! that cannot be seen is not part of what somebody selected, so removing it
//! takes nothing away from them and there is nothing to tell them about.
//!
//! **Text that a person can see is never removed silently.** A selection longer
//! than [`MOST`] is offered in part, and how much was left out comes back as a
//! sentence they read ([`Selection::shortened`]). That is `alo-files`' rule —
//! every bound says it was reached — met where the thing cut short is somebody's
//! own document: a bounded answer that does not say so reads exactly like a
//! complete one, and somebody would conclude the agent had read the lot.
//!
//! # What is deliberately left alone
//!
//! **Newlines and tabs stay**, because a selection is prose or code and both
//! mean something with their shape intact. **Nothing is trimmed**, because
//! leading space in a selected block of code is part of it, and deciding that
//! somebody's indentation is nothing is not this crate's decision to make.
//!
//! **The bidirectional marks stay too**, and that is a decision rather than an
//! oversight. `U+200F` and its neighbours can reorder how a line reads, which is
//! a real trick; they are also how Arabic and Hebrew text is written correctly,
//! and alo OS says it is right-to-left ready. Removing them would corrupt the
//! text of exactly the readers that promise is for, in order to defend a line
//! this crate never draws — the selection is never put into an approval
//! sentence and never shown in a row. Whoever *does* draw it owes the defence,
//! and `alo_record::Line` is the shape of it.

use alo_strings::{Counting, Filling, Said, Strings};

use crate::words;

/// How much of a selection is offered in one turn, in characters.
///
/// A bound rather than a promise of no bound, for the reason every bound in
/// `alo-files` exists: something has to be true of the largest thing an agent
/// can be handed, and 200,000 characters is a long document rather than a
/// selection anybody made deliberately.
pub const MOST: usize = 200_000;

/// What a person had selected.
///
/// There is no empty selection: [`Selection::of`] answers `None` rather than
/// making one, because *nothing selected* is a thing [`crate::Context`] says by
/// having no selection rather than by having an empty one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// What was selected, with the characters nobody can see taken out and cut
    /// at [`MOST`].
    text: String,
    /// How many characters were left out by that cut.
    left_out: usize,
}

impl Selection {
    /// The text a person had selected, or `None` if there was none.
    ///
    /// `None` for a selection that is empty, and for one made entirely of
    /// characters that cannot be seen — which is the same thing from an agent's
    /// point of view and should not be two answers.
    #[must_use]
    pub fn of(text: &str) -> Option<Self> {
        let mut kept = String::new();
        let mut taken = 0usize;
        let mut left_out = 0usize;
        for character in text.chars().filter(|character| !unseeable(*character)) {
            if taken < MOST {
                kept.push(character);
                taken += 1;
            } else {
                left_out += 1;
            }
        }
        if kept.is_empty() {
            return None;
        }
        Some(Self {
            text: kept,
            left_out,
        })
    }

    /// What is offered to the agent.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// How many characters long what is offered is.
    #[must_use]
    pub fn how_long(&self) -> usize {
        self.text.chars().count()
    }

    /// How many characters were left out because the selection was too long.
    #[must_use]
    pub fn left_out(&self) -> usize {
        self.left_out
    }

    /// What a person is told when their selection was too long to offer whole,
    /// and nothing when all of it went.
    ///
    /// Counted through the vocabulary rather than written with a number in an
    /// English sentence, because *character* takes a different form for
    /// different numbers in most of the languages this ships in.
    #[must_use]
    pub fn shortened(&self, strings: &Strings) -> Option<Said> {
        if self.left_out == 0 {
            return None;
        }
        Some(strings.count(
            &words::SELECTION_SHORTENED.key(),
            &Counting::of(u64::try_from(self.left_out).unwrap_or(u64::MAX)),
            // The number is put in by the counting, because how many is what
            // picked the shape of the sentence.
            &Filling::nothing(),
        ))
    }
}

/// Whether a character is one nobody can see, and which could therefore rewrite
/// what is around it without anybody noticing.
///
/// Newline and tab are the two that shape text a person selected rather than
/// hiding it, so they are kept.
fn unseeable(character: char) -> bool {
    character.is_control() && character != '\n' && character != '\t'
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated_counting};
    use alo_strings::Form;

    #[test]
    fn a_selection_is_the_text_as_it_was_selected() {
        let selection = Selection::of("  the invoice\tfrom March\nand the note under it").unwrap();
        assert_eq!(
            selection.text(),
            "  the invoice\tfrom March\nand the note under it"
        );
        assert_eq!(selection.left_out(), 0);
        assert!(selection.shortened(&in_english()).is_none());
    }

    /// **Nothing is trimmed.** Leading space in a selected block of code is
    /// part of what somebody selected, and this crate does not get to decide
    /// their indentation is nothing.
    #[test]
    fn a_selection_keeps_its_shape() {
        let code = "    if invoice.is_late() {\n        remind(invoice);\n    }\n";
        assert_eq!(Selection::of(code).unwrap().text(), code);
    }

    /// **A character nobody can see comes out, and nothing is said about it**,
    /// because nothing was taken away that a person had selected. A bare
    /// carriage return is the one that matters: it puts the cursor back to the
    /// start of the line so that what follows overwrites what came before.
    #[test]
    fn what_cannot_be_seen_is_taken_out_quietly() {
        let selection = Selection::of("total: 40\r\ntotal: 4000\u{7}\u{1b}[2K").unwrap();
        assert_eq!(selection.text(), "total: 40\ntotal: 4000[2K");
        assert_eq!(selection.left_out(), 0);
        assert!(selection.shortened(&in_english()).is_none());
    }

    /// **The marks a right-to-left language needs are left alone.** They can be
    /// used to reorder a line, and removing them would corrupt Arabic and
    /// Hebrew text on a system that says it is right-to-left ready. The defence
    /// belongs where a line is drawn, and nothing here draws one.
    #[test]
    fn the_marks_a_right_to_left_reader_needs_are_kept() {
        let hebrew = "\u{200f}חשבונית מרץ\u{200e} 2026";
        assert_eq!(Selection::of(hebrew).unwrap().text(), hebrew);
    }

    /// Nothing selected is not an empty selection, and neither is a selection
    /// made entirely of characters nobody can see.
    #[test]
    fn nothing_selected_is_no_selection_at_all() {
        assert!(Selection::of("").is_none());
        assert!(Selection::of("\u{1b}\u{7}\r").is_none());
        assert!(Selection::of(" ").is_some());
    }

    /// **A bound that did not say it was reached would read like a complete
    /// answer**, so the bound says so — and says it counted the reader's own
    /// way, which is the whole reason `alo-strings` learned to count.
    #[test]
    fn a_selection_too_long_to_offer_whole_says_how_much_was_left_out() {
        let long: String = "a".repeat(MOST + 3);
        let selection = Selection::of(&long).unwrap();
        assert_eq!(selection.how_long(), MOST);
        assert_eq!(selection.left_out(), 3);

        let said = selection.shortened(&in_english()).unwrap();
        assert!(said.text().contains('3'), "{said}");
        assert!(said.text().contains("characters"), "{said}");
        assert!(said.text().contains("only the first part"), "{said}");
    }

    /// One left out is one character, not one characters — which is English's
    /// two forms, and the reader's own language may have more.
    #[test]
    fn one_character_left_out_is_counted_as_one() {
        let long: String = "a".repeat(MOST + 1);
        let said = Selection::of(&long)
            .unwrap()
            .shortened(&in_english())
            .unwrap();
        assert!(said.text().contains("1 character of"), "{said}");
        assert!(!said.text().contains("characters"), "{said}");
    }

    /// And in a language with three forms it is that language's three, read out
    /// of CLDR rather than out of English's habits: Polish says one thing about
    /// one, another about three, and a third about twenty-five.
    #[test]
    fn how_many_were_left_out_is_counted_the_readers_own_way() {
        let strings = translated_counting(&[
            (Form::One, "{characters} znak nie został przekazany"),
            (Form::Few, "{characters} znaki nie zostały przekazane"),
            (Form::Many, "{characters} znaków nie zostało przekazanych"),
        ]);
        let said = |over: usize| {
            Selection::of(&"a".repeat(MOST + over))
                .unwrap()
                .shortened(&strings)
                .unwrap()
        };
        assert!(said(3).is_translated(), "{}", said(3));
        assert_eq!(said(1).text(), "1 znak nie został przekazany");
        assert_eq!(said(3).text(), "3 znaki nie zostały przekazane");
        assert_eq!(said(25).text(), "25 znaków nie zostało przekazanych");
    }

    /// A cut lands on a character, never inside one. The bound is characters
    /// rather than bytes for exactly this reason: a selection cut at 200,000
    /// bytes could end half way through a letter in every language that needs
    /// more than one byte to write one.
    #[test]
    fn a_selection_is_cut_between_characters_and_never_inside_one() {
        let long: String = "é".repeat(MOST + 2);
        let selection = Selection::of(&long).unwrap();
        assert_eq!(selection.how_long(), MOST);
        assert_eq!(selection.left_out(), 2);
        assert!(selection.text().chars().all(|character| character == 'é'));
    }
}
