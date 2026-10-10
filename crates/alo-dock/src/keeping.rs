//! Where what a person changed about their dock is kept: `dock.toml`, in their
//! own folder, read and written by this crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by. What is written is [`Changes`] — only the difference
//! — under a `format` number of this file's own. No file is a person who has
//! changed nothing. A file that is there and wrong is refused whole, in this
//! crate's words ([`FileNotRead`]), and the dock is where the release puts it.
//! A write is whole or not at all, and read back before it counts.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! **This crate does not know where the folder is.** It is handed the path, by
//! whoever starts the session, so that there is one answer to *where is a
//! person's folder* and it is not in a crate about a dock.
//!
//! **Nothing here watches the file.** A change made in Settings is drawn at
//! once, because Settings and the compositor are one process; a file edited by
//! hand is read at the next sign-in.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::changes::Changes;
use crate::dock::Dock;
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "dock.toml";

/// The shape of `dock.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: when the dock gains its size and one dock per
/// display at v0.5, this number is the dock's to move, and appearance's file
/// does not notice.
pub const FORMAT: i64 = 1;

/// Every key the file may have besides `format`.
///
/// **One of these is dead, and it is no longer `edge`.** Both `edge` and
/// `displays` were dead between [ADR
/// 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
/// fixing the dock to the bottom edge on 2026-09-29 and the setting being
/// offered again on 2026-10-10. **`edge` is live**: a `Changes` carries one, a
/// person's choice is written under it and read back from it.
///
/// `displays` is still dead. It held per-display exceptions, which
/// `docs/features.md` keeps at **[v0.5]** — *per display, so the dock can sit
/// along the bottom of the laptop and down the side of the external screen*.
///
/// **A dead key stays on this list because a key that is not on it is
/// refused.** A file with an unrecognised key in it does not read at all — that
/// is `dock.kept.unknown-key`, which is the right answer for a typo and the
/// wrong one for a file this project itself wrote last release. So such a file
/// reads and its `displays` is ignored. It leaves a person's folder on the next
/// write, because a write replaces the file whole.
///
/// Until ADR 0076 this was one list with the keys a change writes, and a test
/// asserted they were equal; it now asserts the live half of it, which is
/// everything here but `displays`.
///
/// **It held one of three until 2026-09-29.** `displays` was written by
/// `Changes` and missing here, so `keep` wrote the file, read it back, refused
/// its own output with `UnknownKey` and left the file as it was: a person who
/// singled out a display could not save it. The test that was supposed to hold
/// the two together measured a fixture that set only the edge, so it never saw
/// the key. **Anything added to `Changes` belongs here in the same change**, and
/// the test below still says so for every key that is written.
const KEYS: &[&str] = &["hiding", "edge", "displays"];

/// The keys on [`KEYS`] that no change writes, named so the tests below can tell
/// the two apart.
///
/// Nothing outside a test reads this: the tolerance itself is `KEYS` having them,
/// and the reader needs no list of which entries are dead. It is here so that
/// *the list is complete* stays a testable statement now that it is no longer an
/// equality.
#[cfg(test)]
const ONCE_WRITTEN: &[&str] = &["displays"];

impl Kept for Changes {
    const FILE: &'static str = THE_FILE;
    const FORMAT: i64 = FORMAT;
    const KEYS: &'static [&'static str] = KEYS;
    type NotRead = FileNotRead;
    type NotWritten = FileNotWritten;

    fn untouched() -> Self {
        Self::untouched()
    }

    fn not_read(at: &Path, why: Unread) -> Self::NotRead {
        FileNotRead::new(at, why)
    }

    fn not_written(at: &Path, why: Unwritten) -> Self::NotWritten {
        FileNotWritten::new(at, why)
    }
}

/// What the person changed about their dock, read from the file at `at`.
///
/// # Errors
///
/// [`FileNotRead`] for a file that is there and did not read, naming the file
/// and — when that was what was wrong — the key. A file that is not there is not
/// an error: it is [`Changes::untouched`].
pub fn read(at: &Path) -> Result<Changes, FileNotRead> {
    alo_kept::read(at)
}

/// These changes, kept as the whole of the file at `at`.
///
/// # Errors
///
/// [`FileNotWritten`] when the file was not replaced — by the disk, or because
/// the text would not have read back as these changes, or because the file is
/// there and does not read ([`FileNotWritten::did_not_read`]). The file is as it
/// was.
///
/// **A file that does not read is not written over.** It is asked as it is at
/// this moment, not as it was at sign-in, and a person's hand edit with one
/// mistake in it is kept for them to mend; [`put_back_as_shipped`] is the one
/// way to replace it.
pub fn keep(at: &Path, changes: &Changes) -> Result<(), FileNotWritten> {
    alo_kept::keep(at, changes)
}

/// Put this section back as alo OS ships it: the file at `at` replaced by the
/// format line alone, **whatever is there now — including a file that did not
/// read.**
///
/// The one door that writes over such a file, for the person's deliberate act
/// in Settings. It takes no changes, so nothing but the release's dock can
/// reach a broken file through it; afterwards the file reads as
/// [`Changes::untouched`].
///
/// # Errors
///
/// [`FileNotWritten`] when the disk would not take it. The file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<Changes>(at)
}

/// The dock a session draws when a person signs in, from the file at `at`.
///
/// The release's dock with the person's changes over it — or, when the file
/// did not read, the release's dock alone and the refusal beside it, for
/// Settings to say in that section. Never the half of a file that read.
#[must_use]
pub fn at_sign_in(at: &Path) -> (Dock, Option<FileNotRead>) {
    match read(at) {
        Ok(changes) => (Dock::shipped().with(changes), None),
        Err(refused) => (Dock::shipped(), Some(refused)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::hiding::Hiding;

    /// What a change writes, as the keys of the file it becomes.
    fn keys_written(changes: &Changes) -> Vec<String> {
        let text = alo_kept::text_of(changes).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let mut keys: Vec<String> = table
            .keys()
            .filter(|key| *key != alo_kept::THE_FORMAT_KEY)
            .cloned()
            .collect();
        keys.sort();
        keys
    }

    /// **Every key a change writes is on the list** — measured against a change
    /// that sets *every* setting, and against the list with the dead keys taken
    /// off it.
    ///
    /// The earlier version of this set the edge alone and compared what that
    /// wrote against the whole list, which is the same statement only while the
    /// list has one entry. It is how `displays` came to be written by `Changes`
    /// and missing from `KEYS`, so `keep` refused its own output and a person
    /// who singled out a display could not save it. **Anything added to
    /// `Changes` must be set here**, or this stops being able to see it.
    #[test]
    fn every_key_a_change_writes_is_on_the_list() {
        let mut changes = Changes::untouched();
        changes.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        // **Every setting, not one.** This test's own note says anything added
        // to `Changes` must be set here or it stops being able to see the key —
        // and that is not hypothetical: it is how `displays` came to be written
        // and missing from `KEYS`. The edge arrived on 2026-10-10 and this is
        // the line that keeps it in view.
        changes.set_edge(crate::Edge::Left);

        let mut live: Vec<String> = KEYS
            .iter()
            .filter(|key| !ONCE_WRITTEN.contains(*key))
            .map(|key| (*key).to_owned())
            .collect();
        live.sort();
        assert_eq!(keys_written(&changes), live);
    }

    /// **And nothing a change writes is one of the dead ones.** The two lists
    /// have to stay disjoint: a key on both would be written by `Changes` and
    /// declared as something nothing writes, and the test above would then be
    /// measuring one of them against itself minus itself.
    #[test]
    fn nothing_still_written_is_also_declared_dead() {
        let mut changes = Changes::untouched();
        changes.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        // **Every setting, not one.** This test's own note says anything added
        // to `Changes` must be set here or it stops being able to see the key —
        // and that is not hypothetical: it is how `displays` came to be written
        // and missing from `KEYS`. The edge arrived on 2026-10-10 and this is
        // the line that keeps it in view.
        changes.set_edge(crate::Edge::Left);
        for key in keys_written(&changes) {
            assert!(
                !ONCE_WRITTEN.contains(&key.as_str()),
                "{key} is written and also declared dead"
            );
        }
        for dead in ONCE_WRITTEN {
            assert!(
                KEYS.contains(dead),
                "{dead} is not recognised, so a file \
                 written by an earlier release would be refused rather than read"
            );
        }
    }

    /// **Only what was changed is written**: a dock asked to give way is
    /// `hiding` and nothing else.
    ///
    /// What the first test used to check, kept apart from it, because *the list
    /// is complete* and *nothing unasked-for is written* are two promises and a
    /// single assertion could only ever hold one of them.
    #[test]
    fn only_what_was_changed_is_written() {
        for hiding in [Hiding::Never, Hiding::WhenAWindowNeedsTheRoom] {
            let mut changes = Changes::untouched();
            changes.set_hiding(hiding);
            assert_eq!(keys_written(&changes), ["hiding"], "{hiding:?}");
        }
    }

    /// **An untouched dock is a format line and nothing else.**
    #[test]
    fn nothing_changed_is_a_format_line() {
        assert_eq!(
            alo_kept::text_of(&Changes::untouched()).unwrap(),
            "format = 1\n"
        );
    }
}
