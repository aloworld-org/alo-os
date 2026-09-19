//! Setting up a printer a person chose, without anybody choosing a driver, a
//! queue or a protocol.
//!
//! **The driver is the printer's own description of itself.** A printer sold as
//! driverless answers, over IPP, with everything the printing service needs to
//! print on it — what it takes, which paper, in colour or not — and the service
//! builds its setup from that answer (`everywhere`, in its own words). That is
//! the only way this crate sets a printer up. A printer that needs its maker's
//! program is told so in a sentence, and no program is installed and no driver
//! is written ([`CannotSetUp::NeedsItsMakersProgram`]).
//!
//! **The queue is derived** ([`crate::printer::Queue`]) and **the protocol is
//! the address the printer was found at**, so a person's whole part is choosing
//! which printer.
//!
//! **It becomes the printer this machine prints on.** v0.5 ships one printer
//! that works rather than a chooser nobody asked for, so the printer set up last
//! is the one printing uses, and the printing service keeps that choice — this
//! crate keeps nothing of its own to disagree with it.
//!
//! # Never silently
//!
//! [`set_up`] takes one [`Found`] printer and adds that one. Nothing here adds
//! every printer found, nothing adds one as a side effect of finding, and no
//! verb an agent can ask for reaches this function: the only road is the
//! privileged broker carrying out a change a person chose — [`Found::proposal`]'s
//! sentence in Settings, or an agent's proposal to add exactly this printer,
//! approved (`crates/alo-changing-printers`).

use alo_strings::{Filling, Said, Strings};

use crate::found::Found;
use crate::ipp::{Group, Message, Value};
use crate::printer::{Printer, Queue};
use crate::reached::Speaks;
use crate::service::{PrintingService, Unanswered, next_request};
use crate::words;

/// The operation that adds a printer, or changes one.
const ADD_MODIFY_PRINTER: u16 = 0x4003;

/// The operation that makes a printer the one this machine prints on.
const SET_DEFAULT: u16 = 0x400a;

/// The printer state meaning idle: ready for a document.
const IDLE: i32 = 3;

/// The setup the printing service builds from what a printer says about
/// itself.
const FROM_WHAT_IT_SAYS: &str = "everywhere";

/// Why a printer a person chose was not set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CannotSetUp {
    /// It only works with a program from its maker.
    NeedsItsMakersProgram,
    /// It did not answer when it was asked to describe itself.
    DidNotAnswer,
    /// This machine does not allow printers to be added from this account.
    NotPermitted,
    /// This machine's printing service did not respond.
    ServiceNotAnswering,
}

impl CannotSetUp {
    /// The string this crate declares for this.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::NeedsItsMakersProgram => words::CANNOT_SET_UP_NEEDS_ITS_MAKERS_PROGRAM,
            Self::DidNotAnswer => words::CANNOT_SET_UP_DID_NOT_ANSWER,
            Self::NotPermitted => words::CANNOT_SET_UP_NOT_PERMITTED,
            Self::ServiceNotAnswering => words::CANNOT_SET_UP_SERVICE_NOT_ANSWERING,
        }
    }

    /// What a person reads, about the printer they chose.
    #[must_use]
    pub fn said(self, found: &Found, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &found
                .called()
                .fills(words::PRINTER, Filling::nothing(), strings),
        )
    }
}

/// What a person reads once a printer is set up.
#[must_use]
pub fn set_up_said(printer: &Printer, strings: &Strings) -> Said {
    strings.say(
        &words::SET_UP_DONE.key(),
        &printer
            .called()
            .fills(words::PRINTER, Filling::nothing(), strings),
    )
}

/// Set up this printer, which a person chose, and make it the one this machine
/// prints on.
///
/// # Errors
/// [`CannotSetUp`]. A printer that needs its maker's program is refused before
/// anything is sent to the printing service.
pub fn set_up(service: &PrintingService, found: &Found) -> Result<Printer, CannotSetUp> {
    if found.speaks() == Speaks::OnlyThroughItsMakersProgram {
        return Err(CannotSetUp::NeedsItsMakersProgram);
    }
    let queue = Queue::for_device(found.called(), found.device());
    let mut adding = Message::request(ADD_MODIFY_PRINTER, next_request())
        .with(
            Group::Operation,
            "printer-uri",
            vec![Value::uri(&queue.address())],
        )
        .with(
            Group::Printer,
            "device-uri",
            vec![Value::uri(found.device())],
        )
        .with(
            Group::Printer,
            "ppd-name",
            vec![Value::name(FROM_WHAT_IT_SAYS)],
        )
        .with(
            Group::Printer,
            "printer-is-accepting-jobs",
            vec![Value::Boolean(true)],
        )
        .with(Group::Printer, "printer-state", vec![Value::Enum(IDLE)]);
    if let crate::printer::Called::Named(name) = found.called() {
        adding = adding.with(Group::Printer, "printer-info", vec![Value::text(name)]);
    }
    answered(service.exchange("/admin/", &adding))?;

    let default = Message::request(SET_DEFAULT, next_request()).with(
        Group::Operation,
        "printer-uri",
        vec![Value::uri(&queue.address())],
    );
    answered(service.exchange("/admin/", &default))?;

    Ok(Printer::of(
        queue,
        found.called().clone(),
        found.reached().clone(),
    ))
}

/// What an answer from the printing service means for setting up.
fn answered(answer: Result<Message, Unanswered>) -> Result<(), CannotSetUp> {
    match answer {
        Ok(answer) if answer.succeeded() => Ok(()),
        // Forbidden, not authenticated, not authorised.
        Ok(answer) if (0x0401..=0x0403).contains(&answer.code()) => Err(CannotSetUp::NotPermitted),
        // Anything else the service says about adding a driverless printer is
        // that the printer did not describe itself: that is the one thing that
        // can fail once the request is well formed.
        Ok(_) => Err(CannotSetUp::DidNotAnswer),
        Err(Unanswered::NotPermitted) => Err(CannotSetUp::NotPermitted),
        Err(Unanswered::NotRunning | Unanswered::Silent | Unanswered::NotUnderstood) => {
            Err(CannotSetUp::ServiceNotAnswering)
        }
    }
}
