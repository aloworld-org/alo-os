//! Everything this crate can say, read from outside it — against the
//! vocabulary the code actually uses rather than against a copy of it.
//!
//! The crate's own tests assert what each sentence means. These assert the
//! things only a caller can see: that the list declares into a shell's one
//! vocabulary, that a translator handed it can answer every string in it, and
//! that a translation which drops a gap is refused before anybody reads it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_keeping::words::{self, EVERY_WORD, FOR_DAYS};
use alo_keeping::{Damage, Keeping, NotKept, Reading, declare_into, keeping_words};
use alo_strings::cldr::{form_for, knows};
use alo_strings::{CameFrom, Form, Language, Said, Strings, Translation, Vocabulary};

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// Everything this crate says, with nothing translated.
fn in_english() -> Strings {
    Strings::of(keeping_words().unwrap())
}

/// A record read back from a string, for the sentences that are about one.
fn a_shortened_record() -> Reading {
    // Written by this crate at some point and shortened once.
    let head = "{\"format\":1,\"since\":{\"secs_since_epoch\":1760000000,\
                \"nanos_since_epoch\":0},\"under\":{\"for-days\":30}}\n";
    let folder = std::env::temp_dir().join(format!("alo-keeping-said-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("record.jsonl");
    std::fs::write(&path, head).unwrap();
    Reading::at(&path).unwrap()
}

/// **Every string this crate can say is in one list**, and the list goes into a
/// shell's one vocabulary beside every other crate's.
#[test]
fn everything_this_crate_says_declares_into_one_vocabulary() {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
    assert_eq!(vocabulary.counted().count(), 1);

    // And it sits beside another crate's without either being replaced.
    let mut shell = alo_capability::capability_words().unwrap();
    let before = shell.how_many();
    declare_into(&mut shell).unwrap();
    assert_eq!(shell.how_many(), before + EVERY_WORD.len() + 1);
}

/// **Nothing this crate says arrives as English by accident.** Every sentence
/// it can put in front of a person goes through the lookup, and every one of
/// them says where it came from.
#[test]
fn every_sentence_this_crate_can_show_comes_from_the_lookup() {
    let strings = in_english();
    let mut every_said = vec![
        Keeping::Forever.said(&strings),
        Keeping::for_days(30).unwrap().said(&strings),
        Keeping::for_days(0).unwrap_err().said(&strings),
        a_shortened_record().head().said(&strings),
    ];
    let mut damage = Damage::default();
    every_said.extend(damage.said(&strings));
    every_said.push(
        NotKept::NotThere {
            path: "/var/lib/alo/record.jsonl".to_owned(),
        }
        .said(&strings),
    );

    for said in &every_said {
        assert!(!said.is_a_bug(), "{said:?}");
        assert!(said.unfilled().is_empty(), "{said:?}");
    }
    assert!(!every_said.is_empty());
    damage = Damage::default();
    assert!(
        damage.said(&strings).is_empty(),
        "and nothing wrong says nothing"
    );
}

/// **A shell that never declared this crate's words shows keys**, marked, and
/// says it is a bug — rather than being handed English nobody offered to
/// translate.
#[test]
fn a_shell_that_forgot_to_declare_these_words_shows_that_it_forgot() {
    let strings = Strings::of(Vocabulary::empty());
    assert!(Keeping::Forever.said(&strings).is_a_bug());
    assert!(a_shortened_record().head().said(&strings).is_a_bug());
    assert!(
        NotKept::NotARecord {
            path: "/tmp/notes.txt".to_owned(),
        }
        .said(&strings)
        .is_a_bug()
    );
}

/// **The list is what a translator is handed, and what a release note counts.**
/// Nothing has been read by anybody yet, so every string in it is unanswered —
/// and the countable one is listed once, under its own name.
#[test]
fn everything_here_is_still_waiting_for_a_translator() {
    let strings = in_english();
    let waiting = strings.unanswered();
    assert_eq!(waiting.len(), EVERY_WORD.len() + 1);
    assert!(waiting.contains(&FOR_DAYS.key()));
    for word in EVERY_WORD {
        assert!(waiting.contains(&word.key()), "{}", word.named());
    }
}

/// **A record that has been shortened says so in the reader's own language**,
/// which is the sentence this crate exists to put in front of somebody: an
/// absence in a record must not be read as an innocence, in any language.
#[test]
fn the_sentence_about_a_shortened_record_is_read_in_the_readers_language() {
    let vocabulary = keeping_words().unwrap();
    let speaking = vocabulary
        .check(
            Translation::into_language(german())
                .says(
                    words::SHORTENED.key(),
                    "dieser Verlauf reicht nicht bis zum Anfang zurück — was davor geschah, ist \
                     verfallen und wurde entfernt",
                )
                .says(
                    words::WHOLE.key(),
                    "dies ist alles, was auf diesem Rechner geschehen ist — nichts wurde entfernt",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);

    let said = a_shortened_record().head().said(&strings);
    assert!(said.is_translated());
    assert_eq!(said.came_from(), &CameFrom::Translation(german()));
    assert!(said.text().starts_with("dieser Verlauf"), "{}", said.text());
}

/// **How long a record is kept is counted the reader's own way.** German has
/// two shapes as English does, and *one day* is not *30 days* — which a
/// sentence with the number stuck into it would have got wrong in every
/// language that does not share English's two.
#[test]
fn how_long_a_record_is_kept_is_counted_and_never_stuck_into_a_sentence() {
    let vocabulary = keeping_words().unwrap();
    let speaking = vocabulary
        .check(
            Translation::into_language(german())
                .says(
                    FOR_DAYS.key().for_form(Form::One),
                    "was der Agent dieses Rechners getan hat, wird einen Tag lang aufbewahrt und \
                     dann entfernt",
                )
                .says(
                    FOR_DAYS.key().for_form(Form::Other),
                    "was der Agent dieses Rechners getan hat, wird {days} Tage lang aufbewahrt und \
                     dann entfernt",
                ),
        )
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);

    let one = Keeping::for_days(1).unwrap().said(&strings);
    let thirty = Keeping::for_days(30).unwrap().said(&strings);
    assert_eq!(
        one.text(),
        "was der Agent dieses Rechners getan hat, wird einen Tag lang aufbewahrt und dann entfernt"
    );
    assert!(thirty.text().contains("30 Tage"), "{}", thirty.text());
    assert!(one.is_translated() && thirty.is_translated());

    // And the language's own rules pick the form, rather than English's.
    assert!(knows(&german()));
    assert_eq!(form_for(&german(), 1), Some(Form::One));
    assert_eq!(form_for(&german(), 30), Some(Form::Other));
}

/// **A translation that drops a gap is refused before anybody reads it.** A
/// record that could not be written, said in a language nobody here reads,
/// with the path taken out of it, would reach a person as a sentence about
/// nowhere.
#[test]
fn a_translation_that_takes_the_record_out_of_the_sentence_is_refused() {
    let vocabulary = keeping_words().unwrap();
    let wrong = vocabulary
        .check(Translation::into_language(german()).says(
            words::NOT_OPENED.key(),
            "der Verlauf konnte nicht geöffnet werden — {warum}",
        ))
        .unwrap_err();
    let complaints = format!("{wrong:?}");
    assert!(complaints.contains("path"), "{complaints}");
    assert!(complaints.contains("warum"), "{complaints}");
}

/// **Every word this crate declares is one a person could be shown**, and
/// nothing is declared that nothing says. A key nothing looks up is a row a
/// translator is asked to write for no reason.
#[test]
fn nothing_is_declared_that_this_crate_never_says() {
    let strings = in_english();
    let mut shown: Vec<String> = vec![
        Keeping::Forever.said(&strings).into_text(),
        Keeping::for_days(2).unwrap().said(&strings).into_text(),
        Keeping::for_days(0).unwrap_err().said(&strings).into_text(),
        a_shortened_record().head().said(&strings).into_text(),
        Reading::at(&write_a_record("whole", "{\"format\":1}\n"))
            .unwrap()
            .head()
            .said(&strings)
            .into_text(),
    ];

    let torn = write_a_record("torn", "{\"format\":1}\n{\"at\":{\"secs");
    let mut damage = Reading::at(&torn).unwrap().damage().clone();
    shown.extend(damage.said(&strings).into_iter().map(Said::into_text));
    damage = Reading::at(&write_a_record(
        "broken",
        "{\"format\":1}\n{\"at\":{\"secs\n{\"at\":{\"secs\n",
    ))
    .unwrap()
    .damage()
    .clone();
    shown.extend(damage.said(&strings).into_iter().map(Said::into_text));

    for failure in every_failure() {
        shown.push(failure.said(&strings).into_text());
    }

    // Every declared string turned up in something somebody can be shown.
    for word in EVERY_WORD {
        let english = word.says().split(" — ").next().unwrap_or(word.says());
        let english = english.split('{').next().unwrap_or(english).trim();
        assert!(
            shown.iter().any(|said| said.contains(english)),
            "{} is declared and nothing says it",
            word.named()
        );
    }
    assert!(
        shown
            .iter()
            .any(|said| said.contains("day") || said.contains("days")),
        "and the countable one is said too"
    );
}

/// One example of every way this crate says a record is not being kept.
fn every_failure() -> Vec<NotKept> {
    let path = "/var/lib/alo/record.jsonl".to_owned();
    vec![
        NotKept::NotThere { path: path.clone() },
        NotKept::NotARecord { path: path.clone() },
        NotKept::FromANewerAlo {
            path: path.clone(),
            format: 9,
        },
        NotKept::Damaged { path: path.clone() },
        NotKept::NotOpened {
            path: path.clone(),
            why: "permission denied".to_owned(),
        },
        NotKept::NotAddedTo {
            path: path.clone(),
            why: "no space left on device".to_owned(),
        },
        NotKept::NotRead {
            path: path.clone(),
            why: "input/output error".to_owned(),
        },
        NotKept::NotShortened {
            path,
            why: "read-only file system".to_owned(),
        },
    ]
}

/// A record file of this test's own, holding exactly this.
fn write_a_record(what: &str, text: &str) -> std::path::PathBuf {
    let folder =
        std::env::temp_dir().join(format!("alo-keeping-says-{}-{what}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("record.jsonl");
    std::fs::write(&path, text).unwrap();
    path
}

/// Nothing here reads the clock, so a test about time is arithmetic — kept
/// beside the others because a fixture that quietly used `SystemTime::now`
/// would make these tests pass on the day they were written.
#[test]
fn nothing_this_crate_says_depends_on_when_it_is_asked() {
    let noon = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    let thirty = Keeping::for_days(30).unwrap();
    assert_eq!(
        thirty.oldest_kept(noon),
        Some(noon - Duration::from_secs(30 * 24 * 60 * 60))
    );
    assert_eq!(
        Keeping::Forever.said(&in_english()).text(),
        Keeping::Forever.said(&in_english()).text()
    );
}
