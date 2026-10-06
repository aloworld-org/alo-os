//! What the compositor says about its displays, and what it does with the
//! answer.

use super::*;
use crate::{FrameTarget, OutputMetadata, RenderError};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Physical, Size};

/// A display that submits nothing and describes itself.
struct Screen {
    named: &'static str,
    size: (i32, i32),
    make: &'static str,
    millimetres: (i32, i32),
}

impl FrameTarget for Screen {
    fn size(&self) -> Size<i32, Physical> {
        self.size.into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        Ok(vec![])
    }
    fn metadata(&self) -> Result<OutputMetadata, RenderError> {
        Ok(OutputMetadata {
            name: self.named.to_owned(),
            make: self.make.to_owned(),
            model: "a monitor".to_owned(),
            physical_size: self.millimetres,
            refresh: 60_000,
        })
    }
}

/// A server on a private socket.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// **Every presented display is described, and the two are told apart.**
///
/// They are told apart **by socket**, because a panel without a serial is a
/// kind of monitor rather than a monitor — this test's two screens share a
/// make and a model, which is what two identical displays on one desk look
/// like, and the first version of this code made them one identity.
#[test]
fn each_presented_display_is_described() {
    let (_directory, mut server) = server("described");
    let mut left = Screen {
        named: "DP-1",
        size: (1280, 720),
        make: "alo",
        millimetres: (310, 170),
    };
    let mut right = Screen {
        named: "HDMI-A-1",
        size: (1920, 1080),
        make: "alo",
        millimetres: (600, 340),
    };
    server.render(&mut left, 1).expect("the first draws");
    server.render(&mut right, 2).expect("the second draws");

    let reported = server.the_displays_as_reported();
    let sockets: Vec<String> = reported
        .iter()
        .map(|one| one.identity_on_its_own())
        .map(|identity| format!("{identity:?}"))
        .collect();

    assert_eq!(reported.len(), 2, "a display was not described");
    assert!(
        sockets.iter().any(|said| said.contains("DP-1"))
            && sockets.iter().any(|said| said.contains("HDMI-A-1")),
        "the descriptions do not name both displays: {sockets:?}"
    );
}

/// **A display with no physical size is described without one, not refused.**
///
/// `OutputMetadata` says `(0, 0)` means *unknown* and `Reported::of` refuses a
/// zero as a fault. The two vocabularies disagree about what a zero is, and
/// passing one on would lose the display entirely — a screen whose size cannot
/// be worked out is still a screen, and `Scale::worked_out_for` says what
/// happens then.
#[test]
fn a_display_that_reports_no_size_is_still_described() {
    let (_directory, mut server) = server("no-size");
    let mut unknown = Screen {
        named: "DP-1",
        size: (1280, 720),
        make: "alo",
        millimetres: (0, 0),
    };
    server.render(&mut unknown, 1).expect("it draws");

    assert_eq!(
        server.the_displays_as_reported().len(),
        1,
        "a display that could not say how big its glass is was dropped"
    );
}

/// **Nothing presented is nothing described**, rather than an invented screen.
#[test]
fn a_session_that_has_drawn_nothing_describes_nothing() {
    let (_directory, server) = server("nothing");
    assert!(server.the_displays_as_reported().is_empty());
    assert!(
        server.the_screens().is_none(),
        "a session with no displays was given an arrangement"
    );
}

/// **The arrangement is kept when one is offered, and absent when it is not.**
///
/// `None` is the honest state for a desktop that was never given a display
/// model, and every surface then lays out against the one viewport exactly as
/// it did before.
#[test]
fn an_arrangement_is_kept_only_when_a_desktop_offers_one() {
    let (_directory, mut server) = server("kept");
    assert!(server.the_screens().is_none());
    server.these_screens_are(None);
    assert!(
        server.the_screens().is_none(),
        "a desktop answering None left an arrangement behind"
    );
}

/// A screen of a given density, for asking what scale it is laid out at.
///
/// Built here rather than taken from `screens_testing` because both fixtures
/// there are about 165 dpi — a laptop panel and a 27-inch 4K — and two
/// displays that happen to agree cannot show that each was asked separately.
fn a_screen_of(socket: &str, pixels: (u32, u32), millimetres: (u32, u32)) -> Reported {
    Reported::of(
        Socket::named(socket).expect("a socket"),
        None,
        pixels,
        Some(millimetres),
    )
    .expect("a screen")
}

/// The arrangement these screens make, with nothing remembered about them.
fn arranged(reported: Vec<Reported>) -> crate::Screens {
    crate::screens_testing::the_screens(
        reported,
        &alo_displays::Changes::untouched(),
        &alo_appearance::Appearance::shipped(),
        &crate::screens_testing::a_cold_evening(),
    )
}

/// **Each display answers with its own scale, not the session's.**
///
/// `more-than-one-display-plan.md` task 5, and the standing rule it exists
/// for: *no figure reaches a display without passing through that display's
/// scale.* Two displays at different densities is the case that rule was
/// written for, and before this the shell had one number for all of them —
/// the literal `100` `alo-desktop` hands every frame.
///
/// A 24-inch 4K beside a 27-inch 1080p, because those genuinely differ: about
/// 185 dpi against about 82.
#[test]
fn each_display_answers_with_its_own_scale() {
    let (_directory, mut server) = server("own-scale");
    let dense = a_screen_of("DP-1", (3840, 2160), (527, 296));
    let sparse = a_screen_of("DP-2", (1920, 1080), (597, 336));
    server.these_screens_are(Some(arranged(vec![dense, sparse])));

    let on_the_dense = server.the_scale_of_display("DP-1");
    let on_the_sparse = server.the_scale_of_display("DP-2");

    assert_ne!(
        on_the_dense, on_the_sparse,
        "both displays were laid out at one scale"
    );
    assert!(
        on_the_dense > on_the_sparse,
        "the dense display was given the smaller scale: {on_the_dense} against {on_the_sparse}"
    );
    assert_eq!(
        on_the_sparse, 100,
        "a screen below one-to-one density was shrunk rather than left alone"
    );
}

/// **A session with no arrangement lays every display out one to one.**
///
/// Which is what every display got before this existed, so a session that was
/// never given a display model draws exactly as it did. The fallback is not a
/// guess dressed as an answer — a person who has arranged nothing has stated
/// no scale to honour.
#[test]
fn a_session_with_no_arrangement_is_one_to_one() {
    let (_directory, server) = server("no-arrangement");
    assert!(server.the_screens().is_none());
    assert_eq!(server.the_scale_of_display("eDP-1"), 100);
}

/// **A display the arrangement does not know is one to one**, rather than
/// borrowing the scale of a display that happens to be in it.
///
/// The failure this forbids is the quiet one: a monitor plugged in after the
/// arrangement was made, drawn at its neighbour's density.
#[test]
fn a_display_the_arrangement_does_not_know_is_one_to_one() {
    let (_directory, mut server) = server("unknown-display");
    let dense = a_screen_of("DP-1", (3840, 2160), (527, 296));
    server.these_screens_are(Some(arranged(vec![dense])));

    assert!(
        server.the_scale_of_display("DP-1") > 100,
        "the fixture is not dense enough for this test to mean anything"
    );
    assert_eq!(
        server.the_scale_of_display("HDMI-A-9"),
        100,
        "a display nobody arranged was given another display's scale"
    );
}
