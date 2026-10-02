//! **A restore travels; it does not relocate** — task 4 of the canvas plan.
//!
//! Driven through the real restore a person reaches — `set_window_minimized`
//! with `false` — rather than by calling the rule. A test that called
//! `the_view_travels_to` would pass just as well with nothing wired to it, which
//! is how task 6's rule sat unreachable for a fortnight.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with a pointer and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// A mapped frame, kept alive by the returned handle.
fn a_frame(f: &Fixture) -> (Application, WlSurface) {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    let surface = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("one frame is mapped");
    (app, surface)
}

/// The second Place.
fn elsewhere() -> alo_canvas::Place {
    alo_canvas::Place::FIRST
        .next()
        .expect("there is a second place")
}

/// **A window put aside on another Place comes back on that Place, and the view
/// travels there.**
///
/// Task 4's acceptance, both halves in one drive: the person ends up looking at
/// the Place the window was already on, and the window did not come to them.
#[test]
fn a_restore_travels_to_the_place_the_window_was_already_on() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);

    // Put it on another Place, then put it aside. Moving is task 3's primitive;
    // this test only needs the window to be somewhere the person is not.
    f.backend({
        let surface = surface.clone();
        move |s| s.move_the_window_to(&surface, elsewhere())
    });
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.set_window_minimized(&surface, true)
        })
        .expect("a mapped window can be put aside"),
        "the window was not put aside"
    );

    // Putting aside moves nobody: the person is still where they were.
    assert_eq!(
        f.backend(|s| s.the_place_now()),
        alo_canvas::Place::FIRST,
        "putting a window aside moved the person"
    );

    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.set_window_minimized(&surface, false)
        })
        .expect("a put-aside window can come back"),
        "the window did not come back"
    );

    assert_eq!(
        f.backend(|s| s.the_place_now()),
        elsewhere(),
        "a restore did not take the person to the Place the window was on"
    );
}

/// **Its Place is the same before and after, asserted directly.**
///
/// The plan asks for exactly this wording — *asserted directly rather than
/// inferred from where it appears* — because the destructive failure is a restore
/// that **writes** the window's Place to whichever one the person was looking at.
/// Reading the camera afterwards cannot tell that apart from travelling.
#[test]
fn a_restore_does_not_write_the_windows_place() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);

    f.backend({
        let surface = surface.clone();
        move |s| s.move_the_window_to(&surface, elsewhere())
    });
    let before = f.backend({
        let surface = surface.clone();
        move |s| s.the_place_a_window_lives_on(&surface)
    });
    assert_eq!(before, Some(elsewhere()));

    for minimized in [true, false] {
        f.backend({
            let surface = surface.clone();
            move |s| s.set_window_minimized(&surface, minimized).ok()
        });
    }

    let after = f.backend({
        let surface = surface.clone();
        move |s| s.the_place_a_window_lives_on(&surface)
    });
    assert_eq!(
        after, before,
        "a restore moved the window to the Place the person was looking at, \
         which is the one thing task 4 forbids"
    );
}

/// **A restore on the Place already being looked at moves nobody.**
///
/// The ordinary case, and it must cost nothing. A version that always travelled
/// would re-point the camera at the restored window every time, which is a view
/// moving for a reason the person did not ask for.
#[test]
fn a_restore_on_the_place_already_shown_does_not_move_the_person() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);
    let here = f.backend(|s| s.the_place_now());

    for minimized in [true, false] {
        f.backend({
            let surface = surface.clone();
            move |s| s.set_window_minimized(&surface, minimized).ok()
        });
    }

    assert_eq!(
        f.backend(|s| s.the_place_now()),
        here,
        "a restore on the Place already shown moved the person somewhere"
    );
}

/// **Putting a window aside never travels**, however far away it is.
///
/// Travelling is on the way back only. A version that travelled on the way out
/// would take a person to a window they had just chosen to put away.
#[test]
fn putting_a_window_aside_never_travels() {
    let f = fixture();
    let (_app, surface) = a_frame(&f);

    f.backend({
        let surface = surface.clone();
        move |s| s.move_the_window_to(&surface, elsewhere())
    });
    assert!(
        f.backend({
            let surface = surface.clone();
            move |s| s.set_window_minimized(&surface, true)
        })
        .expect("a mapped window can be put aside")
    );

    assert_eq!(
        f.backend(|s| s.the_place_now()),
        alo_canvas::Place::FIRST,
        "putting a window aside took the person to it"
    );
}
