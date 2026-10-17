//! What alo has said into this transcript, beneath what the program wrote.
//!
//! `docs/features.md`: *a command looks stuck; **ask**, and get **waiting for
//! the keyboard**, **for the network**, **for another process or a lock**,
//! **still computing**, or **unknown with the evidence available***. The five
//! answers are [`crate::why_a_command_is_waiting`]; this is where one of them
//! goes once a person has asked, so that something can draw it.
//!
//! # Keys, not English
//!
//! Each remark is an [`alo_strings::Key`] and never a sentence. `CLAUDE.md`
//! makes hardcoded English a bug and the first target is all 24 official EU
//! languages, so what is kept is *which thing was said* and the vocabulary turns
//! it into words at the moment of drawing — in the language the person chose,
//! not the one this crate was written in.
//!
//! It also means this type cannot be used to put arbitrary text on a person's
//! screen. There is no road here from a program's bytes to a line labelled
//! `alo`: a key is a name this repository declares, and
//! `tests/alo_says_only_what_the_vocabulary_declares.rs` holds it to the keys
//! [`crate::why_a_command_is_waiting::WhyItIsWaiting`] can produce.
//!
//! # One remark, not a log
//!
//! Asking twice replaces rather than appends. A person asking *why is it
//! waiting* a second time wants the answer now, not a history of answers — and a
//! transcript that grew by a line every time somebody asked would push the
//! program's own output off the screen for a feature nobody asked to keep.
//!
//! **This is why it is a single slot and not a `Vec`.** The type makes the
//! decision rather than a caller remembering to clear it.

use alo_strings::Key;

/// What alo has said into one transcript.
///
/// Empty by default: a terminal nobody has asked anything carries no alo line,
/// and `regions_of` is then given nought for alo's count and draws no label.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatAloSaid {
    /// The one remark, or none.
    said: Option<Key>,
}

impl WhatAloSaid {
    /// Nothing said yet.
    #[must_use]
    pub const fn nothing() -> Self {
        Self { said: None }
    }

    /// alo says this, in place of whatever it said before.
    ///
    /// Takes a [`Key`] rather than a string for the reason in this module's
    /// header: what is kept is which thing was said, and the words are the
    /// vocabulary's at the moment of drawing.
    pub fn says(&mut self, this: Key) {
        self.said = Some(this);
    }

    /// alo has nothing to say, because there is nothing waiting any more.
    ///
    /// Called when the question has no answer — the command finished, or a
    /// person is at a prompt — so that a reply about a command that has gone
    /// does not sit on screen looking current.
    pub fn nothing_to_say(&mut self) {
        self.said = None;
    }

    /// What alo said, if anything.
    #[must_use]
    pub const fn said(&self) -> Option<&Key> {
        self.said.as_ref()
    }

    /// How many transcript lines alo's remark occupies.
    ///
    /// One per remark, or nought. A count rather than the text, because this is
    /// what [`crate::regions_of`] takes and that function must not be able to
    /// see what anybody wrote.
    #[must_use]
    pub const fn lines(&self) -> usize {
        match self.said {
            Some(_) => 1,
            None => 0,
        }
    }
}

#[cfg(test)]
#[path = "what_alo_said_tests.rs"]
mod tests;
