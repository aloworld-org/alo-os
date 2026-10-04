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
//! # A file that does not read is kept, not lost
//!
//! [`at_sign_in`] answers a fresh arrangement and **says what was wrong**
//! rather than refusing to start: a person whose canvas file was damaged gets
//! their windows where the applications put them, not a session that will not
//! begin.
//!
//! And it **moves that file aside** to [`THE_ONE_THAT_DID_NOT_READ`] rather than
//! leaving it for the next save to overwrite. The owner decided this on
//! 2026-10-03, and the case that forces it is not damage but **age**: `read`
//! refuses a file whose version is not this one, by strict equality, so the
//! first format change after a release would otherwise discard every person's
//! arrangement — windows back at the origin, the camera reset, nothing said, and
//! from their side nothing went wrong. A file written by an older version of our
//! own format **is not wrong, it is old**, and one equality cannot tell those
//! apart. The distinction is the panel lane's, reading this crate on the day it
//! was written.
//!
//! Moving it aside costs a migration nothing and keeps every option open: the
//! bytes are still there, so a later release may read them, somebody may be told
//! about them, or they may simply be what proves what was lost.
//!
//! ## Why this rather than refusing to write
//!
//! `alo_kept::keep` refuses to write over a file that did not read — ADR 0038's
//! clause 3 — and for a settings file that is right: a person's hand edit with
//! one mistake in it is theirs to mend. `alo_kept::kept_text` makes no such
//! check, and **a canvas layout is the case where it should not.** Nobody
//! curates this file by hand, so refusing forever would leave somebody with a
//! canvas that could never remember anything again until they found and deleted
//! a file they do not know exists. Aside, then written cleanly, keeps both
//! halves: nothing destroyed, and nothing stuck.

use std::path::Path;
use std::time::SystemTime;

use crate::{Arrangement, NotArranged};

/// What an unreadable layout is renamed to, so it is kept rather than lost.
///
/// **One slot, holding the most recent file that did not read.** Overwriting an
/// older one is right rather than careless: between two failures this crate
/// wrote at least one file that *did* read, so an earlier set-aside copy has
/// already been superseded by a layout the person then used.
pub const THE_ONE_THAT_DID_NOT_READ: &str = "canvas-layout.toml.unread";

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

/// This layout, kept as the whole of the file at `at` — and **what was kept**.
///
/// The answer is not the argument. What reaches the disk is this layout with
/// each Place's earlier states carried forward and the state being replaced
/// added to them, which is more than the caller handed over, and the caller
/// needs it: a ribbon is drawn from a series, and the series is assembled here.
///
/// **Returned rather than assembled twice.** The session holds the arrangement
/// it last kept, so it could run [`Arrangement::following`] itself and reach
/// the same answer — and then two places would be computing what a Place
/// remembers, agreeing by construction today and by luck after the first edit
/// to either. One assembler, and it hands back its work.
///
/// # Errors
///
/// [`NotKept::NotWritten`] when the file was not replaced — because the disk
/// refused a step, or because the bytes it handed back did not read as this
/// layout. **The file at `at` is as it was**, in both, and so is the series:
/// a caller that keeps the answer keeps nothing on a refusal.
pub fn keep(
    at: &Path,
    arrangement: &Arrangement,
    when: SystemTime,
) -> Result<Arrangement, NotKept> {
    // **The series can only be assembled here.** The shell builds an
    // arrangement from the live canvas each time it keeps one
    // (`the_arrangement_now` starts from `Arrangement::fresh`), so it has no
    // memory of what came before. What is on the disk does, and this is the
    // one moment both are in hand — so a Place's ribbon is carried forward
    // here, and the state being replaced joins it (`Arrangement::following`).
    //
    // **A file that does not read carries nothing forward, and is not an
    // error.** A person whose layout was damaged keeps their new one; what
    // they lose is a ribbon they could not have drawn from a file nothing can
    // parse. That is the same answer `at_sign_in` gives, for the same reason.
    let previous = std::fs::read_to_string(at)
        .ok()
        .and_then(|text| Arrangement::read(&text).ok());
    let keeping = previous.map_or_else(
        || arrangement.clone(),
        |was| arrangement.following(&was, when),
    );
    let text = keeping.written();
    alo_kept::kept_text(at, &text, |back| these_bytes_are(&keeping, back))
        .map_err(NotKept::NotWritten)?;
    Ok(keeping)
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
///
/// **And that file is moved aside rather than left to be overwritten** — see
/// this module's header for why age rather than damage is the case that forces
/// it. A rename the disk refuses changes nothing about the answer: the layout is
/// unreadable either way.
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
        Err(why) => {
            kept_aside(at);
            (Arrangement::fresh(), Some(NotReadBack::NotALayout(why)))
        }
    }
}

/// Move a layout that did not read beside itself, so it is kept.
///
/// **Best effort on purpose.** The caller's answer does not change: the layout
/// did not read, a fresh canvas is what a person gets, and the reason is already
/// in hand. A rename the disk refuses leaves the file where it was — no worse
/// than before this existed, and the next save overwriting it is the behaviour
/// this function exists to avoid rather than a new fault it introduces.
fn kept_aside(at: &Path) {
    let Some(folder) = at.parent() else {
        return;
    };
    let _moved = std::fs::rename(at, folder.join(THE_ONE_THAT_DID_NOT_READ));
}

#[cfg(test)]
mod tests;
