//! **How this person writes a time and a day, told to the seat and passed on.**
//!
//! The owner's decision of 2026-10-28: a person chooses the region their dates
//! and times are written like, and **every** date and time on the machine
//! follows it — the status area's clock, the lock screen's clock and the record.
//! So there is one copy, here, and three roads to it:
//!
//! - **the desktop tells it at sign-in**, from `formats.toml` in the person's
//!   folder and the language they read — [`Server::the_formats_are`], asked
//!   through `TheDesktop::their_formats`, because a file in a person's folder
//!   is the desktop's to read and never this crate's;
//! - **Settings tells it again** the moment a region is kept, through the same
//!   method, so the record a person opens next is already written their way;
//! - **the desktop is told back** on the next frame —
//!   [`Server::the_formats_to_tell`] — because the status area's clock is
//!   written by the desktop, and a clock that waited for the next sign-in
//!   would show a person their choice had not worked.
//!
//! Nothing watches the file, which is the folder's rule (ADR 0038): a hand edit
//! is read at the next sign-in.

use crate::TheirFormats;

use crate::Server;

impl Server {
    /// **How this person writes a time and a day**, from now on.
    pub fn the_formats_are(&mut self, formats: TheirFormats) {
        if formats != self.formats {
            self.formats_untold = true;
        }
        self.formats = formats;
    }

    /// How this person writes a time and a day, as this seat was last told.
    #[must_use]
    pub const fn the_formats(&self) -> &TheirFormats {
        &self.formats
    }

    /// The formats, once, when they changed since the desktop was last told.
    pub(crate) fn the_formats_to_tell(&mut self) -> Option<TheirFormats> {
        if !self.formats_untold {
            return None;
        }
        self.formats_untold = false;
        Some(self.formats.clone())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;

    use alo_formats::Chosen;

    use crate::{Server, TheirFormats};

    fn belgian() -> TheirFormats {
        let mut chosen = Chosen::default();
        chosen.written_as_in(Some("BE"), "fr").unwrap();
        TheirFormats::of(chosen, Some("fr"))
    }

    /// **A change is passed on once, and nothing is passed on when nothing
    /// changed** — so the desktop is not told the same thing sixty times a
    /// second.
    #[test]
    fn a_change_is_told_once_and_no_change_is_not_told() {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut server = Server::bind(runtime.path(), "the-formats-told").unwrap();
        assert_eq!(
            server.the_formats_to_tell(),
            None,
            "told before anything changed"
        );
        server.the_formats_are(belgian());
        assert_eq!(server.the_formats_to_tell(), Some(belgian()));
        assert_eq!(server.the_formats_to_tell(), None, "told twice");
        server.the_formats_are(belgian());
        assert_eq!(
            server.the_formats_to_tell(),
            None,
            "told what had not changed"
        );
        assert_eq!(server.the_formats(), &belgian());
    }
}
