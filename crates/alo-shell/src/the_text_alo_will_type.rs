//! **The text alo will type, as the approval card shows it**: how long it is,
//! how much in it is marked, and the twelve lines in view with every hidden
//! character a tag naming it.
//!
//! ADR 0001 as amended 2026-11-13, in the owner's words: *"There should be no
//! input limits"*. A text alo sends is never cut, never summarised, never
//! refused for what is in it, always shown and never run. What it owes the
//! person is in `alo_capability::TheWholeText`; this is only the card's
//! reading of it, and **the owner decided the look on 2026-11-15**: System/Mono
//! 12/18 in the touches row's `bg/cool` box with a 12 corner, up to twelve
//! lines and then it scrolls, each hidden character a small tag such as
//! *U+200B* in the warning colour. `docs/design/the-ask-alo-task-window.md`
//! records it.
//!
//! # The tag is the character's place, and what it is is said
//!
//! A character a person could miss is drawn as its code, *U+200B*, where it
//! stands, marked [`HowItIsMarked::Unseen`] — drawn in `status/warning` and a
//! heavier weight, so it is told apart from text that happens to read
//! *U+200B* by more than its colour. A letter from another alphabet is
//! visible, so it is drawn and its tag follows it. A screen reader hears each
//! one named in words (`crate::the_text_alo_will_type_as_read`).
//!
//! Nothing here is sent anywhere: the text delivered is
//! `TheWholeText::as_given`, untouched by any of this.

use alo_capability::{Piece, TheWholeText, WhyMarked};
use alo_strings::Strings;

use crate::alos_marked_words::{HowItIsMarked, MarkedWords};
use crate::the_approval_card::THE_MOST_LINES_SHOWN;

/// One text, as the card draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ATextOnTheCard {
    /// How long it is: *12 lines, 418 characters*.
    pub how_long: String,
    /// How many characters in it are marked, when any are.
    pub marked: Option<String>,
    /// The lines in view, joined by newlines, each hidden character its tag.
    pub shown: MarkedWords,
}

impl ATextOnTheCard {
    /// The card's reading of `text`, scrolled to start at `first_line` —
    /// clamped so the last twelve lines are the furthest it goes.
    #[must_use]
    pub fn of(text: &TheWholeText, first_line: usize, strings: &Strings) -> Self {
        let lines = text.lines();
        let starts = first_line.min(lines.len().saturating_sub(THE_MOST_LINES_SHOWN));
        let mut shown = MarkedWords::default();
        for (at, line) in lines
            .iter()
            .skip(starts)
            .take(THE_MOST_LINES_SHOWN)
            .enumerate()
        {
            if at > 0 {
                shown.push("\n", HowItIsMarked::Plain);
            }
            for piece in line.pieces() {
                match piece {
                    Piece::Seen(words) => shown.push(words, HowItIsMarked::Plain),
                    Piece::Unseen(unseen) => {
                        // **A visible letter is drawn, and its tag after it**;
                        // anything else is only its tag, in its place.
                        if unseen.why() == WhyMarked::OtherAlphabet {
                            shown.push(&unseen.character().to_string(), HowItIsMarked::Plain);
                        }
                        // **Two tags in a row are two tags**: a thin space
                        // keeps them apart on the screen, and only there.
                        if shown
                            .marks()
                            .last()
                            .is_some_and(|it| it.how == HowItIsMarked::Unseen)
                        {
                            shown.push("\u{2009}", HowItIsMarked::Plain);
                        }
                        shown.push(&unseen.code(), HowItIsMarked::Unseen);
                    }
                }
            }
        }
        Self {
            how_long: text.how_long(strings).into_text(),
            marked: text
                .what_is_marked(strings)
                .map(alo_strings::Said::into_text),
            shown,
        }
    }

    /// How many lines are said above the box: how long it is, and how much is
    /// marked when anything is.
    #[must_use]
    pub fn said_lines(&self) -> u32 {
        if self.marked.is_some() { 2 } else { 1 }
    }
}

#[cfg(test)]
#[path = "the_text_alo_will_type_tests.rs"]
mod tests;
