//! **One walk, in the order a person meets it.**
//!
//! Task 6 of `docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`.
//! Every other test in these crates holds one sentence, or one step, or one
//! refusal. This holds **the sequence**: what somebody actually reads, from
//! granting a folder to running out of credit, in order, in one place.
//!
//! A machine can pass every test of its parts and still make no sense read
//! through. The table below is the read-through, and it is a test so that a
//! change to any sentence has to face the whole walk rather than only its own
//! assertion.
//!
//! **Nothing here re-decides what the sentences say.** It reads them.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Filling, Strings};

/// The walk, as a person meets it.
///
/// Each row is what they are doing, and the key of the sentence they read while
/// doing it. The keys are looked up in the machine's whole vocabulary, so a
/// sentence that is not collected fails here as loudly as one that is wrong.
const THE_WALK: [(&str, &str); 12] = [
    // Teaching the model from their own documents.
    ("they choose a folder", "adapting.which-folder"),
    ("they say what they want", "adapting.what-it-should-learn"),
    (
        "they see what it will read",
        "adapting.what-it-will-learn-from",
    ),
    ("and what it will skip", "adapting.not-trained-on"),
    ("they approve it", "adapting.start-teaching"),
    (
        "it finishes, and says what it cost",
        "adapting.what-it-cost",
    ),
    ("they decide whether to keep it", "adapting.keep-it-or-not"),
    // Living with it afterwards.
    (
        "later, they take one folder's teaching back",
        "adapting.deleting-this-adapter",
    ),
    (
        "or they revoke the folder itself",
        "adapting.revoking-a-trained-folder",
    ),
    ("and the button that does it", "adapting.delete-the-adapter"),
    // Adding a provider, and running out.
    (
        "their own model cannot be the agent",
        "models.agent.none-clears-the-bar",
    ),
    ("the money runs out", "answering.wrong.ran-out"),
];

/// What the walk's sentences are filled with.
///
/// One of them names where an answer came from, because *nothing was answered*
/// is not a sentence until it says by whom. The provider here is alo's own,
/// which is the point of the last two steps: a person adds it, asks, and the
/// money runs out — and what they read is what they would read about anybody.
fn what_the_walk_fills() -> Filling {
    Filling::of("source", "by alo, in the EU")
}

/// The vocabulary a machine really holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("the machine's vocabulary"))
}

/// **The whole walk reaches a person as sentences, in order.**
#[test]
fn every_step_of_the_walk_is_a_sentence_this_machine_can_say() {
    let strings = what_this_machine_can_say();
    let mut walked: Vec<String> = Vec::new();
    for (doing, key) in THE_WALK {
        let key = alo_strings::Key::unchecked(key);
        let said = strings.say(&key, &what_the_walk_fills());
        assert!(
            !said.text().contains(key.to_string().as_str()),
            "{doing}: reaches a person as a key rather than a sentence"
        );
        assert!(!said.text().is_empty(), "{doing}: says nothing");
        walked.push(format!("{doing} — {}", said.text()));
    }
    assert_eq!(walked.len(), THE_WALK.len());

    // Printed so a person reading the report reads what a person reads.
    for line in &walked {
        println!("{line}");
    }
}

/// **No sentence in the walk names a thing we rented.**
///
/// The disclosed chain names Mistral as the model's maker, which is provenance
/// and belongs to a person; what may not appear is the plumbing — the library
/// that trains, the runtime that serves, the API that is spoken.
#[test]
fn nothing_in_the_walk_names_the_plumbing() {
    let strings = what_this_machine_can_say();
    let mut named: Vec<String> = Vec::new();
    for (doing, key) in THE_WALK {
        let said = strings
            .say(&alo_strings::Key::unchecked(key), &what_the_walk_fills())
            .text()
            .to_lowercase();
        for plumbing in [
            "lora",
            "qlora",
            "peft",
            "transformers",
            "safetensors",
            "gguf",
            "ollama",
            "llama.cpp",
            "checkpoint",
            "epoch",
            "learning rate",
            "api",
            "endpoint",
        ] {
            if said.contains(plumbing) {
                named.push(format!("{doing} says `{plumbing}`"));
            }
        }
    }
    assert!(named.is_empty(), "the walk names the plumbing: {named:#?}");
}

/// **Every sentence these crates say has a translator's note.**
///
/// A sentence without one is a sentence somebody guesses at, and the guess is
/// made in twenty-three languages at once.
#[test]
fn every_sentence_these_crates_say_tells_a_translator_what_it_is_for() {
    let mut without: Vec<String> = Vec::new();
    for word in alo_adapting::words::EVERY_WORD {
        if word.note().is_none() {
            without.push(word.key().to_string());
        }
    }
    assert!(
        without.is_empty(),
        "sentences with nothing to tell a translator: {without:#?}"
    );
    assert_eq!(alo_adapting::words::EVERY_WORD.len(), 10);
}
