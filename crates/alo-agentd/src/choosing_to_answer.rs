//! The person's door chooses a machine this one is paired with to answer their
//! questions.
//!
//! Task 14 of the local-network plan made a paired machine a choice a person
//! can make, and held the choice to the pairings at the moment it is made
//! through `alo_choosing::WhoMayBeAsked` — because a settings store may not
//! depend on anything that reaches the network. The pairings live in one place,
//! behind the daemon's one lock ([`TheNetwork`]), so this is the one road on
//! which the choice can be made against the list that refuses the next
//! question: `alo_protocol::FromAPerson::ChooseMachineToAnswer`, answered here.
//!
//! # The only file in this daemon that writes a person's settings
//!
//! The daemon reads the person's settings once a turn (`crate::questions`) and
//! wrote them nowhere until this file. It writes them here **only because the
//! person's own shell asked it to**, on the person's door, and it writes them
//! through `alo-choosing`'s one way out — the same door a settings panel calls,
//! with the same refusals and the same care about the file — rather than
//! composing a settings file of its own. An agent sending the same request is
//! refused by `alo-protocol` before anything reaches this file, in the words an
//! agent approving something gets; `alo-choosing`'s
//! `tests/no_agents_door_reaches_these_settings.rs` holds that this file is the
//! only one in the daemon that names the writer, and that no agent's door
//! reaches it.
//!
//! # In this order, and every refusal writes nothing
//!
//! 1. **An identity**, or the refusal a pairing request gets for something that
//!    is not one — before the settings file is opened, so a hostname typed into
//!    a shell never reaches the disk.
//! 2. **Somewhere to keep it**: the file the next turn's first question reads,
//!    [`crate::questions::Questions::where_the_settings_are`]. A session with no
//!    home has none, and says so.
//! 3. **The file as it stands**: a file that is there and does not hold is
//!    refused in `alo-choosing`'s own words and left exactly as it is, because
//!    writing a choice over it would lose whatever the person had typed.
//! 4. **The pairing, under the lock**, asked by `alo-choosing` of the very list
//!    that refuses the next verb and the next question from that machine. The
//!    lock is held while the file is written, so a revocation cannot land
//!    between the question and the write.
//!
//! # No model, no fallback, no default
//!
//! What is written is the machine's identity. Which model answers there is that
//! machine's person's (ADR 0008), so nothing here names one; and nothing here
//! chooses anything the person did not name — a refused choice leaves whatever
//! they had chosen before exactly as it was.

use std::path::Path;
use std::time::SystemTime;

use alo_choosing::{Choosing, NotWritten};
use alo_corridor::Naming as _;
use alo_nearby::MachineId;
use alo_protocol::ToAPerson;
use alo_strings::{Filling, Strings};

use crate::network::TheNetwork;
use crate::words::{NOWHERE_TO_KEEP_THE_CHOICE, THAT_IS_NOT_A_MACHINE};

/// Choose the machine the person named to answer their questions, or say in
/// one sentence why nothing was chosen.
///
/// `settings` is where this session's person keeps their settings — the file
/// the next turn reads — and `network` holds the pairings the choice is judged
/// against at `now`.
#[must_use]
pub fn chosen_to_answer(
    machine: &str,
    settings: Option<&Path>,
    network: &TheNetwork,
    strings: &Strings,
    now: SystemTime,
) -> ToAPerson {
    // 1. An identity, before any file is opened.
    let Ok(identity) = MachineId::read(machine.trim()) else {
        return ToAPerson::refused(&strings.say(&THAT_IS_NOT_A_MACHINE.key(), &Filling::nothing()));
    };

    // 2. Somewhere to keep it.
    let Some(at) = settings else {
        return ToAPerson::refused(
            &strings.say(&NOWHERE_TO_KEEP_THE_CHOICE.key(), &Filling::nothing()),
        );
    };

    // 3. The file as it stands.
    let mut choosing = match Choosing::at(at) {
        Ok(choosing) => choosing,
        Err(why) => return ToAPerson::refused(&why.said(strings)),
    };

    // 4. The pairing, under the lock, for as long as the file is written.
    let shared = network.locked();
    match choosing.answered_by_a_paired_machine(identity.as_str(), &*shared, now) {
        // The name the person gave it, beside the identity that was written:
        // asked of the names' own lock, taken second (`crate::names`).
        Ok(()) => ToAPerson::chosen_to_answer(
            identity.as_str(),
            network.names().called(&identity).as_deref(),
        ),
        Err(why) => {
            // The two that are the machine's rather than the person's are read
            // by whoever is fixing it; the person is told the sentence either
            // way.
            if let NotWritten::NotExpressible { why: english, .. }
            | NotWritten::NotKept { why: english, .. } = &why
            {
                eprintln!(
                    "alo-agentd: a machine chosen to answer questions was not written to {}: {english}",
                    why.at().display()
                );
            }
            ToAPerson::refused(&why.said(strings))
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::Cell;
    use std::path::PathBuf;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use alo_capability::Grants;
    use alo_models::Catalogue;
    use alo_nearby::MayAskIts;
    use alo_protocol::{ToAPerson, ToAnAgent};
    use alo_record::Record;

    use super::*;
    use crate::answering::what_a_person_said;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::holding::Holding;
    use crate::pairing::Nearby;
    use crate::questions::{Questions, TheBound, WhoseKeyring};
    use crate::rereading::WhatIsGranted;
    use crate::terms::NoNameYet;
    use crate::testing::{
        NobodyIsNearby, NothingIsRemembered, TheStudioIsAt, a_directory_of_our_own, a_message,
        hour, noon, nothing_has_been_chosen, on_a_machine_that_answers, paired_between, reception,
        the_studio, the_studio_answering,
    };

    /// An agent's question in words.
    const ASKED: &str = r#"{"ask":{"question":"how many invoices are unpaid?"}}"#;

    /// A machine nobody here is paired with.
    fn the_warehouse() -> MachineId {
        MachineId::read("11112222333344445555666677778888").unwrap()
    }

    /// The request, on the person's door, naming `machine`.
    fn choosing(machine: &str) -> String {
        a_message(&format!(
            r#"{{"choose-machine-to-answer":{{"machine":"{machine}"}}}}"#
        ))
    }

    /// Reception's session, with its settings under a directory of this test's
    /// own — no file in it yet — and where that file is.
    fn receptions_session(what: &str) -> (Questions, PathBuf) {
        let config = a_directory_of_our_own(what);
        let questions = Questions::of_a_session(
            Some(config.into_os_string()),
            None,
            Catalogue::built_in().unwrap(),
            TheBound::Nobodys,
            WhoseKeyring::Nobodys,
        );
        let at = questions.where_the_settings_are().unwrap();
        (questions, at)
    }

    /// Reception's pairings, holding its row of a pairing with `with`, made at
    /// noon for a day, permitting `may`.
    fn reception_paired_with(with: MachineId, may: &[MayAskIts]) -> TheNetwork {
        let network = TheNetwork::on(reception());
        let (on_reception, _) = paired_between(reception(), with, may, noon());
        network.locked().pairings_mut().keep(on_reception);
        network
    }

    /// One line on the person's door, while a turn holds the machine.
    fn the_person_says(
        line: &str,
        turning: &mut alo_turn::Turning<'_, '_>,
        questions: &mut Questions,
        network: &TheNetwork,
        strings: &Strings,
        now: SystemTime,
    ) -> ToAPerson {
        let mut grants = Grants::default();
        what_a_person_said(
            line,
            &mut Holding::ATurn { turning, questions },
            &mut WhatIsGranted::of(&mut grants, &NothingIsRemembered),
            &Nearby {
                network,
                looking: &NobodyIsNearby,
                advertising: &crate::testing::nothing_is_advertised(),
            },
            strings,
            now,
        )
        .unwrap()
    }

    /// **A machine chosen on the person's door is the one the next turn's
    /// question goes to**: before the choice nothing answers; the choice is
    /// written where the turn reads; and the next turn's question is answered
    /// down the corridor by the studio, once.
    #[test]
    fn a_machine_chosen_on_the_persons_door_answers_the_next_turns_question() {
        let (mut questions, at) = receptions_session("chosen-through-the-door");
        let network = reception_paired_with(the_studio(), &[MayAskIts::Models]);
        let (studio_at, heard) = the_studio_answering();
        let looking = TheStudioIsAt {
            at: studio_at,
            looked: Cell::new(0),
        };
        let corridor = Corridor {
            network: &network,
            looking: &looking,
            naming: &NoNameYet,
        };
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, grants, strings| {
            let before = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                grants,
                strings,
                hour(),
                noon(),
            );
            assert!(before.refusal().is_some(), "{before:?}");

            let said = the_person_says(
                &choosing(the_studio().as_str()),
                turning,
                &mut questions,
                &network,
                strings,
                noon(),
            );
            assert_eq!(
                said.machine_chosen_to_answer(),
                Some(the_studio().as_str()),
                "{said:?}"
            );

            questions.a_new_turn();
            let after = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                grants,
                strings,
                hour(),
                noon(),
            );
            assert!(
                matches!(&after, ToAnAgent::Answered { text, .. } if text == "Three are unpaid."),
                "{after:?}"
            );
        });

        assert!(
            std::fs::read_to_string(&at)
                .unwrap()
                .contains(&format!("machine = \"{}\"", the_studio().as_str()))
        );
        assert_eq!(heard.load(Ordering::SeqCst), 1);
        assert_eq!(looking.looked.get(), 1);
    }

    /// **`chosen-to-answer` carries the name the person gave the machine beside
    /// the identity**, and the settings still hold the identity alone — a name
    /// is for reading, and what the next question goes to is the identity.
    #[test]
    fn the_machine_chosen_to_answer_is_told_by_its_name_beside_its_identity() {
        let (mut questions, at) = receptions_session("chosen-by-name");
        let network = reception_paired_with(the_studio(), &[MayAskIts::Models]);
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let before = the_person_says(
                &choosing(the_studio().as_str()),
                turning,
                &mut questions,
                &network,
                strings,
                noon(),
            );
            assert_eq!(before.called(), None, "{before:?}");

            let named = the_person_says(
                &a_message(&format!(
                    r#"{{"name-machine":{{"machine":"{}","called":"the studio machine"}}}}"#,
                    the_studio().as_str()
                )),
                turning,
                &mut questions,
                &network,
                strings,
                noon(),
            );
            assert!(named.became_of_naming().is_some(), "{named:?}");

            let said = the_person_says(
                &choosing(the_studio().as_str()),
                turning,
                &mut questions,
                &network,
                strings,
                noon(),
            );
            assert_eq!(said.machine_chosen_to_answer(), Some(the_studio().as_str()));
            assert_eq!(said.called(), Some("the studio machine"), "{said:?}");
        });
        let settings = std::fs::read_to_string(&at).unwrap();
        assert!(
            settings.contains(&format!("machine = \"{}\"", the_studio().as_str())),
            "{settings}"
        );
        assert!(!settings.contains("the studio machine"), "{settings}");
    }

    /// **A choice no pairing permits is refused in words, and writes
    /// nothing**: never paired, paired for something other than its models, a
    /// pairing that ran out, and one revoked — each a refusal naming the
    /// machine, with a file that did not exist still absent and a file that did
    /// left byte for byte.
    #[test]
    fn a_machine_no_pairing_lets_answer_is_refused_and_nothing_is_written() {
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let revoked = reception_paired_with(the_studio(), &[MayAskIts::Models]);
            assert!(revoked.locked().pairings_mut().revoke(&the_studio()));
            let cases = [
                ("never-paired", TheNetwork::on(reception()), noon()),
                (
                    "paired-for-a-workspace",
                    reception_paired_with(the_studio(), &[MayAskIts::Workspace]),
                    noon(),
                ),
                (
                    "ran-out",
                    reception_paired_with(the_studio(), &[MayAskIts::Models]),
                    noon() + Duration::from_secs(2 * 86_400),
                ),
                ("revoked", revoked, noon()),
            ];
            for (what, network, now) in cases {
                let (mut questions, at) = receptions_session(&format!("refused-{what}"));
                let said = the_person_says(
                    &choosing(the_studio().as_str()),
                    turning,
                    &mut questions,
                    &network,
                    strings,
                    now,
                );
                let refusal = said.refusal().unwrap();
                assert!(!refusal.is_a_bug(), "{what}: {refusal:?}");
                assert!(
                    refusal.text().contains(the_studio().as_str()),
                    "{what}: {refusal:?}"
                );
                assert!(!at.exists(), "{what}: a refused choice wrote a file");

                // And a file already there is left exactly as it was.
                std::fs::create_dir_all(at.parent().unwrap()).unwrap();
                let already = format!(
                    "format = 3\n\n[answers]\nmachine = \"{}\"\n",
                    the_warehouse().as_str()
                );
                std::fs::write(&at, &already).unwrap();
                let again = the_person_says(
                    &choosing(the_studio().as_str()),
                    turning,
                    &mut questions,
                    &network,
                    strings,
                    now,
                );
                assert!(again.refusal().is_some(), "{what}: {again:?}");
                assert_eq!(std::fs::read_to_string(&at).unwrap(), already, "{what}");
            }
        });
        assert_eq!(record.len(), 0, "a refused choice was written down");
    }

    /// **Something that is not an identity is refused before the settings are
    /// opened**: a settings file that would not hold is not what the person is
    /// told about, and is left as it was.
    #[test]
    fn something_that_is_not_an_identity_is_refused_before_the_settings_are_opened() {
        let (mut questions, at) = receptions_session("not-an-identity");
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(&at, "this is not settings [").unwrap();
        let network = reception_paired_with(the_studio(), &[MayAskIts::Models]);
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            for named in ["disan-laptop", "192.168.1.20", ""] {
                let said = the_person_says(
                    &choosing(named),
                    turning,
                    &mut questions,
                    &network,
                    strings,
                    noon(),
                );
                let refusal = said.refusal().unwrap();
                assert_eq!(
                    refusal.text(),
                    strings
                        .say(&THAT_IS_NOT_A_MACHINE.key(), &Filling::nothing())
                        .text(),
                    "{named}"
                );
            }
        });
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "this is not settings ["
        );
    }

    /// **A settings file that does not hold is refused in `alo-choosing`'s
    /// words, and is not written over** — even for a machine a pairing lets
    /// answer.
    #[test]
    fn a_settings_file_that_does_not_hold_is_refused_and_left_alone() {
        let (mut questions, at) = receptions_session("unreadable-settings");
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(&at, "this is not settings [").unwrap();
        let network = reception_paired_with(the_studio(), &[MayAskIts::Models]);
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = the_person_says(
                &choosing(the_studio().as_str()),
                turning,
                &mut questions,
                &network,
                strings,
                noon(),
            );
            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
        });
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "this is not settings ["
        );
    }

    /// **A session with nowhere to keep settings chooses nothing**, and says
    /// so rather than writing somewhere else.
    #[test]
    fn a_session_with_nowhere_to_keep_settings_is_refused_in_words() {
        let mut questions = nothing_has_been_chosen();
        assert!(questions.where_the_settings_are().is_none());
        let network = reception_paired_with(the_studio(), &[MayAskIts::Models]);
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = the_person_says(
                &choosing(the_studio().as_str()),
                turning,
                &mut questions,
                &network,
                strings,
                noon(),
            );
            assert_eq!(
                said.refusal().unwrap().text(),
                strings
                    .say(&NOWHERE_TO_KEEP_THE_CHOICE.key(), &Filling::nothing())
                    .text()
            );
        });
    }

    /// **An agent choosing where questions are answered is refused in the words
    /// an agent approving something gets**, the settings are not written, and
    /// the question that follows is not sent anywhere.
    #[test]
    fn an_agent_choosing_a_machine_to_answer_is_refused_and_nothing_is_written() {
        let (mut questions, at) = receptions_session("agent-chooses");
        let network = reception_paired_with(the_studio(), &[MayAskIts::Models]);
        let (studio_at, heard) = the_studio_answering();
        let looking = TheStudioIsAt {
            at: studio_at,
            looked: Cell::new(0),
        };
        let corridor = Corridor {
            network: &network,
            looking: &looking,
            naming: &NoNameYet,
        };
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, grants, strings| {
            let said = what_an_agent_said(
                &choosing(the_studio().as_str()),
                turning,
                &mut questions,
                Some(&corridor),
                grants,
                strings,
                hour(),
                noon(),
            );
            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert!(
                refusal
                    .text()
                    .contains("an agent cannot answer a question that was put to a person"),
                "{refusal:?}"
            );

            questions.a_new_turn();
            let asked = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                grants,
                strings,
                hour(),
                noon(),
            );
            assert!(asked.refusal().is_some(), "{asked:?}");
        });

        assert!(
            !at.exists(),
            "an agent's request wrote the person's settings"
        );
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(heard.load(Ordering::SeqCst), 0);
        assert_eq!(looking.looked.get(), 0);
    }
}
