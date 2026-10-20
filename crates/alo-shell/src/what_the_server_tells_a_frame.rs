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

/// What the lane worked out before the frame, which the frame then borrows.
///
/// **Taken by the caller, every one of them**, because each has to outlive the
/// frame that borrows it — built inside
/// [`what_only_the_server_knows`] they would die at that function's end. The
/// lane holds them on their own lines for exactly that reason and has done
/// since `windows` was the only one.
///
/// # One bundle rather than six arguments
///
/// It was six, and clippy's argument count is what said so. The reason is
/// `crate::the_docks_words::TheDocksWords`' own and it is the better one: six
/// borrowed slices in a row at two call sites is twelve chances to pass the
/// wrong one of two that have the same type, and **`on_the_dock` and
/// `in_more_apps` are both `&[OnTheDock]`**. Swapped, the Dock would draw the
/// overview and *More apps* would draw the Dock — a frame that renders without
/// complaint and is wrong in a way only somebody with a minimised-only
/// application would ever see.
///
/// A struct cannot be passed in the wrong order.
pub(crate) struct WhatTheLaneBuilt<'a> {
    /// Where each window is, from `Server::window_areas`.
    pub(crate) windows: &'a [Rectangle<i32, Physical>],
    /// Who is on the Dock, from `Server::what_the_dock_holds`.
    pub(crate) on_the_dock: &'a [alo_dock::OnTheDock],
    /// What *More apps* lists, from `Server::what_more_apps_holds`.
    pub(crate) in_more_apps: &'a [alo_dock::OnTheDock],
    /// Which of the Dock's surfaces is open, from
    /// `Server::what_the_dock_has_open`.
    pub(crate) the_dock_has_open: &'a crate::WhatTheDockHasOpen,
    /// What the Dock says, already said in this person's language.
    pub(crate) the_docks_words: &'a crate::the_docks_words::TheDocksWords,
    /// The task windows a person has open, already placed and already worded.
    ///
    /// **Borrowed from a bundle the lane keeps alive**, for
    /// `the_docks_words`' reason: the raster holds `&str` and the words are
    /// `String`s somewhere. An empty slice is a machine nobody has asked
    /// anything, which is most machines most of the time.
    pub(crate) tasks: &'a [crate::task_raster::ATaskOnTheDisplay<'a>],
    /// The window of what a person can install, when the seat has one open.
    ///
    /// **Borrowed from a bundle the lane keeps alive**, for `tasks`' reason:
    /// the description holds a borrow of the window and of its words, and the
    /// words are `String`s somewhere. [`None`] is a shut window, which is every
    /// moment until a person presses the Applications entry on their Dock.
    pub(crate) market: Option<crate::market_raster::AMarketOnTheDisplay<'a>>,
    /// The alo Bar, when somebody has it open and has typed into it.
    ///
    /// **Borrowed from a bundle the lane keeps alive**, for `market`'s reason:
    /// the rows hold strings that were said this frame.
    pub(crate) the_bar: Option<crate::bar_raster::TheBarOnTheDisplay<'a>>,
}

/// Fill the fields of `frame` that only `server` and the lane can answer.
///
/// `named` is this display's name, which is what its scale is looked up by.
pub(crate) fn what_only_the_server_knows<'a>(
    frame: &mut DesktopFrame<'a>,
    server: &crate::Server,
    built: WhatTheLaneBuilt<'a>,
    named: &str,
) {
    let WhatTheLaneBuilt {
        windows,
        on_the_dock,
        in_more_apps,
        the_dock_has_open,
        the_docks_words,
        tasks,
        market,
        the_bar,
    } = built;
    frame.tasks = tasks;
    // **Whether a person has the market open**, which a desktop's own state
    // cannot know for `the_dock_has_open`'s reason exactly: it is opened by a
    // press, and presses reach the `Server`.
    frame.market = market;
    // **The Bar, for `market`'s reason exactly**: what was typed and what it
    // found are decided by presses, and presses reach the `Server`.
    frame.the_bar = the_bar;
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
    // The caller builds it with `Server::what_this_persons_dock_holds`, which
    // asks with the pins this session worked out at stand-up. It built one with
    // `Holding::nothing()` until the pins became real, and that was honest
    // while nothing stored them: the list was then the applications with a
    // window open, which is what a machine with no saved pins should show. It
    // would be a bug now, and the no-argument method exists so that it cannot
    // be written by accident.
    frame.on_the_dock = on_the_dock;
    // **And what *More apps* lists**, which is a longer list than the Dock's
    // and not derivable from it: an application whose windows are all minimised
    // is in one and not the other, by the owner's ruling of 2026-10-11.
    // `Holding::in_the_overview` is `what_the_dock_holds` before the membership
    // rule, so the two cannot drift.
    frame.in_more_apps = in_more_apps;
    // **Where the keyboard is on the bar**, which a desktop's own state cannot
    // know for `market`'s reason exactly: F6 and Tab write it and both are
    // presses, and presses reach the `Server`. Read straight off rather than
    // interpreted — `Server::the_keyboard_is_now_in` already decides that only
    // the Dock gets a slot, and a second opinion here would be a second place
    // for the answer to be wrong.
    frame.the_keyboard_is_on_the_bar = server.which_slot_on_the_bar;
    // **Whether the overflow list is open**, which a desktop's own state cannot
    // know: it is turned over by a press, and presses reach the `Server`.
    frame.the_dock_has_open = the_dock_has_open;
    // **And what its heading says**, said by the lane rather than by the draw.
    // This is the lane's answer and not the server's, and it is set here for the
    // reason the other five are: `direct_desktop` builds a frame at two sites,
    // and a field set at each of them is a field that drifts between them.
    frame.the_docks_words = the_docks_words;
}

#[cfg(test)]
#[path = "what_the_server_tells_a_frame_tests.rs"]
mod tests;
