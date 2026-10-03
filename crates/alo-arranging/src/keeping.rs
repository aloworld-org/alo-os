//! The canvas layout, kept in a person's own folder.
//!
//! # Canvas layout persistence, which is not monitor arrangement persistence
//!
//! Two different things in this repository are called an arrangement, and only
//! one of them is this. **This file remembers where a person put their windows
//! and where they were looking** — Places, frames, geometry, and each Place's
//! camera. `alo-displays` remembers **where their monitors are**, and keeps its
//! own file by its own road.
//!
//! The names are close enough that asking *is the arrangement kept* finds the
//! other one and answers yes about the wrong thing, which is how this promise
//! read `Done` for four days while nothing wrote a file. So the words **canvas
//! layout** are used here and in the tests, and the file is named for them.
//!
//! # Whole or nothing, and proved rather than assumed
//!
//! The write is `alo_kept::kept_text`: a sibling written, synced, read back,
//! asked about, and renamed over the real file, with every failure happening
//! before the rename. What this crate adds is the asking — the bytes the disk
//! handed back are parsed by [`Arrangement::read`], with every position and
//! size rebuilt through the checked constructors, and compared to the
//! arrangement that was meant. **So a layout reaches a person's folder only if
//! what is on the disk reads back as the layout being saved**, which is a
//! stronger promise than a write that returned success.
//!
//! # A file that does not read is not written over
//!
//! [`at_sign_in`] answers a fresh arrangement and **says what was wrong**
//! rather than refusing to start: a person whose canvas file was damaged gets
//! their windows where the applications put them, not a session that will not
//! begin. And [`keep`] is given whatever is in memory, so the next save
//! replaces a damaged file with a good one — the damage costs a person their
//! remembered layout and nothing else.

use std::path::Path;

use crate::{Arrangement, NotArranged};

/// The file's name inside the person's folder.
///
/// **Named for canvas layout and not for arrangement**, because the other
/// arrangement in this repository is the monitors' and the two were confused
/// for four days.
pub const THE_FILE: &str = "canvas-layout.toml";

/// Why a canvas layout was not kept.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotKept {
    /// The disk refused a step, or the bytes it handed back were not the
    /// layout. **The file is as it was** in every case.
    ///
    /// Shown with `Debug` because that is what `alo-kept` offers: its refusals
    /// carry no words of their own, on purpose — the crate that owns a file
    /// writes the sentence a person reads, and this crate's sentence is the one
    /// above.
    #[error("the canvas layout was not kept: {0:?}")]
    NotWritten(alo_kept::Unwritten),
}

/// Why a canvas layout did not read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotReadBack {
    /// The file is there and the disk would not hand it over.
    #[error("the canvas layout could not be read from the disk: {0}")]
    Disk(std::io::ErrorKind),
    /// The file is there and is not a canvas layout this release reads —
    /// another version, a hand edit off the plane, or text that is not the
    /// shape at all. The sentence names which.
    #[error("the canvas layout is there and did not read: {0}")]
    NotALayout(NotArranged),
}

/// This layout, kept as the whole of the file at `at`.
///
/// # Errors
///
/// [`NotKept::NotWritten`] when the file was not replaced — because the disk
/// refused a step, or because the bytes it handed back did not read as this
/// layout. **The file at `at` is as it was**, in both.
pub fn keep(at: &Path, arrangement: &Arrangement) -> Result<(), NotKept> {
    let text = arrangement.written();
    alo_kept::kept_text(at, &text, |back| these_bytes_are(arrangement, back))
        .map_err(NotKept::NotWritten)
}

/// Whether the bytes a disk handed back are this layout.
///
/// **The proof, and the reason `keep` is not a plain write.** The bytes are read
/// by the same validated road a sign-in takes — every position and size rebuilt
/// through the checked constructors — and compared to the layout that was meant.
/// A filesystem that answered a sync without keeping what it was given is then a
/// refusal that leaves the person's file alone, rather than a layout that reads
/// as something else tomorrow.
///
/// **A function with a name rather than a closure, because a closure could not
/// be tested.** No unit test can make a disk lie; this does not need one, since
/// the comparison is a function of a layout and some bytes. It was an inline
/// closure until 2026-10-03, when removing its body entirely left all
/// twenty-one tests passing — the guarantee was real in the code and held by
/// nothing.
///
/// # Both failures are one refusal, deliberately
///
/// Whether the bytes did not read at all or read as a *different* layout, what
/// is true is the same: the disk is not holding this one, and the file being
/// replaced is untouched either way. `alo-kept`'s vocabulary has no shape for
/// *this crate's own parse refused*, and inventing one to carry a distinction
/// no caller can act on would be a wider surface for no answer.
///
/// # Errors
///
/// [`alo_kept::Unwritten::ReadBackAsSomethingElse`] for bytes that are not this
/// layout, by any route.
fn these_bytes_are(arrangement: &Arrangement, back: &[u8]) -> Result<(), alo_kept::Unwritten> {
    let Ok(text) = std::str::from_utf8(back) else {
        return Err(alo_kept::Unwritten::ReadBackAsSomethingElse);
    };
    match Arrangement::read(text) {
        Ok(read_back) if &read_back == arrangement => Ok(()),
        Ok(_) | Err(_) => Err(alo_kept::Unwritten::ReadBackAsSomethingElse),
    }
}

/// The layout a person left, read from the file at `at`.
///
/// **A missing file is not an error**: a person who has never arranged anything
/// has no file, and the answer is [`Arrangement::fresh`] with nothing said. A
/// file that is there and does not read answers a fresh arrangement **and** the
/// reason, so a session starts and somebody can be told why their windows are
/// where the applications put them.
pub fn at_sign_in(at: &Path) -> (Arrangement, Option<NotReadBack>) {
    let text = match std::fs::read_to_string(at) {
        Ok(text) => text,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
            return (Arrangement::fresh(), None);
        }
        Err(why) => return (Arrangement::fresh(), Some(NotReadBack::Disk(why.kind()))),
    };
    match Arrangement::read(&text) {
        Ok(arrangement) => (arrangement, None),
        Err(why) => (Arrangement::fresh(), Some(NotReadBack::NotALayout(why))),
    }
}

#[cfg(test)]
mod tests;
