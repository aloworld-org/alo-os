//! **How `alo-enter-the-box` says the box was joined and the command was
//! refused**, so that this crate can tell that apart from a command that ran
//! and failed.
//!
//! Until 2026-11-06 it could not. The helper ends with its own failing code
//! when the command will not start, and a command that ran and returned 1 ends
//! with the same code — so a refused start was written down as a run, and the
//! person was told the command had *run*. On an installed alo OS that was every
//! boxed run (`alo_capability::the_sealed_box_here` has the measurement).
//!
//! **A word of its own, per run, that the command never sees.** The runner
//! makes one and hands it to the helper in [`THE_WORD_IS_CALLED`]; the helper
//! takes it out of what the command inherits before starting it, and writes it
//! only when the start was refused after the box was joined. A command cannot
//! write a word it was never given, so a line carrying it is the helper's.
//! [`Listening`] carries the command's error stream to the person as before,
//! and keeps that one line out of it.

use std::hash::{BuildHasher as _, Hasher as _};
use std::io::Write;

/// The variable the word reaches the helper in, and is removed from before
/// the command starts.
pub const THE_WORD_IS_CALLED: &str = "ALO_ENTER_THE_BOX_SAYS";

/// What the helper adds after the word when the refusal was the kernel's
/// *permission denied*.
pub const DENIED: &str = "denied";

/// What it adds for any other refusal.
pub const OTHERWISE: &str = "otherwise";

/// Why a start was refused after the box was joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartRefused {
    /// The kernel said *permission denied*: the boundary refused it.
    PermissionDenied,
    /// Any other reason.
    Otherwise,
}

/// A word for one run, made from this process's random hashing keys.
pub(crate) fn a_word_of_its_own() -> String {
    let mut hashing = std::collections::hash_map::RandomState::new().build_hasher();
    hashing.write_u64(u64::from(std::process::id()));
    let one = hashing.finish();
    let mut again = std::collections::hash_map::RandomState::new().build_hasher();
    again.write_u64(one);
    format!("alo-enter-the-box-{one:016x}{:016x}", again.finish())
}

/// The line the helper writes, for `word` and the kind of the refusal.
#[must_use]
pub fn the_refusal_line(word: &str, kind: std::io::ErrorKind) -> String {
    let said = if kind == std::io::ErrorKind::PermissionDenied {
        DENIED
    } else {
        OTHERWISE
    };
    format!("{word} {said}\n")
}

/// Carries a command's error stream to `to`, and listens for the helper's
/// line among it.
pub(crate) struct Listening<'a> {
    /// Where everything else goes.
    to: &'a mut (dyn Write + Send),
    /// The word the helper was given, and a space after it.
    word: Vec<u8>,
    /// The start of the line being read, held back while it could still be
    /// the helper's.
    held: Vec<u8>,
    /// Whether what arrives next starts a line.
    at_a_line_start: bool,
    /// Whether the line now arriving is being kept back as the helper's.
    in_its_line: bool,
    /// What the helper said, once heard.
    heard: Option<StartRefused>,
}

impl<'a> Listening<'a> {
    /// Listening for `word` on the way to `to`.
    pub(crate) fn for_the_word(word: &str, to: &'a mut (dyn Write + Send)) -> Self {
        Self {
            to,
            word: format!("{word} ").into_bytes(),
            held: Vec::new(),
            at_a_line_start: true,
            in_its_line: false,
            heard: None,
        }
    }

    /// What the helper said, if it said anything.
    pub(crate) const fn heard(&self) -> Option<StartRefused> {
        self.heard
    }

    /// One byte, as the stream delivers it.
    fn one(&mut self, byte: u8) -> std::io::Result<()> {
        if self.in_its_line {
            if byte == b'\n' {
                let said = self.held.get(self.word.len()..).unwrap_or_default();
                self.heard = Some(if said == DENIED.as_bytes() {
                    StartRefused::PermissionDenied
                } else {
                    StartRefused::Otherwise
                });
                self.held.clear();
                self.in_its_line = false;
                self.at_a_line_start = true;
            } else {
                self.held.push(byte);
            }
            return Ok(());
        }
        if self.at_a_line_start || !self.held.is_empty() {
            self.held.push(byte);
            if self.held.len() <= self.word.len() && self.word.starts_with(&self.held) {
                self.at_a_line_start = false;
                if self.held.len() == self.word.len() {
                    self.in_its_line = true;
                }
                return Ok(());
            }
            // Not the helper's line after all: everything held is the
            // command's, and goes on now.
            let held = std::mem::take(&mut self.held);
            self.to.write_all(&held)?;
            self.at_a_line_start = byte == b'\n';
            return Ok(());
        }
        self.to.write_all(&[byte])?;
        self.at_a_line_start = byte == b'\n';
        Ok(())
    }
}

impl Write for Listening<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        for byte in bytes {
            self.one(*byte)?;
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.to.flush()
    }
}

impl Drop for Listening<'_> {
    /// Whatever was held back and turned out not to be the helper's whole
    /// line is the command's, and is not lost when its stream ends mid-line.
    fn drop(&mut self) {
        if !self.held.is_empty() {
            let held = std::mem::take(&mut self.held);
            self.to.write_all(&held).ok();
            self.to.flush().ok();
        }
    }
}

#[cfg(test)]
#[path = "the_helpers_word_tests.rs"]
mod tests;
