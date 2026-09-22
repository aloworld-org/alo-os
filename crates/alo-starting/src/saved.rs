//! GRUB's environment block, read and written — the one place the last choice
//! is kept.
//!
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)'s
//! third term: *the last choice is the loader's own saved default, and nobody
//! keeps a copy.* So this reads and writes the loader's own file rather than a
//! setting of ours beside it, and [`crate::TheStartingChoice`] is the only
//! thing in alo OS that names the value in it.
//!
//! # The format, and why it is parsed rather than rewritten
//!
//! A block is a fixed number of bytes: the signature line, then `name=value`
//! lines, then `#` to the end. The size never changes, because the loader
//! writes the file in place — it has no filesystem driver that could grow one —
//! and a block this crate rewrote to a different size would be a block the
//! loader could no longer save into. So [`EnvironmentBlock::read`] keeps the
//! length it was given and [`EnvironmentBlock::written`] refuses rather than
//! grow past it.
//!
//! **Everything this does not recognise is kept.** A block holds whatever else
//! the base put in it, and a reader that dropped what it did not understand
//! would quietly take a setting away from a component that is not ours.

use std::fmt::Write as _;

/// The line every environment block begins with, newline included.
pub const SIGNATURE: &str = "# GRUB Environment Block\n";

/// How many bytes a block the loader's own tool makes is.
///
/// Only the size of a block this crate makes from nothing: one that was read is
/// written back at the length it was read at.
pub const LENGTH: usize = 1024;

/// Why something is not an environment block, or could not be written as one.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAnEnvironmentBlock {
    /// It does not begin the way every environment block does.
    #[error("this is not an environment block: it does not begin {SIGNATURE:?}")]
    NotOne,

    /// A line that is neither padding nor a setting.
    #[error("an environment block holds a line that is not a setting: {line:?}")]
    NotALine {
        /// The line, as it was read.
        line: String,
    },

    /// A name with something in it that no name may hold.
    #[error(
        "{name:?} is not a name a setting can be kept under: a name is letters, digits and \
         underscores, and is not empty"
    )]
    NotAName {
        /// The name that was offered.
        name: String,
    },

    /// A value with something in it that would end the line.
    #[error("that value cannot be kept in an environment block: it holds a line ending or a zero")]
    NotAValue,

    /// It would not fit in the block's own size.
    #[error(
        "these settings are {needed} bytes and the environment block is {room}, and a block is \
         never made longer than it was"
    )]
    NoRoom {
        /// How many bytes writing them would take.
        needed: usize,
        /// How many the block has.
        room: usize,
    },
}

/// The loader's environment block: what it keeps, and how long the file is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentBlock {
    /// Every setting, in the order they were read or first set.
    kept: Vec<(String, String)>,
    /// How many bytes the file is, which never changes.
    length: usize,
}

impl Default for EnvironmentBlock {
    fn default() -> Self {
        Self::empty()
    }
}

impl EnvironmentBlock {
    /// A block with nothing in it, the size the loader's own tool makes.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            kept: Vec::new(),
            length: LENGTH,
        }
    }

    /// The block in these bytes.
    ///
    /// # Errors
    /// [`NotAnEnvironmentBlock::NotOne`] for anything that does not begin with
    /// the signature, [`NotAnEnvironmentBlock::NotALine`] for a line that is
    /// neither padding nor a setting, and [`NotAnEnvironmentBlock::NotAName`]
    /// for a setting kept under a name no name may be.
    pub fn read(bytes: &[u8]) -> Result<Self, NotAnEnvironmentBlock> {
        let text = std::str::from_utf8(bytes).map_err(|_| NotAnEnvironmentBlock::NotOne)?;
        let rest = text
            .strip_prefix(SIGNATURE)
            .ok_or(NotAnEnvironmentBlock::NotOne)?;
        let mut kept: Vec<(String, String)> = Vec::new();
        for line in rest.split('\n') {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (name, value) =
                line.split_once('=')
                    .ok_or_else(|| NotAnEnvironmentBlock::NotALine {
                        line: line.to_owned(),
                    })?;
            held_to_a_name(name)?;
            match kept.iter_mut().find(|(known, _)| known == name) {
                Some(already) => already.1 = value.to_owned(),
                None => kept.push((name.to_owned(), value.to_owned())),
            }
        }
        Ok(Self {
            kept,
            length: bytes.len(),
        })
    }

    /// What is kept under this name, if anything.
    #[must_use]
    pub fn kept_as(&self, name: &str) -> Option<&str> {
        self.kept
            .iter()
            .find(|(known, _)| known == name)
            .map(|(_, value)| value.as_str())
    }

    /// Keep `value` under `name`, replacing whatever was there.
    ///
    /// # Errors
    /// [`NotAnEnvironmentBlock::NotAName`] for a name no name may be, and
    /// [`NotAnEnvironmentBlock::NotAValue`] for a value holding a line ending
    /// or a zero byte — either would end the line early and turn the rest of
    /// the value into a setting of its own.
    pub fn keep(&mut self, name: &str, value: &str) -> Result<(), NotAnEnvironmentBlock> {
        held_to_a_name(name)?;
        if value.contains(['\n', '\r', '\0']) {
            return Err(NotAnEnvironmentBlock::NotAValue);
        }
        match self.kept.iter_mut().find(|(known, _)| known == name) {
            Some(already) => already.1 = value.to_owned(),
            None => self.kept.push((name.to_owned(), value.to_owned())),
        }
        Ok(())
    }

    /// How many settings the block holds.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.kept.len()
    }

    /// How many bytes the file is.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.length
    }

    /// The block as the file holds it: the signature, the settings, and `#` to
    /// the end.
    ///
    /// # Errors
    /// [`NotAnEnvironmentBlock::NoRoom`] when the settings would not fit in the
    /// length the block was read at. Nothing is truncated and nothing is
    /// dropped: a block that will not fit is a refusal, because the setting
    /// that would silently go is somebody else's.
    pub fn written(&self) -> Result<Vec<u8>, NotAnEnvironmentBlock> {
        let mut out = String::from(SIGNATURE);
        for (name, value) in &self.kept {
            // Writing into a `String` cannot fail. The result is taken rather
            // than discarded so that nothing here is an ignored error.
            let written = writeln!(out, "{name}={value}");
            debug_assert!(written.is_ok());
        }
        if out.len() > self.length {
            return Err(NotAnEnvironmentBlock::NoRoom {
                needed: out.len(),
                room: self.length,
            });
        }
        while out.len() < self.length {
            out.push('#');
        }
        Ok(out.into_bytes())
    }
}

/// A name a setting may be kept under: letters, digits and underscores, and not
/// empty.
///
/// # Errors
/// [`NotAnEnvironmentBlock::NotAName`] for anything else.
fn held_to_a_name(name: &str) -> Result<(), NotAnEnvironmentBlock> {
    let allowed = !name.is_empty()
        && name
            .chars()
            .all(|each| each.is_ascii_alphanumeric() || each == '_');
    if allowed {
        return Ok(());
    }
    Err(NotAnEnvironmentBlock::NotAName {
        name: name.to_owned(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A block the loader's own tool would have written, holding `settings`.
    fn a_block(settings: &[(&str, &str)]) -> Vec<u8> {
        let mut block = EnvironmentBlock::empty();
        for (name, value) in settings {
            block.keep(name, value).unwrap();
        }
        block.written().unwrap()
    }

    /// Padding, as the loader writes it.
    fn padding(how_much: usize) -> String {
        "#".repeat(how_much)
    }

    /// **A block reads back as what was written into it**, at the same length,
    /// padded to it.
    #[test]
    fn a_block_reads_back_as_what_was_written() {
        let bytes = a_block(&[("saved_entry", "alo-windows"), ("boot_success", "1")]);
        assert_eq!(bytes.len(), LENGTH);
        let read = EnvironmentBlock::read(&bytes).unwrap();
        assert_eq!(read.kept_as("saved_entry"), Some("alo-windows"));
        assert_eq!(read.kept_as("boot_success"), Some("1"));
        assert_eq!(read.how_many(), 2);
        assert_eq!(read.length(), LENGTH);
        assert_eq!(read.written().unwrap(), bytes);
    }

    /// **What this crate does not recognise is kept.** A block holds what the
    /// base put in it, and a setting dropped on the way through would be taken
    /// away from whoever owns it.
    #[test]
    fn a_setting_nobody_here_knows_about_survives_being_written_back() {
        let bytes = a_block(&[("something_of_the_bases", "kept"), ("saved_entry", "")]);
        let mut read = EnvironmentBlock::read(&bytes).unwrap();
        read.keep("saved_entry", "alo-windows").unwrap();
        let again = EnvironmentBlock::read(&read.written().unwrap()).unwrap();
        assert_eq!(again.kept_as("something_of_the_bases"), Some("kept"));
        assert_eq!(again.kept_as("saved_entry"), Some("alo-windows"));
    }

    /// **Anything that is not an environment block is refused rather than
    /// guessed at** — a file of the right size with the wrong first line
    /// included, which is what a block from something else would look like.
    #[test]
    fn what_is_not_a_block_is_refused() {
        for bytes in [
            Vec::new(),
            b"saved_entry=alo-windows\n".to_vec(),
            format!("# GRUB Environment Block{}", padding(999)).into_bytes(),
            vec![0xff; LENGTH],
        ] {
            assert_eq!(
                EnvironmentBlock::read(&bytes),
                Err(NotAnEnvironmentBlock::NotOne),
                "{bytes:?} was read as an environment block"
            );
        }
    }

    /// **A line that is not a setting is refused**, rather than skipped — a
    /// block read wrongly is how the last choice comes back as something
    /// nobody chose.
    #[test]
    fn a_line_that_is_not_a_setting_is_refused() {
        let bytes = format!("{SIGNATURE}saved_entry\n{}", padding(900)).into_bytes();
        assert_eq!(
            EnvironmentBlock::read(&bytes),
            Err(NotAnEnvironmentBlock::NotALine {
                line: "saved_entry".to_owned(),
            })
        );
    }

    /// **A name no name may be is refused**, reading and writing alike.
    #[test]
    fn a_name_that_is_not_a_name_is_refused() {
        for name in [
            "",
            " saved_entry",
            "saved entry",
            "saved-entry",
            "saved.entry",
        ] {
            assert_eq!(
                EnvironmentBlock::empty().keep(name, "x"),
                Err(NotAnEnvironmentBlock::NotAName {
                    name: name.to_owned(),
                }),
                "{name:?} was taken as a name"
            );
            let bytes = format!("{SIGNATURE}{name}=x\n{}", padding(900)).into_bytes();
            assert!(
                matches!(
                    EnvironmentBlock::read(&bytes),
                    Err(NotAnEnvironmentBlock::NotAName { .. }
                        | NotAnEnvironmentBlock::NotALine { .. })
                ),
                "{name:?} was read as a name"
            );
        }
    }

    /// **A value that would end its own line is refused rather than escaped.**
    /// There is no escaping in this format, so a value holding a newline would
    /// become a setting of its own the next time the block was read — which is
    /// how a value a person never typed gets in.
    #[test]
    fn a_value_that_would_end_its_own_line_is_refused() {
        for value in ["a\nb=c", "a\r", "a\0b"] {
            assert_eq!(
                EnvironmentBlock::empty().keep("saved_entry", value),
                Err(NotAnEnvironmentBlock::NotAValue),
                "{value:?} was taken as a value"
            );
        }
    }

    /// **A block is never made longer than it was.** The loader writes the file
    /// in place, so growing it would be growing a file the loader can no longer
    /// save into — which is the last choice silently stopping being kept.
    #[test]
    fn more_than_fits_is_refused_and_nothing_is_dropped() {
        let bytes = a_block(&[("saved_entry", "alo-windows")]);
        let mut block = EnvironmentBlock::read(&bytes).unwrap();
        block.keep("filler", &"x".repeat(LENGTH)).unwrap();
        let Err(NotAnEnvironmentBlock::NoRoom { needed, room }) = block.written() else {
            panic!("a block longer than its own file was written");
        };
        assert!(needed > room);
        assert_eq!(room, LENGTH);
        assert_eq!(block.kept_as("saved_entry"), Some("alo-windows"));
    }

    /// **A block keeps the length it was read at**, rather than the length this
    /// crate would have made.
    #[test]
    fn a_block_keeps_the_length_it_was_read_at() {
        let bytes = format!("{SIGNATURE}saved_entry=\n{}", padding(100)).into_bytes();
        let block = EnvironmentBlock::read(&bytes).unwrap();
        assert_eq!(block.length(), bytes.len());
        assert_eq!(block.written().unwrap().len(), bytes.len());
    }
}
