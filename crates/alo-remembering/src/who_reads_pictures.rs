//! **Who reads the pictures alo takes of a window**, kept in the level file
//! beside the switches for pictures and keys (ADR 0147 §4;
//! `docs/contracts/how-code-runs-file.md`, `pictures-read-by`).
//!
//! The same file, so the same trust (`crate::how_code_runs`): **an agent cannot
//! choose a reader for itself**, because the agent is another login and a file
//! another login could have written is refused here rather than believed. A
//! reader that meets a refusal answers **no one**
//! ([`alo_capability::WhoReadsPictures::as_read`]): a picture is never sent
//! anywhere on the word of a file somebody else may have written.
//!
//! [`who_reads_pictures_chosen`] reads what the file says **at the moment of
//! the choice** and changes the reader alone, so every other answer in it is
//! kept. Over a file that would not read it is refused rather than written,
//! for the terminal's reason: writing it would throw away a level the person
//! may have chosen.

use std::path::Path;

use alo_capability::WhoReadsPictures;

use crate::refusing::NotRemembered;

/// What reads the pictures alo takes, as kept at this path.
///
/// **No file, and a file that says nothing about a reader, are no one** — how
/// a machine arrives.
///
/// # Errors
///
/// Every refusal [`crate::the_level_remembered`] makes, for the same file. **A
/// reader that meets one must answer no one.**
pub fn who_reads_pictures_remembered(at: &Path) -> Result<WhoReadsPictures, NotRemembered> {
    let _ = at;
    Ok(WhoReadsPictures::NoOne)
}

/// The reader chosen, written whole to this path with every other answer in it
/// kept; what the file then says.
///
/// # Errors
///
/// Every refusal [`crate::the_level_remembered`] makes, when the file is there
/// and would not read — **written over, it would lose a level the person
/// chose**; [`NotRemembered::NotTheLevel`] carrying
/// [`crate::NotTheLevel::NotAPicturesReader`] for a service by a name
/// [`WhoReadsPictures::a_provider`] would not make, which is never written; and
/// [`NotRemembered::NotWritten`] for a file that could not be written.
pub fn who_reads_pictures_chosen(
    at: &Path,
    chosen: WhoReadsPictures,
) -> Result<WhoReadsPictures, NotRemembered> {
    let _ = at;
    Ok(chosen)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "who_reads_pictures_tests.rs"]
mod tests;
