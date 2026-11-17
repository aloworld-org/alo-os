//! **Which applications alo may use pictures and keys in**, kept in the level
//! file beside the level (ADR 0143 Q11; `docs/contracts/how-code-runs-file.md`,
//! `pictures-and-keys-in`).
//!
//! The same file, so the same trust (`crate::how_code_runs`): **an agent cannot
//! switch it on for itself**, because the agent is another login and a file
//! another login could have written is refused here rather than believed.
//!
//! # Read at each act
//!
//! [`pictures_and_keys_remembered`] reads the file each time it is asked, and
//! a reader that meets a refusal answers **no application**
//! ([`alo_capability::PicturesAndKeys::as_read`]): a file somebody else may
//! have written cannot speak for the person.
//!
//! # One application switched at a time, the rest kept
//!
//! [`pictures_and_keys_switched`] reads what the file says **at the moment of
//! the press** and changes one application in it, so a second writer's answer
//! is never written back over. Every other key in the file is kept. Over a
//! file that would not read it is **refused** rather than written, for the
//! terminal's reason: writing it would throw away a level the person may have
//! chosen.

use std::path::Path;

use alo_capability::PicturesAndKeys;

use crate::how_code_runs::{kept, what_is_kept};
use crate::refusing::NotRemembered;
use crate::the_level::WhatTheFileSays;

/// The applications alo may use pictures and keys in, as kept at this path.
///
/// **No file, and a file that says nothing about pictures, are none** — how a
/// machine arrives.
///
/// # Errors
///
/// Every refusal [`crate::the_level_remembered`] makes, for the same file. **A
/// reader that meets one must answer no application.**
pub fn pictures_and_keys_remembered(at: &Path) -> Result<PicturesAndKeys, NotRemembered> {
    Ok(what_is_kept(at)?
        .and_then(|says| says.pictures_and_keys_in)
        .unwrap_or_default())
}

/// Pictures and keys switched on or off in one application, written whole to
/// this path with every other answer in it kept; what the file then says.
///
/// An identifier [`PicturesAndKeys::switched_on`] would not keep — a terminal
/// above all — is not switched on, and the answer returned says so.
///
/// # Errors
///
/// Every refusal [`crate::the_level_remembered`] makes, when the file is there
/// and would not read — **written over, it would lose a level the person
/// chose** — and [`NotRemembered::NotWritten`] for a file that could not be
/// written.
pub fn pictures_and_keys_switched(
    at: &Path,
    application: &str,
    on: bool,
) -> Result<PicturesAndKeys, NotRemembered> {
    let before = what_is_kept(at)?.unwrap_or_default();
    let was = before.pictures_and_keys_in.clone().unwrap_or_default();
    let now = if on {
        was.switched_on(application)
    } else {
        was.switched_off(application)
    };
    kept(
        at,
        WhatTheFileSays {
            pictures_and_keys_in: Some(now.clone()),
            ..before
        },
    )?;
    Ok(now)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "pictures_and_keys_tests.rs"]
mod tests;
