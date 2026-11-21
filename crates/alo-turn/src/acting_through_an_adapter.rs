//! **An adapter's verb** — `text_editor.open_document` — carried out for alo:
//! one approved message to the application, over the person's session bus,
//! while the desktop shows that alo is working in their window.
//!
//! Until 2026-11-21 an adapter's verbs reached a person only through `alo-app`
//! in the terminal (`docs/features.md`, *alo is offered an adapter's verbs
//! when a person asks*). The steps, and why each is where it is:
//!
//! 1. **`alo_adapters::Driving::of`**: an adapter's verb, its application
//!    granted to this agent at this moment, installed, and the one message the
//!    call becomes, built from validated values only. Nothing is sent yet.
//! 2. **Where alo works, asked again** (`crate::before_an_adapters_card`): the
//!    card asked it, and the person may have changed the setting since.
//! 3. **The desktop is told before anything is sent**: `work-in-their-window`
//!    on the compositor's door (`alo_acting::working_in`), held while the
//!    message goes. Its answer is the desktop's own reading of the same setting
//!    with the same function, and while it holds, the line is up that alo's
//!    border on the window reads (`alo_shell::alo_at_work::what_is_sending`). A
//!    desktop that refuses, or is not there, means nothing is sent: an adapter's
//!    message into a window the person cannot see alo in is the thing this
//!    order prevents.
//! 4. **One message, once** (`Driving::deliver` consumes the authority), then
//!    `done` on the door, so the line comes off.
//!
//! **Every refusal is written down, and every one means nothing was sent.** A
//! message the application did not answer is recorded as having run — it was
//! sent under the approval — with the sentence saying whether it happened is
//! not known.

use alo_acting::{ToWorkIn, Where, Working};
use alo_adapters::{Driving, NotDelivered};
use alo_applications::{Application, NotInstalled};
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;
use alo_strings::Filling;

use crate::before_an_adapters_card::may_an_adapters_call_go;
use crate::machine::Machine;
use crate::refusing::NotDone;

/// One adapter's verb alo carried out — or sent, to an application that did
/// not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnAdapterVerbRan {
    /// The verb, by its name on the machine's list.
    verb: String,
    /// The application it went to, by its identifier.
    application: String,
    /// Whether the application did not answer, so whether it did it is not
    /// known.
    unanswered: bool,
}

impl AnAdapterVerbRan {
    /// The verb, by its name on the machine's list.
    #[must_use]
    pub fn verb(&self) -> &str {
        &self.verb
    }

    /// The application it went to, by its identifier.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// Whether the application did not answer.
    #[must_use]
    pub const fn unanswered(&self) -> bool {
        self.unanswered
    }
}

/// Carry out one approved adapter verb, once.
///
/// Answers the record's entry and what happened.
pub(crate) fn acting_through_an_adapter(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AnAdapterVerbRan, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    // 1. An adapter's verb, granted, installed, and its one message.
    let driving = match Driving::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(driving) => driving,
        Err(refused) => return refusing(refused),
    };
    let Some((adapter, _)) = machine.the_adapters().carrying_out(call.verb()) else {
        // `Driving::of` answered for this verb, so its adapter is here.
        return refusing(Refused::worded_elsewhere(
            call.clone(),
            NotDelivered::NotThere.said(call.verb(), strings),
        ));
    };
    let identifier = adapter.application;
    let named = machine.what_a_person_calls(identifier);
    let in_words = |word: alo_strings::Word| {
        Refused::worded_elsewhere(
            call.clone(),
            strings.say(&word.key(), &Filling::of("application", named.clone())),
        )
    };

    // 2. Where alo works, asked again now.
    if let Err(refused) = may_an_adapters_call_go(&call, machine) {
        return refusing(refused);
    }
    let Some(bus) = machine.the_session_bus() else {
        return refusing(Refused::worded_elsewhere(
            call.clone(),
            NotDelivered::NotThere.said(&named, strings),
        ));
    };
    let Ok(application) = Application::identified(identifier) else {
        return refusing(in_words(alo_applications::words::COULD_NOT_ASK));
    };

    // 3. The desktop is told before anything is sent.
    let asked = ToWorkIn::at(application, Where::InTheirWindow);
    let working = match machine.at_the_compositors_door() {
        Some(door) => door.working_in(&asked),
        None => return refusing(in_words(alo_applications::words::COULD_NOT_ASK)),
    };
    let working = match working {
        Ok(working) => working,
        Err(_) => return refusing(in_words(alo_applications::words::COULD_NOT_ASK)),
    };
    match working.answered() {
        answered if answered.may_work() => {}
        Working::NotInstalled => {
            return refusing(Refused::worded_elsewhere(
                call.clone(),
                NotInstalled::wanting(identifier).said(strings),
            ));
        }
        Working::NotChosen | Working::NoCopyOfItsOwn => {
            return refusing(in_words(alo_capability::words::NO_COPY_OF_ITS_OWN));
        }
        _ => return refusing(in_words(alo_applications::words::COULD_NOT_ASK)),
    }

    // 4. One message, once, and the line comes off.
    let driven = driving.deliver(bus, strings);
    // What the desktop counted is nothing: their window is never counted.
    let _ = working.done();
    let driven = match driven {
        Ok(driven) => driven,
        Err(refused) => return refusing(refused),
    };
    let entry = Entry::ran(driven.authorised(), strings);
    let entry = match driven.said(strings) {
        Some(said) => entry.telling(&[said]),
        None => entry,
    };
    (
        entry,
        Ok(AnAdapterVerbRan {
            verb: call.verb().to_owned(),
            application: identifier.to_owned(),
            unanswered: driven.unanswered(),
        }),
    )
}

#[cfg(test)]
#[path = "acting_through_an_adapter_tests.rs"]
mod tests;
