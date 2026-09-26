//! The number a window is known by outside this crate.
//!
//! `alo-dividing` and `alo-desktops` both hold windows by
//! `WindowId::from_compositor(u64)`, and this compositor is the thing that has
//! to supply that number. Inside the crate a window is a `WlSurface` and a role
//! handle; neither is a number, and neither crosses a crate boundary.
//!
//! # Never reused, which is the whole point
//!
//! A number is handed out once and never again, even after the window it
//! belonged to has closed. A reused number is a division that still holds a
//! share for a window that is gone and a **different** window walking into it —
//! somebody's new terminal appearing exactly where their closed editor was,
//! sized as the editor, because a tree was holding a number rather than a
//! window.
//!
//! That is also why this is not the `visibility` identity `crate::surfaces`
//! keeps. That one is deliberately fresh for each uninterrupted mapping, so a
//! window that unmaps and maps again is a new one; this one has to survive that
//! — a person's window that flickers is still the window in their layout.
//!
//! # The number lives in the window's own state
//!
//! **Not in a map keyed by the surface, and this is the second attempt.** The
//! first kept a `HashMap` keyed by `surface.id().protocol_id()` — which is the
//! number a surface's *own client* knows it by, and which starts again at 1 for
//! every client. Two applications with one window each are both
//! `wl_surface@3`, so they were given **one number between them**, and a
//! division asked to divide between two windows that are really the same window
//! refuses with *nothing to share with*. A walk with two real clients found
//! that on 2026-09-26; it would have happened to the second application anybody
//! opened.
//!
//! Keying by the whole `ObjectId` fixes the collision and is a
//! `clippy::mutable_key_type`: that type has interior mutability, and a key
//! whose hash the compiler will not vouch for is not one to build somebody's
//! window layout on. So the number goes where the rest of this compositor's
//! per-window state goes — the surface's own data map, exactly as
//! `crate::window_placement` keeps a placement. It is unique by construction,
//! needs no bookkeeping to stay unique, and goes away with the surface.

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::with_states;

/// The next number nobody has had.
///
/// Sixty-four bits at one window per nanosecond is longer than any machine
/// runs, so there is no wrap to handle and no reuse to get wrong.
static NEXT: AtomicU64 = AtomicU64::new(1);

/// The number this window was given, kept in the window's own state.
#[derive(Default)]
struct Given(Mutex<Option<u64>>);

/// Which windows were open when this was last told, so a division can be told
/// when one is not.
#[derive(Debug, Default)]
pub(crate) struct Numbers {
    /// The numbers open at the last telling.
    open: HashSet<u64>,
}

impl Numbers {
    /// This window's number, giving it one if it has none.
    pub(crate) fn of(surface: &WlSurface) -> u64 {
        if let Some(already) = Self::given_to(surface) {
            return already;
        }
        let number = NEXT.fetch_add(1, Ordering::Relaxed);
        with_states(surface, |states| {
            states.data_map.insert_if_missing_threadsafe(Given::default);
            if let Some(given) = states.data_map.get::<Given>() {
                let mut held = given.0.lock().unwrap_or_else(|_| std::process::abort());
                // Whoever reached this lock first owns the number. A number this
                // call took and did not use is simply never used again, which is
                // what a counter that never reuses allows.
                if held.is_none() {
                    *held = Some(number);
                }
            }
        });
        Self::given_to(surface).unwrap_or(number)
    }

    /// This window's number, or [`None`] where it has never had one.
    pub(crate) fn given_to(surface: &WlSurface) -> Option<u64> {
        with_states(surface, |states| {
            states
                .data_map
                .get::<Given>()
                .and_then(|given| *given.0.lock().unwrap_or_else(|_| std::process::abort()))
        })
    }

    /// These are the windows open now; say which numbers have gone.
    ///
    /// No number is given back: see this file's header. What is returned is
    /// what a division needs to let go of.
    pub(crate) fn open_now(&mut self, open: HashSet<u64>) -> Vec<u64> {
        let gone: Vec<u64> = self.open.difference(&open).copied().collect();
        self.open = open;
        gone
    }
}

/// A number for something this compositor draws itself.
///
/// The egress indicator, the approval surface and the agent overlay are on
/// every desktop and belong to no client, so there is no `WlSurface` to hold a
/// number in — but `alo-desktops` holds them as windows and needs numbers for
/// them.
///
/// Taken from the same counter as every other window's, so one of the three can
/// never collide with a client's.
pub(crate) fn reserve() -> u64 {
    NEXT.fetch_add(1, Ordering::Relaxed)
}
