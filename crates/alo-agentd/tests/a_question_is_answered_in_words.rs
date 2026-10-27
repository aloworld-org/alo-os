//! **Is a question answered, or does it end in a change nobody asked for?**
//!
//! On booted machines on 2026-10-25 and again on 2026-10-27, a person granted
//! a folder and asked to have something *read* — *list my documents folder*,
//! *what does the record say was done to this file* — and got a proposal to
//! install Krita, GIMP or Inkscape. The person said no each time and nothing
//! was installed, but nobody got an answer.
//!
//! `crates/alo-driving/tests/does_a_model_look_before_it_proposes.rs` ruled out
//! one cause: a model shown today's text **does** look first. What it cannot
//! see is the step after looking, because it scores one round. On the machine
//! of 2026-10-27 the agent's own log says the turn for *list the files* took
//! **six** read steps and then ended in the install. So this follows a turn to
//! its end, and asks how it ended.
//!
//! # What stands in for what
//!
//! | | |
//! |---|---|
//! | the model | real, at a provider, with a key from a real keyring |
//! | the verbs the model is told about | `alo_turn::the_verbs_a_whole_machine_offers`, the agent's own list |
//! | the folders it is told it may reach | the granted one, as `alo-agent` is told (#958) |
//! | the instructions | `Instructions::SHOWN_TO_A_TURN` unless `ALO_INSTRUCTIONS` names another |
//! | the loop | `alo_agent::driving`, the agent's own |
//! | the verbs carried out | the six file verbs, against a real folder |
//! | **the boundary** | **none** — `NothingIsBounded`, below |
//!
//! **The boundary is absent on purpose**, and it is the one substitution: this
//! measures what a model asks for, which the kernel has no part in, and leaving
//! the kernel out is what lets it run on a developer's machine in minutes
//! rather than on a booted image in an hour. A proposal to install is refused
//! here (this machine carries out file verbs only) where a real machine would
//! put it to the person; the refusal is told back to the model, so a run may go
//! on past where a real turn would have stopped. That is why the verdict reads
//! the model's lines, and counts **any** proposal to a question as a failure.
//!
//! # It is ignored, and it leaves the machine
//!
//! It asks a real provider and costs money, so it is run by name:
//!
//! ```text
//! ALO_PROVIDER_KEY=... ALO_PROVIDER_MODEL=gpt-4o ALO_ROUNDS=3 \
//!   cargo test -p alo-agentd --test a_question_is_answered_in_words -- --ignored --nocapture
//! ```

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use alo_agent::{HowFar, NotAnswered, TheDoor, WhatHappened, driving};
use alo_agentd::questions::{Questions, TheBound, WhoseKeyring};
use alo_agentd::{starting, what_an_agent_said};
use alo_capability::{Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving as _};
use alo_instructing::{Instructions, WhatItMayReach};
use alo_keeping::Writing;
use alo_models::{Catalogue, SourcePolicy};
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::Strings;
use alo_turn::{Machine, Turning};
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;

/// The provider this names, in the settings and in the keyring.
const THE_PROVIDER: &str = "the-owners-test-provider";

/// Who the agent is in this turn.
const THE_AGENT: &str = "@files";

/// How long the turn and the question a person is asked last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// **A boundary that is not one**, named where a reader sees it. The header
/// says why this measurement does not need the kernel's.
#[derive(Debug)]
struct NothingIsBounded;

impl alo_turn::Bounding for NothingIsBounded {
    fn carrying_out(
        &mut self,
        _reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), alo_turn::NoBoundary> {
        doing();
        Ok(())
    }
}

/// The door the socket would have been, in this process, keeping every line.
struct TheDaemonInThisProcess<'a, 'm, 't> {
    turning: &'a mut Turning<'m, 't>,
    questions: &'a mut Questions,
    grants: &'a Grants,
    strings: &'a Strings,
    now: SystemTime,
    lines: Vec<String>,
}

impl TheDoor for TheDaemonInThisProcess<'_, '_, '_> {
    fn asking(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered> {
        let line = request
            .written()
            .map_err(|why| NotAnswered::NotTheProtocol {
                why: format!("the request could not be written: {why}"),
            })?;
        self.lines.push(line.clone());
        Ok(what_an_agent_said(
            &line,
            self.turning,
            self.questions,
            None,
            self.grants,
            self.strings,
            hour(),
            self.now,
        ))
    }
}

/// How one turn ended, as a person would have seen it.
#[derive(Debug, PartialEq, Eq)]
enum Ended {
    /// The model answered in words and proposed nothing.
    Answered,
    /// The model proposed a change at some point, to a request that asked for none.
    Proposed(String),
    /// Anything else: rounds ran out, the machine could not be asked, refused.
    Otherwise(String),
}

/// The verb of the first `propose` line, if the model sent one.
fn first_proposal(lines: &[String]) -> Option<String> {
    lines.iter().find_map(|line| {
        let value: serde_json::Value = serde_json::from_str(line).ok()?;
        let verb = value.pointer("/asks/propose/verb")?.as_str()?;
        Some(verb.to_owned())
    })
}

/// One turn: a request in a person's words, over a folder they granted.
fn one_turn(key: &str, endpoint: &str, model: &str, request: &str, instructions: Instructions) -> Ended {
    let keyring = alo_keyring_fixture::AKeyringOfOurOwn::started("a-question-is-answered");
    let named = format!("provider/{THE_PROVIDER}");
    {
        let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
            .expect("the fixture's address is an address")
            .build()
            .expect("the fixture's bus accepts a connection");
        let service = SecretService::connect_with_existing(EncryptionType::Dh, connection)
            .expect("the fixture serves the Secret Service");
        let collection = service
            .get_default_collection()
            .expect("the fixture has a default collection");
        let mut attributes = HashMap::new();
        attributes.insert("xdg:schema", "dev.alo.Provider");
        attributes.insert("reference", named.as_str());
        collection
            .create_item("the owner's test provider key", attributes, key.as_bytes(), true, "text/plain")
            .expect("an item can be stored");
    }

    let theirs = std::env::temp_dir().join(format!(
        "alo-question-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos())
    ));
    let config = theirs.join(alo_choosing::THE_FOLDER);
    std::fs::create_dir_all(&config).expect("a directory for the person's settings");
    std::fs::write(
        config.join(alo_choosing::THE_SETTINGS),
        format!(
            "format = 2\n\n[answers]\nprovider = {{ name = \"{THE_PROVIDER}\", model = \
             \"{model}\" }}\n\n[[provider]]\nname = \"{THE_PROVIDER}\"\nendpoint = \
             \"{endpoint}\"\nregion = \"not stated for a test key\"\n"
        ),
    )
    .expect("the person's settings can be written");

    // A folder called Documents, as on the machine, with something in it.
    let documents = theirs.join("Documents");
    std::fs::create_dir_all(&documents).expect("a Documents folder");
    let documents = OnThisMachine
        .real(&documents)
        .expect("the folder resolves")
        .into_path_buf();
    std::fs::write(documents.join("march.pdf"), "March, 4180.00").expect("an invoice");
    std::fs::write(documents.join("notes.txt"), "call the bank").expect("a note");

    let mut questions = Questions::of_a_session(
        Some(theirs.clone().into_os_string()),
        None,
        Catalogue::built_in().expect("the built-in catalogue reads"),
        TheBound::ThePersons(SourcePolicy::Anywhere),
        WhoseKeyring::On(keyring.bus()),
    );
    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let mut writing = Writing::opening(&theirs.join("record.jsonl")).expect("a record can be opened");
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;

    let now = SystemTime::now();
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(THE_AGENT, Reach::Folder(documents.clone()), now, hour())
            .expect("a grant over a folder this person owns"),
    );

    let offered = alo_turn::the_verbs_a_whole_machine_offers().expect("the machine's verbs declare");
    let may_reach = WhatItMayReach::these([documents.as_path()]);
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        &mut writing,
    )
    .expect("the six declare");
    let mut turning = Turning::beginning(
        Context::at_invocation(now),
        THE_AGENT,
        hour(),
        &mut grants,
        &mut machine,
    )
    .expect("a turn can begin");

    let request = request.replace("{folder}", &documents.display().to_string());
    let mut door = TheDaemonInThisProcess {
        turning: &mut turning,
        questions: &mut questions,
        grants: &grants,
        strings: &strings,
        now,
        lines: Vec::new(),
    };
    let happened = driving(
        &request,
        &offered,
        &may_reach,
        &mut door,
        HowFar::THE_ORDINARY,
        instructions,
    );

    println!("  asked: {request}");
    for line in door.lines.iter().filter(|line| !line.contains("\"ask\"")) {
        println!("    {line}");
    }
    // **What the model was shown on its second round**, when asked for: the
    // first read's answer is in it or it is not, and that is a question about
    // a prompt rather than about the model.
    if std::env::var_os("ALO_SHOW_SECOND_PROMPT").is_some()
        && let Some(second) = door.lines.iter().filter(|line| line.contains("\"ask\"")).nth(1)
    {
        let question = serde_json::from_str::<serde_json::Value>(second)
            .ok()
            .and_then(|value| {
                value
                    .pointer("/asks/ask/question")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| second.clone());
        println!("--- the second prompt ---\n{question}\n--- end ---");
    }
    let ended = match (&happened, first_proposal(&door.lines)) {
        (_, Some(verb)) => Ended::Proposed(verb),
        (WhatHappened::ItWasAnsweredInWords { said, .. }, None) => {
            println!("    answered: {said}");
            Ended::Answered
        }
        (other, None) => Ended::Otherwise(format!("{other:?}")),
    };
    println!("    ended: {ended:?}");
    let _ = std::fs::remove_dir_all(&theirs);
    ended
}

/// **A request to read is answered in words, and nothing is proposed.**
#[test]
#[ignore = "asks a real provider, costs money, and leaves the machine: run it by name"]
fn a_request_to_read_is_answered_and_proposes_nothing() {
    let key = std::env::var("ALO_PROVIDER_KEY").expect("ALO_PROVIDER_KEY names the key to ask with");
    let endpoint =
        std::env::var("ALO_PROVIDER").unwrap_or_else(|_| "https://api.openai.com".to_owned());
    let model = std::env::var("ALO_PROVIDER_MODEL").unwrap_or_else(|_| "gpt-4o".to_owned());
    let rounds: usize = std::env::var("ALO_ROUNDS")
        .ok()
        .and_then(|rounds| rounds.parse().ok())
        .unwrap_or(3);
    let instructions = match std::env::var("ALO_INSTRUCTIONS").as_deref() {
        Err(_) => Instructions::SHOWN_TO_A_TURN,
        Ok("one-example-per-door") => Instructions::OneExamplePerDoor,
        Ok("several-steps") => Instructions::SeveralSteps,
        Ok("as-first-written") => Instructions::AsFirstWritten,
        Ok("every-door-and-an-answer") => Instructions::EveryDoorAndAnAnswer,
        Ok(other) => panic!("ALO_INSTRUCTIONS is one of the four sets, not `{other}`"),
    };
    println!(
        "model {model} at {endpoint}, under {} (sha256 {}), {rounds} round(s) each",
        instructions.named(),
        instructions.digest()
    );

    // The walks' own words, and one that needs reading a file.
    let requests = [
        "list the files in {folder} with their sizes",
        "what is in my documents folder",
        "what does the note in {folder} say",
    ];
    let mut failed = Vec::new();
    for request in requests {
        for round in 1..=rounds {
            println!("round {round}");
            let ended = one_turn(&key, &endpoint, &model, request, instructions);
            if ended != Ended::Answered {
                failed.push(format!("{request}: {ended:?}"));
            }
        }
    }
    let asked = requests.len() * rounds;
    println!("\nanswered {} of {asked}", asked - failed.len());
    assert!(
        failed.is_empty(),
        "{} of {asked} requests to read were not answered:\n  {}",
        failed.len(),
        failed.join("\n  ")
    );
}
