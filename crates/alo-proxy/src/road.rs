//! Every road out of this machine, and which way one takes.
//!
//! *Machine-wide* is a claim about coverage, and a claim about coverage is only
//! as good as the list it is checked against. [`Road`] is that list: the closed
//! set of reasons anything on this machine reaches the network, with no member
//! meaning *something else*. A road added to the system without being added
//! here fails to compile at the exhaustive match at the bottom of this file, and
//! a road on this list that no crate takes is found by the test in
//! `tests/every_road_out_takes_it.rs`.
//!
//! It is `alo_egress::Errand`'s shape, deliberately, and the six that are also
//! errands are held to being the same six — by a test in `tests/`, since this
//! crate depends on nothing that could say so at compile time.
//!
//! # The indicator names the real destination, and this file is why it can
//!
//! [`Way::Through`] carries the proxy. It carries **no destination**, because
//! the destination never changed: the road is still going where it was going,
//! and the proxy is how. So there is nothing here a caller could hand an
//! indicator by mistake — the thing an `alo_egress::Destination` is built from
//! is `crate::Reaching`, which is what the caller already had.
//!
//! *It went to the proxy* is not what a person needs to know. On a machine sold
//! on sovereignty, the indicator answers *where did my work go*, and on a
//! company network the answer is the same whether or not a proxy is in the
//! middle of it.

use serde::{Deserialize, Serialize};

use crate::address::ProxyAddress;

/// A reason something on this machine reaches the network.
///
/// A closed list, and the *machine-wide* half of this crate's promise as code
/// rather than as a sentence. There is no `Road::Other(String)`: a road nobody
/// named is a road nobody could hold to the proxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Road {
    /// Signing a person in against their alo identity.
    SigningIn,
    /// Downloading a model, so this machine can answer on its own hardware.
    FetchingAModel,
    /// Asking whether there is a newer deployment of the operating system.
    CheckingForAnUpdate,
    /// Fetching an application a person chose.
    InstallingAnApplication,
    /// Asking whether there are newer versions of installed applications.
    CheckingForApplicationUpdates,
    /// Fetching a newer version of an application.
    UpdatingAnApplication,
    /// Putting a question, or a test, to a provider that answers elsewhere.
    ///
    /// The one road on this list that an **agent** can cause. It is here
    /// because the proxy is the machine's and not the errand list's: a company
    /// network with no other way out has no other way out for a question
    /// either.
    AskingAProvider,
}

impl Road {
    /// Every road there is, in the order this file declares them.
    ///
    /// What the test per road walks, and what a settings panel would list if it
    /// ever showed a person what their machine reaches the network for.
    pub const EVERY: [Self; 7] = [
        Self::SigningIn,
        Self::FetchingAModel,
        Self::CheckingForAnUpdate,
        Self::InstallingAnApplication,
        Self::CheckingForApplicationUpdates,
        Self::UpdatingAnApplication,
        Self::AskingAProvider,
    ];

    /// Whether this road is one alo OS takes on its own, with no agent behind
    /// it — which is `alo_egress::Errand`'s list.
    #[must_use]
    pub const fn is_an_errand(self) -> bool {
        !matches!(self, Self::AskingAProvider)
    }
}

/// Which way a road out goes.
///
/// There are two, and there is deliberately no third meaning *whatever the
/// client decides*: a road whose way out was never decided is a road that went
/// somewhere nobody chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Way {
    /// Straight out, with nothing in the middle.
    Straight,
    /// Through this proxy.
    ///
    /// **It names the proxy and not the destination**, because the destination
    /// did not change — this file's second section says why that matters to the
    /// indicator.
    Through(ProxyAddress),
}

impl Way {
    /// The proxy this way goes through, or [`None`] for straight out.
    #[must_use]
    pub const fn through(&self) -> Option<&ProxyAddress> {
        match self {
            Self::Straight => None,
            Self::Through(proxy) => Some(proxy),
        }
    }

    /// Whether this road goes straight out.
    #[must_use]
    pub const fn is_straight(&self) -> bool {
        matches!(self, Self::Straight)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::address::SpokenTo;

    /// **Every road is named, and there is no member meaning *something
    /// else*.** A road added to the machine fails to compile here, which is
    /// where whoever adds one finds out that the proxy has to reach it.
    #[test]
    fn every_road_out_of_this_machine_is_on_the_list() {
        assert_eq!(Road::EVERY.len(), 7);
        for road in Road::EVERY {
            match road {
                Road::SigningIn
                | Road::FetchingAModel
                | Road::CheckingForAnUpdate
                | Road::InstallingAnApplication
                | Road::CheckingForApplicationUpdates
                | Road::UpdatingAnApplication
                | Road::AskingAProvider => {}
            }
        }
    }

    /// Six of the seven are roads alo OS takes with nobody having asked, which
    /// is `alo_egress::Errand`'s list; the seventh is an agent's.
    #[test]
    fn six_of_the_roads_are_ones_alo_os_takes_on_its_own() {
        assert_eq!(
            Road::EVERY
                .iter()
                .filter(|road| road.is_an_errand())
                .count(),
            6
        );
        assert!(!Road::AskingAProvider.is_an_errand());
    }

    /// **A way out names the proxy and never a destination.** There is nothing
    /// in it a caller could hand an indicator instead of where the road is
    /// actually going.
    #[test]
    fn a_way_out_names_the_proxy_and_never_a_destination() {
        let proxy = ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap();
        let through = Way::Through(proxy.clone());
        assert_eq!(through.through(), Some(&proxy));
        assert!(!through.is_straight());

        assert_eq!(Way::Straight.through(), None);
        assert!(Way::Straight.is_straight());
    }
}
