//! The glide: a duration, eased, ended by the person's hand, and a jump where
//! they asked for less motion.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;
use std::time::{Duration, Instant};

use alo_canvas::At;

use super::{THE_GLIDE_LASTS, eased};
use crate::Server;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

fn at(x: i32, y: i32) -> At {
    At::checked(x, y).expect("a point on the plane")
}

/// **A duration, never a speed**: a short way and a long way both end at the
/// same moment, and neither has ended a moment before.
#[test]
fn a_glide_lasts_the_same_however_far_it_goes() {
    for (far, named) in [
        (at(40, 0), "a-short-glide"),
        (at(90_000, -60_000), "a-long-glide"),
    ] {
        let (_directory, mut server) = server(named);
        let began = Instant::now();
        assert!(
            server.the_view_glides_to(far, began),
            "the control: it moves"
        );
        assert!(server.the_view_is_gliding());
        assert_eq!(
            server.the_camera().at(),
            At::origin(),
            "it jumped before the first frame"
        );
        assert!(
            server.the_glide_moves_on(began + THE_GLIDE_LASTS / 2),
            "it ended before its time"
        );
        assert_ne!(
            server.the_camera().at(),
            far,
            "it arrived halfway through its time"
        );
        assert!(
            server.the_glide_moves_on(began + THE_GLIDE_LASTS - Duration::from_millis(1)),
            "it ended before its time"
        );
        assert!(
            !server.the_glide_moves_on(began + THE_GLIDE_LASTS),
            "it is still moving after its time"
        );
        assert_eq!(server.the_camera().at(), far, "it did not arrive exactly");
        assert!(!server.the_view_is_gliding());
    }
}

/// **It moves through the points between**, gently at the start and at the
/// end: a tenth of the time covers less than a tenth of the way, and half
/// the time covers half.
#[test]
fn a_glide_passes_between_and_eases_in_and_out() {
    let (_directory, mut server) = server("eased");
    let began = Instant::now();
    let to = at(1000, 0);
    assert!(server.the_view_glides_to(to, began));
    assert!(server.the_glide_moves_on(began + THE_GLIDE_LASTS / 10));
    let early = server.the_camera().at().x;
    assert!(
        early > 0 && early < 100,
        "a tenth of the time went {early} of 1000"
    );
    assert!(server.the_glide_moves_on(began + THE_GLIDE_LASTS / 2));
    assert_eq!(
        server.the_camera().at().x,
        500,
        "half the time is not half the way"
    );
    assert!(server.the_glide_moves_on(began + THE_GLIDE_LASTS * 9 / 10));
    let late = server.the_camera().at().x;
    assert!(
        late > 900 && late < 1000,
        "nine tenths of the time went {late}"
    );
    assert_eq!(
        server.the_camera().zoom(),
        alo_canvas::Zoom::LIFE_SIZE,
        "the view zoomed"
    );
    assert!((eased(0.0)).abs() < f64::EPSILON && (eased(1.0) - 1.0).abs() < f64::EPSILON);
}

/// **With less motion asked for, the view jumps**: it is there at once, and
/// nothing is left gliding.
#[test]
fn with_reduced_motion_the_view_jumps() {
    let (_directory, mut server) = server("reduced-motion");
    server.the_motion_is_reduced(true);
    let to = at(2400, 300);
    assert!(server.the_view_glides_to(to, Instant::now()));
    assert_eq!(server.the_camera().at(), to, "it did not jump");
    assert!(
        !server.the_view_is_gliding(),
        "it glides with less motion asked for"
    );
}

/// **Less motion asked for mid-glide** finishes the glide at once rather than
/// moving on for the rest of it.
#[test]
fn turning_reduced_motion_on_mid_glide_finishes_it() {
    let (_directory, mut server) = server("reduced-mid-glide");
    let to = at(-800, 200);
    assert!(server.the_view_glides_to(to, Instant::now()));
    assert!(server.the_view_is_gliding(), "the control: it is gliding");
    server.the_motion_is_reduced(true);
    assert_eq!(server.the_camera().at(), to);
    assert!(!server.the_view_is_gliding());
}

/// **The person's hand wins**: a pan mid-glide ends it where the person put
/// the view, and a later frame does not drag the view back.
#[test]
fn a_pan_mid_glide_ends_it() {
    let (_directory, mut server) = server("a-pan-ends-it");
    let began = Instant::now();
    assert!(server.the_view_glides_to(at(5000, 0), began));
    assert!(server.the_glide_moves_on(began + THE_GLIDE_LASTS / 2));
    assert!(server.pan_the_canvas(-10, 0).is_some());
    let theirs = server.the_camera().at();
    assert!(!server.the_view_is_gliding(), "the glide fought the pan");
    assert!(!server.the_glide_moves_on(began + THE_GLIDE_LASTS));
    assert_eq!(
        server.the_camera().at(),
        theirs,
        "a later frame moved the view the person put"
    );
}

/// **Going to another Place ends it**: the point it was going to is on the
/// Place the person left.
#[test]
fn going_to_another_place_ends_it() {
    let (_directory, mut server) = server("another-place");
    let began = Instant::now();
    assert!(server.the_view_glides_to(at(3000, 0), began));
    let next = server
        .the_place_now()
        .next()
        .expect("a Place after this one");
    server.surfaces.place = next;
    assert!(!server.the_glide_moves_on(began + THE_GLIDE_LASTS / 2));
    assert!(!server.the_view_is_gliding());
    assert_eq!(
        server.the_camera().at(),
        At::origin(),
        "it moved the view on another Place"
    );
}

/// **Nowhere to go is no glide**, and a window opened mid-glide is placed
/// against where the view is going.
#[test]
fn a_glide_to_where_the_view_is_does_nothing_and_the_destination_is_known() {
    let (_directory, mut server) = server("nowhere-to-go");
    assert!(!server.the_view_glides_to(At::origin(), Instant::now()));
    assert!(!server.the_view_is_gliding());
    let to = at(700, -300);
    assert!(server.the_view_glides_to(to, Instant::now()));
    assert_eq!(server.the_camera_after_the_glide().at(), to);
    assert_eq!(
        server.the_camera().at(),
        At::origin(),
        "the control: it has not moved yet"
    );
}
