//! What this crate says while something is leaving the machine, in a language
//! somebody actually reads.
//!
//! The unit tests beside each file ask whether one sentence says the right
//! thing. This one asks the question the crate exists to answer from outside:
//! **is the line law 1 promises — the one that says something is leaving right
//! now — readable by the person the machine belongs to, and does anything say
//! so when nobody has translated it?**
//!
//! German, because these are sentences rather than labels and German moves the
//! verb; a translation that read like English with the words swapped would not
//! be exercising anything. The German here is the test's, not a translation
//! this repository ships: there are still zero of those.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::Grantee;
use alo_egress::{
    Destination, DestinationError, EgressPolicy, Errand, Indicator, Leaving, OnItsOwn, Why,
    declare_into, egress_words, words,
};
use alo_models::{InferenceSource, Region};
use alo_strings::{Key, Language, Strings, Translation, Vocabulary};
use std::time::{Duration, SystemTime};

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// This crate's words, with these said in German and German preferred.
fn speaking_german(said: &[(alo_strings::Word, &str)]) -> Strings {
    let vocabulary = egress_words().unwrap();
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

/// Noon, because nothing here reads the clock.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

/// **Every string this crate can say declares**, and it declares into a
/// vocabulary that already holds somebody else's.
///
/// That is the arrangement on a real machine: one vocabulary, one area per
/// crate, every crate adding its own.
#[test]
fn everything_this_crate_says_declares_beside_everybody_elses() {
    let mut vocabulary = alo_models::model_words().unwrap();
    let theirs = vocabulary.how_many();
    declare_into(&mut vocabulary).unwrap();

    assert_eq!(vocabulary.how_many(), theirs + words::EVERY_WORD.len());
    for word in words::EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.named());
    }
    // Nothing here counts, so nothing here is a plural.
    assert_eq!(vocabulary.counted().count(), 0);
    // And every key of this crate's is a key, checked from outside the crate
    // that wrote it.
    for word in words::EVERY_WORD {
        assert_eq!(Key::named(word.named()), Ok(word.key()));
    }
}

/// **The sentence law 1 exists to put in front of somebody**, in the language
/// they read — whole, with the place named inside it in the same language, and
/// with the agent's name left alone.
///
/// This is the string in this repository it would be least acceptable to leave
/// untranslated: it is the entire visible half of *nothing leaves silently*.
#[test]
fn the_indicator_line_is_read_in_the_language_the_person_reads() {
    let strings = speaking_german(&[
        (words::IS_ASKING, "{agent} stellt {destination} eine Frage"),
        (
            words::A_PROVIDER_SOMEWHERE,
            "{provider}, der nicht gesagt hat, wo er läuft",
        ),
    ]);

    let mut indicator = Indicator::default();
    let departing = indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::asking(&Grantee::named("@mail"), &somewhere()).unwrap(),
            noon(),
        )
        .unwrap();

    let said = indicator.showing().first().unwrap().said(&strings);
    assert!(said.is_translated());
    assert_eq!(
        said.text(),
        "@mail stellt someone, der nicht gesagt hat, wo er läuft eine Frage"
    );
    assert!(!said.text().contains("is asking"), "{said}");

    indicator.ended(departing);
    assert!(indicator.is_quiet());
}

/// **Where an answer came from and where a thing is going say the same thing
/// about the same provider.**
///
/// `Destination::of` maps an `InferenceSource` onto a destination in one place,
/// so the two crates describe one provider twice: `alo-models` as provenance
/// — *by someone, which has not said where it runs* — and this crate as a
/// place. They are different sentences on purpose, because they sit in
/// different grammatical positions. What they must never do is differ about the
/// **provider**: one of them quietly dropping *has not said where it runs*
/// would leave a person reading a reassuring line about a question that is
/// about to leave the continent.
#[test]
fn where_an_answer_came_from_and_where_it_is_going_agree_about_the_provider() {
    let mut vocabulary = alo_models::model_words().unwrap();
    declare_into(&mut vocabulary).unwrap();
    let strings = Strings::of(vocabulary);

    for (source, must_say) in [
        (somewhere(), vec!["someone", "has not said where it runs"]),
        (
            InferenceSource::Hosted {
                provider: "alo".to_owned(),
                region: Region::Declared("the EU".to_owned()),
            },
            vec!["alo", "the EU"],
        ),
        (
            InferenceSource::PairedMachine {
                machine: "the studio workstation".to_owned(),
            },
            vec!["the studio workstation", "on your network"],
        ),
    ] {
        let came_from = source.shown(&strings);
        let going_to = Destination::of(&source).unwrap().shown(&strings);
        for said in must_say {
            assert!(came_from.contains(said), "{came_from}");
            assert!(going_to.contains(said), "{going_to}");
        }
    }

    // A question answered here is not a destination at all, so there is no
    // second sentence to disagree with the first.
    assert_eq!(
        Destination::of(&InferenceSource::ThisMachine).unwrap_err(),
        DestinationError::NothingLeaves
    );
}

/// **The whole crate can say no in German**: the rule an organisation set, the
/// place it refused, and the address an agent named that could not be shown.
#[test]
fn a_person_is_told_what_was_stopped_in_the_language_they_read() {
    let strings = speaking_german(&[
        (
            words::OUTSIDE_THE_REGION,
            "dieser Rechner darf nur {region} erreichen, und {destination} erfüllt das nicht",
        ),
        (
            words::A_PROVIDER_SOMEWHERE,
            "{provider}, der nicht gesagt hat, wo er läuft",
        ),
        (
            words::NOT_SHOWABLE,
            "die Adresse enthält ein Zeichen, das nicht angezeigt werden kann — die Anzeige muss \
             in einer Zeile lesbar sein",
        ),
    ]);

    let leaving = Leaving::asking(&Grantee::named("@mail"), &somewhere()).unwrap();
    let refused = EgressPolicy::InRegion("the EU".to_owned())
        .refusal(&leaving)
        .unwrap();
    let said = refused.said(&strings);
    assert!(said.is_translated());
    assert!(said.text().contains("nur the EU erreichen"), "{said}");
    assert!(said.text().contains("nicht gesagt hat"), "{said}");
    assert!(!said.text().contains("does not meet"), "{said}");

    let unshowable = Destination::at("alo.example\u{1b}[2K").unwrap_err();
    assert_eq!(unshowable, DestinationError::NotPrintable);
    assert!(
        unshowable
            .said(&strings)
            .text()
            .starts_with("die Adresse enthält"),
        "{unshowable:?}"
    );
}

/// **What nobody has translated is English, and says so.** Half a vocabulary is
/// the ordinary state of a language somebody is still working on, and a machine
/// that could not tell the difference would be one where *shown English because
/// nobody translated it* is invisible.
#[test]
fn what_nobody_translated_is_english_and_says_so() {
    let strings = speaking_german(&[(words::IS_ASKING, "{agent} stellt {destination} eine Frage")]);

    let fetching = Leaving::because(
        &Grantee::named("@files"),
        Why::Fetching,
        Destination::at("alo.example").unwrap(),
    )
    .said(&strings);
    assert!(!fetching.is_translated());
    assert!(!fetching.is_a_bug());
    assert_eq!(
        fetching.text(),
        "@files is fetching something from alo.example"
    );

    // And what is left to do is countable, which is what a release note needs.
    let left = strings.missing_from(&german());
    assert_eq!(left.len(), words::EVERY_WORD.len() - 1);
    assert!(left.contains(&words::IS_FETCHING.key()));
    assert!(!left.contains(&words::IS_ASKING.key()));
}

/// **Nothing about what may leave depends on a string table.** With no words at
/// all the policy refuses exactly what it refused before, an address that could
/// rewrite the indicator is still refused, and every answer names its key and
/// says it is a bug — so a shell that forgot to declare this crate's words finds
/// out, and nothing is quietly permitted.
#[test]
fn the_policy_decides_the_same_with_no_words_at_all() {
    let nothing = Strings::of(Vocabulary::empty());
    let leaving = Leaving::asking(&Grantee::named("@mail"), &somewhere()).unwrap();

    let mut indicator = Indicator::default();
    let refused = indicator
        .beginning(&EgressPolicy::InTheBuilding, leaving, noon())
        .unwrap_err();
    assert!(indicator.is_quiet());
    let said = refused.said(&nothing);
    assert!(said.is_a_bug());
    assert!(
        said.text().contains("egress.policy.outside-the-building"),
        "{said}"
    );

    assert_eq!(
        Destination::at("alo.example\nand nothing is leaving").unwrap_err(),
        DestinationError::NotPrintable
    );
    assert!(
        DestinationError::NotPrintable
            .said(&nothing)
            .text()
            .contains("egress.destination.not-printable")
    );
}

/// **A translation that drops where something is going is refused.**
///
/// An indicator line that lost `{destination}` would tell a person in their own
/// language that something is leaving, without saying where — which is worse
/// than the English, because it would look like the feature working.
#[test]
fn a_translation_that_drops_where_something_is_going_is_refused() {
    let vocabulary = egress_words().unwrap();
    let wrongs = vocabulary
        .check(
            Translation::into_language(german())
                .says(words::IS_SENDING.key(), "{agent} sendet etwas")
                .says(
                    words::OUTSIDE_THE_BUILDING.key(),
                    "dieser Rechner behält alles im Haus",
                ),
        )
        .unwrap_err();
    assert_eq!(wrongs.how_many(), 2);
    assert!(wrongs.to_string().contains("destination"), "{wrongs}");
}

/// **★ No telemetry, read from outside the crate and in a language that is not
/// English.** What alo OS does with nobody having asked it to is on the same
/// indicator as what an agent does, said in the reader's own words, and the
/// promise beside the list is one of those words rather than a sentence in a
/// document.
#[test]
fn what_the_machine_does_on_its_own_is_read_in_the_language_the_person_reads() {
    let strings = speaking_german(&[
        (
            words::ALO_IS_FETCHING_A_MODEL,
            "alo OS holt ein Modell von {destination}",
        ),
        (words::IS_ASKING, "{agent} stellt {destination} eine Frage"),
        (
            words::A_PROVIDER_SOMEWHERE,
            "{provider}, der nicht gesagt hat, wo er läuft",
        ),
    ]);

    let mut indicator = Indicator::default();
    let departing = indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::asking(&Grantee::named("@mail"), &somewhere()).unwrap(),
            noon(),
        )
        .unwrap();
    let underway = indicator.beginning_on_its_own(
        OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::at("models.alo.example").unwrap(),
        ),
        noon(),
    );

    // One list, two kinds of line, one language.
    let lines: Vec<String> = indicator
        .showing()
        .iter()
        .map(|shown| shown.said(&strings).text().to_owned())
        .collect();
    assert_eq!(lines.len(), 2);
    assert!(
        lines.iter().all(|line| !line.contains("is asking")),
        "{lines:?}"
    );
    assert_eq!(
        lines.last().map(String::as_str),
        Some("alo OS holt ein Modell von models.alo.example")
    );
    assert!(
        indicator
            .showing()
            .iter()
            .all(|shown| shown.said(&strings).is_translated()),
        "{lines:?}"
    );

    // And the line about the machine's own errand names nobody, because
    // nobody asked for it.
    assert_eq!(
        indicator
            .showing()
            .last()
            .and_then(|shown| shown.showing().agent()),
        None
    );

    indicator.ended(departing);
    indicator.ended_on_its_own(underway);
    assert!(indicator.is_quiet());
}

/// **The promise itself is a string a person reads**, and it is worth what it
/// is worth in the language they read it in.
///
/// Greek, because *no telemetry* is the sentence a sovereignty product is
/// bought on and the member states this repository exists for do not read
/// English by default. A promise a person cannot read is a promise made to
/// somebody else.
#[test]
fn the_no_telemetry_promise_is_read_by_somebody_who_does_not_read_english() {
    let greek = Language::written("el").unwrap();
    let vocabulary = egress_words().unwrap();
    let speaking = vocabulary
        .check(Translation::into_language(greek.clone()).says(
            words::ALO_REACHES_NOTHING_ELSE.key(),
            "το alo OS συνδέεται στο δίκτυο για αυτούς τους λόγους και για κανέναν άλλον, και ποτέ \
             για να πει κάτι για το πώς χρησιμοποιείτε αυτόν τον υπολογιστή",
        ))
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[greek]);

    let said = Errand::nothing_else(&strings);
    assert!(said.is_translated(), "{said}");
    assert!(said.text().contains("κανέναν άλλον"), "{said}");
    assert!(!said.text().contains("no others"), "{said}");

    // Every reason on the list has a line of its own, so the promise is about
    // something a person can also check one line at a time.
    assert_eq!(Errand::EVERY.len(), 3);
    for errand in Errand::EVERY {
        assert!(
            words::EVERY_WORD.contains(&errand.word()),
            "{}",
            errand.word().named()
        );
    }
}
