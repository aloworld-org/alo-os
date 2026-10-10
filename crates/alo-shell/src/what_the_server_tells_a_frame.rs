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
/// `windows` is the server's own `window_areas()`, taken by the caller because
/// it has to outlive the frame that borrows it. `named` is this display's name,
/// which is what its scale is looked up by.
pub(crate) fn what_only_the_server_knows<'a>(
    frame: &mut DesktopFrame<'a>,
    server: &crate::Server,
    windows: &'a [Rectangle<i32, Physical>],
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
    // **How many the Dock holds**, which nothing set before 2026-10-10.
    //
    // `Holding::nothing()` is the honest argument and not a placeholder:
    // `alo_dock::Holding` is the pinned list, no file in this shell reads or
    // writes one, and `how_many_the_dock_holds` documents that every caller
    // passes it today. What it then counts is the applications with a window
    // open — a true count, and a Dock drawn from it is a Dock of what is
    // running, which is what a machine with no saved pins should show.
    frame.dock_holds = server.how_many_the_dock_holds(&alo_dock::Holding::nothing());
}

#[cfg(test)]
#[path = "what_the_server_tells_a_frame_tests.rs"]
mod tests;
