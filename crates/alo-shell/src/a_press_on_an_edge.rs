//! What a press on a window's external edge reaches.
//!
//! §6 of `docs/design/the-external-window-edge.md`. The decision — *which
//! control, or the movement region, or nothing* — is
//! [`crate::the_window_edge_reveals::what_a_press_does`], read off the same
//! classification the reveal reads. This file is the wiring, and it **reuses
//! every mechanism rather than adding one**:
//!
//! | the specification | what already does it |
//! |---|---|
//! | dragging the movement region moves the window | `Surfaces::start_move_from_the_name` |
//! | Minimise puts the window aside through the existing mechanism | `Server::asked_to_put_aside` |
//! | Maximise/restore preserves ordinary geometry | `Server::set_window_maximized` |
//! | Close uses the application's normal close request | `Server::request_window_close` |
//!
//! # Moving acts now; the other three wait, and that is not a preference
//!
//! A drag has to take hold on the press that started it or the pointer has
//! already moved on, and `start_move_from_the_name` answers `bool` — so it is
//! called here. The other three are **recorded and met in
//! `crate::direct_desktop`**, for the reason `Server::asked_to_put_aside`'s own
//! note gives: putting a window aside needs the live `Panel`, which this crate
//! lays out and `alo-desktop` owns, and input has no desktop in scope. Close and
//! maximise join it rather than acting here because their mechanisms answer with
//! their own error types, and a press handler that returned `InputError` for a
//! refused close would be relabelling somebody else's failure as an input fault.
//!
//! # Which window, when two edges overlap
//!
//! The first mapped surface whose edge claims the point, in
//! `Server::mapped_surfaces` order. Two windows stacked within 44 logical
//! pixels of each other have overlapping edges, and **this is a rule rather
//! than a measurement**: nothing in this crate yet orders mapped surfaces by
//! what is in front, so *topmost wins* cannot be implemented honestly here.
//! Said out loud because a person who stacks two windows that close will find
//! the wrong one answers, and the fix is an ordering this crate does not have
//! rather than a different hit test.

use smithay::backend::input::ButtonState;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::the_window_edge_reveals::{WhatAPressDoes, what_a_press_does};
use crate::window_edge::{OnTheEdge, edge_of};

/// Which existing mechanism an edge's control reaches.
///
/// **The decision, with no window in it**, which is the division
/// `crate::peeking_at_a_put_aside_window` already draws and states: *the
/// decision therefore lives here, where the tests are.* A `WlSurface` cannot be
/// made without a client, so a mapping held only inside the act would be
/// checkable on a machine with a compositor and two real applications and
/// nowhere else — and the mapping is the part that can be wrong. Close reaching
/// the put-aside road would compile, run, and put a window away when a person
/// asked to shut it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheRoad {
    /// `Server::asked_to_put_aside`, met in `crate::direct_desktop` where the
    /// panel is.
    PutItAside,
    /// `Server::set_window_maximized`, toggled against the window's own mode.
    MaximiseOrRestore,
    /// `Server::request_window_close` — the application's own close request,
    /// with its own unsaved-work handling.
    AskItToClose,
    /// **Nothing on this machine does it.** Not a default and not a catch-all:
    /// the one control in this state is the window menu, which the owner's
    /// ruling of 2026-10-09 put on the edge and which nothing in this crate
    /// opens. A press that reached a silent arm is what a person reports as
    /// *the button does nothing*, so this is returned to a caller that says so.
    NothingHasOneYet,
}

/// Which road one of the edge's controls takes.
///
/// Exhaustive on purpose — no `_` arm. A fifth control added to
/// [`OnTheEdge`] should stop this compiling and make somebody decide, rather
/// than falling into whichever arm a catch-all pointed at.
pub(crate) const fn the_road_an_edge_press_takes(does: OnTheEdge) -> TheRoad {
    match does {
        OnTheEdge::Minimise => TheRoad::PutItAside,
        OnTheEdge::Maximise => TheRoad::MaximiseOrRestore,
        OnTheEdge::Close => TheRoad::AskItToClose,
        OnTheEdge::Menu => TheRoad::NothingHasOneYet,
    }
}

impl crate::Server {
    /// Act on a press that landed on some window's external edge.
    ///
    /// Answers whether the press was the edge's. `true` means **nothing is
    /// delivered to the client**: the edge is the shell's own surface, outside
    /// the application's content, and no client was ever told about it — the
    /// same answer, for the same reason, that the resize bands and the name band
    /// give in `crate::pointer`.
    ///
    /// Releases are not the edge's: a drag's release is consumed by
    /// `Surfaces::window_move_button`, which `pointer_button` asks before this,
    /// and a control acts on its press. So this answers `false` for anything
    /// that is not a press.
    pub(crate) fn a_press_on_an_edge(&mut self, button: u32, state: ButtonState) -> bool {
        if state != ButtonState::Pressed {
            return false;
        }
        let Some(at) = self.where_the_pointer_is_in_logical_pixels() else {
            // **No pointer is not the origin.** The same rule its accessor
            // carries: a seat that has never had a pointing device would
            // otherwise press whatever window's edge contains `(0, 0)`.
            return false;
        };
        let decorations = crate::window_edge_who_draws::who_draws_a_frame();
        let mapped: Vec<WlSurface> = self.mapped_surfaces().cloned().collect();
        for surface in mapped {
            let edge = edge_of(
                crate::where_a_window_is(&surface),
                decorations,
                self.is_this_windows_edge_revealed(&surface),
            );
            match what_a_press_does(&edge, at) {
                WhatAPressDoes::Nothing => {}
                WhatAPressDoes::MovesTheWindow => {
                    // **Taken hold of now, not recorded.** It answers `false`
                    // where another gesture or a menu already owns the pointer,
                    // and then this press is not the edge's after all.
                    return self.surfaces.start_move_from_the_name(&surface, button);
                }
                WhatAPressDoes::This(does) => {
                    self.asked_on_an_edge.push((surface, does));
                    return true;
                }
            }
        }
        false
    }

    /// Carry out what was pressed on an edge, now that the desktop is in scope.
    ///
    /// Called once a frame by the draw, which is where the `Panel` is. Returns
    /// what it could not do, so a caller that holds the panel finishes the rest
    /// — rather than this file either reaching for a panel it has not got or
    /// dropping an action a person asked for.
    ///
    /// **Every ask is taken off the list, whether it succeeded or not.** A press
    /// that failed is not retried next frame: a person pressed Close once and a
    /// refusal is an answer, while an ask that stayed on the list would be a
    /// close request sent sixty times a second.
    pub(crate) fn meet_what_was_pressed_on_an_edge(&mut self) -> Vec<(WlSurface, OnTheEdge)> {
        let asked = std::mem::take(&mut self.asked_on_an_edge);
        let mut for_the_caller = Vec::new();
        for (surface, does) in asked {
            match the_road_an_edge_press_takes(does) {
                // **The existing road, not a second one.** `asked_to_put_aside`
                // is already met in `crate::direct_desktop` with the live panel,
                // so an edge's Minimise joins the queue a person's other ways of
                // putting a window aside already use, and arrives there having
                // been through the same rules.
                TheRoad::PutItAside => self.asked_to_put_aside.push(surface),
                TheRoad::MaximiseOrRestore => {
                    // Toggle, which is what *Maximise/restore* means: the one
                    // control does both and the window's own mode says which.
                    let already = self.surfaces.has_window_mode(&surface);
                    // A refusal is dropped deliberately rather than unwrapped —
                    // `clippy::unwrap_used` is denied here, and a client that
                    // refuses to be maximised is not an input error. What a
                    // person sees is the window not moving, which is the honest
                    // outcome of a client saying no.
                    let _ = self.set_window_maximized(&surface, !already);
                }
                TheRoad::AskItToClose => {
                    // The application's **normal** close request, so its own
                    // unsaved-work handling runs. Nothing here closes a window.
                    let _ = self.request_window_close(&surface);
                }
                TheRoad::NothingHasOneYet => for_the_caller.push((surface, does)),
            }
        }
        for_the_caller
    }
}

#[cfg(test)]
#[path = "a_press_on_an_edge_tests.rs"]
mod tests;
