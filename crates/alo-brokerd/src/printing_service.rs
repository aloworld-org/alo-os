//! This machine's printing service, as the printers' verbs reach it.
//!
//! Nothing but `alo-printing`'s own operations, each failure kept as what that
//! crate named it. What those names mean to a person is said by the surface
//! that asked, from the broker's answer; what is written here reaches the
//! broker's record as `not-carried` and this service's log.

use alo_broker::NotCarried;
use alo_printing::{Found, Printer, PrintingService};

use crate::printers::{PrintService, Reported};

impl Reported for Found {
    fn as_reported(&self) -> &[u8] {
        Found::as_reported(self)
    }
}

impl Reported for Printer {
    fn as_reported(&self) -> &[u8] {
        Printer::as_reported(self)
    }
}

impl PrintService for PrintingService {
    type Found = Found;
    type SetUp = Printer;

    fn found(&self) -> Result<Vec<Found>, NotCarried> {
        alo_printing::find(self).map_err(|why| not_carried("finding printers", &why))
    }

    fn set_up(&self, found: &Found) -> Result<(), NotCarried> {
        alo_printing::set_up(self, found)
            .map(drop)
            .map_err(|why| not_carried("setting a printer up", &why))
    }

    fn set_up_here(&self) -> Result<Vec<Printer>, NotCarried> {
        alo_printing::printers_set_up(self)
            .map_err(|why| not_carried("listing the printers set up", &why))
    }

    fn remove(&self, printer: &Printer) -> Result<(), NotCarried> {
        alo_printing::remove(self, printer).map_err(|why| not_carried("removing a printer", &why))
    }

    fn make_default(&self, printer: &Printer) -> Result<(), NotCarried> {
        alo_printing::make_default(self, printer)
            .map_err(|why| not_carried("choosing the printer this machine prints on", &why))
    }
}

/// What the printing service answered, for the log.
fn not_carried(doing: &str, why: &dyn std::fmt::Debug) -> NotCarried {
    NotCarried(format!("the printing service refused {doing}: {why:?}"))
}
