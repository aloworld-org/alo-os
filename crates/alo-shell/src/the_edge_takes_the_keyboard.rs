//! The server's side of the keyboard on a window's edge: one state per window,
//! and the two reveal conditions it answers.
//!
//! `crate::window_edge_keys` is the machine — a key and a laid-out edge in, an
//! outcome out, and it knows nothing about a server. This is the file that
//! **holds** one of those per mapped window, which is what makes it a road a
//! person can actually walk rather than a type nothing owns.
//!
//! # Why per window, and why a list
//!
//! `crate::server`'s `edges_revealed` gives both reasons and they are the same
//! reasons: the edge is per window, so a single state here would move every
//! window's focus ring at once; and `WlSurface` is not `Ord`, the count is the
//! number of windows a person has open, and the sibling list is already a list.
//!
//! **Pruned against what is mapped**, by [`Server::only_the_edges_still_open`],
//! which `crate::the_window_edge_reveals` calls where it prunes its own. A
//! `WlSurface` is a protocol object at an id the server reuses, so a stale
//! entry would hand the next window at that id a focus ring somebody else put
//! there.
//!
//! # The edge comes from the caller
//!
//! Every method here takes the `WindowEdge` the frame is being painted from,
//! for `crate::the_window_edge_is_pointed_at`' reason: *a second `edge_of`
//! here would be a second layout that could disagree with the one being
//! painted*. A focus ring on a control at coordinates nothing drew is the same
//! fault as a press landing where no control was drawn.
//!
//! **With one exception, and it is the arrival.**
//! [`Server::the_keyboard_reached_this_windows_edge`] lays a revealed edge out
//! itself when the one it is given has no controls, because the arrival is the
//! event that *changes* whether the edge is revealed and so cannot be answered
//! from a layout computed before it. Its own documentation gives the whole
//! argument.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::window_edge::{OnTheEdge, WindowEdge};
use crate::window_edge_keys::{AnEdgeKey, TheKeyboardOnTheEdge, WhatTheEdgeKeyDid};
use crate::window_menu::{MenuRow, WhatARowDoes, WindowMenu};
use crate::window_menu_keys::TheKeyboardInTheMenu;

impl crate::Server {
    /// Where the keyboard is on this window's edge.
    ///
    /// [`TheKeyboardOnTheEdge::Nowhere`] for a window nothing has reached,
    /// which is the honest answer and the same shape
    /// `is_this_windows_edge_revealed` takes for a window nothing has told
    /// about a pointer.
    #[must_use]
    pub fn where_the_keyboard_is_on_this_windows_edge(
        &self,
        surface: &WlSurface,
    ) -> TheKeyboardOnTheEdge {
        looked_up(&self.keyboard_on_edges, surface)
    }

    /// The keyboard arrives on this window's edge.
    ///
    /// What a host calls when a person tabs to the edge.
    ///
    /// # A concealed edge is still reachable, and that needs one extra layout
    ///
    /// `edge_of` lays a concealed edge out with no controls, and **every edge
    /// starts concealed** — `Revealing::covered` is the state a window has
    /// until something reveals it. So answering from the edge the host is
    /// painting would mean a person who never touches a pointer could never
    /// reach an edge at all, which is the fault `crate::access_nodes` already
    /// names in its own words: *revealing is what a person sees and is never
    /// what a person can reach*.
    ///
    /// So where the given edge has no controls this lays one out **as
    /// revealed**, from the same window and the same
    /// `the_room_an_edge_is_clamped_in`
    /// the reveal loop uses, and names the control the keyboard lands on from
    /// that. It is the one place in this crate that builds a second layout,
    /// and the reason is that the keyboard's arrival is the one event which
    /// *changes* whether the edge is revealed: it cannot be answered from a
    /// layout computed before it happened. The arrival is what makes
    /// `Revealing::the_keyboard` true, so the next frame paints the revealed
    /// edge this named a control from.
    pub fn the_keyboard_reached_this_windows_edge(
        &mut self,
        surface: &WlSurface,
        edge: &WindowEdge,
    ) -> WhatTheEdgeKeyDid {
        let revealed = if edge.controls.is_empty() {
            Some(crate::window_edge::edge_of(
                crate::where_a_window_is(surface),
                crate::window_edge_who_draws::who_draws_a_frame(),
                true,
                self.the_room_an_edge_is_clamped_in(),
            ))
        } else {
            None
        };
        let mut where_it_is = self.where_the_keyboard_is_on_this_windows_edge(surface);
        let did = where_it_is.reached(revealed.as_ref().unwrap_or(edge));
        self.remember(surface, where_it_is);
        did
    }

    /// A key, for the window whose edge has the keyboard.
    ///
    /// `menu` is the laid-out menu where the host has one, which it does
    /// whenever it is drawing a menu. [`None`] is honest and the machine
    /// refuses rather than guessing at rows.
    pub fn this_windows_edge_was_told_a_key(
        &mut self,
        surface: &WlSurface,
        key: AnEdgeKey,
        edge: &WindowEdge,
        menu: Option<&WindowMenu>,
    ) -> WhatTheEdgeKeyDid {
        let mut where_it_is = self.where_the_keyboard_is_on_this_windows_edge(surface);
        let did = where_it_is.told(key, edge, menu);
        self.remember(surface, where_it_is);
        did
    }

    /// A person pressed the Menu control with a pointer: open the menu.
    ///
    /// **The same state the keyboard road sets**, so a menu opened by pointer
    /// and one opened by Enter on the Menu control are one menu rather than
    /// two that have to be kept in step. The trigger is recorded as
    /// [`OnTheEdge::Menu`] either way, which is what a dismissal gives focus
    /// back to.
    ///
    /// **Pressing it again closes it**, which is what every menu a person has
    /// used does and what a person will try first when they opened one by
    /// mistake. Without it the only way out by pointer would be a key.
    pub(crate) fn the_menu_was_opened_on(&mut self, surface: &WlSurface) {
        let now = match self.where_the_keyboard_is_on_this_windows_edge(surface) {
            TheKeyboardOnTheEdge::InTheMenu(_) => TheKeyboardOnTheEdge::OnAControl(OnTheEdge::Menu),
            TheKeyboardOnTheEdge::Nowhere | TheKeyboardOnTheEdge::OnAControl(_) => {
                TheKeyboardOnTheEdge::InTheMenu(TheKeyboardInTheMenu::opened_from(OnTheEdge::Menu))
            }
        };
        self.remember(surface, now);
    }

    /// A person pressed a row of the open menu: it acts, and the menu closes.
    ///
    /// Answers what the row does, for a caller that performs it, and [`None`]
    /// where the press was on a row this window will not allow — which is a
    /// refusal rather than nothing happening, and the menu stays open so the
    /// person can see that the row they pressed is the muted one.
    pub(crate) fn a_row_of_the_menu_was_pressed(
        &mut self,
        surface: &WlSurface,
        row: MenuRow,
    ) -> Option<WhatARowDoes> {
        if !row.available {
            return None;
        }
        self.remember(surface, TheKeyboardOnTheEdge::OnAControl(OnTheEdge::Menu));
        Some(row.does)
    }

    /// Keep only the windows still open.
    ///
    /// Called by `crate::the_window_edge_reveals`, which already has the list
    /// of mapped surfaces and prunes its own state from it. One walk of that
    /// list rather than two, and this file stays the only writer of its own
    /// field.
    pub(crate) fn only_the_edges_still_open(&mut self, mapped: &[WlSurface]) {
        only_what_is_still_open(&mut self.keyboard_on_edges, mapped);
    }

    /// Write this window's state back.
    fn remember(&mut self, surface: &WlSurface, where_it_is: TheKeyboardOnTheEdge) {
        remember_in(&mut self.keyboard_on_edges, surface, where_it_is);
    }
}

/// Where the keyboard is, in a table of them.
///
/// **A free function over the table rather than a method on the server**, so
/// the bookkeeping can be tested at all: every method above needs a
/// `WlSurface`, which needs a client, a socket pair and a dispatch loop —
/// `a_frame_per_display_tests` shows what that costs. The table's behaviour is
/// not about what a surface is, so it is written where it can be asked about
/// one of anything.
fn looked_up<K: PartialEq>(table: &[(K, TheKeyboardOnTheEdge)], key: &K) -> TheKeyboardOnTheEdge {
    table
        .iter()
        .find(|(it, _)| it == key)
        .map_or(TheKeyboardOnTheEdge::Nowhere, |(_, where_it_is)| {
            *where_it_is
        })
}

/// Write a state back, and **drop the entry when it becomes `Nowhere`**.
///
/// A missing entry already means `Nowhere`, so one that holds it is a row
/// saying what its absence would have said. Storing it would grow the table by
/// one for every window a key was ever pressed near, and leaving it after the
/// keyboard went away would make the table's length mean nothing.
fn remember_in<K: PartialEq + Clone>(
    table: &mut Vec<(K, TheKeyboardOnTheEdge)>,
    key: &K,
    where_it_is: TheKeyboardOnTheEdge,
) {
    if where_it_is == TheKeyboardOnTheEdge::Nowhere {
        table.retain(|(it, _)| it != key);
        return;
    }
    match table.iter_mut().find(|(it, _)| it == key) {
        Some((_, held)) => *held = where_it_is,
        None => table.push((key.clone(), where_it_is)),
    }
}

/// Keep only the keys still in this list.
fn only_what_is_still_open<K: PartialEq>(table: &mut Vec<(K, TheKeyboardOnTheEdge)>, open: &[K]) {
    table.retain(|(it, _)| open.contains(it));
}

#[cfg(test)]
#[path = "the_edge_takes_the_keyboard_tests.rs"]
mod tests;
