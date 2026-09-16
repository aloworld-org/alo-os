//! What this machine converts, said only when something here converts.
//!
//! ADR 0039 §1: `alo-opening` is told this machine converts the three formats
//! **only when the service answers**. A machine without it says *nothing here
//! opens it*, which is true, and never *this converts* followed by a failure.

use alo_opening::{Kind, NotAnAbility, ThisMachine};

use crate::conversion::Conversion;
use crate::service::ConvertingService;

/// The same machine, which also converts the three formats into a PDF when
/// the converting service answers — and exactly as it was when it does not.
///
/// A machine that converts opens the PDF it converts into; saying so here is
/// saying it once.
///
/// # Errors
/// [`NotAnAbility`] when the machine was already said to open one of the three
/// as it is, which would leave which of two answers a person gets to the order
/// they were said in.
pub fn with_what_converts(
    machine: ThisMachine,
    service: &ConvertingService,
) -> Result<ThisMachine, NotAnAbility> {
    if !service.answers() {
        return Ok(machine);
    }
    let mut machine = machine.opens(Kind::Pdf)?;
    for conversion in Conversion::EVERY {
        machine = machine.converts(conversion.from(), conversion.into())?;
    }
    Ok(machine)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::io::Cursor;

    use alo_opening::{Cannot, Decided, Outcome, decide};

    use crate::testing::the_document;

    /// **A machine whose service does not answer converts nothing**, and says
    /// so as *nothing here opens it*.
    #[test]
    fn without_the_service_nothing_is_said_to_convert() {
        let nowhere = std::env::temp_dir().join(format!(
            "alo-converting-no-machine-{}/socket",
            std::process::id()
        ));
        let machine = with_what_converts(
            ThisMachine::with_nothing(),
            &ConvertingService::at(&nowhere),
        )
        .unwrap();
        assert_eq!(machine, ThisMachine::with_nothing());
        let decided = decide(
            &mut Cursor::new(the_document("sample.docx")),
            OsStr::new("sample.docx"),
            &machine,
        )
        .unwrap();
        assert_eq!(
            decided,
            Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(
                Kind::WordDocument
            )))
        );
    }
}
