//! A question from a paired machine, at the door — and answered by this
//! machine's own model.
//!
//! `alo-asking`'s corridor puts a question to a paired machine at
//! [`alo_asking::THE_QUESTION_PATH`] on the port presence advertises, with a
//! proof in [`alo_asking::THE_PROOF_HEADER`]. This is the daemon's door for
//! it, and it judges a question in the order every door on this port judges
//! everything: **the proof first**, through the one `alo_corridor::Doorway`
//! the verb wire uses — so a proof spent on a question is refused as a verb
//! and the other way round — then **the pairing's own list**: a pairing that
//! does not permit asking this machine's models (`alo_nearby::MayAskIts::Models`)
//! permits nothing here, however good the proof. Only then is the body read
//! as anything, because the proof is over the bytes and a body altered in
//! transit fails at the proof, which is the true thing.
//!
//! # And then it is answered, by this machine's own model and nothing else
//!
//! What answers is what the **person here** chose for their own questions,
//! looked for at every question exactly as it is looked for at every local
//! turn ([`crate::questions`]) — so a model picked in Settings this morning
//! answers for the machine down the corridor this afternoon, and no default
//! decides it for them (ADR 0008). Three answers and one refusal follow from
//! that, and each is one word on the wire:
//!
//! - **Nothing chosen, nothing running, or a settings file that does not
//!   hold**: [`NOT_ANSWERED_HERE`], the word this door already had, because
//!   the asking machine's person is owed *that machine could not answer*
//!   and nothing about why a stranger's settings are as they are.
//! - **A provider chosen**: [`ANSWERS_ELSEWHERE`]. This machine's person
//!   sends their own questions off the building, and a question from a
//!   paired machine is **never** forwarded there — not to a provider, not to
//!   a third machine. ADR 0008 in the direction ADR 0003 makes sharpest: the
//!   question travels one corridor, and this machine answers for another
//!   only with what is on its own disk. Refused before anything is put
//!   anywhere, with nothing written.
//! - **A model on this machine**: put to it through
//!   [`alo_turn::Machine::answering_for`] — inside no turn of the asking
//!   machine's — recorded as `alo_record::Entry::answered_for` with the
//!   origin named, and the answer sent back in the OpenAI-compatible shape
//!   the corridor reads ([`alo_asking::an_answer_on_the_wire`]), leaving
//!   under a departure the indicator shows and the record keeps. Law 1 is
//!   not suspended for the length of a corridor.
//! - **A model that does not answer**: [`NOTHING_ANSWERED`], or
//!   [`NO_MODEL_HERE`] when what was to answer was not there — the two
//!   statuses the corridor already reads as *the machine down the corridor
//!   is having trouble* and *the model was not there*.
//!
//! **Nothing here is a fallback.** A question this machine cannot answer is
//! not put anywhere else, and a question from a machine that is not paired,
//! or whose pairing does not say *models*, is refused before anybody knows
//! what it asked.
//!
//! # The model named in the question is not the model that answers
//!
//! The corridor puts a question with a model name in it, because that is the
//! shape; the machine that answers puts it to the model **its** person chose,
//! and names that model in the answer. Which model runs on this machine is
//! this machine's person's setting, and the asking machine does not choose
//! what runs here any more than it chooses what its verbs may reach.

use std::io::Write as _;
use std::net::TcpStream;
use std::time::SystemTime;

use alo_answering::WentWrong;
use alo_asking::{THE_PROOF_HEADER, a_question_off_the_wire, an_answer_on_the_wire};
use alo_corridor::{AtTheDoor, Doorway, Naming, Replying};
use alo_egress::Departing;
use alo_nearby::http::{self, Message};
use alo_nearby::{MachineId, MayAskIts, NotNearby, NotProven, Origin, Pairings, Proof};
use alo_turn::{AnsweredFor, Arriving, Machine, NoAnswer};

use crate::questions::{Questions, WhatAnswers};
use crate::refusing::NotServed;

/// What the wire says for a question it proved and this machine has nothing
/// chosen to answer with.
pub const NOT_ANSWERED_HERE: &str = alo_asking::NOT_ANSWERED_HERE;

/// What the wire says for a question from a pairing that does not permit
/// asking this machine's models.
pub const NOT_PERMITTED: &str = alo_asking::NOT_PERMITTED;

/// What the wire says for a question this machine's person answers their own
/// questions elsewhere for: a provider is chosen here, and a question from
/// another machine is not sent there.
pub const ANSWERS_ELSEWHERE: &str = alo_asking::ANSWERS_ELSEWHERE;

/// What the wire says for a body that proved itself and was not a question.
pub const NOT_A_QUESTION: &str = "not-a-question";

/// What the wire says when the model here was asked and did not answer.
pub const NOTHING_ANSWERED: &str = "nothing-answered-here";

/// What the wire says when what was to answer was not there.
pub const NO_MODEL_HERE: &str = "no-model-here";

/// What became of a question at the door.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Questioned {
    /// It carried no proof, and was refused before anything was asked.
    NoProof,
    /// Its proof did not hold, and was refused before anything was asked.
    NotProven(NotProven),
    /// It proved where it came from, and that pairing does not permit asking
    /// this machine's models.
    NotPermitted(MachineId),
    /// It proved where it came from, the pairing permits it, and the body was
    /// not a question.
    NotAQuestion(MachineId),
    /// It proved where it came from, the pairing permits it, and nothing on
    /// this machine is chosen to answer it.
    NotAnsweredHere(MachineId),
    /// It proved where it came from, the pairing permits it, and this
    /// machine's person answers their own questions from a provider — which a
    /// question from another machine is never sent to.
    AnswersElsewhere(MachineId),
    /// The model here was asked and did not answer; nothing was written.
    NothingAnswered {
        /// The machine that asked.
        from: MachineId,
        /// What the model here said about it.
        why: WentWrong,
    },
    /// The model answered and the rule in force held the answer back, written
    /// down as such; the connection closed with nothing on it.
    HeldBack(MachineId),
    /// The model answered and the origin's name could not be put on the
    /// indicator, so nothing left and the connection closed with nothing on
    /// it.
    NotShown(MachineId),
    /// The model answered and the answer went back under a departure.
    Answered(MachineId),
}

/// What became of a question, once the reply was written: what was decided,
/// or that the reply could not be written and the other machine heard
/// nothing.
pub type Replied = Result<Questioned, NotNearby>;

/// Everything a question is judged against, at the moment.
pub struct Asking<'a> {
    /// This machine's pairings at the moment.
    pub pairings: &'a Pairings,
    /// What the person here chose to answer questions, looked for once per
    /// question.
    pub questions: &'a mut Questions,
    /// What the person here called the other machines; decides nothing.
    pub naming: &'a dyn Naming,
}

/// Judge one question, answer it on the wire, and spend the departure its
/// answer left under.
///
/// `doorway` holds the proofs seen and the machine, or the remote turn
/// holding it; the answer is put through whichever has it.
///
/// # Errors
///
/// [`NotServed::NothingIsWrittenDown`] when what happened could not be
/// written down — an answer that left with no record of it, or one refused
/// with no record — and [`NotServed::TheMachineWasLost`] when the doorway has
/// neither the machine nor a turn. A reply that could not be written is not
/// an error: it is [`Replied`]'s `Err`, and the service goes on.
pub fn answered(
    mut stream: TcpStream,
    message: &Message,
    doorway: &mut Doorway<'_, '_>,
    asking: Asking<'_>,
    now: SystemTime,
) -> Result<Replied, NotServed> {
    let Judgement {
        questioned,
        written,
        departed,
    } = judged(message, doorway, asking, now)?;
    let sent = match &written {
        Some(written) => stream
            .write_all(written.as_bytes())
            .map_err(|why| NotNearby::TheNetwork(why.to_string())),
        None => Ok(()),
    };
    // The departure is spent whatever the write said: it went, or enough of
    // it went that the indicator cannot say it did not.
    if let Some((origin, departing)) = departed {
        match through(doorway) {
            Some(through) => through.answer_returned(&origin, departing)?,
            None => return Err(NotServed::TheMachineWasLost),
        }
    }
    Ok(sent.map(|()| questioned))
}

/// What one question came to, before any byte went back.
struct Judgement {
    /// What was decided.
    questioned: Questioned,
    /// The bytes that go back, or nothing — the rule held the answer back,
    /// or the origin could not be shown, and the connection closes empty.
    written: Option<String>,
    /// The departure the answer leaves under, to be spent once the bytes
    /// have gone, and whom it leaves for.
    departed: Option<(Origin, Departing)>,
}

impl Judgement {
    /// A word said back, with nothing leaving under a departure.
    fn said(questioned: Questioned, status: u16, reason: &str, word: &str) -> Self {
        Self {
            questioned,
            written: Some(http::a_reply(status, reason, &format!("{word}\n"))),
            departed: None,
        }
    }

    /// Nothing goes back at all.
    fn nothing(questioned: Questioned) -> Self {
        Self {
            questioned,
            written: None,
            departed: None,
        }
    }
}

/// The judgement, with no socket in sight.
fn judged(
    message: &Message,
    doorway: &mut Doorway<'_, '_>,
    asking: Asking<'_>,
    now: SystemTime,
) -> Result<Judgement, NotServed> {
    // 1. The proof, before anything else.
    let proof = match message.header(THE_PROOF_HEADER).map(Proof::read) {
        None | Some(Err(_)) => {
            return Ok(Judgement {
                questioned: Questioned::NoProof,
                written: Some(Replying::before_the_door(AtTheDoor::NoProof).written()),
                departed: None,
            });
        }
        Some(Ok(proof)) => proof,
    };
    let origin = match doorway.proven(&proof, &message.body, asking.pairings, asking.naming, now) {
        Ok(origin) => origin,
        Err(why) => {
            return Ok(Judgement {
                questioned: Questioned::NotProven(why),
                written: Some(Replying::before_the_door(AtTheDoor::NotProven(why)).written()),
                departed: None,
            });
        }
    };
    let from = origin.machine().clone();

    // 2. The pairing's own list.
    if !asking.pairings.permits(&from, MayAskIts::Models, now) {
        return Ok(Judgement::said(
            Questioned::NotPermitted(from),
            403,
            "Forbidden",
            NOT_PERMITTED,
        ));
    }

    // 3. What the person here chose, looked for now: a question from another
    // machine is no turn of theirs, so what was found for the last one is
    // not held for this one.
    asking.questions.a_new_turn();
    let (chosen, runtime, places) = match asking.questions.what_answers() {
        WhatAnswers::Nothing | WhatAnswers::NotRunning | WhatAnswers::NotSet(_) => {
            return Ok(not_answered_here(from));
        }
        // A provider, or another paired machine: a question from a machine down
        // the corridor is never passed on to either (ADR 0003, ADR 0008).
        WhatAnswers::FromAProvider { .. } | WhatAnswers::FromAPairedMachine { .. } => {
            return Ok(Judgement::said(
                Questioned::AnswersElsewhere(from),
                503,
                "Service Unavailable",
                ANSWERS_ELSEWHERE,
            ));
        }
        WhatAnswers::OnThisMachine {
            chosen,
            runtime,
            places,
        } => (chosen, runtime, places),
    };
    // Every rule permits a machine answering on itself, and it is answered
    // rather than assumed away.
    let Ok(permission) = chosen.asking(Some(places.policy())) else {
        return Ok(not_answered_here(from));
    };

    // 4. The body, now that the proof over it held — read as a question for
    // the model the person here chose, whatever model the body named.
    let Ok(question) = a_question_off_the_wire(&message.body, chosen.model()) else {
        return Ok(Judgement::said(
            Questioned::NotAQuestion(from),
            400,
            "Bad Request",
            NOT_A_QUESTION,
        ));
    };

    // 5. This machine's own model, through whatever holds the machine.
    let Some(through) = through(doorway) else {
        return Err(NotServed::TheMachineWasLost);
    };
    let answered = through.answering_for(
        &origin,
        &question,
        permission,
        runtime.in_words(),
        places.policy(),
        now,
    );
    Ok(match answered {
        Ok(answered) => {
            let (answer, departing) = answered.into_parts();
            Judgement {
                questioned: Questioned::Answered(from),
                written: Some(http::a_json_reply(
                    200,
                    "OK",
                    &an_answer_on_the_wire(&answer),
                )),
                departed: Some((origin, departing)),
            }
        }
        Err(NoAnswer::DidNotAnswer(failed)) => {
            let why = failed.why();
            let (status, reason, word) = match why {
                WentWrong::NoModelThere => (404, "Not Found", NO_MODEL_HERE),
                WentWrong::NothingAnswered
                | WentWrong::TookTooLong
                | WentWrong::NothingUsable
                | WentWrong::KeyNotAccepted
                | WentWrong::RanOut
                | WentWrong::SentSomewhereElse
                | WentWrong::NoWayThere
                // Not reachable from this machine's own model, which is the only
                // thing a question from another machine is put to; answered as
                // what it would mean rather than assumed away.
                | WentWrong::RefusedThere(_)
                | WentWrong::HavingTrouble(_) => (503, "Service Unavailable", NOTHING_ANSWERED),
            };
            Judgement::said(
                Questioned::NothingAnswered { from, why },
                status,
                reason,
                word,
            )
        }
        Err(NoAnswer::HeldBack(_)) => Judgement::nothing(Questioned::HeldBack(from)),
        Err(NoAnswer::CannotBeShown(_)) => Judgement::nothing(Questioned::NotShown(from)),
        Err(NoAnswer::NotRecorded { .. }) => return Err(NotServed::NothingIsWrittenDown),
        // A body that read as a question is a question; a permission for
        // this machine is for this machine; a machine outside any turn has
        // no closed turn and no boundary to enter. None of these can be
        // reached, and each is answered rather than assumed away.
        Err(
            NoAnswer::NotAQuestion(_)
            | NoAnswer::Miswired(_)
            | NoAnswer::TurnClosed
            | NoAnswer::NotBounded(_),
        ) => not_answered_here(from),
    })
}

/// The word for a machine with nothing chosen to answer with.
fn not_answered_here(from: MachineId) -> Judgement {
    Judgement::said(
        Questioned::NotAnsweredHere(from),
        503,
        "Service Unavailable",
        NOT_ANSWERED_HERE,
    )
}

/// Whatever holds the machine: the remote turn under way, or the machine
/// itself.
enum Through<'a, 'm, 't> {
    /// A remote turn holds it.
    ATurn(&'a mut Arriving<'m, 't>),
    /// Nobody does.
    TheMachine(&'a mut Machine<'t>),
}

/// What holds the doorway's machine, or `None` when it was lost.
fn through<'a, 'm, 't>(doorway: &'a mut Doorway<'m, 't>) -> Option<Through<'a, 'm, 't>> {
    if doorway.turn().is_some() {
        doorway.turn().map(Through::ATurn)
    } else {
        doorway.machine().map(Through::TheMachine)
    }
}

impl Through<'_, '_, '_> {
    /// [`alo_turn::Machine::answering_for`], through whichever has the
    /// machine.
    fn answering_for(
        self,
        origin: &Origin,
        question: &alo_asking::Question,
        answering: alo_answering::Answering,
        runtime: &dyn alo_models::ModelRuntime,
        policy: &alo_models::SourcePolicy,
        now: SystemTime,
    ) -> Result<AnsweredFor, NoAnswer> {
        match self {
            Self::ATurn(arriving) => {
                arriving.answering_for(origin, question, answering, runtime, policy, now)
            }
            Self::TheMachine(machine) => {
                machine.answering_for(origin, question, answering, runtime, policy, now)
            }
        }
    }

    /// [`alo_turn::Machine::answer_returned`], through whichever has the
    /// machine.
    ///
    /// # Errors
    ///
    /// [`NotServed::NothingIsWrittenDown`]: the answer left and there is no
    /// evidence of it, and the service stops for it.
    fn answer_returned(self, origin: &Origin, departing: Departing) -> Result<(), NotServed> {
        let returned = match self {
            Self::ATurn(arriving) => arriving.answer_returned(origin, departing),
            Self::TheMachine(machine) => machine.answer_returned(origin, departing),
        };
        returned.map_err(|_| NotServed::NothingIsWrittenDown)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use alo_answering::WentWrong;
    use alo_asking::{THE_PROOF_HEADER, THE_QUESTION_PATH};
    use alo_choosing::{Chosen, Which};
    use alo_corridor::Doorway;
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_models::{Catalogue, RuntimeError, SourcePolicy};
    use alo_nearby::http::Message;
    use alo_nearby::{MayAskIts, NotProven, Pairing, Pairings, Proof};
    use alo_record::{Happened, Only, Record};
    use alo_turn::Machine;

    use super::{
        ANSWERS_ELSEWHERE, Asking, Judgement, NO_MODEL_HERE, NOT_A_QUESTION, NOT_ANSWERED_HERE,
        NOTHING_ANSWERED, Questioned, judged,
    };
    use crate::questions::{Questions, TheBound, WhoseKeyring};
    use crate::terms::NoNameYet;
    use crate::testing::{
        NothingIsBounded, a_directory_of_our_own, a_runtime_saying, in_english, noon,
        nothing_has_been_chosen, paired_between, reception, the_studio,
    };

    /// The body the corridor puts, word for word.
    const A_QUESTION: &str =
        r#"{"model":"m","messages":[{"role":"user","content":"hello"}],"stream":false}"#;

    /// A question as the corridor puts one, with these headers and this body.
    fn a_question(headers: Vec<(String, String)>, body: &str) -> Message {
        Message {
            first: format!("POST {THE_QUESTION_PATH} HTTP/1.1"),
            headers,
            body: body.to_owned(),
        }
    }

    /// This body, proven by reception at this moment.
    fn proven_by(on_reception: &Pairing, body: &str, at: std::time::SystemTime) -> Message {
        let proof = Proof::made(on_reception, &reception(), body.as_bytes(), at);
        a_question(vec![(THE_PROOF_HEADER.to_owned(), proof.said())], body)
    }

    /// The studio's pairings, holding its row about reception permitting
    /// `may`, and reception's own row to make proofs with.
    fn paired_for(may: &[MayAskIts]) -> (Pairings, Pairing) {
        let (on_reception, on_studio) = paired_between(reception(), the_studio(), may, noon());
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        (pairings, on_reception)
    }

    /// A machine whose person chose a model here, answered by this runtime.
    fn a_person_who_chose_a_model_here(said: Result<String, RuntimeError>) -> Questions {
        Questions::already_found(
            Chosen::of(Which::Catalogue, "the-model-chosen-here").unwrap(),
            a_runtime_saying(said),
            TheBound::Nobodys,
        )
    }

    /// A machine whose person chose a provider for their own questions.
    fn a_person_who_chose_a_provider(what: &str) -> Questions {
        let config = a_directory_of_our_own(what);
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            "format = 2

[answers]
provider = { name = \"Mistral\", model = \"mistral-small-latest\" }

[[provider]]
name = \"Mistral\"
endpoint = \"https://api.mistral.ai\"
region = \"the EU\"
",
        )
        .unwrap();
        Questions::of_a_session(
            Some(config.into_os_string()),
            None,
            Catalogue::built_in().unwrap(),
            TheBound::Nobodys,
            WhoseKeyring::Nobodys,
        )
    }

    /// What the studio decided about each message `doing` puts through, with
    /// its record and whether its indicator was quiet afterwards.
    fn on_the_studio(
        doing: impl FnOnce(&mut Doorway<'_, '_>) -> Vec<Judgement>,
    ) -> (Vec<Judgement>, Record, bool) {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut record = Record::default();
        let judged = {
            let mut machine = Machine::carrying_out_file_verbs(
                &strings,
                &OnThisMachine,
                &mut bounding,
                &mut indicator,
                &mut record,
            )
            .unwrap();
            let mut doorway = Doorway::at(
                the_studio(),
                &mut machine,
                Duration::from_secs(3_600),
                Duration::from_secs(3_600),
            )
            .unwrap();
            doing(&mut doorway)
        };
        (judged, record, indicator.is_quiet())
    }

    /// One question judged at the studio against these pairings and this
    /// choice, at this moment.
    fn one(
        doorway: &mut Doorway<'_, '_>,
        message: &Message,
        pairings: &Pairings,
        questions: &mut Questions,
        at: std::time::SystemTime,
    ) -> Judgement {
        judged(
            message,
            doorway,
            Asking {
                pairings,
                questions,
                naming: &NoNameYet,
            },
            at,
        )
        .unwrap()
    }

    /// The status line's number.
    fn status(judgement: &Judgement) -> u16 {
        alo_nearby::http::status_of(
            judgement
                .written
                .as_deref()
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap()
    }

    /// **The proof is judged first and a question without one, or with one
    /// that does not hold, is refused before anything is asked** — and a
    /// proof that held once is a replay the second time, through the same
    /// memory the verb wire refuses replays with. Nothing is written, and
    /// the machine with nothing chosen says so in its one word.
    #[test]
    fn a_question_is_proven_before_anything_else_and_a_replay_is_refused() {
        let (pairings, on_reception) = paired_for(&[MayAskIts::Models]);
        let mut questions = nothing_has_been_chosen();
        let (judged, record, _) = on_the_studio(|doorway| {
            let first = one(
                doorway,
                &a_question(Vec::new(), A_QUESTION),
                &pairings,
                &mut questions,
                noon(),
            );
            let proven = proven_by(&on_reception, A_QUESTION, noon());
            let second = one(doorway, &proven, &pairings, &mut questions, noon());
            let replayed = one(
                doorway,
                &proven,
                &pairings,
                &mut questions,
                noon() + Duration::from_secs(1),
            );
            vec![first, second, replayed]
        });
        assert_eq!(judged.first().unwrap().questioned, Questioned::NoProof);
        assert!(status(judged.first().unwrap()) >= 400);
        assert_eq!(
            judged.get(1).unwrap().questioned,
            Questioned::NotAnsweredHere(reception())
        );
        assert_eq!(status(judged.get(1).unwrap()), 503);
        assert!(
            judged
                .get(1)
                .unwrap()
                .written
                .as_deref()
                .unwrap()
                .contains(NOT_ANSWERED_HERE)
        );
        assert_eq!(
            judged.get(2).unwrap().questioned,
            Questioned::NotProven(NotProven::AlreadySeen)
        );
        assert!(record.is_empty(), "a question wrote something down");
    }

    /// **A pairing that does not say models permits no question**, however
    /// good the proof — the list is what it says — and the body is not read.
    #[test]
    fn a_pairing_for_something_else_does_not_open_this_door() {
        let (pairings, on_reception) = paired_for(&[MayAskIts::Workspace]);
        let mut questions = a_person_who_chose_a_model_here(Ok("an answer".to_owned()));
        let (judged, record, _) = on_the_studio(|doorway| {
            vec![one(
                doorway,
                &proven_by(&on_reception, "not even a question", noon()),
                &pairings,
                &mut questions,
                noon(),
            )]
        });
        assert_eq!(
            judged.first().unwrap().questioned,
            Questioned::NotPermitted(reception())
        );
        assert_eq!(status(judged.first().unwrap()), 403);
        assert!(record.is_empty());
    }

    /// **An unpaired machine's question is refused before anybody knows what
    /// it asked**: the proof names a pairing this machine does not hold.
    #[test]
    fn an_unpaired_machines_question_is_refused_at_the_proof() {
        let (_, on_reception) = paired_for(&[MayAskIts::Models]);
        let mut questions = a_person_who_chose_a_model_here(Ok("an answer".to_owned()));
        let (judged, record, _) = on_the_studio(|doorway| {
            vec![one(
                doorway,
                &proven_by(&on_reception, A_QUESTION, noon()),
                &Pairings::none(),
                &mut questions,
                noon(),
            )]
        });
        assert_eq!(
            judged.first().unwrap().questioned,
            Questioned::NotProven(NotProven::NotWithThatMachine)
        );
        assert!(record.is_empty());
    }

    /// **A proven, permitted question is put to this machine's own model**
    /// — the one the person here chose, not the one the question named —
    /// recorded as answered for reception with the origin named, and the
    /// answer goes back in the shape the corridor reads, under a departure
    /// that is written down as left once the bytes have gone.
    #[test]
    fn a_proven_question_is_answered_by_this_machines_own_model_and_leaves_under_a_departure() {
        let (pairings, on_reception) = paired_for(&[MayAskIts::Models]);
        let mut questions =
            a_person_who_chose_a_model_here(Ok("No, not without written consent.".to_owned()));
        let (judged, record, quiet) = on_the_studio(|doorway| {
            let mut judgement = one(
                doorway,
                &proven_by(&on_reception, A_QUESTION, noon()),
                &pairings,
                &mut questions,
                noon(),
            );
            // The answer is on the indicator until the bytes have gone.
            assert!(!doorway.machine().unwrap().showing().is_quiet());
            let (origin, departing) = judgement.departed.take().unwrap();
            doorway
                .machine()
                .unwrap()
                .answer_returned(&origin, departing)
                .unwrap();
            vec![judgement]
        });
        let judgement = judged.first().unwrap();
        assert_eq!(judgement.questioned, Questioned::Answered(reception()));
        assert_eq!(status(judgement), 200);
        let written = judgement.written.as_deref().unwrap();
        assert!(
            written.contains("content-type: application/json"),
            "{written}"
        );
        let body = written.split("\r\n\r\n").nth(1).unwrap();
        assert!(
            body.contains(r#""content":"No, not without written consent.""#),
            "{body}"
        );
        assert!(
            body.contains(r#""model":"the-model-chosen-here""#),
            "the model the question named answered, rather than the one the person here chose: {body}"
        );
        assert!(!body.contains(r#""model":"m""#), "{body}");
        assert!(
            quiet,
            "the line stayed on the indicator after the answer had gone"
        );

        assert_eq!(record.len(), 2, "{record:?}");
        assert!(
            record.everything().any(|entry| matches!(
                entry.happened(),
                Happened::AnsweredForAnotherMachine { .. }
            ))
        );
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Egress))
                .count(),
            1
        );
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is(reception().as_str()))
        }));
    }

    /// **A question answered for a paired machine names it by the name its
    /// person here gave it on the person's door** — on the indicator while the
    /// answer leaves, on the departure, and on both record entries — while the
    /// machine it is answered for is still reception by identity.
    #[test]
    fn a_question_answered_for_a_paired_machine_names_it_by_its_given_name() {
        use crate::answering::what_a_person_said;
        use crate::holding::Holding;
        use crate::network::TheNetwork;
        use crate::pairing::Nearby;
        use crate::rereading::WhatIsGranted;
        use crate::testing::{NobodyIsNearby, NothingIsRemembered, a_message};

        const CALLED: &str = "the reception machine";
        let (pairings, on_reception) = paired_for(&[MayAskIts::Models]);
        let network = TheNetwork::on(the_studio());
        *network.locked().pairings_mut() = pairings;
        let mut questions = a_person_who_chose_a_model_here(Ok("Three are unpaid.".to_owned()));
        let strings = in_english();
        let (answered, record, quiet) = on_the_studio(|doorway| {
            let named = what_a_person_said(
                &a_message(&format!(
                    r#"{{"name-machine":{{"machine":"{}","called":"{CALLED}"}}}}"#,
                    reception().as_str()
                )),
                &mut Holding::TheNetwork {
                    doorway: &mut *doorway,
                    network: &network,
                    questions: &mut nothing_has_been_chosen(),
                },
                &mut WhatIsGranted::of(
                    &mut alo_capability::Grants::default(),
                    &NothingIsRemembered,
                ),
                &Nearby {
                    network: &network,
                    looking: &NobodyIsNearby,
                },
                &strings,
                noon(),
            )
            .unwrap();
            assert_eq!(named.called(), Some(CALLED), "{named:?}");

            let mut judgement = judged(
                &proven_by(&on_reception, A_QUESTION, noon()),
                doorway,
                Asking {
                    pairings: network.locked().pairings(),
                    questions: &mut questions,
                    naming: network.names(),
                },
                noon(),
            )
            .unwrap();
            let showing = doorway.machine().unwrap().showing().showing();
            assert_eq!(showing.len(), 1);
            assert_eq!(
                showing.first().unwrap().leaving().unwrap().destination(),
                &alo_egress::Destination::PairedMachine {
                    machine: CALLED.to_owned()
                },
                "the indicator did not name the machine by its name"
            );
            let (origin, departing) = judgement.departed.take().unwrap();
            assert_eq!(origin.called(), CALLED);
            assert_eq!(origin.machine(), &reception());
            assert_eq!(
                departing.destination(),
                &alo_egress::Destination::PairedMachine {
                    machine: CALLED.to_owned()
                }
            );
            doorway
                .machine()
                .unwrap()
                .answer_returned(&origin, departing)
                .unwrap();
            vec![judgement]
        });
        assert_eq!(
            answered.first().unwrap().questioned,
            Questioned::Answered(reception())
        );
        assert!(quiet);
        assert_eq!(record.len(), 2, "{record:?}");
        assert!(
            record
                .everything()
                .all(|entry| entry.origin().is_some_and(|from| from.is(CALLED))),
            "{record:?}"
        );
    }

    /// **A question from a paired machine reaches the pinned runtime here
    /// exactly as before — never held to the envelope** (ADR 0032, decision
    /// 4): a paired machine is a separate measurement, and what crosses the
    /// corridor is asked in words. Read off the runtime's own socket.
    #[test]
    fn a_question_from_a_paired_machine_is_never_held_to_the_envelope() {
        let (pairings, on_reception) = paired_for(&[MayAskIts::Models]);
        let (runtime, served) = crate::testing::a_runtime_served(
            r#"{"message":{"role":"assistant","content":"Three are unpaid."}}"#,
        );
        let mut questions = Questions::already_found_pinned(
            Chosen::of(Which::Catalogue, "the-model-chosen-here").unwrap(),
            runtime,
            TheBound::Nobodys,
        );
        let (judged, _, _) = on_the_studio(|doorway| {
            let mut judgement = one(
                doorway,
                &proven_by(&on_reception, A_QUESTION, noon()),
                &pairings,
                &mut questions,
                noon(),
            );
            let (origin, departing) = judgement.departed.take().unwrap();
            doorway
                .machine()
                .unwrap()
                .answer_returned(&origin, departing)
                .unwrap();
            vec![judgement]
        });
        assert_eq!(
            judged.first().unwrap().questioned,
            Questioned::Answered(reception())
        );
        let body = crate::testing::the_body_of(&served.join().unwrap());
        assert!(
            body.get("format").is_none(),
            "a paired machine's question was held to a shape: {body}"
        );
    }

    /// **A machine whose person chose a provider refuses the question in
    /// words**: nothing is put to the provider, nothing is put to a model
    /// here, and nothing is written. Never a fallback, in either direction.
    #[test]
    fn a_machine_whose_person_chose_a_provider_refuses_the_question_in_words() {
        let (pairings, on_reception) = paired_for(&[MayAskIts::Models]);
        let mut questions = a_person_who_chose_a_provider("answers-elsewhere");
        let (judged, record, quiet) = on_the_studio(|doorway| {
            vec![one(
                doorway,
                &proven_by(&on_reception, A_QUESTION, noon()),
                &pairings,
                &mut questions,
                noon(),
            )]
        });
        let judgement = judged.first().unwrap();
        assert_eq!(
            judgement.questioned,
            Questioned::AnswersElsewhere(reception())
        );
        assert_eq!(status(judgement), 503);
        assert!(
            judgement
                .written
                .as_deref()
                .unwrap()
                .contains(ANSWERS_ELSEWHERE)
        );
        assert!(judgement.departed.is_none());
        assert!(record.is_empty(), "{record:?}");
        assert!(quiet);
    }

    /// **A body that proved itself and is not a question is refused as
    /// such**, after the proof and the pairing and before anything is put
    /// anywhere.
    #[test]
    fn a_proven_body_that_is_not_a_question_is_refused_as_not_a_question() {
        let (pairings, on_reception) = paired_for(&[MayAskIts::Models]);
        let mut questions = a_person_who_chose_a_model_here(Ok("an answer".to_owned()));
        let (judged, record, _) = on_the_studio(|doorway| {
            vec![one(
                doorway,
                &proven_by(
                    &on_reception,
                    r#"{"verb":"list_folder","given":[]}"#,
                    noon(),
                ),
                &pairings,
                &mut questions,
                noon(),
            )]
        });
        let judgement = judged.first().unwrap();
        assert_eq!(judgement.questioned, Questioned::NotAQuestion(reception()));
        assert_eq!(status(judgement), 400);
        assert!(
            judgement
                .written
                .as_deref()
                .unwrap()
                .contains(NOT_A_QUESTION)
        );
        assert!(record.is_empty());
    }

    /// **A model here that does not answer says so, and writes nothing**: a
    /// runtime that was not reachable is *nothing answered*, and a model that
    /// was not there is *no model here*.
    #[test]
    fn a_model_here_that_does_not_answer_says_so_and_writes_nothing() {
        let (pairings, on_reception) = paired_for(&[MayAskIts::Models]);
        let mut unreachable = a_person_who_chose_a_model_here(Err(RuntimeError::Unreachable));
        let mut not_there = a_person_who_chose_a_model_here(Err(RuntimeError::NotInstalled(
            "the-model-chosen-here".to_owned(),
        )));
        let (judged, record, quiet) = on_the_studio(|doorway| {
            vec![
                one(
                    doorway,
                    &proven_by(&on_reception, A_QUESTION, noon()),
                    &pairings,
                    &mut unreachable,
                    noon(),
                ),
                one(
                    doorway,
                    &proven_by(&on_reception, A_QUESTION, noon() + Duration::from_secs(1)),
                    &pairings,
                    &mut not_there,
                    noon() + Duration::from_secs(1),
                ),
            ]
        });
        let first = judged.first().unwrap();
        assert_eq!(
            first.questioned,
            Questioned::NothingAnswered {
                from: reception(),
                why: WentWrong::NothingAnswered
            }
        );
        assert_eq!(status(first), 503);
        assert!(first.written.as_deref().unwrap().contains(NOTHING_ANSWERED));
        let second = judged.get(1).unwrap();
        assert_eq!(
            second.questioned,
            Questioned::NothingAnswered {
                from: reception(),
                why: WentWrong::NoModelThere
            }
        );
        assert_eq!(status(second), 404);
        assert!(second.written.as_deref().unwrap().contains(NO_MODEL_HERE));
        assert!(record.is_empty(), "{record:?}");
        assert!(quiet);
    }

    /// **A rule that says nothing leaves holds the answer back**: the model
    /// answered and that is written down, the held-back departure is written
    /// down with the origin named, nothing goes back on the wire, and the
    /// indicator is quiet.
    #[test]
    fn a_rule_that_says_nothing_leaves_holds_the_answer_back() {
        let (pairings, on_reception) = paired_for(&[MayAskIts::Models]);
        let mut questions = Questions::already_found(
            Chosen::of(Which::Catalogue, "the-model-chosen-here").unwrap(),
            a_runtime_saying(Ok("an answer".to_owned())),
            TheBound::AnOrganisations(SourcePolicy::ThisMachineOnly),
        );
        let (judged, record, quiet) = on_the_studio(|doorway| {
            vec![one(
                doorway,
                &proven_by(&on_reception, A_QUESTION, noon()),
                &pairings,
                &mut questions,
                noon(),
            )]
        });
        let judgement = judged.first().unwrap();
        assert_eq!(judgement.questioned, Questioned::HeldBack(reception()));
        assert!(judgement.written.is_none(), "{:?}", judgement.written);
        assert!(judgement.departed.is_none());
        assert!(quiet);
        assert_eq!(record.len(), 2, "{record:?}");
        assert!(
            record
                .everything()
                .any(|entry| matches!(entry.happened(), Happened::HeldBack { .. }))
        );
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(Only::Egress))
                .count(),
            0
        );
    }
}
