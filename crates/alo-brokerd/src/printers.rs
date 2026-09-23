//! The printers' three verbs, carried out.
//!
//! By the time a verb reaches here the door has decided it is exactly one a
//! person approved, once, and written that down. What is left is to do it to the
//! right printer and to no other — and the broker was never told which printer
//! in any form it could act on. It was told thirty-two bytes. So each verb asks
//! the printing service what it has **now**, digests what the service reported
//! for each printer the way the side that asked digested it, and acts on the one
//! that matches:
//!
//! | verb | asks | acts with |
//! |---|---|---|
//! | `printers.add` | the printers the service can find | `alo_printing::set_up` |
//! | `printers.remove` | the printers set up | `alo_printing::remove` |
//! | `printers.set-default` | the printers set up | `alo_printing::make_default` |
//!
//! **No match is no change**, and so are two: a printer that went away between
//! the approval and now is not replaced by whichever one is nearest, and an
//! identity two printers share names neither. **A verb that is not a printer's
//! is not carried out here** — the network, updates and storage are tasks of
//! their own — and the answer says so rather than pretending.
//!
//! `alo-printing` decides what a printer is and what is wrong with it. Nothing
//! here decides either, and nothing here reads a printer's name or address:
//! only [`Reported::as_reported`], digested.

use alo_broker::{Carrying, Identity, NotCarried, SystemVerb};

/// Something the printing service reported, by the bytes it reported it under.
pub trait Reported {
    /// The bytes, for digesting and nothing else.
    fn as_reported(&self) -> &[u8];
}

/// This machine's printing service, as the three verbs need it.
///
/// A trait rather than `alo_printing::PrintingService` itself so that which
/// printer is acted on — the decision this file exists for — is tested against
/// every shape of answer, including the ones a real service rarely gives.
/// `crate::printing_service` is the one implementation a machine runs.
pub trait PrintService {
    /// A printer the service found and has not set up.
    type Found: Reported;
    /// A printer set up on this machine.
    type SetUp: Reported;

    /// Every printer the service can find now.
    ///
    /// # Errors
    /// [`NotCarried`] when the service could not be asked.
    fn found(&self) -> Result<Vec<Self::Found>, NotCarried>;

    /// Set this printer up.
    ///
    /// # Errors
    /// [`NotCarried`] with what the service said.
    fn set_up(&self, found: &Self::Found) -> Result<(), NotCarried>;

    /// Every printer set up now.
    ///
    /// # Errors
    /// [`NotCarried`] when the service could not be asked.
    fn set_up_here(&self) -> Result<Vec<Self::SetUp>, NotCarried>;

    /// Remove this printer.
    ///
    /// # Errors
    /// [`NotCarried`] with what the service said.
    fn remove(&self, printer: &Self::SetUp) -> Result<(), NotCarried>;

    /// Make this printer the one this machine prints on.
    ///
    /// # Errors
    /// [`NotCarried`] with what the service said.
    fn make_default(&self, printer: &Self::SetUp) -> Result<(), NotCarried>;
}

/// What carries the broker's verbs out on this machine.
#[derive(Debug)]
pub struct Printers<S> {
    /// The printing service.
    service: S,
}

impl<S: PrintService> Printers<S> {
    /// Carry the printers' verbs out against this printing service.
    #[must_use]
    pub const fn against(service: S) -> Self {
        Self { service }
    }

    /// The printing service, for a test to look at what it was asked.
    #[must_use]
    pub const fn service(&self) -> &S {
        &self.service
    }
}

impl<S: PrintService> Carrying for Printers<S> {
    fn carry(&mut self, verb: SystemVerb, _approval: u64) -> Result<(), NotCarried> {
        match verb {
            SystemVerb::AddPrinter(identity) => {
                let found = self.service.found()?;
                self.service.set_up(the_one(&found, identity)?)
            }
            SystemVerb::RemovePrinter(identity) => {
                let set_up = self.service.set_up_here()?;
                self.service.remove(the_one(&set_up, identity)?)
            }
            SystemVerb::SetDefaultPrinter(identity) => {
                let set_up = self.service.set_up_here()?;
                self.service.make_default(the_one(&set_up, identity)?)
            }
            // Named one by one rather than with a wildcard, so the task that
            // carries one of these out meets this line in the compiler.
            SystemVerb::JoinNetwork(_)
            | SystemVerb::ForgetNetwork(_)
            | SystemVerb::SetRadio(_)
            | SystemVerb::SetProxy(_)
            | SystemVerb::ApplyStagedUpdate(_)
            | SystemVerb::RollBack(_)
            | SystemVerb::MountDrive(_)
            | SystemVerb::EjectDrive(_)
            | SystemVerb::RestartIntoWindows(_)
            | SystemVerb::StartByDefault(_) => Err(NotCarried(format!(
                "{} is not carried out on this machine yet, so nothing was changed",
                verb.name()
            ))),
        }
    }
}

/// The one printer reported under this identity, or no change.
fn the_one<T: Reported>(among: &[T], identity: Identity) -> Result<&T, NotCarried> {
    let mut matching = among
        .iter()
        .filter(|printer| Identity::of_what_was_reported(printer.as_reported()) == identity);
    match (matching.next(), matching.next()) {
        (Some(one), None) => Ok(one),
        (None, _) => Err(NotCarried(
            "the printing service reports no printer with the identity that was approved, so \
             nothing was changed"
                .to_owned(),
        )),
        (Some(_), Some(_)) => Err(NotCarried(
            "the printing service reports more than one printer with the identity that was \
             approved, so nothing was changed"
                .to_owned(),
        )),
    }
}
