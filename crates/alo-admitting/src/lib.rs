//! What a report must pass before any surface may draw it.
//!
//! Task 7 of `docs/autonomy/putting-a-window-aside.md` — *Alo working in a
//! minimised window* — recorded that nothing carried *alo is working on this
//! window* from the agent service to the crate holding the Panel, and that
//! choosing who should was the owner's. **The owner chose on 2026-10-02: the
//! reports cross through the trusted coordinator, the drawing crate does not
//! depend on the assistant-service crate, and a small neutral contract carries
//! the updates.** This crate is the coordinator.
//!
//! # Authority comes from the person, and never from a report
//!
//! This is the whole of the design, and it is visible in the two entry points:
//!
//! | | |
//! |---|---|
//! | [`Reports::handed_over`] | **The person's act.** A window is given to alo for a named task, with what it may change. This is where authority enters. |
//! | [`Reports::arrived`] | **A report.** It can say how a task is going and nothing else. It cannot create the association, cannot rename the task, and cannot widen the scope. |
//!
//! So a report is not trusted and does not need to be. It is checked against
//! something the person did, and a report about a window nobody handed over, or
//! naming a task this window is not running, or claiming more than was granted,
//! is refused **by name** rather than drawn.
//!
//! # A lost connection is not silence
//!
//! The owner's ruling is explicit that a lost connection becomes *status
//! unavailable* and never remains *working*. [`Reports::the_reporter_is_gone`]
//! is that, and `alo_put_aside::WhatAloIsDoing::StatusUnavailable` is the state
//! it supplies. Doing nothing instead would leave the last report on screen
//! claiming work is under way for as long as the panel held it, and the person
//! would read a dropped connection as a task still running.
//!
//! **Ending is a different thing from being lost**, and both are said out loud:
//! [`Reports::taken_back`] is the window ceasing to be alo's, which supplies
//! `Nothing` and draws no agent section at all.

use alo_dock::window::WindowId;
use alo_put_aside::{
    Scope, WhatAloIsDoing, alo_at_work::AtWork, how_far_alo_has_got::Progress,
    requires_you::RequiresYou,
};
use alo_reported::{MayChange, Reported, ReportedWindow};

/// What the person allowed alo to change in a window.
///
/// Set by [`Reports::handed_over`] and never by a report. A report's own
/// [`MayChange`] is a claim compared against this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Granted {
    /// It may read and propose, and change nothing.
    ReadingOnly,
    /// It may change these, and nothing else.
    TheseFiles(Vec<std::path::PathBuf>),
}

/// Why a report was not admitted.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAdmitted {
    /// No window was handed over under this handle.
    ///
    /// **Refused rather than mapped to anything.** A report that could conjure
    /// an association would be a report granting itself one.
    #[error("no window was handed to alo under handle {0}")]
    NobodyHandedThatOver(u64),
    /// The window is alo's, for a different task than this report names.
    ///
    /// A second sender cannot take over a window by reporting against it, and a
    /// stale sender cannot overwrite the task that replaced its own.
    #[error("that window was handed over for a different task, so this report is not about it")]
    ADifferentTask,
    /// The report claims it may change more than the person allowed.
    ///
    /// **The report is refused, not narrowed.** Narrowing it would draw a
    /// sentence the sender did not say, about a scope it does not believe it
    /// has, which is a worse answer than refusing.
    #[error("the report claims more than was granted for that window")]
    MoreThanWasGranted,
    /// The report was whole on the wire and cannot be a panel report.
    ///
    /// Carried through rather than flattened, because the panel's own
    /// constructor is where *a report may not be drawn half-built* is enforced,
    /// and its refusals say which part was missing.
    #[error("the report cannot be shown: {0}")]
    ItCannotBeShown(#[from] alo_put_aside::alo_at_work::NotAtWork),
    /// Progress past a hundred hundredths.
    #[error("the report says it is {0} hundredths through, which is past the end")]
    PastTheEnd(u8),
}

/// One window the person handed to alo, and the terms they handed it on.
#[derive(Debug, Clone)]
struct HandedOver {
    /// The handle a sender will report under.
    handle: ReportedWindow,
    /// The window it stands for, which only this crate maps it to.
    window: WindowId,
    /// The task the person handed it over for.
    task: String,
    /// What the person allowed, which a report is checked against.
    granted: Granted,
}

/// Every window handed to alo, and the reports admitted against them.
#[derive(Debug, Clone, Default)]
pub struct Reports {
    /// One entry per window the person handed over, newest last. A `Vec`
    /// because a person has a handful of windows put aside, not thousands, and
    /// a map would buy nothing a reader could notice.
    handed: Vec<HandedOver>,
}

impl Reports {
    /// Nothing has been handed to alo.
    #[must_use]
    pub const fn new() -> Self {
        Self { handed: Vec::new() }
    }

    /// **The person handed this window to alo**, for this task, on these terms.
    ///
    /// This is the only way an association is created, and the only place a
    /// scope is set. Handing the same window over again replaces the terms,
    /// which is a person changing their mind rather than a sender doing so.
    pub fn handed_over(
        &mut self,
        handle: ReportedWindow,
        window: WindowId,
        task: &str,
        granted: Granted,
    ) {
        self.handed.retain(|held| held.window != window);
        self.handed.push(HandedOver {
            handle,
            window,
            task: task.to_owned(),
            granted,
        });
    }

    /// **The window is no longer alo's**, so the panel draws no agent section.
    ///
    /// Returns the window and the state to show, or [`None`] if it was not
    /// alo's to begin with. `Nothing` rather than `StatusUnavailable`: work
    /// that ended is not work whose status is unknown.
    pub fn taken_back(&mut self, window: WindowId) -> Option<(WindowId, WhatAloIsDoing)> {
        let held = self.handed.iter().any(|held| held.window == window);
        self.handed.retain(|held| held.window != window);
        held.then_some((window, WhatAloIsDoing::Nothing))
    }

    /// **The channel carrying this window's reports is gone.**
    ///
    /// Returns the window and `StatusUnavailable`, so what is on screen stops
    /// claiming the work is under way. The association is kept: the person has
    /// not taken the window back, and a sender that comes back is reporting
    /// about the same task on the same terms.
    ///
    /// [`None`] if this window is not alo's, because there is nothing on screen
    /// making a claim that could have gone stale.
    pub fn the_reporter_is_gone(&self, window: WindowId) -> Option<(WindowId, WhatAloIsDoing)> {
        self.handed
            .iter()
            .any(|held| held.window == window)
            .then_some((window, WhatAloIsDoing::StatusUnavailable))
    }

    /// A report arrived. Check it, and say what the panel should show.
    ///
    /// # Errors
    ///
    /// Every refusal in [`NotAdmitted`], each by name. A refused report changes
    /// nothing: the window keeps whatever it was last shown, because a refusal
    /// is not news about the task.
    pub fn arrived(&self, report: &Reported) -> Result<(WindowId, WhatAloIsDoing), NotAdmitted> {
        let held = self
            .handed
            .iter()
            .find(|held| held.handle == report.window())
            .ok_or(NotAdmitted::NobodyHandedThatOver(report.window().handle()))?;
        if held.task != report.task() {
            return Err(NotAdmitted::ADifferentTask);
        }
        let scope = within(report.may_change(), &held.granted)?;
        let progress = match report.hundredths() {
            None => None,
            Some(hundredths) => {
                Some(Progress::of(hundredths).map_err(|_| NotAdmitted::PastTheEnd(hundredths))?)
            }
        };
        let requires_you = match report.waiting_for() {
            None => RequiresYou::No,
            Some(what) => RequiresYou::Yes(what.to_owned()),
        };
        let work = AtWork::on(
            report.task(),
            scope,
            progress,
            report.last_confirmed(),
            requires_you,
        )?;
        Ok((held.window, WhatAloIsDoing::AtWork(work)))
    }
}

/// The claimed scope as a panel scope, if the grant covers it.
///
/// **Reading is always within changing.** A sender that says it is only reading
/// is claiming less than any grant allows, so it passes against both.
fn within(claimed: &MayChange, granted: &Granted) -> Result<Scope, NotAdmitted> {
    match (claimed, granted) {
        (MayChange::NothingYet, _) => Ok(Scope::ReadsOnly),
        (MayChange::TheseFiles(_), Granted::ReadingOnly) => Err(NotAdmitted::MoreThanWasGranted),
        (MayChange::TheseFiles(claimed), Granted::TheseFiles(allowed)) => claimed
            .iter()
            .all(|path| allowed.contains(path))
            .then(|| Scope::MayChange(claimed.clone()))
            .ok_or(NotAdmitted::MoreThanWasGranted),
    }
}

#[cfg(test)]
mod tests;
