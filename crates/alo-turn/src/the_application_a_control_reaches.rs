//! The application a checked adapter reaches belongs in the call's requirements.
//!
//! An adapter can take no application argument, but its read or change still
//! reaches one. Attach that requirement before proposal or authorization, so a
//! withdrawn grant refuses the question itself and execution records retain the
//! actual grant or Full-access authority. Delivery checks permission again.

use alo_capability::{Ask, Call};

use crate::machine::Machine;

/// Keep every existing requirement and add the checked adapter's application.
pub(crate) fn the_application_a_control_reaches(call: Call, machine: &Machine<'_>) -> Call {
    match machine.application_controls().0.carrying_out(call.verb()) {
        Some((adapter, _)) => call.also_requiring(Ask::Application(adapter.application.to_owned())),
        None => call,
    }
}
