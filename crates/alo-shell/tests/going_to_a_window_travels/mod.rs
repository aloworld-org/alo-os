//! **Going to an application's window takes the view to it where none of it
//! is in sight** — `the-canvas-as-a-workspace.md` §11, *if the chosen window is
//! elsewhere: travel to its Place and location*.
//!
//! Seen on a booted machine on 2026-11-12: Firefox's opening glided the view
//! away from Dolphin, and Super+Tab then gave Dolphin the keyboard — Firefox
//! went inactive — while Dolphin stayed off the screen. Driven here through
//! `switch_window`, the road Alt+Tab takes, rather than by calling the rule.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_shell::WindowSwitchDirection as Direction;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Rectangle};

/// The viewport these tests render to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with a keyboard and an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// A mapped window, kept alive by the returned handle.
fn a_window(f: &Fixture) -> (Application, WlSurface) {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    let surface = f
        .backend(|s| s.mapped_surfaces().last().cloned())
        .expect("a window is mapped");
    (app, surface)
}

/// What the person sees, on the plane, at life size.
fn the_view(f: &Fixture) -> Rectangle<i32, Logical> {
    let at = f.backend(|s| s.the_camera().at());
    Rectangle::new((at.x, at.y).into(), VIEWPORT.into())
}

/// Where a window is on the plane.
fn where_it_is(f: &Fixture, surface: &WlSurface) -> Rectangle<i32, Logical> {
    let surface = surface.clone();
    f.backend(move |_| alo_shell::where_a_window_is(&surface))
}

/// Put a window somewhere, as a drag does.
fn put(f: &Fixture, surface: &WlSurface, at: (i32, i32)) {
    let surface = surface.clone();
    assert!(
        f.backend(move |s| s.place_window(&surface, at).is_ok()),
        "the control: the window could be put there"
    );
}

/// Look somewhere on the plane, as a pan does.
fn look_at(f: &Fixture, at: (i32, i32)) {
    assert!(
        f.backend(move |s| s.look_at_the_canvas(
            alo_canvas::At::checked(at.0, at.1).expect("a point on the plane")
        ))
        .is_some(),
        "the control: the view moved"
    );
}

/// **Alt+Tab to a window out of sight brings the view to it.**
#[test]
fn going_to_a_window_out_of_sight_brings_the_view_to_it() {
    let f = fixture();
    let (_dolphin_app, dolphin) = a_window(&f);
    let (_firefox_app, firefox) = a_window(&f);
    put(&f, &dolphin, (260, 107));
    put(&f, &firefox, (6000, 4000));
    assert!(f.focus_surface(firefox.clone()).is_ok());
    look_at(&f, (5990, 3990));
    assert!(
        !the_view(&f).overlaps(where_it_is(&f, &dolphin)),
        "the control: Dolphin is out of sight"
    );
    let went_to = f
        .backend(|s| s.switch_window(Direction::Forward))
        .expect("there is a window to go to");
    assert_eq!(went_to, dolphin, "the control: Alt+Tab went to Dolphin");
    assert!(
        the_view(&f).overlaps(where_it_is(&f, &dolphin)),
        "Dolphin has the keyboard and is still out of sight: {:?} {:?}",
        the_view(&f),
        where_it_is(&f, &dolphin)
    );
}

/// **And it arrives whole: its name band on the screen and its bottom above
/// the Dock.** Seen on a booted machine on 2026-11-13: Super+Tab brought
/// Dolphin into sight with its band off the top (`w26-37`), because the view
/// stopped with the window's own corner at the screen's. The test above passed
/// throughout: it asked only that some of the window was in sight.
#[test]
fn going_to_a_window_out_of_sight_shows_its_band_above_the_dock() {
    const DOCK: i32 = 64;
    let f = fixture();
    f.backend(|s| {
        s.the_fixed_controls_were_drawn(
            "the-one-display",
            alo_shell::FixedControlsDrawn {
                dock_band: Some(Rectangle::new(
                    (0, VIEWPORT.1 - DOCK).into(),
                    (VIEWPORT.0, DOCK).into(),
                )),
                panel_reserved: Rectangle::default(),
                what_is_leaving: None,
                top_controls: None,
            },
            alo_appearance::TextScale::ordinary(),
        );
    });
    let (_dolphin_app, dolphin) = a_window(&f);
    let (_firefox_app, firefox) = a_window(&f);
    put(&f, &dolphin, (260, 107));
    put(&f, &firefox, (6000, 4000));
    assert!(f.focus_surface(firefox.clone()).is_ok());
    look_at(&f, (5990, 3990));
    let went_to = f
        .backend(|s| s.switch_window(Direction::Forward))
        .expect("there is a window to go to");
    assert_eq!(went_to, dolphin, "the control: Alt+Tab went to Dolphin");
    let view = the_view(&f);
    let window = where_it_is(&f, &dolphin);
    let band_top = window.loc.y - alo_shell::THE_STRIP_IS_TALL;
    assert!(
        band_top >= view.loc.y,
        "Dolphin's band is off the top: band at {band_top}, view from {}",
        view.loc.y
    );
    assert!(
        window.loc.y + window.size.h <= view.loc.y + VIEWPORT.1 - DOCK,
        "Dolphin's bottom is under the Dock: {window:?} in {view:?}"
    );
    assert!(
        window.loc.x >= view.loc.x,
        "Dolphin's left edge is off the screen"
    );
}

/// **The refusal: a window any part of which is in sight is gone to where the
/// person sees it**, and the view does not move.
#[test]
fn going_to_a_window_in_sight_moves_nothing() {
    let f = fixture();
    let (_one_app, one) = a_window(&f);
    let (_other_app, other) = a_window(&f);
    put(&f, &one, (100, 100));
    put(&f, &other, (400, 300));
    assert!(f.focus_surface(other.clone()).is_ok());
    let before = the_view(&f);
    let went_to = f
        .backend(|s| s.switch_window(Direction::Forward))
        .expect("there is a window to go to");
    assert_eq!(
        went_to, one,
        "the control: Alt+Tab went to the other window"
    );
    assert_eq!(the_view(&f), before, "the view moved to a window in sight");
}
