//! Knocking at the compositor's door: one line out, one line back.
//!
//! `docs/contracts/application-door.md` is the published surface and
//! [ADR 0108](../../../docs/decisions/0108-the-compositor-carries-an-application-verb-out-and-needs-no-token-to.md)
//! is why the door exists and why nothing proves anything to it.
//! `alo_turn::TheDoorTheCompositorServes` is the trait; this is the one
//! implementation that reaches a real socket.
//!
//! # Why the path is spelt here rather than borrowed
//!
//! `alo_shell::the_application_doors_place` declares the same two names. This
//! file does **not** depend on that crate and must not: `alo-shell` is a
//! Wayland compositor, and a daemon that linked one to learn a filename would
//! be unbuildable anywhere a compositor is not.
//!
//! So the name is written twice, which is exactly the shape this repository
//! distrusts — *two constants agree with each other whatever either of them
//! says*. What makes it safe is that neither is held to the other: both are
//! held to the **contract document**, which is the thing a third party reads
//! too. `tests/the_contract_describes_this_door.rs` is that check on this side.
//!
//! # One connection per act, and no session
//!
//! A new connection for every request, and that is ADR 0108's rule rather than
//! an inefficiency: *one line is one act*, because nothing about this door is an
//! authorisation — the authority was the turn's and it was spent before the line
//! was written. A kept connection would be a thing that looks like a session.
//!
//! It is also the only arrangement that survives the ordinary case: the
//! compositor may start after the daemon, stop, and start again, and a daemon
//! holding a connection from before would be holding a pipe to a process that
//! has gone.
//!
//! # It waits, and the compositor does not
//!
//! The compositor reads this door from a frame loop and so must never block.
//! **This side is the opposite**: a turn is already waiting for an answer, and a
//! blocking read with a bound is simpler and more honest than a poll. The bound
//! matters — a wedged compositor must not hold a turn open for ever — and
//! `PATIENCE` is it.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_acting::{Answer, Request};
use alo_turn::{NotAtTheDoor, TheDoorTheCompositorServes};

/// The compositor's socket, beside the daemon's own in the same directory.
///
/// Part of `docs/contracts/application-door.md`'s published surface, and held
/// to that document by this crate's own test rather than to `alo-shell`'s copy.
pub const THE_COMPOSITORS_SOCKET: &str = "shell.sock";

/// How long a turn may wait for the compositor to answer one line.
///
/// Generous for a local socket where the answer is one word, and short enough
/// that a wedged compositor loses a turn rather than wedging the daemon. The
/// compositor's own bound is in *frames* because it must not block at all;
/// this one is in seconds because waiting is this side's job.
const PATIENCE: Duration = Duration::from_secs(5);

/// The compositor's door on this machine.
///
/// **Holds a path and never a connection.** One is made per act and dropped
/// with it, which is this file's own documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtTheCompositorsDoor {
    /// Where to knock.
    socket: PathBuf,
}

impl AtTheCompositorsDoor {
    /// The door in this person's own directory, beside the daemon's socket.
    ///
    /// Takes the directory rather than working it out, because the daemon
    /// already knows it: `crate::place` owns `/run/alo/<uid>` and made it.
    /// Two places deciding one path is how they come to disagree.
    #[must_use]
    pub fn beside(directory: &Path) -> Self {
        Self {
            socket: directory.join(THE_COMPOSITORS_SOCKET),
        }
    }

    /// Where this would knock.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// Connect, with both bounds set before anything is written.
    fn knocking(&self) -> Result<UnixStream, NotAtTheDoor> {
        // **A path that is not there is *no compositor*, not a failure.** A
        // machine at the sign-in screen is in exactly that state, and
        // `alo_turn` turns it into the person's own *this machine could not
        // reach {application}*.
        let talking = UnixStream::connect(&self.socket).map_err(|why| match why.kind() {
            std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused => {
                NotAtTheDoor::NoCompositor
            }
            _ => NotAtTheDoor::NotAnswered {
                why: why.to_string(),
            },
        })?;
        // **Set one at a time.** An array of both would run the second call
        // whatever the first answered, and then report only the first — two
        // syscalls made to find out about one.
        talking
            .set_read_timeout(Some(PATIENCE))
            .map_err(|why| NotAtTheDoor::NotAnswered {
                why: why.to_string(),
            })?;
        talking
            .set_write_timeout(Some(PATIENCE))
            .map_err(|why| NotAtTheDoor::NotAnswered {
                why: why.to_string(),
            })?;
        Ok(talking)
    }
}

impl TheDoorTheCompositorServes for AtTheCompositorsDoor {
    fn asking(&mut self, request: &Request) -> Result<Answer, NotAtTheDoor> {
        let mut talking = self.knocking()?;
        let said = format!("{}\n", request.written());
        talking
            .write_all(said.as_bytes())
            .and_then(|()| talking.flush())
            .map_err(|why| NotAtTheDoor::NotAnswered {
                why: why.to_string(),
            })?;

        let mut heard = String::new();
        let read = BufReader::new(&talking)
            .read_line(&mut heard)
            .map_err(|why| NotAtTheDoor::NotAnswered {
                why: why.to_string(),
            })?;
        if read == 0 {
            // **Closed with nothing written** is what the compositor does to a
            // caller it will not serve, and to a line it could not read. From
            // here both are *it did not answer*: a daemon that read silence as
            // success would report work that never happened.
            return Err(NotAtTheDoor::NotAnswered {
                why: "the compositor closed the connection without answering".to_owned(),
            });
        }
        // **An unknown word is never read as success**, which is
        // `alo_acting::Answer::read`'s own asymmetry: a compositor newer or
        // older than this daemon must not be able to say something that reads
        // as *done*.
        Answer::read(&heard).ok_or_else(|| NotAtTheDoor::NotAnswered {
            why: format!("the compositor answered {heard:?}, which this machine does not know"),
        })
    }
}

#[cfg(test)]
// Counted: 11 `unwrap` and 3 `panic!`, and no `expect` — an unfulfilled
// expectation is an error in this workspace, so only what is used is listed.
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;

    use alo_acting::{Acting, Answer, NotActed, Request};
    use alo_applications::Application;
    use alo_turn::{NotAtTheDoor, TheDoorTheCompositorServes as _};

    use super::{AtTheCompositorsDoor, THE_COMPOSITORS_SOCKET};

    /// A directory of this test's own, under the system temporary one.
    ///
    /// Shallow deliberately: a Unix socket path is bounded at about 108 bytes
    /// by the kernel, and a deep root would fail for that reason while looking
    /// like one of the refusals this file is about.
    fn a_directory(what: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("alo-knock-{}-{what}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// The mail client, as an act about it.
    fn focus_the_mail_client() -> Request {
        Request::to(Acting::Focus(
            Application::identified("org.gnome.Geary").unwrap(),
        ))
    }

    /// What a stand-in compositor does with each line it is handed.
    ///
    /// `Copy`, because it is matched by value inside the accept loop.
    #[derive(Clone, Copy)]
    enum Behaving {
        /// Answer this word, with a newline.
        Saying(&'static str),
        /// Read the line and close with nothing written.
        SayingNothing,
    }

    /// A stand-in compositor on a real socket, in a thread.
    ///
    /// Answers `how_many` connections and then stops, and reports every line it
    /// was handed — which is how *a new connection per act* is checked rather
    /// than assumed.
    fn a_compositor_that(
        at: &Path,
        behaving: Behaving,
        how_many: usize,
    ) -> (PathBuf, mpsc::Receiver<String>) {
        let socket = at.join(THE_COMPOSITORS_SOCKET);
        let listening = UnixListener::bind(&socket).unwrap();
        let (telling, heard) = mpsc::channel();
        std::thread::spawn(move || {
            for _ in 0..how_many {
                let Ok((talking, _)) = listening.accept() else {
                    return;
                };
                let mut line = String::new();
                if BufReader::new(&talking).read_line(&mut line).is_err() {
                    return;
                }
                if telling.send(line).is_err() {
                    return;
                }
                match behaving {
                    Behaving::Saying(word) => {
                        let mut writing = &talking;
                        let _ = writing.write_all(format!("{word}\n").as_bytes());
                        let _ = writing.flush();
                    }
                    // Dropping it is the whole behaviour.
                    Behaving::SayingNothing => {}
                }
                drop(talking);
            }
        });
        (socket, heard)
    }

    /// **The socket is beside the daemon's own, named what the contract says.**
    #[test]
    fn the_door_is_beside_the_daemons_own_socket() {
        let door = AtTheCompositorsDoor::beside(std::path::Path::new("/run/alo/1000"));
        assert_eq!(
            door.socket(),
            std::path::Path::new("/run/alo/1000/shell.sock")
        );
        assert_eq!(THE_COMPOSITORS_SOCKET, "shell.sock");
    }

    /// **No socket is *no compositor*, not a failure.** A machine at the
    /// sign-in screen is in exactly that state, and `alo_turn` turns this into
    /// the person's own *this machine could not reach {application}*.
    #[test]
    fn a_machine_with_no_compositor_answers_no_compositor() {
        let at = a_directory("nobody-home");
        let mut door = AtTheCompositorsDoor::beside(&at);

        assert_eq!(
            door.asking(&focus_the_mail_client()),
            Err(NotAtTheDoor::NoCompositor)
        );

        let _ = std::fs::remove_dir_all(at);
    }

    /// **The whole round trip, over a real socket**: the exact line goes out
    /// and the one word comes back.
    #[test]
    fn one_line_goes_out_and_one_word_comes_back() {
        let at = a_directory("round-trip");
        let (_socket, heard) = a_compositor_that(&at, Behaving::Saying("acted"), 1);
        let mut door = AtTheCompositorsDoor::beside(&at);

        assert_eq!(door.asking(&focus_the_mail_client()), Ok(Answer::Acted));
        assert_eq!(heard.recv().unwrap(), "focus org.gnome.Geary\n");

        let _ = std::fs::remove_dir_all(at);
    }

    /// **Every refusal comes back as itself**, so the wire's words and this
    /// machine's vocabulary cannot drift apart.
    #[test]
    fn each_refusal_comes_back_as_itself() {
        for (word, expected) in [
            ("no-window", NotActed::NoWindow),
            ("not-installed", NotActed::NotInstalled),
            ("it-did-not", NotActed::ItDidNot),
            ("could-not-ask", NotActed::CouldNotAsk),
        ] {
            let at = a_directory(&format!("refusal-{word}"));
            let (_socket, _heard) = a_compositor_that(&at, Behaving::Saying(word), 1);
            let mut door = AtTheCompositorsDoor::beside(&at);

            assert_eq!(
                door.asking(&focus_the_mail_client()),
                Ok(Answer::NotActed(expected)),
                "{word} did not come back as itself"
            );

            let _ = std::fs::remove_dir_all(at);
        }
    }

    /// **A compositor that closes with nothing written is *it did not answer*,
    /// never *it worked*.**
    ///
    /// Closing with nothing is what the compositor does to a caller it will not
    /// serve and to a line it could not read. A daemon that read silence as
    /// success would report work that never happened.
    #[test]
    fn silence_is_never_read_as_success() {
        let at = a_directory("silence");
        let (_socket, _heard) = a_compositor_that(&at, Behaving::SayingNothing, 1);
        let mut door = AtTheCompositorsDoor::beside(&at);

        match door.asking(&focus_the_mail_client()) {
            Err(NotAtTheDoor::NotAnswered { why }) => {
                assert!(why.contains("without answering"), "{why}");
            }
            other => panic!("silence answered {other:?}"),
        }

        let _ = std::fs::remove_dir_all(at);
    }

    /// **A word this machine does not know is never read as success either.**
    ///
    /// The asymmetry `alo_acting::Answer::read` is built around: a compositor
    /// newer or older than this daemon must not be able to say something that
    /// reads as *done*.
    #[test]
    fn a_word_this_machine_does_not_know_is_not_success() {
        for word in ["probably", "acted-ish", "ACTED", "", "ok"] {
            let at = a_directory(&format!("unknown-{}", word.len()));
            let (_socket, _heard) = a_compositor_that(&at, Behaving::Saying(word), 1);
            let mut door = AtTheCompositorsDoor::beside(&at);

            match door.asking(&focus_the_mail_client()) {
                Err(NotAtTheDoor::NotAnswered { .. }) => {}
                other => panic!("{word:?} was read as {other:?}"),
            }

            let _ = std::fs::remove_dir_all(at);
        }
    }

    /// **One connection per act, and no session.**
    ///
    /// ADR 0108's rule rather than an efficiency question: nothing about this
    /// door is an authorisation, so a kept connection would be a thing that
    /// looks like one. It is also the only arrangement that survives a
    /// compositor stopping and starting between two acts.
    #[test]
    fn each_act_is_its_own_connection() {
        let at = a_directory("two-acts");
        let (_socket, heard) = a_compositor_that(&at, Behaving::Saying("acted"), 2);
        let mut door = AtTheCompositorsDoor::beside(&at);

        // The stand-in answers exactly two connections and stops. If the
        // daemon reused one, the second act would find nobody accepting.
        assert_eq!(door.asking(&focus_the_mail_client()), Ok(Answer::Acted));
        assert_eq!(door.asking(&focus_the_mail_client()), Ok(Answer::Acted));
        assert_eq!(heard.recv().unwrap(), "focus org.gnome.Geary\n");
        assert_eq!(heard.recv().unwrap(), "focus org.gnome.Geary\n");

        let _ = std::fs::remove_dir_all(at);
    }

    /// **Each of the three acts crosses as its own two words.**
    #[test]
    fn each_act_crosses_as_its_own_two_words() {
        let mail = Application::identified("org.gnome.Geary").unwrap();
        for (acting, expected) in [
            (Acting::Open(mail.clone()), "open org.gnome.Geary\n"),
            (Acting::Focus(mail.clone()), "focus org.gnome.Geary\n"),
            (Acting::Close(mail.clone()), "close org.gnome.Geary\n"),
        ] {
            let at = a_directory(&format!("act-{}", acting.on_the_wire()));
            let (_socket, heard) = a_compositor_that(&at, Behaving::Saying("acted"), 1);
            let mut door = AtTheCompositorsDoor::beside(&at);

            assert_eq!(door.asking(&Request::to(acting)), Ok(Answer::Acted));
            assert_eq!(heard.recv().unwrap(), expected);

            let _ = std::fs::remove_dir_all(at);
        }
    }
}
