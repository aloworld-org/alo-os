//! Where a person's own name is given to their desktop, and how it is read
//! back.
//!
//! # Why the name needs a second home
//!
//! The account store ([`crate::THE_ACCOUNTS`]) is root's and mode `0600`,
//! because it holds every account's password hash. The sign-in screen reads it
//! and the person's desktop cannot — the desktop runs as the person's own
//! login, which must never be able to read anybody's hash. So the one thing
//! the desktop needs from that file, **the name this person typed when they
//! made their account**, is copied out of it at every sign-in by
//! `alo-giving-a-name` into a file of its own:
//!
//! ```text
//! /var/lib/alo-names            0711 root:root   anybody may pass, only root may list
//! /var/lib/alo-names/<number>   0600 <number>    the person's, and nobody else's
//! ```
//!
//! [ADR 0136](../../../docs/decisions/0136-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)
//! is the decision and says why the three other roads were not taken.
//! `docs/contracts/the-persons-name.md` is the shape, for anything else that
//! wants to read it.
//!
//! # Read as something a person may have edited
//!
//! The file is the person's own, so they can write anything into it. That is
//! their right — it is their name on their machine — and it is also why
//! [`as_read`] believes only a bounded line of printable text: a desktop that
//! drew whatever bytes it found would draw a terminal escape, a line break or a
//! megabyte of text into a strip 18 units tall.
//!
//! # Everything here runs on any host
//!
//! No owner, no mode, no capability: this file names the place and the shape.
//! Writing the file and giving it to the person is `alo-giving-a-name`'s, and
//! it is the only thing that may (`crates/alo-giving-a-folder`'s
//! `only_here_may_give_a_directory_away.rs` holds both to that).

use std::io::Read as _;
use std::path::{Path, PathBuf};

/// Where each person's name is given to them, one file per account number.
pub const THE_NAMES: &str = "/var/lib/alo-names";

/// The mode the folder gets: anybody may pass through it, only root may list
/// it, so one person cannot learn who else has a name here.
pub const THE_NAMES_MODE: u32 = 0o711;

/// The mode one person's file gets: theirs, and nobody else's.
pub const A_NAMES_MODE: u32 = 0o600;

/// The most a name file is read to, in bytes.
///
/// A name the account store accepts is at most 32 bytes. The file is the
/// person's own and they may put a longer name in it, in any script, so this
/// is generous — 256 bytes is 64 characters of any script there is — and it is
/// still a bound: a desktop does not read a file of unknown size on the way to
/// drawing a frame.
pub const THE_LONGEST_NAME: usize = 256;

/// Where account `person`'s name is, under `under`.
///
/// `under` is [`THE_NAMES`] on a machine and a temporary folder in a test.
#[must_use]
pub fn where_their_name_is(under: &Path, person: u32) -> PathBuf {
    under.join(person.to_string())
}

/// The file's text for this name: the name, and one line ending.
#[must_use]
pub fn as_written(name: &str) -> String {
    format!("{name}\n")
}

/// The name a file's text holds, if it holds one worth showing.
///
/// One line ending is taken off the end, then the rest is believed only if it
/// is not empty, is at most [`THE_LONGEST_NAME`] bytes, and contains no
/// control character — a line break, a tab or a terminal escape is not part of
/// a name, and drawing one would draw something nobody typed.
///
/// [`None`] is *nothing to show*, which the desktop draws as no name rather
/// than as an empty one.
#[must_use]
pub fn as_read(text: &str) -> Option<String> {
    let line = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(text);
    let believed =
        !line.is_empty() && line.len() <= THE_LONGEST_NAME && !line.chars().any(char::is_control);
    believed.then(|| line.to_owned())
}

/// Why a name file could not be read back.
#[derive(Debug, thiserror::Error)]
pub enum NameNotRead {
    /// The file is there and the machine would not read it.
    #[error("{at} could not be read, so this desktop shows no name: {why}")]
    NotRead {
        /// Which file.
        at: String,
        /// What the machine said.
        why: String,
    },
    /// The file holds something that is not one short line of text.
    #[error(
        "{at} does not hold one line of printable text of at most {THE_LONGEST_NAME} bytes, so \
         this desktop shows no name"
    )]
    NotAName {
        /// Which file.
        at: String,
    },
}

/// Account `person`'s name, as it was last given to them under `under`.
///
/// `Ok(None)` when no file is there — an account whose person has not signed in
/// since this was built, or a machine where `alo-giving-a-name` could not run.
/// That is the ordinary state of a machine before its first sign-in and is not
/// a failure.
///
/// # Errors
///
/// [`NameNotRead`] when the file is there and could not be read, or held
/// something [`as_read`] does not believe.
pub fn their_name(under: &Path, person: u32) -> Result<Option<String>, NameNotRead> {
    let at = where_their_name_is(under, person);
    let file = match std::fs::File::open(&at) {
        Ok(file) => file,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(why) => {
            return Err(NameNotRead::NotRead {
                at: at.display().to_string(),
                why: why.to_string(),
            });
        }
    };
    // One byte more than a name may be, so a longer file is seen to be longer
    // rather than silently cut to a name nobody typed.
    let mut bytes = Vec::with_capacity(THE_LONGEST_NAME + 3);
    file.take(u64::try_from(THE_LONGEST_NAME + 3).unwrap_or(u64::MAX))
        .read_to_end(&mut bytes)
        .map_err(|why| NameNotRead::NotRead {
            at: at.display().to_string(),
            why: why.to_string(),
        })?;
    String::from_utf8(bytes)
        .ok()
        .and_then(|text| as_read(&text))
        .map(Some)
        .ok_or_else(|| NameNotRead::NotAName {
            at: at.display().to_string(),
        })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_name_given_tests.rs"]
mod tests;
