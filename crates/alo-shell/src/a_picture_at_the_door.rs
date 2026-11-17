//! **A picture alo asked for at the door, and its answer on the way out**
//! (`picture-of`, `docs/contracts/application-door.md`).
//!
//! The door reads the one line and hands it on with its connection; the
//! compositor takes the picture and answers through [`AskedToPicture::answered`](crate::a_picture_at_the_door::AskedToPicture::answered).
//! The answer is the head line and, for a picture, exactly the PNG's bytes
//! after it.
//!
//! **Sent on across turns, never waited for.** A PNG is larger than a socket
//! holds, and the door never blocks the compositor's thread — the rule every
//! act here keeps. So what does not fit is kept in a [`SendingAPicture`](crate::a_picture_at_the_door::SendingAPicture) and
//! the door sends on from it each time it is visited, as it reads a long text
//! on across frames. A daemon that has gone, or a write that fails, ends it.

use std::io::Write as _;
use std::os::unix::net::UnixStream;

use alo_acting::{Pictured, ToPicture};

/// A picture asked for, with the connection that asked.
#[derive(Debug)]
pub struct AskedToPicture {
    /// What was asked.
    asked: ToPicture,
    /// The connection, which is answered once.
    talking: UnixStream,
}

impl AskedToPicture {
    /// The door read `asked` on `talking`.
    pub(crate) const fn of(asked: ToPicture, talking: UnixStream) -> Self {
        Self { asked, talking }
    }

    /// What was asked.
    #[must_use]
    pub const fn asked(&self) -> &ToPicture {
        &self.asked
    }

    /// Answer: the head line, and for [`Pictured::Picture`] the PNG's bytes
    /// after it — **only** for a picture, so a refusal never carries bytes.
    #[must_use]
    pub fn answered(self, answer: Pictured, png: Vec<u8>) -> SendingAPicture {
        // **A head that says another length is never sent**: the daemon would
        // read past the picture or stop inside it.
        let answer = match answer {
            Pictured::Picture { bytes, .. } if bytes != png.len() => Pictured::CouldNotDraw,
            other => other,
        };
        let mut bytes = answer.written().into_bytes();
        bytes.push(b'\n');
        if matches!(answer, Pictured::Picture { .. }) {
            bytes.extend_from_slice(&png);
        }
        SendingAPicture {
            talking: self.talking,
            bytes,
            sent: 0,
        }
    }
}

/// An answer part-way out.
#[derive(Debug)]
pub struct SendingAPicture {
    /// The connection.
    talking: UnixStream,
    /// The whole answer: the head line and what follows it.
    bytes: Vec<u8>,
    /// How much of it has gone.
    sent: usize,
}

impl SendingAPicture {
    /// Send as much more as the socket takes now: what is left, or [`None`]
    /// once it is all sent or cannot be.
    #[must_use]
    pub fn sent_on(mut self) -> Option<Self> {
        while let Some(left) = self.bytes.get(self.sent..).filter(|left| !left.is_empty()) {
            match self.talking.write(left) {
                Ok(0) => return None,
                Ok(written) => self.sent += written,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Some(self),
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => return None,
            }
        }
        let _ = self.talking.flush();
        None
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or a short read is the failure being reported"
)]
#[path = "a_picture_at_the_door_tests.rs"]
mod tests;
