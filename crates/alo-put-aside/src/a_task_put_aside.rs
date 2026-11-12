//! **A running task in the panel**, beside the windows put aside.
//!
//! The owner approved `07 / A running task in the put-aside panel` (`23:2`) on
//! 2026-11-11 with *"Approve as drawn"*: a task a person minimises keeps
//! working and waits in this panel with what it is doing and Stop, **windows
//! and tasks together, newest put aside first**, because the panel answers
//! *what did I put aside*.
//!
//! # Why a task is not a [`crate::Preview`]
//!
//! A preview is one application window: it has a patch, a zoom, a privacy and
//! an application to name it by, and bringing it back is a window travelling
//! to where it was. A task window is drawn by the shell itself and has none of
//! that — *no preview picture: a task has no window to photograph*, in the
//! design's own words. So it is its own kind of entry, and the only things the
//! panel keeps about it are **which task** and **which Place it was asked on**,
//! because *a task belongs to the Place it was asked on and returns there*.
//!
//! **What the task is doing is not kept here.** The card is read from the task
//! each time it is drawn (`alo_tasks::OnItsCard`), so a card cannot go on
//! saying *working* about work that stopped while nobody redrew it from a
//! copy.

use alo_canvas::Place;

/// Which task, as the shell numbers them.
///
/// **A number the shell minted and this crate only compares**, as
/// [`alo_dock::window::WindowId`] is for windows: the shell's task list is the
/// only thing that knows what it names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskNumber(usize);

impl TaskNumber {
    /// The task the shell numbered `at`.
    #[must_use]
    pub const fn numbered(at: usize) -> Self {
        Self(at)
    }

    /// The number the shell gave it.
    #[must_use]
    pub const fn at(self) -> usize {
        self.0
    }
}

/// A task in the panel: which one, and the Place it returns to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ATaskAside {
    /// Which task.
    task: TaskNumber,
    /// The Place it was asked on.
    place: Place,
}

impl ATaskAside {
    /// The task `task`, asked on `place`.
    #[must_use]
    pub const fn of(task: TaskNumber, place: Place) -> Self {
        Self { task, place }
    }

    /// Which task.
    #[must_use]
    pub const fn task(self) -> TaskNumber {
        self.task
    }

    /// The Place it was asked on, which is where it goes back to.
    #[must_use]
    pub const fn place(self) -> Place {
        self.place
    }
}

/// One thing in the panel, in the panel's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aside<'a> {
    /// A window put aside.
    AWindow(&'a crate::Preview),
    /// A task put aside.
    ATask(ATaskAside),
    /// One of alo's own windows put aside — the terminal, Settings or the
    /// record (`crate::one_of_alos_own_windows`).
    AnAloWindow(crate::one_of_alos_own_windows::AnAloWindowAside),
}
