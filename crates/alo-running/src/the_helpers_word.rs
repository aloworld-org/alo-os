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
//! `Listening` carries the command's error stream to the person as before,
//! and keeps that one line out of it.
//!
//! # The line carries the helper's own account, and nothing else reaches the
//! person
//!
//! **Until 2026-11-08 the helper wrote that line and then its English one
//! too** — *`/usr/bin/ls did not start inside /sys/fs/cgroup/…: Permission
//! denied (os error 13)`* — and only the first was kept back, so the second
//! reached the person's terminal above the plain sentence `alo-try` prints
//! (walked on image `f7cea72c`). Now the account rides on the word's line
//! ([`the_refusal_line`]), is heard with it ([`TheHelperSaid`]), and the
//! helper writes nothing else once it has written the word
//! ([`what_the_helper_writes`]). The caller decides where the account goes;
//! `alo-try` gives it to the journal.

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

/// What the helper said on its line: why the start was refused, and its own
/// account of it for whoever administers the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheHelperSaid {
    /// Why, as a kind the caller can decide on.
    pub refused: StartRefused,
    /// The helper's own account, in English, on one line. Empty where it gave
    /// none.
    pub account: String,
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

/// The line the helper writes, for `word`, the kind of the refusal and its own
/// `account` of it: `<word> denied <account>`.
///
/// **One line whatever the account holds**: a line break in it — a program's
/// name may have one — becomes a space, so the account cannot end the line
/// early and put the rest in front of the person.
#[must_use]
pub fn the_refusal_line(word: &str, kind: std::io::ErrorKind, account: &str) -> String {
    let said = if kind == std::io::ErrorKind::PermissionDenied {
        DENIED
    } else {
        OTHERWISE
    };
    let account: String = account
        .chars()
        .map(|one| if one == '\n' || one == '\r' { ' ' } else { one })
        .collect();
    format!("{word} {said} {account}\n")
}

/// Everything the helper writes to its error stream when a start is refused:
/// its `account`, and how.
///
/// - **Given a word, and the box was joined**: the word's line, carrying the
///   account, **and nothing else**. The runner keeps that line from the
///   person, so a refusal it can name reaches the person only as the
///   caller's own sentence.
/// - **Otherwise** — no word, or refused before the box was joined, which the
///   runner has no word for — the account as an ordinary line, as it always
///   was: a refusal nobody listens for is said where somebody can read it.
#[must_use]
pub fn what_the_helper_writes(
    word: Option<&str>,
    joined: Option<std::io::ErrorKind>,
    account: &str,
) -> String {
    match (word, joined) {
        (Some(word), Some(kind)) => the_refusal_line(word, kind, account),
        _ => format!("alo-enter-the-box: {account}\n"),
    }
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
    heard: Option<TheHelperSaid>,
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
    pub(crate) fn heard(&self) -> Option<TheHelperSaid> {
        self.heard.clone()
    }

    /// One byte, as the stream delivers it.
    fn one(&mut self, byte: u8) -> std::io::Result<()> {
        if self.in_its_line {
            if byte == b'\n' {
                // `denied <account>`: the kind up to the first space, and the
                // account after it. A line with no account is still heard.
                let said = self.held.get(self.word.len()..).unwrap_or_default();
                let (kind, account) = match said.iter().position(|one| *one == b' ') {
                    Some(at) => (
                        said.get(..at).unwrap_or_default(),
                        said.get(at + 1..).unwrap_or_default(),
                    ),
                    None => (said, &[][..]),
                };
                self.heard = Some(TheHelperSaid {
                    refused: if kind == DENIED.as_bytes() {
                        StartRefused::PermissionDenied
                    } else {
                        StartRefused::Otherwise
                    },
                    account: String::from_utf8_lossy(account).into_owned(),
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
