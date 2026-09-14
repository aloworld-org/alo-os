//! The one question, asked: given this file, what can this machine do with it?
//!
//! [`decide`] takes an open file, the name it was given and what this machine
//! has, and answers with a [`Decided`]. In that order and only that order:
//!
//! 1. **The bytes decide what the file is** (`crate::looking`). The name is not
//!    consulted, and could not change the answer if it were.
//! 2. **What the machine has decides the outcome**: open as it is, convert a
//!    copy, or cannot — and why.
//! 3. **The name is compared with what the bytes said** (`crate::naming`), and
//!    a disagreement becomes the finding the outcome is wrapped in.
//!
//! # Deciding is not doing
//!
//! Nothing here opens, converts, prints or runs anything, and nothing reads a
//! byte that no rule asked for. Nothing here asks anything off this machine
//! either — there is no network in this crate or under it, so *what is this
//! file* can never be an errand a person did not know they had sent.
//!
//! # A file that cannot be read is not a file that cannot be opened
//!
//! When the file itself will not answer — it was removed while being looked at,
//! or the disk under it stopped — nothing was learned, and [`Unreadable`] says
//! exactly that. It is an error rather than a fourth outcome, because an
//! outcome is a statement about the file and this is the absence of one.

use std::ffi::OsStr;
use std::io::{self, Read, Seek};

use alo_strings::{Filling, Said, Strings};

use crate::appears::{Appears, Macros};
use crate::decided::Decided;
use crate::looking;
use crate::machine::{Handles, ThisMachine};
use crate::naming::Named;
use crate::outcome::{Cannot, Costs, Outcome};
use crate::reading::Reading;
use crate::words;

/// A file that would not be read, so nothing was decided about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unreadable {
    /// What the file said when it was asked, for a log. Never shown to a
    /// person: a return code is not a sentence.
    why: io::ErrorKind,
}

impl Unreadable {
    /// What the file said when it was asked.
    #[must_use]
    pub const fn why(self) -> io::ErrorKind {
        self.why
    }

    /// The sentence a person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&words::UNREADABLE.key(), &Filling::nothing())
    }
}

impl From<io::Error> for Unreadable {
    fn from(error: io::Error) -> Self {
        Self { why: error.kind() }
    }
}

/// Decide what this machine can do with a file.
///
/// `file` is a file somebody allowed to be opened, already open; `name` is the
/// name it was given, used only to notice when it lies. Every read says where it
/// starts, so the file's position when it is handed over does not matter — and
/// it is left wherever the last read ended, so whoever reads it next seeks
/// first.
///
/// # Errors
/// [`Unreadable`] when the file will not answer, so nothing was decided.
pub fn decide<F: Read + Seek>(
    file: &mut F,
    name: &OsStr,
    machine: &ThisMachine,
) -> Result<Decided, Unreadable> {
    let mut reading = Reading::of(file)?;
    let looked = looking::look(&mut reading)?;
    let outcome = outcome_of(looked.appears, looked.macros, machine);
    Ok(Decided::of(Named::from_the_name(name), outcome))
}

/// What a file's bytes say, as this machine's answer about it.
fn outcome_of(appears: Appears, macros: Macros, machine: &ThisMachine) -> Outcome {
    match appears {
        Appears::A(kind) => match machine.handles(kind) {
            Handles::Opens => Outcome::OpensAsItIs { kind, macros },
            Handles::Converts(into) => Outcome::Converts {
                from: kind,
                into,
                costs: Costs::of(macros),
            },
            Handles::Nothing => Outcome::CannotOpen(Cannot::NothingHereOpens(kind)),
        },
        Appears::AProgram => Outcome::CannotOpen(Cannot::AProgram),
        Appears::APasswordProtectedDocument => Outcome::CannotOpen(Cannot::PasswordProtected),
        Appears::Nothing => Outcome::CannotOpen(Cannot::Empty),
        Appears::Unrecognised => Outcome::CannotOpen(Cannot::Unrecognised),
        Appears::Damaged(container) => Outcome::CannotOpen(Cannot::Damaged(container)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::appears::Container;
    use crate::kind::Kind;
    use crate::testing::in_english;

    /// **Every answer the bytes can give becomes the outcome that matches
    /// it**, so no outcome can claim to be about something the bytes did not
    /// say.
    #[test]
    fn every_answer_becomes_its_outcome() {
        let machine = ThisMachine::with_nothing()
            .opens(Kind::Pdf)
            .unwrap()
            .converts(Kind::WordDocument, Kind::Pdf)
            .unwrap();
        for appears in [
            Appears::A(Kind::Pdf),
            Appears::A(Kind::WordDocument),
            Appears::A(Kind::PngImage),
            Appears::AProgram,
            Appears::APasswordProtectedDocument,
            Appears::Nothing,
            Appears::Unrecognised,
            Appears::Damaged(Container::OlderOffice),
        ] {
            assert_eq!(
                outcome_of(appears, Macros::NoneSeen, &machine).appears(),
                appears
            );
        }
        assert!(matches!(
            outcome_of(Appears::A(Kind::WordDocument), Macros::Inside, &machine),
            Outcome::Converts { into: Kind::Pdf, costs, .. } if costs.macros() == Macros::Inside
        ));
    }

    /// **A file that will not answer is not decided about**, and what a person
    /// reads says nothing was learned rather than anything about the file.
    #[test]
    fn a_file_that_will_not_answer_is_not_decided_about() {
        /// A file whose disk stopped answering.
        struct Stopped;
        impl Read for Stopped {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("the disk stopped"))
            }
        }
        impl Seek for Stopped {
            fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
                Ok(64)
            }
        }

        let refused = decide(
            &mut Stopped,
            OsStr::new("report.pdf"),
            &ThisMachine::with_nothing(),
        )
        .unwrap_err();
        assert_eq!(refused.why(), io::ErrorKind::Other);
        assert_eq!(
            refused.said(&in_english()).text(),
            "This file could not be read, so nothing has been decided about it"
        );
    }
}
