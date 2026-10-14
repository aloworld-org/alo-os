//! **Which application `Super`+`Tab` moves to**, over every list that matters.
//!
//! The arithmetic only — no display, no seat, no surfaces. Which list it is
//! given is the compositor's half and is the thin part; this is the half with
//! the cases in it.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dock::window::AppId;

use super::the_next_application;
use crate::window_switch::WindowSwitchDirection::{Backward, Forward};

/// An application, named the way a real one is.
fn app(name: &str) -> AppId {
    AppId::named(name).expect("a named application")
}

/// Three applications in a stable order.
fn three() -> Vec<AppId> {
    vec![app("org.one"), app("org.two"), app("org.three")]
}

#[test]
fn forward_from_the_first_is_the_second() {
    let order = three();
    assert_eq!(
        the_next_application(&order, Some(&app("org.one")), Forward),
        Some(app("org.two"))
    );
}

#[test]
fn backward_from_the_second_is_the_first() {
    let order = three();
    assert_eq!(
        the_next_application(&order, Some(&app("org.two")), Backward),
        Some(app("org.one"))
    );
}

#[test]
fn forward_from_the_last_wraps_to_the_first() {
    let order = three();
    assert_eq!(
        the_next_application(&order, Some(&app("org.three")), Forward),
        Some(app("org.one"))
    );
}

/// **The one place this arithmetic could have gone wrong in a way a test is the
/// only way to see.** Backward from the first is a subtraction that would go
/// below zero, which panics in debug and wraps to a vast number in release — so
/// it is written as `+ len - 1` and this is the assertion that holds it there.
#[test]
fn backward_from_the_first_wraps_to_the_last() {
    let order = three();
    assert_eq!(
        the_next_application(&order, Some(&app("org.one")), Backward),
        Some(app("org.three"))
    );
}

/// **One application is nowhere to go**, and that is a refusal rather than a
/// wrap onto itself: focusing what is already focused is a keypress that looks
/// like it did nothing while having done exactly what it said.
#[test]
fn one_application_is_nowhere_to_go() {
    let order = vec![app("org.only")];
    assert_eq!(
        the_next_application(&order, Some(&app("org.only")), Forward),
        None
    );
    assert_eq!(
        the_next_application(&order, Some(&app("org.only")), Backward),
        None
    );
}

#[test]
fn an_empty_list_is_nowhere_to_go_from() {
    assert_eq!(the_next_application(&[], None, Forward), None);
    assert_eq!(
        the_next_application(&[], Some(&app("org.gone")), Forward),
        None
    );
}

/// **A focused application not in the list starts at an end**, which is a real
/// state and not a broken one: the focused window may be a surface the Dock
/// does not count at all.
#[test]
fn a_focused_application_that_is_not_in_the_list_starts_at_an_end() {
    let order = three();
    assert_eq!(
        the_next_application(&order, Some(&app("org.elsewhere")), Forward),
        Some(app("org.one")),
        "forward from outside the list should start at the beginning"
    );
    assert_eq!(
        the_next_application(&order, Some(&app("org.elsewhere")), Backward),
        Some(app("org.three")),
        "backward from outside the list should start at the end"
    );
}

#[test]
fn nothing_focused_starts_at_an_end() {
    let order = three();
    assert_eq!(
        the_next_application(&order, None, Forward),
        Some(app("org.one"))
    );
    assert_eq!(
        the_next_application(&order, None, Backward),
        Some(app("org.three"))
    );
}

/// **Every application is reached, and each exactly once, before it comes
/// round.** Three single steps would each pass with a function that always
/// answered *the second one*; walking the whole ring is what catches that.
#[test]
fn walking_forward_reaches_every_application_once_and_comes_back() {
    let order = three();
    let mut at = app("org.one");
    let mut seen = vec![at.clone()];
    for _ in 1..order.len() {
        at = the_next_application(&order, Some(&at), Forward).expect("three is somewhere to go");
        assert!(
            !seen.contains(&at),
            "the walk repeated {at:?} before the ring closed"
        );
        seen.push(at.clone());
    }
    assert_eq!(
        seen.len(),
        order.len(),
        "the walk did not reach every application"
    );
    assert_eq!(
        the_next_application(&order, Some(&at), Forward),
        Some(app("org.one")),
        "the ring did not close"
    );
}
