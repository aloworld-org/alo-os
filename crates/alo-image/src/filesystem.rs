//! The one filesystem an alo OS machine is installed onto, and the guard that
//! keeps it one.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) chose
//! the base's own snapshot as what *undo what the agent did* rewinds from, and
//! a snapshot needs a filesystem that has them. **A filesystem is chosen at
//! install and cannot be converted afterwards**, so the argument
//! `bootc install to-disk --filesystem` is given is not one line among many: it
//! is the whole of whether a machine can ever undo anything, decided once,
//! before anybody's disk is written.
//!
//! # Why a constant here rather than in the installer
//!
//! The installer names the filesystem and this crate reads the installer, which
//! looks the wrong way round until the failure is named: the argument is
//! written in more than one place — `crates/alo-installing/src/writing.rs`,
//! which is what the environment runs on a person's real disk, and
//! `docs/booting.md`, which is what a person types to make a development disk.
//! Two places is one place too many for something that cannot be undone, so
//! [`THE_ONLY_FILESYSTEM`] is the value and every writer takes it from here.
//!
//! That alone stops the installer naming something else *on purpose*. It does
//! not stop a **second** `--filesystem` appearing later — one added beside the
//! first for a case somebody had, which would install some machines onto a
//! filesystem with no snapshots and leave nobody to notice. That is what
//! [`TheFilesystem`] is for: it reads every `--filesystem` in the text it is
//! given and says which were named, so a second one is a failing test rather
//! than a machine that cannot undo.
//!
//! # What it deliberately does not read
//!
//! Only the files that decide what **a machine alo OS installs** gets. A test
//! elsewhere in this repository that installs a scratch disk on `ext4` to
//! measure something about `ext4` is measuring `ext4` on purpose, and a guard
//! that swept the whole tree would call that a defect. The scope is named by
//! whoever calls [`TheFilesystem::read`], and `tests/` names it.
//!
//! # And what it does not decide
//!
//! Nothing here says what a snapshot is *for*, when one is taken or how long it
//! is kept: that is ADR 0045's, and `alo-keeping-up`'s. This says only that the
//! disk a person ends up with is one a snapshot can be taken on at all.

/// The only filesystem `bootc install to-disk` is ever given.
///
/// `btrfs`, because it is the one of the three the pinned `bootc` accepts —
/// `xfs`, `ext4`, `btrfs` — that has subvolumes and read-only snapshots, which
/// is what ADR 0045's undo rewinds from. Measured on the pinned release in
/// `docs/quirks.md`: the base makes no subvolume of its own with it, and a
/// read-only snapshot on the pinned kernel works.
pub const THE_ONLY_FILESYSTEM: &str = "btrfs";

/// The argument that names it.
const THE_ARGUMENT: &str = "--filesystem";

/// What [`THE_ONLY_FILESYSTEM`] is called where Rust names it rather than
/// spelling it.
///
/// A writer that passes the constant is passing this value — that is the whole
/// point of there being a constant — so a reader that called
/// `THE_ONLY_FILESYSTEM.to_owned()` a second filesystem would refuse the one
/// shape this module exists to encourage.
const BY_ITS_NAME: &str = "THE_ONLY_FILESYSTEM";

/// Every filesystem a text names as the one to install onto.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheFilesystem {
    /// Each value that followed `--filesystem`, in the order they were read.
    named: Vec<String>,
}

impl TheFilesystem {
    /// Every `--filesystem` in this text, read off it.
    ///
    /// Read leniently, which is `crate::disk`'s shape: a text naming none reads
    /// as one that names none, and the judgement is the caller's. The value is
    /// whatever follows the argument on the same line, with the quoting a Rust
    /// string literal or a shell would put around it taken off, so
    /// `--filesystem btrfs`, `--filesystem "btrfs"` and a `"btrfs".to_owned()`
    /// on the line after `"--filesystem".to_owned(),` are all read as `btrfs`.
    #[must_use]
    pub fn read(text: &str) -> Self {
        let mut named = Vec::new();
        let mut the_next_word_is_it = false;
        for line in text.lines() {
            if line.trim_start().starts_with("//") || line.trim_start().starts_with('#') {
                continue;
            }
            for word in line.split_whitespace().map(bare) {
                if std::mem::replace(&mut the_next_word_is_it, false) && !word.is_empty() {
                    if word.contains(BY_ITS_NAME) {
                        named.push(THE_ONLY_FILESYSTEM.to_owned());
                    } else {
                        named.push(word.to_owned());
                    }
                    continue;
                }
                if word == THE_ARGUMENT {
                    the_next_word_is_it = true;
                }
            }
            // A line ending in the argument names its value on the next line,
            // which is how both the document's continued command and the
            // installer's list of arguments are written.
        }
        Self { named }
    }

    /// Every filesystem this text named, in the order it named them.
    #[must_use]
    pub fn named(&self) -> &[String] {
        &self.named
    }

    /// Whether this text names a filesystem at all.
    #[must_use]
    pub fn names_one(&self) -> bool {
        !self.named.is_empty()
    }

    /// Every filesystem named that is not [`THE_ONLY_FILESYSTEM`].
    ///
    /// Empty where the text names the one filesystem however many times — the
    /// same value twice is one decision written twice, which is what
    /// [`THE_ONLY_FILESYSTEM`] exists to make harmless.
    #[must_use]
    pub fn anything_else(&self) -> Vec<&str> {
        self.named
            .iter()
            .map(String::as_str)
            .filter(|named| *named != THE_ONLY_FILESYSTEM)
            .collect()
    }
}

/// A word with the quoting a Rust literal, a shell or a comma would put around
/// it taken off.
fn bare(word: &str) -> &str {
    let word = word.trim_end_matches(',');
    let word = word
        .strip_suffix(".to_owned()")
        .or_else(|| word.strip_suffix(".to_string()"))
        .unwrap_or(word);
    word.trim_matches('"').trim_matches('\'')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The argument's value is read wherever the argument is written**, in
    /// each of the three shapes this repository writes it in: a shell command
    /// on one line, a shell command continued onto the next, and a Rust list of
    /// owned strings.
    #[test]
    fn every_way_this_repository_names_a_filesystem_reads_the_same() {
        for text in [
            "bootc install to-disk --wipe --filesystem btrfs /dev/disk/by-id/x",
            "      bootc install to-disk --via-loopback --wipe \\\n        --filesystem btrfs /output/alo-os.raw",
            "            \"--filesystem\".to_owned(),\n            THE_ONLY_FILESYSTEM.to_owned(),",
            "                \"--filesystem\",\n                \"btrfs\",",
        ] {
            let read = TheFilesystem::read(text);
            assert!(read.names_one(), "nothing was read out of `{text}`");
        }
    }

    /// **A writer that passes the constant has named the one filesystem**,
    /// which is the shape this module exists to encourage: the installer writes
    /// `THE_ONLY_FILESYSTEM.to_owned()`, and reading that as some other
    /// filesystem would refuse the right answer.
    #[test]
    fn the_constant_passed_by_its_name_is_the_one_filesystem() {
        let read = TheFilesystem::read(
            "            \"--filesystem\".to_owned(),\n            THE_ONLY_FILESYSTEM.to_owned(),",
        );

        assert_eq!(read.named(), [THE_ONLY_FILESYSTEM]);
        assert!(read.anything_else().is_empty());
    }

    /// **The one filesystem reads as nothing else named**, however many times a
    /// text names it — the installer's program and its own test name it twice
    /// in one file, and that is one decision written twice.
    #[test]
    fn the_one_filesystem_named_twice_is_still_one_decision() {
        let read = TheFilesystem::read(
            "bootc install to-disk --filesystem btrfs /output/a.raw\n\
             bootc install to-disk --filesystem btrfs /output/b.raw\n",
        );

        assert_eq!(read.named(), ["btrfs", "btrfs"]);
        assert!(read.anything_else().is_empty());
    }

    /// **A constant of somebody's own is not the one filesystem.** It is how
    /// the value came to be written twice in the first place: a private
    /// `THE_FILESYSTEM` inside one crate, whose value nothing outside it sees.
    #[test]
    fn a_constant_of_somebodys_own_is_not_the_one_filesystem() {
        let read = TheFilesystem::read(
            "            \"--filesystem\".to_owned(),\n            THE_FILESYSTEM.to_owned(),",
        );

        assert_eq!(read.anything_else(), ["THE_FILESYSTEM"]);
    }

    /// **A second filesystem is found and named.** It is the shape the mistake
    /// arrives in: not a new installer, one more `--filesystem` beside the
    /// first for a case somebody had — and a machine installed through it could
    /// never undo anything, with nobody to notice.
    #[test]
    fn a_second_filesystem_beside_the_first_is_found() {
        let read = TheFilesystem::read(
            "bootc install to-disk --filesystem btrfs /output/a.raw\n\
             bootc install to-disk --filesystem ext4 /output/b.raw\n",
        );

        assert_eq!(read.anything_else(), ["ext4"]);
    }

    /// **A text that names no filesystem names none**, rather than refusing:
    /// the wrong states are the ones a guard has to be able to see, and *the
    /// installer stopped naming one at all* is one of them.
    #[test]
    fn a_text_with_no_filesystem_in_it_names_none() {
        let read = TheFilesystem::read("bootc install to-disk --wipe /output/alo-os.raw");

        assert!(!read.names_one());
        assert!(read.anything_else().is_empty());
    }

    /// **A commented-out argument names nothing**, which is what a half-finished
    /// edit really leaves behind — in a recipe, in a document and in Rust.
    #[test]
    fn a_commented_argument_names_nothing() {
        for commented in [
            "# bootc install to-disk --filesystem ext4 /output/alo-os.raw",
            "// the installer used to pass --filesystem ext4 here",
        ] {
            assert!(
                !TheFilesystem::read(commented).names_one(),
                "`{commented}` was read as naming a filesystem"
            );
        }
    }
}
