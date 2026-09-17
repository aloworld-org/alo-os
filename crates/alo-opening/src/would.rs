//! What would open a file this machine cannot: said after why, every time.
//!
//! A refusal that stops at *why* is the dialogue with one button. A person who
//! has just been told a file will not open is holding one question — *so what
//! do I do?* — and each [`crate::Cannot`] answers it with exactly one [`Would`].
//!
//! # A closed list, and where each one sends a person
//!
//! Five answers, and the difference between them is the direction they send
//! somebody in:
//!
//! - **back to whoever has the original** — for a file with nothing in it or one
//!   that does not hold together, because the file is the problem and a
//!   complete copy is the fix ([`Would::ACompleteCopy`]);
//! - **to whoever sent it, for a copy without its lock**
//!   ([`Would::ACopyWithoutThePassword`]);
//! - **to whoever sent it, for the document they meant** — a program is never
//!   opened, so the only thing that helps is the document itself
//!   ([`Would::TheDocumentItself`]);
//! - **to another machine, or to a different format** — the file is fine and
//!   this machine has nothing for it ([`Would::AnotherMachineOrFormat`]);
//! - **to whoever sent it, to say what it is** — this machine does not know,
//!   so it cannot name a machine that would ([`Would::WhoeverMadeIt`]).
//!
//! *Damaged* and *not recognised* map to different answers, and a test holds
//! that: one sends a person back for another copy, the other onwards to another
//! program, and a person sent the wrong way wastes a day.
//!
//! # What is never an answer
//!
//! **Sending the file somewhere to find out.** No answer here offers to upload
//! it, look it up or hand it to a service — that offer is the sentence this list
//! exists to replace, and on a machine sold on sovereignty it is an errand the
//! person did not ask for. **Guessing** is not an answer either: nothing here
//! says a file *might* open somewhere, and no answer names a program, a library
//! or anything this machine rents.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// What would open a file this machine cannot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Would {
    /// A complete copy, from whoever has the original.
    ACompleteCopy,
    /// A copy saved without the password, from whoever sent it.
    ACopyWithoutThePassword,
    /// The document itself, rather than a program, from whoever sent it.
    TheDocumentItself,
    /// A machine with a program for this kind of file, or a copy saved in a
    /// different format.
    AnotherMachineOrFormat,
    /// Whoever sent it saying what made it, or sending a copy saved in a
    /// different format.
    WhoeverMadeIt,
}

impl Would {
    /// Every answer, in the order [`crate::words::THE_REMEDIES`] words them.
    pub const EVERY: [Self; 5] = [
        Self::ACompleteCopy,
        Self::ACopyWithoutThePassword,
        Self::TheDocumentItself,
        Self::AnotherMachineOrFormat,
        Self::WhoeverMadeIt,
    ];

    /// The word this answer is said with.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::ACompleteCopy => words::WOULD_A_COMPLETE_COPY,
            Self::ACopyWithoutThePassword => words::WOULD_A_COPY_WITHOUT_THE_PASSWORD,
            Self::TheDocumentItself => words::WOULD_THE_DOCUMENT_ITSELF,
            Self::AnotherMachineOrFormat => words::WOULD_ANOTHER_MACHINE_OR_FORMAT,
            Self::WhoeverMadeIt => words::WOULD_WHOEVER_MADE_IT,
        }
    }

    /// The sentence a person reads, after the one saying why.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every answer is worded by the word in its place in the list**, so the
    /// list of answers and the list of words cannot drift apart.
    #[test]
    fn every_answer_is_worded_by_its_own_word() {
        assert_eq!(Would::EVERY.len(), words::THE_REMEDIES.len());
        for (would, word) in Would::EVERY.iter().zip(words::THE_REMEDIES.iter()) {
            assert_eq!(would.word(), *word, "{would:?}");
        }
    }
}
