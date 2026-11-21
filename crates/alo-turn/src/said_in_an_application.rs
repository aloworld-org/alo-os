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
///
/// **And `install_application`**, which names the application it installs in
/// the same argument: not an act *in* it, but the one a walk most needs to see
/// done or not done before it acts in it (the launch audit of 2026-11-21).
const IN_AN_APPLICATION: [&str; 17] = [
    alo_software::INSTALL_APPLICATION,
    "open_application",
    "focus_application",
    "close_application",
    alo_applications::OPEN_FILE_IN_APPLICATION,
    alo_adapters::READ_WINDOW,
    alo_adapters::ACTIVATE_CONTROL,
    alo_adapters::TYPE_TEXT,
    alo_adapters::SCROLL_TO,
    alo_adapters::SCROLL_PAGE,
    alo_adapters::SET_VALUE,
    alo_adapters::CHOOSE_OPTION,
    alo_adapters::SELECT_ITEM,
    alo_adapters::OPEN_ITEM,
    alo_adapters::OPEN_MENU,
    alo_adapters::PRESS_KEY,
    alo_adapters::TYPE_WITHOUT_SAVING,
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
            } else if let Some(set) = did.set() {
                if set.unanswered() {
                    InAnApplication::Unanswered
                } else {
                    InAnApplication::Done
                }
            } else if let Some(pressed) = did.pressed() {
                if pressed.unanswered() {
                    InAnApplication::Unanswered
                } else {
                    InAnApplication::Done
                }
            } else if let Some(opened) = did.item_opened() {
                if opened.unanswered() {
                    InAnApplication::Unanswered
                } else {
                    InAnApplication::Done
                }
            } else if let Some(chosen) = did.chosen() {
                if chosen.unanswered() {
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
            } else if did
                .menu_opened()
                .is_some_and(crate::AMenuOpened::unanswered)
            {
                InAnApplication::Unanswered
            } else {
                InAnApplication::Done
            }
        }
    };
    alo_in_an_application(verb, application, became)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_capability::{Call, Given, Grantee, Grants};

    use super::*;
    use crate::testing::noon;

    /// An authorised read of `verb` from this list, with these arguments, by
    /// an agent at full access.
    fn authorised(
        verbs: &alo_capability::Verbs,
        verb: &str,
        given: &[(&str, Given)],
    ) -> Authorised {
        let call = Call::of(verbs.of(verb).unwrap(), given).unwrap();
        let agent = Grantee::named("@alo");
        let grants = Grants::default().at_full_access_for(&agent);
        Authorised::read(&call, &agent, &grants, noon()).unwrap()
    }

    /// **An act in an application is named by its verb and its application**,
    /// read off the authorised call.
    #[test]
    fn an_act_in_an_application_is_named_by_its_verb_and_application() {
        let verbs = alo_adapters::fallback_verbs().unwrap();
        let reading = authorised(
            &verbs,
            alo_adapters::READ_WINDOW,
            &[("application", Given::text("org.kde.dolphin"))],
        );
        assert_eq!(
            about(&reading),
            Some((
                alo_adapters::READ_WINDOW.to_owned(),
                "org.kde.dolphin".to_owned()
            ))
        );
    }

    /// **An installation is said too, by the application it installs**, so a
    /// walk can check alo installed what it was asked for — and what it did
    /// not: no walk had ever seen `install_application` run (the launch audit
    /// of 2026-11-21, item 6), and nothing on the journal said whether it had.
    #[test]
    fn an_installation_is_named_by_the_application_it_installs() {
        let verbs = alo_software::software_verbs().unwrap();
        let call = Call::of(
            verbs.of(alo_software::INSTALL_APPLICATION).unwrap(),
            &[
                ("application", Given::text("io.github.seadve.Kooha")),
                ("source", Given::text("flathub")),
            ],
        )
        .unwrap();
        let agent = Grantee::named("@alo");
        let grants = Grants::default().at_full_access_for(&agent);
        let installing = Authorised::at_full_access(&call, &agent, &grants, noon()).unwrap();
        assert_eq!(
            about(&installing),
            Some((
                alo_software::INSTALL_APPLICATION.to_owned(),
                "io.github.seadve.Kooha".to_owned()
            ))
        );
    }

    /// **A verb that does not act in an application says nothing**, so the
    /// journal is not filled with every file read.
    #[test]
    fn a_verb_outside_applications_says_nothing() {
        let verbs = alo_files::file_verbs().unwrap();
        let listing = authorised(
            &verbs,
            "list_folder",
            &[("folder", Given::text("/home/anna"))],
        );
        assert_eq!(about(&listing), None);
    }

    /// **A refusal is said as not done**, whatever the verb.
    #[test]
    fn a_refusal_is_said_as_not_done() {
        let said = said(
            alo_adapters::READ_WINDOW,
            "org.kde.dolphin",
            &Err(NotDone::NeverAsked(
                alo_capability::ProposalError::ReadDoesNotWait {
                    verb: alo_adapters::READ_WINDOW.to_owned(),
                },
            )),
        );
        assert_eq!(
            said,
            "alo in an application: accessible.read_window in org.kde.dolphin, not done"
        );
    }
}
