//! What a session keeps about its displays, across the things that happen to
//! them.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use alo_dividing::{Point, Size, WindowId};

/// A display the size of an ordinary screen.
fn a_screen() -> Area {
    Area::of(Point::at(0, 0), Size::of(1920, 1080)).unwrap()
}

/// The three surfaces that are on every desktop, as this crate's own.
fn promises() -> Promises {
    Promises::of(
        WindowId::from_compositor(1),
        WindowId::from_compositor(2),
        WindowId::from_compositor(3),
    )
    .unwrap()
}

/// The one display these tests use.
fn a_display() -> DisplayId {
    DisplayId::from_compositor(1)
}

/// An application by name.
fn held_by(named: &str) -> HeldBy {
    HeldBy::named(named).unwrap()
}

/// A window an application holds, at a number.
fn a_window(id: u64) -> alo_dividing::Window {
    alo_dividing::Window::any_size(WindowId::from_compositor(id))
}

/// A display divided between two windows, **by `alo-dividing`**.
///
/// Through the crate's own road — a chord dividing the focused window's share
/// with the next one — because a division this crate assembled would be this
/// crate deciding a layout, which is the thing the constraint forbids.
fn divided_between(first: u64, second: u64) -> Division {
    let mut division = Division::of(a_screen());
    division
        .divide_with_next(
            a_window(first),
            Some(a_window(second)),
            alo_dividing::Side::Left,
        )
        .unwrap();
    division
}

/// **A session with no display holds nothing, which is a real state.**
///
/// It is what a machine is between the sign-in screen ending and the first
/// frame, and asking it about a display it does not have is a refusal rather
/// than an empty answer.
#[test]
fn a_session_with_no_display_holds_nothing() {
    let desk = Desk::new();

    assert!(desk.is_empty());
    assert!(desk.dividing(a_display()).is_none());
    assert!(desk.desktops(a_display()).is_none());
}

/// **A display that arrives has desktops, and one nothing divided has no
/// division.**
///
/// An undivided display is not a display with an empty tree: nothing has
/// divided it, and that is what `None` says.
#[test]
fn a_display_that_arrives_has_desktops_and_no_division() {
    let mut desk = Desk::new();

    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| None)
        .unwrap();

    assert!(!desk.is_empty());
    assert!(desk.desktops(a_display()).is_some());
    assert!(
        desk.dividing(a_display()).is_none(),
        "a display nobody divided came back divided"
    );
}

/// **Plugging the same display in twice is refused.**
///
/// The second call would take the first's desktops away, and a person's windows
/// with them. Refused where it happens rather than smoothed over.
#[test]
fn the_same_display_arriving_twice_is_refused() {
    let mut desk = Desk::new();
    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| None)
        .unwrap();

    let refused = desk
        .display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| None)
        .unwrap_err();

    assert!(matches!(refused, NotADisplay::AlreadyThere(_)));
}

/// **A display that leaves is remembered, and the same screen returning finds
/// what it left.**
///
/// The whole point of remembering: a person who unplugs a screen at a desk and
/// plugs it in again tomorrow gets their arrangement, not an empty display.
#[test]
fn a_screen_that_returns_finds_the_division_it_left() {
    let mut desk = Desk::new();
    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| None)
        .unwrap();

    desk.divide(a_display(), divided_between(10, 11));
    assert!(desk.dividing(a_display()).is_some());

    desk.display_left(a_display(), "screen-1", &|_| {
        Some(held_by("com.example.mail"))
    })
    .unwrap();
    assert!(desk.is_empty());

    // The same screen comes back, with that application still running.
    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|held| {
        (held == &held_by("com.example.mail")).then(|| a_window(77))
    })
    .unwrap();

    assert!(
        desk.dividing(a_display()).is_some(),
        "the screen came back without the division it left"
    );
}

/// **It comes back under the window numbers that are open now.**
///
/// A window number is this session's and not yesterday's. The division returns
/// holding the numbers the windows have today, which is what
/// `alo_dividing::Remembered::restored` is for — and a division holding
/// yesterday's numbers would point at windows that do not exist.
#[test]
fn the_division_comes_back_under_todays_window_numbers() {
    let mut desk = Desk::new();
    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| None)
        .unwrap();
    desk.divide(a_display(), divided_between(10, 11));
    desk.display_left(a_display(), "screen-1", &|_| {
        Some(held_by("com.example.mail"))
    })
    .unwrap();

    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| {
        Some(a_window(4242))
    })
    .unwrap();

    let back = desk.dividing(a_display()).unwrap();
    let numbers: Vec<u64> = back
        .shares()
        .into_iter()
        .map(|share| share.window().to_compositor())
        .collect();
    assert!(
        numbers.contains(&4242) && !numbers.contains(&10),
        "the division came back under yesterday's window numbers: {numbers:?}"
    );
}

/// **A screen returning with nothing it remembered still open is undivided.**
///
/// Not an empty tree and not yesterday's arrangement over windows that are
/// gone: nothing of what it remembered is open, so nothing divides it.
#[test]
fn a_screen_whose_applications_are_all_closed_comes_back_undivided() {
    let mut desk = Desk::new();
    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| None)
        .unwrap();
    desk.divide(a_display(), divided_between(10, 11));
    desk.display_left(a_display(), "screen-1", &|_| {
        Some(held_by("com.example.mail"))
    })
    .unwrap();

    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| None)
        .unwrap();

    assert!(
        desk.dividing(a_display()).is_none(),
        "a screen came back divided over windows that are not open"
    );
}

/// **A different screen does not get this one's arrangement.**
///
/// Divisions are remembered per display, keyed by the screen. Plugging in a
/// different monitor must not lay somebody's laptop arrangement over it.
#[test]
fn another_screen_does_not_inherit_this_ones_division() {
    let mut desk = Desk::new();
    desk.display_arrived(a_display(), "screen-1", a_screen(), promises(), &|_| None)
        .unwrap();
    desk.divide(a_display(), divided_between(10, 11));
    desk.display_left(a_display(), "screen-1", &|_| {
        Some(held_by("com.example.mail"))
    })
    .unwrap();

    desk.display_arrived(a_display(), "screen-2", a_screen(), promises(), &|_| {
        Some(a_window(77))
    })
    .unwrap();

    assert!(
        desk.dividing(a_display()).is_none(),
        "a different screen was given another screen's division"
    );
}

/// **A display that was never here cannot be switched or unplugged.**
#[test]
fn a_display_that_is_not_here_is_refused_rather_than_answered() {
    let mut desk = Desk::new();

    assert!(matches!(
        desk.display_left(a_display(), "screen-1", &|_| None)
            .unwrap_err(),
        NotADisplay::Unknown(_)
    ));
    assert!(matches!(
        desk.switch(a_display(), Switch::Next).unwrap_err(),
        NotADisplay::Unknown(_)
    ));
}
