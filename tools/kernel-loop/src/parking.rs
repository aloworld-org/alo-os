//! Putting a task that did not gate onto a branch of its own, with what names
//! it carried along.
//!
//! The git of parking is [`crate::repository::parked`], where every git this
//! loop can run lives. This is the step before it and the step after: deciding
//! what the branch carries besides the work, and taking the loop's own copy
//! back once the branch has it.
//!
//! # What a parked branch carries, and why it has to
//!
//! A parked branch is a photograph of the tree plus **one file that names the
//! task** — `.kernel-loop/handoff.toml`, force-added because the directory is
//! ignored. That file is what makes the branch recoverable anywhere: the task,
//! the files, the evidence, the report. On the road that produces most parks
//! there is no such file. The gates refuse a first worker, the repair path moves
//! that worker's handoff into `.kernel-loop/refused/`, and the second worker is
//! stopped before writing its own. Task 35 taught `recover` to reconstruct
//! from `refused/` — on the checkout that parked the branch, and nowhere else,
//! because `refused/` is inside the ignored directory and a copy of the branch
//! does not bring it. The one reason parked work is a branch rather than a
//! stash is that a branch can be copied.
//!
//! So parking now carries the newest refused handoff for the task **onto the
//! branch**, under a name of its own — `.kernel-loop/refused-handoff.toml` —
//! when there is no handoff waiting to carry instead. A distinct name, so that
//! nothing reads a handoff the gates refused as one the parked worker wrote:
//! `recover` reports a branch recovered from it as reconstructed, differences
//! and all, exactly as it reports one reconstructed from `refused/`.
//!
//! # What it still does not do
//!
//! It pushes nothing; `main` is the only branch this loop publishes. It never
//! writes `.kernel-loop/handoff.toml` itself — that is the worker's word, and a
//! supervisor that wrote it would be inventing one. Parking with neither a
//! handoff nor a refused one still parks: a branch with the work and no name
//! is still better than no branch, and `recover` says in words what it could
//! not find.

use std::path::{Path, PathBuf};

use crate::handoff::Handed;
use crate::repository;

/// What parking did.
#[derive(Debug)]
pub struct Parked {
    /// The local branch the work is on.
    pub branch: String,

    /// The refused handoff the branch carries in place of one the worker
    /// wrote, named by the entry it was copied from — or [`None`] when the
    /// branch carries the worker's own handoff, or nothing at all.
    pub carried: Option<PathBuf>,
}

/// Put a task's unfinished work on a branch of its own, carrying the newest
/// refused handoff for it when the worker left none.
///
/// The task is named twice because the two halves of parking know it two
/// ways: the branch is named for the **number**, as [`repository::parked`]
/// has always named it, and a refused handoff names the **name**, which is
/// what [`Handed::carried_for_parking`] looks for.
///
/// # Errors
/// A sentence when the refused handoff could not be copied or git refused a
/// step, in which case the work is still in the tree and the loop stops —
/// which is the right answer for a checkout that will not do as it is asked.
/// The copy is taken back before the error is handed on, so a failed park
/// leaves no file for the next one to carry under the wrong task.
pub fn parked(
    at: &Path,
    ours: &Path,
    number: u32,
    named: &str,
    why: &str,
) -> Result<Parked, String> {
    let carried = Handed::carried_for_parking(ours, named)?;
    let branch = repository::parked(at, number, why);
    // Git has already taken it off the working tree when the switch back to
    // `main` succeeded, because the branch tracks it and `main` does not. This
    // is for the park that did not get that far.
    Handed::the_carried_copy_is_taken_back(ours);
    Ok(Parked {
        branch: branch?,
        carried,
    })
}
