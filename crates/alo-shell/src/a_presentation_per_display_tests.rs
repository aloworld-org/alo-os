//! **A session holds one presentation per display** —
//! `docs/autonomy/more-than-one-display-plan.md` task 3.
//!
//! What these hold is the protocol state: two `wl_output` globals, each with
//! its own metadata, and a retirement that reaches both. What they do **not**
//! hold is a second frame being drawn — that is task 4 — so each display's
//! frame is submitted here by calling `render` twice, which is what the loop
//! will do once it exists.

use std::cell::RefCell;

use smithay::utils::{Physical, Size};

use crate::{FrameTarget, OutputMetadata, RenderError, Server};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// A display that submits nothing and says who it is.
struct Screen {
    /// The connector name, which is what tells two displays apart.
    named: &'static str,
    /// Its mode, so the two differ in more than a name.
    size: (i32, i32),
    /// Whether `retire` refuses, for the backend-is-going-away case.
    retires: bool,
    /// How many times this display was asked to retire.
    retired: RefCell<usize>,
}

impl Screen {
    fn new(named: &'static str, size: (i32, i32)) -> Self {
        Self {
            named,
            size,
            retires: true,
            retired: RefCell::new(0),
        }
    }
}

impl FrameTarget for Screen {
    fn size(&self) -> Size<i32, Physical> {
        self.size.into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        Ok(vec![])
    }
    fn retire(&mut self) -> Result<(), RenderError> {
        *self.retired.borrow_mut() += 1;
        if self.retires {
            Ok(())
        } else {
            Err(RenderError::RetirementUnsupported)
        }
    }
    fn metadata(&self) -> Result<OutputMetadata, RenderError> {
        Ok(OutputMetadata {
            name: self.named.to_owned(),
            make: "alo".to_owned(),
            model: "test".to_owned(),
            physical_size: (310, 170),
            refresh: 60_000,
        })
    }
}

/// A server on a socket of its own.
///
/// The runtime directory is `0700` because `Socket::bind` refuses anything
/// wider — a Wayland socket somebody else can reach is not this session's.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// **Two displays are two presentations, each with its own output.**
///
/// The thing this replaces refused outright: one `Presentation` holds the
/// identity of the display it has seen, and `validate_target` answers
/// `OutputIdentityChanged` for any other — so handing the server a second
/// display's frame was not *undrawn*, it was **refused**.
#[test]
fn two_displays_are_two_presentations_each_with_its_own_output() {
    let (_directory, mut server) = server("two-displays");
    let mut left = Screen::new("DP-1", (1280, 720));
    let mut right = Screen::new("HDMI-A-1", (1920, 1080));

    server
        .render(&mut left, 1)
        .expect("the first display draws");
    server
        .render(&mut right, 2)
        .expect("the second display draws rather than being refused");

    let named: Vec<&str> = server.presentations.keys().map(String::as_str).collect();
    assert_eq!(
        named,
        vec!["DP-1", "HDMI-A-1"],
        "a display is missing, or two share one presentation"
    );
    assert!(
        server
            .presentations
            .values()
            .all(|one| one.output.is_some() && one.global.is_some()),
        "a display has no wl_output global of its own"
    );
}

/// **Each display's metadata is its own**, which is what makes the globals
/// describe different screens rather than two names for one.
#[test]
fn each_display_keeps_its_own_metadata() {
    let (_directory, mut server) = server("own-metadata");
    let mut left = Screen::new("DP-1", (1280, 720));
    let mut right = Screen::new("HDMI-A-1", (1920, 1080));
    server.render(&mut left, 1).expect("the first draws");
    server.render(&mut right, 2).expect("the second draws");

    let sizes: Vec<Option<(i32, i32)>> = server
        .presentations
        .values()
        .map(|one| {
            one.output
                .as_ref()
                .and_then(|output| output.current_mode())
                .map(|mode| (mode.size.w, mode.size.h))
        })
        .collect();
    assert_eq!(
        sizes,
        vec![Some((1280, 720)), Some((1920, 1080))],
        "the two displays do not have their own modes"
    );
}

/// **The same display twice is the same presentation**, not a second one.
///
/// A name is the identity, so a display drawing frame after frame must not
/// grow a presentation per frame — which is the failure a map keyed by
/// anything else would have.
#[test]
fn the_same_display_drawn_twice_is_one_presentation() {
    let (_directory, mut server) = server("same-twice");
    let mut screen = Screen::new("DP-1", (1280, 720));
    server.render(&mut screen, 1).expect("a frame");
    server.render(&mut screen, 2).expect("another frame");
    assert_eq!(server.presentations.len(), 1);
}

/// **A display whose make changes under the same name is still a fault.**
///
/// The refusal that used to stand for *this compositor has one output* keeps
/// the job it was written for: a connector that reports a different monitor
/// without changing its name is something to refuse, not a third display.
#[test]
fn an_identity_changing_under_one_name_is_still_refused() {
    let (_directory, mut server) = server("identity");
    let mut screen = Screen::new("DP-1", (1280, 720));
    server.render(&mut screen, 1).expect("a frame");

    struct Swapped;
    impl FrameTarget for Swapped {
        fn size(&self) -> Size<i32, Physical> {
            (1280, 720).into()
        }
        fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
            Ok(vec![])
        }
        fn metadata(&self) -> Result<OutputMetadata, RenderError> {
            Ok(OutputMetadata {
                name: "DP-1".to_owned(),
                make: "somebody else".to_owned(),
                model: "a different monitor".to_owned(),
                physical_size: (500, 300),
                refresh: 60_000,
            })
        }
    }

    assert!(
        matches!(
            server.render(&mut Swapped, 2),
            Err(RenderError::OutputIdentityChanged)
        ),
        "a different monitor on the same connector was taken as the same display"
    );
}

/// **Retiring the session withdraws every display's global, and asks the
/// backend once.**
///
/// `target.retire()` is the backend going away, not a display. Calling it per
/// presentation would retire the same backend twice and refuse the second
/// time, which is the fault the split exists to avoid.
#[test]
fn retiring_withdraws_every_global_and_asks_the_backend_once() {
    let (_directory, mut server) = server("retiring");
    let mut left = Screen::new("DP-1", (1280, 720));
    let mut right = Screen::new("HDMI-A-1", (1920, 1080));
    server.render(&mut left, 1).expect("the first draws");
    server.render(&mut right, 2).expect("the second draws");

    server
        .retire_output(&mut right)
        .expect("the session retires");

    assert_eq!(
        *right.retired.borrow(),
        1,
        "the backend was asked to retire once per display"
    );
    assert!(
        server
            .presentations
            .values()
            .all(|one| one.output.is_none() && one.global.is_none()),
        "a display kept its global through a retirement"
    );
}

/// **A target naming no display this session presented is refused.**
///
/// With one display this was *the identity changed*; with several it is *that
/// is not one of mine*, and both are the same mistake — retiring through a
/// target the compositor never drew to.
#[test]
fn retiring_through_a_display_nobody_presented_is_refused() {
    let (_directory, mut server) = server("not-mine");
    let mut screen = Screen::new("DP-1", (1280, 720));
    server.render(&mut screen, 1).expect("a frame");

    let mut stranger = Screen::new("VGA-1", (800, 600));
    assert!(
        matches!(
            server.retire_output(&mut stranger),
            Err(RenderError::OutputIdentityChanged)
        ),
        "the session retired through a display it never drew to"
    );
    assert_eq!(*stranger.retired.borrow(), 0, "the backend was still asked");
}
