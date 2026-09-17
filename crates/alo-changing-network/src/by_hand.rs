//! A person's own change to the network, in Settings — through the same broker
//! verbs an agent's approved proposal becomes.
//!
//! ADR 0009: *anything an agent verb can do, a person must also be able to do
//! by hand*, and `docs/features.md` names the network pane of *Settings, as one
//! place* as the plain way. So Settings lists networks ([`Listed`]), a person
//! picks one and the change they picked, and that becomes **the broker's own
//! verb** — identical to what an agent's approved proposal becomes, crossing
//! the same door, written into the same record. There is no second road into
//! the machine's network settings for a surface to take instead.
//!
//! A person picks a network by what Settings shows, not by typing its name, so
//! a network whose name is not text a sentence could hold is still one a person
//! can join. The person's click is the approval: there is no turn behind it and
//! no proposal to number, so the token is issued under `alo_broker::BY_HAND`,
//! and the broker's record says a person made this change themselves.

use alo_broker::{Identity, SystemVerb};
use alo_networks::{NetworkName, Protection, TheNetworks};

/// A network as a person chooses among them in Settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    /// What it announces itself as.
    name: NetworkName,
    /// How it is protected, for a network in range; nothing for a saved one.
    protection: Option<Protection>,
    /// The broker's verb for the change a person makes by picking it.
    verb: SystemVerb,
}

impl Listed {
    /// Every network in range, each joined by picking it.
    #[must_use]
    pub fn in_range(networks: &TheNetworks) -> Vec<Self> {
        networks
            .visible
            .iter()
            .map(|network| Self {
                name: network.name().clone(),
                protection: Some(network.protection()),
                verb: SystemVerb::JoinNetwork(Identity::of_what_was_reported(
                    &network.as_reported(),
                )),
            })
            .collect()
    }

    /// Every saved network, each forgotten by picking it.
    #[must_use]
    pub fn saved(networks: &TheNetworks) -> Vec<Self> {
        networks
            .saved
            .iter()
            .map(|network| Self {
                name: network.name().clone(),
                protection: None,
                verb: SystemVerb::ForgetNetwork(Identity::of_what_was_reported(
                    &network.as_reported(),
                )),
            })
            .collect()
    }

    /// What it announces itself as.
    #[must_use]
    pub const fn name(&self) -> &NetworkName {
        &self.name
    }

    /// How it is protected, for a network in range.
    #[must_use]
    pub const fn protection(&self) -> Option<Protection> {
        self.protection
    }

    /// The broker's verb for picking it: joining it if it is in range,
    /// forgetting it if it is saved.
    #[must_use]
    pub const fn picked(&self) -> SystemVerb {
        self.verb
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::choosing::chosen;
    use crate::wanted::Change;
    use alo_networks::{Saved, Visible};

    /// **A person's pick is the very verb an agent's approved proposal for the
    /// same network becomes**, and a network no sentence could name can still
    /// be picked.
    #[test]
    fn a_pick_in_settings_is_the_same_verb_a_proposal_becomes() {
        let named = |name: &[u8]| NetworkName::announced(name).unwrap();
        let networks = TheNetworks {
            visible: vec![
                Visible::reported(named(b"Home"), Protection::Password, 70),
                Visible::reported(named(&[0xff, 0x00, 0x01]), Protection::Open, 30),
            ],
            saved: vec![Saved::reported(named(b"Office"), "office")],
            wireless_on: true,
            primary: None,
        };
        let in_range = Listed::in_range(&networks);
        assert_eq!(
            in_range.first().map(Listed::picked),
            chosen(&Change::Join("Home".into()), &networks).ok()
        );
        assert_eq!(
            Listed::saved(&networks).first().map(Listed::picked),
            chosen(&Change::Forget("Office".into()), &networks).ok()
        );
        let unnameable = in_range.get(1).unwrap();
        assert!(unnameable.name().called().is_none());
        assert!(matches!(unnameable.picked(), SystemVerb::JoinNetwork(_)));
    }
}
