//! **A file alo was approved to open in an application**, as the door hands
//! it to the desktop (`open_file_in_application`; the owner, 2026-11-14:
//! *"Yes, add it"*).
//!
//! The door reads `open-file` and hands the ask on with its connection,
//! because **where it opens is the desktop's to decide**: alo's own copy at
//! the person's default, which only the desktop can start (it holds the
//! copies), or the person's own window where they chose that in Settings,
//! read from their own file at this ask. The desktop answers with where it
//! opened, or why nothing did (`alo_acting::Opened`), and the connection goes.

use std::io::Write as _;
use std::os::unix::net::UnixStream;
use std::path::Path;

use alo_acting::{Opened, ToOpenAFile};

/// One file alo asked to open, with the connection that asked.
#[derive(Debug)]
pub struct AskedToOpenAFile {
    /// What was asked.
    asked: ToOpenAFile,
    /// The connection, which is answered once.
    talking: UnixStream,
}

impl AskedToOpenAFile {
    /// The door read `asked` on `talking`.
    pub(crate) const fn of(asked: ToOpenAFile, talking: UnixStream) -> Self {
        Self { asked, talking }
    }

    /// The application, by its identifier.
    #[must_use]
    pub fn application(&self) -> &str {
        self.asked.application().identifier()
    }

    /// The file.
    #[must_use]
    pub fn file(&self) -> &Path {
        self.asked.file()
    }

    /// Answer where it opened, or why it did not, and let the connection go.
    /// A daemon that has gone is not told.
    pub fn answered(mut self, opened: Opened) {
        let _ = self.talking.write_all(opened.written().as_bytes());
        let _ = self.talking.write_all(b"\n");
        let _ = self.talking.flush();
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::io::Read as _;

    use super::*;

    /// **What the desktop answers is the one word the daemon reads**, and the
    /// connection is let go after it.
    #[test]
    fn the_answer_is_one_word_and_the_connection_goes() {
        let (daemon, desktop) = UnixStream::pair().unwrap();
        let asked = ToOpenAFile::with(
            alo_applications::Application::identified("com.vscodium.codium").unwrap(),
            Path::new("/var/home/anna/hello.txt"),
        )
        .unwrap();
        let one = AskedToOpenAFile::of(asked, desktop);
        assert_eq!(one.application(), "com.vscodium.codium");
        assert_eq!(one.file(), Path::new("/var/home/anna/hello.txt"));
        one.answered(Opened::InAlosCopy);
        let mut heard = String::new();
        let mut daemon = daemon;
        daemon.read_to_string(&mut heard).unwrap();
        assert_eq!(heard, "opened-in-alos-copy\n");
    }
}
