//! What the person's desktop says happened at GitHub, kept in the record.
//!
//! The owner's decision of 2026-11-13, on the GitHub sign-in sheet and Settings
//! → Accounts: *every sign-in, sign-out and failure is recorded.* The desktop
//! runs GitHub's own command, `gh`, under the person's account and is not the
//! record's writer; this service is (`docs/contracts/record-file.md`). So the
//! desktop says what came of it on the person's own door (`github-sign-in`),
//! and this file writes it down and answers that it did.
//!
//! # What it keeps
//!
//! Exactly what arrived, as `alo_record::AtGitHub` — which `alo-protocol` made
//! with `alo-record`'s own constructors, so a login is held to the shape of one
//! and gh's last line is one line of a record. Nothing is decided here: the
//! desktop watched `gh` and this service did not, and a service that second-
//! guessed what it was told would be a record of its own opinion.
//!
//! # What it refuses, and how
//!
//! A malformed one — a login that is not a login, a reason that is not one of
//! the four, a field nobody declared — never reaches here. `alo-protocol`
//! refuses it as unreadable, and it is answered as every unreadable line on the
//! person's door is: refused in words, **and nothing written**, because nothing
//! happened that a record could truthfully say. An agent sending it is refused
//! on its own door before it is read at all.
//!
//! # Not an agent's
//!
//! No agent is named in what it writes, and there is no verb that signs in.

use std::time::SystemTime;

use alo_keeping::NotKept;
use alo_protocol::{AtGitHub, ToAPerson};

use crate::holding::Holding;

/// Keep `what` in the record, through whatever holds the machine, and say so.
///
/// # Errors
/// [`NotKept`] when the record could not be written — the service stops, as it
/// does for every entry it cannot keep, and the desktop hears no answer.
pub fn kept(
    what: AtGitHub,
    holding: &mut Holding<'_, '_, '_>,
    now: SystemTime,
) -> Result<ToAPerson, NotKept> {
    holding.a_github_sign_in_was_kept(what, now)?;
    Ok(ToAPerson::GitHubSignInKept)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_capability::Grants;
    use alo_keeping::Keeping;
    use alo_record::{Happened, Record, WhyNotSignedIn};
    use alo_turn::Shortened;

    use super::*;
    use crate::answering::what_a_person_said;
    use crate::doing::what_an_agent_said;
    use crate::network::TheNetwork;
    use crate::pairing::Nearby;
    use crate::rereading::WhatIsGranted;
    use crate::testing::{
        NobodyIsNearby, NothingIsRemembered, a_message, hour, noon, nothing_has_been_chosen,
        nothing_is_advertised, on_a_machine_that_answers, on_a_machine_with_no_turn, reception,
    };

    /// The person's desktop sends `asks`, on a machine with no turn, writing
    /// into `record`.
    fn the_desktop_says(
        asks: &str,
        record: &mut dyn alo_turn::Shortening,
    ) -> Result<ToAPerson, NotKept> {
        let network = TheNetwork::on(reception());
        on_a_machine_with_no_turn(
            "keeping-github-sign-ins",
            record,
            |machine, _, strings, _, _| {
                let mut grants = Grants::default();
                what_a_person_said(
                    &a_message(asks),
                    &mut Holding::Nobody(machine),
                    &mut WhatIsGranted::of(&mut grants, &NothingIsRemembered),
                    &Nearby {
                        network: &network,
                        looking: &NobodyIsNearby,
                        advertising: &nothing_is_advertised(),
                    },
                    strings,
                    noon(),
                )
            },
        )
    }

    /// **Each kind the desktop can report is kept, as it was reported, under
    /// nobody's authority** — and the desktop is told it was.
    #[test]
    fn what_the_desktop_reports_is_kept_and_it_is_told_so() {
        for (asks, what) in [
            (
                r#"{"github-sign-in":{"signed-in":{"login":"anna"}}}"#,
                AtGitHub::signed_in("anna").unwrap(),
            ),
            (
                r#"{"github-sign-in":{"signed-out":{"login":"anna"}}}"#,
                AtGitHub::signed_out(Some("anna")).unwrap(),
            ),
            (
                r#"{"github-sign-in":{"did-not-sign-in":{"why":"gh-is-not-here"}}}"#,
                AtGitHub::did_not_sign_in(WhyNotSignedIn::GhIsNotHere),
            ),
            (
                r#"{"github-sign-in":{"did-not-sign-in":{"why":{"gh-failed":{"said":"HTTP 502"}}}}}"#,
                AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some("HTTP 502"))),
            ),
            (
                r#"{"github-sign-in":{"did-not-sign-in":{"why":"no-code-arrived"}}}"#,
                AtGitHub::did_not_sign_in(WhyNotSignedIn::NoCodeArrived),
            ),
            (
                r#"{"github-sign-in":{"did-not-sign-in":{"why":"they-cancelled"}}}"#,
                AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled),
            ),
            (
                r#"{"github-sign-in":{"did-not-sign-out":{"said":"not logged in"}}}"#,
                AtGitHub::did_not_sign_out(Some("not logged in")),
            ),
        ] {
            let mut record = Record::default();
            let said = the_desktop_says(asks, &mut record).unwrap();
            assert_eq!(said, ToAPerson::GitHubSignInKept, "{asks}");
            assert_eq!(record.len(), 1, "{asks}: {record:?}");
            let entry = record.everything().next().unwrap();
            assert_eq!(entry.agent(), None, "{asks}");
            assert_eq!(entry.at(), noon());
            assert!(!entry.happened().caused_egress(), "{asks}");
            assert_eq!(entry.happened(), &Happened::AtGitHub { what }, "{asks}");
        }
    }

    /// **A malformed one is turned away and nothing is written**, as every
    /// unreadable line on the person's door is: a login that is not one, a
    /// reason that is not one of the four, a token riding along.
    #[test]
    fn a_malformed_one_is_refused_and_nothing_is_written() {
        for asks in [
            r#"{"github-sign-in":{"signed-in":{"login":"anna smith"}}}"#,
            r#"{"github-sign-in":{"signed-in":{"login":""}}}"#,
            r#"{"github-sign-in":{"signed-in":{"login":"anna","token":"gho_x"}}}"#,
            r#"{"github-sign-in":{"did-not-sign-in":{"why":"it-rained"}}}"#,
            r#"{"github-sign-in":{}}"#,
        ] {
            let mut record = Record::default();
            let said = the_desktop_says(asks, &mut record).unwrap();
            let refusal = said.refusal().unwrap();
            assert!(!refusal.text().is_empty(), "{asks}");
            assert_ne!(said, ToAPerson::GitHubSignInKept, "{asks}");
            assert_eq!(record.len(), 0, "{asks}: {record:?}");
        }
    }

    /// **An agent sending it is refused**, in the words an agent approving
    /// something gets, and no sign-in is written into the person's record.
    #[test]
    fn an_agent_reporting_a_sign_in_is_refused_and_none_is_kept() {
        let mut questions = nothing_has_been_chosen();
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, grants, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"github-sign-in":{"signed-in":{"login":"anna"}}}"#),
                turning,
                &mut questions,
                None,
                grants,
                strings,
                hour(),
                noon(),
                crate::running_a_command::Commands::where_nothing_was_chosen(),
            );
            let refusal = said.refusal().unwrap();
            assert!(
                refusal
                    .text()
                    .contains("an agent cannot answer a question that was put to a person"),
                "{refusal:?}"
            );
        });
        assert!(
            !record
                .everything()
                .any(|entry| matches!(entry.happened(), Happened::AtGitHub { .. })),
            "{record:?}"
        );
    }

    /// A record that cannot be written to, which is what a full disk looks
    /// like from here.
    struct ANoSpaceLeftDisk;

    impl alo_turn::Kept for ANoSpaceLeftDisk {
        fn keep(&mut self, _entry: alo_record::Entry) -> Result<(), NotKept> {
            Err(NotKept::NotAddedTo {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                why: "no space left on device".to_owned(),
            })
        }

        fn written_down(&self) -> Result<Record, NotKept> {
            Ok(Record::default())
        }
    }

    impl alo_turn::Shortening for ANoSpaceLeftDisk {
        fn shorten(&mut self, _keeping: Keeping, _now: SystemTime) -> Result<Shortened, NotKept> {
            Ok(Shortened::NotOnADisk)
        }
    }

    /// **A sign-in that cannot be written down is never answered as kept**:
    /// the service is handed the failure, and stops on it.
    #[test]
    fn a_sign_in_that_cannot_be_written_down_is_not_answered_as_kept() {
        let said = the_desktop_says(
            r#"{"github-sign-in":{"signed-in":{"login":"anna"}}}"#,
            &mut ANoSpaceLeftDisk,
        );
        assert!(matches!(said, Err(NotKept::NotAddedTo { .. })), "{said:?}");
    }
}
