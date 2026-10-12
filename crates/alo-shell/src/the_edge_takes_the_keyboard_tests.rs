//! The table of which window's edge has the keyboard.
//!
//! **What is asked here and what is not.** The three free functions carry every
//! decision this file makes — where an unknown window is, when an entry is
//! written, when it is dropped, and what a closed window leaves behind — and
//! they are asked with a stand-in key. The methods above them are each one
//! line over one of these, and reaching one needs a `WlSurface`, which needs a
//! client, a socket pair and a dispatch loop: `a_frame_per_display_tests`
//! shows what that costs for one surface. So the arithmetic is tested and the
//! three one-line methods are not, which is said here rather than left for a
//! reader to discover from a coverage report.

use super::{looked_up, only_what_is_still_open, remember_in};
use crate::window_edge::OnTheEdge;
use crate::window_edge_keys::TheKeyboardOnTheEdge;
use crate::window_menu_keys::TheKeyboardInTheMenu;

/// A stand-in for a surface: anything the table can compare and clone.
type AWindow = &'static str;

fn on_close() -> TheKeyboardOnTheEdge {
    TheKeyboardOnTheEdge::OnAControl(OnTheEdge::Close)
}

fn in_a_menu() -> TheKeyboardOnTheEdge {
    TheKeyboardOnTheEdge::InTheMenu(TheKeyboardInTheMenu::opened_from(OnTheEdge::Menu))
}

/// **A window nothing has reached is `Nowhere`**, not an absent answer a
/// caller has to interpret.
#[test]
fn a_window_the_keyboard_has_never_reached_is_nowhere() {
    let table: Vec<(AWindow, TheKeyboardOnTheEdge)> = Vec::new();
    assert_eq!(
        looked_up(&table, &"a window"),
        TheKeyboardOnTheEdge::Nowhere
    );
}

/// One window's state says nothing about another's.
#[test]
fn one_windows_keyboard_is_not_anothers() {
    let mut table: Vec<(AWindow, TheKeyboardOnTheEdge)> = Vec::new();
    remember_in(&mut table, &"mail", on_close());
    assert_eq!(looked_up(&table, &"mail"), on_close());
    assert_eq!(
        looked_up(&table, &"files"),
        TheKeyboardOnTheEdge::Nowhere,
        "a focus ring on one window must not appear on another"
    );
}

/// Writing twice replaces rather than appends.
#[test]
fn writing_a_window_twice_leaves_one_entry() {
    let mut table: Vec<(AWindow, TheKeyboardOnTheEdge)> = Vec::new();
    remember_in(&mut table, &"mail", on_close());
    remember_in(&mut table, &"mail", in_a_menu());
    assert_eq!(table.len(), 1, "{table:?}");
    assert_eq!(looked_up(&table, &"mail"), in_a_menu());
}

/// **`Nowhere` drops the entry**, so the table's length means something.
#[test]
fn the_keyboard_leaving_takes_the_entry_with_it() {
    let mut table: Vec<(AWindow, TheKeyboardOnTheEdge)> = Vec::new();
    remember_in(&mut table, &"mail", on_close());
    assert_eq!(table.len(), 1);
    remember_in(&mut table, &"mail", TheKeyboardOnTheEdge::Nowhere);
    assert!(table.is_empty(), "{table:?}");
    assert_eq!(
        looked_up(&table, &"mail"),
        TheKeyboardOnTheEdge::Nowhere,
        "and the answer is unchanged, which is why the row was worth nothing"
    );
}

/// **A window at rest is never written at all.**
#[test]
fn a_window_nothing_happened_to_is_not_stored() {
    let mut table: Vec<(AWindow, TheKeyboardOnTheEdge)> = Vec::new();
    remember_in(&mut table, &"mail", TheKeyboardOnTheEdge::Nowhere);
    assert!(
        table.is_empty(),
        "every window a key was pressed near would be a row: {table:?}"
    );
}

/// **A window that closes takes its focus ring with it.**
///
/// A `WlSurface` is a protocol object at an id the server reuses, so a stale
/// row would hand the next window at that id a focus ring somebody else put
/// there — which is the fault the sibling `edges_revealed` prunes for.
#[test]
fn a_window_that_closed_leaves_no_focus_ring_for_the_next_one() {
    let mut table: Vec<(AWindow, TheKeyboardOnTheEdge)> = Vec::new();
    remember_in(&mut table, &"mail", on_close());
    remember_in(&mut table, &"files", in_a_menu());
    assert_eq!(table.len(), 2);

    only_what_is_still_open(&mut table, &["files"]);
    assert_eq!(
        looked_up(&table, &"mail"),
        TheKeyboardOnTheEdge::Nowhere,
        "the closed window still has a focus ring"
    );
    assert_eq!(
        looked_up(&table, &"files"),
        in_a_menu(),
        "and the open one lost its own, which is the other half"
    );
}

/// Nothing open empties the table.
#[test]
fn nothing_open_leaves_nothing_held() {
    let mut table: Vec<(AWindow, TheKeyboardOnTheEdge)> = Vec::new();
    remember_in(&mut table, &"mail", on_close());
    only_what_is_still_open(&mut table, &[]);
    assert!(table.is_empty(), "{table:?}");
}
