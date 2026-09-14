//! Everything under a folder, gathered by as many walks as its size needs.
//!
//! `alo_files::MOST_WALKED` is how many things one walk looks at, so that a
//! verb over a granted folder is bounded whatever the folder holds. A
//! person's photo library or a source tree with its dependencies is more than
//! that, and an index that stopped where one walk stopped would be a search
//! box that never finds the rest. So the index **walks on**, and the walking
//! on is `alo-files`' — `alo_files::Walking::throughout`, the same walker
//! asked again from every folder it named and did not enter, under the same
//! bound, until nothing is left unentered, every `below` spelled from the
//! folder the person named. It was written here first, for the index, and
//! moved next to the walk when the tree of sizes in `alo-measuring` needed
//! the same thing: two copies would be two opinions about which folder is
//! walked again and what is counted once.
//!
//! # What still cannot be made whole
//!
//! A folder holding more than `most` things **at one level** stops every walk
//! in the same place, so it is walked once more and, when that walk stops
//! inside it again, left in [`Gathered::not_entered`]; the gathering is not
//! [`Gathered::whole`], and the sentence above the index says a folder holds
//! more than one walk looks at. A folder the machine would not let a later
//! walk read is an unread folder in the gathering, exactly as one the first
//! walk stepped over, so that *nothing matched* is never said about a folder
//! nobody looked at.
//!
//! # One file walks
//!
//! This is the one file in the crate that names the walker, and
//! `tests/nothing_here_opens_a_socket_or_asks_anybody.rs` holds it to one:
//! the index walks with `alo-files` and with nothing else, under a measuring
//! policy — a link is a step with its own bytes and is never followed, a
//! folder the machine would not read is noted rather than fatal, and a folder
//! on another filesystem is noted and not entered.

use std::path::Path;

use alo_files::{Failed, Walking};

pub(crate) use alo_files::Gathered;

/// Everything under `folder`, gathered by walks of at most `most` things
/// each, walked on from every folder a walk left unentered.
///
/// # Errors
///
/// [`Failed`] when the folder itself is not there, is a file, or could not
/// be read — the first walk's own refusal. A folder found *inside* it that a
/// later walk could not read is an unread folder in the gathering, not an
/// error, because the rest of the gathering is still true.
pub(crate) fn everything_under(folder: &Path, most: usize) -> Result<Gathered, Failed> {
    Walking::measuring(most).throughout(folder)
}
