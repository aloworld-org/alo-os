//! What a person reads when a change to the printers was made, and when it was
//! not.
//!
//! One sentence each, and each refusal carries only what its sentence fills in:
//! the name of the printer, as it gave it. The broker answers in one word from
//! a closed list and has no person in front of it; this is where that word
//! becomes words, in the reader's language.

use alo_broker::{Answer, SystemVerb};
use alo_printing::Called;
use alo_record::AtTheBroker;
use alo_strings::{Filling, Said, Strings};

use crate::wanted::Change;
use crate::words;

/// Why a change to the printers was not made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotChanged {
    /// What was handed over was not an approved change to the printers.
    NotAPrinterChange,
    /// The printing service could not be asked which printers there are.
    PrintingNotAnswering,
    /// No printer this machine can find has this name.
    NoneFoundCalled(Called),
    /// No printer set up on this machine has this name.
    NoneSetUpCalled(Called),
    /// More than one printer has this name.
    MoreThanOneCalled(Called),
    /// The approval was refused where the change is made.
    ApprovalNotAccepted,
    /// The change was accepted, and the machine could not finish it.
    CouldNotFinish(Called),
    /// The machine has stopped writing down changes, so it makes none.
    NotBeingKept,
    /// Nothing was there to make the change.
    NothingMakesChanges,
}

impl NotChanged {
    /// What a person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let (word, called) = match self {
            Self::NotAPrinterChange => (words::NOT_A_PRINTER_CHANGE, None),
            Self::PrintingNotAnswering => (words::PRINTING_NOT_ANSWERING, None),
            Self::NoneFoundCalled(called) => (words::NONE_FOUND_CALLED, Some(called)),
            Self::NoneSetUpCalled(called) => (words::NONE_SET_UP_CALLED, Some(called)),
            Self::MoreThanOneCalled(called) => (words::MORE_THAN_ONE_CALLED, Some(called)),
            Self::ApprovalNotAccepted => (words::APPROVAL_NOT_ACCEPTED, None),
            Self::CouldNotFinish(called) => (words::COULD_NOT_FINISH, Some(called)),
            Self::NotBeingKept => (words::NOT_BEING_KEPT, None),
            Self::NothingMakesChanges => (words::NOTHING_MAKES_CHANGES, None),
        };
        strings.say(&word.key(), &filled(called, strings))
    }

    /// What the broker's answer to a change to the printer called this means.
    ///
    /// # Errors
    /// The refusal, for every answer but `carried`.
    pub fn from_the_brokers(answer: Answer, called: &Called) -> Result<(), Self> {
        match answer {
            Answer::Carried => Ok(()),
            Answer::NotKept => Err(Self::NotBeingKept),
            Answer::Refused(AtTheBroker::NotCarried) => Err(Self::CouldNotFinish(called.clone())),
            // Everything else the door refuses is about the approval, or about
            // who asked — which, for the one process that asks, is the same
            // fact: this approval did not make this change. Named one by one so
            // a reason added to the broker's list meets this line.
            Answer::Refused(
                AtTheBroker::NotTheAgentService
                | AtTheBroker::NotARequest
                | AtTheBroker::NotOneOfItsVerbs
                | AtTheBroker::NotApproved
                | AtTheBroker::ApprovalSpent
                | AtTheBroker::ApprovalLapsed,
            ) => Err(Self::ApprovalNotAccepted),
        }
    }
}

/// What a person reads once a change was made to the printer called this.
#[must_use]
pub fn changed_said(verb: &SystemVerb, called: &Called, strings: &Strings) -> Option<Said> {
    let word = match Change::of(verb)? {
        Change::Add => alo_printing::words::SET_UP_DONE,
        Change::Remove => words::REMOVED,
        Change::MakeDefault => words::MADE_DEFAULT,
    };
    Some(strings.say(&word.key(), &filled(Some(called), strings)))
}

/// The filling for a sentence about the printer called this, or about none.
fn filled(called: Option<&Called>, strings: &Strings) -> Filling {
    match called {
        Some(called) => called.fills(words::PRINTER, Filling::nothing(), strings),
        None => Filling::nothing(),
    }
}
