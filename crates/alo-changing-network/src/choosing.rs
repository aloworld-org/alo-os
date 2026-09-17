//! Which network a change is to: the one network called exactly what the person
//! approved, as the network manager reports it now.
//!
//! Both roads end in the same place — the broker's own verb, holding an
//! `alo_broker::Identity` digested from what the network manager reported — and
//! neither ever produces anything else. Settings draws [`crate::Listed`] networks
//! and a person picks one (`crate::by_hand`); an agent's approved sentence names
//! a network, and [`chosen`] finds the one network listed under exactly that
//! name, or refuses.
//!
//! # A name is matched exactly, once
//!
//! The name a network announces, character for character, against the name the
//! person approved. **No network with that name is no change, and neither is
//! two**: an open network calling itself by the name of the protected one next
//! door is exactly where guessing would join the wrong one, and the refusal
//! sends the person to Settings, where each shows whether it asks for a
//! password. A network asking for an organisation's sign-in is refused by name,
//! since this machine does not set one up in v0.5.

use alo_broker::{Identity, SystemVerb};
use alo_networks::{Protection, TheNetworks};

use crate::refusing::NotChanged;
use crate::wanted::Change;

/// The broker's verb for this change, to the one network called exactly what
/// was approved, among what the network manager reports.
///
/// # Errors
/// [`NotChanged`]: no network has that name, more than one does, or it asks for
/// an organisation's sign-in.
pub fn chosen(change: &Change, networks: &TheNetworks) -> Result<SystemVerb, NotChanged> {
    match change {
        Change::Join(called) => {
            let mut matching = networks
                .visible
                .iter()
                .filter(|network| network.name().called() == Some(called.as_str()));
            match (matching.next(), matching.next()) {
                (Some(one), None) if one.protection() == Protection::Enterprise => {
                    Err(NotChanged::AsksForASignIn(called.clone()))
                }
                (Some(one), None) => Ok(SystemVerb::JoinNetwork(Identity::of_what_was_reported(
                    &one.as_reported(),
                ))),
                (None, _) => Err(NotChanged::NoneVisibleCalled(called.clone())),
                (Some(_), Some(_)) => Err(NotChanged::MoreThanOneCalled(called.clone())),
            }
        }
        Change::Forget(called) => {
            let mut matching = networks
                .saved
                .iter()
                .filter(|network| network.name().called() == Some(called.as_str()));
            match (matching.next(), matching.next()) {
                (Some(one), None) => Ok(SystemVerb::ForgetNetwork(Identity::of_what_was_reported(
                    &one.as_reported(),
                ))),
                (None, _) => Err(NotChanged::NoneSavedCalled(called.clone())),
                (Some(_), Some(_)) => Err(NotChanged::MoreThanOneCalled(called.clone())),
            }
        }
        Change::Wireless(switch) => Ok(SystemVerb::SetRadio(*switch)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_broker::Switch;
    use alo_networks::{NetworkName, Saved, Visible};

    /// A name.
    fn named(name: &str) -> NetworkName {
        NetworkName::announced(name.as_bytes()).unwrap()
    }

    /// A street with a twin, a café and an office network.
    fn a_street() -> TheNetworks {
        TheNetworks {
            visible: vec![
                Visible::reported(named("Home"), Protection::Password, 70),
                Visible::reported(named("Home"), Protection::Open, 95),
                Visible::reported(named("Cafe"), Protection::Open, 40),
                Visible::reported(named("Work"), Protection::Enterprise, 60),
            ],
            saved: vec![
                Saved::reported(named("Cafe"), "cafe"),
                Saved::reported(named("Office"), "office-1"),
                Saved::reported(named("Office"), "office-2"),
            ],
            wireless_on: true,
            primary: None,
        }
    }

    /// **The one network with exactly that name is the one changed.**
    #[test]
    fn the_one_network_with_that_name_is_chosen() {
        let street = a_street();
        let cafe = Identity::of_what_was_reported(&street.visible.get(2).unwrap().as_reported());
        assert_eq!(
            chosen(&Change::Join("Cafe".into()), &street),
            Ok(SystemVerb::JoinNetwork(cafe))
        );
        let saved = Identity::of_what_was_reported(&street.saved.first().unwrap().as_reported());
        assert_eq!(
            chosen(&Change::Forget("Cafe".into()), &street),
            Ok(SystemVerb::ForgetNetwork(saved))
        );
        assert_eq!(
            chosen(&Change::Wireless(Switch::Off), &street),
            Ok(SystemVerb::SetRadio(Switch::Off))
        );
    }

    /// **A twin, a name nobody has, a name two saved networks share, a name
    /// differing only in case, and an organisation's sign-in each change
    /// nothing.**
    #[test]
    fn a_name_nobody_has_or_two_share_changes_nothing() {
        let street = a_street();
        for (change, refused) in [
            (
                Change::Join("Home".into()),
                NotChanged::MoreThanOneCalled("Home".into()),
            ),
            (
                Change::Join("home".into()),
                NotChanged::NoneVisibleCalled("home".into()),
            ),
            (
                Change::Join("Library".into()),
                NotChanged::NoneVisibleCalled("Library".into()),
            ),
            (
                Change::Join("Work".into()),
                NotChanged::AsksForASignIn("Work".into()),
            ),
            (
                Change::Forget("Office".into()),
                NotChanged::MoreThanOneCalled("Office".into()),
            ),
            (
                Change::Forget("Home".into()),
                NotChanged::NoneSavedCalled("Home".into()),
            ),
        ] {
            assert_eq!(chosen(&change, &street), Err(refused), "{change:?}");
        }
    }
}
