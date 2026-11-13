//! A text argument as the approval shows it: all of it, with every character a
//! person could miss marked where it is.
//!
//! ADR 0001, amended 2026-11-13, in the owner's words: *"There should be no
//! input limits"* and *"We should make something user-friendly and no
//! limits"*. So a text an agent sends is never cut, never summarised in place
//! of itself, and never refused for holding something hard to see. What it
//! owes the person instead is three things, and they are this file:
//!
//! - **all of it** — [`TheWholeText::lines`] holds every line and every
//!   character, and nothing in this type can drop one; putting the pieces
//!   back together gives exactly the text that arrived;
//! - **how long it is**, before they start reading — [`TheWholeText::how_long`]
//!   says it in lines and characters, in the reader's language;
//! - **what could be missed** — each such character is a [`Piece::Unseen`],
//!   named in plain words with its code point ([`Unseen::said`]), and
//!   [`TheWholeText::what_is_marked`] says how many there are.
//!
//! This is the model and not the drawing. The surface that draws it lets the
//! person scroll through every line; until a verb declares a text argument
//! (stage 6 of `docs/autonomy/demo-two-alo-inside-applications-plan.md`)
//! nothing reaches one, and `docs/design/the-ask-alo-task-window.md` records
//! that the approved frames draw no text block.

use alo_strings::{Counting, Filling, Said, Strings};

use crate::unseen::{WhyMarked, from_another_alphabet, on_its_own};
use crate::words;

/// A text, ready to be shown whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheWholeText {
    /// Exactly what arrived.
    given: String,
    /// Every line of it, in order.
    lines: Vec<ShownLine>,
    /// How many characters it holds — Unicode scalar values, as a person
    /// moving a cursor through it would mostly count them.
    characters: usize,
    /// How many of them are marked.
    unseen: usize,
}

/// One line of a text, as it is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShownLine {
    /// What is on it, in order.
    pieces: Vec<Piece>,
    /// How it ended.
    ending: Ending,
}

/// How one line ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// With a newline.
    Newline,
    /// With a carriage return and a newline — the way Windows ends a line,
    /// which is a line break and not something hidden.
    CarriageReturnAndNewline,
    /// It is the last line, and nothing ended it.
    EndOfText,
}

/// A run of a line that is shown as it is, or one character that is marked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// Characters a person sees as they are.
    Seen(String),
    /// One character a person could miss, which the approval marks.
    Unseen(Unseen),
}

/// One character a person could miss, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unseen {
    /// The character itself, which is delivered exactly as it is.
    character: char,
    /// Why it is marked.
    why: WhyMarked,
}

impl Unseen {
    /// The character itself.
    #[must_use]
    pub fn character(&self) -> char {
        self.character
    }

    /// Why it is marked.
    #[must_use]
    pub fn why(&self) -> WhyMarked {
        self.why
    }

    /// Its code point, as Unicode writes it: `U+200B`.
    #[must_use]
    pub fn code(&self) -> String {
        format!("U+{:04X}", u32::from(self.character))
    }

    /// What the mark says, in the reader's language: *invisible character
    /// U+200B*.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = Filling::of("code", self.code());
        match self.why {
            WhyMarked::Control => strings.say(&words::TEXT_HOLDS_A_CONTROL.key(), &filling),
            WhyMarked::Direction => strings.say(&words::TEXT_HOLDS_A_DIRECTION.key(), &filling),
            WhyMarked::Invisible => strings.say(&words::TEXT_HOLDS_AN_INVISIBLE.key(), &filling),
            WhyMarked::Space => strings.say(&words::TEXT_HOLDS_A_SPACE.key(), &filling),
            WhyMarked::OtherAlphabet => strings.say(
                &words::TEXT_HOLDS_ANOTHER_ALPHABET.key(),
                &filling.and("letter", self.character.to_string()),
            ),
        }
    }
}

impl ShownLine {
    /// What is on it, in order.
    #[must_use]
    pub fn pieces(&self) -> &[Piece] {
        &self.pieces
    }

    /// How it ended.
    #[must_use]
    pub fn ending(&self) -> Ending {
        self.ending
    }

    /// This line exactly as it arrived, without its ending.
    #[must_use]
    pub fn as_given(&self) -> String {
        let mut line = String::new();
        for piece in &self.pieces {
            match piece {
                Piece::Seen(seen) => line.push_str(seen),
                Piece::Unseen(unseen) => line.push(unseen.character),
            }
        }
        line
    }

    /// One line's characters, in pieces.
    fn of(text: &str, ending: Ending) -> Self {
        let characters: Vec<char> = text.chars().collect();
        let foreign = foreign_letters(&characters);
        let mut pieces = Vec::new();
        let mut seen = String::new();
        for (character, is_foreign) in characters.iter().zip(foreign) {
            let why = on_its_own(*character).or(is_foreign.then_some(WhyMarked::OtherAlphabet));
            match why {
                Some(why) => {
                    if !seen.is_empty() {
                        pieces.push(Piece::Seen(std::mem::take(&mut seen)));
                    }
                    pieces.push(Piece::Unseen(Unseen {
                        character: *character,
                        why,
                    }));
                }
                None => seen.push(*character),
            }
        }
        if !seen.is_empty() {
            pieces.push(Piece::Seen(seen));
        }
        Self { pieces, ending }
    }

    /// How many marked characters it holds.
    fn unseen(&self) -> usize {
        self.pieces
            .iter()
            .filter(|piece| matches!(piece, Piece::Unseen(_)))
            .count()
    }
}

/// For each character of a line, whether it is a letter from another alphabet
/// inside a word — a word being a run of letters and digits.
fn foreign_letters(characters: &[char]) -> Vec<bool> {
    let mut foreign = Vec::with_capacity(characters.len());
    for run in characters.chunk_by(|one, next| one.is_alphanumeric() == next.is_alphanumeric()) {
        if run
            .first()
            .is_some_and(|character| character.is_alphanumeric())
        {
            foreign.extend(from_another_alphabet(run));
        } else {
            foreign.extend(std::iter::repeat_n(false, run.len()));
        }
    }
    foreign
}

impl TheWholeText {
    /// A text, read into lines, with what could be missed found.
    ///
    /// Every character is kept. The text is split at each newline; a carriage
    /// return immediately before a newline is part of that line's ending, and
    /// one anywhere else is marked. A text that ends with a newline shows the
    /// empty line after it, as an editor does.
    #[must_use]
    pub fn of(text: &str) -> Self {
        let mut lines = Vec::new();
        let mut rest = text;
        while let Some((line, after)) = rest.split_once('\n') {
            let (line, ending) = match line.strip_suffix('\r') {
                Some(line) => (line, Ending::CarriageReturnAndNewline),
                None => (line, Ending::Newline),
            };
            lines.push(ShownLine::of(line, ending));
            rest = after;
        }
        lines.push(ShownLine::of(rest, Ending::EndOfText));
        let unseen = lines.iter().map(ShownLine::unseen).sum();
        Self {
            given: text.to_owned(),
            lines,
            characters: text.chars().count(),
            unseen,
        }
    }

    /// Exactly what arrived.
    #[must_use]
    pub fn as_given(&self) -> &str {
        &self.given
    }

    /// Every line, in order — all of them.
    #[must_use]
    pub fn lines(&self) -> &[ShownLine] {
        &self.lines
    }

    /// How many lines the text holds, counted as a person counts them: a
    /// newline at the very end ends the last line rather than starting another.
    #[must_use]
    pub fn how_many_lines(&self) -> usize {
        let trailing_empty =
            self.lines.len() > 1 && self.lines.last().is_some_and(|line| line.pieces.is_empty());
        self.lines.len() - usize::from(trailing_empty)
    }

    /// How many characters the text holds.
    #[must_use]
    pub fn characters(&self) -> usize {
        self.characters
    }

    /// How many characters in it are marked.
    #[must_use]
    pub fn unseen(&self) -> usize {
        self.unseen
    }

    /// How long it is, in the reader's language: *12 lines, 418 characters*.
    #[must_use]
    pub fn how_long(&self, strings: &Strings) -> Said {
        let lines = strings.count(
            &words::TEXT_LINES.key(),
            &counting(self.how_many_lines()),
            &Filling::nothing(),
        );
        let characters = strings.count(
            &words::TEXT_CHARACTERS.key(),
            &counting(self.characters),
            &Filling::nothing(),
        );
        strings.say(
            &words::TEXT_HOW_LONG.key(),
            &Filling::nothing()
                .and_said("lines", &lines)
                .and_said("characters", &characters),
        )
    }

    /// What the approval says above the text when something in it is marked,
    /// and nothing when nothing is.
    #[must_use]
    pub fn what_is_marked(&self, strings: &Strings) -> Option<Said> {
        (self.unseen > 0).then(|| {
            strings.count(
                &words::TEXT_UNSEEN.key(),
                &counting(self.unseen),
                &Filling::nothing(),
            )
        })
    }
}

/// A number of things, as `alo-strings` counts them. A text this machine
/// cannot count in a `u64` is not one it can hold, so saturating keeps the
/// sentence countable rather than making the type carry the impossible.
fn counting(how_many: usize) -> Counting {
    Counting::of(u64::try_from(how_many).unwrap_or(u64::MAX))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_whole_text_tests.rs"]
mod tests;
