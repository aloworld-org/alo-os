//! What a person approved about which system this machine starts.

use alo_broker::{Identity, SystemVerb};

/// One of the two changes to the machine that are on this road.
///
/// Two, and they are two on purpose: **the next start and the default are
/// different acts.** One is *take me across once and change nothing about this
/// computer*; the other is *this is what this computer does from now on*. One
/// verb doing both would be a sentence a person approved that meant two things
/// ([ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
/// *what stays as it was*, and
/// [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md),
/// *what this does not decide*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Start the Windows on this machine the next time it starts, once, and
    /// leave everything else about how it starts as it was.
    RestartIntoWindows,
    /// Start one of the two systems whenever nobody chooses at the menu, from
    /// now on. Which one is the verb's own argument
    /// ([`crate::to_start_by_default`]).
    StartByDefault,
}

impl Change {
    /// The broker's verb for this change, about the thing with this identity.
    ///
    /// For [`Change::RestartIntoWindows`] the identity is the digest of what
    /// the firmware reported for a start-up entry
    /// ([`crate::Entry::as_reported`]); for [`Change::StartByDefault`] it is
    /// the identity of one of the two systems
    /// ([`crate::the_identity_of`]), and [`crate::to_start_by_default`] is the
    /// way to make that one without digesting anything by hand. Either way it
    /// is what whoever carries the verb out compares against what this machine
    /// reports at that moment.
    #[must_use]
    pub const fn to(self, identity: Identity) -> SystemVerb {
        match self {
            Self::RestartIntoWindows => SystemVerb::RestartIntoWindows(identity),
            Self::StartByDefault => SystemVerb::StartByDefault(identity),
        }
    }

    /// Which of them the broker's verb is, if it is one of this road's.
    #[must_use]
    pub const fn of(verb: &SystemVerb) -> Option<Self> {
        match verb {
            SystemVerb::RestartIntoWindows(_) => Some(Self::RestartIntoWindows),
            SystemVerb::StartByDefault(_) => Some(Self::StartByDefault),
            SystemVerb::AddPrinter(_)
            | SystemVerb::RemovePrinter(_)
            | SystemVerb::SetDefaultPrinter(_)
            | SystemVerb::JoinNetwork(_)
            | SystemVerb::ForgetNetwork(_)
            | SystemVerb::SetRadio(_)
            | SystemVerb::SetProxy(_)
            | SystemVerb::ApplyStagedUpdate(_)
            | SystemVerb::RollBack(_)
            | SystemVerb::MountDrive(_)
            | SystemVerb::EjectDrive(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offered::the_identity_of;
    use crate::systems::System;
    use alo_broker::Switch;

    /// A start-up entry, as a digest of one.
    fn an_entry() -> Identity {
        Identity::of_what_was_reported(b"a start-up entry")
    }

    /// **Each reads back from its verb, and no other verb is this road's.**
    #[test]
    fn each_reads_back_and_no_other_verb_is_this_road() {
        assert_eq!(
            Change::of(&Change::RestartIntoWindows.to(an_entry())),
            Some(Change::RestartIntoWindows)
        );
        assert_eq!(
            Change::of(&Change::StartByDefault.to(the_identity_of(System::Windows))),
            Some(Change::StartByDefault)
        );
        for verb in SystemVerb::one_of_each(an_entry(), Switch::On) {
            let is_this_road = matches!(
                verb,
                SystemVerb::RestartIntoWindows(_) | SystemVerb::StartByDefault(_)
            );
            assert_eq!(Change::of(&verb).is_some(), is_this_road, "{}", verb.name());
        }
    }

    /// **The verb carries the entry it was approved for and nothing else**, so
    /// an approval for one machine's Windows is not an approval for whatever is
    /// in that slot afterwards.
    #[test]
    fn the_verb_carries_the_entry_it_was_approved_for() {
        let one = Change::RestartIntoWindows.to(an_entry());
        let another =
            Change::RestartIntoWindows.to(Identity::of_what_was_reported(b"another entry"));
        assert_ne!(one, another);
        assert_eq!(one.name(), "starting.windows-next");
    }

    /// **The two changes are never one another**, even over the same identity:
    /// an approval to go across once is not an approval to change what this
    /// computer does from now on.
    #[test]
    fn going_across_once_is_never_changing_the_default() {
        let identity = the_identity_of(System::Windows);
        let once = Change::RestartIntoWindows.to(identity);
        let always = Change::StartByDefault.to(identity);
        assert_ne!(once, always);
        assert_eq!(once.name(), "starting.windows-next");
        assert_eq!(always.name(), "starting.default");
    }
}
