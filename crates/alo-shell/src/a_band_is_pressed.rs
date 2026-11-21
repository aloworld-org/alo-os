//! **What a press on any window's band does: one road for every window.**
//!
//! Being written: see `a_band_is_pressed_tests.rs`, which holds every kind of
//! window to the rule before this file carries it.

#[cfg(all(test, target_os = "linux"))]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_band_is_pressed_tests.rs"]
mod tests;
