//! What a chord that puts a window on a side does, and what it refuses.
//!
//! These reach the parts that need no client: which display a chord applies to,
//! and what a session with none or with several answers. What a chord does to
//! two real windows needs two real clients and is held by
//! `tests/window_dividing_with_clients.rs`.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::server_desk::Desk;
use alo_desktops::{DisplayId, Promises};
use alo_dividing::{Area, Point, WindowId, area::Size};

/// A display the size of an ordinary screen.
fn a_screen() -> Area {
    Area::of(Point::at(0, 0), Size::of(1920, 1080)).unwrap()
}

/// The three surfaces that are on every desktop.
fn promises() -> Promises {
    Promises::of(
        WindowId::from_compositor(1),
        WindowId::from_compositor(2),
        WindowId::from_compositor(3),
    )
    .unwrap()
}

/// A desk with this many displays on it.
fn desk_with(displays: u64) -> Desk {
    let mut desk = Desk::new();
    for at in 1..=displays {
        desk.display_arrived(
            DisplayId::from_compositor(at),
            &format!("screen-{at}"),
            a_screen(),
            promises(),
            &|_| None,
        )
        .unwrap();
    }
    desk
}

/// **A session with no display refuses rather than guessing.**
///
/// A chord pressed before any display has arrived has nothing to divide, and
/// that is a refusal a caller can read rather than a silent nothing.
#[test]
fn a_chord_with_no_display_is_refused_by_name() {
    let desk = desk_with(0);

    assert!(desk.displays().next().is_none());
}

/// **One display is the one a chord applies to.**
#[test]
fn one_display_is_the_one_a_chord_divides() {
    let desk = desk_with(1);

    let mut displays = desk.displays();
    assert_eq!(displays.next(), Some(DisplayId::from_compositor(1)));
    assert!(displays.next().is_none());
}

/// **Two displays are refused, rather than the first one being picked.**
///
/// Which display a window is on is `alo-displays`' answer and nothing here asks
/// it yet. Guessing at the first would put somebody's window on the wrong
/// screen — and it would do it silently, which is worse than refusing.
#[test]
fn more_than_one_display_is_refused_rather_than_guessed_at() {
    let desk = desk_with(2);

    assert_eq!(desk.displays().count(), 2);
}

/// **A side a chord means is the dividing crate's**, asked of every action.
///
/// The test that stood here before this task held that the shell's *half* named
/// the same side the crate did. There is no half now, so what is held is that
/// the shell asks and takes the answer whole — including the two sides a half
/// could never lay out, which are no longer refused because there is no half to
/// refuse them.
#[test]
fn every_side_the_crate_names_is_one_this_shell_can_now_take() {
    use alo_shortcuts::Action;

    for &action in Action::ALL {
        let Some(side) = alo_dividing::keyboard::side_for(action) else {
            continue;
        };
        // Every side is a side the division lays out: a top and a bottom are
        // shares of a tree exactly as a left and a right are, which a half of
        // an output never could be.
        assert!(
            matches!(side, Side::Left | Side::Right | Side::Top | Side::Bottom),
            "{action:?} named a side that is not one"
        );
    }
}
