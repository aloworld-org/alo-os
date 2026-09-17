//! Whether a change to the network cuts the connection this conversation is
//! answered over.
//!
//! The broker plan: **a network change that would cut a turn's own connection
//! says so before it is approved.** A person approving *turn Wi-Fi off* while
//! the assistant is answered by a provider over that Wi-Fi would otherwise
//! watch the conversation stop mid-sentence and not know why. So every network
//! verb carries what it does to the conversation as an argument its sentence
//! names ([`crate::verbs`]), and this is the one place that works out what the
//! true answer is, from what the network manager reports and from where the
//! conversation is answered. `crate::proposing` refuses a proposal that says
//! anything else, and `crate::carrying_out` refuses to carry out an approval
//! once it is no longer true.
//!
//! # When a change cuts it
//!
//! Only when the conversation is answered **over the network**. A conversation
//! answered by a model on this machine keeps going whatever the Wi-Fi does, and
//! saying otherwise would be a warning that teaches people to ignore warnings.
//! Then:
//!
//! | change | cuts the conversation's connection when |
//! |---|---|
//! | join a network | the machine sends through Wi-Fi now, over a different network |
//! | forget a network | the machine sends through that network now |
//! | turn Wi-Fi off | the machine sends through Wi-Fi now |
//! | turn Wi-Fi on | never |
//!
//! A machine sending through a cable keeps its connection through every Wi-Fi
//! change, and a machine with no connection has none to lose.

use alo_broker::Switch;
use alo_networks::TheNetworks;

use crate::wanted::{Change, ThisConversation};

/// Where the conversation making a change is answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answered {
    /// By a model on this machine.
    OnThisMachine,
    /// By a model reached over the network: a provider, or a machine nearby.
    OverTheNetwork,
}

/// What this change does to a conversation answered so, on a machine whose
/// network manager reports this.
#[must_use]
pub fn would(change: &Change, networks: &TheNetworks, answered: Answered) -> ThisConversation {
    let cut = answered == Answered::OverTheNetwork
        && networks.primary.as_ref().is_some_and(|primary| {
            let joined_is_called = |called: &str| {
                networks.joined().and_then(|joined| joined.name().called()) == Some(called)
            };
            match change {
                Change::Join(called) => primary.over_wireless() && !joined_is_called(called),
                Change::Forget(called) => joined_is_called(called),
                Change::Wireless(Switch::Off) => primary.over_wireless(),
                Change::Wireless(Switch::On) => false,
            }
        });
    if cut {
        ThisConversation::LosesItsConnection
    } else {
        ThisConversation::KeepsItsConnection
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_networks::{NetworkName, Primary, Saved};

    /// A machine sending through the saved network Home, over Wi-Fi or a cable.
    fn at_home(over_wireless: bool) -> TheNetworks {
        let named = |name: &str| NetworkName::announced(name.as_bytes()).unwrap();
        TheNetworks {
            visible: Vec::new(),
            saved: vec![
                Saved::reported(named("Home"), "home"),
                Saved::reported(named("Office"), "office"),
            ],
            wireless_on: true,
            primary: Some(Primary::reported(
                over_wireless,
                if over_wireless { "home" } else { "cable" },
            )),
        }
    }

    /// **Answered over the network, on Wi-Fi: every change that takes the
    /// machine off its Wi-Fi network says so, and nothing else does.**
    #[test]
    fn on_wifi_a_change_that_leaves_the_network_cuts_the_conversation() {
        let home = at_home(true);
        let over = Answered::OverTheNetwork;
        let loses = ThisConversation::LosesItsConnection;
        let keeps = ThisConversation::KeepsItsConnection;
        assert_eq!(would(&Change::Join("Office".into()), &home, over), loses);
        assert_eq!(would(&Change::Join("Home".into()), &home, over), keeps);
        assert_eq!(would(&Change::Forget("Home".into()), &home, over), loses);
        assert_eq!(would(&Change::Forget("Office".into()), &home, over), keeps);
        assert_eq!(would(&Change::Wireless(Switch::Off), &home, over), loses);
        assert_eq!(would(&Change::Wireless(Switch::On), &home, over), keeps);
    }

    /// **On a cable, or answered on this machine, or with no connection at all,
    /// no Wi-Fi change cuts the conversation.**
    #[test]
    fn on_a_cable_or_answered_here_nothing_cuts_the_conversation() {
        let changes = [
            Change::Join("Office".into()),
            Change::Forget("Home".into()),
            Change::Wireless(Switch::Off),
        ];
        let unplugged = TheNetworks {
            primary: None,
            ..at_home(true)
        };
        for change in &changes {
            for (networks, answered) in [
                (at_home(false), Answered::OverTheNetwork),
                (at_home(true), Answered::OnThisMachine),
                (unplugged.clone(), Answered::OverTheNetwork),
            ] {
                assert_eq!(
                    would(change, &networks, answered),
                    ThisConversation::KeepsItsConnection,
                    "{change:?} {answered:?}"
                );
            }
        }
    }
}
