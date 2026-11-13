//! **What a screen reader is told happened**, between one tree and the next.
//!
//! A reader speaks when it is told something moved: a window came to the
//! front, the keyboard went somewhere, the thing with the keyboard changed its
//! name. A tree that is only *there* is read by nobody — Orca walks from where
//! the keyboard is, and until this file the shell never said where that was,
//! so a sheet alo put up and every key went to was silent.
//!
//! So this decides, from what the keyboard was on last time and the tree now,
//! the few things to tell — and only for alo's own sheets and Settings, the
//! places the shell itself holds the keyboard (`crate::access_sheet_in_front`).
//! `crate::access_bus` says them on the bus.
//!
//! # Four things, and when
//!
//! - **A sheet comes to the front** — a new question, not the same one redrawn,
//!   or Settings becoming the window in front: it is activated, and the
//!   keyboard is said to be on its control.
//! - **The keyboard moves on the same sheet** — an arrow on *Open with*, or on
//!   a Settings page: the keyboard is said to be on the new row.
//! - **The control with the keyboard is called something new** — the password
//!   was not right, and the field is now named by the line that says so.
//! - **The control with the keyboard is set or chosen, or stops being** — a
//!   switch pressed on Settings (*added 2026-11-12*).
//!
//! Nothing else is said. A sheet going down says nothing, because what the
//! keyboard goes back to is a window this tree does not hold; and nothing is
//! said when no sheet is up, so the desktop behaves exactly as it did before.

use crate::access_nodes::ReadAloudTree;
use crate::access_sheet_in_front::TheSheet;

/// Where the keyboard was on the sheet in front, as last told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WhereTheKeyboardWas {
    /// Which sheet.
    which: TheSheet,
    /// The keyboard's node, counted from the sheet's own.
    from_the_sheet: usize,
    /// Where that node was on the bus.
    path: String,
    /// What that node was called.
    called: String,
    /// Whether that node was set or chosen.
    checked: bool,
}

/// One thing a reader is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AnEvent {
    /// The sheet at `path`, called `name`, came to the front.
    InFront {
        /// Its own node.
        path: String,
        /// What it is called.
        name: String,
    },
    /// The keyboard is now on the node at `path`.
    TheKeyboardIsHere {
        /// Where.
        path: String,
    },
    /// The node with the keyboard, at `path`, is now called `name`.
    NowCalled {
        /// Where.
        path: String,
        /// What it is called now.
        name: String,
    },
    /// The node with the keyboard, at `path`, is now set or chosen, or no
    /// longer is: a switch pressed on Settings, a choice made there.
    NowChecked {
        /// Where.
        path: String,
        /// Whether it is set now.
        checked: bool,
    },
}

/// What to tell a reader, now that the tree is `now` and the keyboard was
/// last told to be at `was` — and where it is now, to remember for next time.
pub(crate) fn what_happened(
    was: Option<&WhereTheKeyboardWas>,
    now: &ReadAloudTree,
) -> (Vec<AnEvent>, Option<WhereTheKeyboardWas>) {
    let Some(sheet) = now.in_front() else {
        return (Vec::new(), None);
    };
    let (Some(its_own), Some(keyboard)) =
        (now.nodes().get(sheet.at), now.nodes().get(sheet.keyboard))
    else {
        return (Vec::new(), None);
    };
    let is = WhereTheKeyboardWas {
        which: sheet.which,
        from_the_sheet: sheet.the_keyboard_from_the_sheet(),
        path: keyboard.path.clone(),
        called: keyboard.name.clone(),
        checked: crate::access_roles::is_checked(keyboard.states),
    };
    let here = AnEvent::TheKeyboardIsHere {
        path: keyboard.path.clone(),
    };
    let told = match was {
        // **Moved, or renumbered under a reader's feet**: a window opening
        // behind the sheet moves every path after it, and a reader holding
        // the old path would be holding some other node — so it is told
        // where the keyboard is again.
        Some(was)
            if was.which == is.which
                && (was.from_the_sheet != is.from_the_sheet || was.path != is.path) =>
        {
            vec![here]
        }
        Some(was) if was.which == is.which => {
            let mut told = Vec::new();
            if was.called != is.called {
                told.push(AnEvent::NowCalled {
                    path: keyboard.path.clone(),
                    name: is.called.clone(),
                });
            }
            // **A switch pressed where the keyboard is**, or a choice made:
            // what a sighted person sees flip, a reader is told.
            if was.checked != is.checked {
                told.push(AnEvent::NowChecked {
                    path: keyboard.path.clone(),
                    checked: is.checked,
                });
            }
            told
        }
        _ => vec![
            AnEvent::InFront {
                path: its_own.path.clone(),
                name: its_own.name.clone(),
            },
            here,
        ],
    };
    (told, Some(is))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "access_events_tests.rs"]
mod tests;
