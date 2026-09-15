//! Asking how a printer is, and which printer this machine prints on.
//!
//! Both are questions put to this machine's printing service, and both answer
//! in this crate's own terms: a printer is ready or it has [`Stopped`] for one
//! of five reasons, and the printer this machine prints on is a [`Printer`] or
//! there is none yet. **Neither ever answers with what the service said**, so
//! a failure to ask is itself one of the five — not answering, having tried
//! the service.

use alo_strings::{Filling, Said, Strings};

use crate::ipp::{Group, Message, Value};
use crate::printer::{Called, Printer, Queue};
use crate::reached::Reached;
use crate::service::{PrintingService, Unanswered, next_request};
use crate::stopped::{Stopped, Tried};
use crate::words;

/// The operation that asks a printer's attributes.
const GET_PRINTER_ATTRIBUTES: u16 = 0x000b;

/// The operation that asks which printer is the one this machine prints on.
const GET_DEFAULT: u16 = 0x4001;

/// The status meaning there is no such printer, or no default one.
const NOT_FOUND: u16 = 0x0406;

/// How a printer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    /// Nothing is wrong with it.
    Ready,
    /// It stopped, and this is what is wrong.
    Stopped(Stopped),
}

impl Condition {
    /// What a person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self {
            Self::Ready => strings.say(&words::READY.key(), &Filling::nothing()),
            Self::Stopped(stopped) => stopped.said(strings),
        }
    }
}

/// How this printer is, now.
#[must_use]
pub fn how_is(service: &PrintingService, printer: &Printer) -> Condition {
    let queue = printer.queue();
    let request = Message::request(GET_PRINTER_ATTRIBUTES, next_request())
        .with(
            Group::Operation,
            "printer-uri",
            vec![Value::uri(&queue.address())],
        )
        .with(
            Group::Operation,
            "requested-attributes",
            vec![
                Value::keyword("printer-state"),
                Value::keyword("printer-state-reasons"),
                Value::keyword("printer-is-accepting-jobs"),
            ],
        );
    match service.exchange(&queue.path(), &request) {
        Ok(answer) if answer.succeeded() => condition_in(&answer),
        Ok(answer) if answer.code() == NOT_FOUND => Condition::Stopped(Stopped::NotAnswering {
            tried: Tried::LookingForItsSetUp,
        }),
        Ok(_) | Err(_) => Condition::Stopped(Stopped::NotAnswering {
            tried: Tried::AskingThisMachinesPrinting,
        }),
    }
}

/// How a printer is, from the answer describing it.
fn condition_in(answer: &Message) -> Condition {
    let attribute = |name| answer.attribute(Group::Printer, name);
    let state = attribute("printer-state").and_then(|state| state.integer());
    let reasons: Vec<&str> = attribute("printer-state-reasons")
        .map(|reasons| reasons.texts().collect())
        .unwrap_or_default();
    let accepting =
        attribute("printer-is-accepting-jobs").and_then(|accepting| accepting.boolean());
    Stopped::of_the_printer(state, &reasons, accepting).map_or(Condition::Ready, Condition::Stopped)
}

/// Why there is no printer to print on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoPrinter {
    /// None has been set up yet.
    NoneSetUp,
    /// The printing service could not be asked.
    Stopped(Stopped),
}

impl NoPrinter {
    /// What a person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self {
            Self::NoneSetUp => strings.say(&words::NO_PRINTER_YET.key(), &Filling::nothing()),
            Self::Stopped(stopped) => stopped.said(strings),
        }
    }
}

/// The printer this machine prints on.
///
/// # Errors
/// [`NoPrinter`]: none has been set up, or the printing service could not be
/// asked.
pub fn this_machines_printer(service: &PrintingService) -> Result<Printer, NoPrinter> {
    let request = Message::request(GET_DEFAULT, next_request()).with(
        Group::Operation,
        "requested-attributes",
        vec![
            Value::keyword("printer-name"),
            Value::keyword("device-uri"),
            Value::keyword("printer-info"),
        ],
    );
    let not_answering = NoPrinter::Stopped(Stopped::NotAnswering {
        tried: Tried::AskingThisMachinesPrinting,
    });
    let answer = match service.exchange("/", &request) {
        Ok(answer) => answer,
        Err(
            Unanswered::NotRunning
            | Unanswered::Silent
            | Unanswered::NotUnderstood
            | Unanswered::NotPermitted,
        ) => {
            return Err(not_answering);
        }
    };
    if answer.code() == NOT_FOUND {
        return Err(NoPrinter::NoneSetUp);
    }
    if !answer.succeeded() {
        return Err(not_answering);
    }
    let text = |name| {
        answer
            .attribute(Group::Printer, name)
            .and_then(|value| value.text())
    };
    let queue = text("printer-name").and_then(Queue::answered);
    let reached = text("device-uri").and_then(Reached::of_device);
    match (queue, reached) {
        (Some(queue), Some((reached, _))) => Ok(Printer::of(
            queue,
            Called::announced(text("printer-info")),
            reached,
        )),
        // A default printer this crate did not set up, or one at an address
        // that is not a printer: not one it prints on, and the person sets up
        // the one they want.
        _ => Err(NoPrinter::NoneSetUp),
    }
}
