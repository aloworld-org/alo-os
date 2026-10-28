//! The language the sheet chose, followed from the machine's file to the
//! person's dates and words.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_strings::{Language, Strings, Translation};

use super::the_languages_they_read;
use crate::the_language_a_person_chose::{AboutTheLanguage, TheLanguageAPersonChose};

/// A machine and a person, each with a folder of their own.
struct AMachine {
    _folder: tempfile::TempDir,
    machine: PathBuf,
    settings: PathBuf,
}

fn a_fresh_machine() -> AMachine {
    let folder = tempfile::tempdir().unwrap();
    let machine = folder.path().join("etc").join("language.toml");
    std::fs::create_dir_all(machine.parent().unwrap()).unwrap();
    let settings = folder.path().join("home").join("alo").join("settings.toml");
    AMachine {
        machine,
        settings,
        _folder: folder,
    }
}

fn german() -> Language {
    Language::written("de").unwrap()
}

/// 2026-10-28 17:38 in Berlin, which is 16:38 universal time.
fn half_past_five() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_793_205_480)
}

/// **A fresh person who picked German on the sheet gets German dates and words,
/// with nothing else chosen** — the owner's decision of 2026-10-28, walked from
/// the sheet's keeper to what the desktop draws.
#[test]
fn a_fresh_person_who_picked_german_reads_german_dates_and_words() {
    let at = a_fresh_machine();
    // The sheet, as the compositor keeps it.
    let mut sheet = TheLanguageAPersonChose::nothing_yet();
    assert_eq!(
        sheet.chose(Some("de"), &at.machine),
        AboutTheLanguage::Kept("de".to_owned())
    );

    // The person's first sign-in: no settings file at all.
    assert!(!at.settings.exists());
    let read = the_languages_they_read(&at.settings, &at.machine);
    assert_eq!(read.languages, vec![german()]);
    assert!(read.for_the_log.is_some(), "a language given said nothing");
    assert_eq!(
        alo_choosing::Settings::at(&at.settings)
            .unwrap()
            .languages()
            .to_vec(),
        vec![german()],
        "the language was used and not kept as theirs"
    );

    // Dates: nothing in formats.toml, so German's own way.
    let formats = crate::TheirFormats::of(
        alo_formats::Chosen::default(),
        read.languages.first().map(Language::tag),
    )
    .in_zone(jiff::tz::TimeZone::get("Europe/Berlin").unwrap());
    assert_eq!(formats.regionally().language(), "de");
    assert_eq!(formats.time_of(half_past_five()).as_deref(), Some("17:38"));
    let three_days_back = half_past_five() - Duration::from_secs(3 * 24 * 60 * 60);
    assert_eq!(
        formats.day_of(three_days_back, half_past_five()),
        Some(crate::TheDay::On("25.10.2026".to_owned()))
    );

    // Words: where the machine has German, the desktop says German.
    let vocabulary = alo_formats::formats_words().unwrap();
    let speaking = vocabulary
        .check(Translation::into_language(german()).says(alo_formats::words::TODAY.key(), "Heute"))
        .unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&read.languages);
    assert_eq!(crate::TheDay::Today.said(&strings), "Heute");
}

/// **A person who has said what they read keeps it**, whatever the machine
/// was set up in, and nothing is written.
#[test]
fn a_person_who_said_what_they_read_keeps_it() {
    let at = a_fresh_machine();
    alo_formats::the_machines_language::keep(&at.machine, &german()).unwrap();
    let mut choosing = alo_choosing::Choosing::at(&at.settings).unwrap();
    let french = Language::written("fr").unwrap();
    choosing.reading(vec![french.clone()]).unwrap();
    let before = std::fs::read_to_string(&at.settings).unwrap();
    let read = the_languages_they_read(&at.settings, &at.machine);
    assert_eq!(read.languages, vec![french]);
    assert_eq!(read.for_the_log, None);
    assert_eq!(std::fs::read_to_string(&at.settings).unwrap(), before);
}

/// **A machine nobody set up gives nobody a language**, and writes nothing.
#[test]
fn a_machine_nobody_set_up_gives_nothing() {
    let at = a_fresh_machine();
    let read = the_languages_they_read(&at.settings, &at.machine);
    assert!(read.languages.is_empty());
    assert!(
        !at.settings.exists(),
        "nothing to give and a file was written"
    );
}

/// **Settings that do not read are not written over**, and the reason is
/// said rather than a language guessed into a file nobody can read.
#[test]
fn settings_that_do_not_read_are_left_and_said() {
    let at = a_fresh_machine();
    alo_formats::the_machines_language::keep(&at.machine, &german()).unwrap();
    std::fs::create_dir_all(at.settings.parent().unwrap()).unwrap();
    std::fs::write(&at.settings, "this is not toml").unwrap();
    let read = the_languages_they_read(&at.settings, &at.machine);
    assert!(read.languages.is_empty());
    assert!(read.for_the_log.is_some());
    assert_eq!(
        std::fs::read_to_string(&at.settings).unwrap(),
        "this is not toml"
    );
}

/// **A machine file that does not read gives nothing**, and says so.
#[test]
fn a_machine_file_that_does_not_read_gives_nothing_and_says_so() {
    let at = a_fresh_machine();
    std::fs::write(&at.machine, "format = 1\nlanguage = 3\n").unwrap();
    let read = the_languages_they_read(&at.settings, &at.machine);
    assert!(read.languages.is_empty());
    assert!(read.for_the_log.is_some());
    assert!(!at.settings.exists());
}
