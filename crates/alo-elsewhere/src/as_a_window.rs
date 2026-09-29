//! A machine the person added, on the canvas, as a window.
//!
//! # There is no machine-window type, and that is the design
//!
//! A machine's window is an ordinary [`alo_dock::Window`]. This file adds one
//! thing and only one: **a note saying which machine a window is showing.**
//!
//! The alternative — a `MachineWindow` of its own — is what the design note
//! warns against by implication when it says *a machine is one more window, and
//! the Dock's rule covers it unamended*. A second window model would need its
//! own placement, its own idea of being put aside, its own travelling, and its
//! own answer to what a click does; and the day those four answers drifted from
//! the Dock's, a person would find that one kind of window behaved differently
//! for no reason they could see.
//!
//! So [`WhichMachines`] is a side table from [`WindowId`] to
//! [`alo_nearby::MachineId`], and everything else about the window is the
//! Dock's: it sits [`alo_dock::HowItSits::OnTheCanvas`], is
//! [`alo_dock::HowItSits::PutAside`], or is
//! [`alo_dock::HowItSits::FillingTheScreen`], exactly as anything else does.
//!
//! # One icon per machine
//!
//! [`an_app_for`] gives each machine a stable [`AppId`], so the Dock groups a
//! machine's windows the way it groups an application's — and the owner's rule
//! then reads correctly without being rewritten: *click an app to return to
//! where you last used it* becomes *click a machine to return to where you last
//! were on it*.
//!
//! One identifier for all machines together was rejected: it would put every
//! machine behind one icon, so returning to *the one in the studio* would mean
//! picking it out of a preview list every time, which is precisely the case the
//! Dock's rule exists to spare a person.

use alo_dock::{AppId, NotAnApp, WindowId};
use alo_nearby::MachineId;

/// What an [`AppId`] made for a machine begins with.
///
/// A prefix rather than the bare identity, so a machine's icon can never
/// collide with an application whose name happens to be thirty-two hexadecimal
/// characters, and so anything reading a list of identifiers can tell which
/// ones are machines without a second table.
pub const A_MACHINES_APP: &str = "machine.";

/// The identifier the Dock groups one machine's windows under.
///
/// Stable for a machine and different for every other, which is what makes
/// *return to where you last were on that machine* mean anything.
///
/// # Errors
///
/// [`NotAnApp`] if the identity produced an empty name, which cannot happen for
/// an identity `alo-nearby` made — it is returned rather than unwrapped because
/// a library that panics on its own identifier takes the shell with it.
pub fn an_app_for(machine: &MachineId) -> Result<AppId, NotAnApp> {
    AppId::named(&format!("{A_MACHINES_APP}{}", machine.as_str()))
}

/// Which window is showing which of the person's machines.
///
/// Only windows that are showing a machine are in here. A window absent from
/// this table is an ordinary local window, and that is the whole meaning of its
/// absence.
#[derive(Debug, Default, Clone)]
pub struct WhichMachines {
    /// One row per window showing a machine, in the order they opened.
    showing: Vec<(WindowId, MachineId)>,
}

impl WhichMachines {
    /// No window is showing a machine.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            showing: Vec::new(),
        }
    }

    /// Say that this window is showing this machine.
    ///
    /// Answers `false` if that window is already showing one, and changes
    /// nothing. A window shows one machine for as long as it exists: silently
    /// repointing it would move a person from one machine to another under a
    /// title that did not change.
    pub fn opened(&mut self, window: WindowId, machine: MachineId) -> bool {
        if self.is_elsewhere(window) {
            return false;
        }
        self.showing.push((window, machine));
        true
    }

    /// The window has closed.
    pub fn closed(&mut self, window: WindowId) -> bool {
        let was = self.showing.len();
        self.showing.retain(|(each, _)| *each != window);
        self.showing.len() != was
    }

    /// Which machine this window is showing, if it is showing one.
    #[must_use]
    pub fn machine_of(&self, window: WindowId) -> Option<&MachineId> {
        self.showing
            .iter()
            .find(|(each, _)| *each == window)
            .map(|(_, machine)| machine)
    }

    /// Whether this window is showing a machine rather than something here.
    ///
    /// **The question slice six is built on.** A window that is elsewhere has
    /// to say so, and has to keep saying so when it fills the screen.
    #[must_use]
    pub fn is_elsewhere(&self, window: WindowId) -> bool {
        self.machine_of(window).is_some()
    }

    /// Every window showing this machine, in the order they opened.
    pub fn windows_of<'a>(&'a self, machine: &MachineId) -> impl Iterator<Item = WindowId> + 'a {
        let machine = machine.clone();
        self.showing
            .iter()
            .filter(move |(_, each)| *each == machine)
            .map(|(window, _)| *window)
    }

    // **There is deliberately no `forget_machine` here, and there was.**
    //
    // The first version of this file removed a machine's rows when the person
    // removed the machine from their list. Building [`crate::Marked`] showed
    // what that did: a window still showing another machine fell out of this
    // table, and the next question about it answered *here*. **A remote window
    // that looks local is the exact failure the marking exists to prevent**, and
    // this method was the shortest road to it.
    //
    // A window leaves this table when the window closes, through
    // [`Self::closed`], and by no other road. A machine the person removed is
    // answered by [`crate::Marked::ElsewhereUnnamed`]: still elsewhere, still
    // said, without a name to say it with.

    /// How many windows are showing a machine.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.showing.len()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_dock::{
        HowItSits, Patch, Spot, TheView, WhatAClickDoes, Window, Windows, what_a_click_does,
    };

    use super::*;

    fn identity(last: char) -> MachineId {
        let mut said = "0123456789abcdef0123456789abcde".to_owned();
        said.push(last);
        MachineId::read(&said).expect("thirty-two hexadecimal characters")
    }

    fn a_patch(x: i64) -> Patch {
        Patch::of(Spot::at(x, 0), 800, 600).expect("a patch with extent")
    }

    /// **A machine's icon is stable and its own.** Stable, or returning to
    /// where you last were on a machine means nothing; its own, or two machines
    /// share one icon.
    #[test]
    fn each_machine_has_its_own_stable_identifier() {
        let first = an_app_for(&identity('a')).expect("an identifier");
        let again = an_app_for(&identity('a')).expect("an identifier");
        let other = an_app_for(&identity('b')).expect("an identifier");

        assert_eq!(first, again, "the same machine gave two identifiers");
        assert_ne!(first, other, "two machines share one identifier");
        assert!(first.name().starts_with(A_MACHINES_APP));
    }

    /// **The Dock's rule covers a machine unamended.** This is the claim the
    /// design note makes, tested through the Dock's own function rather than
    /// restated: a machine's window is found, and a click travels to it exactly
    /// as it would for an application.
    #[test]
    fn the_docks_own_rule_answers_for_a_machine_window() {
        let machine = an_app_for(&identity('a')).expect("an identifier");
        let mut windows = Windows::none();
        windows.opened(Window::of(
            WindowId::numbered(1),
            machine.clone(),
            "the one in the studio",
            a_patch(5_000),
            HowItSits::OnTheCanvas,
        ));

        // The view is somewhere else on the plane, so the canvas must travel.
        let view = TheView::showing(a_patch(0));
        assert_eq!(
            what_a_click_does(&windows, &machine, view),
            WhatAClickDoes::TravelTo(WindowId::numbered(1)),
            "the Dock did not treat a machine like anything else it holds"
        );
    }

    /// **A machine sits every way a window can sit**, which is what *one more
    /// window* means. Moved through all three with the Dock's own type.
    #[test]
    fn a_machine_window_sits_every_way_a_window_can() {
        let machine = an_app_for(&identity('a')).expect("an identifier");
        let mut windows = Windows::none();
        windows.opened(Window::of(
            WindowId::numbered(1),
            machine,
            "the one at home",
            a_patch(0),
            HowItSits::OnTheCanvas,
        ));

        for sits in [
            HowItSits::PutAside,
            HowItSits::FillingTheScreen,
            HowItSits::OnTheCanvas,
        ] {
            assert!(windows.now_sits(WindowId::numbered(1), sits));
            assert_eq!(
                windows
                    .window(WindowId::numbered(1))
                    .expect("the window")
                    .sits(),
                sits
            );
        }
    }

    /// A window absent from the table is an ordinary window here, and that is
    /// the entire meaning of its absence.
    #[test]
    fn a_window_that_is_not_listed_is_not_elsewhere() {
        let mut which = WhichMachines::none();
        assert!(!which.is_elsewhere(WindowId::numbered(1)));

        which.opened(WindowId::numbered(1), identity('a'));
        assert!(which.is_elsewhere(WindowId::numbered(1)));
        assert!(!which.is_elsewhere(WindowId::numbered(2)));
    }

    /// **A window shows one machine for as long as it exists.** Repointing it
    /// would move somebody from one machine to another under a title that did
    /// not change.
    #[test]
    fn a_window_cannot_be_repointed_at_another_machine() {
        let mut which = WhichMachines::none();
        assert!(which.opened(WindowId::numbered(1), identity('a')));
        assert!(!which.opened(WindowId::numbered(1), identity('b')));

        assert_eq!(
            which.machine_of(WindowId::numbered(1)),
            Some(&identity('a'))
        );
        assert_eq!(which.how_many(), 1);
    }

    /// Several windows onto one machine, which is what makes it a machine
    /// rather than a screenshot.
    #[test]
    fn one_machine_may_have_several_windows() {
        let mut which = WhichMachines::none();
        which.opened(WindowId::numbered(1), identity('a'));
        which.opened(WindowId::numbered(2), identity('a'));
        which.opened(WindowId::numbered(3), identity('b'));

        let its: Vec<_> = which.windows_of(&identity('a')).collect();
        assert_eq!(its, vec![WindowId::numbered(1), WindowId::numbered(2)]);
    }

    /// **A window leaves this table when the window closes, and by no other
    /// road.** Removing a machine from the person's list must not take its
    /// windows out of here: one that did would answer *here* for a window still
    /// showing somewhere else, which is the failure `Marked` exists to prevent.
    #[test]
    fn only_closing_a_window_takes_it_out_of_the_table() {
        let mut which = WhichMachines::none();
        which.opened(WindowId::numbered(1), identity('a'));
        which.opened(WindowId::numbered(2), identity('b'));

        assert!(which.closed(WindowId::numbered(1)));
        assert!(!which.is_elsewhere(WindowId::numbered(1)));
        assert!(
            which.is_elsewhere(WindowId::numbered(2)),
            "an untouched window stopped saying where it is"
        );
    }

    /// Closing says whether there was one, so a caller can tell *done* from
    /// *there was nothing there*.
    #[test]
    fn closing_says_whether_there_was_one() {
        let mut which = WhichMachines::none();
        which.opened(WindowId::numbered(1), identity('a'));
        assert!(which.closed(WindowId::numbered(1)));
        assert!(!which.closed(WindowId::numbered(1)));
        assert_eq!(which.how_many(), 0);
    }
}
