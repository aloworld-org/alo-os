//! **Does the agent actually do anything?**
//!
//! Every other measurement in this repository stops one step short of that.
//! `alo-driving` grades whether a model's answer is *a call this machine would
//! act on* — `Outcome::Drove` is literally **would**, not did — and
//! `the_agent_asks_a_real_provider` proves the daemon can put a question to a
//! provider with a key it looked up. Neither renames a file.
//!
//! This does. A real model is asked what to do about a request in plain words,
//! its answers are carried out against a real folder under a real grant inside
//! the kernel's boundary, the change it proposes is approved, and then **the
//! file is looked for at its new name on the disk**.
//!
//! # What stands in for what, said before anything is claimed
//!
//! | | |
//! |---|---|
//! | the model | real, at a provider, with a key from a real keyring |
//! | the verbs | real: `alo_files::file_verbs`, against a real folder |
//! | the boundary | real: `ByTheKernel`, so what may be reached is the kernel's answer |
//! | the record | real, written to a real file |
//! | the turn | real: `alo_turn::Turning`, which decides every step |
//! | **the socket** | **absent** — see below |
//! | **the person** | **this test**, which approves the change |
//!
//! **The socket is the one thing missing, and it cannot be here.**
//! `two_doors_on_one_socket` says why: *telling the two apart takes two logins,
//! and a test process has one. Whoever runs the tests is the person … a real
//! agent connecting as its own user is owed with the rest of the verification
//! that needs a machine set up as a machine.* The daemon refuses to start where
//! the person and the agent are one login. So this drives
//! `alo_agentd::what_an_agent_said` — *one line from an agent, carried out
//! against the turn it belongs to*, which is the function the socket hands
//! every line to — and what is untested here is `SO_PEERCRED` and nothing else.
//! `alo_agent::AtTheDaemonsSocket` is the door that needs the two users.
//!
//! **This test plays the person**, calling `Turning::approving` where a shell
//! on the person's door would. What that skips is the shell, not the rule: the
//! approval is spent once by the same code either way, which
//! `one_approval_and_the_sentence_a_person_approves` holds from the other side.
//!
//! # It is skipped without a key, and loudly
//!
//! It asks a real provider and costs money, so the gate runs it with no key and
//! it skips — printing what it did not do, because a quiet pass here is the one
//! shape of pass worth being suspicious of.
//!
//! # Three failure modes, and which of them the machine can catch
//!
//! Measured on 2026-10-13, `gpt-4o-mini`, the request *find whichever invoice is
//! for March and add `-final` to its name* over a folder holding `march.pdf`
//! and `april.pdf`.
//!
//! **A shape the protocol cannot read** — the verb's name where the door
//! belongs. Fixed, and by construction rather than by asking: ADR 0107 holds a
//! provider to a schema and the shape became impossible. Four runs after it,
//! none.
//!
//! **A path that is not there** — `March_invoice.txt`, invented. Caught when
//! the change is *proposed*, by `alo_turn::the_sentence_is_about_something`, and
//! **the agent then corrects itself**: it searched, was shown `march.pdf`, and
//! proposed renaming that. This is the loop working.
//!
//! **A path that is there and is the wrong one — and the machine cannot catch
//! this.** Two of four runs proposed renaming the *folder*,
//! `given: [{"named":"file","is":".../Invoices"},{"named":"name","is":"-final"}]`.
//! The folder exists. The grants cover it. The argument names are right. There
//! is nothing for the capability model to refuse, and no schema can fix a value
//! that is well-formed and wrong.
//!
//! **What catches it is the sentence**, and only the sentence: *rename
//! …/Invoices to -final*. A person reads that and says no. Which is why
//! `CLAUDE.md`'s *what a person approves is that sentence* is load-bearing
//! rather than decorative, and why the sentence being **true** — the previous
//! commit's work — is what makes it readable at all.
//!
//! So two of four runs here end with a person declining something daft, which
//! is the system working. This test asserts the work was *done*, so it fails on
//! those: the assertion is about the model finishing a job, not about the
//! machine being safe, and the machine was safe in all four.
//!
//! # And what a failure here means
//!
//! **A model that cannot finish this is a result, not a broken test.** No
//! multi-step turn has ever been measured: every grade in
//! `data/catalogue.toml` was earned at one step, and `gpt-4o-mini` scoring ten
//! of ten on 2026-10-13 was ten single-step exercises. If a model reads the
//! folder and then cannot decide what to propose, that is the first measurement
//! anybody has of the thing `ROADMAP.md`'s launch list actually promises, and
//! it is worth more than a green tick.

#![cfg(target_os = "linux")]
// Counted rather than copied: no bare `unwrap` is left in this file, so
// `clippy::unwrap_used` is not listed — an unfulfilled expectation is an error
// in this workspace.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime};

use alo_agent::{HowFar, NotAnswered, TheDoor, WhatHappened, driving};
use alo_agentd::questions::{Questions, TheBound, WhoseKeyring};
use alo_agentd::{ByTheKernel, starting, what_an_agent_said};
use alo_bounding::{Imposed, Pinned};
use alo_capability::{Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving as _};
use alo_instructing::Instructions;
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

/// **One of these on this machine at a time, and nothing forced.**
///
/// The sibling `the_agent_asks_a_real_provider`'s guard, for its two reasons: a
/// mutex so two tests in one binary do not impose a boundary at once, and
/// `Waited::on_this_kernel` so another checkout's kernel tests wait rather than
/// collide.
fn the_only_one_on_this_machine() -> (MutexGuard<'static, ()>, alo_bounding::Waited) {
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    let ours = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);
    let kernel = alo_bounding::Waited::on_this_kernel()
        .expect("this kernel can be taken, and nothing is forced if it cannot");
    (ours, kernel)
}

/// The boundary, imposed and taken away with this value.
struct AsAMachineHasIt {
    pinned: Pinned,
    _loaded: Imposed,
}

impl AsAMachineHasIt {
    fn on_this_kernel(what: &str) -> Self {
        let pinned = Pinned::beneath(
            &PathBuf::from("/sys/fs/bpf").join(format!("alo-does-{}-{what}", std::process::id())),
        );
        pinned.taken_away();
        pinned
            .made()
            .expect("this machine has a BPF filesystem at /sys/fs/bpf");
        let loaded = Imposed::once(&pinned).unwrap_or_else(|why| {
            panic!("no boundary could be imposed, so nothing here is being tested: {why}")
        });
        Self {
            pinned,
            _loaded: loaded,
        }
    }
}

impl Drop for AsAMachineHasIt {
    fn drop(&mut self) {
        self.pinned.taken_away();
    }
}

/// **The door the socket would have been**, in this process.
///
/// `what_an_agent_said` is the function `alo-agentd` hands every line from an
/// agent to. Holding the turn here rather than a stream is what makes this
/// testable at all on a machine with one login — and it is the only
/// substitution in this file, named in the header beside what it costs.
struct TheDaemonInThisProcess<'a, 'm, 't> {
    turning: &'a mut Turning<'m, 't>,
    questions: &'a mut Questions,
    grants: &'a Grants,
    strings: &'a Strings,
    now: SystemTime,
    /// Every line it was given, so the test can say what the model really asked
    /// for rather than describing it.
    lines: Vec<String>,
    /// How long the **model** took, summed over every question put to it.
    ///
    /// The owner asked for an end-to-end number as well as a standing one, and
    /// a single total would not answer it: almost all of a turn is a provider
    /// thinking, and *alo OS is slow* and *the model is slow* are different
    /// claims with different answers. So the two are separated here, at the one
    /// place that can tell them apart — a `FromAnAgent::Ask` is a question to a
    /// model and everything else is this machine.
    ///
    /// `alo-turn`'s `what_a_turn_costs` measures this machine's share on its
    /// own, far more precisely and on every build. This is the other half: what
    /// a person actually waits through.
    asking_a_model: Duration,
    /// How many times a model was asked, so the total can be read per round
    /// trip rather than only in sum.
    rounds: usize,
}

impl TheDoor for TheDaemonInThisProcess<'_, '_, '_> {
    fn asking(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered> {
        let line = request
            .written()
            .map_err(|why| NotAnswered::NotTheProtocol {
                why: format!("the request could not be written: {why}"),
            })?;
        self.lines.push(line.clone());
        // **Only a question to a model is the model's time.** A `read` or a
        // `propose` goes nowhere near a provider, so counting the whole call
        // would attribute this machine's work to whoever is answering.
        let asking_a_model = matches!(request, FromAnAgent::Ask { .. });
        let began = Instant::now();
        let answered = what_an_agent_said(
            &line,
            self.turning,
            self.questions,
            None,
            self.grants,
            self.strings,
            hour(),
            self.now,
        );
        if asking_a_model {
            self.asking_a_model += began.elapsed();
            self.rounds += 1;
        }
        Ok(answered)
    }
}

/// **A real model drives real verbs over a real folder, and the file moves.**
#[test]
fn a_real_model_is_asked_in_words_and_a_file_really_moves() {
    let Ok(key) = std::env::var("ALO_PROVIDER_KEY") else {
        println!(
            "SKIPPED, and this is not a pass: ALO_PROVIDER_KEY is not set, so no model was \
             asked anything and no file moved. This test asks a real provider and costs money, \
             so the gate runs it with no key and it skips. Nothing else in this repository \
             measures whether the agent does any work, so a green run without this is a green \
             run that has not answered that question."
        );
        return;
    };
    let endpoint =
        std::env::var("ALO_PROVIDER").unwrap_or_else(|_| "https://api.openai.com".to_owned());
    let model = std::env::var("ALO_PROVIDER_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_owned());

    // A real Secret Service on a bus nobody else can reach, stored through a
    // client of its own rather than through `TheKeyring` — `alo-secrets`'
    // reason: a test that stored and read through one object would be proving
    // itself.
    let keyring = alo_keyring_fixture::AKeyringOfOurOwn::started("the-agent-does");
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
            .create_item(
                "the owner's test provider key",
                attributes,
                key.as_bytes(),
                true,
                "text/plain",
            )
            .expect("an item can be stored");
    }

    // The person's own folder: their settings, their record, and the folder the
    // agent is granted. Written rather than built, so what the daemon parses is
    // what a person would have.
    let theirs = std::env::temp_dir().join(format!("alo-agent-does-{}", std::process::id()));
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

    // The work: one folder, two files, and a grant over it. Resolved, because a
    // grant is over a real place and `docs/quirks.md` records why.
    let invoices = theirs.join("Invoices");
    std::fs::create_dir_all(&invoices).expect("a folder of invoices");
    let invoices = OnThisMachine
        .real(&invoices)
        .expect("the folder resolves")
        .into_path_buf();
    let march = invoices.join("march.pdf");
    std::fs::write(&march, "March, 4180.00").expect("an invoice");
    std::fs::write(invoices.join("april.pdf"), "April, 220.00").expect("another invoice");

    let mut questions = Questions::of_a_session(
        Some(theirs.clone().into_os_string()),
        None,
        Catalogue::built_in().expect("the built-in catalogue reads"),
        TheBound::ThePersons(SourcePolicy::Anywhere),
        WhoseKeyring::On(keyring.bus()),
    );

    let _one = the_only_one_on_this_machine();
    let kernel = AsAMachineHasIt::on_this_kernel("does");
    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let mut writing =
        Writing::opening(&theirs.join("record.jsonl")).expect("a record can be opened");
    let mut indicator = Indicator::default();
    let mut bounding =
        ByTheKernel::beneath(&kernel.pinned).expect("a service can open the map a loader pinned");

    let now = SystemTime::now();
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(THE_AGENT, Reach::Folder(invoices.clone()), now, hour())
            .expect("a grant over a folder this person owns"),
    );

    // **Which instructions, named rather than assumed.** A grade is about a
    // set, so a run says which one it was under: `one-example-per-door` is what
    // a turn shows today, and `several-steps` is the set written on 2026-10-13
    // after both models guessed a filename rather than looking.
    let instructions = match std::env::var("ALO_INSTRUCTIONS").as_deref() {
        Err(_) | Ok("one-example-per-door") => Instructions::OneExamplePerDoor,
        Ok("several-steps") => Instructions::SeveralSteps,
        Ok("as-first-written") => Instructions::AsFirstWritten,
        Ok(other) => panic!("ALO_INSTRUCTIONS is one of the three sets, not `{other}`"),
    };
    println!(
        "under the instructions {}, sha256 {}",
        instructions.named(),
        instructions.digest()
    );

    // The request, in the words a person would use. Nothing in it names a verb.
    //
    // **Overridable, because the wording is the measurement.** A request that
    // names the file is one step; one that says *whichever invoice is for
    // March* is two, because the model has to look first. `ALO_REQUEST` is how
    // a harder one is put without editing this file, and `{folder}` in it is
    // replaced with the folder the agent was granted.
    let request = std::env::var("ALO_REQUEST").map_or_else(
        |_| {
            format!(
                "In the folder {}, rename the file march.pdf to march-final.pdf.",
                invoices.display()
            )
        },
        |asked| asked.replace("{folder}", &invoices.display().to_string()),
    );
    println!("the person asked: {request}");

    // **What the model is told about.** `file_verbs()` is the same function
    // `Machine::carrying_out_file_verbs` builds its registry from, so the two
    // lists are equal here by construction.
    //
    // **In production they are not equal by construction, and that is a gap**
    // rather than a detail of this test: the agent is on the other side of a
    // socket and the agent's door has `read`, `propose` and `ask` and no way to
    // ask *what can I do here*. So an agent that built its list from
    // `alo_declared::every_verb_this_machine_ships` would tell a model about
    // verbs the daemon's machine does not offer, and the model would spend its
    // rounds being refused. Named here because this is where it was noticed.
    let offered = alo_files::file_verbs().expect("the six declare");
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

    let (happened, lines) = {
        let mut door = TheDaemonInThisProcess {
            turning: &mut turning,
            questions: &mut questions,
            grants: &grants,
            strings: &strings,
            now,
            lines: Vec::new(),
            asking_a_model: Duration::ZERO,
            rounds: 0,
        };
        // **Enter to done**, which is the number the owner asked for beside the
        // standing one. The clock starts on the request and stops when the loop
        // has finished, so what it covers is exactly what a person waits
        // through.
        let pressed_enter = Instant::now();
        let happened = driving(&request, &offered, &mut door, HowFar::of(6), instructions);
        let enter_to_done = pressed_enter.elapsed();
        let this_machine = enter_to_done.saturating_sub(door.asking_a_model);
        println!("\n--- Enter to done ---");
        println!("  the whole turn          {enter_to_done:?}");
        println!(
            "  of which the model      {:?} over {} round trip(s)",
            door.asking_a_model, door.rounds
        );
        println!("  of which this machine   {this_machine:?}");
        if enter_to_done.as_micros() > 0 {
            let share = door.asking_a_model.as_micros() * 100 / enter_to_done.as_micros();
            println!("  the model's share       {share}%");
        }
        println!(
            "  model {model} at {}",
            std::env::var("ALO_PROVIDER").unwrap_or_else(|_| "https://api.openai.com".to_owned())
        );
        println!("--- (one run, on whatever this ran on; nothing re-measures it) ---\n");
        (happened, door.lines)
    };

    println!("\nwhat the agent asked for, in order:");
    for line in &lines {
        // The question is the long one; print what it asked the machine for.
        if !line.contains("\"ask\"") {
            println!("  {line}");
        }
    }
    println!("\nhow it ended: {happened:?}\n");

    // **The measurement.** Everything above is the arrangement; this is the
    // result, and a failure here is a fact about the model rather than a fault
    // in the test — see this file's header.
    let WhatHappened::AChangeIsWaiting {
        number, sentence, ..
    } = &happened
    else {
        panic!(
            "the model did not reach a change to propose. It took {} step(s). This is the first \
             measurement of a multi-step turn anybody has, and it says the model could not \
             finish one: {happened:?}",
            happened.after().len()
        );
    };
    println!("the sentence the person is asked: {sentence}");
    assert!(
        sentence.contains("march"),
        "the sentence a person would approve does not name the file: {sentence}"
    );

    // And now the person says yes. This test plays them — the header says so —
    // and the approval is spent by the same code a shell spends it with.
    // The number is found among what is really waiting, which is what
    // `alo_agentd::answering`'s own `under` does and the only road there is: a
    // `ProposalId` cannot be made from a `u64`, because *a number for it would
    // be a number an agent could change*.
    let waiting = turning
        .waiting_at(now)
        .find(|waiting| waiting.id.as_u64() == *number)
        .map(|waiting| waiting.id)
        .expect("the change the model proposed is the one waiting");
    let did = turning
        .approving(waiting, &grants, now)
        .expect("the person approved the sentence the model proposed");
    println!("what the machine did: {did:?}");

    // **The file, on the disk.** This is the line that answers the question in
    // this file's title; every assertion above it is about intentions.
    let finished = invoices.join("march-final.pdf");
    assert!(
        finished.is_file(),
        "the change was approved and {} is not there, so nothing was done",
        finished.display()
    );
    assert!(
        !march.is_file(),
        "{} is still there, so it was copied rather than renamed",
        march.display()
    );
    assert_eq!(
        std::fs::read_to_string(&finished).expect("the renamed invoice reads"),
        "March, 4180.00",
        "the file at the new name is not the one that was there"
    );
    println!(
        "\nTHE AGENT DID THE WORK: {} is now {}",
        march.display(),
        finished.display()
    );
}
