//! What this crate says, against the vocabulary the code actually uses.
//!
//! The unit tests read the list; this reads what a person would see, in a
//! language that is not the one the code is written in. Polish, because the
//! one countable sentence here — *counted together with N other processes* —
//! has three forms for whole numbers in Polish where English has two, and a
//! sentence assembled out of English pieces would show it at five.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_measuring::{Gone, Network, NotMeasured, Number, Source, measuring_words};
use alo_strings::{Form, Key, Language, Said, Strings, Translation};

/// A key, from this crate's list.
fn key(named: &str) -> Key {
    Key::named(named).expect("a key")
}

/// Polish, and this crate's ten strings in it.
fn in_polish() -> Strings {
    let vocabulary = measuring_words().expect("this crate's own words");
    let polish = Language::written("pl").expect("a language");
    let shared = key("measuring.network.shared");
    let translation = Translation::into_language(polish.clone())
        .says(
            key("measuring.not-on-this-host"),
            "Co jest uruchomione, można odczytać tylko na samym alo OS.",
        )
        .says(
            key("measuring.unreadable"),
            "Nie udało się odczytać, co jest uruchomione, z {at}.",
        )
        .says(
            key("measuring.no-interval"),
            "Dwa odczyty wymagają czasu między nimi.",
        )
        .says(
            key("measuring.same-moment"),
            "Oba odczyty pochodzą z tej samej chwili, więc nie można nic powiedzieć o tempie.",
        )
        .says(
            key("measuring.process.gone"),
            "Zakończony od ostatniego odczytu.",
        )
        .says(
            key("measuring.number.not-yet"),
            "Uruchomiony od ostatniego odczytu.",
        )
        .says(key("measuring.number.withheld"), "Jądro tego nie pokazało.")
        .says(
            key("measuring.number.not-said"),
            "Nie liczone dla tego procesu.",
        )
        .says(
            key("measuring.network.this-process-alone"),
            "Ruch sieciowy liczony tylko dla tego procesu.",
        )
        .says(
            shared.for_form(Form::One),
            "Ruch sieciowy liczony razem z jednym innym procesem.",
        )
        .says(
            shared.for_form(Form::Few),
            "Ruch sieciowy liczony razem z {others} innymi procesami.",
        )
        .says(
            shared.for_form(Form::Many),
            "Ruch sieciowy liczony razem z {others} innymi procesami.",
        );
    let speaking = vocabulary
        .check(translation)
        .expect("a translation of ten strings with no gaps in it");
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).expect("a checked translation");
    strings.prefers(&[polish]);
    strings
}

/// A source for these tests.
fn from() -> Source {
    Source::of("/proc/42/io", "rchar")
}

/// Every sentence this crate says itself, rendered against these strings.
fn everything_said_here(strings: &Strings) -> Vec<Said> {
    let mut said = vec![
        NotMeasured::NotOnThisHost.said(strings),
        NotMeasured::Unreadable {
            at: PathBuf::from("/proc/stat"),
            why: "no such file".to_owned(),
        }
        .said(strings),
        NotMeasured::NoInterval.said(strings),
        NotMeasured::SameMoment.said(strings),
        Gone {
            pid: 42,
            name: "cat".to_owned(),
        }
        .said(strings),
        Network {
            namespace: Some(1),
            shared_with: 0,
        }
        .said(strings),
        Network {
            namespace: Some(1),
            shared_with: 1,
        }
        .said(strings),
        Network {
            namespace: Some(1),
            shared_with: 3,
        }
        .said(strings),
        Network {
            namespace: Some(1),
            shared_with: 5,
        }
        .said(strings),
    ];
    for number in [
        Number::Withheld {
            from: from(),
            why: String::new(),
        },
        Number::NotSaid { from: from() },
        Number::NotYet { from: from() },
    ] {
        said.push(
            number
                .instead(strings)
                .expect("a sentence in place of a number"),
        );
    }
    said
}

/// A machine with no translations shows the English, and says so about
/// itself; nothing is a key on somebody's screen and nothing has a gap.
#[test]
fn a_machine_with_no_translations_still_says_everything_in_english() {
    let strings = Strings::of(measuring_words().expect("this crate's own words"));
    let said = everything_said_here(&strings);
    assert_eq!(said.len(), 12);
    for said in said {
        assert!(!said.is_a_bug(), "{said}");
        assert!(!said.is_translated(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.text().starts_with("measuring."), "{said}");
    }
}

/// And a machine that has them shows those, whole — with the Polish forms
/// chosen by Polish rules: one, a few, and many.
#[test]
fn every_sentence_is_read_in_the_language_the_person_reads() {
    let strings = in_polish();
    for said in everything_said_here(&strings) {
        assert!(said.is_translated(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
    }

    let unreadable = NotMeasured::Unreadable {
        at: PathBuf::from("/proc/stat"),
        why: String::new(),
    }
    .said(&strings);
    assert_eq!(
        unreadable.text(),
        "Nie udało się odczytać, co jest uruchomione, z /proc/stat."
    );

    let one = Network {
        namespace: None,
        shared_with: 1,
    }
    .said(&strings);
    assert_eq!(
        one.text(),
        "Ruch sieciowy liczony razem z jednym innym procesem."
    );
    let few = Network {
        namespace: None,
        shared_with: 3,
    }
    .said(&strings);
    assert_eq!(
        few.text(),
        "Ruch sieciowy liczony razem z 3 innymi procesami."
    );
    let many = Network {
        namespace: None,
        shared_with: 22,
    }
    .said(&strings);
    assert_eq!(
        many.text(),
        "Ruch sieciowy liczony razem z 22 innymi procesami."
    );
}
