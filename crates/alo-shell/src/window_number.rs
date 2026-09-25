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

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use smithay::reexports::wayland_server::{Resource as _, protocol::wl_surface::WlSurface};

/// The next number nobody has had.
///
/// Sixty-four bits at one window per nanosecond is longer than any machine
/// runs, so there is no wrap to handle and no reuse to get wrong.
static NEXT: AtomicU64 = AtomicU64::new(1);

/// Which number each window has.
#[derive(Debug, Default)]
pub(crate) struct Numbers {
    /// By the surface's protocol identity, which is stable while it lives.
    given: HashMap<u32, u64>,
}

impl Numbers {
    /// This window's number, giving it one if it has none.
    pub(crate) fn of(&mut self, surface: &WlSurface) -> u64 {
        *self
            .given
            .entry(surface.id().protocol_id())
            .or_insert_with(|| NEXT.fetch_add(1, Ordering::Relaxed))
    }

    /// Keep only the windows still here, and say which numbers went.
    ///
    /// No number is given back: see this file's header. What is forgotten is
    /// the way back from a surface, so a closed window's entry does not grow
    /// the map forever — and what is returned is what a division needs to let
    /// go of.
    pub(crate) fn keep_only(&mut self, alive: &std::collections::HashSet<u32>) -> Vec<u64> {
        let gone: Vec<u64> = self
            .given
            .iter()
            .filter(|(at, _)| !alive.contains(*at))
            .map(|(_, number)| *number)
            .collect();
        self.given.retain(|at, _| alive.contains(at));
        gone
    }

    /// Which protocol identity a surface has, for the caller collecting them.
    pub(crate) fn identity(surface: &WlSurface) -> u32 {
        surface.id().protocol_id()
    }
}
