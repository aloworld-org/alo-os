//! What a command will do, said before it runs — and never said as a guarantee.
//!
//! `docs/features.md`: *Explain a command before it runs. Plain words for what
//! it does, with anything destructive, anything needing elevated rights and
//! anything leaving the machine marked. Edit it, or try it in the box first.
//! **Called an explanation and not a guarantee**, because a script, an alias or
//! something it downloads can change what happens.*
//!
//! This is the value that promise produces. It is not the asking: the question
//! goes to a model through the agent, because neither this crate nor the shell
//! ships one, and what comes back arrives here.
//!
//! # Nothing can ask yet, and the reason is not a missing chord
//!
//! **alo does not know what a person has typed, and cannot find out.** Measured
//! on 2026-10-17, in this crate:
//!
//! | what was looked for | what is there |
//! |---|---|
//! | anything holding the typed line | nothing. [`crate::what_a_person_typed`] turns a key into bytes and `a_shell_on_a_pseudoterminal.rs`'s `a_person_typed` writes them to the pseudoterminal, keeping none |
//! | the command in front | a **process id**, from which `/proc` gives the command line of something **already running** |
//! | shell integration | none. The grid is `alacritty_terminal` with a `VoidListener`, and `terminal_output_becomes_a_grid.rs` says so in its own words: *nothing is listening* |
//!
//! So the three roads to the text of a command nobody has run are each a
//! decision rather than a morning's work:
//!
//! - **read it off the screen.** The current row after the prompt — except that
//!   nothing tells a prompt from input, because the grid is cells a program
//!   wrote. It is `tests/nothing_reads_the_bytes_to_decide_who_spoke.rs`'s fault
//!   one storey up: deciding what the *person* typed by reading what a
//!   *program* printed, and then sending it to a model.
//! - **keep our own copy of the bytes we forwarded.** Wrong, and demonstrably:
//!   a shell's line editor changes the line without telling anyone — an arrow
//!   key, a backspace, history, tab completion — so the copy diverges the first
//!   time somebody presses Up, and diverges silently.
//! - **shell integration**, which is how every other terminal answers this: the
//!   shell reports its prompt and its line in an escape sequence. It is the real
//!   answer, and it is a **new mechanism, a published contract, and a security
//!   question** — any program can print that sequence, so a terminal that
//!   believed it would let a program say what the person typed. That is the
//!   gutter's own problem again, which ADR 0113 settled by never reading bytes
//!   to decide an origin.
//!
//! **So this crate carries the answer and nothing carries the question.** That
//! is deliberate and it is said here rather than left to be discovered, and it
//! is the same shape [`crate::why_a_command_is_waiting`] was landed in: complete
//! public API of this crate, reachable the moment something can ask, with no
//! chord bound to it. **No `alo_shortcuts::Action` was added**, because a chord
//! bound to a surface that cannot work teaches a person a road and then takes it
//! away — which is what `alo-shell`'s `tests/the_road_every_action_takes.rs`
//! exists to refuse.
//!
//! # The three marks are the model's claims, and this type says so
//!
//! A mark is a thing *the answer said*, not a thing this machine checked. The
//! difference is the whole design of the file, because the two read identically
//! on a screen and only one of them is evidence.
//!
//! So [`WorthSeeing`] is reached through [`WhatACommandWillDo::marked`], whose
//! name is what it is for, and no method here is called `is_destructive` or
//! `is_safe`. A caller cannot ask this type whether a command is safe, because
//! it does not know and neither does the model.
//!
//! # There is no unmarked-means-safe, and that is enforced by having no such word
//!
//! An empty [`WhatACommandWillDo::marked`] means **nothing was marked**. It does
//! not mean *nothing is destructive*, and nothing here will say that it does.
//!
//! This is `CLAUDE.md`'s *a flag that fails open is a flag that will be
//! forgotten in the direction that matters*, answered by never writing the flag:
//! a model that misses a destructive command produces an explanation with no
//! mark, and an explanation with no mark says only that. The repository's own
//! shorter form is *absent is stronger than defaulted* — a default can be failed
//! to override, and an absence cannot be overridden wrongly.
//!
//! # The disclaimer is not optional, and a method is how that is held
//!
//! [`WhatACommandWillDo::and_not_a_guarantee`] takes `&self` and always answers a
//! word. There is no shape of this type that lacks it and no argument that turns
//! it off, so *called an explanation and not a guarantee* is a property of every
//! explanation rather than a sentence a drawing path remembers to add.
//!
//! It is the shape [`crate::why_a_command_is_waiting`]'s sibling in
//! `alo-capability` already uses — a method on the value although the answer
//! does not depend on it — and for the same reason: asking every value and
//! getting the same answer is how *always* becomes checkable.
//!
//! # The plain words are not ours, and they are not translated
//!
//! Every other string this crate shows is an [`alo_strings::Key`] resolved
//! against the vocabulary in the language the person chose. The plain words of
//! an explanation cannot be: they are a sentence a model wrote, in whatever
//! language it answered in, and this repository has no key for them.
//!
//! That is why they are kept as a [`String`] here while everything else is a
//! key, and why [`crate::what_alo_said`] keeps the two apart in its own type
//! rather than flattening them to text. A sentence of ours and a sentence of a
//! model's are different kinds of claim, and a transcript that drew them the
//! same way would be a machine with two accounts of one moment.

use std::collections::BTreeSet;

use crate::words;

/// Something about a command worth seeing before pressing Enter.
///
/// The three `docs/features.md` names, and no fourth. Each is **what an answer
/// claimed**, never what this machine found — see this module's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorthSeeing {
    /// It destroys something.
    Destructive,
    /// It needs rights beyond the person's own.
    NeedsElevatedRights,
    /// Something leaves this machine.
    LeavesTheMachine,
}

impl WorthSeeing {
    /// Every mark, for a test that must cover all of them.
    pub const ALL: [Self; 3] = [
        Self::Destructive,
        Self::NeedsElevatedRights,
        Self::LeavesTheMachine,
    ];

    /// The words a person reads for this mark.
    #[must_use]
    pub const fn the_word(self) -> alo_strings::Word {
        match self {
            Self::Destructive => words::IT_DESTROYS_SOMETHING,
            Self::NeedsElevatedRights => words::IT_NEEDS_MORE_THAN_YOUR_RIGHTS,
            Self::LeavesTheMachine => words::SOMETHING_LEAVES_THIS_MACHINE,
        }
    }
}

/// What a command will do, as an explanation that came back.
///
/// Built from the plain words and the marks the answer carried. There is no
/// constructor that takes a program's output, and
/// `tests/alo_says_only_what_the_vocabulary_declares.rs` holds this crate to
/// that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatACommandWillDo {
    /// The plain words, as the answer wrote them. Not a key, not translated.
    in_plain_words: String,
    /// What the answer marked, de-duplicated and in a settled order so one
    /// explanation reads the same way however the marks arrived.
    marked: BTreeSet<WorthSeeing>,
}

impl WhatACommandWillDo {
    /// An explanation in these words, marked with these.
    ///
    /// Duplicates in `marked` collapse, and the order it arrives in is not kept:
    /// a mark is a fact about the command and not a position in a list, so two
    /// answers that marked the same things are the same explanation.
    #[must_use]
    pub fn of(in_plain_words: &str, marked: &[WorthSeeing]) -> Self {
        Self {
            in_plain_words: in_plain_words.to_owned(),
            marked: marked.iter().copied().collect(),
        }
    }

    /// The plain words, as the answer wrote them.
    ///
    /// **Not this repository's words.** A caller drawing these is drawing a
    /// model's sentence, which is why [`crate::what_alo_said`] carries it in a
    /// variant of its own rather than beside a key.
    #[must_use]
    pub fn in_plain_words(&self) -> &str {
        &self.in_plain_words
    }

    /// What the answer marked, in a settled order.
    ///
    /// **Empty means nothing was marked**, which is not the same as nothing
    /// being destructive. This module's header says why there is no method here
    /// that would let a caller blur the two.
    pub fn marked(&self) -> impl Iterator<Item = WorthSeeing> + '_ {
        self.marked.iter().copied()
    }

    /// How many marks it carried.
    #[must_use]
    pub fn how_many_marks(&self) -> usize {
        self.marked.len()
    }

    /// *An explanation, not a guarantee* — always, for every explanation.
    ///
    /// Takes `&self` and answers unconditionally. There is no shape of this type
    /// without it, which is the point: see this module's header.
    #[must_use]
    pub const fn and_not_a_guarantee(&self) -> alo_strings::Word {
        words::AN_EXPLANATION_AND_NOT_A_GUARANTEE
    }
}

#[cfg(test)]
#[path = "what_a_command_will_do_tests.rs"]
mod tests;
