//! **The grammar accepts every call `alo-capability` accepts, and nothing else
//! the protocol would refuse.**
//!
//! Task 17 of `docs/autonomy/v0-5-the-models-measured-plan.md`. A grammar that
//! forbade a call the machine would have acted on would measure the grammar
//! rather than the model, and one that allowed a call the protocol refuses would
//! measure nothing at all. So every verb this machine declares is written out as
//! the call a correct answer makes and put to the grammar, and the ways of
//! getting it wrong are put to it too.
//!
//! **What reads the grammar here is this file, not `llama.cpp`.** The engine's
//! own parser is the authority on GBNF, and nothing in this workspace can stand
//! in for it; what this holds is that the grammar `alo_driving::grammar_for`
//! writes says what it means to say, in the subset of GBNF that function emits —
//! literals, alternation, sequence, groups, character classes and `*` and `?`.
//! That the engine reads it the same way is checked by the measurement itself:
//! a run where the grammar and the engine disagreed would answer nothing, or
//! answer something `alo-protocol` refuses, and the report would say so.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::{Effect, Takes, Verbs};
use alo_driving::{digest_of, grammar_for};

mod gbnf;
// Only `the_verbs` is wanted here: this file reads a grammar and puts calls to
// it, and never asks a model anything.
#[allow(
    dead_code,
    reason = "the measurement itself is run by against_a_model_on_this_machine.rs"
)]
mod measuring;

/// The call a correct answer to an exercise about this verb makes.
fn the_call(verbs: &Verbs, name: &str) -> String {
    let verb = verbs.all().find(|verb| verb.name() == name).unwrap();
    let door = match verb.effect() {
        Effect::Read => "read",
        Effect::Change => "propose",
    };
    let given: Vec<String> = verb
        .args()
        .iter()
        .map(|arg| {
            let is = match arg.takes() {
                Takes::Path => "\"/home/anna/Invoices\"".to_owned(),
                Takes::Application => "\"org.alo.Writer\"".to_owned(),
                Takes::Name { .. } => "\"march.pdf\"".to_owned(),
                Takes::Count { most, .. } => format!("{most}"),
                Takes::Choice(options) => format!("\"{}\"", options.first().unwrap().name()),
            };
            format!("{{\"named\":\"{}\",\"is\":{is}}}", arg.name())
        })
        .collect();
    format!(
        "{{\"format\":1,\"asks\":{{\"{door}\":{{\"verb\":\"{}\",\"given\":[{}]}}}}}}",
        verb.name(),
        given.join(",")
    )
}

/// **Every verb this machine declares can be called through the grammar** — all
/// ten of them, through the door its effect requires.
#[test]
fn every_call_this_machine_accepts_is_a_call_the_grammar_allows() {
    let verbs = measuring::the_verbs();
    let grammar = gbnf::Grammar::of(&grammar_for(&verbs));
    let mut held = 0;
    for verb in verbs.all() {
        let call = the_call(&verbs, verb.name());
        assert!(
            grammar.accepts(&call),
            "the grammar refuses a call this machine accepts: {call}"
        );
        // And what `alo-protocol` and `alo-capability` make of it is a call that
        // drives the verbs, which is the other half of *accepts*.
        let attempt = alo_driving::Exercises::over(&verbs)
            .unwrap()
            .all()
            .find(|exercise| exercise.verb() == verb.name())
            .map(|exercise| {
                alo_driving::Exercises::over(&verbs)
                    .unwrap()
                    .attempt(exercise, &call)
            });
        if let Some(attempt) = attempt {
            assert!(
                !matches!(attempt.outcome(), alo_driving::Outcome::NotAMessage(_)),
                "{}: {:?}",
                verb.name(),
                attempt.outcome()
            );
        }
        held += 1;
    }
    assert_eq!(held, verbs.all().count());
    assert!(held >= 10, "only {held} verbs were put to the grammar");
}

/// **And every way of getting it wrong that the wrapper's schema could not
/// prevent is refused by the grammar.**
#[test]
fn the_ways_a_call_goes_wrong_are_refused_by_the_grammar() {
    let verbs = measuring::the_verbs();
    let grammar = gbnf::Grammar::of(&grammar_for(&verbs));
    let right = the_call(&verbs, "rename_file");
    assert!(grammar.accepts(&right));

    for (wrong, what) in [
        (
            right.replace("\"propose\"", "\"read\""),
            "a change through the read door",
        ),
        (
            right.replace("\"named\":\"file\",\"is\":", "\"is\":\"x\",\"named\":"),
            "the protocol's keys the other way round",
        ),
        (
            right.replace("rename_file", "rename_the_file"),
            "a verb this machine does not have",
        ),
        (
            right.replace("\"named\":\"name\"", "\"named\":\"to\""),
            "an argument this verb does not take",
        ),
        (
            right.replace("\"format\":1", "\"format\":2"),
            "an envelope of another version",
        ),
        (
            format!("{right} and here is why"),
            "a sentence after the call",
        ),
        (
            right.replace(",\"given\":[", ",\"with\":["),
            "a key the protocol does not have",
        ),
    ] {
        if wrong == right {
            panic!("the case for `{what}` did not change the call");
        }
        assert!(
            !grammar.accepts(&wrong),
            "the grammar allows {what}: {wrong}"
        );
    }

    // A read verb through the read door is still allowed, so the door rule is
    // about the effect and not about refusing one door.
    assert!(grammar.accepts(&the_call(&verbs, "list_folder")));
}

/// **The grammar is named by its digest**, as a grade under it records it.
#[test]
fn a_grammar_is_named_by_the_digest_of_its_text() {
    let verbs = measuring::the_verbs();
    let grammar = grammar_for(&verbs);
    let digest = digest_of(&grammar);
    assert_eq!(digest.len(), 64);
    assert!(
        digest
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    );
    assert_eq!(
        digest,
        digest_of(&grammar_for(&verbs)),
        "the same verbs write the same grammar"
    );
    assert_ne!(
        digest,
        digest_of(&format!("{grammar}\n")),
        "a grammar that differs by a byte is another grammar"
    );
}
