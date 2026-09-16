//! The rented screen-capture mechanism, reached.
//!
//! [`TheScreenCast`] is [`crate::Grabs`] on a real machine: it asks the rented
//! mechanism for one frame of the screen and hands back what came out. That is
//! the whole of it, on purpose — nothing is decided here, and nothing about the
//! mechanism is patched, wrapped or extended (ADR 0011: the media server, the
//! screen cast and the encoder are rented, configured and never written).
//!
//! # The frames come through the machine's own media server, and that is why
//! the indicator works
//!
//! The screen reaches this machine's graph as a stream that the screen cast
//! opened, and this reads one frame off it. It could have been done another
//! way — a compositor has the pixels and could have handed them over directly —
//! and that way would have been invisible to `alo-in-use`, which reads the
//! media server's record and nothing else. Going through the media server means
//! alo OS's own picture of the screen is on the indicator **because it is a
//! stream like anybody else's**, which is `docs/features.md`'s *by any
//! application, including ours* meant rather than claimed. [`crate::announcing`]
//! is the other half.
//!
//! # No shell, and an environment cleared down to what it needs
//!
//! The program is started directly, so no argument is interpreted by anything
//! but the tool. Its environment is cleared and given back four things: where
//! the machine's own programs are, the C locale — what it prints is read to
//! decide things, and a translated message would be an answer nothing
//! recognised — the properties the capture announces itself under, and the one
//! variable a client needs in order to find the machine's media server at all.
//! The shape is `alo_in_use::TheMediaServer`'s, and the fourth is the
//! difference: a media server's client that cannot find its socket is a
//! screenshot that never happens.
//!
//! # What it is told
//!
//! Which stream to read, how many frames to take — one — and how much of each
//! edge to cut away, which is [`crate::What::across`] already decided. It is
//! not told what is being captured and not told where the picture is going.
//!
//! # A machine that cannot take a picture says so
//!
//! Three different things can go wrong and they are three different sentences,
//! because a person fixes them differently: the tool is not there
//! ([`crate::NotGrabbed::NothingReadsTheScreen`]), it is there and failed
//! ([`crate::NotGrabbed::NoPicture`]), or it answered with nothing usable
//! ([`crate::NotGrabbed::NothingCameBack`]).

use std::io::ErrorKind;
use std::process::{Command, Stdio};

use crate::announcing;
use crate::grabs::{Grabs, NotGrabbed};
use crate::picture::Picture;
use crate::region::Region;
use crate::screen::Screen;

/// The rented mechanism's own tool for reading a stream and writing a picture.
const THE_TOOL: &str = "gst-launch-1.0";

/// Where a machine's own programs are, for a cleared environment.
const WHERE_ITS_PROGRAMS_ARE: &str = "/usr/bin:/bin";

/// The variable a client of the media server finds its socket through.
///
/// Passed through rather than invented: what it holds is where this session's
/// own runtime files are, which is the session's to say and not this crate's.
/// Without it the tool cannot reach the machine's media server at all.
const WHERE_THE_SESSION_KEEPS_ITS_SOCKETS: &str = "XDG_RUNTIME_DIR";

/// The variable the media server reads a client's own properties from.
const WHAT_A_CLIENT_CALLS_ITSELF: &str = "PIPEWIRE_PROPS";

/// This machine's screen-capture mechanism, reading one stream of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheScreenCast {
    /// The program asked for a picture.
    program: String,
    /// The stream on the machine's graph that the screen cast opened.
    stream: u32,
}

impl TheScreenCast {
    /// The mechanism on this machine, reading the stream the screen cast opened
    /// for what the person picked.
    ///
    /// The stream is the screen cast's own number for it. Opening that stream
    /// is the picking — the whole screen, one window, or nothing — and it
    /// belongs to the shell plan's later tasks and to `alo-portals`, not here.
    #[must_use]
    pub fn reading(stream: u32) -> Self {
        Self {
            program: THE_TOOL.to_owned(),
            stream,
        }
    }

    /// The same, through a program named here.
    ///
    /// Test-only, and deliberately: a machine has one screen-capture mechanism,
    /// and a public way of pointing a screenshot at another program would be a
    /// second answer to *what is reading my screen*.
    #[cfg(test)]
    fn reached_by(program: &str, stream: u32) -> Self {
        Self {
            program: program.to_owned(),
            stream,
        }
    }

    /// What the tool is told: one frame of that stream, cut to that rectangle,
    /// as a picture on its output.
    ///
    /// Split out so that the whole of *which pixels* is a value a test can read
    /// without a machine — which is the half a rented tool cannot be asked to
    /// get right on our behalf.
    fn asking_for(&self, across: Region, on: Screen) -> Vec<String> {
        let stream = self.stream;
        let (left, top) = (across.from_the_left(), across.from_the_top());
        let (right, bottom) = (across.to_the_right_on(on), across.below_on(on));
        vec![
            // Say nothing but the picture: what it prints is the file.
            "-q".to_owned(),
            format!("pipewiresrc path={stream} num-buffers=1"),
            "!".to_owned(),
            "videoconvert".to_owned(),
            "!".to_owned(),
            format!("videocrop left={left} right={right} top={top} bottom={bottom}"),
            "!".to_owned(),
            "pngenc".to_owned(),
            "!".to_owned(),
            "fdsink fd=1".to_owned(),
        ]
    }

    /// What the tool answered, turned into a picture.
    ///
    /// Split from starting it so that every way of answering badly is a test
    /// rather than a paragraph: a machine cannot be made to fail a rented tool
    /// on demand, and the reading of a failure is the half that has to be right.
    fn what_it_answered(
        &self,
        succeeded: bool,
        said: &[u8],
        instead: &[u8],
    ) -> Result<Picture, NotGrabbed> {
        if !succeeded {
            return Err(NotGrabbed::NoPicture {
                said: format!(
                    "{} failed: {}",
                    self.program,
                    String::from_utf8_lossy(instead).trim()
                ),
            });
        }
        Picture::of(said.to_vec())
    }
}

impl Grabs for TheScreenCast {
    fn grab(&mut self, across: Region, on: Screen) -> Result<Picture, NotGrabbed> {
        let mut asking = Command::new(&self.program);
        asking
            .args(self.asking_for(across, on))
            .env_clear()
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("PATH", WHERE_ITS_PROGRAMS_ARE)
            .env(WHAT_A_CLIENT_CALLS_ITSELF, announcing::announced())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(sockets) = std::env::var_os(WHERE_THE_SESSION_KEEPS_ITS_SOCKETS) {
            asking.env(WHERE_THE_SESSION_KEEPS_ITS_SOCKETS, sockets);
        }
        match asking.output() {
            Ok(output) => {
                self.what_it_answered(output.status.success(), &output.stdout, &output.stderr)
            }
            Err(why) if why.kind() == ErrorKind::NotFound => {
                Err(NotGrabbed::NothingReadsTheScreen {
                    said: format!("{} is not on this machine: {why}", self.program),
                })
            }
            Err(why) => Err(NotGrabbed::NoPicture {
                said: format!("{} could not be started: {why}", self.program),
            }),
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

    /// The screen these tests are about.
    fn a_screen() -> Screen {
        Screen::measuring(1920, 1080).unwrap()
    }

    /// **The mechanism is told which stream, one frame, and what to cut away**,
    /// and the four edges are the rectangle this crate already decided.
    #[test]
    fn the_mechanism_is_told_one_frame_of_one_stream_and_what_to_cut_away() {
        let cast = TheScreenCast::reading(57);
        let asking = cast
            .asking_for(Region::of(100, 50, 400, 300).unwrap(), a_screen())
            .join(" ");

        assert!(asking.contains("path=57"), "{asking}");
        assert!(asking.contains("num-buffers=1"), "{asking}");
        assert!(
            asking.contains("left=100 right=1420 top=50 bottom=730"),
            "{asking}"
        );
    }

    /// **The whole screen cuts nothing away**, which is the same path through
    /// the same pipeline rather than a second one that could disagree with it.
    #[test]
    fn the_whole_screen_cuts_nothing_away() {
        let cast = TheScreenCast::reading(57);
        let asking = cast
            .asking_for(Region::over(a_screen()), a_screen())
            .join(" ");
        assert!(asking.contains("left=0 right=0 top=0 bottom=0"), "{asking}");
    }

    /// **Nothing in what the tool is told says where the picture goes**, and
    /// nothing says what is being captured: a mechanism that knew about a
    /// folder could put a picture somewhere nobody asked.
    #[test]
    fn the_mechanism_is_never_told_where_the_picture_goes() {
        let cast = TheScreenCast::reading(57);
        let asking = cast
            .asking_for(Region::of(0, 0, 10, 10).unwrap(), a_screen())
            .join(" ");
        for elsewhere in ["location=", "filesink", "/home", "clipboard"] {
            assert!(!asking.contains(elsewhere), "{asking} names {elsewhere}");
        }
        assert!(asking.contains("fdsink"), "{asking}");
    }

    /// **A machine with nothing that reads the screen says so**, in its own
    /// sentence rather than by handing back an empty picture. This is the
    /// refusal every machine without the rented mechanism takes, including
    /// every machine this test suite runs on.
    #[test]
    fn a_machine_with_no_mechanism_refuses_rather_than_returning_nothing() {
        let mut nowhere = TheScreenCast::reached_by("alo-there-is-no-such-program", 1);
        let refused = nowhere
            .grab(Region::of(0, 0, 10, 10).unwrap(), a_screen())
            .unwrap_err();
        assert!(
            matches!(refused, NotGrabbed::NothingReadsTheScreen { .. }),
            "{refused:?}"
        );
        assert!(refused.diagnosis().contains("alo-there-is-no-such-program"));
    }

    /// **A tool that ran and failed is not a picture**, and what it said is
    /// kept for whoever is fixing it.
    #[test]
    fn a_tool_that_ran_and_failed_is_refused_with_what_it_said() {
        let cast = TheScreenCast::reading(57);
        let refused = cast
            .what_it_answered(false, b"", b"no such node 57\n")
            .unwrap_err();
        assert!(
            matches!(refused, NotGrabbed::NoPicture { .. }),
            "{refused:?}"
        );
        assert!(refused.diagnosis().contains("no such node 57"));
        assert!(refused.diagnosis().contains(THE_TOOL));
    }

    /// **A tool that succeeded and printed nothing is refused too**, rather
    /// than read as a very small picture that would reach somebody's folder as
    /// an empty file under a message saying it was saved.
    #[test]
    fn a_tool_that_answered_with_nothing_is_refused() {
        let cast = TheScreenCast::reading(57);
        let refused = cast.what_it_answered(true, b"", b"").unwrap_err();
        assert!(
            matches!(refused, NotGrabbed::NothingCameBack { .. }),
            "{refused:?}"
        );
    }

    /// **A picture that came back is the picture**, exactly as it arrived.
    #[test]
    fn a_picture_that_came_back_is_handed_on_as_it_arrived() {
        let cast = TheScreenCast::reading(57);
        let picture = cast
            .what_it_answered(true, b"\x89PNG\r\n\x1a\nthe rest", b"")
            .unwrap();
        assert!(picture.bytes().starts_with(b"\x89PNG"));
    }

    /// **The machine's mechanism is the rented tool** and nothing else — the
    /// one place that name appears.
    #[test]
    fn the_machines_mechanism_is_the_rented_tool() {
        assert_eq!(TheScreenCast::reading(1).program, THE_TOOL);
    }

    /// **The capture announces itself as alo OS**, which is what puts it on the
    /// indicator for its moment; `announcing.rs` is where that argument lives.
    #[test]
    fn the_capture_announces_itself_as_alo_os() {
        let announced = announcing::announced();
        assert!(announced.contains(alo_in_use::heard::ALO_OSS_OWN_IDENTIFIER));
        assert_eq!(WHAT_A_CLIENT_CALLS_ITSELF, "PIPEWIRE_PROPS");
    }
}
