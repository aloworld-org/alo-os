//! A file a person brought, measured, and what their machine does with the
//! grade.
//!
//! Task 4 of `docs/autonomy/v0-5-the-models-measured-plan.md`: *whether what a
//! person brings can be the agent*. The measurement is `alo-driving`'s, the same
//! fixed ten through the same door; what this file holds is where the result
//! goes and what it decides, walked through the one door a person's settings
//! are changed by and read back off the disk:
//!
//! | The acceptance | The test |
//! |---|---|
//! | the grade is written into the person's own settings beside the file, with the machine and date | [`a_grade_is_written_beside_the_file_with_the_machine_it_was_earned_on`] |
//! | a brought file that grades *reliably* may be given the agent | [`a_file_that_drives_the_verbs_reliably_may_be_given_the_agent`] |
//! | one that does not is offered for what it is | [`a_file_that_does_not_is_offered_for_what_it_is`] |
//! | the sentence for each outcome is in the machine's vocabulary, and none nudges toward the catalogue | [`the_sentence_for_each_outcome_is_the_machines_and_none_of_them_nudges`] |
//! | a grade for weights nobody brought, or with no machine a reader could check, is not written | [`a_grade_that_cannot_be_placed_is_not_written_and_the_file_is_what_it_was`] |
//!
//! That the grade travels nowhere is `a_grade_travels_nowhere.rs`, which reads
//! this crate's shipped source.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_choosing::{Choosing, NotWritten, Settings};
use alo_models::{Driving, MeasuredOn, WeightsError, words};
use alo_strings::Strings;

/// A folder on this machine's own disk that only this test uses, made fresh.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-choosing-measured-{what}-{}",
        std::process::id()
    ));
    if folder.exists() {
        fs::remove_dir_all(&folder).unwrap();
    }
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// Settings with one brought file on them, and the file's id.
fn a_person_who_brought(what: &str) -> (PathBuf, Choosing, String) {
    let folder = a_folder_of_our_own(what);
    let at = folder
        .join(alo_choosing::THE_FOLDER)
        .join(alo_choosing::THE_SETTINGS);
    let file = folder.join("their-own.Q4_K_M.gguf");
    fs::write(&file, b"GGUF").unwrap();
    let mut choosing = Choosing::at(&at).unwrap();
    choosing.bringing_a_file(&file).unwrap();
    (at, choosing, "their-own.Q4_K_M.gguf".to_owned())
}

/// The machine a measurement ran on.
fn the_machine() -> MeasuredOn {
    MeasuredOn {
        machine: "Apple M3, 8 GB unified memory".to_owned(),
        date: "2026-09-14".to_owned(),
        runtime: "Ollama 0.34.0".to_owned(),
        drove: None,
        of: None,
    }
}

/// The vocabulary a machine actually holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// The settings as they are on the disk now.
fn read_back(at: &Path) -> Settings {
    Settings::at(at).unwrap()
}

#[test]
fn a_grade_is_written_beside_the_file_with_the_machine_it_was_earned_on() {
    let (at, mut choosing, id) = a_person_who_brought("written");
    choosing
        .measuring(&id, Driving::Rarely, the_machine())
        .unwrap();

    let on_disk = fs::read_to_string(&at).unwrap();
    assert!(on_disk.contains("drives-verbs = \"rarely\""), "{on_disk}");
    assert!(on_disk.contains("[brought.measured]"), "{on_disk}");
    assert!(
        on_disk.contains("Apple M3, 8 GB unified memory"),
        "{on_disk}"
    );
    assert!(on_disk.contains("2026-09-14"), "{on_disk}");

    let read = read_back(&at);
    let theirs = read.brought().get(&id).unwrap();
    assert_eq!(theirs.drives_verbs, Driving::Rarely);
    assert_eq!(theirs.measured, Some(the_machine()));
    assert!(
        theirs.file.is_some(),
        "the grade was written and the file it is about was lost"
    );
}

#[test]
fn a_file_that_drives_the_verbs_reliably_may_be_given_the_agent() {
    let (at, mut choosing, id) = a_person_who_brought("reliably");
    choosing
        .measuring(&id, Driving::Reliably, the_machine())
        .unwrap();
    let read = read_back(&at);
    assert_eq!(read.brought().for_the_agent().len(), 1);
    assert!(read.brought().get(&id).unwrap().can_be_the_agent());
}

#[test]
fn a_file_that_does_not_is_offered_for_what_it_is() {
    for grade in [Driving::Sometimes, Driving::Rarely] {
        let (at, mut choosing, id) = a_person_who_brought(&format!("{grade:?}"));
        choosing.measuring(&id, grade, the_machine()).unwrap();
        let read = read_back(&at);
        let theirs = read.brought().get(&id).unwrap();
        assert!(!theirs.can_be_the_agent(), "{grade:?}");
        assert!(read.brought().for_the_agent().is_empty(), "{grade:?}");
        // Still on the list, and still a model their questions go to.
        assert_eq!(read.brought().weights.len(), 1, "{grade:?}");
    }
}

#[test]
fn the_sentence_for_each_outcome_is_the_machines_and_none_of_them_nudges() {
    let strings = what_this_machine_can_say();
    let mut said = Vec::new();
    for (grade, word) in [
        (Driving::Reliably, words::WEIGHTS_MEASURED_THE_AGENT),
        (Driving::Sometimes, words::WEIGHTS_MEASURED_NOT_THE_AGENT),
        (Driving::Rarely, words::WEIGHTS_MEASURED_NOT_THE_AGENT),
    ] {
        let (at, mut choosing, id) = a_person_who_brought(&format!("said-{grade:?}"));
        choosing.measuring(&id, grade, the_machine()).unwrap();
        let line = read_back(&at)
            .brought()
            .get(&id)
            .unwrap()
            .measurement(&strings);
        assert!(!line.is_a_bug(), "{grade:?}: {line}");
        assert_eq!(
            line.text(),
            strings
                .say(&word.key(), &alo_strings::Filling::nothing())
                .text()
        );
        said.push(line.into_text());
    }
    for line in &said {
        for nudge in words::NUDGES {
            assert!(
                !line.to_lowercase().contains(nudge),
                "`{nudge}` beside somebody's own weights: {line}"
            );
        }
        assert!(!line.to_lowercase().contains("catalogue"), "{line}");
    }
}

#[test]
fn a_grade_that_cannot_be_placed_is_not_written_and_the_file_is_what_it_was() {
    let (at, mut choosing, id) = a_person_who_brought("refused");
    let before = fs::read(&at).unwrap();

    let refused = choosing
        .measuring("nobody-brought-this.gguf", Driving::Reliably, the_machine())
        .unwrap_err();
    assert!(
        matches!(&refused, NotWritten::NothingToMeasure { model, .. } if model == "nobody-brought-this.gguf"),
        "{refused:?}"
    );
    assert!(
        refused
            .said(&what_this_machine_can_say())
            .text()
            .contains("nobody-brought-this.gguf")
    );
    assert_eq!(fs::read(&at).unwrap(), before);

    let vague = MeasuredOn {
        machine: "my laptop".to_owned(),
        ..the_machine()
    };
    let refused = choosing
        .measuring(&id, Driving::Reliably, vague)
        .unwrap_err();
    assert!(
        matches!(
            &refused,
            NotWritten::NotWeights { why: WeightsError::GradeNotPlaced(named), .. } if *named == id
        ),
        "{refused:?}"
    );
    assert_eq!(fs::read(&at).unwrap(), before);
    assert!(
        !read_back(&at)
            .brought()
            .get(&id)
            .unwrap()
            .can_be_the_agent()
    );
}
