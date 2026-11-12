//! The portal service asking the desktop to start the application a link
//! opens in, at the desktop's door.
//!
//! The owner, 2026-11-12: *"Yes, add it (Recommended)"*. The wire is
//! `docs/contracts/application-door.md`'s `open-link` act: one line,
//! `open-link <application> <link>`, and one word back. The socket is
//! `/run/alo/<uid>/shell.sock`, the door `alo-agentd` already knocks at, and
//! this process may knock because it runs as the person — the door admits
//! that uid and nobody else (`alo-portald.service`, `User=`).
//!
//! # What each answer becomes
//!
//! | heard | [`LinkStarted`] |
//! |---|---|
//! | `acted` | `Asked` |
//! | `not-installed` | `NotInstalled` |
//! | no socket, or nobody listening | `NoDesktop` — a session at the sign-in screen, or a desktop that has stopped |
//! | anything else, nothing, or a timeout | `CouldNotAsk` |
//!
//! **An unknown answer is never read as success**, the door's own rule.
//!
//! # And the person choosing — `choose-link`
//!
//! When an application asks for the person to choose, the line is
//! `choose-link <asking> <link> <default> [<another> …]` and the desktop holds
//! the connection while its *Open with* sheet is up. The desktop takes the
//! sheet down after five minutes and answers `unanswered`; this waits
//! [`THE_PERSONS_PATIENCE`] — a little longer — so a desktop that froze cannot
//! hold an application's request for ever, and a silence is `could-not-ask`,
//! never a choice.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_acting::{Answer, Chose, NotActed, ToChooseALink, ToOpenALink};
use alo_applications::{Application, Link};
use alo_portals::{LinkStarted, OpensLinks, SheetAnswer};

/// The desktop's socket in a person's own directory.
pub const THE_DESKTOPS_SOCKET: &str = "shell.sock";

/// How long the desktop has to answer. It answers from its frame loop in the
/// same breath as reading, so this is generous; it is a bound so a desktop
/// that has frozen cannot hold an application's request for ever.
const PATIENCE: Duration = Duration::from_secs(5);

/// How long the person has to answer the *Open with* sheet, as this side
/// counts it: the desktop's five minutes and half a minute more.
const THE_PERSONS_PATIENCE: Duration = Duration::from_secs(5 * 60 + 30);

/// The desktop's door on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtTheDesktopsDoor {
    /// Where to knock.
    socket: PathBuf,
}

impl AtTheDesktopsDoor {
    /// The door for the person with this uid: `/run/alo/<uid>/shell.sock`.
    #[must_use]
    pub fn of(uid: u32) -> Self {
        Self::at(
            Path::new("/run/alo")
                .join(uid.to_string())
                .join(THE_DESKTOPS_SOCKET),
        )
    }

    /// The door at this socket — for a test, which serves one of its own.
    #[must_use]
    pub fn at(socket: impl Into<PathBuf>) -> Self {
        Self {
            socket: socket.into(),
        }
    }

    /// Where this knocks.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// Say the line and read one line back, waiting `patience` for it.
    fn said(&self, line: &str, patience: Duration) -> Heard {
        let mut talking = match UnixStream::connect(&self.socket) {
            Ok(talking) => talking,
            Err(why)
                if matches!(
                    why.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) =>
            {
                return Heard::NoDesktop;
            }
            Err(_) => return Heard::Nothing,
        };
        if talking.set_read_timeout(Some(patience)).is_err()
            || talking.set_write_timeout(Some(PATIENCE)).is_err()
        {
            return Heard::Nothing;
        }
        if talking
            .write_all(format!("{line}\n").as_bytes())
            .and_then(|()| talking.flush())
            .is_err()
        {
            return Heard::Nothing;
        }
        let mut heard = String::new();
        if BufReader::new(&talking).read_line(&mut heard).is_err() {
            return Heard::Nothing;
        }
        Heard::This(heard)
    }

    /// Say the line and read the one word back.
    fn asked(&self, line: &str) -> LinkStarted {
        let heard = match self.said(line, PATIENCE) {
            Heard::This(heard) => heard,
            Heard::NoDesktop => return LinkStarted::NoDesktop,
            Heard::Nothing => return LinkStarted::CouldNotAsk,
        };
        match Answer::read(heard.trim_end_matches(['\r', '\n'])) {
            Some(Answer::Acted) => LinkStarted::Asked,
            Some(Answer::NotActed(NotActed::NotInstalled)) => LinkStarted::NotInstalled,
            Some(Answer::NotActed(_)) | None => LinkStarted::CouldNotAsk,
        }
    }
}

/// What came back from the desktop.
enum Heard {
    /// One line, with its newline.
    This(String),
    /// No socket, or nobody listening at it.
    NoDesktop,
    /// It could not be asked, or said nothing in time.
    Nothing,
}

impl OpensLinks for AtTheDesktopsDoor {
    fn opened(&self, application: &str, link: &Link) -> LinkStarted {
        let Ok(application) = Application::identified(application) else {
            return LinkStarted::CouldNotAsk;
        };
        self.asked(&ToOpenALink::with(application, link.clone()).written())
    }

    fn chosen(&self, asking: &str, link: &Link, offered: &[String]) -> SheetAnswer {
        let Ok(asking) = Application::identified(asking) else {
            return SheetAnswer::CouldNotAsk;
        };
        // **The default first, and at most as many as one sheet offers**: a
        // machine with more browsers than that offers its first sixteen.
        let offered: Vec<Application> = offered
            .iter()
            .take(alo_acting::choosing_a_link::THE_MOST_OFFERED)
            .filter_map(|one| Application::identified(one).ok())
            .collect();
        let Ok(asked) = ToChooseALink::of(asking, link.clone(), offered) else {
            return SheetAnswer::CouldNotAsk;
        };
        let heard = match self.said(&asked.written(), THE_PERSONS_PATIENCE) {
            Heard::This(heard) => heard,
            Heard::NoDesktop => return SheetAnswer::NoDesktop,
            Heard::Nothing => return SheetAnswer::CouldNotAsk,
        };
        match Chose::read(&heard, &asked) {
            Some(Chose::Chosen(application)) => {
                SheetAnswer::Picked(application.identifier().to_owned())
            }
            Some(Chose::Cancelled) => SheetAnswer::Cancelled,
            Some(Chose::Unanswered) => SheetAnswer::Unanswered,
            Some(Chose::CannotShow) => SheetAnswer::CannotShow,
            None => SheetAnswer::CouldNotAsk,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::Read as _;
    use std::os::unix::net::UnixListener;

    /// A desktop of this test's own that hears one line and answers `word`.
    fn a_desktop_answering(
        what: &str,
        word: &'static str,
    ) -> (PathBuf, std::thread::JoinHandle<String>) {
        let folder =
            std::env::temp_dir().join(format!("alo-portald-door-{}-{what}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let socket = folder.join(THE_DESKTOPS_SOCKET);
        let listening = UnixListener::bind(&socket).unwrap();
        let hearing = std::thread::spawn(move || {
            let (mut talking, _) = listening.accept().unwrap();
            let mut heard = Vec::new();
            let mut byte = [0_u8; 1];
            while talking.read(&mut byte).unwrap() == 1 && byte[0] != b'\n' {
                heard.push(byte[0]);
            }
            talking.write_all(word.as_bytes()).unwrap();
            String::from_utf8(heard).unwrap()
        });
        (socket, hearing)
    }

    fn a_link() -> Link {
        Link::checked("https://example.com/invoice").unwrap()
    }

    /// **The line is the contract's, and `acted` is the only success.**
    #[test]
    fn the_desktop_is_asked_in_the_contracts_words() {
        for (what, word, started) in [
            ("acted", "acted\n", LinkStarted::Asked),
            (
                "not-installed",
                "not-installed\n",
                LinkStarted::NotInstalled,
            ),
            ("could-not-ask", "could-not-ask\n", LinkStarted::CouldNotAsk),
            ("unknown", "opened-it-promise\n", LinkStarted::CouldNotAsk),
            ("silent", "", LinkStarted::CouldNotAsk),
        ] {
            let (socket, hearing) = a_desktop_answering(what, word);
            let door = AtTheDesktopsDoor::at(&socket);
            assert_eq!(
                door.opened("org.mozilla.firefox", &a_link()),
                started,
                "{what}"
            );
            assert_eq!(
                hearing.join().unwrap(),
                "open-link org.mozilla.firefox https://example.com/invoice"
            );
        }
    }

    /// **The person's choice is asked in the contract's words, and only an
    /// answer naming what was offered is a choice.**
    #[test]
    fn the_person_is_asked_to_choose_in_the_contracts_words() {
        let offered = [
            "org.mozilla.firefox".to_owned(),
            "io.gitlab.librewolf-community".to_owned(),
        ];
        for (what, word, chose) in [
            (
                "chosen",
                "chosen io.gitlab.librewolf-community\n",
                SheetAnswer::Picked("io.gitlab.librewolf-community".to_owned()),
            ),
            ("cancelled", "cancelled\n", SheetAnswer::Cancelled),
            ("unanswered", "unanswered\n", SheetAnswer::Unanswered),
            ("cannot-show", "cannot-show\n", SheetAnswer::CannotShow),
            (
                "not-offered",
                "chosen org.example.Evil\n",
                SheetAnswer::CouldNotAsk,
            ),
            ("silent", "", SheetAnswer::CouldNotAsk),
        ] {
            let (socket, hearing) = a_desktop_answering(&format!("choose-{what}"), word);
            let door = AtTheDesktopsDoor::at(&socket);
            assert_eq!(
                door.chosen("org.gnome.Papers", &a_link(), &offered),
                chose,
                "{what}"
            );
            assert_eq!(
                hearing.join().unwrap(),
                "choose-link org.gnome.Papers https://example.com/invoice org.mozilla.firefox \
                 io.gitlab.librewolf-community"
            );
        }
        let nobody = AtTheDesktopsDoor::at(std::env::temp_dir().join(format!(
            "alo-portald-no-desktop-to-choose-{}/shell.sock",
            std::process::id()
        )));
        assert_eq!(
            nobody.chosen("org.gnome.Papers", &a_link(), &offered),
            SheetAnswer::NoDesktop
        );
    }

    /// **No desktop is said as no desktop**, not as a failure to ask one.
    #[test]
    fn no_desktop_is_said_as_no_desktop() {
        let door = AtTheDesktopsDoor::at(std::env::temp_dir().join(format!(
            "alo-portald-no-desktop-{}/shell.sock",
            std::process::id()
        )));
        assert_eq!(
            door.opened("org.mozilla.firefox", &a_link()),
            LinkStarted::NoDesktop
        );
        assert_eq!(
            AtTheDesktopsDoor::of(1000).socket(),
            Path::new("/run/alo/1000/shell.sock")
        );
    }
}
