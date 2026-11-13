//! **A wheel over the side column moves it, within its ends, and a wheel over
//! a column that fits goes where it went before** — and the refusal paths: a
//! shut Settings, a nonsense wheel, a wheel at either end.

use smithay::backend::input::{Axis, AxisSource};
use smithay::input::pointer::AxisFrame;

use super::*;

/// A wheel turned this far, positive downwards.
fn a_scroll_of(down: f64) -> AxisFrame {
    AxisFrame::new(0)
        .source(AxisSource::Wheel)
        .value(Axis::Vertical, down)
}

/// **A notch is one category**, down for a wheel turned toward the person,
/// and nonsense is nothing.
#[test]
fn a_notch_is_one_category_and_nonsense_is_nothing() {
    assert_eq!(how_far_down(&a_scroll_of(15.0), 50), 50);
    assert_eq!(how_far_down(&a_scroll_of(-30.0), 50), -100);
    let mut nonsense = a_scroll_of(15.0);
    nonsense.axis.1 = f64::NAN;
    assert_eq!(how_far_down(&nonsense, 50), 0);
    nonsense.axis.1 = f64::INFINITY;
    assert_eq!(how_far_down(&nonsense, 50), 0);
}

/// **A wheel moves it within its ends, says whether it moved, and stops it
/// following the keyboard.**
#[test]
fn a_wheel_moves_it_within_its_ends() {
    let mut scroll = TheSideColumnsScroll::at_the_top();
    assert!(scroll.follows);
    // At the top, up moves nothing.
    assert!(!scroll.wheeled(-50, 200));
    assert!(!scroll.follows);
    assert!(scroll.wheeled(150, 200));
    assert_eq!(scroll.scrolled, 150);
    assert!(scroll.wheeled(150, 200));
    assert_eq!(scroll.scrolled, 200);
    // At the end, down moves nothing.
    assert!(!scroll.wheeled(50, 200));
    // A column that fits never moves.
    let mut fits = TheSideColumnsScroll::at_the_top();
    assert!(!fits.wheeled(50, 0));
    assert_eq!(fits.scrolled, 0);
}

/// **A shut Settings has no column to move.**
#[test]
fn a_shut_settings_is_not_scrolled() {
    let mut window = SettingsWindow::closed();
    assert!(!window.the_side_column_wheeled(50, 200));
    window.the_side_column_was_drawn_at(80);
    assert!(window.open().is_none());
}
