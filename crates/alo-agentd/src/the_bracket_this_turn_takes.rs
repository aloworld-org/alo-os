//! Where this person's brackets go, and the one a turn under way has taken.
//!
//! `alo_turn::the_bracket_a_turn_takes` makes the snapshots; this says **whose
//! home and whose directory**, and holds the answer for the length of one turn.
//! ADR 0045 puts the taking in `alo-turn` and nothing in that crate learns which
//! person it is serving — deliberately, because a library cannot know which
//! person a process is.
//!
//! # The environment is the person, and it is read in one place
//!
//! `crate::settings`' reasoning, unchanged and for the same reason (ADR 0019
//! §3): `alo-agentd` runs **as the person, one instance per login**, started by
//! `systemd` inside their session — so `$HOME` as this process has it *is*
//! their home, and `geteuid` is them. Reading it is the daemon reading its own
//! environment rather than reaching into somebody's home directory.
//!
//! **That is a condition and not an assumption.** The day something serves two
//! people from one process, this becomes one person's turn bracketed against
//! another's home, and it becomes so silently. So [`WhereBracketsGo::of_a_session`] takes every
//! part as an argument and [`WhereBracketsGo::for_this_session`] is the only thing that reads the
//! environment, exactly as `crate::settings` is arranged, and the fix that day
//! is a caller's change.
//!
//! # A bracket is per changing turn, and is taken at the first change
//!
//! `docs/contracts/kept-undo-folder.md` draws **one directory per kept changing
//! turn**, and ADR 0045 takes the snapshot *before a changing turn*. A turn that
//! only answers questions changes nothing and needs no bracket.
//!
//! **That is not a tidiness: a read-only snapshot cannot be removed by the
//! person's own session.** Measured on a real filesystem: a person may create
//! one and may not delete it — that needs `CAP_SYS_ADMIN`, which is why ADR
//! 0045's seventh term puts expiry in a privileged unit. So a bracket taken for
//! every turn would leave an undeletable snapshot behind every question anybody
//! ever asked, and the person's own machine could not clear them.
//!
//! So [`TheTurnsBracket::before_a_change`] is called at each approval and takes
//! the snapshot **once**, on the first — which is still before anything changed,
//! because an approval is redeemed and only then run.
//!
//! # What it cannot yet say, and it is written down rather than papered over
//!
//! `alo_kept_undo::TheTurn` holds **one** sentence — *what it did, in the
//! sentence the person approved*. A turn may carry more than one approved
//! change (`Turning` keeps a list of what became of each). This records the
//! **first**, which is the whole truth for the ordinary turn and part of it for
//! a turn that changed two things.
//!
//! Joining several into one line would be this file inventing wording, which is
//! not its to invent; a second field is a change to a published contract. It is
//! named in `docs/autonomy/the-mac-lanes-order-while-the-owner-is-away.md` as
//! the owner's to settle.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_turn::the_bracket_a_turn_takes::{
    ASideOfTheTurn, WhereThisTurnIsKept, WhyNot, where_their_settings_are,
};

/// Where this person's own undo directory is, and what a bracket is taken of.
///
/// Made once per session and handed to each turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereBracketsGo {
    /// This person's own directory under the undo folder.
    whose: PathBuf,
    /// Their home, which is what a snapshot is taken of.
    home: PathBuf,
}

impl WhereBracketsGo {
    /// For the session this process is running in.
    ///
    /// The environment is read here and nowhere else in this crate's bracket
    /// road. [`None`] where the session names no home: a login with no home has
    /// nothing to snapshot, and that is a person's state rather than a fault.
    #[must_use]
    pub fn for_this_session() -> Option<Self> {
        Self::of_a_session(
            Path::new(alo_kept_undo::THE_FOLDER),
            rustix::process::geteuid().as_raw(),
            std::env::var_os(alo_choosing::HOME).as_deref(),
        )
    }

    /// The same, for a session named rather than inherited.
    ///
    /// `under` is the undo folder; `person` is their account number, which is
    /// what `alo_giving_a_folder::what_to_make` names their directory by — and
    /// the contract calls both names **opaque**, so nothing reads it back.
    #[must_use]
    pub fn of_a_session(under: &Path, person: u32, home: Option<&OsStr>) -> Option<Self> {
        let home = PathBuf::from(home?);
        if !home.is_absolute() {
            return None;
        }
        Some(Self {
            whose: under.join(person.to_string()),
            home,
        })
    }

    /// This person's own directory under the undo folder.
    #[must_use]
    pub fn whose(&self) -> &Path {
        &self.whose
    }

    /// Their home, which a bracket is taken of.
    #[must_use]
    pub fn home(&self) -> &Path {
        &self.home
    }
}

/// A name for one turn's directory, unique on this machine.
///
/// The contract: *a kept turn's name is whatever the process that took the
/// bracket chose*, and both directory names are opaque — nothing reads them,
/// parses them or orders by them. So this is built to be unique rather than to
/// be read: the second it began, this process, and which turn of this process
/// it is.
///
/// The process is in it because a counter alone resets when the daemon
/// restarts, and two turns in one second either side of a restart would
/// otherwise collide.
fn a_name_for(began: SystemTime, process: u32, which: u64) -> String {
    let seconds = began
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    format!("{seconds}-{process}-{which}")
}

/// The bracket of the turn under way: none yet, or the one it took.
///
/// Held beside the turn in `crate::Holding::ATurn`, so it begins and ends with
/// the turn and a round cannot be given one belonging to another.
#[derive(Debug)]
pub struct TheTurnsBracket {
    /// Where this person's brackets go, or [`None`] on a session with no home.
    ///
    /// **Owned rather than borrowed**, which costs two paths cloned once per
    /// turn and buys the absence of a `&'a mut T<'a>` in
    /// `crate::Holding::ATurn` — a shape that compiles only with the lifetimes
    /// tangled and that the next person to touch the carrier would have to
    /// untangle before changing anything else.
    goes: Option<WhereBracketsGo>,
    /// The name this turn's directory would have.
    named: String,
    /// The directory, once the first change has been approved.
    taken: Option<WhereThisTurnIsKept>,
    /// Why no bracket was taken, where that has been found out.
    why_not: Option<WhyNot>,
    /// The first sentence a person approved in this turn.
    did: Option<String>,
}

impl TheTurnsBracket {
    /// A turn that has taken no bracket yet.
    #[must_use]
    pub fn none_yet(
        goes: Option<WhereBracketsGo>,
        began: SystemTime,
        process: u32,
        which: u64,
    ) -> Self {
        Self {
            goes,
            named: a_name_for(began, process, which),
            taken: None,
            why_not: None,
            did: None,
        }
    }

    /// Take the bracket, if this is the first change this turn has approved.
    ///
    /// Called before each approval is redeemed and does its work **once**: see
    /// this module's header for why a bracket is per changing turn and not per
    /// turn.
    ///
    /// Answers nothing, because there is nothing a caller may do about it. ADR
    /// 0045's third term: *a machine too full to snapshot still runs the turn*,
    /// and *a verb that refused to run for want of an undo would be a machine
    /// that stopped working to protect a feature.* What the outcome is good for
    /// is [`Self::what_was_kept`], which a person reads.
    pub fn before_a_change(&mut self, said: &str) {
        if self.did.is_none() {
            self.did = Some(said.to_owned());
        }
        if self.taken.is_some() || self.why_not.is_some() {
            return;
        }
        let Some(goes) = self.goes.as_ref() else {
            return;
        };
        if let Err(why) = where_their_settings_are(goes.whose(), goes.home()) {
            self.why_not = Some(why);
            return;
        }
        match WhereThisTurnIsKept::made(goes.whose(), &self.named) {
            Err(why) => self.why_not = Some(why),
            Ok(kept) => match kept.keeping(alo_kept_undo::BEFORE, goes.home()) {
                ASideOfTheTurn::Kept(_) => self.taken = Some(kept),
                ASideOfTheTurn::NotKept(why) => self.why_not = Some(why),
            },
        }
    }

    /// Close the bracket: take the other side and write down what the turn did.
    ///
    /// Called once, as the turn ends. A turn that took no bracket does nothing
    /// here, which is the ordinary case — most turns answer questions.
    pub fn as_the_turn_ends(&mut self, ended: SystemTime) {
        let (Some(kept), Some(goes), Some(did)) = (&self.taken, self.goes.as_ref(), &self.did)
        else {
            return;
        };
        if let ASideOfTheTurn::NotKept(why) = kept.keeping(alo_kept_undo::AFTER, goes.home()) {
            self.why_not = Some(why);
        }
        if let Err(why) = kept.what_it_did(ended, did) {
            self.why_not = Some(why);
        }
    }

    /// What this turn kept, in the words a person reads.
    ///
    /// `NothingOnThisMachine` where the session had no home to snapshot;
    /// `NotThisTurn` where it was attempted and not taken — ADR 0045's third
    /// term; `EitherSideOfTheTurn` where both sides are there.
    ///
    /// **The comparing is somebody else's**, which is why the bracket handed
    /// back is empty: `alo-keeping-up`'s own words are *made by whoever compared
    /// them, which is not this crate*, and the comparing is the privileged
    /// path's with the base's own tools.
    #[must_use]
    pub fn what_was_kept(&self) -> alo_keeping_up::WhatWasKept {
        match (&self.taken, self.goes.as_ref()) {
            (Some(_), _) => {
                alo_keeping_up::WhatWasKept::EitherSideOfTheTurn(alo_keeping_up::Bracket::default())
            }
            (None, None) => alo_keeping_up::WhatWasKept::NothingOnThisMachine,
            (None, Some(_)) => alo_keeping_up::WhatWasKept::NotThisTurn,
        }
    }

    /// Why no bracket was taken, where that was found out.
    #[must_use]
    pub const fn why_not(&self) -> Option<&WhyNot> {
        self.why_not.as_ref()
    }

    /// Where this turn's bracket is, if it took one.
    #[must_use]
    pub fn taken_at(&self) -> Option<&Path> {
        self.taken.as_ref().map(WhereThisTurnIsKept::at)
    }

    /// The first sentence this turn approved.
    ///
    /// **Only a test reads this, which is why it is `cfg(test)`.** On a machine
    /// the sentence is read back out of `kept.json` beside the snapshots, and a
    /// test cannot get that far without a real filesystem — so without this
    /// door *the first sentence is the one kept* would be a claim nothing here
    /// checks.
    #[cfg(test)]
    pub(crate) fn the_first_sentence_for_tests(&self) -> Option<&str> {
        self.did.as_deref()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming the name a taker refused is the failure being reported"
)]
#[path = "the_bracket_this_turn_takes_tests.rs"]
mod tests;
