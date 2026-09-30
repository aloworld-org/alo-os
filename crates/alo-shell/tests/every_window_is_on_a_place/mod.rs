//! **Every mapped window answers which Place it is on** — task 1 of
//! `docs/autonomy/the-canvas-and-its-places.md`.
//!
//! The canvas half of this task is arithmetic and is tested where the arithmetic
//! is, in `alo-canvas`. This is the half that could not be tested there: that a
//! real window, from a real client, through the real XDG handshake, has a Place —
//! and that *Show all* fits the Place being looked at rather than every window
//! this compositor happens to be holding.
//!
//! # Why a window's Place survives an unmap and its position does not
//!
//! `crate::window_placement` resets a placement when a window unmaps, because a
//! position belongs to one mapping lifetime. A Place does not reset, and the
//! difference is not a detail: a window that unmaps and maps again — the XDG
//! fresh-handshake path — is the same window, and putting it back on whichever
//! surface the person happens to be looking at would move somebody's window while
//! their back was turned. There is a test below for exactly that, because the two
//! rules live in neighbouring files and the wrong one is easy to copy.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with a pointer and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// Complete the real XDG handshake and attach a buffer. **Kept alive**: dropping
/// an `Application` disconnects its client, which unmaps the frame.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// **A machine that has never been used is looking at its first Place.**
///
/// Asserted rather than assumed, because every other test here reads this value
/// and a different starting Place would make all of them pass for the wrong
/// reason.
#[test]
fn a_fresh_compositor_is_looking_at_the_first_place() {
    let f = fixture();
    let place = f.backend(|s| s.the_place_now());
    assert_eq!(place, alo_canvas::Place::FIRST);
}

/// **Every mapped window answers which Place it is on.**
///
/// The acceptance sentence of task 1, at the level a person meets it: a real
/// client's window, and the compositor has an answer to *which surface is this
/// on* that it did not have before.
#[test]
fn a_mapped_window_answers_which_place_it_is_on() {
    let f = fixture();
    let _app = mapped(&f);
    let (place, looking) = f.backend(|s| {
        let surface = s.mapped_surfaces().last().cloned();
        let surface = surface.expect("one frame is mapped");
        (s.the_place_of_the_window(&surface), s.the_place_now())
    });
    assert_eq!(
        place,
        Some(looking),
        "a window opened on the Place the person was looking at"
    );
}

/// **Every frame on the plane carries its Place, not just the surface behind it.**
///
/// `the_frames_on_the_plane` is what *Show all*, the drag extent and the
/// arrangement are all computed from. A Place the compositor knows but that never
/// reaches a `Frame` would leave every one of those still asking the old
/// ambiguous question.
#[test]
fn the_frames_on_the_plane_carry_the_place_they_are_on() {
    let f = fixture();
    let _first = mapped(&f);
    let _second = mapped(&f);
    let (frames, looking) = f.backend(|s| (s.the_frames_on_the_plane(), s.the_place_now()));
    assert_eq!(frames.len(), 2, "two frames are mapped");
    for frame in &frames {
        assert_eq!(frame.place(), looking);
    }
}

/// **A window put on another Place is not fitted by *Show all* here.**
///
/// The behaviour the whole task exists for, and the one that fails silently
/// without it: *Show all* would zoom out far enough to include a window on a
/// surface the person is not looking at, and the window they asked to see would
/// become a speck. Nothing refuses — the arithmetic is the same arithmetic — so
/// this is asserted through the extent rather than through an error.
#[test]
fn show_all_fits_this_place_and_not_a_window_moved_off_it() {
    let f = fixture();
    let _here = mapped(&f);
    let _leaving = mapped(&f);

    // Drag the second frame far from the first, so that an extent holding both is
    // measurably wider than one holding the first alone.
    let moved = f.backend(|s| {
        let surface = s.mapped_surfaces().last().cloned()?;
        s.place_window(&surface, (9_000, 6_000)).ok()?;
        Some(surface)
    });
    let moved = moved.expect("the second frame could be placed");

    let both = f
        .backend(|s| {
            let frames = s.the_frames_on_the_plane();
            alo_canvas::plane::reached_by(&frames, s.the_place_now())
        })
        .expect("two frames reach somewhere");

    // Now put it on another Place. Nothing about its rectangle changes.
    let elsewhere = alo_canvas::Place::FIRST
        .next()
        .expect("there is a second place");
    f.backend(move |s| s.move_the_window_to(&moved, elsewhere));

    let after = f
        .backend(|s| {
            let frames = s.the_frames_on_the_plane();
            alo_canvas::plane::reached_by(&frames, s.the_place_now())
        })
        .expect("one frame still reaches somewhere");

    assert!(
        after.to.x < both.to.x || after.to.y < both.to.y,
        "a window on another Place was still being fitted by Show all: \
         {both:?} before, {after:?} after"
    );
}

/// **A window keeps its Place across an unmap and a remap.**
///
/// Its *position* does not, deliberately — `crate::window_placement::reset` is
/// called on unmap because a position belongs to one mapping lifetime. Which
/// surface a window lives on is the opposite kind of fact, and a window that came
/// back on whichever Place the person was looking at would be somebody's window
/// moved while their back was turned.
#[test]
fn an_unmap_and_a_remap_leave_a_window_on_the_place_it_was_on() {
    let f = fixture();
    let mut app = mapped(&f);
    let surface = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("one frame is mapped");
    let before = {
        let surface = surface.clone();
        f.backend(move |s| s.the_place_of_the_window(&surface))
    };
    assert_eq!(before, Some(alo_canvas::Place::FIRST));

    // The unmap and fresh XDG handshake `window_close`'s own test uses.
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    app.configure();
    app.attach();
    app.sync();

    let after = f.backend(move |s| s.the_place_of_the_window(&surface));
    assert_eq!(after, before, "a remap moved a window to another Place");
}
