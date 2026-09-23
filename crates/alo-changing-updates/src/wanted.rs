//! Which of the two changes to this machine's system a person approved.

use alo_broker::{Identity, SystemVerb};

/// What a person approved about their machine's system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Apply an update that has been staged.
    Apply,
    /// Go back to the version the machine ran before its last update.
    GoBack,
}

impl Change {
    /// The broker's verb for this, about the build with this identity.
    ///
    /// The identity is the digest of what a person approved — for applying an
    /// update, the pair of builds `alo_keeping_up::Staging::approved` decides
    /// from; for going back, the machine as it was when going back was offered.
    #[must_use]
    pub const fn to(self, identity: Identity) -> SystemVerb {
        match self {
            Self::Apply => SystemVerb::ApplyStagedUpdate(identity),
            Self::GoBack => SystemVerb::RollBack(identity),
        }
    }

    /// Which of the two the broker's verb is, if it is one of the updates'.
    #[must_use]
    pub const fn of(verb: &SystemVerb) -> Option<Self> {
        match verb {
            SystemVerb::ApplyStagedUpdate(_) => Some(Self::Apply),
            SystemVerb::RollBack(_) => Some(Self::GoBack),
            SystemVerb::AddPrinter(_)
            | SystemVerb::RemovePrinter(_)
            | SystemVerb::SetDefaultPrinter(_)
            | SystemVerb::JoinNetwork(_)
            | SystemVerb::ForgetNetwork(_)
            | SystemVerb::SetRadio(_)
            | SystemVerb::SetProxy(_)
            | SystemVerb::MountDrive(_)
            | SystemVerb::EjectDrive(_)
            | SystemVerb::RestartIntoWindows(_)
            | SystemVerb::StartByDefault(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_broker::Switch;

    /// A build, as a digest of one.
    fn a_build() -> Identity {
        Identity::of_what_was_reported(b"a build")
    }

    /// **Both read back from their verb, and no other verb is one of these.**
    #[test]
    fn both_read_back_and_no_other_verb_is_an_updates() {
        for change in [Change::Apply, Change::GoBack] {
            assert_eq!(Change::of(&change.to(a_build())), Some(change));
        }
        for verb in SystemVerb::one_of_each(a_build(), Switch::Off) {
            let is_an_updates = matches!(
                verb,
                SystemVerb::ApplyStagedUpdate(_) | SystemVerb::RollBack(_)
            );
            assert_eq!(
                Change::of(&verb).is_some(),
                is_an_updates,
                "{}",
                verb.name()
            );
        }
    }

    /// **Applying and going back are not the same verb**, so an approval of one
    /// is never redeemed as the other — which, for these two, is the difference
    /// between a machine that moves forward and one that moves back.
    #[test]
    fn applying_and_going_back_are_different_verbs() {
        assert_ne!(Change::Apply.to(a_build()), Change::GoBack.to(a_build()));
    }
}
