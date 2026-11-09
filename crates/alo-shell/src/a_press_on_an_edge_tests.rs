//! What a press on a band reaches: the double-click rule, held at its edges.

use super::*;

/// **Two presses on one band, soon and close, are a double-click.**
#[test]
fn two_presses_soon_and_close_on_one_band_are_a_double_click() {
    assert!(is_a_double_click(true, Duration::from_millis(200), (1, -2)));
    assert!(is_a_double_click(true, A_DOUBLE_CLICK_IS_WITHIN, (4, 4)));
}

/// **Too slow is two clicks**, and the second starts a drag like any other.
#[test]
fn a_second_press_too_late_is_not_a_double_click() {
    assert!(!is_a_double_click(
        true,
        A_DOUBLE_CLICK_IS_WITHIN + Duration::from_millis(1),
        (0, 0)
    ));
}

/// **Too far apart is two clicks**, so a press, a drag and a press elsewhere
/// never maximises by surprise.
#[test]
fn a_second_press_too_far_away_is_not_a_double_click() {
    assert!(!is_a_double_click(true, Duration::from_millis(100), (5, 0)));
    assert!(!is_a_double_click(
        true,
        Duration::from_millis(100),
        (0, -5)
    ));
}

/// **Two presses on two windows' bands are never one double-click.**
#[test]
fn presses_on_two_windows_are_never_one_double_click() {
    assert!(!is_a_double_click(
        false,
        Duration::from_millis(100),
        (0, 0)
    ));
}
