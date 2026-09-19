//! Which of the broker's verbs this machine carries out, and the refusal for
//! every other — by name.
//!
//! The door hands over a verb only once it has decided the verb is exactly one a
//! person approved and has written that down. [`Carriers`] sends it to what
//! carries it out. The match below names every verb on the closed list with no
//! wildcard, so the task that carries the updates out meets its own line in
//! the compiler rather than a verb that fell through.
//!
//! **The update verbs wait on a decision.** The base's own program refuses to
//! change the machine for anything but root holding `CAP_SYS_ADMIN`, and this
//! process holds no capability. How an update is carried out without giving the
//! broker that is ADR 0053, proposed; until it is accepted and built, both update
//! verbs are answered `not-carried`, in the record, and nothing is run.

use alo_broker::{Carrying, NotCarried, SystemVerb};
use alo_drives::DriveService;
use alo_networks::NetworkService;
use alo_printing::PrintingService;

use crate::network::Network;
use crate::printers::{PrintService, Printers};
use crate::proxy::Proxy;
use crate::storage::Storage;

/// Everything that carries a verb out on this machine.
#[derive(Debug)]
pub struct Carriers<S, D, P = PrintingService> {
    /// The network's three verbs.
    network: Network<S>,
    /// The proxy.
    proxy: Proxy,
    /// The two storage verbs.
    storage: Storage<D>,
    /// The printers, when the caller supplies their service.
    printers: Option<Printers<P>>,
}

impl<S: NetworkService, D: DriveService> Carriers<S, D> {
    /// The carriers, over this network manager, this proxy and this disk
    /// service.
    #[must_use]
    pub const fn of(network: Network<S>, proxy: Proxy, storage: Storage<D>) -> Self {
        Self {
            network,
            proxy,
            storage,
            printers: None,
        }
    }
}

impl<S: NetworkService, D: DriveService, P: PrintService> Carriers<S, D, P> {
    /// Add the printer carrier, preserving the network, proxy and storage.
    ///
    /// [`Carriers::of`] keeps its published three-argument signature and refuses
    /// printer verbs until this method supplies their service. The process
    /// supplies this machine's printing service; tests can supply their own.
    #[must_use]
    pub fn with_printers<T: PrintService>(self, printers: Printers<T>) -> Carriers<S, D, T> {
        Carriers {
            network: self.network,
            proxy: self.proxy,
            storage: self.storage,
            printers: Some(printers),
        }
    }

    /// What carries printer verbs out, if supplied.
    #[must_use]
    pub const fn printers(&self) -> Option<&Printers<P>> {
        self.printers.as_ref()
    }

    /// What carries the network's verbs out, for a test to look at.
    #[must_use]
    pub const fn network(&self) -> &Network<S> {
        &self.network
    }

    /// What carries the storage verbs out, for a test to look at.
    #[must_use]
    pub const fn storage(&self) -> &Storage<D> {
        &self.storage
    }
}

impl<S: NetworkService, D: DriveService, P: PrintService> Carrying for Carriers<S, D, P> {
    fn carry(&mut self, verb: SystemVerb, approval: u64) -> Result<(), NotCarried> {
        match verb {
            SystemVerb::JoinNetwork(identity) => self.network.join(identity),
            SystemVerb::ForgetNetwork(identity) => self.network.forget(identity),
            SystemVerb::SetRadio(switch) => self.network.radio(switch),
            SystemVerb::SetProxy(identity) => self.proxy.set(identity),
            SystemVerb::MountDrive(identity) => self.storage.mount(identity),
            SystemVerb::EjectDrive(identity) => self.storage.eject(identity),
            SystemVerb::ApplyStagedUpdate(_) | SystemVerb::RollBack(_) => Err(NotCarried(format!(
                "{} waits on ADR 0053: the base changes the machine only for a process holding a \
                 capability the broker does not hold, so nothing was run",
                verb.name()
            ))),
            SystemVerb::AddPrinter(_)
            | SystemVerb::RemovePrinter(_)
            | SystemVerb::SetDefaultPrinter(_) => match self.printers.as_mut() {
                Some(printers) => printers.carry(verb, approval),
                None => Err(NotCarried(format!(
                    "{} is not carried out on this machine yet, so nothing was changed",
                    verb.name()
                ))),
            },
        }
    }
}
