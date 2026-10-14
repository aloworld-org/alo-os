//! Who produced a piece of the transcript, and the one rule that makes the
//! answer worth anything.
//!
//! [ADR 0113](../../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
//! decides that a narrow gutter beside the transcript belongs to the terminal
//! and says who is speaking. This module is the half of that promise which is
//! not about drawing: **what the four answers are, and where each one can come
//! from.**
//!
//! # The rule, which is an absence rather than a check
//!
//! **Origin is where bytes entered the terminal. It is never what they say.**
//!
//! A program can print any bytes, including `system`, including a sentence
//! that reads exactly like one of alo's. `docs/features.md` promises
//! *Listening verified*, *Permissions have not changed* and *Observed at
//! 13:15* — the lines carrying every ounce of trust in this product — and in a
//! plain transcript a program can print all three, in the same font, in the
//! same stream.
//!
//! So this module offers **no way at all** to obtain a [`WhoIsSpeaking`] from
//! text. There is no parser, no `FromStr`, no constructor that takes a string
//! and no heuristic over output. The only way to name an origin is to be the
//! code that knows where the bytes came from, and that code names it in the
//! same breath as receiving them.
//!
//! That is deliberately a *missing function* rather than a validating one. A
//! check can be passed wrongly and a default can be failed to override; an
//! absence can be neither. `the_origin_cannot_be_got_from_the_bytes` is the
//! test that holds the absence in place, because an absence is exactly the
//! kind of guarantee a later change deletes without noticing.
//!
//! # What the live terminal produces today, and why that is honest
//!
//! Four origins exist because the design draws four and ADR 0113 decides four.
//! **One is produced**, because a pseudoterminal is the only thing that writes
//! to the transcript today: everything on the screen came from the program, so
//! everything on the screen is labelled as the program's. The person's own
//! typing is echoed back by the shell, so it arrives as the program's bytes and
//! is labelled as such — which is the rule above being obeyed rather than an
//! omission.
//!
//! The other three become reachable when alo itself writes into a transcript.
//! Until then there is nothing to label them, and inventing a producer would be
//! the one thing this module exists to prevent.

use crate::words;

/// Who produced a region of the transcript.
///
/// The four the design draws and
/// [ADR 0113](../../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
/// decides. The design's own annotation defines them: *`PROGRAM` includes
/// parsed rich results. `SYSTEM` means an attributed observation, with scope
/// and time. `ALO` is interpretation. `YOU` is input.*
///
/// **There is no fifth, and no "unknown".** An origin nobody could name would
/// be a region whose label says nothing while occupying the place a person has
/// been taught to trust, which is worse than no gutter at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhoIsSpeaking {
    /// A program's own output, rich results parsed from it included.
    ///
    /// **The only one a pseudoterminal can produce**, and it is produced by
    /// being read off one rather than by anything the bytes contain.
    AProgram,
    /// alo's own interpretation.
    ///
    /// *Interpretation*, in the design's word, and the distinction ADR 0113
    /// turns on: an alo sentence is not evidence because alo said it.
    Alo,
    /// An attributed observation, with its scope and the time it was made.
    ///
    /// Not "alo is confident": a separate trusted observer measured something
    /// and this is what it saw. The region says what was observed and when.
    TheSystem,
    /// What the person submitted.
    ThePerson,
}

impl WhoIsSpeaking {
    /// The word the gutter draws for this origin.
    ///
    /// A key rather than a string, because `CLAUDE.md` makes hardcoded English
    /// a bug and the first target is all 24 official EU languages. The English
    /// under each key is in [`crate::words`], lowercase, because that is how
    /// the design draws them — `you`, `alo`, `system`, `program`, read off the
    /// transcript node itself.
    #[must_use]
    pub const fn the_word(self) -> alo_strings::Word {
        match self {
            Self::AProgram => words::A_PROGRAM,
            Self::Alo => words::ALO,
            Self::TheSystem => words::THE_SYSTEM,
            Self::ThePerson => words::THE_PERSON,
        }
    }

    /// Every origin, in the order the design's transcript shows them.
    ///
    /// For tests that must cover all of them and for a vocabulary check that
    /// must not be able to miss one. A list a reader can see is the whole list
    /// is the point; `every_origin_has_a_word_of_its_own` walks it.
    pub const ALL: [Self; 4] = [Self::ThePerson, Self::Alo, Self::TheSystem, Self::AProgram];
}

/// One run of transcript lines that share an origin.
///
/// **Contiguous, and labelled once.** ADR 0113: *each region is labelled once,
/// not each line* — which the design draws plainly, an `alo` region three lines
/// tall carrying one `alo` in the gutter.
///
/// Lines are counted from the top of what is displayed, so `from` is the line
/// the label is drawn beside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ARegion {
    /// Who produced these lines.
    pub who: WhoIsSpeaking,
    /// The first line of the region, counted from the top of the display.
    pub from: usize,
    /// How many lines the region covers. Never zero: a region of no lines is
    /// not a region, and [`regions_of`] does not produce one.
    pub lines: usize,
}

impl ARegion {
    /// Whether this line falls inside the region.
    #[must_use]
    pub const fn holds(self, line: usize) -> bool {
        line >= self.from && line < self.from + self.lines
    }

    /// Whether the label is drawn beside this line.
    ///
    /// **Once per region**, which is the whole of ADR 0113's rule about
    /// repetition, so it is one function rather than a condition every caller
    /// writes for itself.
    #[must_use]
    pub const fn labelled_at(self, line: usize) -> bool {
        line == self.from
    }
}

/// The regions of a transcript this many lines tall, all of them a program's.
///
/// **The whole display, as one region.** A pseudoterminal is the only writer
/// today, so there is exactly one origin on screen and splitting it would be
/// inventing a boundary nobody drew.
///
/// It takes the line count rather than reading the screen, because nothing
/// about the answer depends on what the screen says — which is the rule at the
/// top of this file, expressed in a signature. A function given the text could
/// be changed to look at it; this one has nothing to look at.
///
/// Answers an empty list for an empty display, so a caller drawing labels draws
/// none rather than one beside a line that is not there.
#[must_use]
pub fn regions_of(lines: usize) -> Vec<ARegion> {
    if lines == 0 {
        return Vec::new();
    }
    vec![ARegion {
        who: WhoIsSpeaking::AProgram,
        from: 0,
        lines,
    }]
}

#[cfg(test)]
#[path = "who_is_speaking_tests.rs"]
mod tests;
