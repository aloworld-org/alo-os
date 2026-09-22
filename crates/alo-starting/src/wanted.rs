//! What a person approved about which system this machine starts next.

use alo_broker::{Identity, SystemVerb};

/// The one change to the machine that is on this road.
///
/// One rather than two, and deliberately: **which system the machine starts by
/// default is not here.** That is the loader's own saved choice
/// ([`crate::TheStartingChoice`]), written where the loader writes it, and a
/// verb of its own over it would be a second road to one fact —
/// [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)'s
/// third term is exactly the refusal of that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Start the Windows on this machine the next time it starts, once, and
    /// leave everything else about how it starts as it was.
    RestartIntoWindows,
}

impl Change {
    /// The broker's verb for this, about the start-up entry with this identity.
    ///
    /// The identity is the digest of what the firmware reported for the entry
    /// (`alo_starting::Entry::as_reported`), which is what whoever carries it
    /// out digests again from what the firmware reports at that moment.
    #[must_use]
    pub const fn to(self, identity: Identity) -> SystemVerb {
        match self {
            Self::RestartIntoWindows => SystemVerb::RestartIntoWindows(identity),
        }
    }

    /// Which of them the broker's verb is, if it is one of this road's.
    #[must_use]
    pub const fn of(verb: &SystemVerb) -> Option<Self> {
        match verb {
            SystemVerb::RestartIntoWindows(_) => Some(Self::RestartIntoWindows),
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
    use alo_broker::Switch;

    /// A start-up entry, as a digest of one.
    fn an_entry() -> Identity {
        Identity::of_what_was_reported(b"a start-up entry")
    }

    /// **It reads back from its verb, and no other verb is this one.**
    #[test]
    fn it_reads_back_and_no_other_verb_is_this_road() {
        assert_eq!(
            Change::of(&Change::RestartIntoWindows.to(an_entry())),
            Some(Change::RestartIntoWindows)
        );
        for verb in SystemVerb::one_of_each(an_entry(), Switch::On) {
            let is_this_road = matches!(verb, SystemVerb::RestartIntoWindows(_));
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
}
