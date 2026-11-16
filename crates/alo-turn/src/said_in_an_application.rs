//! **What a turn says on the journal when alo acts in an application**, so a
//! walk on a booted machine can check each act happened — and which ones did
//! not — without reading the screen.
//!
//! The desktop already says what it was asked at its door: a file opened
//! (`alo_acting::said_on_the_log::a_file_was_opened`), a text typed by keys
//! (`alo_typed`). Reading a window, pressing a control and opening, focusing or
//! closing an application never reach a line of the desktop's own, so nothing
//! on the journal said whether they ran. This says it for every act alo makes
//! in an application, in one sentence declared in `alo-acting`, by the verb's
//! name, the application and what became of it.
//!
//! **Never the text typed, a control's name or a window's title**: those may
//! name the person's document, and the journal is read by more than the
//! person. Written as the daemon's own stderr, which is the journal.

use alo_acting::said_on_the_log::{InAnApplication, alo_in_an_application};
use alo_capability::{Authorised, Value};

use crate::refusing::NotDone;
use crate::what_a_verb_did::WhatAVerbDid;

/// The verbs alo acts in an application by: each one names the application in
/// its `application` argument.
const IN_AN_APPLICATION: [&str; 7] = [
    "open_application",
    "focus_application",
    "close_application",
    alo_applications::OPEN_FILE_IN_APPLICATION,
    alo_adapters::READ_WINDOW,
    alo_adapters::ACTIVATE_CONTROL,
    alo_adapters::TYPE_TEXT,
];

/// The verb and the application an authorised call acts in, before it is
/// carried out — or [`None`] for a verb that does not act in an application.
#[must_use]
pub(crate) fn about(authorised: &Authorised) -> Option<(String, String)> {
    let verb = authorised.verb();
    if !IN_AN_APPLICATION.contains(&verb) {
        return None;
    }
    match authorised.call().value("application") {
        Some(Value::Application(application)) => Some((verb.to_owned(), application.clone())),
        _ => None,
    }
}

/// The sentence for what became of it.
#[must_use]
pub(crate) fn said(verb: &str, application: &str, did: &Result<WhatAVerbDid, NotDone>) -> String {
    let became = match did {
        Err(_) => InAnApplication::NotDone,
        Ok(did) => {
            if let Some(read) = did.the_windows_showed() {
                if read.shown().shows_any_text() {
                    InAnApplication::ReadWithText
                } else {
                    InAnApplication::ReadWithNoText
                }
            } else if let Some(pressed) = did.pressed() {
                if pressed.unanswered() {
                    InAnApplication::Unanswered
                } else {
                    InAnApplication::Done
                }
            } else if let Some(typed) = did.typed() {
                if typed.saved() {
                    InAnApplication::TypedAndSaved
                } else {
                    InAnApplication::TypedAndNotSaved
                }
            } else {
                InAnApplication::Done
            }
        }
    };
    alo_in_an_application(verb, application, became)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A refusal is said as not done**, whatever the verb.
    #[test]
    fn a_refusal_is_said_as_not_done() {
        let said = said(
            alo_adapters::READ_WINDOW,
            "org.kde.dolphin",
            &Err(NotDone::NeverAsked(alo_capability::ProposalError::ReadDoesNotWait {
                verb: alo_adapters::READ_WINDOW.to_owned(),
            })),
        );
        assert_eq!(
            said,
            "alo in an application: accessible.read_window in org.kde.dolphin, not done"
        );
    }
}
