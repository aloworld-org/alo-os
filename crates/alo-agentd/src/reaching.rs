//! The person's door, reaching the remote turn that holds the machine.
//!
//! A change an agent on a paired machine proposed waits for **this** machine's
//! person (ADR 0003: *pairing lets A ask; it never lets A act*), and until
//! this file existed nothing on the person's door reached it: `approve`,
//! `decline` and `waiting` answered *nothing is happening* while
//! `alo_turn::Arriving` held the machine. This is [`crate::answering`]'s
//! twin for that turn — the same three requests, the same words, against the
//! remote turn's doors rather than the local one's — and it is a file of its
//! own because it changes when those doors change.
//!
//! # What is the same
//!
//! A number off the wire is not a handle: it is **found** among what is really
//! waiting on the turn, and a number nothing is waiting under is
//! `alo_capability::AnswerError::NothingWaiting`'s own sentence, exactly as
//! for a local turn. One approval is one execution, spent by being answered.
//! *No* is a whole answer.
//!
//! # What is different
//!
//! **The pairing is asked at the moment of approval.**
//! [`alo_turn::Arriving::approving`] asks this machine's pairings beside its
//! grants, so a pairing revoked between the change being proposed and the
//! person saying yes stops it at the moment it would have run — the lock over
//! the pairings (`crate::network`) is taken for exactly that call.
//!
//! **What is waiting says which machine.** What the person approves is the
//! sentence, and a sentence for a change from the machine down the corridor
//! has to say so: every change comes back stamped with the name this machine's
//! person gave that machine (`alo_protocol::Standing::from_a_machine`).
//!
//! **Nothing here reaches a read or a question.** A remote turn has no door
//! for either from this side, and the person's door has no request that
//! would ask for one.

use std::time::SystemTime;

use alo_capability::{Grants, ProposalId};
use alo_protocol::{FromAPerson, ToAPerson};
use alo_strings::Strings;
use alo_turn::Arriving;

use crate::answering::nothing_is_waiting;
use crate::network::TheNetwork;
use crate::rereading;

/// The three things a person can send about a turn, each against the remote
/// turn holding the machine.
///
/// `network` holds the pairings the approval is judged against, taken for the
/// length of that one call.
pub fn answered_to(
    answered: FromAPerson,
    arriving: &mut Arriving<'_, '_>,
    network: &TheNetwork,
    grants: &Grants,
    strings: &Strings,
    now: SystemTime,
) -> ToAPerson {
    match answered {
        FromAPerson::Waiting => {
            let called = arriving.origin().called().to_owned();
            ToAPerson::waiting_from_a_machine(arriving.waiting_at(now), &called, strings, now)
        }
        // The knock, the four about a pairing and choosing a machine to answer
        // are not about the turn, and
        // the one caller answers them before they reach here — the same arm,
        // and the same true sentence, as `crate::answering` keeps for a local
        // turn.
        FromAPerson::Granted
        | FromAPerson::Pair { .. }
        | FromAPerson::ConfirmPairing { .. }
        | FromAPerson::RevokePairing { .. }
        | FromAPerson::Pairings
        | FromAPerson::ChooseMachineToAnswer { .. }
        | FromAPerson::NameMachine { .. }
        | FromAPerson::ClearMachineName { .. }
        | FromAPerson::Workspaces => ToAPerson::refused(&rereading::what_to_say(strings)),
        FromAPerson::Approve { number } => match under(arriving, number, now) {
            Some(waiting) => {
                let shared = network.locked();
                match arriving.approving(waiting, shared.pairings(), grants, now) {
                    Ok(answer) => ToAPerson::did(&answer),
                    Err(why) => ToAPerson::refused(&why.said(strings)),
                }
            }
            None => nothing_is_waiting(number, strings),
        },
        FromAPerson::Decline { number } => match under(arriving, number, now) {
            Some(waiting) => match arriving.declining(waiting, now) {
                Ok(()) => ToAPerson::Declined,
                Err(why) => ToAPerson::refused(&why.said(strings)),
            },
            None => nothing_is_waiting(number, strings),
        },
    }
}

/// The change really waiting under this number on the remote turn, if one is.
///
/// The borrow ends with the search, so what comes back is a handle the turn
/// made rather than one this file invented from a number somebody sent.
fn under(arriving: &Arriving<'_, '_>, number: u64, now: SystemTime) -> Option<ProposalId> {
    arriving
        .waiting_at(now)
        .find(|waiting| waiting.id.as_u64() == number)
        .map(|waiting| waiting.id)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use alo_capability::{Given, Grant, Grants, Reach};
    use alo_corridor::{Carried, Door, Doorway, Judged, Replying};
    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_nearby::{MayAskIts, Pairing, Proof};
    use alo_protocol::ToAPerson;
    use alo_record::{Only, Record};
    use alo_strings::Strings;
    use alo_turn::Machine;

    use crate::answering::what_a_person_said;
    use crate::holding::Holding;
    use crate::network::TheNetwork;
    use crate::rereading::WhatIsGranted;
    use crate::testing::{
        NobodyIsNearby, NothingIsBounded, NothingIsRemembered, a_folder_with_an_invoice, a_message,
        hour, in_english, noon, nothing_has_been_chosen, paired_between, reception, the_studio,
    };

    /// The studio, paired with reception for its models and its workspace,
    /// with a folder granted to reception's principal, a doorway on it, and
    /// `doing` run against all of it: the person's door is
    /// `what_a_person_said` with the network holding the machine.
    fn on_the_studio<T>(
        what: &str,
        record: &mut Record,
        doing: impl FnOnce(
            &mut Doorway<'_, '_>,
            &TheNetwork,
            &mut Grants,
            &Pairing,
            &Strings,
            &Path,
        ) -> T,
    ) -> T {
        // The pairings live behind the network's lock; the doorway judges a
        // verb against a borrow taken for the length of one call.
        let strings = in_english();
        let (folder, _) = a_folder_with_an_invoice(what);
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            record,
        )
        .unwrap();
        let mut doorway = Doorway::at(the_studio(), &mut machine, hour(), hour()).unwrap();
        let (on_reception, on_studio) = paired_between(
            reception(),
            the_studio(),
            &[MayAskIts::Models, MayAskIts::Workspace],
            noon(),
        );
        let network = TheNetwork::on(the_studio());
        network.locked().pairings_mut().keep(on_studio);
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                &format!("machine:{}", reception().as_str()),
                Reach::Folder(folder.clone()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        doing(
            &mut doorway,
            &network,
            &mut grants,
            &on_reception,
            &strings,
            &folder,
        )
    }

    /// Reception proposes renaming the invoice, through the network's door,
    /// and the number the change waits under comes back.
    fn a_change_from_reception(
        doorway: &mut Doorway<'_, '_>,
        network: &TheNetwork,
        grants: &mut Grants,
        on_reception: &Pairing,
        folder: &Path,
        at: std::time::SystemTime,
    ) -> u64 {
        let body = Carried::of(
            "rename_file",
            &[
                (
                    "file",
                    Given::text(folder.join("march.pdf").to_string_lossy().into_owned()),
                ),
                ("name", Given::text("march-final.pdf")),
            ],
        )
        .said();
        let proof = Proof::made(on_reception, &reception(), body.as_bytes(), at);
        let judged = doorway.judged(
            Door::Change,
            Some(&proof),
            &body,
            network.locked().pairings(),
            grants,
            // What a running machine answers with: the names given on the
            // person's door, asked while the network's lock is held.
            network.names(),
            &EgressPolicy::InTheBuilding,
            at,
        );
        let Judged::Reply {
            replying:
                Replying::Answered {
                    departing,
                    answered,
                },
            ..
        } = judged
        else {
            unreachable!("{judged:?}")
        };
        let number = answered.waits().unwrap();
        doorway.returned(departing).unwrap();
        number
    }

    /// One line from the person's shell, with the network holding the
    /// machine.
    fn the_person_says(
        line: &str,
        doorway: &mut Doorway<'_, '_>,
        network: &TheNetwork,
        grants: &mut Grants,
        strings: &Strings,
    ) -> ToAPerson {
        what_a_person_said(
            &a_message(line),
            &mut Holding::TheNetwork {
                doorway,
                network,
                questions: &mut nothing_has_been_chosen(),
            },
            &mut WhatIsGranted::of(grants, &NothingIsRemembered),
            &crate::pairing::Nearby {
                network,
                looking: &NobodyIsNearby,
            },
            strings,
            noon(),
        )
        .unwrap()
    }

    /// **A change a paired machine proposed is listed for the person here
    /// with the machine named, approved on the person's door, and runs
    /// exactly once**: the second approval finds nothing waiting under the
    /// number, in the ordinary words.
    #[test]
    fn a_change_from_a_paired_machine_is_listed_approved_and_runs_once() {
        let mut record = Record::default();
        on_the_studio(
            "reaching-approved",
            &mut record,
            |doorway, network, grants, on_reception, strings, folder| {
                let number =
                    a_change_from_reception(doorway, network, grants, on_reception, folder, noon());
                let invoice = folder.join("march.pdf");
                assert!(invoice.is_file(), "a proposal moved a file");

                let said = the_person_says(r#"{"waiting":{}}"#, doorway, network, grants, strings);
                let changes = said.changes().unwrap();
                assert_eq!(changes.len(), 1);
                assert_eq!(changes.first().unwrap().number(), number);
                assert_eq!(
                    changes.first().unwrap().from(),
                    Some(reception().as_str()),
                    "{changes:?}"
                );
                assert!(
                    changes
                        .first()
                        .unwrap()
                        .sentence()
                        .text()
                        .contains("march-final.pdf")
                );

                let approve = format!(r#"{{"approve":{{"number":{number}}}}}"#);
                let said = the_person_says(&approve, doorway, network, grants, strings);
                assert!(said.done().is_some(), "{said:?}");
                assert!(!invoice.is_file(), "the file did not move on the disk");

                let again = the_person_says(&approve, doorway, network, grants, strings);
                let refusal = again.refusal().unwrap();
                assert!(refusal.text().contains(&number.to_string()), "{refusal:?}");
                assert!(!refusal.is_a_bug());
                assert_eq!(doorway.turn().unwrap().waiting_at(noon()).count(), 0);
            },
        );
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Executions))
                .count(),
            1,
            "{record:?}"
        );
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is(reception().as_str()))
        }));
    }

    /// **A change a paired machine proposed names the machine by the name its
    /// person gave it on the person's door** — on the list of what is waiting
    /// and on every record entry the change leaves — **and the name proves
    /// nothing**: the grant it runs under is the one made to reception's
    /// identity, and a proof from a machine no pairing stands with is refused
    /// however the machines here are named.
    #[test]
    fn a_change_from_a_paired_machine_names_it_by_its_given_name_and_the_name_proves_nothing() {
        const CALLED: &str = "the reception machine";
        let mut record = Record::default();
        on_the_studio(
            "reaching-named",
            &mut record,
            |doorway, network, grants, on_reception, strings, folder| {
                let named = the_person_says(
                    &format!(
                        r#"{{"name-machine":{{"machine":"{}","called":"{CALLED}"}}}}"#,
                        reception().as_str()
                    ),
                    doorway,
                    network,
                    grants,
                    strings,
                );
                assert_eq!(named.called(), Some(CALLED), "{named:?}");

                let number =
                    a_change_from_reception(doorway, network, grants, on_reception, folder, noon());
                let said = the_person_says(r#"{"waiting":{}}"#, doorway, network, grants, strings);
                let changes = said.changes().unwrap();
                assert_eq!(changes.len(), 1);
                assert_eq!(changes.first().unwrap().from(), Some(CALLED), "{changes:?}");

                let approve = format!(r#"{{"approve":{{"number":{number}}}}}"#);
                let said = the_person_says(&approve, doorway, network, grants, strings);
                assert!(said.done().is_some(), "{said:?}");
                assert!(!folder.join("march.pdf").is_file());

                // A machine nobody here is paired with, making a proof with a
                // row of its own: refused at the proof, names or no names.
                let warehouse =
                    alo_nearby::MachineId::read("11112222333344445555666677778888").unwrap();
                let (on_warehouse, _) = paired_between(
                    warehouse.clone(),
                    the_studio(),
                    &[MayAskIts::Models],
                    noon(),
                );
                let body = Carried::of(
                    "list_folder",
                    &[("folder", Given::text(folder.to_string_lossy().into_owned()))],
                )
                .said();
                let later = noon() + Duration::from_secs(1);
                let proof = Proof::made(&on_warehouse, &warehouse, body.as_bytes(), later);
                let judged = doorway.judged(
                    Door::Read,
                    Some(&proof),
                    &body,
                    network.locked().pairings(),
                    grants,
                    network.names(),
                    &EgressPolicy::InTheBuilding,
                    later,
                );
                assert!(
                    matches!(
                        judged,
                        Judged::Reply {
                            replying: Replying::BeforeTheDoor(alo_corridor::AtTheDoor::NotProven(
                                alo_nearby::NotProven::NotWithThatMachine
                            )),
                            ..
                        }
                    ),
                    "{judged:?}"
                );
            },
        );
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Executions))
                .count(),
            1,
            "{record:?}"
        );
        assert!(!record.is_empty());
        assert!(
            record
                .everything()
                .all(|entry| entry.origin().is_some_and(|from| from.is(CALLED))),
            "{record:?}"
        );
    }

    /// **No is a whole answer on a remote turn too**: nothing runs, the
    /// number stops waiting, and it is written down with the origin named.
    #[test]
    fn a_change_from_a_paired_machine_the_person_declined_runs_nothing() {
        let mut record = Record::default();
        on_the_studio(
            "reaching-declined",
            &mut record,
            |doorway, network, grants, on_reception, strings, folder| {
                let number =
                    a_change_from_reception(doorway, network, grants, on_reception, folder, noon());
                let said = the_person_says(
                    &format!(r#"{{"decline":{{"number":{number}}}}}"#),
                    doorway,
                    network,
                    grants,
                    strings,
                );
                assert_eq!(said, ToAPerson::Declined);
                assert!(folder.join("march.pdf").is_file(), "a declined change ran");
                assert_eq!(doorway.turn().unwrap().waiting_at(noon()).count(), 0);
            },
        );
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Executions))
                .count(),
            0
        );
        assert!(
            record.everything().any(|entry| matches!(
                entry.happened(),
                alo_record::Happened::Stopped {
                    how: alo_record::Stopped::ByThePerson,
                    ..
                }
            )),
            "{record:?}"
        );
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is(reception().as_str()))
        }));
    }

    /// **A pairing revoked between the proposal and the approval stops the
    /// change at the moment it would have run**: the refusal names the
    /// machine, nothing runs, and the approval is spent.
    #[test]
    fn an_approval_after_the_pairing_was_revoked_runs_nothing() {
        let mut record = Record::default();
        on_the_studio(
            "reaching-revoked",
            &mut record,
            |doorway, network, grants, on_reception, strings, folder| {
                let number =
                    a_change_from_reception(doorway, network, grants, on_reception, folder, noon());
                assert!(network.locked().pairings_mut().revoke(&reception()));

                let said = the_person_says(
                    &format!(r#"{{"approve":{{"number":{number}}}}}"#),
                    doorway,
                    network,
                    grants,
                    strings,
                );
                let refusal = said.refusal().unwrap();
                assert!(refusal.text().contains("no longer paired"), "{refusal:?}");
                assert!(!refusal.is_a_bug());
                assert!(
                    folder.join("march.pdf").is_file(),
                    "a revoked pairing's change ran"
                );
            },
        );
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Executions))
                .count(),
            0
        );
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Refusals))
                .count(),
            1,
            "{record:?}"
        );
    }

    /// **A number nobody proposed is refused on a remote turn** by looking at
    /// what is really waiting, and the real change goes on waiting; and with
    /// the network holding a free machine — no remote turn — the same number
    /// is refused in the same words and nothing is waiting.
    #[test]
    fn a_number_nobody_proposed_answers_nothing_on_a_remote_turn_or_without_one() {
        let mut record = Record::default();
        on_the_studio(
            "reaching-invented",
            &mut record,
            |doorway, network, grants, on_reception, strings, folder| {
                let said = the_person_says(r#"{"waiting":{}}"#, doorway, network, grants, strings);
                assert_eq!(said.changes(), Some([].as_slice()));
                let said = the_person_says(
                    r#"{"approve":{"number":424242}}"#,
                    doorway,
                    network,
                    grants,
                    strings,
                );
                assert!(said.refusal().is_some(), "{said:?}");

                a_change_from_reception(
                    doorway,
                    network,
                    grants,
                    on_reception,
                    folder,
                    noon() + Duration::from_secs(1),
                );
                let said = the_person_says(
                    r#"{"approve":{"number":424242}}"#,
                    doorway,
                    network,
                    grants,
                    strings,
                );
                assert!(said.refusal().is_some(), "{said:?}");
                assert!(folder.join("march.pdf").is_file());
                assert_eq!(doorway.turn().unwrap().waiting_at(noon()).count(), 1);
            },
        );
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Executions))
                .count(),
            0
        );
    }
}
