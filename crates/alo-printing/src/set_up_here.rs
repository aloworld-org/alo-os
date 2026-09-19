//! The printers already set up on this machine.
//!
//! [`crate::find`] answers *what is near this machine*; this answers *what has
//! this machine already been given*. The two are different lists on purpose: a
//! printer is removed, or made the one this machine prints on, from the second,
//! and only a printer somebody chose is ever on it.
//!
//! # Only what this crate could have set up
//!
//! The printing service may keep queues nobody set up through alo OS — one an
//! administrator added by hand, a printer made of software. A queue is listed
//! here only if its name is one this crate could have made and its address is
//! one a printer is found at ([`crate::Reached::of_device`]); anything else is
//! not a printer this machine offers anybody a change to, and is left exactly
//! where it is.
//!
//! # Asking changes nothing
//!
//! One request, and it is the one that lists printers.

use crate::ipp::{Group, Message, Value};
use crate::printer::{Called, Printer, Queue};
use crate::reached::Reached;
use crate::service::{PrintingService, Unanswered, next_request};

/// The operation that lists the printers the printing service keeps.
const GET_PRINTERS: u16 = 0x4002;

/// The status meaning there is nothing of what was asked for.
const NOT_FOUND: u16 = 0x0406;

/// Every printer set up on this machine, in the order the printing service
/// listed them.
///
/// # Errors
/// [`Unanswered`] when the service could not be asked. An empty list is not an
/// error: it is a machine nobody has set a printer up on yet.
pub fn printers_set_up(service: &PrintingService) -> Result<Vec<Printer>, Unanswered> {
    let request = Message::request(GET_PRINTERS, next_request()).with(
        Group::Operation,
        "requested-attributes",
        vec![
            Value::keyword("printer-name"),
            Value::keyword("device-uri"),
            Value::keyword("printer-info"),
        ],
    );
    let answer = service.exchange("/", &request)?;
    // The service answers *not found* when it keeps no printer at all.
    if answer.code() == NOT_FOUND {
        return Ok(Vec::new());
    }
    if !answer.succeeded() {
        return Err(Unanswered::NotUnderstood);
    }
    Ok(set_up_in(&answer))
}

/// The printers in an answer listing them, each once.
fn set_up_in(answer: &Message) -> Vec<Printer> {
    let mut set_up: Vec<Printer> = Vec::new();
    for (group, attributes) in answer.groups() {
        if group != Group::Printer {
            continue;
        }
        let text = |name: &str| {
            attributes
                .iter()
                .find(|attribute| attribute.name() == name)
                .and_then(|attribute| attribute.text())
        };
        let Some(queue) = text("printer-name").and_then(Queue::answered) else {
            continue;
        };
        let Some((reached, _)) = text("device-uri").and_then(Reached::of_device) else {
            continue;
        };
        if set_up.iter().any(|already| already.queue() == &queue) {
            continue;
        }
        set_up.push(Printer::of(
            queue,
            Called::announced(text("printer-info")),
            reached,
        ));
    }
    set_up
}
