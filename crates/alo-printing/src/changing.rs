//! Changing a printer that is already set up: removing it, or making it the
//! one this machine prints on.
//!
//! Two decided changes, each one request to the printing service and nothing
//! else. **Neither is reachable by an agent directly**: they are what the
//! privileged broker carries out once a person approved exactly that change
//! (`crates/alo-brokerd`), and what a person's own choice in Settings reaches
//! through the same broker. This crate decides what a printer is and says what
//! is wrong with it; it does not decide whether a change should happen.
//!
//! # Removing a printer removes a place documents can go
//!
//! Which is the reason removing is a change a person approves, and the reason a
//! printer this crate did not set up is not one it removes: only a [`Printer`]
//! from [`crate::printers_set_up`] or [`crate::this_machines_printer`] can be
//! handed here, and neither lists a queue this crate could not have made.

use crate::ipp::{Group, Message, Value};
use crate::printer::Printer;
use crate::service::{PrintingService, Unanswered, next_request};

/// The operation that removes a printer.
const DELETE_PRINTER: u16 = 0x4004;

/// The operation that makes a printer the one this machine prints on.
const SET_DEFAULT: u16 = 0x400a;

/// The status meaning there is no such printer.
const NOT_FOUND: u16 = 0x0406;

/// Why a change to a printer that is set up was not made.
///
/// Read by whatever carries the change out and writes it down, never by a
/// person: what a person reads is said by the surface that asked, from the
/// answer the broker gave. So it has no words of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CannotChange {
    /// The printing service no longer keeps this printer.
    NoLongerThere,
    /// The printing service would not let this machine make the change from
    /// this account.
    NotPermitted,
    /// This machine's printing service did not respond, or answered with
    /// something that is not an answer.
    ServiceNotAnswering,
}

/// Remove this printer from this machine.
///
/// # Errors
/// [`CannotChange`], and nothing has been removed.
pub fn remove(service: &PrintingService, printer: &Printer) -> Result<(), CannotChange> {
    changed(service, DELETE_PRINTER, printer)
}

/// Make this printer the one this machine prints on.
///
/// # Errors
/// [`CannotChange`], and the printer this machine prints on is still the one it
/// was.
pub fn make_default(service: &PrintingService, printer: &Printer) -> Result<(), CannotChange> {
    changed(service, SET_DEFAULT, printer)
}

/// One administrative request about one printer, and what its answer means.
fn changed(
    service: &PrintingService,
    operation: u16,
    printer: &Printer,
) -> Result<(), CannotChange> {
    let request = Message::request(operation, next_request()).with(
        Group::Operation,
        "printer-uri",
        vec![Value::uri(&printer.queue().address())],
    );
    match service.exchange("/admin/", &request) {
        Ok(answer) if answer.succeeded() => Ok(()),
        Ok(answer) if answer.code() == NOT_FOUND => Err(CannotChange::NoLongerThere),
        // Forbidden, not authenticated, not authorised.
        Ok(answer) if (0x0401..=0x0403).contains(&answer.code()) => Err(CannotChange::NotPermitted),
        Err(Unanswered::NotPermitted) => Err(CannotChange::NotPermitted),
        Ok(_) | Err(Unanswered::NotRunning | Unanswered::Silent | Unanswered::NotUnderstood) => {
            Err(CannotChange::ServiceNotAnswering)
        }
    }
}
