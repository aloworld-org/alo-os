//! A turn's question to the machine its person chose on this network.
//!
//! *A machine without a GPU discovers the one with it, and the agents just
//! work.* `alo_asking::Asking::to_a_paired_machine` is the corridor as a door,
//! `alo_turn::Answers::PairedMachine` is a turn holding it, and
//! `crate::questioned` is the machine at the other end answering. This file is
//! the join on the asking side: the person's choice, as `alo-choosing` read it,
//! turned into a question down the corridor — or into the one sentence that
//! says why nothing was sent.
//!
//! # In this order, and every step before anything leaves
//!
//! 1. **The organisation's bound**, asked of the place the person chose
//!    (`SourcePolicy::InTheBuilding` and its siblings). A refusal is written
//!    down exactly as a provider's is — `crate::doing` words and keeps it —
//!    and nothing about the network is asked.
//! 2. **The pairing, again.** A choice was refused at choosing when no pairing
//!    let the machine answer (`alo_choosing::AMachine::permitted`), and a
//!    pairing can be revoked or run out since: so it is asked again, of the
//!    very list behind the lock that refuses that machine's next verb, at the
//!    moment of this question. A refusal says nothing was sent, and why.
//! 3. **Where it is, measured now.** An address is never kept or typed
//!    (ADR 0003): the machine is looked for by its identity, as the person's
//!    door looks for one to pair with, and one that does not answer is not
//!    asked.
//! 4. **The question**, through the turn, under a departure the indicator
//!    shows and the record keeps as left, from inside a boundary permitting
//!    the one address discovery measured.
//!
//! # Never a fallback, and never a default
//!
//! This file is reached only when the person's settings name a paired
//! machine, and every refusal in it ends the question: nothing here asks the
//! runtime on this machine or a provider instead, and nothing elsewhere asks
//! a paired machine because something else did not answer. What the machine
//! down the corridor says when it will not answer reaches the agent as that
//! machine's own word, rendered here (`alo_answering::RefusedThere`).
//!
//! # A name, where the person here gave one
//!
//! The indicator and the record name the machine by what the person here
//! called it (`alo_corridor::Naming`), and by its identity until somebody has
//! named it — the same rule `alo_nearby::Origin` keeps on the answering side.

use std::time::SystemTime;

use alo_answering::Answering;
use alo_asking::DownTheCorridor;
use alo_choosing::{AMachine, WHAT_THAT_MACHINE_CHOSE, WhoMayBeAsked};
use alo_corridor::Naming;
use alo_nearby::{MachineId, MayAskIts};
use alo_protocol::{Answered, ToAnAgent};
use alo_strings::{Filling, Strings};
use alo_turn::{Answers, Places, Turning};

use crate::doing::{nothing_answered, putting, refused_by_a_rule};
use crate::looking::LookingFor;
use crate::network::{Shared, TheNetwork};
use crate::words::{THE_CHOSEN_MACHINE_DID_NOT_ANSWER, THE_CHOSEN_MACHINE_IS_NOT_PAIRED, Word};

/// What a question down the corridor needs of this machine's neighbours.
///
/// Three references rather than three arguments, for `crate::pairing::Nearby`'s
/// reason: they are one fact — who this machine is paired with, where they
/// are now, and what the person here calls them.
#[derive(Clone, Copy)]
pub struct Corridor<'a> {
    /// The pairings, behind the one lock.
    pub network: &'a TheNetwork,
    /// Where a machine is looked for by its identity, at the moment.
    pub looking: &'a dyn LookingFor,
    /// What the person here called the machines they paired with.
    pub naming: &'a dyn Naming,
}

impl std::fmt::Debug for Corridor<'_> {
    /// Written by hand because the naming is a trait object.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Corridor")
            .field("network", &self.network)
            .field("looking", &self.looking)
            .finish_non_exhaustive()
    }
}

impl WhoMayBeAsked for Shared {
    /// Asked of the one list the wire refuses a revoked machine with.
    ///
    /// Something that is not a machine's identity is paired with nothing.
    fn may_ask_the_models_of(&self, machine: &str, now: SystemTime) -> bool {
        MachineId::read(machine)
            .is_ok_and(|machine| self.pairings().permits(&machine, MayAskIts::Models, now))
    }
}

/// Put an agent's question to the paired machine its person chose, or say in
/// one sentence why nothing was sent.
///
/// `corridor` is [`None`] where this door has no network behind it — a test
/// that is not about one — and a paired choice there is paired with nothing,
/// which is exactly what it is refused as.
#[expect(
    clippy::too_many_arguments,
    reason = "what a question takes, the choice and its place, and whether an organisation set \
              the bound; a struct would exist only to be unpacked on the next line"
)]
pub(crate) fn put_down_the_corridor(
    chosen: &AMachine,
    places: &Places<'_>,
    corridor: Option<&Corridor<'_>>,
    question: &str,
    answered: Answered,
    turning: &mut Turning<'_, '_>,
    by_an_organisation: bool,
    strings: &Strings,
    now: SystemTime,
) -> ToAnAgent {
    let identity = MachineId::read(chosen.machine()).ok();
    let called = corridor
        .zip(identity.as_ref())
        .and_then(|(corridor, machine)| corridor.naming.called(machine))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| chosen.machine().to_owned());
    let not_sent = |word: Word| ToAnAgent::refused(&strings.say(&word.key(), &Filling::nothing()));

    // 1. The bound, before anything about the network is asked.
    let source = alo_models::InferenceSource::PairedMachine {
        machine: called.clone(),
    };
    let permission = match Answering::chosen(source, places.policy()) {
        Ok(permission) => permission,
        Err(why) => return refused_by_a_rule(turning, &why, by_an_organisation, strings, now),
    };

    // 2. The pairing, as it stands at this question.
    let (Some(corridor), Some(identity)) = (corridor, identity) else {
        return not_sent(THE_CHOSEN_MACHINE_IS_NOT_PAIRED);
    };
    if chosen
        .still_permitted(&*corridor.network.locked(), now)
        .is_err()
    {
        return not_sent(THE_CHOSEN_MACHINE_IS_NOT_PAIRED);
    }

    // 3. Where it answers, measured now — with the lock not held, because
    // looking waits on the network.
    let Some(found) = corridor.looking.look_for(&identity) else {
        return not_sent(THE_CHOSEN_MACHINE_DID_NOT_ANSWER);
    };

    // The corridor is made under the lock and holds a copy of the pairing, so
    // a revocation in the moment since step 2 is refused here rather than
    // raced past.
    let down_the_corridor = {
        let shared = corridor.network.locked();
        DownTheCorridor::paired(
            shared.pairings(),
            shared.proposals().here(),
            &identity,
            &called,
            found.where_it_answers(),
            None,
            now,
        )
    };
    let Ok(down_the_corridor) = down_the_corridor else {
        return not_sent(THE_CHOSEN_MACHINE_IS_NOT_PAIRED);
    };

    // 4. The question, through the turn.
    match putting(
        turning,
        answered,
        question,
        WHAT_THAT_MACHINE_CHOSE,
        permission,
        &Answers::PairedMachine(down_the_corridor),
        places,
        now,
    ) {
        Ok(answer) => {
            ToAnAgent::answered(answer.text(), &answer.came_from(strings), answer.model())
        }
        Err(why) => ToAnAgent::refused(&nothing_answered(&why, strings)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::Cell;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use alo_capability::Grants;
    use alo_models::{Catalogue, SourcePolicy};
    use alo_record::{Asking as AskingAbout, Only, Record};

    use super::*;
    use crate::doing::what_an_agent_said;
    use crate::questions::{Questions, TheBound, WhoseKeyring};
    use crate::terms::NoNameYet;
    use crate::testing::{
        TheStudioIsAt, a_directory_of_our_own, a_message, hour, noon, on_a_machine_that_answers,
        paired_between, reception, the_studio, the_studio_answering,
    };

    /// An agent's question in words.
    const ASKED: &str = r#"{"ask":{"question":"how many invoices are unpaid?"}}"#;

    /// Reception's machine, whose person chose the studio to answer their
    /// questions — under `bound`.
    fn reception_choosing_the_studio(what: &str, bound: TheBound) -> Questions {
        let config = a_directory_of_our_own(what);
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            format!(
                "format = 3\n\n[answers]\nmachine = \"{}\"\n",
                the_studio().as_str()
            ),
        )
        .unwrap();
        Questions::of_a_session(
            Some(config.into_os_string()),
            None,
            Catalogue::built_in().unwrap(),
            bound,
            WhoseKeyring::Nobodys,
        )
    }

    /// Reception's pairings, holding its row of a pairing with the studio made
    /// at noon for a day, permitting its models.
    fn reception_paired_with_the_studio() -> TheNetwork {
        let network = TheNetwork::on(reception());
        let (on_reception, _) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        network.locked().pairings_mut().keep(on_reception);
        network
    }

    /// How many entries say something left.
    fn departures(record: &Record) -> usize {
        record
            .answering(&AskingAbout::anything().only(Only::Egress))
            .count()
    }

    /// The sentence this crate says for `word`.
    fn saying(strings: &Strings, word: Word) -> String {
        strings
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    }

    /// **A question to the machine the person chose is answered down the
    /// corridor — and once the pairing is revoked, the very next question is
    /// refused in words, with nothing sent.**
    #[test]
    fn a_pairing_revoked_between_two_questions_refuses_the_second_in_words() {
        let mut questions = reception_choosing_the_studio("revoked", TheBound::Nobodys);
        let network = reception_paired_with_the_studio();
        let (at, heard) = the_studio_answering();
        let looking = TheStudioIsAt {
            at,
            looked: Cell::new(0),
        };
        let corridor = Corridor {
            network: &network,
            looking: &looking,
            naming: &NoNameYet,
        };
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let first = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );
            assert!(
                matches!(&first, ToAnAgent::Answered { text, .. } if text == "Three are unpaid."),
                "{first:?}"
            );

            assert!(network.locked().pairings_mut().revoke(&the_studio()));

            let second = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );
            assert_eq!(
                second.refusal().unwrap().text(),
                saying(strings, THE_CHOSEN_MACHINE_IS_NOT_PAIRED)
            );
        });

        assert_eq!(
            heard.load(Ordering::SeqCst),
            1,
            "the second question was sent"
        );
        assert_eq!(looking.looked.get(), 1, "a revoked machine was looked for");
        assert_eq!(departures(&record), 1);
    }

    /// A network where the studio answers, remembering every identity it was
    /// asked for — so a test can say what a machine was looked for *by*.
    #[derive(Debug)]
    struct LookedForBy {
        /// The studio's network.
        studio: TheStudioIsAt,
        /// Every identity looked for, in order.
        asked: std::cell::RefCell<Vec<MachineId>>,
    }

    impl LookingFor for LookedForBy {
        fn look_for(&self, machine: &MachineId) -> Option<alo_nearby::Found> {
            self.asked.borrow_mut().push(machine.clone());
            self.studio.look_for(machine)
        }
    }

    /// **A question down the corridor names the machine by the name its
    /// person gave it on the person's door** — the departure the indicator
    /// shows and the record entry it leaves (`Entry::left` is made only from
    /// the departure the indicator hands out) say *the studio machine*, and so
    /// does where the answer came from — **and the name never crosses**: what
    /// the studio heard holds no byte of it, and the machine was looked for by
    /// its identity and nothing else.
    #[test]
    fn a_question_down_the_corridor_names_the_machine_by_its_name_and_the_name_never_crosses() {
        use crate::answering::what_a_person_said;
        use crate::holding::Holding;
        use crate::pairing::Nearby;
        use crate::rereading::WhatIsGranted;
        use crate::testing::{
            NobodyIsNearby, NothingIsRemembered, the_studio_answering_and_keeping,
        };

        const CALLED: &str = "the studio machine";
        let mut questions = reception_choosing_the_studio("named", TheBound::Nobodys);
        let network = reception_paired_with_the_studio();
        let (at, heard, kept) = the_studio_answering_and_keeping();
        let looking = LookedForBy {
            studio: TheStudioIsAt {
                at,
                looked: Cell::new(0),
            },
            asked: std::cell::RefCell::new(Vec::new()),
        };
        let corridor = Corridor {
            network: &network,
            looking: &looking,
            naming: network.names(),
        };
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let named = what_a_person_said(
                &a_message(&format!(
                    r#"{{"name-machine":{{"machine":"{}","called":"{CALLED}"}}}}"#,
                    the_studio().as_str()
                )),
                &mut Holding::ATurn {
                    turning: &mut *turning,
                    questions: &mut questions,
                },
                &mut WhatIsGranted::of(&mut Grants::default(), &NothingIsRemembered),
                &Nearby {
                    network: &network,
                    looking: &NobodyIsNearby,
                },
                strings,
                noon(),
            )
            .unwrap();
            assert_eq!(named.called(), Some(CALLED), "{named:?}");

            let said = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );
            let ToAnAgent::Answered {
                text, came_from, ..
            } = &said
            else {
                unreachable!("{said:?}")
            };
            assert_eq!(text, "Three are unpaid.");
            assert!(came_from.text().contains(CALLED), "{came_from:?}");
            assert!(
                !came_from.text().contains(the_studio().as_str()),
                "{came_from:?}"
            );
        });

        assert!(
            record.everything().any(|entry| matches!(
                entry.happened(),
                alo_record::Happened::Left {
                    destination: alo_egress::Destination::PairedMachine { machine },
                    ..
                } if machine == CALLED
            )),
            "{record:?}"
        );
        assert_eq!(departures(&record), 1);
        assert_eq!(heard.load(Ordering::SeqCst), 1);
        let crossed = kept.lock().unwrap();
        assert_eq!(crossed.len(), 1);
        assert!(
            !crossed.first().unwrap().contains(CALLED),
            "the name crossed to the other machine: {crossed:?}"
        );
        assert_eq!(*looking.asked.borrow(), vec![the_studio()]);
    }

    /// **A pairing that ran out since the choice refuses the question in the
    /// same words**, before the network is asked anything.
    #[test]
    fn a_pairing_that_ran_out_since_choosing_refuses_the_question_in_words() {
        let mut questions = reception_choosing_the_studio("expired", TheBound::Nobodys);
        let network = reception_paired_with_the_studio();
        let (at, heard) = the_studio_answering();
        let looking = TheStudioIsAt {
            at,
            looked: Cell::new(0),
        };
        let corridor = Corridor {
            network: &network,
            looking: &looking,
            naming: &NoNameYet,
        };
        let two_days_later = noon() + Duration::from_secs(2 * 86_400);
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                &Grants::default(),
                strings,
                hour(),
                two_days_later,
            );
            assert_eq!(
                said.refusal().unwrap().text(),
                saying(strings, THE_CHOSEN_MACHINE_IS_NOT_PAIRED)
            );
        });

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(heard.load(Ordering::SeqCst), 0);
        assert_eq!(looking.looked.get(), 0);
        assert_eq!(departures(&record), 0);
    }

    /// **An organisation's bound is asked before anything leaves**: a rule
    /// permitting only this machine refuses the paired machine, written down as
    /// a refusal, and nobody is looked for.
    #[test]
    fn an_organisations_bound_refuses_the_paired_machine_before_the_network_is_asked() {
        let mut questions = reception_choosing_the_studio(
            "bounded",
            TheBound::AnOrganisations(SourcePolicy::ThisMachineOnly),
        );
        let network = reception_paired_with_the_studio();
        let (at, heard) = the_studio_answering();
        let looking = TheStudioIsAt {
            at,
            looked: Cell::new(0),
        };
        let corridor = Corridor {
            network: &network,
            looking: &looking,
            naming: &NoNameYet,
        };
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );
            assert!(said.refusal().is_some(), "{said:?}");
        });

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(heard.load(Ordering::SeqCst), 0);
        assert_eq!(
            looking.looked.get(),
            0,
            "the network was asked before the bound"
        );
        assert_eq!(departures(&record), 0);
        assert_eq!(record.len(), 1, "the rule's refusal was not written down");
    }

    /// **A machine the person chose that does not answer on the network is not
    /// asked, and nothing else is asked instead** — the words say so.
    #[test]
    fn a_chosen_machine_not_on_the_network_is_refused_and_nothing_else_answers() {
        let mut questions = reception_choosing_the_studio("not-there", TheBound::Nobodys);
        let network = reception_paired_with_the_studio();
        let corridor = Corridor {
            network: &network,
            looking: &crate::testing::NobodyIsNearby,
            naming: &NoNameYet,
        };
        let mut record = Record::default();

        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(ASKED),
                turning,
                &mut questions,
                Some(&corridor),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );
            assert_eq!(
                said.refusal().unwrap().text(),
                saying(strings, THE_CHOSEN_MACHINE_DID_NOT_ANSWER)
            );
        });
        assert_eq!(record.len(), 0);
    }
}
