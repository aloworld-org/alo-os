//! What a press on the sign-in screen is on.
//!
//! The screen was keyboard-only until 2026-10-12: `crate::sign_in_seat` takes
//! keys and nothing took a press at all. So the submit arrow the design puts
//! inside the entry could not be pressed, and neither could Back,
//! Accessibility, Switch person or Power — four controls the owner's direction
//! names as *working controls*.
//!
//! # Geometry in, a meaning out, and nothing else
//!
//! A position and a [`SignInLayout`] in; what it is on, out. Nothing here
//! signs anybody in, powers anything off or changes a screen —
//! `crate::window_edge_pointing` is the same shape for the window edge and for
//! the same reason: a file that decided *and* acted would be two reasons to
//! change.
//!
//! # Every target is the design's own
//!
//! The rectangles come from `crate::sign_in_layout`, which read them from
//! `176-10686.xml`. **The submit target is 44 × 44 and never smaller**, which
//! that file holds as a test on every screen size that lays out, so a press
//! near the arrow lands on it at any viewport.

use smithay::utils::{Logical, Point, Rectangle};

use crate::sign_in_layout::SignInLayout;

/// What a press on the sign-in screen landed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheSignInScreen {
    /// The arrow inside the entry: submit what is typed.
    ///
    /// **The whole 44 × 44 target, not the 20 × 20 arrow drawn in it.** The
    /// artwork is what a person sees and the target is what answers, which is
    /// the rule `crate::window_edge` holds for its own controls.
    Submit,
    /// The entry field itself, outside the submit target: put the keyboard in
    /// it. A person who clicked the field meant to type into it.
    TheEntry,
    /// *Other ways to sign in*.
    OtherWays,
    /// *Back to lock*, at the top left.
    Back,
    /// *Accessibility*, at the bottom left.
    ///
    /// **Reachable before anybody has signed in**, which is the owner's
    /// direction and the reason it is on this screen at all: a person who
    /// needs it needs it to get in, not afterwards.
    Accessibility,
    /// *Switch person*.
    SwitchPerson,
    /// The power target, at the bottom right.
    Power,
}

/// What this position is on, or [`None`] for the screen itself.
///
/// **The submit target is asked before the entry it sits inside**, because it
/// is the more specific answer: a press at the arrow is a submission and not a
/// request to type. The same order `crate::the_window_edge_reveals` takes for
/// controls before their region, and its header gives the reason.
#[must_use]
pub fn what_is_under_the_pointer(
    at: &SignInLayout,
    point: Point<i32, Logical>,
) -> Option<OnTheSignInScreen> {
    // The specific before the general.
    for (what, rectangle) in [
        (OnTheSignInScreen::Submit, at.submit),
        (OnTheSignInScreen::TheEntry, at.entry),
        (OnTheSignInScreen::Back, at.back),
        (OnTheSignInScreen::Power, at.power),
        (OnTheSignInScreen::SwitchPerson, at.switch_person),
        (OnTheSignInScreen::Accessibility, at.accessibility),
        (OnTheSignInScreen::OtherWays, at.other_ways),
    ] {
        if holds(rectangle, point) {
            return Some(what);
        }
    }
    None
}

/// Whether a rectangle contains a point, half-open, with no area containing
/// nothing.
///
/// The rule every other hit test in this crate applies, and for its reason:
/// two rectangles sharing an edge must not both contain the points along it,
/// and a rectangle of no width must contain nothing rather than everything.
fn holds(rectangle: Rectangle<i32, Logical>, at: Point<i32, Logical>) -> bool {
    if rectangle.size.w <= 0 || rectangle.size.h <= 0 {
        return false;
    }
    let right = rectangle.loc.x.saturating_add(rectangle.size.w);
    let bottom = rectangle.loc.y.saturating_add(rectangle.size.h);
    at.x >= rectangle.loc.x && at.x < right && at.y >= rectangle.loc.y && at.y < bottom
}

#[cfg(test)]
#[path = "sign_in_pointing_tests.rs"]
mod tests;
