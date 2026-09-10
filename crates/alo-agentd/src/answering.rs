//! One line from the person's shell, answered against the turn that is under
//! way.
//!
//! [`crate::doing`]'s twin, and it differs in one thing that shapes the whole
//! file: **there may be no turn.** A shell is signed in for as long as the
//! person is, and an agent's turn lasts a few seconds in the middle of that, so
//! every question here has to have an answer for a machine on which nothing is
//! happening.
//!
//! # A number off the wire is not a handle
//!
//! `alo_protocol::FromAPerson` carries a number, because a number is what a
//! shell drew beside a sentence. It is **found** among what is really waiting
//! rather than turned into a handle, and a number nothing is waiting under is
//! the capability model's own refusal. That is what stops a shell — or anything
//! that can reach the person's door — from answering a change by guessing at a
//! number.
//!
//! # Nothing is waiting is an answer, not a refusal
//!
//! *What is waiting* on a machine with no turn under way is an empty list, and
//! it is the same empty list a turn with nothing outstanding gives. A shell
//! drawing the person's changes must not have to tell those apart, because to
//! the person they are one fact: there is nothing to answer.
//!
//! **Answering** something when there is no turn is different, and is refused:
//! a number that is not waiting is a number that is not waiting, whether the
//! turn ended a moment ago or never began. What the person is told is
//! `alo_capability::AnswerError::NothingWaiting`'s sentence, which is the one
//! they would have been told inside a turn — so a change that lapsed while they
//! were reading it does not produce a different explanation depending on
//! whether the agent has hung up yet.
//!
//! # The fourth request is not about the turn at all
//!
//! `alo_protocol::FromAPerson::Granted` is the person saying that what they
//! granted has changed, and their grants outlive every turn — so it is answered
//! the same way whether one is under way or not, and what it reaches is
//! `crate::rereading` rather than anything here. This file does the dispatch and
//! the wording, as it does for the other three, and decides nothing about a
//! grant.
//!
//! It is why what arrives here is a [`Holding`] rather than an
//! `Option<&mut Turning>`: the refusal has to be written down whether or not an
//! agent happens to be connected, and between turns the turn is not there to
//! write it through.

use std::time::SystemTime;

use alo_capability::{AnswerError, Grants, ProposalId};
use alo_keeping::NotKept;
use alo_protocol::{FromAPerson, ToAPerson};
use alo_strings::Strings;
use alo_turn::Turning;

use crate::holding::Holding;
use crate::rereading::{self, WhatIsGranted};

/// Read one line as something the person said, and answer it.
///
/// Answers with what to say back, always. A message that was not a request is
/// refused in `alo-protocol`'s own words, and nothing reaches the turn.
///
/// The grants are borrowed mutably for the one request that replaces them —
/// the person saying what they granted has changed — and that is the only road
/// in this service by which the list a turn is answered against moves while the
/// service is running. Nothing an agent sends reaches it.
///
/// # Errors
///
/// [`NotKept`] when something happened and could not be written down. The
/// service stops on it, for `docs/contracts/daemon-protocol.md`'s reason: a
/// service that went on acting once it could not write a record would be doing
/// exactly what *every execution and every refusal leaves a record* exists to
/// prevent.
pub fn what_a_person_said(
    line: &str,
    holding: &mut Holding<'_, '_, '_>,
    granted: &mut WhatIsGranted<'_>,
    strings: &Strings,
    now: SystemTime,
) -> Result<ToAPerson, NotKept> {
    match FromAPerson::read(line) {
        Ok(FromAPerson::Granted) => what_is_granted_changed(holding, granted, strings, now),
        Ok(answered) => Ok(answered_to(
            answered,
            holding.turning(),
            granted.holding(),
            strings,
            now,
        )),
        Err(why) => Ok(ToAPerson::refused(&why.said(strings))),
    }
}

/// What is granted has changed: read it again, or say why it was not.
///
/// The grant this turn's own invocation made is handed to
/// [`rereading::read_again`] so that it survives the replacement under the
/// handle the turn will end it by. Everything else about the list is the file's.
fn what_is_granted_changed(
    holding: &mut Holding<'_, '_, '_>,
    granted: &mut WhatIsGranted<'_>,
    strings: &Strings,
    now: SystemTime,
) -> Result<ToAPerson, NotKept> {
    let lent = holding.turning().and_then(|turning| turning.granted());
    match granted.read_again(lent, now) {
        Ok(how_many) => Ok(ToAPerson::granted(how_many)),
        Err(why) => {
            // The English goes to whoever is standing the machine up; the
            // person is told one sentence, and it is the same value that is
            // written down, so the screen and the record cannot become two
            // accounts of one moment.
            eprintln!("alo-agentd: {why}");
            let said = rereading::what_to_say(strings);
            holding.the_grants_were_not_read_again(&said, now)?;
            Ok(ToAPerson::refused(&said))
        }
    }
}

/// The three things a person can send about a turn, each against it if there is
/// one.
fn answered_to(
    answered: FromAPerson,
    turning: Option<&mut Turning<'_, '_>>,
    grants: &Grants,
    strings: &Strings,
    now: SystemTime,
) -> ToAPerson {
    let Some(turning) = turning else {
        return match answered.number() {
            Some(number) => nothing_is_waiting(number, strings),
            None => ToAPerson::waiting(std::iter::empty(), strings, now),
        };
    };

    match answered {
        FromAPerson::Waiting => ToAPerson::waiting(turning.waiting_at(now), strings, now),
        // The knock is not about the turn, and the one caller answers it before
        // it reaches here. Unreachable while that is so, and answered rather
        // than assumed away — with the sentence that is true of a machine that
        // did not read its list again, because that is what has happened if
        // this arm is ever taken.
        FromAPerson::Granted => ToAPerson::refused(&rereading::what_to_say(strings)),
        FromAPerson::Approve { number } => match under(turning, number, now) {
            Some(waiting) => match turning.approving(waiting, grants, now) {
                Ok(answer) => ToAPerson::did(&answer),
                Err(why) => ToAPerson::refused(&why.said(strings)),
            },
            None => nothing_is_waiting(number, strings),
        },
        FromAPerson::Decline { number } => match under(turning, number, now) {
            Some(waiting) => match turning.declining(waiting, now) {
                Ok(()) => ToAPerson::Declined,
                Err(why) => ToAPerson::refused(&why.said(strings)),
            },
            None => nothing_is_waiting(number, strings),
        },
    }
}

/// The change really waiting under this number, if one is.
///
/// The borrow ends with the search, so what comes back is a handle the turn
/// made rather than one this file invented from a number somebody sent.
fn under(turning: &Turning<'_, '_>, number: u64, now: SystemTime) -> Option<ProposalId> {
    turning
        .waiting_at(now)
        .find(|waiting| waiting.id.as_u64() == number)
        .map(|waiting| waiting.id)
}

/// The capability model's own sentence for a number nothing is waiting under.
///
/// Borrowed rather than written here for item 9e's reason: the screen and the
/// record render the same value, so neither can be a language the other is not
/// — and a person who answers a change twice reads the same sentence whichever
/// road the second answer took.
fn nothing_is_waiting(number: u64, strings: &Strings) -> ToAPerson {
    ToAPerson::refused(&AnswerError::NothingWaiting { number }.said(strings))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::doing::what_an_agent_said;
    use crate::testing::{
        NothingIsRemembered, a_file_holding, a_message, granting, hour, noon,
        nothing_has_been_chosen, on_a_machine, on_a_machine_with_no_turn,
    };
    use alo_capability::{Ask, Grantee};
    use alo_record::Record;
    use alo_turn::Machine;
    use std::path::Path;

    /// One line from the person's shell, answered against the turn under way.
    ///
    /// Nothing in these is about the grants file, so the file they are given is
    /// the one a machine has on its first morning.
    fn said_in_a_turn(
        line: &str,
        turning: &mut Turning<'_, '_>,
        grants: &mut Grants,
        strings: &Strings,
    ) -> ToAPerson {
        what_a_person_said(
            line,
            &mut Holding::ATurn {
                turning,
                questions: &mut nothing_has_been_chosen(),
            },
            &mut WhatIsGranted::of(grants, &NothingIsRemembered),
            strings,
            noon(),
        )
        .unwrap()
    }

    /// The same, on a machine with no turn under way, and against whichever
    /// grants file the test is about.
    fn said_with_no_turn(
        line: &str,
        machine: &mut Machine<'_>,
        grants: &mut Grants,
        remembering: &dyn crate::rereading::Remembering,
        strings: &Strings,
    ) -> ToAPerson {
        what_a_person_said(
            line,
            &mut Holding::Nobody(machine),
            &mut WhatIsGranted::of(grants, remembering),
            strings,
            noon(),
        )
        .unwrap()
    }

    /// Whether `@files` may reach this path at the moment these tests are at.
    fn may_reach(grants: &Grants, at: &Path) -> bool {
        grants.permits(&Grantee::named("@files"), &Ask::path(at), noon())
    }

    /// Propose a rename, and answer with the number it is waiting under.
    fn a_change_waiting(
        turning: &mut Turning<'_, '_>,
        grants: &Grants,
        strings: &Strings,
        invoice: &Path,
    ) -> u64 {
        let said = what_an_agent_said(
            &a_message(&format!(
                r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"march-final.pdf"}}]}}}}"#,
                invoice.display()
            )),
            turning,
            &mut nothing_has_been_chosen(),
            grants,
            strings,
            hour(),
            noon(),
        );
        assert!(said.waits_for_a_person());
        turning.waiting_at(noon()).next().unwrap().id.as_u64()
    }

    /// **A change proposed on one door is approved on the other**, and the file
    /// only moves when the second message arrives.
    #[test]
    fn a_change_is_approved_and_then_it_runs() {
        on_a_machine("approved", |turning, grants, strings, _, invoice| {
            let number = a_change_waiting(turning, grants, strings, invoice);
            assert!(invoice.is_file(), "a proposal moved a file");

            let said = said_in_a_turn(
                &a_message(&format!(r#"{{"approve":{{"number":{number}}}}}"#)),
                turning,
                grants,
                strings,
            );

            assert!(said.done().is_some(), "{said:?}");
            assert!(!invoice.is_file(), "the file did not move on the disk");
        });
    }

    /// **One approval is one execution.** The second answer finds nothing
    /// waiting under that number, which is the same sentence a number nobody
    /// ever proposed gets.
    #[test]
    fn the_same_number_cannot_be_approved_twice() {
        on_a_machine("twice", |turning, grants, strings, _, invoice| {
            let number = a_change_waiting(turning, grants, strings, invoice);
            let approve = a_message(&format!(r#"{{"approve":{{"number":{number}}}}}"#));

            assert!(
                said_in_a_turn(&approve, turning, grants, strings)
                    .done()
                    .is_some()
            );
            let again = said_in_a_turn(&approve, turning, grants, strings);
            assert!(again.refusal().is_some(), "{again:?}");
        });
    }

    /// **No is a whole answer**, and it is written down: nothing runs, and the
    /// number stops waiting.
    #[test]
    fn a_change_the_person_declined_runs_nothing() {
        on_a_machine("declined", |turning, grants, strings, _, invoice| {
            let number = a_change_waiting(turning, grants, strings, invoice);

            let said = said_in_a_turn(
                &a_message(&format!(r#"{{"decline":{{"number":{number}}}}}"#)),
                turning,
                grants,
                strings,
            );

            assert_eq!(said, ToAPerson::Declined);
            assert!(invoice.is_file(), "a declined change ran anyway");
            assert_eq!(turning.waiting_at(noon()).count(), 0);
        });
    }

    /// **What is waiting is the turn's own list**, each change carrying the
    /// sentence the person is being asked about rather than a number alone.
    #[test]
    fn what_is_waiting_comes_back_with_its_sentences() {
        on_a_machine("waiting", |turning, grants, strings, _, invoice| {
            a_change_waiting(turning, grants, strings, invoice);

            let said = said_in_a_turn(&a_message(r#"{"waiting":{}}"#), turning, grants, strings);

            let changes = said.changes().unwrap();
            assert_eq!(changes.len(), 1);
            assert!(
                changes
                    .first()
                    .unwrap()
                    .sentence()
                    .text()
                    .contains("march-final.pdf")
            );
        });
    }

    /// **Nothing is waiting on a machine with no turn**, and it is an answer
    /// rather than a refusal: a shell drawing the person's changes must not
    /// have to tell *no turn* from *nothing outstanding*, because to them they
    /// are one fact.
    #[test]
    fn a_machine_with_no_turn_says_nothing_is_waiting() {
        let mut record = Record::default();
        on_a_machine_with_no_turn("no-turn", &mut record, |machine, grants, strings, _, _| {
            let said = said_with_no_turn(
                &a_message(r#"{"waiting":{}}"#),
                machine,
                grants,
                &NothingIsRemembered,
                strings,
            );
            assert_eq!(said.changes(), Some([].as_slice()));
        });
    }

    /// **Answering when there is no turn is refused**, in the same words a
    /// number nothing is waiting under gets inside one — so a person whose
    /// agent has hung up while they were reading is told what happened rather
    /// than told something about a daemon.
    #[test]
    fn answering_with_no_turn_under_way_is_refused_in_the_ordinary_words() {
        let mut record = Record::default();
        on_a_machine_with_no_turn(
            "no-turn-answer",
            &mut record,
            |machine, grants, strings, _, _| {
                for line in [
                    a_message(r#"{"approve":{"number":7}}"#),
                    a_message(r#"{"decline":{"number":7}}"#),
                ] {
                    let said =
                        said_with_no_turn(&line, machine, grants, &NothingIsRemembered, strings);
                    let refusal = said.refusal().unwrap();
                    assert!(refusal.text().contains('7'), "{refusal:?}");
                    assert!(!refusal.is_a_bug());
                }
            },
        );
    }

    /// **A grant made while the service is running is honoured in the same
    /// session**, without a restart: the person's side says what is granted has
    /// changed, the machine reads its own file, and the folder it was refusing a
    /// moment ago is one it permits.
    #[test]
    fn a_grant_made_now_reaches_a_running_service() {
        let mut record = Record::default();
        on_a_machine_with_no_turn(
            "granted-knock",
            &mut record,
            |machine, grants, strings, folder, invoice| {
                let file = a_file_holding("granted-knock-file", &granting(folder, noon()), noon());
                // The service is serving under an empty list, as one that signed
                // in before the folder was picked would be.
                *grants = Grants::default();
                assert!(!may_reach(grants, invoice));

                let said = said_with_no_turn(
                    &a_message(r#"{"granted":{}}"#),
                    machine,
                    grants,
                    &file,
                    strings,
                );

                assert_eq!(said.holding(), Some(1), "{said:?}");
                assert!(
                    may_reach(grants, invoice),
                    "the grant the person made did not reach the running service"
                );
            },
        );
        assert_eq!(
            record.len(),
            0,
            "reading the grants again is not something that happened to an agent"
        );
    }

    /// **A revocation takes effect on the next question asked.** The file is
    /// what is granted, so a grant that is no longer in it is one that was
    /// revoked — and the answer is the grants saying no rather than the older
    /// list saying yes.
    #[test]
    fn a_revocation_takes_effect_on_the_next_question() {
        let mut record = Record::default();
        on_a_machine_with_no_turn(
            "revoked-knock",
            &mut record,
            |machine, grants, strings, _, invoice| {
                let file = a_file_holding("revoked-knock-file", &Grants::default(), noon());
                assert!(may_reach(grants, invoice), "nothing was granted to revoke");

                let said = said_with_no_turn(
                    &a_message(r#"{"granted":{}}"#),
                    machine,
                    grants,
                    &file,
                    strings,
                );

                assert_eq!(said.holding(), Some(0), "{said:?}");
                assert!(!may_reach(grants, invoice), "a revoked grant came back");
            },
        );
    }

    /// **A grants file that has stopped being believable leaves the grants the
    /// service already had**, says so in the person's own language, and the
    /// refusal is written down. A machine that forgot what was granted because
    /// somebody chmodded a file is a machine that went silent.
    #[test]
    fn an_unbelievable_grants_file_leaves_the_grants_alone_and_is_written_down() {
        use std::os::unix::fs::PermissionsExt as _;

        let mut record = Record::default();
        on_a_machine_with_no_turn(
            "unbelievable-knock",
            &mut record,
            |machine, grants, strings, folder, invoice| {
                let file =
                    a_file_holding("unbelievable-knock-file", &granting(folder, noon()), noon());
                std::fs::set_permissions(file.path(), std::fs::Permissions::from_mode(0o666))
                    .unwrap();

                let said = said_with_no_turn(
                    &a_message(r#"{"granted":{}}"#),
                    machine,
                    grants,
                    &file,
                    strings,
                );

                let refusal = said.refusal().unwrap();
                assert!(!refusal.is_a_bug(), "{refusal:?}");
                assert!(
                    refusal.text().contains("nothing was widened"),
                    "{refusal:?}"
                );
                assert!(
                    may_reach(grants, invoice),
                    "the grants were emptied by a file nobody could believe"
                );
            },
        );
        assert_eq!(record.len(), 1, "the refusal was not written down");
        let only = record.everything().next().unwrap();
        assert!(
            matches!(
                only.happened(),
                alo_record::Happened::GrantsNotReadAgain { .. }
            ),
            "{only:?}"
        );
        assert!(
            only.agent().is_none(),
            "an agent was named for something the person's own side did"
        );
    }

    /// **A person cannot ask for a read**, and the refusal is `alo-protocol`'s:
    /// the two doors are two before anything reaches a turn.
    #[test]
    fn a_person_asking_for_a_read_is_refused_before_the_turn() {
        on_a_machine("wrong-door", |turning, grants, strings, folder, _| {
            let said = said_in_a_turn(
                &a_message(&format!(
                    r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
                    folder.display()
                )),
                turning,
                grants,
                strings,
            );

            assert!(said.refusal().is_some(), "{said:?}");
        });
    }

    /// **A number nobody proposed is refused**, and it is refused by looking at
    /// what is really waiting rather than by trusting the number.
    #[test]
    fn a_number_nobody_proposed_answers_nothing() {
        on_a_machine("invented", |turning, grants, strings, _, invoice| {
            a_change_waiting(turning, grants, strings, invoice);

            let said = said_in_a_turn(
                &a_message(r#"{"approve":{"number":424242}}"#),
                turning,
                grants,
                strings,
            );

            assert!(said.refusal().is_some(), "{said:?}");
            assert!(invoice.is_file(), "an invented number ran a change");
            assert_eq!(
                turning.waiting_at(noon()).count(),
                1,
                "the real change stopped waiting"
            );
        });
    }
}
