//! Printing a document an agent was approved to print — the verb, carried out.
//!
//! Four questions, in this order, and the first *no* is the answer:
//!
//! 1. **Is this printing?** An [`Authorised`] call for any other verb is
//!    refused: an approval is for one sentence, and it never carries over.
//! 2. **If the printer is across the network, has that been shown?** A document
//!    sent there leaves this machine, so it is an [`alo_egress::Leaving`] under
//!    the agent the call was authorised for, and nothing is sent without the
//!    [`Departing`] the indicator hands out for exactly that egress. The
//!    indicator lights before the connection opens; law 1, *visible at the
//!    moment it happens*, is only true if it is asked first.
//! 3. **Is it something a printer takes?** Decided from the file's bytes
//!    ([`crate::printable`]); a program named as a PDF never reaches paper.
//! 4. **Did the printer take it?** If not, what is wrong is one of
//!    [`Stopped`]'s five — never what the printing service said.
//!
//! **This does not take a path.** Which file may be read is a grant, and the
//! code that holds grants opens the file; this is handed it open, and reads the
//! name to compare against its bytes from the call a person approved.

use std::io::{Read, Seek};
use std::path::Path;

use alo_capability::{Authorised, Value as Argument};
use alo_egress::Departing;
use alo_strings::{Filling, Said, Strings};

use crate::asking::{Condition, how_is};
use crate::document::{NotPrintable, printable};
use crate::ipp::{Group, Message, Value};
use crate::printer::Printer;
use crate::service::{PrintingService, Unanswered, next_request};
use crate::stopped::{Stopped, Tried};
use crate::verbs::{DOCUMENT, PRINT_DOCUMENT};
use crate::words;

/// The operation that hands a printer one document.
const PRINT_JOB: u16 = 0x0002;

/// A document handed to the printer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Printed {
    /// The number the printing service gave the job, if it gave one.
    job: Option<i32>,
}

impl Printed {
    /// The number the printing service gave the job — for a record, never for
    /// a person.
    #[must_use]
    pub fn job(&self) -> Option<i32> {
        self.job
    }

    /// What a person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&words::PRINTED.key(), &Filling::nothing())
    }
}

/// Why nothing was printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotPrinted {
    /// What was approved was not printing a document.
    NotPrinting,
    /// The printer is across the network and that had not been shown.
    NotShownLeaving,
    /// The file is not something a printer takes.
    NotPrintable(NotPrintable),
    /// The printer did not take it.
    Stopped(Stopped),
}

impl NotPrinted {
    /// What a person reads, in order.
    ///
    /// A printer that stopped is said the way it is said when anybody asks how
    /// it is, and then — because this document was not taken and will not come
    /// out when the trouble is put right — that it needs printing again. A
    /// refused document already says nothing was printed and that sending it
    /// again changes nothing, so nothing is added to it.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Vec<Said> {
        match self {
            Self::NotPrinting => vec![strings.say(&words::NOT_PRINTING.key(), &Filling::nothing())],
            Self::NotShownLeaving => {
                vec![strings.say(&words::NOT_SHOWN_LEAVING.key(), &Filling::nothing())]
            }
            Self::NotPrintable(not) => not.said(strings),
            Self::Stopped(Stopped::RefusedTheJob) => vec![Stopped::RefusedTheJob.said(strings)],
            Self::Stopped(stopped) => vec![
                stopped.said(strings),
                strings.say(&words::NOT_TAKEN.key(), &Filling::nothing()),
            ],
        }
    }
}

/// Print the document this call was approved for, on this printer.
///
/// `departing` is what the egress indicator handed out for this print when the
/// printer is across the network, and [`None`] when it is on this machine.
///
/// # Errors
/// [`NotPrinted`], and nothing has been sent to any printer unless the answer
/// is [`NotPrinted::Stopped`].
pub fn print<F: Read + Seek>(
    service: &PrintingService,
    authorised: Authorised,
    printer: &Printer,
    departing: Option<&Departing>,
    document: &mut F,
) -> Result<Printed, NotPrinted> {
    if authorised.verb() != PRINT_DOCUMENT {
        return Err(NotPrinted::NotPrinting);
    }
    let Some(Argument::Path(path)) = authorised.call().value(DOCUMENT) else {
        return Err(NotPrinted::NotPrinting);
    };
    let name = Path::new(path).file_name().unwrap_or_default().to_owned();

    match printer.reached().leaving(authorised.under()) {
        Ok(None) => {}
        Ok(Some(leaving)) => {
            if departing.is_none_or(|departing| departing.leaving() != &leaving) {
                return Err(NotPrinted::NotShownLeaving);
            }
        }
        Err(_) => return Err(NotPrinted::NotShownLeaving),
    }

    let printable = printable(document, &name).map_err(NotPrinted::NotPrintable)?;
    let queue = printer.queue();
    let request = Message::request(PRINT_JOB, next_request())
        .with(
            Group::Operation,
            "printer-uri",
            vec![Value::uri(&queue.address())],
        )
        .with(
            Group::Operation,
            "document-format",
            vec![Value::mime(printable.format())],
        );

    let answer =
        match service.exchange_carrying(&queue.path(), &request, document, printable.length()) {
            Ok(answer) => answer,
            Err(Unanswered::NotPermitted) => {
                return Err(NotPrinted::Stopped(Stopped::RefusedTheJob));
            }
            Err(Unanswered::NotRunning | Unanswered::Silent | Unanswered::NotUnderstood) => {
                return Err(NotPrinted::Stopped(Stopped::NotAnswering {
                    tried: Tried::AskingThisMachinesPrinting,
                }));
            }
        };
    if answer.succeeded() {
        return Ok(Printed {
            job: answer
                .attribute(Group::Job, "job-id")
                .and_then(|job| job.integer()),
        });
    }
    Err(NotPrinted::Stopped(match answer.code() {
        // Forbidden, not authenticated, not authorised; too large; a format
        // the printer does not take, or one it could not read.
        0x0401..=0x0403 | 0x0408 | 0x040a | 0x0411 => Stopped::RefusedTheJob,
        // No such printer: it was removed since it was set up.
        0x0406 => Stopped::NotAnswering {
            tried: Tried::LookingForItsSetUp,
        },
        // Anything else is a printer that is not taking this now; asking it
        // how it is says why, and a printer that says nothing is wrong has
        // refused this document.
        _ => match how_is(service, printer) {
            Condition::Stopped(stopped) => stopped,
            Condition::Ready => Stopped::RefusedTheJob,
        },
    }))
}
