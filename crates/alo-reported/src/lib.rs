//! What alo said it was doing, as it crosses to the surface that draws it.
//!
//! Task 7 of `docs/autonomy/putting-a-window-aside.md` asked who carries *alo is
//! working on this window* from the agent service to the crate holding the
//! Panel, recorded that nothing did, and recorded that choosing between three
//! answers was the owner's rather than a lane's. **The owner chose the third on
//! 2026-10-02: a third thing bridges them.** This crate is that third thing.
//!
//! # What makes it a contract rather than a model
//!
//! It declares no crate of ours. `alo-put-aside` already holds [the panel's own
//! report type] and could not serve here: it declares `alo-canvas` and
//! `alo-dock`, so a reporter built against it would drag the panel's model into
//! the agent service, which is the dependency the ruling forbids running the
//! other way.
//!
//! It also names **its own window handle** rather than borrowing one. There are
//! three `WindowId` types in this workspace — `alo_dock::window::WindowId`,
//! `alo_dividing::window::WindowId` and `alo_capturing::window::WindowId`, two
//! of them `u64` and one `u32` — so a contract that picked one would make every
//! reporter depend on whichever crate owns it, and a contract that picked wrong
//! would be silently converting between two different window numbering schemes.
//! [`ReportedWindow`] is an opaque handle, and mapping it to a window is the
//! coordinator's work, where the mapping can be refused.
//!
//! # A report grants no authority
//!
//! **Nothing here is a permission, a grant, or a request.** A [`Reported`] says
//! what its sender claims to be doing; it cannot make that true, cannot widen
//! what the sender may touch, and cannot cause a window to exist. The scope in
//! [`Reported::may_change`] is a **claim to be checked against what was actually
//! granted**, never a grant in itself — the coordinator compares it and refuses
//! a report claiming more than the person allowed. A reader of this crate who
//! treats a field here as an authorisation has read it backwards.
//!
//! This is why the type carries no identity of the sender and no token. Proving
//! who is speaking belongs to the channel the report arrived on, and a proof
//! carried *inside* the thing being proved is not a proof.
//!
//! # Why every part arrives at once
//!
//! [`Reported::of`] takes the whole report and refuses an incomplete one, for
//! the reason `alo_put_aside::AtWork` gives for the same choice: a report
//! assembled field by field is a report that can be drawn half-built, and
//! half-built is the empty control that whole surface refuses.

use std::path::PathBuf;

/// The window a report is about, as its sender knows it.
///
/// Opaque on purpose. It is a handle the sender was given, not a window, and
/// turning it into one is the coordinator's work — see this module's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReportedWindow(u64);

impl ReportedWindow {
    /// The window this handle stands for.
    #[must_use]
    pub const fn of(handle: u64) -> Self {
        Self(handle)
    }

    /// The handle, for a coordinator that has a mapping for it.
    #[must_use]
    pub const fn handle(self) -> u64 {
        self.0
    }
}

/// What a report claims it may change.
///
/// **A claim, not a grant.** The coordinator checks it against what the person
/// actually allowed and refuses a report claiming more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MayChange {
    /// It says it is only reading, and will propose before changing anything.
    NothingYet,
    /// It says it may change these, and nothing else.
    TheseFiles(Vec<PathBuf>),
}

/// One report of what alo is doing in one window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reported {
    /// The window this is about, as the sender knows it.
    window: ReportedWindow,
    /// The task, in the words it was given in.
    task: String,
    /// What it claims it may change. A claim, never a grant.
    may_change: MayChange,
    /// How far through, where the sender could say.
    hundredths: Option<u8>,
    /// The last thing it did that the person approved.
    last_confirmed: Option<String>,
    /// What it has stopped and is waiting for, if it has.
    waiting_for: Option<String>,
}

/// Why a report was not made.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotReported {
    /// A report about no named task is a report about nothing.
    #[error("a report names the task it is about, and this one is blank")]
    ItNamesNoTask,
    /// Said rather than clamped, for `alo_put_aside::Progress`' own reason: a
    /// clamp turns a sender's arithmetic mistake into a full bar, which is the
    /// most convincing wrong answer available.
    #[error("{0} hundredths is past the end, and a clamp would draw it as finished")]
    PastTheEnd(u8),
    /// An empty sentence is worse than no sentence: it draws a heading with
    /// nothing under it, which is the empty control the panel refuses.
    #[error("a blank last-confirmed action is not the same as nothing confirmed yet")]
    ItConfirmedNothing,
    /// A stop with no reason cannot be answered by the person it stopped for.
    #[error("a report that says it is waiting says what it is waiting for")]
    ItWaitsForNothing,
}

impl Reported {
    /// A report of alo working on a named task in a window.
    ///
    /// Every part at once — see this module's header for why.
    ///
    /// # Errors
    ///
    /// [`NotReported::ItNamesNoTask`] for a blank task.
    /// [`NotReported::PastTheEnd`] for progress above a hundred hundredths.
    /// [`NotReported::ItConfirmedNothing`] for a blank last-confirmed action;
    /// pass [`None`] to say nothing has been confirmed yet, which is a
    /// different and honest answer.
    /// [`NotReported::ItWaitsForNothing`] for a blank reason for waiting; pass
    /// [`None`] to say it is not waiting.
    pub fn of(
        window: ReportedWindow,
        task: &str,
        may_change: MayChange,
        hundredths: Option<u8>,
        last_confirmed: Option<&str>,
        waiting_for: Option<&str>,
    ) -> Result<Self, NotReported> {
        if task.trim().is_empty() {
            return Err(NotReported::ItNamesNoTask);
        }
        if let Some(hundredths) = hundredths.filter(|&hundredths| hundredths > 100) {
            return Err(NotReported::PastTheEnd(hundredths));
        }
        if last_confirmed.is_some_and(|said| said.trim().is_empty()) {
            return Err(NotReported::ItConfirmedNothing);
        }
        if waiting_for.is_some_and(|said| said.trim().is_empty()) {
            return Err(NotReported::ItWaitsForNothing);
        }
        Ok(Self {
            window,
            task: task.to_owned(),
            may_change,
            hundredths,
            last_confirmed: last_confirmed.map(ToOwned::to_owned),
            waiting_for: waiting_for.map(ToOwned::to_owned),
        })
    }

    /// The window this report is about, as its sender knows it.
    #[must_use]
    pub const fn window(&self) -> ReportedWindow {
        self.window
    }

    /// The task, in the words it was given in.
    #[must_use]
    pub fn task(&self) -> &str {
        &self.task
    }

    /// What it claims it may change, to be checked and never trusted.
    #[must_use]
    pub const fn may_change(&self) -> &MayChange {
        &self.may_change
    }

    /// How far through, where the sender could say.
    #[must_use]
    pub const fn hundredths(&self) -> Option<u8> {
        self.hundredths
    }

    /// The last thing it did that the person approved.
    #[must_use]
    pub fn last_confirmed(&self) -> Option<&str> {
        self.last_confirmed.as_deref()
    }

    /// What it has stopped and is waiting for, if it has.
    #[must_use]
    pub fn waiting_for(&self) -> Option<&str> {
        self.waiting_for.as_deref()
    }
}

#[cfg(test)]
mod tests;
