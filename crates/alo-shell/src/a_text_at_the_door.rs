//! **A text alo asked to type, read off the door whole** —
//! `docs/contracts/application-door.md`, *`type-into` and `save-in`*
//! (ADR 0153).
//!
//! `type-into <application> <bytes>` says how many bytes of text follow. The
//! text has no limit, so it can be longer than a frame's read: like every other
//! caller at the door it is read **across frames, never waited for** — but a
//! caller that is still sending keeps its patience, and only one that stops is
//! dropped. Exactly that many bytes are taken; more than that, or bytes that
//! are not UTF-8, is a caller with a bug, closed with nothing written.

use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::net::UnixStream;

use alo_acting::{ToType, Typed};

// How many frames a caller may send nothing while a text is part-way read:
// the door's own, about a second.
use crate::the_application_door::PATIENCE;

/// How much is read from a caller in one go.
const A_READ: usize = 64 * 1024;

/// A text part-way read.
#[derive(Debug)]
pub(crate) struct ReadingText {
    /// What was asked.
    asked: ToType,
    /// How many bytes the text is.
    bytes: usize,
    /// The connection, not blocking.
    talking: UnixStream,
    /// The text so far.
    heard: Vec<u8>,
    /// Frames left with nothing arriving before this caller is dropped.
    patience: u8,
}

/// How far a text has got.
pub(crate) enum SoFar {
    /// Still arriving. Ask again next frame.
    Arriving(ReadingText),
    /// All of it, ready to type.
    Whole(AskedToType),
    /// The caller went, stopped, or said more or other than it promised.
    Dropped,
}

impl ReadingText {
    /// Begin reading the text `asked` promises, with whatever already followed
    /// its line. A save has no text, and is whole at once.
    pub(crate) fn begun(asked: ToType, talking: UnixStream, already: Vec<u8>) -> SoFar {
        let bytes = match &asked {
            ToType::Text { bytes, .. } => *bytes,
            ToType::Save { .. } | ToType::Check { .. } => {
                return if already.is_empty() {
                    SoFar::Whole(AskedToType {
                        asked,
                        text: String::new(),
                        talking,
                    })
                } else {
                    SoFar::Dropped
                };
            }
        };
        Self {
            asked,
            bytes,
            talking,
            heard: already,
            patience: PATIENCE,
        }
        .looked_at()
    }

    /// Take whatever has arrived, and say how far the text has got.
    pub(crate) fn looked_at(mut self) -> SoFar {
        let mut arrived = vec![0_u8; A_READ];
        let mut any = false;
        while self.heard.len() < self.bytes {
            match self.talking.read(&mut arrived) {
                Ok(0) => return SoFar::Dropped,
                Ok(read) => {
                    let Some(said) = arrived.get(..read) else {
                        return SoFar::Dropped;
                    };
                    self.heard.extend_from_slice(said);
                    any = true;
                }
                Err(why) if why.kind() == ErrorKind::WouldBlock => break,
                Err(why) if why.kind() == ErrorKind::Interrupted => {}
                Err(_) => return SoFar::Dropped,
            }
        }
        if self.heard.len() > self.bytes {
            return SoFar::Dropped;
        }
        if self.heard.len() < self.bytes {
            if any {
                self.patience = PATIENCE;
            } else {
                self.patience = self.patience.saturating_sub(1);
                if self.patience == 0 {
                    return SoFar::Dropped;
                }
            }
            return SoFar::Arriving(self);
        }
        match String::from_utf8(self.heard) {
            Ok(text) => SoFar::Whole(AskedToType {
                asked: self.asked,
                text,
                talking: self.talking,
            }),
            Err(_) => SoFar::Dropped,
        }
    }
}

/// A typing asked at the door, whole, waiting for the compositor to carry it
/// out and answer.
#[derive(Debug)]
pub struct AskedToType {
    /// What was asked.
    asked: ToType,
    /// The text, whole — empty for a save.
    text: String,
    /// The connection, not blocking.
    talking: UnixStream,
}

impl AskedToType {
    /// What was asked.
    #[must_use]
    pub const fn asked(&self) -> &ToType {
        &self.asked
    }

    /// The text, whole — empty for a save.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Say what happened, and hang up.
    pub fn answered(mut self, typed: Typed) {
        let _ = self.talking.write_all(typed.written().as_bytes());
        let _ = self.talking.write_all(b"\n");
        let _ = self.talking.flush();
    }
}

#[cfg(test)]
#[path = "a_text_at_the_door_tests.rs"]
mod tests;
