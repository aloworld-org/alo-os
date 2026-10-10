//! The frame fields only the server can answer, filled in one place.
//!
//! # Why a function rather than four lines at each site
//!
//! `crate::direct_desktop` builds a `DesktopFrame` twice — once for the display
//! the loop holds and once inside the loop over every other display — and both
//! have to fill the same fields from the same server. Four lines in two places
//! is eight lines that must agree, and `crate::nested_desktop` already says the
//! rule out loud about this very frame: *two of them in two files is how they
//! drift apart*.
//!
//! The Mac's framing when they found the Dock's field unset, 2026-10-10, is the
//! one worth keeping: **a one-line patch at each site would have been correct
//! code in the wrong shape**, and the wrong shape is the thing neither lane has
//! been catching by looking at behaviour.
//!
//! # What the division is
//!
//! `crate::direct_desktop`'s own comment states it, and it is the reason this
//! exists at all: *the desktop's own state has no server in it, so it cannot
//! say where the windows are; the server cannot say what the person chose about
//! the dock.* So the desktop answers what a person chose and the server answers
//! what is open, and a frame needs both.
//!
//! # The Dock's count, which was the field nobody set
//!
//! `dock_holds` was **absent from `direct_desktop` entirely** — `grep` found
//! zero mentions — while its three neighbours were each filled from the server.
//! `alo-desktop` passes `0`, so the Dock drew a bar sized for nothing however
//! many applications were open.
//!
//! **And what this does not fix.** The count becomes true in the process
//! `stand_the_desktop_up` runs, which is the same process `alo-desktop` is. What
//! remains is that **no unit starts that binary**: 8 `ExecStart=` lines under
//! `image/usr/lib/systemd/`, 0 of them naming it, and `alo-compositor` prints
//! *this process has nothing yet to draw in it* after sign-in and returns
//! success. So this removes a missing input and not the reason nothing draws.

use smithay::utils::{Physical, Rectangle};

use crate::nested_desktop::DesktopFrame;

/// Fill the fields of `frame` that only `server` can answer.
///
/// `windows` is the server's own `window_areas()` and `on_the_dock` is its
/// `what_the_dock_holds`, **both taken by the caller** because both have to
/// outlive the frame that borrows them — built here, they would die at this
/// function's end. `named` is this display's name, which is what its scale is
/// looked up by.
pub(crate) fn what_only_the_server_knows<'a>(
    frame: &mut DesktopFrame<'a>,
    server: &crate::Server,
    windows: &'a [Rectangle<i32, Physical>],
    on_the_dock: &'a [alo_dock::OnTheDock],
    the_overflows_heading: &'a str,
    named: &str,
) {
    frame.windows = windows;
    // A desktop's own state cannot know that a window has taken the whole
    // screen, and the Dock gives way to one that has.
    frame.filling_the_screen = server.a_window_is_filling_the_screen();
    // **This display's own scale**, which was the literal `100` that
    // `alo-desktop` hands every frame — one physical pixel per logical one,
    // true of this laptop and of nothing dense.
    // `more-than-one-display-plan.md` task 5, and the standing rule it exists
    // for: *no figure reaches a display without passing through that display's
    // scale.* The number is the person's, held in their arrangement;
    // `Server::the_scale_of_display` only looks it up.
    frame.display_scale = server.the_scale_of_display(named);
    // **What the Dock holds**, which nothing set before 2026-10-10 and which
    // was a count until the same day an application with no artwork gained a
    // letter to show.
    //
    // The caller builds it with `Holding::nothing()`, which is the honest
    // argument and not a placeholder: `alo_dock::Holding` is the pinned list,
    // no file in this shell reads or writes one, and `what_the_dock_holds`
    // documents that every caller passes it today. What it then lists is the
    // applications with a window open — true, and a Dock drawn from it is a
    // Dock of what is running, which is what a machine with no saved pins
    // should show.
    frame.on_the_dock = on_the_dock;
    // **Whether the overflow list is open**, which a desktop's own state cannot
    // know: it is turned over by a press, and presses reach the `Server`.
    frame.the_overflow_is_open = server.the_overflow_is_open();
    // **And what its heading says**, said by the lane rather than by the draw.
    // This is the lane's answer and not the server's, and it is set here for the
    // reason the other five are: `direct_desktop` builds a frame at two sites,
    // and a field set at each of them is a field that drifts between them.
    frame.the_overflows_heading = the_overflows_heading;
}

#[cfg(test)]
#[path = "what_the_server_tells_a_frame_tests.rs"]
mod tests;
