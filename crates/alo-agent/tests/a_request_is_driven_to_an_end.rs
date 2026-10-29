//! **The order of a turn**, against a door that answers from a list.
//!
//! What this proves is the loop: that a read's answer reaches the model on the
//! next round, that a proposal ends the turn, that a refused read does not, and
//! that a turn getting nowhere stops. What it cannot prove is anything about a
//! model — the door answers from a script, so every "model" answer here is one
//! this file wrote.
//!
//! **That distinction is why it is said at the top.** Green here means the
//! machinery is in the right order. It does not mean a model can finish a job,
//! which is unmeasured: every grade in `data/catalogue.toml` was earned at one
//! step, and `gpt-4o-mini` scoring ten of ten on 2026-10-13 was ten of those.
//!
//! The verbs and the change that waits are the real ones —
//! `alo_files::file_verbs` and a `Proposal` through `Approvals` — because a
//! hand-made registry or a hand-made `Standing` would let this pass against
//! shapes the daemon never sends.

// Counted rather than copied: thirteen `unwrap`, one `expect` and five
// `panic!`. An unfulfilled expectation is an error in this workspace, so a list
// with something unused in it fails as surely as one with something missing.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_agent::{HowFar, NotAnswered, TheDoor, WhatHappened, driving, driving_after};
use alo_capability::{Approvals, Given, Grant, Grantee, Grants, Proposal, Reach, Verbs};
use alo_files::{OnThisMachine, Resolving as _};
use alo_instructing::{Instructions, WhatItMayReach};
use alo_protocol::{Done, FromAnAgent, Standing, ToAnAgent, Wording};
use alo_strings::Strings;

/// A door that answers from a list and remembers every request it was given.
struct ADoorThatAnswers {
    /// What it will answer, oldest first.
    answers: Vec<Result<ToAnAgent, NotAnswered>>,
    /// Every request it was given, oldest first.
    asked: Vec<FromAnAgent>,
}

impl ADoorThatAnswers {
    fn with(answers: Vec<Result<ToAnAgent, NotAnswered>>) -> Self {
        Self {
            answers,
            asked: Vec::new(),
        }
    }

    /// The question text of the nth `Ask` it was given.
    fn the_question(&self, nth: usize) -> &str {
        let mut seen = 0;
        for request in &self.asked {
            if let FromAnAgent::Ask { question, .. } = request {
                if seen == nth {
                    return question;
                }
                seen += 1;
            }
        }
        panic!("it was not asked {} questions", nth + 1)
    }
}

impl TheDoor for ADoorThatAnswers {
    fn asking(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered> {
        self.asked.push(request.clone());
        if self.answers.is_empty() {
            return Err(NotAnswered::NotReached {
                why: "the list of answers ran out".to_owned(),
            });
        }
        self.answers.remove(0)
    }
}

/// The verbs a real machine has.
fn the_verbs() -> Verbs {
    alo_files::file_verbs().unwrap()
}

/// Every word a machine answering these messages has loaded.
fn everything_this_machine_says() -> Strings {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_protocol::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

fn the_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A folder of this test's own, resolved — a grant is over a real place.
fn a_folder_of_our_own() -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let at = std::env::temp_dir().join(format!(
        "alo-agent-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(&at).unwrap();
    OnThisMachine.real(&at).unwrap().into_path_buf()
}

/// A change really waiting, built the way the daemon builds one.
///
/// Held in the caller's frame because `Standing::of` borrows the `Waiting` out
/// of the `Approvals`, and both outlive the answer this door scripts.
fn a_change_really_waiting() -> (Approvals, Strings) {
    let folder = a_folder_of_our_own();
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March").unwrap();
    let mut grants = Grants::default();
    grants.grant(Grant::checked("@files", Reach::Folder(folder), the_moment(), hour()).unwrap());
    let call = the_verbs()
        .call(
            "rename_file",
            &[
                ("file", Given::text(invoice.display().to_string())),
                ("name", Given::text("march-final.pdf")),
            ],
        )
        .unwrap();
    let proposal = Proposal::checked(
        &call,
        &Grantee::named("@files"),
        &grants,
        the_moment(),
        hour(),
    )
    .unwrap();
    let mut approvals = Approvals::default();
    approvals.propose(proposal);
    (approvals, everything_this_machine_says())
}

/// The change out of those approvals, as it crosses to an agent.
fn standing_in(approvals: &Approvals, strings: &Strings) -> Standing {
    let waiting = approvals
        .waiting_at(the_moment())
        .next()
        .expect("the change this fixture proposed");
    Standing::of(waiting, strings, the_moment())
}

/// A model's answer, as the machine hands one back.
fn the_model_said(text: &str) -> Result<ToAnAgent, NotAnswered> {
    Ok(ToAnAgent::Answered {
        text: text.to_owned(),
        came_from: a_real_refusal(),
        model: "a-model".to_owned(),
    })
}

/// A refusal the capability model really made, as one crosses.
///
/// Through `Authorised::read` with no grants rather than a hand-made `Wording`:
/// the sentence is then the one `alo-capability` wrote and a person would read,
/// so what this test scripts is a shape the daemon sends.
fn a_real_refusal() -> Wording {
    let strings = everything_this_machine_says();
    let call = the_verbs()
        .call(
            "list_folder",
            &[("folder", Given::text("/home/anna/Secrets"))],
        )
        .unwrap();
    let refused = alo_capability::Authorised::read(
        &call,
        &Grantee::named("@files"),
        &Grants::default(),
        the_moment(),
    )
    .expect_err("nothing was granted, so a read of it is refused");
    Wording::of(&refused.said(&strings))
}

/// What a listing answers with.
fn it_listed(text: &str) -> ToAnAgent {
    ToAnAgent::did(Done::of_a_file_verb(&alo_files::Answer::Read(
        text.to_owned(),
    )))
}

/// The line a model writes to ask for a read.
fn a_read_of(folder: &str) -> String {
    format!(
        r#"{{"format":1,"asks":{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{folder}"}}]}}}}}}"#
    )
}

/// The line a model writes to propose a change.
fn a_proposed_rename() -> String {
    r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/a.pdf"},{"named":"name","is":"b.pdf"}]}}}"#.to_owned()
}

/// **A read's answer reaches the model on the next round.**
///
/// The assertion that matters most in this file: without it the loop would ask
/// the same question twice and no model could ever take a second step.
#[test]
fn what_a_read_found_is_shown_back_on_the_next_round() {
    let verbs = the_verbs();
    let listed = it_listed("march.pdf\napril.pdf");
    let answer_line = listed.written().unwrap();
    let mut door = ADoorThatAnswers::with(vec![
        the_model_said(&a_read_of("/home/anna/Invoices")),
        Ok(listed),
        the_model_said("they are march.pdf and april.pdf"),
    ]);

    let happened = driving(
        "tidy my invoices",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    );
    assert_eq!(happened.after().len(), 1, "the read it did was dropped");

    let second = door.the_question(1);
    assert!(
        second.contains(&answer_line),
        "the model was not shown what its read found"
    );
    assert!(
        second.contains("You have already asked for this"),
        "the second question is not a continued turn"
    );
    assert!(
        !door.the_question(0).contains("You have already asked"),
        "the first question claims a turn had already begun"
    );
}

/// A proposal the machine accepts is the end: a change waits for the person,
/// with the number they answer by and the sentence they read.
#[test]
fn an_accepted_proposal_waits_for_the_person() {
    let verbs = the_verbs();
    let (approvals, strings) = a_change_really_waiting();
    let standing = standing_in(&approvals, &strings);
    let expected = standing.sentence().text().to_owned();
    let number = standing.number();

    let mut door = ADoorThatAnswers::with(vec![
        the_model_said(&a_proposed_rename()),
        Ok(ToAnAgent::Proposed(standing)),
    ]);
    let happened = driving(
        "rename it",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    );
    assert!(happened.is_waiting_for_the_person(), "{happened:?}");
    match happened {
        WhatHappened::AChangeIsWaiting {
            number: answered_by,
            sentence,
            after,
            ..
        } => {
            assert_eq!(answered_by, number);
            assert_eq!(sentence, expected);
            assert!(after.is_empty(), "it read nothing and recorded a read");
        }
        other => panic!("{other:?}"),
    }
}

/// **A model that answers in words has finished a request that needed no
/// change.** *Where is that file?* ends here, with nothing to approve.
#[test]
fn words_that_are_not_a_request_are_the_answer() {
    let verbs = the_verbs();
    let mut door = ADoorThatAnswers::with(vec![the_model_said("It is in /home/anna/Invoices.")]);
    match driving(
        "where is it?",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    ) {
        WhatHappened::ItWasAnsweredInWords { said, .. } => {
            assert_eq!(said, "It is in /home/anna/Invoices.");
        }
        other => panic!("{other:?}"),
    }
}

/// **An answer in the held shape ends the turn with its words** (ADR 0133).
///
/// A reply held to the envelope cannot be a line that is not JSON, so before
/// this door existed *list my documents folder* read the folder and then had no
/// way to say what it found. This is the second round of exactly that: a read,
/// what it found, and the answer — and the answer is never sent to the machine,
/// because it is said to the person.
#[test]
fn an_answer_after_a_read_ends_the_turn_with_its_words() {
    let verbs = the_verbs();
    let mut door = ADoorThatAnswers::with(vec![
        the_model_said(&a_read_of("/home/anna/Documents")),
        Ok(it_listed("march.pdf\nnotes.txt")),
        the_model_said(
            r#"{"format":1,"asks":{"answer":{"text":"Your Documents folder has march.pdf and notes.txt."}}}"#,
        ),
    ]);
    match driving(
        "what is in my documents folder",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    ) {
        WhatHappened::ItWasAnsweredInWords { said, after } => {
            assert_eq!(said, "Your Documents folder has march.pdf and notes.txt.");
            assert_eq!(after.len(), 1, "the read it did was dropped");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        door.asked.len(),
        3,
        "the answer was sent to the machine as though it were a request: {:?}",
        door.asked
    );
}

/// **An answer is the first thing it may say, too**: a turn shows the answer's
/// shape from the first round (ADR 0133, the owner's choice).
#[test]
fn an_answer_on_the_first_round_is_the_answer() {
    let verbs = the_verbs();
    let mut door = ADoorThatAnswers::with(vec![the_model_said(
        r#"{"format":1,"asks":{"answer":{"text":"I can only work with the folders you gave me."}}}"#,
    )]);
    match driving(
        "what is the weather",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    ) {
        WhatHappened::ItWasAnsweredInWords { said, after } => {
            assert_eq!(said, "I can only work with the folders you gave me.");
            assert!(after.is_empty());
        }
        other => panic!("{other:?}"),
    }
}

/// **A refused read is a step, not an end**: the model is told so and may ask
/// for something else, which is the behaviour `driving`'s header argues for and
/// a test is the only way to hold.
#[test]
fn a_refused_read_is_told_back_and_the_turn_goes_on() {
    let verbs = the_verbs();
    let (approvals, strings) = a_change_really_waiting();
    let refusal = ToAnAgent::Refused(a_real_refusal());
    let refusal_line = refusal.written().unwrap();
    let mut door = ADoorThatAnswers::with(vec![
        the_model_said(&a_read_of("/home/anna/Secrets")),
        Ok(refusal),
        the_model_said(&a_proposed_rename()),
        Ok(ToAnAgent::Proposed(standing_in(&approvals, &strings))),
    ]);

    let happened = driving(
        "tidy",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    );
    assert!(
        happened.is_waiting_for_the_person(),
        "a refused read ended the turn: {happened:?}"
    );
    assert_eq!(happened.after().len(), 1, "the refused read was not kept");
    assert!(
        door.the_question(1).contains(&refusal_line),
        "the model was not told its read was refused"
    );
}

/// **A model that only ever reads stops when the rounds run out**, and what it
/// did on the way is still there.
#[test]
fn a_turn_that_gets_nowhere_stops_and_keeps_what_it_did() {
    let verbs = the_verbs();
    let mut answers = Vec::new();
    for _ in 0..3 {
        answers.push(the_model_said(&a_read_of("/home/anna/Invoices")));
        answers.push(Ok(it_listed("nothing new")));
    }
    let mut door = ADoorThatAnswers::with(answers);
    match driving(
        "go round for ever",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::of(3),
        Instructions::SHOWN_TO_A_TURN,
    ) {
        WhatHappened::ItRanOutOfRounds { after } => assert_eq!(after.len(), 3),
        other => panic!("{other:?}"),
    }
}

/// A door that cannot be reached ends the turn in English, with how far it got.
#[test]
fn a_door_that_cannot_be_reached_says_so_in_english() {
    let verbs = the_verbs();
    let mut door = ADoorThatAnswers::with(vec![Err(NotAnswered::NotReached {
        why: "no such socket".to_owned(),
    })]);
    match driving(
        "anything",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    ) {
        WhatHappened::TheMachineCouldNotBeAsked { why, after } => {
            assert!(why.contains("no such socket"), "{why}");
            assert!(after.is_empty());
        }
        other => panic!("{other:?}"),
    }
}

/// **Nothing this loop sends is an approval.** It cannot be — `TheDoor` is
/// typed to the agent's half of the protocol — and this walks every request it
/// made to say so, because *it does not compile* is the guarantee and *nobody
/// checked* is how a guarantee stops being one when somebody widens a trait.
#[test]
fn every_request_it_makes_is_one_an_agent_may_make() {
    let verbs = the_verbs();
    let (approvals, strings) = a_change_really_waiting();
    let mut door = ADoorThatAnswers::with(vec![
        the_model_said(&a_read_of("/home/anna/Invoices")),
        Ok(it_listed("a.pdf")),
        the_model_said(&a_proposed_rename()),
        Ok(ToAnAgent::Proposed(standing_in(&approvals, &strings))),
    ]);
    driving(
        "tidy",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    );

    assert!(!door.asked.is_empty(), "it asked for nothing at all");
    for request in &door.asked {
        // Total on purpose: a fourth variant on `FromAnAgent` fails to compile
        // here, which is where a widening of the agent's half gets noticed.
        match request {
            FromAnAgent::Read { .. } | FromAnAgent::Propose { .. } | FromAnAgent::Ask { .. } => {}
        }
    }
}

/// **A turn that got nowhere can say what it tried.**
///
/// This is the state a booted machine was in on 2026-10-24: a real question
/// reached a real provider, eight rounds ran, and the only thing the machine
/// said was *it got nowhere in 8 step(s)*. The reason each round failed was
/// built, pushed onto the steps, and then reduced to a count — so there was
/// nothing to act on and no way to tell a provider answering badly from one
/// not answering at all.
///
/// The shape here is the one the machine really hit: the model replies with
/// something that **looks like a request and is not one**, which
/// `crate::driving` records as *not a request* and goes round again.
#[test]
fn a_turn_that_got_nowhere_says_why_each_round_failed() {
    let verbs = the_verbs();
    let answers = (0..3)
        .map(|_| the_model_said("{\"this\": \"is json and is not a request\"}"))
        .collect();
    let mut door = ADoorThatAnswers::with(answers);
    let happened = driving(
        "what is the capital of uganda",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::of(3),
        Instructions::SHOWN_TO_A_TURN,
    );

    let WhatHappened::ItRanOutOfRounds { after } = happened else {
        panic!("a model that never answers should run out of rounds: {happened:?}");
    };
    assert_eq!(after.len(), 3, "the rounds were not all kept");

    for (which, step) in after.iter().enumerate() {
        let said = step.what_came_back();
        assert!(
            said.contains("not a request"),
            "step {} says nothing a person could act on: {said:?}",
            which + 1
        );
    }
}

/// **And what it says is the machine's side, never the model's.**
///
/// The control. `AStep` holds both lines and only one of them is safe to put
/// in a service log: the person's words and whatever a verb read for them are
/// on the other side. A diagnosis that leaked the question would be a worse
/// bargain than no diagnosis.
#[test]
fn what_a_step_says_does_not_carry_the_question() {
    let verbs = the_verbs();
    let answers = (0..2)
        .map(|_| the_model_said("{\"not\": \"a request\"}"))
        .collect();
    let mut door = ADoorThatAnswers::with(answers);
    let happened = driving(
        "a question nobody else should read",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::of(2),
        Instructions::SHOWN_TO_A_TURN,
    );
    let WhatHappened::ItRanOutOfRounds { after } = happened else {
        panic!("{happened:?}");
    };
    for step in &after {
        assert!(
            !step.what_came_back().contains("nobody else should read"),
            "the diagnosis carried the person's own question"
        );
    }
}

/// **A follow-up shows the model the conversation so far on every round**, in
/// the question the turn asks the daemon and nowhere else — and a first request
/// shows none (ADR 0138).
#[test]
fn a_follow_up_carries_the_earlier_conversation_in_every_question_it_asks() {
    use alo_instructing::{AnEarlierTurn, TheConversationSoFar};

    let verbs = the_verbs();
    let earlier = TheConversationSoFar::of(vec![
        AnEarlierTurn::ThePersonSaid("list the files in /home/anna/Invoices".to_owned()),
        AnEarlierTurn::AloAnswered("march.pdf (12 KB) and april.pdf (40 KB)".to_owned()),
    ]);
    let mut door = ADoorThatAnswers::with(vec![
        the_model_said(&a_read_of("/home/anna/Invoices")),
        Ok(it_listed("march.pdf\napril.pdf")),
        the_model_said("april.pdf is the biggest, at 40 KB"),
    ]);
    let happened = driving_after(
        &earlier,
        "and how big is the biggest one?",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    );
    assert!(matches!(
        happened,
        WhatHappened::ItWasAnsweredInWords { .. }
    ));
    for nth in 0..2 {
        let question = door.the_question(nth);
        assert!(
            question.contains(&earlier.told()),
            "round {nth}: {question}"
        );
        assert!(question.starts_with(Instructions::SHOWN_TO_A_TURN.text()));
    }
    // Only questions: the conversation is never a request of its own.
    assert_eq!(door.asked.len(), 3, "{:?}", door.asked);

    let mut first = ADoorThatAnswers::with(vec![the_model_said("there are two")]);
    let _ = driving(
        "list the files in /home/anna/Invoices",
        &verbs,
        &WhatItMayReach::not_said(),
        &mut first,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    );
    assert!(
        !first.the_question(0).contains("The earlier conversation:"),
        "a first request was shown a conversation nobody had"
    );
}
