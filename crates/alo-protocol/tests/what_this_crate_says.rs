//! What this crate says, against the vocabulary it really declares.
//!
//! The unit tests beside each file check a sentence at a time. These check the
//! list as a whole, from outside the crate, the way a shell holds it: one
//! vocabulary, everything looked up through it, and a translation checked
//! against the source rather than against a copy of it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_protocol::{EVERY_WORD, FromAPerson, FromAnAgent, NotUnderstood, protocol_words, words};
use alo_strings::{Language, Strings, Translation, Vocabulary};

/// Every way a message can fail to be a request, made by the doors themselves
/// rather than built by hand — so a refusal that no message can actually
/// produce would fail here rather than pass.
fn every_refusal_a_door_can_make() -> Vec<NotUnderstood> {
    let too_long = format!(
        r#"{{"format":1,"asks":{{"ask":{{"question":"{}"}}}}}}"#,
        "a".repeat(alo_protocol::LONGEST)
    );
    vec![
        FromAnAgent::read(&too_long).unwrap_err(),
        FromAnAgent::read("{\"format\":1,\"asks\":{\"ask\":{\"question\":\"a\"}}}\nand again")
            .unwrap_err(),
        FromAnAgent::read(r#"{"format":2,"asks":{"whatever":{}}}"#).unwrap_err(),
        FromAnAgent::read(r#"{"format":0,"asks":{"ask":{"question":"a"}}}"#).unwrap_err(),
        FromAnAgent::read("nothing like a message").unwrap_err(),
        FromAnAgent::read(r#"{"format":1,"asks":{"approve":{"number":7}}}"#).unwrap_err(),
        FromAPerson::read(r#"{"format":1,"asks":{"ask":{"question":"a"}}}"#).unwrap_err(),
    ]
}

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// **Every refusal a door can make has a sentence of its own**, and there are
/// exactly as many of them as there are words. A list with a word nothing says
/// is a translator's time spent on a string nobody reads; a door with a refusal
/// nothing declares is a key on somebody's screen.
#[test]
fn every_refusal_a_client_can_meet_says_something_of_its_own() {
    let strings = Strings::of(protocol_words().unwrap());
    let mut said: Vec<String> = every_refusal_a_door_can_make()
        .iter()
        .map(|why| why.said(&strings).into_text())
        .collect();
    assert_eq!(said.len(), EVERY_WORD.len());
    said.sort();
    said.dedup();
    assert_eq!(
        said.len(),
        EVERY_WORD.len(),
        "two refusals a client can meet say the same thing"
    );
}

/// **Everything this crate says is something it declares.** The road from a
/// refusal to a sentence is `NotUnderstood::word`, and a word it named that
/// nothing declared would compile and reach a person as a key.
#[test]
fn everything_this_crate_says_is_something_it_declares() {
    let vocabulary = protocol_words().unwrap();
    let strings = Strings::of(vocabulary);
    for why in every_refusal_a_door_can_make() {
        let said = why.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().len() > 10, "{said}");
    }
}

/// **A machine that loaded no words at all refuses exactly the same things.**
/// What a message *is* was decided before anybody asked for a sentence, so a
/// vocabulary that failed to load cannot make a daemon accept something.
#[test]
fn a_machine_with_no_words_refuses_the_same_messages() {
    let nothing = Strings::of(Vocabulary::empty());
    let refused = every_refusal_a_door_can_make();
    assert_eq!(refused.len(), EVERY_WORD.len());
    for why in refused {
        let said = why.said(&nothing);
        assert!(said.is_a_bug(), "{said}");
        assert!(said.text().contains("protocol."), "{said}");
    }
}

/// A refusal reads in the person's own language, and the ones that matter most
/// are the two about who asked.
#[test]
fn a_refusal_reads_in_the_language_the_person_has() {
    let vocabulary = protocol_words().unwrap();
    let translation = Translation::into_language(german())
        .says(
            words::NOT_FOR_AN_AGENT.key(),
            "ein Assistent kann keine Frage beantworten, die einem Menschen gestellt wurde",
        )
        .says(
            words::NOT_FOR_A_PERSON.key(),
            "das fragt ein Assistent während eines Zuges, es ist keine Antwort eines Menschen",
        );
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(protocol_words().unwrap());
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);

    let agent_approving = FromAnAgent::read(r#"{"format":1,"asks":{"approve":{"number":7}}}"#)
        .unwrap_err()
        .said(&strings);
    assert!(agent_approving.is_translated());
    assert!(agent_approving.text().starts_with("ein Assistent"));

    let person_reading = FromAPerson::read(r#"{"format":1,"asks":{"ask":{"question":"a"}}}"#)
        .unwrap_err()
        .said(&strings);
    assert!(person_reading.is_translated());
    assert!(person_reading.text().starts_with("das fragt"));
}

/// **A translation that is only half done says so**, which is `alo-strings`'
/// rule and the reason a release note can count what is left: five of these
/// seven are still English on a German machine, and every one of them says it.
#[test]
fn what_nobody_translated_yet_is_countable() {
    let vocabulary = protocol_words().unwrap();
    let translation = Translation::into_language(german()).says(
        words::NOT_READABLE.key(),
        "dieser Rechner konnte diese Nachricht nicht lesen",
    );
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(protocol_words().unwrap());
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);

    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() - 1);
}

/// This crate's list goes into a vocabulary beside every other crate's without
/// a collision, which is what a machine really loads.
#[test]
fn this_list_sits_beside_every_other_crates_list() {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    let before = vocabulary.how_many();

    alo_protocol::declare_into(&mut vocabulary).unwrap();
    assert_eq!(vocabulary.how_many(), before + EVERY_WORD.len());

    let strings = Strings::of(vocabulary);
    let said = FromAnAgent::read(r#"{"format":1,"asks":{"approve":{"number":7}}}"#)
        .unwrap_err()
        .said(&strings);
    assert!(!said.is_a_bug(), "{said}");
}
