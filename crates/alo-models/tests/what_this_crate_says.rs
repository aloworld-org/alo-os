//! What this crate says about models, providers and keys, in a language
//! somebody actually reads.
//!
//! The unit tests beside each file ask whether one sentence says the right
//! thing. This one asks the question the crate exists to answer from outside:
//! **can somebody be told where their question is about to go, and why it will
//! not go there, in their own language — and does anything say so when nobody
//! has translated it?**
//!
//! German, because most of this is sentences rather than labels and German
//! moves the verb; a translation that read like English with the words swapped
//! would not be exercising anything. The German here is the test's, not a
//! translation this repository ships: there are still zero of those.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::{
    InferenceSource, NoAgentHere, NotTried, Provider, ProviderError, Region, RuntimeError, Secret,
    SecretError, SourcePolicy, Weights, declare_into, model_words, words,
};
use alo_strings::{Key, Language, Strings, Translation, Vocabulary};

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// This crate's words, with these said in German and German preferred.
fn speaking_german(said: &[(alo_strings::Word, &str)]) -> Strings {
    let vocabulary = model_words().unwrap();
    let mut translation = Translation::into_language(german());
    for (word, says) in said {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);
    strings
}

/// A provider that has not said where it runs — the one somebody has to make a
/// decision about.
fn somewhere() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "someone".to_owned(),
        region: Region::Unknown,
    }
}

/// **Every string this crate can say declares**, and it declares into a
/// vocabulary that already holds somebody else's.
///
/// That is the arrangement on a real machine: one vocabulary, one area per
/// crate, every crate adding its own.
#[test]
fn everything_this_crate_says_declares_beside_everybody_elses() {
    let mut vocabulary = Vocabulary::empty();
    vocabulary
        .says(
            alo_strings::Phrase::says(Key::named("files.example").unwrap(), "something else")
                .unwrap(),
        )
        .unwrap();
    declare_into(&mut vocabulary).unwrap();

    assert_eq!(vocabulary.how_many(), words::EVERY_WORD.len() + 1);
    for word in words::EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.named());
    }
    // Nothing here counts, so nothing here is a plural (item 10's decision,
    // asserted from outside the crate that made it).
    assert_eq!(vocabulary.counted().count(), 0);
}

/// **The sentence somebody reads before they paste a contract into a question**
/// is theirs to read, and the parts of it that are not language come through as
/// they were written.
///
/// This is the string that has to survive translation intact: a person deciding
/// whether to send a document somewhere is entitled to the uncomfortable
/// version, in their own words.
#[test]
fn where_an_answer_would_come_from_is_said_in_the_language_the_person_reads() {
    let strings = speaking_german(&[
        (
            words::BY_A_PROVIDER_SOMEWHERE,
            "von {provider}, der nicht gesagt hat, wo er läuft",
        ),
        (words::ON_THIS_MACHINE, "auf diesem Rechner"),
    ]);

    let said = somewhere().shown(&strings);
    assert!(said.starts_with("von someone"), "{said}");
    assert!(said.contains("nicht gesagt hat"), "{said}");
    assert_eq!(
        InferenceSource::ThisMachine.shown(&strings),
        "auf diesem Rechner"
    );
}

/// **The whole crate can say no in German**: the rule an organisation set, the
/// provider somebody was adding, the key they pasted, and the test that never
/// happened because of the rule.
#[test]
fn a_person_is_told_no_in_the_language_they_read() {
    let strings = speaking_german(&[
        (
            words::OUTSIDE_THE_REGION,
            "dieser Rechner ist auf Inferenz nur in {region} eingestellt, und {source} erfüllt \
             das nicht",
        ),
        (
            words::BY_A_PROVIDER_SOMEWHERE,
            "von {provider}, der nicht gesagt hat, wo er läuft",
        ),
        (
            words::INSECURE_ENDPOINT,
            "diese Adresse ist nicht https, daher würden Ihr Schlüssel und Ihre Fragen \
             unverschlüsselt übertragen — nutzen Sie https oder einen Dienst auf diesem Rechner",
        ),
        (
            words::KEY_NOT_SENDABLE,
            "dieser Schlüssel enthält etwas, das nicht gesendet werden kann — kopieren Sie ihn \
             erneut, ohne die Zeile darum",
        ),
    ]);

    // The rule, and the place it refused, in one language.
    let refusal = SourcePolicy::InRegion("the EU".to_owned())
        .refusal(&somewhere())
        .unwrap();
    let said = refusal.said(&strings);
    assert!(said.is_translated());
    assert!(said.text().contains("nur in the EU"), "{said}");
    assert!(said.text().contains("nicht gesagt hat"), "{said}");
    assert!(!said.text().contains("has not said"), "{said}");

    // A test that never happened carries that refusal whole, so it says the
    // same thing in the same language rather than a summary of it.
    let not_tried = NotTried::Forbidden(refusal.clone());
    assert_eq!(not_tried.said(&strings).text(), said.text());

    // The provider somebody was adding.
    let provider = Provider::checked("Somewhere", "http://api.example.com", Region::Unknown, None)
        .unwrap_err();
    assert_eq!(provider, ProviderError::InsecureEndpoint);
    assert!(
        provider.said(&strings).text().contains("unverschlüsselt"),
        "{provider:?}"
    );

    // The key they pasted the line around.
    let key = Secret::typed("sk-live\r\nx-something: else").unwrap_err();
    assert_eq!(key, SecretError::NotSendable);
    assert!(key.said(&strings).text().starts_with("dieser Schlüssel"));
}

/// **Somebody who brings their own weights is told two things in their own
/// language, and neither of them is a refusal.**
///
/// This is the promise `docs/features.md` makes about hardware somebody owns —
/// *the machine warns and then gets out of the way* — asked from outside the
/// crate that keeps it. A German reader gets the warning about their machine's
/// memory and the line saying the licence is theirs, and the model is on the
/// list either way.
#[test]
fn weights_somebody_brought_are_costed_and_the_licence_stays_theirs() {
    let strings = speaking_german(&[
        (
            words::WEIGHTS_LARGER_THAN_MEMORY,
            "diese Gewichte sind größer als der Speicher dieses Rechners — alo OS führt sie \
             trotzdem aus, und dieser Rechner wird langsam sein",
        ),
        (
            words::LICENCE_IS_YOURS,
            "diese Gewichte gehören Ihnen, und ihre Bedingungen auch — alo OS hat die Lizenz \
             eines Modells nicht gelesen, das es Ihnen nicht angeboten hat",
        ),
    ]);

    // Forty gigabytes of somebody's own weights on a sixteen gigabyte machine.
    let theirs = Weights::checked("their-own-70b", 40_000_000_000).unwrap();
    let [cost, licence] = theirs.lines(&strings, 16.0);
    assert!(cost.is_translated());
    assert!(cost.text().contains("trotzdem"), "{cost}");
    assert!(licence.is_translated());
    assert!(licence.text().contains("nicht gelesen"), "{licence}");

    // Neither line counts anything out loud, and both numbers are still
    // available to whoever writes them the way this region writes a size.
    for line in [&cost, &licence] {
        assert!(!line.text().chars().any(|c| c.is_ascii_digit()), "{line}");
    }
    assert!(theirs.costs_on(16.0).larger_than_memory());
    assert_eq!(theirs.costs_on(16.0).machine_gb(), 16.0);

    // And nothing anywhere in this refused it: the weights are what they were.
    assert_eq!(theirs.bytes_on_disk, 40_000_000_000);
}

/// **A machine whose catalogue has nothing for the agent names all three
/// answers, in the reader's language, and picks none of them.**
///
/// The unit tests ask whether each sentence is right. This one asks the thing
/// the crate exists to get right from outside: a German reader who has just been
/// told that no model here can be given the agent is shown *why*, then that the
/// catalogue is not the only list, then the two places ADR 0008 leaves open —
/// three lines, in that order, with nothing choosing between them.
#[test]
fn a_machine_with_no_model_for_the_agent_names_every_answer_and_takes_none() {
    let strings = speaking_german(&[
        (
            words::NONE_MEASURED,
            "kein Modell, das auf diesem Rechner läuft, wurde daraufhin gemessen, ob es die Verben \
             bedienen kann",
        ),
        (
            words::WEIGHTS_YOU_ALREADY_HAVE,
            "Sie können alo OS auf Gewichte richten, die Sie bereits auf diesem Rechner haben, und \
             es führt sie aus",
        ),
        (
            words::THE_OTHER_PLACES,
            "Sie können ein Modell auf einem gekoppelten Rechner in Ihrem Netz verwenden oder \
             einen Anbieter, den Sie hinzufügen",
        ),
    ]);

    let refused = NoAgentHere::NoneMeasured { to_choose_from: 2 };
    let [why, brought, elsewhere] = refused.lines(&strings);
    for line in [&why, &brought, &elsewhere] {
        assert!(line.is_translated(), "{line}");
    }
    assert!(why.text().contains("gemessen"), "{why}");
    assert!(brought.text().contains("bereits"), "{brought}");
    assert!(elsewhere.text().contains("Anbieter"), "{elsewhere}");

    // The order is outward from the machine, and it survives translation: what
    // stays here is above what leaves, because that is decided by the code and
    // not by whichever language is loaded.
    assert!(brought.text().contains("diesem Rechner"), "{brought}");
    assert!(elsewhere.text().contains("Netz"), "{elsewhere}");

    // Neither line counts anything out loud. How many models there were to
    // choose from is a number beside them, in the reader's own way of writing
    // one.
    for line in [&why, &brought, &elsewhere] {
        assert!(!line.text().chars().any(|c| c.is_ascii_digit()), "{line}");
    }
    assert_eq!(refused.to_choose_from(), 2);
    assert_eq!(refused.measured(), 0);
}

/// **A refusal is never shown with only some of its answers, even when only some
/// of them are translated.** A machine part-way through a language would
/// otherwise show a German refusal above two English alternatives with nothing
/// saying which was which, and *shown English because nobody translated it* is
/// exactly what `alo-strings` exists to keep visible.
#[test]
fn a_half_translated_refusal_still_carries_all_three_lines_and_marks_them() {
    let strings = speaking_german(&[(
        words::WEIGHTS_YOU_ALREADY_HAVE,
        "Sie können alo OS auf Gewichte richten, die Sie bereits haben",
    )]);

    let [why, brought, elsewhere] = NoAgentHere::NothingToChooseFrom.lines(&strings);
    assert!(!why.is_translated());
    assert!(brought.is_translated());
    assert!(!elsewhere.is_translated());
    assert!(!why.text().is_empty());
    assert!(!elsewhere.text().is_empty());
}

/// **What nobody has translated is English, and says so.** Half a vocabulary is
/// the ordinary state of a language somebody is still working on, and a machine
/// that could not tell the difference would be one where *shown English because
/// nobody translated it* is invisible.
#[test]
fn what_nobody_translated_is_english_and_says_so() {
    let strings = speaking_german(&[(words::ON_THIS_MACHINE, "auf diesem Rechner")]);

    assert!(
        strings
            .say(
                &words::ON_THIS_MACHINE.key(),
                &alo_strings::Filling::nothing()
            )
            .is_translated()
    );

    let untranslated = RuntimeError::Unreachable.said(&strings);
    assert!(!untranslated.is_translated());
    assert!(!untranslated.is_a_bug());
    assert_eq!(untranslated.text(), "the model runtime is not reachable");

    // And what is left to do is countable, which is what a release note needs.
    let left = strings.missing_from(&german());
    assert_eq!(left.len(), words::EVERY_WORD.len() - 1);
    assert!(left.contains(&words::RUNTIME_UNREACHABLE.key()));
    assert!(!left.contains(&words::ON_THIS_MACHINE.key()));
}

/// **A refusal never depends on a string table.** With no words at all, the
/// policy refuses exactly what it refused before, the provider is still not
/// added, and every answer names its rule by key and says it is a bug — so a
/// shell that forgot to declare this crate's words finds out, and nothing is
/// quietly permitted.
#[test]
fn a_refusal_without_the_words_still_refuses_and_says_the_rule() {
    let nothing = Strings::of(Vocabulary::empty());

    let refusal = SourcePolicy::ThisMachineOnly.refusal(&somewhere());
    assert!(refusal.is_some());
    let said = refusal.unwrap().said(&nothing);
    assert!(said.is_a_bug());
    assert!(
        said.text().contains("models.policy.not-this-machine"),
        "{said}"
    );

    assert!(
        Provider::checked("  ", "https://api.example.com", Region::Unknown, None)
            .unwrap_err()
            .said(&nothing)
            .is_a_bug()
    );
    assert!(Secret::typed("   ").unwrap_err().said(&nothing).is_a_bug());
    assert!(NotTried::KeyNotAccepted.said(&nothing).is_a_bug());
}

/// **A translation that drops what was refused is refused.**
///
/// A policy refusal that lost `{source}` would tell somebody in their own
/// language that this machine has a rule, without saying what it stopped —
/// which is worse than the English, because nothing anywhere would say it had
/// happened.
#[test]
fn a_translation_that_drops_where_the_question_would_have_gone_is_refused() {
    let vocabulary = model_words().unwrap();
    let wrongs = vocabulary
        .check(
            Translation::into_language(german())
                .says(
                    words::OUTSIDE_THE_BUILDING.key(),
                    "dieser Rechner behält Fragen im Haus",
                )
                .says(
                    words::MODEL_NOT_INSTALLED.key(),
                    "{model} ist nicht installiert",
                ),
        )
        .unwrap_err();
    assert_eq!(wrongs.how_many(), 1);
    assert!(wrongs.to_string().contains("source"), "{wrongs}");
}

/// **A key never reaches a sentence, in any language.** Neither of the two
/// strings about a key has a gap, so a translation cannot invent one to put a
/// credential in — `alo-strings` refuses a gap the source does not have, and
/// this is that guarantee asserted where it matters most.
#[test]
fn a_translation_cannot_make_a_place_for_a_key_to_appear() {
    let vocabulary = model_words().unwrap();
    let wrongs = vocabulary
        .check(
            Translation::into_language(german())
                .says(words::KEY_BLANK.key(), "fügen Sie den Schlüssel {key} ein"),
        )
        .unwrap_err();
    assert_eq!(wrongs.how_many(), 1);
    assert!(wrongs.to_string().contains("key"), "{wrongs}");
}
