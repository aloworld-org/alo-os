//! Which of the broker's verbs this machine carries out, and the refusal for
//! every other — by name.
//!
//! The door hands over a verb only once it has decided the verb is exactly one a
//! person approved and has written that down. [`Carriers`] sends it to what
//! carries it out. The match below names every verb on the closed list with no
//! wildcard, so the task that carries the printers, updates or storage out
//! meets its own line in the compiler rather than a verb that fell through.

use alo_broker::{Carrying, NotCarried, SystemVerb};
use alo_networks::NetworkService;

use crate::network::Network;
use crate::proxy::Proxy;

/// Everything that carries a verb out on this machine.
#[derive(Debug)]
pub struct Carriers<S> {
    /// The network's three verbs.
    network: Network<S>,
    /// The proxy.
    proxy: Proxy,
}

impl<S: NetworkService> Carriers<S> {
    /// The carriers, over this network manager and this proxy.
    #[must_use]
    pub const fn of(network: Network<S>, proxy: Proxy) -> Self {
        Self { network, proxy }
    }

    /// What carries the network's verbs out, for a test to look at.
    #[must_use]
    pub const fn network(&self) -> &Network<S> {
        &self.network
    }
}

impl<S: NetworkService> Carrying for Carriers<S> {
    fn carry(&mut self, verb: SystemVerb, _approval: u64) -> Result<(), NotCarried> {
        match verb {
            SystemVerb::JoinNetwork(identity) => self.network.join(identity),
            SystemVerb::ForgetNetwork(identity) => self.network.forget(identity),
            SystemVerb::SetRadio(switch) => self.network.radio(switch),
            SystemVerb::SetProxy(identity) => self.proxy.set(identity),
            SystemVerb::AddPrinter(_)
            | SystemVerb::RemovePrinter(_)
            | SystemVerb::SetDefaultPrinter(_)
            | SystemVerb::ApplyStagedUpdate(_)
            | SystemVerb::RollBack(_)
            | SystemVerb::MountDrive(_)
            | SystemVerb::EjectDrive(_) => Err(NotCarried(format!(
                "{} is not carried out on this machine yet, so nothing was changed",
                verb.name()
            ))),
        }
    }
}
