//! Which printer a change is to: listed as the printing service reports them,
//! and chosen by a person's click or by the name they approved.
//!
//! Both roads end in the same place — an [`Identity`] digested from what the
//! printing service reported — and neither ever produces anything else. Settings
//! draws [`Listed`] printers and a person picks one ([`crate::by_hand`]); an
//! agent's approved sentence names a printer, and [`chosen`] finds the one
//! printer listed under exactly that name, or refuses.
//!
//! # A name is matched exactly, once
//!
//! The name a printer gave itself, character for character, against the name
//! the person approved. **No printer with that name is no change, and neither
//! is two**: an office with two of one model is exactly where guessing would
//! remove the wrong printer, and the refusal sends the person to Settings, where
//! the two are told apart by where they are.

use alo_broker::{Identity, SystemVerb};
use alo_printing::{Called, Found, Printer, PrintingService};

use crate::refusing::NotChanged;
use crate::wanted::{Change, Wanted};

/// A printer as a person chooses among them: what it is called, and the
/// identity a change to it crosses into the broker under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    /// What it is called.
    called: Called,
    /// The digest of what the printing service reported it under.
    identity: Identity,
}

impl Listed {
    /// A printer the printing service found.
    #[must_use]
    pub fn found(found: &Found) -> Self {
        Self::reported(found.called().clone(), found.as_reported())
    }

    /// A printer set up on this machine.
    #[must_use]
    pub fn set_up(printer: &Printer) -> Self {
        Self::reported(printer.called().clone(), printer.as_reported())
    }

    /// A printer called this, reported under these bytes.
    #[must_use]
    pub fn reported(called: Called, reported: &[u8]) -> Self {
        Self {
            called,
            identity: Identity::of_what_was_reported(reported),
        }
    }

    /// What it is called.
    #[must_use]
    pub const fn called(&self) -> &Called {
        &self.called
    }

    /// The identity a change to it crosses into the broker under.
    #[must_use]
    pub const fn identity(&self) -> Identity {
        self.identity
    }
}

/// The printing service could not be asked which printers there are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotAnswering;

/// This machine's printers, as a change to one is chosen among them.
///
/// A trait so that which printer a name is — the decision this file exists for —
/// is tested against every shape of list; [`PrintingService`] is the one a
/// machine uses.
pub trait ThisMachinesPrinters {
    /// Every printer the printing service can find now.
    ///
    /// # Errors
    /// [`NotAnswering`].
    fn found(&self) -> Result<Vec<Listed>, NotAnswering>;

    /// Every printer set up now.
    ///
    /// # Errors
    /// [`NotAnswering`].
    fn set_up_here(&self) -> Result<Vec<Listed>, NotAnswering>;
}

impl ThisMachinesPrinters for PrintingService {
    fn found(&self) -> Result<Vec<Listed>, NotAnswering> {
        alo_printing::find(self)
            .map(|found| found.iter().map(Listed::found).collect())
            .map_err(|_| NotAnswering)
    }

    fn set_up_here(&self) -> Result<Vec<Listed>, NotAnswering> {
        alo_printing::printers_set_up(self)
            .map(|set_up| set_up.iter().map(Listed::set_up).collect())
            .map_err(|_| NotAnswering)
    }
}

/// The broker's verb for the change a person approved, to the one printer
/// called exactly what they approved.
///
/// # Errors
/// [`NotChanged`]: the printing service did not answer, no printer has that
/// name, or more than one does.
pub fn chosen(
    wanted: &Wanted,
    printers: &impl ThisMachinesPrinters,
) -> Result<SystemVerb, NotChanged> {
    let listed = if wanted.change().is_of_a_printer_found() {
        printers.found()
    } else {
        printers.set_up_here()
    }
    .map_err(|NotAnswering| NotChanged::PrintingNotAnswering)?;

    let called = Called::Named(wanted.called().to_owned());
    let mut matching = listed.iter().filter(|printer| printer.called() == &called);
    match (matching.next(), matching.next()) {
        (Some(one), None) => Ok(wanted.change().to(one.identity())),
        (None, _) if wanted.change() == Change::Add => Err(NotChanged::NoneFoundCalled(called)),
        (None, _) => Err(NotChanged::NoneSetUpCalled(called)),
        (Some(_), Some(_)) => Err(NotChanged::MoreThanOneCalled(called)),
    }
}
