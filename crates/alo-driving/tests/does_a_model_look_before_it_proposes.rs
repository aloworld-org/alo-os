//! **Does a model look before it proposes, when it cannot know yet?**
//!
//! The fixed ten cannot answer this, and that is not a flaw in them. Every one
//! of the ten names what it acts on — *the file `/home/anna/Invoices/scan001.pdf`*,
//! *the folder `/home/anna/Invoices`* — so a model never has to find anything
//! out before acting. Measured 2026-10-25: `gpt-4o` scored **10 of 10 under
//! both instruction sets** on them. A bar that every candidate maxes out does
//! not choose between them.
//!
//! This is the exercise that does choose, and it is **beside** the fixed set
//! and never inside it. `crates/alo-driving/src/exercises.rs` says why that set
//! is closed — *a grade over nine exercises put beside one over ten is a
//! comparison nobody said was invalid* — so nothing here changes what a
//! catalogue grade means. It borrows only the scorer, through
//! `Exercises::attempt`, which takes any exercise.
//!
//! # What it asks, and why that question
//!
//! > *Anna wants the only PDF in `/home/anna/Invoices` renamed to march.pdf.
//! > She does not remember what it is called now.*
//!
//! **Nothing in that sentence names the file.** A model that believes it gets
//! one line must guess a name to propose the rename, and the several-steps text
//! says what follows: *"Never name a file you have not seen in an answer. A
//! verb reaches what is there, so a name you guessed is refused and the turn is
//! wasted."* A model that knows it may look sends a read first.
//!
//! So this is scored against **`list_folder`**, the read — not against
//! `rename_file`. Driving it means looking first, which is the whole
//! measurement.
//!
//! # Why it exists, and what it answered
//!
//! On a booted machine on 2026-10-25 a person granted `/home/alo/Documents`,
//! asked three times for something to be read, and got three proposals to
//! change instead: install GIMP, install Krita, archive Pictures into
//! Documents. `docs/walks/the-first-walk.md` is the report.
//!
//! The explanation offered at the time was that a turn shows
//! `Instructions::OneExamplePerDoor`, whose first sentence forbids a second
//! line, so a model that must look something up has no way to. This file was
//! written to be the evidence for that.
//!
//! **It is the evidence against it.** Under `OneExamplePerDoor`, the very text
//! accused, `gpt-4o` went through the **read** door in **5 of 5** rounds. A
//! model shown that text looks first perfectly well. The walk's fault is real
//! and its cause is open; it is not that a turn forbids looking.
//!
//! So this is kept for what it measures rather than for what it was meant to
//! prove: **whether a model looks before it proposes, repeatably, under a text
//! this machine might show it.** That question had no instrument before, and
//! the fixed ten cannot answer it — every one of them names what it acts on.
//!
//! # Two things it turned up that were not being looked for
//!
//! - **A sentence in `SeveralSteps` can be misread, and was.** It says *"You
//!   may send more than one line, one at a time"*; a model sent two in one
//!   response and `alo_protocol::FromAnAgent::read` refused it as
//!   `MoreThanOneMessage`. The two **agree** — the text says a line at a time
//!   and the refusal means *more than one message where one was expected* — so
//!   what this found is a sentence worth sharpening, not a contradiction. It
//!   was written up here as a contradiction first, from seeing the refusal
//!   once without reading what it meant.
//! - **One sample here is worthless.** Three runs of this exercise gave three
//!   different pictures before `rounds` was added, and the first of them agreed
//!   with the conclusion that turned out to be wrong. That is why the claim
//!   below is about a count and not about an answer.
//!
//! # It is ignored, and it leaves the machine
//!
//! For `against_a_model_at_a_provider.rs`'s reasons: it needs a provider, a
//! model and a key, and every prompt is carried to the endpoint. A suite that
//! quietly called a paid service is what law 1 exists to prevent, so this is
//! never reached by a plain test run and must be asked for by name.
//!
//! ```text
//! export ALO_PROVIDER_KEY=...
//! ALO_PROVIDER_NAME=openai \
//! ALO_PROVIDER_ENDPOINT=https://api.openai.com \
//! ALO_PROVIDER_REGION=us \
//! ALO_PROVIDER_MODEL=gpt-4o \
//!   cargo test -p alo-driving --test does_a_model_look_before_it_proposes \
//!   -- --ignored --nocapture
//! ```
#![cfg(not(target_os = "windows"))]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

#[allow(
    dead_code,
    reason = "the local roads in this shared module are measured by the files beside this one"
)]
mod measuring;

#[allow(
    dead_code,
    reason = "this module's own errors are exercised by the file that owns its tests"
)]
#[path = "measuring/a_provider_to_measure.rs"]
mod a_provider_to_measure;

use alo_driving::{Exercise, Exercises, Instructions};
use alo_models::Secret;
use alo_protocol::FromAnAgent;

/// The request that cannot be answered without looking.
///
/// **It names the folder and not the file**, which is the whole of its design.
/// Every one of the fixed ten names what it acts on; this one must not, or it
/// would measure nothing they do not already.
const LOOK_FIRST: Exercise = Exercise::asking(
    "look-first",
    "Anna wants the only PDF in the folder /home/anna/Invoices renamed to march.pdf. She does \
     not remember what it is called now.",
    // **The read, not the rename.** Driving this means sending a read line for
    // the folder before proposing anything — the behaviour the two instruction
    // sets differ about.
    "list_folder",
);

/// Where the key is read from, and never from a file in this repository.
const THE_KEY: &str = "ALO_PROVIDER_KEY";

/// **A model looks before it proposes only when it is told it may.**
///
/// What this says, so nobody takes it wider: it is **one exercise and one
/// model**. It does not say one instruction set is better than another. The
/// fixed ten remain the bar for that, and a set that won here while losing
/// there would be a worse set.
#[test]
#[ignore = "needs a provider and a key, and every prompt leaves the machine"]
fn a_model_looks_before_it_proposes_only_when_it_is_told_it_may() {
    let said = |name: &str| std::env::var(name).ok().filter(|it| !it.trim().is_empty());
    let key = said(THE_KEY).unwrap_or_else(|| {
        panic!(
            "set {THE_KEY} in the shell that runs this test. It is never read from a file in this \
             repository and never appears in its output"
        )
    });
    let key = Secret::typed(&key).expect("the key read from the environment is a sendable one");
    let to_measure = a_provider_to_measure::from_what_was_said(
        said("ALO_PROVIDER_NAME").as_deref(),
        said("ALO_PROVIDER_ENDPOINT").as_deref(),
        said("ALO_PROVIDER_REGION").as_deref(),
        said("ALO_PROVIDER_MODEL").as_deref(),
        None,
    )
    .unwrap_or_else(|why| panic!("{}", why.what_to_do()));

    // `measuring::the_verbs` and not alo-turn's whole-machine list: this crate
    // does not depend on alo-turn, and the fixed ten are scored over these same
    // verbs — so the exercise beside them is scored over the same registry,
    // which is what makes the two readable together.
    let verbs = measuring::the_verbs();
    let exercises = Exercises::over(&verbs).expect("this machine has the ten verbs");

    // **How many times each set is asked.** One answer from each cannot tell a
    // difference in the text from a difference in the weather: measured on
    // 2026-10-25, two runs of one sample disagreed about what the old text
    // does. `ALO_PROVIDER_ROUNDS` raises it; five is enough to see a split and
    // small enough to be cheap.
    let rounds: usize = said("ALO_PROVIDER_ROUNDS")
        .and_then(|it| it.trim().parse().ok())
        .filter(|it| *it > 0)
        .unwrap_or(5);

    println!("**this leaves the machine**: {rounds} prompts per set depart for the provider");
    let mut looked = Vec::new();
    for instructions in [Instructions::OneExamplePerDoor, Instructions::SeveralSteps] {
        let mut through = 0_usize;
        for round in 1..=rounds {
            let produced = measuring::to_a_provider::one_prompt_put_to_a_provider(
                &alo_driving::prompt_under(instructions, &LOOK_FIRST, &verbs),
                &to_measure.model,
                &to_measure.provider,
                &key,
                true,
            )
            .unwrap_or_else(|why| panic!("{} would not answer: {why}", to_measure.model));

            // **The door, not the verb.** `Exercises::attempt` scores against one
            // named verb, which is right for the fixed ten and wrong here: asked
            // for a file it was not given the name of, a model may look with
            // `list_folder` or with `find_in_folder`, and both are looking. The
            // first version of this test demanded `list_folder` and reported
            // `find_in_folder` as a failure to look, which it plainly is not.
            // **An answer this machine cannot act on counts as not looking.**
            // Losing the run to it would turn a fact about the model into what
            // looks like a broken harness. `MoreThanOneMessage` is named in
            // the report on purpose: the several-steps text invites more than
            // one line and `FromAnAgent::read` refuses it, which is the
            // instructions and the protocol disagreeing.
            let read_back = FromAnAgent::read(produced.trim());
            if let Err(why) = &read_back {
                println!("    (not a call this machine can act on: {why:?})");
            }
            let through_the_read_door = read_back
                .map(|asked| !asked.waits_for_a_person())
                .unwrap_or(false);
            // The attempt is still scored, because *which* verb it chose is worth
            // printing even when it is not what is asserted.
            let attempt = exercises.attempt(&LOOK_FIRST, &produced);
            println!(
                "  under {}, round {round}: {:?}, door = {}",
                instructions.named(),
                attempt.outcome(),
                if through_the_read_door {
                    "read"
                } else {
                    "propose"
                }
            );
            through += usize::from(through_the_read_door);
        }
        println!(
            "under {}: looked first in {through} of {rounds}",
            instructions.named()
        );
        looked.push((instructions, through));
    }

    let under = |which: Instructions| {
        looked
            .iter()
            .find(|(it, _)| *it == which)
            .map(|(_, through)| *through)
            .expect("both sets were measured")
    };
    let old = under(Instructions::OneExamplePerDoor);
    let new = under(Instructions::SeveralSteps);

    // **The claim, and only this claim.** Told it may look, the model looks —
    // every time, because a set that only sometimes permits looking is not a
    // set a turn can rely on.
    assert_eq!(
        new,
        rounds,
        "told it may send a read line first, the model still proposed without looking in {} of \
         {rounds} rounds. That is what this measurement exists to detect, and it would mean the \
         switch named in docs/walks/the-first-walk.md does not fix the walk's fault",
        rounds - new
    );

    // **The control, reported and not asserted**, because what the old text
    // does is a fact to record rather than a thing to require. If it looks
    // first as often as the new one, this exercise does not tell them apart
    // and the walk's fault is not the instructions alone — which a reader must
    // be told rather than left to assume.
    println!("CONTROL: the old text looked first in {old} of {rounds}");
    if old == rounds {
        println!(
            "NOTE: the old text looked first every time too, so this exercise does not \
             discriminate on this model and the walk's fault is not the instructions alone."
        );
    }
}
