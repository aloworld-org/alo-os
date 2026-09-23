//! Which of the broker's verbs this machine carries out, and the refusal for
//! every other — by name.
//!
//! The door hands over a verb only once it has decided the verb is exactly one a
//! person approved and has written that down. [`Carriers`] sends it to what
//! carries it out. The match below names every verb on the closed list with no
//! wildcard, so the task that carries the updates out meets its own line in
//! the compiler rather than a verb that fell through.
//!
//! **The update verbs are carried out by a unit this process starts.** The
//! base's own program refuses to change the machine for anything but root
//! holding `CAP_SYS_ADMIN`, and this process holds no capability. ADR 0053,
//! accepted option B, decided how they are carried out without giving the
//! broker that: [`crate::Updates`] checks what a person approved, hands it to a
//! folder only root can read, asks systemd to start the one unit that holds
//! what the base asks for, and waits for its result. Nothing here runs the base.

use alo_broker::{Carrying, NotCarried, SystemVerb};
use alo_drives::DriveService;
use alo_networks::NetworkService;
use alo_printing::PrintingService;
use alo_starting::{
    Entry, Firmware, NotAnswering, NotDone, NotRead, NotWritten, System, TheLoader,
};

use crate::by_default::ByDefault;
use crate::network::Network;
use crate::next_start::NextStart;
use crate::printers::{PrintService, Printers};
use crate::proxy::Proxy;
use crate::storage::Storage;
use crate::units::StartingUnits;
use crate::updates::Updates;

/// Everything that carries a verb out on this machine.
#[derive(Debug)]
pub struct Carriers<S, D, P = PrintingService, U = NoUnits, F = NoFirmware, L = NoLoader> {
    /// The network's three verbs.
    network: Network<S>,
    /// The proxy.
    proxy: Proxy,
    /// The two storage verbs.
    storage: Storage<D>,
    /// The printers, when the caller supplies their service.
    printers: Option<Printers<P>>,
    /// The two update verbs, when the caller supplies something that starts
    /// their units.
    updates: Option<Updates<U>>,
    /// *Restart into Windows*, when the caller supplies a firmware.
    next_start: Option<NextStart<F>>,
    /// Which system starts when nobody chooses, when the caller supplies the
    /// loader's files.
    by_default: Option<ByDefault<L>>,
}

/// What a broker built without the loader's files has: no file to change, and
/// no way to make one either.
///
/// The counterpart of [`NoFirmware`], for the same reason: [`Carriers::of`]
/// keeps its published signature and its type stays nameable. A broker holding
/// this answers *start this system when nobody chooses* `not-carried`, which is
/// what a machine with no loader to ask would do anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoLoader {}

impl TheLoader for NoLoader {
    fn offering(&self) -> Result<Vec<System>, NotRead> {
        match *self {}
    }

    fn saved(&self) -> Result<Vec<u8>, NotRead> {
        match *self {}
    }

    fn save(&self, _: &[u8]) -> Result<(), NotWritten> {
        match *self {}
    }
}

/// What a broker built without a firmware has: no firmware to ask, and no way
/// to make one either.
///
/// The counterpart of [`NoUnits`], for the same reason: [`Carriers::of`] keeps
/// its published signature and its type stays nameable. A broker holding this
/// answers *Restart into Windows* `not-carried`, which is what a machine with
/// no firmware to ask would do anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoFirmware {}

impl Firmware for NoFirmware {
    fn entries(&self) -> Result<Vec<Entry>, NotAnswering> {
        match *self {}
    }

    fn start_next(&self, _: u16) -> Result<(), NotDone> {
        match *self {}
    }
}

/// What a broker built without anything to start units has: no way to start
/// one, and no way to make one either.
///
/// It exists so [`Carriers::of`] keeps its published signature and its type
/// stays nameable. A broker holding this answers both update verbs
/// `not-carried`, which is what a machine with no systemd to ask would do
/// anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoUnits {}

impl StartingUnits for NoUnits {
    fn start(&self, _: crate::units::TheUnit) -> Result<(), crate::units::NotDone> {
        match *self {}
    }
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
            updates: None,
            next_start: None,
            by_default: None,
        }
    }
}

impl<
    S: NetworkService,
    D: DriveService,
    P: PrintService,
    U: StartingUnits,
    F: Firmware,
    L: TheLoader,
> Carriers<S, D, P, U, F, L>
{
    /// Add the printer carrier, preserving the network, proxy and storage.
    ///
    /// [`Carriers::of`] keeps its published three-argument signature and refuses
    /// printer verbs until this method supplies their service. The process
    /// supplies this machine's printing service; tests can supply their own.
    #[must_use]
    pub fn with_printers<T: PrintService>(
        self,
        printers: Printers<T>,
    ) -> Carriers<S, D, T, U, F, L> {
        Carriers {
            network: self.network,
            proxy: self.proxy,
            storage: self.storage,
            printers: Some(printers),
            updates: self.updates,
            next_start: self.next_start,
            by_default: self.by_default,
        }
    }

    /// Add the update carrier, preserving everything else.
    ///
    /// Additive for the same reason [`Carriers::with_printers`] is: what was
    /// published keeps working, and a broker built without this answers both
    /// update verbs `not-carried` rather than pretending.
    #[must_use]
    pub fn with_updates<T: StartingUnits>(self, updates: Updates<T>) -> Carriers<S, D, P, T, F, L> {
        Carriers {
            network: self.network,
            proxy: self.proxy,
            storage: self.storage,
            printers: self.printers,
            updates: Some(updates),
            next_start: self.next_start,
            by_default: self.by_default,
        }
    }

    /// Add the carrier for *Restart into Windows*, preserving everything else.
    ///
    /// Additive for the same reason [`Carriers::with_updates`] is: what was
    /// published keeps working, and a broker built without this answers the
    /// verb `not-carried` rather than pretending. The process supplies this
    /// machine's own firmware; tests supply their own.
    #[must_use]
    pub fn with_next_start<T: Firmware>(self, next: NextStart<T>) -> Carriers<S, D, P, U, T, L> {
        Carriers {
            network: self.network,
            proxy: self.proxy,
            storage: self.storage,
            printers: self.printers,
            updates: self.updates,
            next_start: Some(next),
            by_default: self.by_default,
        }
    }

    /// Add the carrier for *start this system when nobody chooses*, preserving
    /// everything else.
    ///
    /// Additive for the same reason [`Carriers::with_next_start`] is, and
    /// separate from it for a reason of its own: the next start and the default
    /// are two acts, and a machine may be able to carry out one of them and not
    /// the other. The process supplies this machine's own loader files; tests
    /// supply their own.
    #[must_use]
    pub fn with_by_default<T: TheLoader>(
        self,
        by_default: ByDefault<T>,
    ) -> Carriers<S, D, P, U, F, T> {
        Carriers {
            network: self.network,
            proxy: self.proxy,
            storage: self.storage,
            printers: self.printers,
            updates: self.updates,
            next_start: self.next_start,
            by_default: Some(by_default),
        }
    }

    /// What carries *Restart into Windows* out, if supplied.
    #[must_use]
    pub const fn next_start(&self) -> Option<&NextStart<F>> {
        self.next_start.as_ref()
    }

    /// What carries *start this system when nobody chooses* out, if supplied.
    #[must_use]
    pub const fn by_default(&self) -> Option<&ByDefault<L>> {
        self.by_default.as_ref()
    }

    /// What carries printer verbs out, if supplied.
    #[must_use]
    pub const fn printers(&self) -> Option<&Printers<P>> {
        self.printers.as_ref()
    }

    /// What carries the update verbs out, if supplied.
    #[must_use]
    pub const fn updates(&self) -> Option<&Updates<U>> {
        self.updates.as_ref()
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

    /// One of the two update verbs, carried out — or refused by name on a
    /// broker that was built with nothing to start their units.
    fn updating(
        &self,
        verb: SystemVerb,
        carry: impl FnOnce(&Updates<U>) -> Result<(), NotCarried>,
    ) -> Result<(), NotCarried> {
        match self.updates.as_ref() {
            Some(updates) => carry(updates),
            None => Err(NotCarried(format!(
                "{} is not carried out on this machine: nothing here can start the unit that \
                 does it, so nothing was run",
                verb.name()
            ))),
        }
    }
}

impl<
    S: NetworkService,
    D: DriveService,
    P: PrintService,
    U: StartingUnits,
    F: Firmware,
    L: TheLoader,
> Carrying for Carriers<S, D, P, U, F, L>
{
    fn carry(&mut self, verb: SystemVerb, approval: u64) -> Result<(), NotCarried> {
        match verb {
            SystemVerb::JoinNetwork(identity) => self.network.join(identity),
            SystemVerb::ForgetNetwork(identity) => self.network.forget(identity),
            SystemVerb::SetRadio(switch) => self.network.radio(switch),
            SystemVerb::SetProxy(identity) => self.proxy.set(identity),
            SystemVerb::MountDrive(identity) => self.storage.mount(identity),
            SystemVerb::EjectDrive(identity) => self.storage.eject(identity),
            SystemVerb::ApplyStagedUpdate(identity) => {
                self.updating(verb, |updates| updates.apply(identity))
            }
            SystemVerb::RollBack(identity) => {
                self.updating(verb, |updates| updates.go_back(identity))
            }
            SystemVerb::RestartIntoWindows(identity) => match self.next_start.as_ref() {
                Some(next) => next.windows(identity),
                None => Err(NotCarried(format!(
                    "{} is not carried out on this machine: nothing here can ask its firmware, so \
                     nothing was changed",
                    verb.name()
                ))),
            },
            SystemVerb::StartByDefault(identity) => match self.by_default.as_ref() {
                Some(by_default) => by_default.set(identity),
                None => Err(NotCarried(format!(
                    "{} is not carried out on this machine: nothing here can reach the files that \
                     say which system it starts, so nothing was changed",
                    verb.name()
                ))),
            },
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
