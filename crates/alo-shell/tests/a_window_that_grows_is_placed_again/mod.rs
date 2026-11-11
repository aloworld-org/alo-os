//! **A window alo placed, which then grows over another before anybody has
//! touched it, is placed again** — on real client connections, through the
//! production placing road.
//!
//! The walk machine's canvas layout recorded VSCodium at 457 × 157 and then, at
//! the same corner, at 1200 × 800: an application whose first frame is not the
//! size it settles at. A space found for the first frame is no promise about
//! the second, so `alo_shell`'s `a_window_that_grows_is_placed_again` asks again
//! when the size changes. Each case has its refusal beside it: a window the
//! person moved, and a window that grew over nothing.

#![expect(
    clippy::panic,
    reason = "in a test, a panic names the fixture that did not hold"
)]

use super::support::{Application, Fixture};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Physical, Rectangle, Size};

/// A view with room to spare, so the first placing has a choice.
const A_VIEW: (i32, i32) = (1280, 800);

/// The windows on the screen, front first.
fn front_to_back(f: &Fixture) -> Vec<WlSurface> {
    f.backend(|s| s.mapped_surfaces().cloned().collect())
}

/// A window of `app_id` drawn `size` big, mapped on a connection of its own,
/// and the compositor's handle on it.
fn mapped(f: &Fixture, app_id: &str, size: (i32, i32)) -> (Application, WlSurface) {
    let before = front_to_back(f);
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    assert!(
        app.set_a_viewport_destination(size.0, size.1),
        "the control: this compositor offers a viewport"
    );
    app.attach();
    app.sync();
    let new: Vec<WlSurface> = front_to_back(f)
        .into_iter()
        .filter(|surface| !before.contains(surface))
        .collect();
    match new.as_slice() {
        [one] => (app, one.clone()),
        other => panic!("expected one new window, found {}", other.len()),
    }
}

/// The application draws itself at a new size, as Electron does once it has
/// settled.
fn grows_to(app: &mut Application, size: (i32, i32)) {
    assert!(app.set_a_viewport_destination(size.0, size.1));
    app.sync();
}

/// Place every window that just opened or grew, as the frame loop does.
fn placed(f: &Fixture) -> usize {
    f.backend(move |s| {
        s.place_every_window_that_just_opened(
            "",
            Size::<i32, Physical>::from(A_VIEW),
            alo_strings::Direction::LeftToRight,
        )
    })
}

/// Where a window is on the plane.
fn where_it_is(f: &Fixture, surface: &WlSurface) -> Rectangle<i32, Logical> {
    let surface = surface.clone();
    f.backend(move |_| alo_shell::where_a_window_is(&surface))
}

/// Put a window somewhere, as the person's drag does.
fn moved_to(f: &Fixture, surface: &WlSurface, at: (i32, i32)) {
    let surface = surface.clone();
    assert!(
        f.backend(move |s| s.place_window(&surface, at).is_ok()),
        "the control: the window could be moved"
    );
}

/// What is drawn of a window at life size: the window and its title strip.
fn drawn(window: Rectangle<i32, Logical>) -> Rectangle<i32, Logical> {
    alo_shell::edge_of(
        window,
        alo_shell::Decorations::TheShellDraws,
        true,
        Size::from((1_000_000, 1_000_000)),
    )
    .strip
    .merge(window)
}

/// A desk with one window, the person having started `org.example.Codium`,
/// whose first frame is small: it is placed, and a second window is put just
/// to the right of where it landed — the neighbour it will grow over.
fn a_small_first_frame_beside_a_neighbour() -> (Fixture, Application, WlSurface, WlSurface, [Application; 2]) {
    let f = Fixture::keyboard();
    let (editor_app, editor) = mapped(&f, "org.example.Editor", (400, 300));
    assert_eq!(placed(&f), 1, "the control: the editor was placed");
    assert!(f.focus_surface(editor).is_ok());
    let not_started = f.backend(|s| {
        s.the_person_asked_to_start(
            alo_dock::AppId::named("org.example.Codium")
                .unwrap_or_else(|why| panic!("an application name: {why:?}")),
        );
        s.start_what_was_asked_for(&|_, _| true)
    });
    assert!(not_started.is_empty(), "the control: the start was sent");
    let (codium, theirs) = mapped(&f, "org.example.Codium", (457, 157));
    assert_eq!(placed(&f), 1, "the control: the small first frame was placed");
    let first = where_it_is(&f, &theirs);
    let (neighbour_app, neighbour) = mapped(&f, "org.example.Neighbour", (200, 200));
    moved_to(
        &f,
        &neighbour,
        (first.loc.x + first.size.w + 40, first.loc.y + 60),
    );
    (f, codium, theirs, neighbour, [editor_app, neighbour_app])
}

/// **The walk's case: a small first frame, then the real size over a
/// neighbour.** It is placed again, covering nothing that is drawn.
#[test]
fn a_window_that_grows_over_another_is_placed_again_covering_nothing() {
    let (f, mut codium, theirs, neighbour, _kept) = a_small_first_frame_beside_a_neighbour();
    let before = where_it_is(&f, &theirs);
    grows_to(&mut codium, (1200, 800));
    assert!(
        drawn(Rectangle::new(before.loc, (1200, 800).into()))
            .overlaps(drawn(where_it_is(&f, &neighbour))),
        "the control: at its old corner the grown window covers the neighbour"
    );
    assert_eq!(placed(&f), 1, "the grown window was not placed again");
    let now = drawn(where_it_is(&f, &theirs));
    for other in front_to_back(&f).iter().filter(|other| **other != theirs) {
        let other = drawn(where_it_is(&f, other));
        assert!(
            !now.overlaps(other),
            "the grown window {now:?} covers {other:?}"
        );
    }
    // **Once**: a later frame at the same size moves nothing.
    assert_eq!(placed(&f), 0, "a window was placed again with nothing changed");
}

/// **The refusal: a window the person moved is never moved by alo**, however
/// its application resizes it afterwards.
#[test]
fn a_window_the_person_moved_is_not_placed_again_when_it_grows() {
    let (f, mut codium, theirs, _neighbour, _kept) = a_small_first_frame_beside_a_neighbour();
    let before = where_it_is(&f, &theirs);
    moved_to(&f, &theirs, (before.loc.x, before.loc.y));
    grows_to(&mut codium, (1200, 800));
    assert_eq!(placed(&f), 0, "a window the person put somewhere was moved");
    assert_eq!(
        where_it_is(&f, &theirs).loc,
        before.loc,
        "a window the person put somewhere was moved"
    );
}

/// **The refusal: a window that grows over nothing stays where it is.**
#[test]
fn a_window_that_grows_over_nothing_stays_where_it_is() {
    let (f, mut codium, theirs, neighbour, _kept) = a_small_first_frame_beside_a_neighbour();
    let before = where_it_is(&f, &theirs);
    moved_to(&f, &neighbour, (before.loc.x + 5000, before.loc.y));
    grows_to(&mut codium, (600, 300));
    assert_eq!(placed(&f), 0, "a window that covers nothing was moved");
    assert_eq!(where_it_is(&f, &theirs).loc, before.loc);
    // And it is still alo's to answer for: grown again over the neighbour,
    // brought back beside it, it is placed again.
    let after = where_it_is(&f, &theirs);
    moved_to(
        &f,
        &neighbour,
        (after.loc.x + after.size.w + 40, after.loc.y + 60),
    );
    grows_to(&mut codium, (1200, 800));
    assert_eq!(
        placed(&f),
        1,
        "a window that grew twice was answered only the first time"
    );
}
