//! What alo has said into this transcript, beneath what the program wrote.
//!
//! Two things reach an `alo` line, and this type is where the difference is
//! kept:
//!
//! - **why a command is waiting** — one of the five answers of
//!   [`crate::why_a_command_is_waiting`], which are this repository's own words;
//! - **what a command will do** — an explanation of a command nobody has run
//!   yet, from [`crate::what_a_command_will_do`], whose plain words a model
//!   wrote.
//!
//! `docs/features.md` promises both: *a command looks stuck; **ask**, and get
//! **waiting for the keyboard**…*, and *Explain a command before it runs. Plain
//! words for what it does…*
//!
//! # Our words and a model's are different claims, so they are different variants
//!
//! [`AloSRemark`] does not flatten to text. A key is a name this repository
//! declares and resolves in the language the person chose; a model's sentence is
//! a stranger's prose in whatever language it answered in, and nothing here can
//! translate it or vouch for it.
//!
//! A type that held both as `String` would lose that, and the loss would be
//! invisible: both draw as a line labelled `alo`. So [`AloSLine`] carries the
//! distinction all the way to whatever draws it, and the drawing side has to
//! decide what to do about it rather than being handed text.
//!
//! # There is still no road from a program's bytes to an `alo` line
//!
//! This was true when the type held only keys, and it has to stay true now that
//! it holds a sentence. It does, because the only way to a model's words is a
//! [`crate::what_a_command_will_do::WhatACommandWillDo`], which is built from an
//! answer to a question somebody asked — and a program's output is not one.
//!
//! `tests/alo_says_only_what_the_vocabulary_declares.rs` holds this crate to it:
//! every key this type can hold is declared in the vocabulary, and the public
//! doors onto this type are a closed list that test reads off the source.
//!
//! **That test is why the explanation lives behind its own type** rather than
//! `says_in_plain_words(&str)`, which would have been shorter and would have put
//! a `&str` door onto an `alo` line for anybody to find.
//!
//! # One remark, not a log
//!
//! Asking twice replaces rather than appends — whichever question was asked.
//! Somebody asking *why is it waiting* a second time wants the answer now, not a
//! history of answers; somebody who asks what a command will do and then asks
//! why it is waiting has moved on, and the earlier remark is about something
//! else.
//!
//! **This is why it is a single slot and not a `Vec`.** The type makes the
//! decision rather than a caller remembering to clear it.
//!
//! # The count and the lines cannot disagree
//!
//! [`WhatAloSaid::lines`] is a number and [`WhatAloSaid::the_lines`] is the
//! lines themselves, and the first exists because [`crate::regions_of`] takes
//! counts and must not be able to see what anybody wrote. Two answers about one
//! thing is how a gutter label ends up beside the wrong row, so
//! `what_alo_said_tests.rs` asserts they agree for every shape this type has.

use alo_strings::Key;

use crate::what_a_command_will_do::WhatACommandWillDo;

/// One remark alo has made, and which kind of claim it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AloSRemark {
    /// One of this repository's own words, by key.
    ///
    /// The five answers to *why is it waiting* arrive this way.
    InOurWords(Key),
    /// An explanation of a command nobody has run yet.
    ///
    /// Its plain words are a model's; its marks and its disclaimer are ours.
    AnExplanation(WhatACommandWillDo),
}

/// One line of what alo said, as something that draws it needs it.
///
/// The distinction this type exists to carry is in its two names: one is a key
/// to resolve, the other is prose to show as it arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AloSLine {
    /// This repository's own words, by key, to be resolved in the person's
    /// language at the moment of drawing.
    InOurWords(Key),
    /// A sentence a model wrote, in whatever language it answered in.
    ///
    /// **Not translated and not translatable**: there is no key for it, because
    /// nobody here wrote it. A drawing path that treated this as one of ours
    /// would be claiming authorship of a stranger's sentence.
    AsTheModelSaidIt(String),
}

/// What alo has said into one transcript.
///
/// Empty by default: a terminal nobody has asked anything carries no alo line,
/// and `regions_of` is then given nought for alo's count and draws no label.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatAloSaid {
    /// The one remark, or none.
    said: Option<AloSRemark>,
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
        self.said = Some(AloSRemark::InOurWords(this));
    }

    /// alo explains what a command will do, in place of whatever it said before.
    ///
    /// Takes the explanation rather than its text, so that the only prose that
    /// can reach an `alo` line is prose that came back from a question somebody
    /// asked.
    pub fn explains(&mut self, this: WhatACommandWillDo) {
        self.said = Some(AloSRemark::AnExplanation(this));
    }

    /// alo has nothing to say.
    ///
    /// Called when the question has no answer — the command finished, or a
    /// person is at a prompt — so that a reply about a command that has gone
    /// does not sit on screen looking current.
    pub fn nothing_to_say(&mut self) {
        self.said = None;
    }

    /// The remark, if there is one.
    #[must_use]
    pub const fn remark(&self) -> Option<&AloSRemark> {
        self.said.as_ref()
    }

    /// The lines to draw, in reading order.
    ///
    /// An explanation is the plain words, then each mark, then the disclaimer —
    /// which is last because it is about everything above it, and present always
    /// because [`WhatACommandWillDo::and_not_a_guarantee`] has no shape that
    /// omits it.
    #[must_use]
    pub fn the_lines(&self) -> Vec<AloSLine> {
        match &self.said {
            None => Vec::new(),
            Some(AloSRemark::InOurWords(key)) => vec![AloSLine::InOurWords(key.clone())],
            Some(AloSRemark::AnExplanation(explained)) => {
                let mut lines = Vec::with_capacity(explained.how_many_marks() + 2);
                lines.push(AloSLine::AsTheModelSaidIt(
                    explained.in_plain_words().to_owned(),
                ));
                for mark in explained.marked() {
                    lines.push(AloSLine::InOurWords(mark.the_word().key()));
                }
                lines.push(AloSLine::InOurWords(explained.and_not_a_guarantee().key()));
                lines
            }
        }
    }

    /// How many transcript lines alo's remark occupies.
    ///
    /// A count rather than the text, because this is what [`crate::regions_of`]
    /// takes and that function must not be able to see what anybody wrote.
    ///
    /// Counted here rather than by building [`Self::the_lines`], because this is
    /// asked once a frame and that would allocate for a number. The two are
    /// asserted to agree in this module's tests, which is the only thing that
    /// makes a second answer about one subject safe.
    #[must_use]
    pub fn lines(&self) -> usize {
        match &self.said {
            None => 0,
            Some(AloSRemark::InOurWords(_)) => 1,
            // The plain words, each mark, and the disclaimer.
            Some(AloSRemark::AnExplanation(explained)) => explained.how_many_marks() + 2,
        }
    }
}

#[cfg(test)]
#[path = "what_alo_said_tests.rs"]
mod tests;
