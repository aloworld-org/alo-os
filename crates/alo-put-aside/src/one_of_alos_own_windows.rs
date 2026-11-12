//! **One of alo's own windows in the panel**: the terminal, Settings or the
//! record, put aside beside the applications' windows and the tasks.
//!
//! The owner's approved design `10 / Minimised windows, Mac-style` (`27:2` in
//! `XbSmsBYix0TrcQ0Qzaup64`, 2026-11-12) puts every window a person minimises
//! in this panel, and alo's own windows are windows like any other since
//! ADR 0145 put them on the canvas. Until this file they could not be
//! minimised at all: their edge carried Close alone, because a Minimise that
//! reached nothing would have looked live and not been.
//!
//! # Why it is not a [`crate::Preview`]
//!
//! A preview is an **application's** window: a compositor's number, a patch on
//! the plane, an application to name it by and a privacy the person set. alo's
//! windows are drawn by the shell itself, and there is at most one of each, so
//! the panel keeps only **which one** and **the Place it was on** — the same two
//! facts it keeps for a task (`crate::a_task_put_aside`), and for the same
//! reason: everything else is read from the window each time it is drawn.

use alo_canvas::Place;

/// Which of alo's own windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlosOwnWindow {
    /// alo's terminal.
    TheTerminal,
    /// Settings.
    Settings,
    /// The record — *History* on the Dock.
    TheRecord,
}

/// One of alo's windows in the panel: which one, and the Place it returns to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnAloWindowAside {
    /// Which window.
    which: AlosOwnWindow,
    /// The Place it was on when it was put aside.
    place: Place,
}

impl AnAloWindowAside {
    /// `which`, put aside from `place`.
    #[must_use]
    pub const fn of(which: AlosOwnWindow, place: Place) -> Self {
        Self { which, place }
    }

    /// Which window.
    #[must_use]
    pub const fn which(self) -> AlosOwnWindow {
        self.which
    }

    /// The Place it was on, which is where it goes back to.
    #[must_use]
    pub const fn place(self) -> Place {
        self.place
    }
}
