//! The way out: a value as the text of its file, and the file replaced by it.
//!
//! **Nothing is written that this alo OS does not read back as the same
//! value.** [`text_of`] asks that of the text before a byte reaches the disk,
//! and [`keep`] asks it again of the bytes the disk hands back, before the
//! rename that makes them count. The failure it exists for is the quiet one: a
//! shape with a value its file cannot hold, written as something else and
//! answered with `Ok(())`, so that Settings shows one thing and the next
//! sign-in draws another.
//!
//! **And only the difference.** The untouched value is written as the format
//! line alone. A shape that would write more for it is refused, because a file
//! holding what nobody changed is a file that stops a later release's new
//! default from ever reaching that person.

use std::path::Path;

use crate::disk;
use crate::kept::Kept;
use crate::reading::from_text;
use crate::replacing::Over;
use crate::unwritten::Unwritten;

/// The key every kept file begins with.
pub const THE_FORMAT_KEY: &str = "format";

/// This value as the text of its file.
///
/// Nothing touches a disk here.
///
/// # Errors
///
/// [`Unwritten::NotExpressible`] when the value is not a table of keys or has a
/// `format` key of its own, [`Unwritten::NotOnlyTheDifference`] when the
/// untouched value would write anything but its format, and
/// [`Unwritten::ReadBackRefused`] or [`Unwritten::ReadBackAsSomethingElse`]
/// when the text does not read back as this value.
pub fn text_of<K: Kept>(value: &K) -> Result<String, Unwritten> {
    let table = match toml::Value::try_from(value) {
        Ok(toml::Value::Table(table)) => table,
        Ok(_) => {
            return Err(Unwritten::NotExpressible {
                why: "the shape is not a table of keys".to_owned(),
            });
        }
        Err(why) => {
            return Err(Unwritten::NotExpressible {
                why: why.to_string(),
            });
        }
    };
    if table.contains_key(THE_FORMAT_KEY) {
        return Err(Unwritten::NotExpressible {
            why: "the shape has a format key of its own".to_owned(),
        });
    }
    if !table.is_empty() && *value == K::untouched() {
        return Err(Unwritten::NotOnlyTheDifference);
    }
    let mut text = format!("{THE_FORMAT_KEY} = {}\n", K::FORMAT);
    if !table.is_empty() {
        let keys = toml::to_string(&table).map_err(|why| Unwritten::NotExpressible {
            why: why.to_string(),
        })?;
        text.push('\n');
        text.push_str(&keys);
    }
    reads_back_as(value, &text)?;
    Ok(text)
}

/// This value, kept as the whole of the file at `at`.
///
/// The folder the file sits in is made if it is not there, because the first
/// change a person makes is exactly the moment it is not.
///
/// # Errors
///
/// [`Kept::not_written`], in the owning crate's words, for every refusal
/// [`text_of`] makes, for a relative `at`, for bytes the disk hands back that
/// are not the same value, for every step the disk refuses, and
/// [`Unwritten::OverAFileThatDidNotRead`] when the file at `at` is there and
/// does not read at the moment of the write. The file at `at` is as it was in
/// all of them.
pub fn keep<K: Kept>(at: &Path, value: &K) -> Result<(), K::NotWritten> {
    written(at, value, Over::WhatReads)
}

/// The person who has changed nothing, kept as the whole of the file at `at` —
/// **whatever is there now, including a file that did not read.**
///
/// The one door that writes over such a file, and the deliberate act ADR 0038
/// names: *put this section back as shipped*. It takes no value, so nothing but
/// [`Kept::untouched`] can reach a broken file through it. What it writes is the
/// format line alone, which reads as the release's settings.
///
/// # Errors
///
/// [`Kept::not_written`], in the owning crate's words, for a relative `at` and
/// every step the disk refuses. The file at `at` is as it was in all of them.
pub fn put_back_as_shipped<K: Kept>(at: &Path) -> Result<(), K::NotWritten> {
    written(at, &K::untouched(), Over::Anything)
}

/// This value written whole to `at`, over whatever `over` allows.
fn written<K: Kept>(at: &Path, value: &K, over: Over) -> Result<(), K::NotWritten> {
    if !at.has_root() {
        return Err(K::not_written(at, Unwritten::NotWhereItBelongs));
    }
    let text = text_of(value).map_err(|why| K::not_written(at, why))?;
    disk::kept(at, &text, |on_disk| {
        match std::str::from_utf8(on_disk) {
            Ok(back) if back == text => reads_back_as(value, back)?,
            Ok(_) | Err(_) => return Err(Unwritten::ReadBackAsSomethingElse),
        }
        // Last, immediately before the rename: the file as it is at the moment
        // of the write, not as it was at sign-in or before the text was staged.
        over.allows::<K>(at)
    })
    .map_err(|why| K::not_written(at, why))
}

/// Whether this text is this value, read.
fn reads_back_as<K: Kept>(value: &K, text: &str) -> Result<(), Unwritten> {
    match from_text::<K>(text) {
        Ok(back) if back == *value => Ok(()),
        Ok(_) => Err(Unwritten::ReadBackAsSomethingElse),
        Err(why) => Err(Unwritten::ReadBackRefused(why)),
    }
}

/// This text, written whole to `at` or not at all, for a shape that owns its
/// own format.
///
/// [`keep`] is the road for a [`Kept`]: it serialises the value, checks the
/// text reads back as the same value, and writes. **This is the same disk
/// discipline without the serialisation** — a sibling written, synced, read
/// back, asked about, and renamed over the real file, with every way it can
/// fail failing before the rename.
///
/// # Why a crate would want this rather than [`Kept`]
///
/// Because a format can be one [`Kept`] cannot describe. [`Kept`] requires
/// `DeserializeOwned`, and a crate whose rule is *nothing is read back
/// unvalidated* cannot provide it without giving up the rule: a derived
/// `Deserialize` rebuilds a value without asking the constructors that make it
/// legal. `alo-arranging` is the case this was added for — every position and
/// size it reads is rebuilt through its own checked constructors, and it has no
/// `Deserialize` for a camera on purpose.
///
/// Such a crate still wants **this** part, and writing a second copy of it
/// beside this one would be two implementations of the only step that must not
/// be got wrong.
///
/// # What `holds` is for
///
/// It is handed the bytes **the disk gave back**, after the sync and before the
/// rename, and its refusal is the call's refusal. That is where a caller checks
/// its own format read back as the value it meant — the same check [`keep`]
/// makes for a [`Kept`], written by whoever owns the format. A caller that
/// answers `Ok` without looking has the atomicity and not the proof.
///
/// # Errors
///
/// [`Unwritten::Disk`] at the first step the disk refused, or whatever `holds`
/// refused the bytes with. **The file at `at` is untouched in every one of
/// them**, including a refusal from `holds`.
pub fn kept_text(
    at: &Path,
    text: &str,
    holds: impl FnOnce(&[u8]) -> Result<(), Unwritten>,
) -> Result<(), Unwritten> {
    crate::disk::kept(at, text, holds)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{Edge, Example, NotATable, TakesTheFormat, WritesWhatNobodyChanged};

    /// **The discipline is reachable by a crate that keeps its own format**, which
    /// is the only thing this road adds.
    ///
    /// Everything about the write itself is already held by `disk::tests` —
    /// the text being there afterwards, bytes that do not hold leaving the file
    /// as it was, a refused write leaving no sibling, the folder being made, and
    /// the file being its owner's alone. Repeating those here would re-assert
    /// seven existing tests through a one-line delegation.
    ///
    /// **What they cannot hold is that the delegation exists.** `disk::kept` is
    /// `pub(crate)`, so before this road a crate with a validated bespoke format
    /// could not reach it at all and would have written a second copy of the
    /// only step that must not be got wrong. This asserts the round trip
    /// through the public name, and both halves of `holds` — the answer it is
    /// given, and the refusal it can return.
    #[test]
    fn a_crate_with_its_own_format_can_reach_the_discipline() {
        let folder = crate::testing::a_folder_of_our_own("kept-text-reachable");
        let at = folder.join("canvas.toml");

        let mut what_the_disk_said = Vec::new();
        kept_text(&at, "format = 1\nplaces = []\n", |back| {
            what_the_disk_said = back.to_vec();
            Ok(())
        })
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "format = 1\nplaces = []\n"
        );
        assert_eq!(
            what_the_disk_said, b"format = 1\nplaces = []\n",
            "holds was asked about something other than the bytes written"
        );
        assert_eq!(
            kept_text(&at, "format = 1\nplaces = [1]\n", |_back| Err(
                Unwritten::ReadBackAsSomethingElse
            )),
            Err(Unwritten::ReadBackAsSomethingElse),
            "a caller's own refusal did not come back as the call's refusal"
        );
    }

    /// **Only what changed is in the text**, after the format line.
    #[test]
    fn only_what_changed_is_written() {
        let changed = Example {
            edge: Some(Edge::Left),
            ..Example::untouched()
        };
        assert_eq!(
            text_of(&changed).unwrap(),
            "format = 1\n\nedge = \"left\"\n"
        );
    }

    /// **Nothing changed is a format line and nothing else.**
    #[test]
    fn nothing_changed_is_a_format_line() {
        assert_eq!(text_of(&Example::untouched()).unwrap(), "format = 1\n");
    }

    /// **A shape that writes what nobody changed is refused.**
    #[test]
    fn a_shape_that_writes_what_nobody_changed_is_refused() {
        assert_eq!(
            text_of(&WritesWhatNobodyChanged::untouched()).unwrap_err(),
            Unwritten::NotOnlyTheDifference
        );
    }

    /// **A shape that is not a table of keys is refused**, rather than written
    /// under a key this crate invents.
    #[test]
    fn a_shape_that_is_not_a_table_is_refused() {
        assert!(matches!(
            text_of(&NotATable(vec!["one".to_owned()])).unwrap_err(),
            Unwritten::NotExpressible { .. }
        ));
    }

    /// **A shape with a `format` of its own is refused**, because it would be
    /// overwritten by the file's.
    #[test]
    fn a_shape_with_a_format_of_its_own_is_refused() {
        assert!(matches!(
            text_of(&TakesTheFormat { format: 7 }).unwrap_err(),
            Unwritten::NotExpressible { .. }
        ));
    }

    /// **A value its own reader would refuse is not written.** The fixture
    /// writes a key its list forgot, which is a release adding a key to the
    /// shape and not to [`Kept::KEYS`].
    #[test]
    fn a_value_that_does_not_read_back_is_refused() {
        let forgot = Example {
            forgotten: Some(true),
            ..Example::untouched()
        };
        assert_eq!(
            text_of(&forgot).unwrap_err(),
            Unwritten::ReadBackRefused(crate::Unread::UnknownKey {
                key: "forgotten".to_owned()
            })
        );
    }
}
