//! A question the person asked themselves, put where their settings say.
//!
//! The one request on the person's door that reaches off this machine, and the
//! only road to a model in this service that is **not an agent turn**.
//!
//! # Why the daemon answers it at all
//!
//! [ADR 0125](../../../docs/decisions/0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)
//! sent a person's question here because a command that exits in a second can
//! show nothing and keep nothing.
//! [ADR 0126](../../../docs/decisions/0126-a-persons-own-question-is-a-third-authority-on-the-indicator-and-in-the-record.md)
//! took that reason away — a person's own question goes on no indicator,
//! because they chose their provider in Settings — and then **measured what was
//! left**, which is the reason this file is here rather than in `alo-ask`:
//!
//! | what a person's question needs first | who can do it |
//! |---|---|
//! | the organisation's rule, read with the checks that guard the file naming which login is the agent | 1710 lines inside this crate, which nothing depends on |
//! | **signing in to a proxy that asks who this machine is** | this unit alone — the password is a systemd credential at `/run/credentials/alo-agentd.service` (ADR 0059) |
//! | the person's provider key | whoever holds the keyring, which is this service for the person it runs as |
//!
//! A command that went straight out instead of through the proxy would repeat a
//! fault already found and fixed here: `crate::the_road_out`'s own header
//! records *a machine read its organisation's proxy off the disk and then asked
//! a provider directly.*
//!
//! # No turn, no grant, no approval, and no agent named
//!
//! Nothing here consults a grant, proposes anything, or takes a bracket. A
//! question is a read of a model, and a person reading an answer on their own
//! machine is not an agent acting — ADR 0125 marked conflating the two *the
//! part most likely to be got wrong by whoever builds it*, and the protocol
//! keeps them apart with two variants rather than one and a flag.
//!
//! # What is still asked, in the order it is asked in
//!
//! The ordering is [`crate::a_provider_ready_to_be_asked`]'s and is shared with
//! the agent's road rather than written again: **the organisation's rule, then
//! which way out of this machine, then the person's key.** A policy refusal
//! never reaches for a credential. Then
//! `alo_asking::a_question_nobody_is_showing` asks the same rule about the
//! **resolved destination**, which is a different subject from the provider's
//! stated source and not a second asking of one question.

use alo_asking::{NotPut, Question, a_question_nobody_is_showing};
use alo_protocol::ToAPerson;
use alo_strings::{Filling, Said, Strings};

use crate::a_provider_ready_to_be_asked::{NotPrepared, a_provider_ready_to_be_asked};
use crate::doing::{a_rule_refused, said_about};
use crate::questions::{Questions, WhatAnswers};
use crate::words::{NOTHING_ANSWERS_QUESTIONS, NOTHING_WAS_ASKED};

/// Answer the person's own question, or say in one sentence why not.
///
/// `questions` is what their settings and this machine's runtime come out of,
/// looked for at the question rather than at start-up because a person may
/// change their mind between two questions.
/// **No moment is passed in and there is no parameter for one.** Everything
/// else on this door takes `now`, because everything else writes something
/// down. Nothing here does — a person's own question goes on no indicator and
/// into no record ([ADR 0126](../../../docs/decisions/0126-a-persons-own-question-is-a-third-authority-on-the-indicator-and-in-the-record.md))
/// — so an unused moment would be the one thing in this file that read as
/// though something were kept.
pub fn answered(question: &str, questions: &mut Questions, strings: &Strings) -> ToAPerson {
    // Taken before `what_answers` borrows `questions`, exactly as
    // `crate::doing::put_to_a_model` takes them.
    let keyring = questions.whose_keyring();
    let by_an_organisation = questions.by_an_organisation();
    let the_road_out = questions.road_out();

    match questions.what_answers() {
        // The three a person is owed in somebody else's words: *choose
        // something* is this crate's, *the runtime is not reachable* is
        // `alo-models`', *your settings file says this* is `alo-choosing`'s.
        // None of them is reworded here.
        WhatAnswers::Nothing => {
            ToAPerson::refused(&strings.say(&NOTHING_ANSWERS_QUESTIONS.key(), &Filling::nothing()))
        }
        WhatAnswers::NotRunning => {
            ToAPerson::refused(&alo_models::RuntimeError::Unreachable.said(strings))
        }
        WhatAnswers::NotSet(why) => ToAPerson::refused(&why.said(strings)),
        // **A question answered on this machine never reaches this file.**
        // `alo-ask` answers a local model itself and always has: it causes no
        // egress, needs no credential and needs no proxy, so there is nothing
        // here it could not do. A person whose settings name this machine is
        // told so rather than silently answered on a road built for a
        // provider, because a daemon that answered it would be a second place
        // a local question is put.
        // And a machine down the corridor is the person's own too, and is not
        // this road either: `crate::corridor` asks the bound, the pairing and
        // the network in that order, against a turn.
        //
        // Both get the one sentence that is true of a question this machine
        // did not put anywhere, which is `crate::doing`'s rule for a
        // miswiring: there is nothing for a person to do about this repository
        // disagreeing with itself, so they are told what is true and useful
        // rather than something inventive.
        WhatAnswers::OnThisMachine { .. } | WhatAnswers::FromAPairedMachine { .. } => {
            ToAPerson::refused(&nothing_was_asked(strings))
        }
        WhatAnswers::FromAProvider {
            provider,
            model,
            places,
        } => {
            // **The question is built where the model is known**, because
            // `alo_asking::Question::asked` takes both and refuses an empty
            // one. A person who pressed return by mistake is owed *nothing was
            // asked* rather than a report about their provider.
            let Ok(asked) = Question::asked(question, model) else {
                return ToAPerson::refused(&nothing_was_asked(strings));
            };
            let prepared = a_provider_ready_to_be_asked(
                provider,
                places.policy(),
                &the_road_out,
                &keyring,
                |permission, hosted| {
                    // Resolved here and not by the request, which is ADR 0020's
                    // separation — `alo_turn::asking::registering`'s own doc
                    // says this is the daemon's to do and that it is not a
                    // turn's.
                    let to = alo_turn::asking::registering(hosted.where_it_would_connect());
                    a_question_nobody_is_showing(
                        permission,
                        places.everywhere_else(),
                        places.policy(),
                        &asked,
                        &hosted,
                        &to,
                    )
                },
            );

            match prepared {
                Ok(Ok(answer)) => {
                    ToAPerson::answered(answer.text(), &answer.came_from(strings), answer.model())
                }
                Ok(Err(why)) => ToAPerson::refused(&nothing_was_put(&why, strings)),
                // The three the shared ordering refuses, worded exactly as the
                // agent's road words them — `crate::doing::a_rule_refused` and
                // `said_about` are shared for that reason, so a person and an
                // agent refused by one rule read one sentence.
                Err(NotPrepared::ARuleRefused(why)) => {
                    ToAPerson::refused(&a_rule_refused(&why, by_an_organisation, strings))
                }
                Err(NotPrepared::NoRoadOut(why)) => ToAPerson::refused(&why.said(strings)),
                Err(NotPrepared::TheKeyIsNotStored(why)) => {
                    ToAPerson::refused(&strings.say(&said_about(why).key(), &Filling::nothing()))
                }
            }
        }
    }
}

/// The sentence for a question that was not put, or was put and not answered.
///
/// `alo_asking::NotPut` carries the value that decided each one, and each is
/// worded by whoever knows it: the rule by `alo-egress`, the failure by
/// `alo-answering`. The two that are this repository disagreeing with itself
/// have no sentence of their own, because there is nothing for a person to do
/// about them — so they get the one thing that is true and useful.
fn nothing_was_put(why: &NotPut, strings: &Strings) -> Said {
    match why {
        NotPut::HeldBack(refused) => strings.say(&refused.word().key(), &Filling::nothing()),
        NotPut::DidNotAnswer(failed) => failed.said(strings),
        NotPut::NotSomewhereNameable(_) | NotPut::Miswired(_) | NotPut::NotWhatFailed(_) => {
            nothing_was_asked(strings)
        }
    }
}

/// Nothing was asked, said once.
fn nothing_was_asked(strings: &Strings) -> Said {
    strings.say(&NOTHING_WAS_ASKED.key(), &Filling::nothing())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_models::{Catalogue, SourcePolicy};

    use super::answered;
    use crate::questions::{Questions, TheBound, WhoseKeyring};
    use crate::testing::{a_directory_of_our_own, in_english};

    /// Settings naming one provider the person chose, with a key.
    ///
    /// The key is the instrument, as it is in
    /// `crate::a_provider_ready_to_be_asked`: a refusal that names the keyring
    /// is a refusal that opened one.
    fn a_person_who_chose_a_provider(called: &str) -> std::ffi::OsString {
        let config = a_directory_of_our_own(called);
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        // **The contract's own example, and no credential in it.** A file that
        // invents a field for a key is refused *whole*
        // (`docs/contracts/person-settings.md`), and a fixture that did so
        // made every test below pass on *your settings cannot be read* \u2014
        // measured 2026-10-19 by mutating the rule away and watching nothing
        // fail.
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            "format = 3\n\n[answers]\nprovider = { name = \"Mistral\", model = \
             \"mistral-small-latest\" }\n\n[[provider]]\nname = \"Mistral\"\nendpoint = \
             \"https://api.mistral.ai\"\nregion = \"the EU\"\n",
        )
        .unwrap();
        config.into_os_string()
    }

    /// A machine with these settings and this rule, and nobody's keyring.
    fn a_machine(called: &str, bound: TheBound) -> Questions {
        Questions::of_a_session(
            Some(a_person_who_chose_a_provider(called)),
            None,
            Catalogue::built_in().unwrap(),
            bound,
            // Nobody's: every lookup is `NotStored::Unavailable`, so a keyring
            // opened out of turn shows up as the **wrong refusal** rather than
            // as nothing at all.
            WhoseKeyring::Nobodys,
        )
    }

    /// **The organisation's rule still refuses a person's own question**, and
    /// it refuses before the keyring is opened.
    ///
    /// This is the test ADR 0126 asks for by name: the indicator went away and
    /// the rule did not, because they are different things — law 1 and
    /// ADR 0016. A person on a machine set to keep questions in the building is
    /// refused in words, nothing is sent, and their credential is never
    /// reached for.
    #[test]
    fn a_machine_that_keeps_questions_in_the_building_refuses_the_persons_own() {
        let strings = in_english();
        let mut questions = a_machine(
            "persons-question-bound",
            TheBound::AnOrganisations(SourcePolicy::ThisMachineOnly),
        );

        let said = answered("may the tenant sublet?", &mut questions, &strings);
        let refusal = said.refusal().expect("a rule refused it");

        assert!(!refusal.is_a_bug(), "{refusal:?}");
        // The rule's own sentence, not the store's. `alo-secrets` words a
        // keyring that will not answer, and none of its four reasons may appear
        // here — if one does, something opened a keyring on the way to saying
        // no.
        // **Which refusal, not that one happened.** With nobody's keyring there
        // is always *a* refusal, so an assertion that one came back passes with
        // the rule removed entirely — measured 2026-10-19 by removing it and
        // watching four tests stay green. So this names the rule's own
        // sentence, and names the administrator's half of it.
        assert_eq!(
            refusal.text(),
            "this machine is set to answer only on itself, and by Mistral, in the EU is somewhere \
             else — an administrator set that rule for this machine, so it is not yours to change",
            "the rule did not refuse it; something else did"
        );
    }

    /// A machine where nobody has chosen anything says so, in this crate's own
    /// sentence, and asks nothing of anybody.
    #[test]
    fn a_machine_where_nobody_chose_anything_says_so() {
        let strings = in_english();
        let config = a_directory_of_our_own("persons-question-nothing");
        let mut questions = Questions::of_a_session(
            Some(config.into_os_string()),
            None,
            Catalogue::built_in().unwrap(),
            TheBound::Nobodys,
            WhoseKeyring::Nobodys,
        );

        let said = answered("may the tenant sublet?", &mut questions, &strings);
        let refusal = said.refusal().expect("nothing has been chosen");
        assert!(!refusal.is_a_bug(), "{refusal:?}");
    }

    /// **Nothing was asked** is what an empty question gets, and it is got
    /// before anything about a provider is said.
    ///
    /// A person who pressed return by mistake is owed this rather than a report
    /// about their provider or their keyring.
    #[test]
    fn an_empty_question_is_refused_as_nothing_asked() {
        let strings = in_english();
        let mut questions = a_machine("persons-question-empty", TheBound::Nobodys);

        let said = answered("   ", &mut questions, &strings);
        let refusal = said.refusal().expect("that was not a question");
        assert!(!refusal.is_a_bug(), "{refusal:?}");
    }

    /// Every refusal on this road is somebody's sentence and none of them is a
    /// marked key, which is what a person actually reads.
    #[test]
    fn every_refusal_this_road_makes_is_a_real_sentence() {
        let strings = in_english();
        for (called, bound) in [
            (
                "persons-question-every-bound",
                TheBound::AnOrganisations(SourcePolicy::ThisMachineOnly),
            ),
            ("persons-question-every-region", {
                TheBound::AnOrganisations(SourcePolicy::InRegion("the EU".to_owned()))
            }),
            ("persons-question-every-none", TheBound::Nobodys),
        ] {
            let mut questions = a_machine(called, bound);
            let said = answered("may the tenant sublet?", &mut questions, &strings);
            let refusal = said
                .refusal()
                .expect("nobody's keyring holds this provider's key");
            assert!(!refusal.is_a_bug(), "{called}: {refusal:?}");
            assert!(!refusal.text().is_empty(), "{called}");
        }
    }
}
