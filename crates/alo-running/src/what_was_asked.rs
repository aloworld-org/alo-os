//! What was asked for, before anything checked it.
//!
//! [`crate::ARun`] is a request that has been checked. **This is one that has
//! not**, and it exists because of where the checking has to happen.
//!
//! # The hole this closes
//!
//! `ARun::checked` refuses four things: a level that is not the sealed box, a
//! relative program, a program that is not there, and a run naming nowhere to
//! reach. Until now a caller called it *before* [`crate::WhereCommandsRun::ran`]
//! — so each of those four refusals happened outside the only road that writes a
//! record, and **a request refused on the way in left no trace at all.**
//!
//! `CLAUDE.md`'s gate is *every execution **and every refusal** leaves a
//! record*. Four of this crate's eight refusals were outside it. That is not a
//! thing a test would have caught, because nothing was wrong with either half:
//! the checking was right and the recording was right, and the join was where
//! the promise fell through.
//!
//! So the unchecked request is the thing a caller hands over, the checking
//! happens inside, and there is one road in. [`crate::ARun`] stays public
//! because a surface wants to validate before it asks a person anything — what
//! it no longer is, is the way in.

use std::{ffi::OsString, path::PathBuf};

use alo_capability::HowCodeRuns;

/// A command somebody asked to run, as asked, before anything judged it.
///
/// Every field is public and nothing here is checked: that is the type's whole
/// purpose. [`crate::ARun::checked`] is what judges it, and
/// [`crate::WhereCommandsRun::ran`] is what calls that where the answer can be
/// written down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskedToRun {
    /// How the person said code may run on this machine.
    ///
    /// Passed in rather than read, for the reason `alo-capability` gives about
    /// every other choice: a person's choices live in their own settings, and
    /// this crate is handed the answer.
    pub chose: HowCodeRuns,
    /// The command, which has to be an absolute path and is not checked here.
    pub program: PathBuf,
    /// What to give it, unchanged and never parsed.
    pub arguments: Vec<OsString>,
    /// The folders it may reach.
    pub may_reach: Vec<PathBuf>,
}
