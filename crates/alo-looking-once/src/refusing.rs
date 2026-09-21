//! Why a machine did not come away from a start knowing whether there is an
//! update.
//!
//! Told apart by **what somebody can do about them**, which is the rule
//! `alo_looking::NoAnswer` is written to and is the reason this is a second
//! enumeration rather than a sixth member of that one: those five are answers a
//! check produced, and every one of them means the check happened. These are the
//! roads where it did not happen at all, or happened and could not be written
//! down — and each of them is a machine that is wrong in a way looking again
//! will not fix.
//!
//! **Nothing here is a sentence a person reads.** Every one of these is for
//! whoever administers the machine, in the journal, exactly as `alo-boundaryd`
//! and `alo-agentd` write theirs. A person is told there is an update by a
//! surface reading the answer that was kept; a start where nothing could be
//! kept tells them nothing, which is the honest outcome — the machine is as it
//! was, and the next start asks again.

use alo_keeping::NotKept as NotRecorded;
use alo_looking::{NoAnswer, NotKept as NotKeptAnswer};
use alo_updating::NotRead;

use crate::road::NoRoad;

/// Why this start did not end with an answer kept.
#[derive(Debug, thiserror::Error)]
pub enum DidNotLook {
    /// The pin this machine was built with does not name a place to ask.
    ///
    /// It cannot happen on a machine that shipped — `alo-image` refuses a
    /// malformed pin long before a build is made from it — and it is here
    /// rather than unwrapped because a program that panics tells whoever reads
    /// the journal less than a sentence does.
    #[error("the pin this machine was built with does not name a place to ask: {0}")]
    NoPlace(alo_looking::NotAPlace),
    /// The machine's proxy setting could not be turned into a road out.
    ///
    /// **The check is then not made.** Never made straight out instead, which
    /// would send a company's traffic around its own rule with nobody told.
    #[error("{0}")]
    NoRoad(#[from] NoRoad),
    /// The base would not say which build this machine is running.
    ///
    /// Without it there is nothing to compare an offer against, so there is no
    /// question to ask and no departure is made.
    ///
    /// **Read rather than asked, since 2026-09-21.** The base's own command
    /// refuses the person outright, so what this reads is what the base has
    /// already written down (`alo_updating::WrittenDown`) — the sentence stays
    /// the same because what it tells whoever reads the journal is the same:
    /// this machine does not know what it is running, and so it asked nothing.
    #[error("the base would not say which build this machine is running: {0:?}")]
    TheBaseWouldNotSay(NotRead),
    /// The record could not be opened, so nothing was asked.
    ///
    /// A machine that cannot write down what it is about to do does not do it.
    #[error(
        "nothing was asked, because the record it would be written in could not be opened: {0:?}"
    )]
    NoRecordToWriteIn(NotRecorded),
    /// The check was made and the departure could not be written down.
    ///
    /// Worse than the one above and said differently: the machine reached the
    /// network and its record cannot account for it, which is the one failure
    /// in this file that has already happened rather than been prevented.
    #[error("the check left this machine and could not be written into the record: {0:?}")]
    NotWrittenDown(NotRecorded),
    /// The check was refused, and this is what to say about it.
    #[error("nothing was found out: {0}")]
    NothingFoundOut(NoAnswer),
    /// The check was refused, and this machine has already said it has no way
    /// out.
    ///
    /// `alo_looking::SaidOnce`'s decision, honoured rather than worked around:
    /// there is no refusal left inside this to word, which is the whole point
    /// of it.
    #[error("nothing was found out, and this machine has already said it has no way out")]
    SaidAlready,
    /// The answer could not be kept, so no surface will read it back.
    #[error("the answer could not be kept: {0}")]
    NotKept(NotKeptAnswer),
}

impl DidNotLook {
    /// Whether this machine reached the network before it stopped.
    ///
    /// What whoever is reading the journal wants to know first, and what law 1
    /// makes a question worth being able to answer exactly: three of these
    /// happen before a single packet goes anywhere.
    #[must_use]
    pub const fn nothing_left_this_machine(&self) -> bool {
        matches!(
            self,
            Self::NoPlace(_)
                | Self::NoRoad(_)
                | Self::TheBaseWouldNotSay(_)
                | Self::NoRecordToWriteIn(_)
        )
    }
}
