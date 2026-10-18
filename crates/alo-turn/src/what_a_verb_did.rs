//! What a verb did, when the verbs are no longer all one family.
//!
//! Until this file existed a turn answered with `alo_files::Answer` and nothing
//! else, because the only verbs this machine could carry out were the six file
//! verbs. [`crate::Machine`]'s own header says what that was: *the list an
//! agent may ask from is the list this machine can carry out*, and *adding an
//! executor and adding to the offered list are the same edit*. This is the
//! second executor arriving, so the answer stops being one crate's type.
//!
//! # Why an enum here and not a trait
//!
//! A trait would let a verb family answer with anything, and the thing on the
//! other end of a turn is a socket: `alo_protocol::Done` has to be able to put
//! every answer into words and onto the wire, and it can only do that for
//! answers it knows. **A closed list is what makes the wire total** — adding a
//! family means adding an arm here and a shape there, and the compiler names
//! every place that has to change. A trait would let a family ship an answer
//! nothing can send, which is the same fault as a verb nothing can execute one
//! level up.
//!
//! # It is never the verb's name
//!
//! Which arm this is says which *kind of thing happened*, not which verb ran.
//! Two file verbs answer `Answer::Moved` and the record is where the verb's
//! own name is kept. A caller that wants to know what was asked reads the
//! record or the authorisation, never this.

use alo_files::Answer;
use alo_record::WhatTouchedIt;
use alo_software::{Installed, WhatTheCatalogueHolds};

use crate::acting_on_an_application::Reached;

/// What a verb this machine carried out answered with.
///
/// One arm per family of verbs that has an executor here — which is the same
/// list as the verbs an agent may ask for, by [`crate::Machine`]'s rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatAVerbDid {
    /// One of the six file verbs: what was found, read, or where a file now is.
    ToFiles(Answer),
    /// An application was installed, and what it is sandboxed by.
    AnApplicationInstalled(Installed),
    /// An application was opened, brought to the front, or asked to close.
    ///
    /// It carries the act and the application rather than a word, because
    /// *which* application had *what* asked of it is the whole of what
    /// happened — the compositor's answer on success is one word, `acted`.
    AnApplicationWasReached(Reached),
    /// What the catalogue holds, under one kind of work or all of them.
    ///
    /// **The first arm here that is a read rather than a change**, which is
    /// what it is for: `install_application` beside it waits for an approval
    /// and this answers inside the turn (ADR 0001 §5). Nothing on the machine
    /// moved because of it, and `alo_software::WhatTheCatalogueHolds`' own
    /// header is why nothing about the person is in it either.
    WhatCanBeInstalled(WhatTheCatalogueHolds),
    /// What the record says was done to one file.
    ///
    /// **The second arm here that is a read**, and the first that answers out
    /// of what already happened rather than out of the machine as it stands.
    /// `alo_record::WhatTouchedIt` carries its own two qualifying sentences —
    /// that this is what alo did, and where nothing names the file, that
    /// nothing does — because an empty answer drawn as an empty list reads as
    /// a claim about the file that nobody made.
    WhatTouchedAFile(WhatTouchedIt),
}

impl WhatAVerbDid {
    /// What a file verb answered, when it was one.
    ///
    /// For the callers that only ever ask file verbs — of which there are
    /// several, because the six were the only family for a year — so that they
    /// read one method rather than matching an arm they can never take.
    #[must_use]
    pub const fn to_files(&self) -> Option<&Answer> {
        match self {
            Self::ToFiles(answer) => Some(answer),
            Self::AnApplicationInstalled(_)
            | Self::AnApplicationWasReached(_)
            | Self::WhatCanBeInstalled(_)
            | Self::WhatTouchedAFile(_) => None,
        }
    }

    /// What was installed, when an application was.
    #[must_use]
    pub const fn installed(&self) -> Option<&Installed> {
        match self {
            Self::AnApplicationInstalled(installed) => Some(installed),
            Self::ToFiles(_)
            | Self::AnApplicationWasReached(_)
            | Self::WhatCanBeInstalled(_)
            | Self::WhatTouchedAFile(_) => None,
        }
    }

    /// What the catalogue holds, when it was looked at.
    #[must_use]
    pub const fn what_can_be_installed(&self) -> Option<&WhatTheCatalogueHolds> {
        match self {
            Self::WhatCanBeInstalled(holds) => Some(holds),
            Self::ToFiles(_)
            | Self::AnApplicationInstalled(_)
            | Self::AnApplicationWasReached(_)
            | Self::WhatTouchedAFile(_) => None,
        }
    }

    /// What the record says was done to a file, when it was asked.
    #[must_use]
    pub const fn what_touched_a_file(&self) -> Option<&WhatTouchedIt> {
        match self {
            Self::WhatTouchedAFile(found) => Some(found),
            Self::ToFiles(_)
            | Self::AnApplicationInstalled(_)
            | Self::AnApplicationWasReached(_)
            | Self::WhatCanBeInstalled(_) => None,
        }
    }

    /// What was asked of an application, when one was reached.
    #[must_use]
    pub const fn reached(&self) -> Option<&Reached> {
        match self {
            Self::AnApplicationWasReached(reached) => Some(reached),
            Self::ToFiles(_)
            | Self::AnApplicationInstalled(_)
            | Self::WhatCanBeInstalled(_)
            | Self::WhatTouchedAFile(_) => None,
        }
    }
}

impl From<Answer> for WhatAVerbDid {
    fn from(answer: Answer) -> Self {
        Self::ToFiles(answer)
    }
}

impl From<Installed> for WhatAVerbDid {
    fn from(installed: Installed) -> Self {
        Self::AnApplicationInstalled(installed)
    }
}

impl From<Reached> for WhatAVerbDid {
    fn from(reached: Reached) -> Self {
        Self::AnApplicationWasReached(reached)
    }
}

impl From<WhatTheCatalogueHolds> for WhatAVerbDid {
    fn from(holds: WhatTheCatalogueHolds) -> Self {
        Self::WhatCanBeInstalled(holds)
    }
}

impl From<WhatTouchedIt> for WhatAVerbDid {
    fn from(found: WhatTouchedIt) -> Self {
        Self::WhatTouchedAFile(found)
    }
}

#[cfg(test)]
mod tests {
    use super::WhatAVerbDid;
    use alo_files::Answer;

    #[test]
    fn a_file_answer_is_not_an_installation_and_says_so() {
        let did = WhatAVerbDid::from(Answer::Read("hello".to_owned()));
        assert_eq!(did.to_files(), Some(&Answer::Read("hello".to_owned())));
        assert!(
            did.installed().is_none(),
            "a file read answered as an installation"
        );
    }
}
