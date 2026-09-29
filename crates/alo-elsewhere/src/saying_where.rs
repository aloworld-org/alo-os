//! What a window carries so a person can tell it is not this machine.
//!
//! # The cost of getting this wrong
//!
//! `docs/design/a-persons-other-machines.md`: *a window that is elsewhere can be
//! mistaken for one that is here, and the cost is a password typed into the
//! wrong machine.*
//!
//! # It is a word, never a colour
//!
//! `docs/design/who-is-acting.md` settles the neighbouring question — telling
//! alo apart from the person — and its rule is taken whole here rather than
//! restated in a second form: **a signal carried by hue alone is not a signal**
//! ([ADR 0010](../../../docs/decisions/0010-terracotta-is-reserved-and-never-alone.md)).
//! Around one man in twelve cannot rely on hue, and EN 301 549 has required
//! otherwise for twenty years.
//!
//! So [`Marked::Elsewhere`] carries **the machine's name**, which is a word a
//! person reads and a screen reader speaks. There is no colour in this file and
//! no flag meaning *tint it*. Whatever a shell paints is confirmation beside the
//! word, never the signal itself.
//!
//! # It survives the window filling the screen
//!
//! [`Marked::is_shown_while`] answers true for every way a window can sit, and a
//! test walks all three. That looks like a function that does nothing, and it is
//! the point: **there is no state in which this marking is suppressed, and no
//! argument a caller could pass to suppress it.**
//!
//! Full screen is exactly when it matters. Every other cue — the Dock, the
//! window's own frame, the canvas around it — is gone, and what is left is a
//! screen showing a machine. A marking that faded there would be absent in the
//! one case it exists for.
//!
//! # A machine the person removed
//!
//! A window whose machine is no longer on the person's list still says it is
//! elsewhere, with no name. **It must never fall back to looking local**, which
//! is the failure this whole file exists to prevent, and it would be the easiest
//! one to write by accident.

use alo_dock::{HowItSits, WindowId};

use crate::as_a_window::WhichMachines;
use crate::machine::TheName;
use crate::machines::TheMachines;
use crate::words;

/// What a window says about where it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marked {
    /// This machine's own window. It carries nothing, because *here* is the
    /// ordinary case and marking it would make the marking meaningless.
    Here,
    /// It is showing another of the person's machines, named.
    Elsewhere {
        /// What the person calls that machine.
        called: TheName,
    },
    /// It is showing a machine the person has since removed from their list.
    ///
    /// Still elsewhere, still said, and deliberately **not** [`Self::Here`].
    ElsewhereUnnamed,
}

impl Marked {
    /// What this window must say about where it is.
    #[must_use]
    pub fn of(which: &WhichMachines, machines: &TheMachines, window: WindowId) -> Self {
        let Some(machine) = which.machine_of(window) else {
            return Self::Here;
        };
        machines
            .each()
            .find(|(each, _)| each.which() == machine)
            .map_or(Self::ElsewhereUnnamed, |(each, _)| Self::Elsewhere {
                called: each.called().clone(),
            })
    }

    /// Whether this window is showing another machine.
    #[must_use]
    pub const fn is_elsewhere(&self) -> bool {
        matches!(self, Self::Elsewhere { .. } | Self::ElsewhereUnnamed)
    }

    /// Whether the marking is shown while a window sits this way.
    ///
    /// **Always, and that is the contract.** There is no way a caller can ask
    /// for it to be hidden, because full screen — when every other cue is gone —
    /// is the one case it exists for.
    #[must_use]
    pub const fn is_shown_while(&self, _sits: HowItSits) -> bool {
        self.is_elsewhere()
    }

    /// The key of the sentence a person reads, if this window says anything.
    ///
    /// [`Self::Here`] answers `None`: a local window says nothing about where it
    /// is, because every window saying *here* would leave a person reading the
    /// word rather than noticing the one that differs.
    #[must_use]
    pub const fn word(&self) -> Option<&'static str> {
        match self {
            Self::Here => None,
            Self::Elsewhere { .. } => Some(words::ON_MACHINE),
            Self::ElsewhereUnnamed => Some(words::ON_ANOTHER_MACHINE),
        }
    }

    /// The machine's name, to fill the sentence with.
    #[must_use]
    pub fn called(&self) -> Option<&TheName> {
        match self {
            Self::Elsewhere { called } => Some(called),
            Self::Here | Self::ElsewhereUnnamed => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_nearby::MachineId;

    use super::*;
    use crate::machine::AMachine;

    fn identity(last: char) -> MachineId {
        let mut said = "0123456789abcdef0123456789abcde".to_owned();
        said.push(last);
        MachineId::read(&said).expect("thirty-two hexadecimal characters")
    }

    fn a_name(called: &str) -> TheName {
        TheName::given(called).expect("a name")
    }

    fn one_added() -> TheMachines {
        let mut machines = TheMachines::none_yet();
        machines
            .add(AMachine::added(identity('a'), a_name("in the studio")))
            .expect("added");
        machines
    }

    fn showing(window: u64, machine: char) -> WhichMachines {
        let mut which = WhichMachines::none();
        which.opened(WindowId::numbered(window), identity(machine));
        which
    }

    /// A window of this machine's own says nothing about where it is — every
    /// window saying *here* would leave a person reading the word rather than
    /// noticing the one that differs.
    #[test]
    fn a_local_window_says_nothing() {
        let marked = Marked::of(&WhichMachines::none(), &one_added(), WindowId::numbered(1));
        assert_eq!(marked, Marked::Here);
        assert!(!marked.is_elsewhere());
        assert_eq!(marked.word(), None);
    }

    /// **It is a word, never a colour.** The marking carries the name the
    /// person gave the machine, which a screen reader can speak.
    #[test]
    fn a_window_elsewhere_carries_the_machines_name() {
        let marked = Marked::of(&showing(1, 'a'), &one_added(), WindowId::numbered(1));
        assert_eq!(
            marked,
            Marked::Elsewhere {
                called: a_name("in the studio")
            }
        );
        assert_eq!(
            marked.called().expect("a name").as_str(),
            "in the studio",
            "the marking lost the word and would have to rely on a colour"
        );
        assert_eq!(marked.word(), Some("elsewhere.window.on-machine"));
    }

    /// **It survives the window filling the screen**, which is the one case it
    /// exists for: every other cue is gone and what is left is a screen showing
    /// a machine. Walked over all three ways a window can sit.
    #[test]
    fn the_marking_survives_every_way_a_window_can_sit() {
        let marked = Marked::of(&showing(1, 'a'), &one_added(), WindowId::numbered(1));
        for sits in [
            HowItSits::OnTheCanvas,
            HowItSits::PutAside,
            HowItSits::FillingTheScreen,
        ] {
            assert!(
                marked.is_shown_while(sits),
                "a window elsewhere stopped saying so while {sits:?}"
            );
        }
    }

    /// And a local window has nothing to show in any of them, which is the
    /// other half of the same answer.
    #[test]
    fn a_local_window_shows_nothing_in_any_state() {
        for sits in [
            HowItSits::OnTheCanvas,
            HowItSits::PutAside,
            HowItSits::FillingTheScreen,
        ] {
            assert!(!Marked::Here.is_shown_while(sits));
        }
    }

    /// **A machine the person removed does not make its window look local.**
    /// The easiest failure to write by accident, and the worst one to ship.
    #[test]
    fn a_window_of_a_removed_machine_still_says_it_is_elsewhere() {
        let mut machines = one_added();
        assert!(machines.forget(&identity('a')));

        let marked = Marked::of(&showing(1, 'a'), &machines, WindowId::numbered(1));
        assert_eq!(marked, Marked::ElsewhereUnnamed);
        assert!(
            marked.is_elsewhere(),
            "a removed machine made its window look like this one"
        );
        assert_eq!(marked.word(), Some("elsewhere.window.on-another-machine"));
        assert_eq!(marked.called(), None);
        assert!(marked.is_shown_while(HowItSits::FillingTheScreen));
    }

    /// A window showing a machine the person never added is in the same
    /// position, and answers the same way.
    #[test]
    fn a_window_of_a_machine_never_added_is_still_elsewhere() {
        let marked = Marked::of(&showing(1, 'b'), &one_added(), WindowId::numbered(1));
        assert_eq!(marked, Marked::ElsewhereUnnamed);
    }

    /// Two windows onto two machines each say their own machine's name.
    #[test]
    fn two_machines_are_told_apart_by_name() {
        let mut machines = one_added();
        machines
            .add(AMachine::added(identity('b'), a_name("at home")))
            .expect("added");

        let mut which = WhichMachines::none();
        which.opened(WindowId::numbered(1), identity('a'));
        which.opened(WindowId::numbered(2), identity('b'));

        let first = Marked::of(&which, &machines, WindowId::numbered(1));
        let second = Marked::of(&which, &machines, WindowId::numbered(2));
        assert_ne!(first, second);
        assert_eq!(first.called().expect("a name").as_str(), "in the studio");
        assert_eq!(second.called().expect("a name").as_str(), "at home");
    }
}
