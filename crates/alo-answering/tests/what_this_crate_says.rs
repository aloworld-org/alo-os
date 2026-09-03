//! Everything this crate can say, put through the whole path a translation
//! takes: declared, checked, partly translated, and shown.
//!
//! The crate's own tests take one sentence at a time. This is the other half:
//! the real vocabulary — not a fixture that resembles it — walked in languages
//! that make the exercise worth doing. Greek is written in an alphabet where a
//! key left untranslated is unmistakable rather than merely odd; Estonian is
//! one of the official languages a product selling "English plus the big five"
//! would have skipped.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been read by anybody: there is no screen, and there are still no
//! translations in this repository.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_answering::words::{self, EVERY_WORD};
use alo_answering::{Answering, WentWrong, answering_words};
use alo_models::{InferenceSource, Region, SourcePolicy};
use alo_strings::{
    CameFrom, Filling, Key, Language, Phrase, Showing, Strings, Translation, Vocabulary,
};

/// One of the tests' languages.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// A provider that has not said where it runs — the one ADR 0008 says must
/// never be made to look like one that has.
fn somewhere() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "someone".to_owned(),
        region: Region::Unknown,
    }
}

/// This crate's words beside `alo-models`', which every sentence here needs
/// because every sentence here names a place.
fn everything() -> Vocabulary {
    let mut vocabulary = answering_words().unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// **Every string this crate can say declares**, and it declares into a
/// vocabulary that already holds somebody else's.
#[test]
fn everything_this_crate_says_joins_one_vocabulary_beside_another_crates() {
    let mut vocabulary = Vocabulary::empty();
    vocabulary
        .says(
            Phrase::says(
                Key::named("files.verb.read-file.purpose").unwrap(),
                "read what is in a file",
            )
            .unwrap(),
        )
        .unwrap();
    words::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();

    for word in EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.key());
    }
}

/// A machine with no translations at all is the machine this repository ships
/// today, and on it every one of these keys still answers with the sentence the
/// code declared rather than with the key.
#[test]
fn with_no_translations_at_all_every_sentence_is_still_a_sentence() {
    let strings = Strings::of(answering_words().unwrap());
    for word in EVERY_WORD {
        let said = strings.say(&word.key(), &Filling::nothing());
        assert_eq!(said.came_from(), &CameFrom::TheSource, "{}", word.key());
        assert_eq!(said.text(), word.says(), "{}", word.key());
    }
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len());
}

/// **The sentence a person approves, translated, with the place still in it.**
/// `alo_strings::Vocabulary::check` refuses a translation that drops a gap the
/// source has, so an offer that stopped saying where the question would go
/// would never load at all.
#[test]
fn an_offer_cannot_be_translated_into_one_that_does_not_say_where_it_goes() {
    let wrongs = answering_words()
        .unwrap()
        .check(Translation::into_language(language("el")).says(
            words::ASK_OUTSIDE_INSTEAD.key(),
            "στείλτε αυτήν την ερώτηση αλλού, μόνο αυτήν τη φορά",
        ))
        .unwrap_err();
    assert!(wrongs.to_string().contains("{source}"), "{wrongs}");
}

/// **The whole line is one language, or it says it is not.** An offer in Greek
/// naming a provider in English is half a sentence its reader can read, and
/// that is exactly what `Said::is_translated` exists to notice.
#[test]
fn an_offer_and_the_place_inside_it_are_one_language_or_the_line_says_otherwise() {
    let vocabulary = everything();
    let greek = vocabulary
        .check(
            Translation::into_language(language("el"))
                .says(
                    words::ASK_OUTSIDE_INSTEAD.key(),
                    "να απαντηθεί αυτή η ερώτηση {source}, μόνο αυτήν τη φορά — η ερώτηση θα \
                     έβγαινε από αυτόν τον υπολογιστή και από το κτίριο",
                )
                .says(
                    words::NOTHING_WAS_SENT.key(),
                    "δεν στάλθηκε τίποτα πουθενά, και δεν θα σταλεί αν δεν το πείτε εσείς",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(greek).unwrap();
    strings.prefers(&[language("el")]);

    let failed = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
        .unwrap()
        .did_not_answer(
            WentWrong::NothingAnswered,
            &[somewhere()],
            &SourcePolicy::Anywhere,
        )
        .unwrap();

    // The reassurance is wholly Greek: it has no gaps at all.
    let sent = failed.nothing_was_sent(&strings);
    assert!(sent.is_translated(), "{sent}");
    assert!(sent.text().starts_with("δεν στάλθηκε"), "{sent}");

    // The offer is Greek with an English clause in it, because nobody has
    // translated `alo-models`' description of a provider that will not say
    // where it runs — and the line says so rather than passing for Greek.
    let offer = failed.elsewhere().offers().first().cloned().unwrap();
    let said = offer.said(&strings);
    assert!(!said.is_translated(), "{said}");
    assert!(said.text().starts_with("να απαντηθεί"), "{said}");
    assert!(said.text().contains("has not said where it runs"), "{said}");

    // And the failure line is the same story: Greek nowhere, so nothing about
    // it claims to be Greek.
    let failure = failed.said(&strings);
    assert!(!failure.is_translated(), "{failure}");
}

/// **A half-translated crate says which half.** A shell being built in Estonian
/// can count what is left, and what reaches a person meanwhile is marked in
/// development rather than passed off as Estonian.
#[test]
fn what_nobody_has_translated_yet_is_visible_rather_than_silently_english() {
    let vocabulary = answering_words().unwrap();
    let estonian = vocabulary
        .check(
            Translation::into_language(language("et"))
                .says(
                    words::NOTHING_WAS_SENT.key(),
                    "kuhugi ei saadetud midagi ja ei saadeta enne, kui te seda ütlete",
                )
                .says(
                    words::NOWHERE_ELSE.key(),
                    "sellele küsimusele pole mujal vastajat seadistatud",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(estonian).unwrap();
    strings.prefers(&[language("et")]);

    assert_eq!(strings.unanswered().len(), EVERY_WORD.len() - 2);

    // One nobody has reached yet is marked, rather than looking like Estonian
    // somebody wrote.
    strings.shown(Showing::InDevelopment);
    let untranslated = strings.say(&words::NOT_ON_OFFER.key(), &Filling::nothing());
    assert!(untranslated.text().starts_with('«'), "{untranslated}");
    assert!(!untranslated.is_translated());
}

/// **The provider's name and the region it stated are never translated.** They
/// are somebody's own words about themselves, and a translation of either would
/// name a place that does not exist — `alo-files`' rule about a filename, at
/// the far end of the system.
#[test]
fn what_a_provider_calls_itself_is_not_the_languages_to_change() {
    let vocabulary = everything();
    let greek = vocabulary
        .check(
            Translation::into_language(language("el"))
                .says(
                    words::ASK_OUTSIDE_INSTEAD.key(),
                    "να απαντηθεί αυτή η ερώτηση {source}, μόνο αυτήν τη φορά — η ερώτηση θα \
                     έβγαινε από αυτόν τον υπολογιστή και από το κτίριο",
                )
                .says(
                    alo_models::words::BY_A_PROVIDER.key(),
                    "από {provider}, σε {region}",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(greek).unwrap();
    strings.prefers(&[language("el")]);

    let alo = InferenceSource::Hosted {
        provider: "alo".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    };
    let offer = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
        .unwrap()
        .did_not_answer(WentWrong::NothingUsable, &[alo], &SourcePolicy::Anywhere)
        .unwrap()
        .elsewhere()
        .offers()
        .first()
        .cloned()
        .unwrap();

    let said = offer.said(&strings);
    assert!(said.is_translated(), "{said}");
    assert!(said.text().contains("από alo"), "{said}");
    assert!(said.text().contains("the EU"), "{said}");
}
