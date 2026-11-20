//! **A click alo asked for at the door**, held until the compositor carries it
//! out in the frame, and answered in one word (`click-at`,
//! `docs/contracts/application-door.md`).

use std::io::Write as _;
use std::os::unix::net::UnixStream;

use alo_acting::{Clicked, ToClick};

/// One click alo asked for, with the connection its answer goes back on.
#[derive(Debug)]
pub struct AskedToClick {
    /// What was asked.
    asked: ToClick,
    /// Where the answer goes.
    talking: UnixStream,
}

impl AskedToClick {
    /// A click read off the door, and its connection.
    pub(crate) const fn of(asked: ToClick, talking: UnixStream) -> Self {
        Self { asked, talking }
    }

    /// What was asked.
    #[must_use]
    pub const fn asked(&self) -> &ToClick {
        &self.asked
    }

    /// Answer it, once, and let the connection go.
    pub fn answered(mut self, clicked: Clicked) {
        let _ = self.talking.write_all(clicked.written().as_bytes());
        let _ = self.talking.write_all(b"\n");
        let _ = self.talking.flush();
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "a_click_at_the_door_tests.rs"]
mod tests;
