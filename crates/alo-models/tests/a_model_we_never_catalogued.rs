//! A file on this machine is offered as a model with no catalogue entry, whose
//! licence is the person's and whose size was measured rather than stated.
//!
//! The unit tests beside `weights.rs` ask whether each door does what it says.
//! This one asks the thing `docs/features.md` promises from outside: *the
//! catalogue recommends; it does not gate*. A real file, the catalogue this
//! repository ships, and the three lines a person is shown — in a language
//! somebody actually reads, and in one nobody has translated.
//!
//! | The acceptance | The test |
//! |---|---|
//! | offered with no catalogue entry, licence *yours*, size measured off the file | [`a_file_is_offered_beside_the_catalogue_with_no_entry_in_it_and_its_size_measured`] |
//! | nothing about the catalogue's own entries changes | [`bringing_a_file_changes_nothing_about_the_catalogues_own_entries`] |
//! | the three lines are translated whole and none of them nudges | [`the_three_lines_are_read_in_the_language_the_person_reads_and_none_nudges`] |
//! | a path that is not a file is refused naming the path | [`a_path_that_is_not_a_file_is_refused_in_words_naming_the_path`] |

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_models::{Brought, Catalogue, Driving, Weights, WeightsError, model_words, words};
use alo_strings::{Language, Strings, Translation};

/// A folder on this machine's own disk that only this test uses, made fresh.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-models-never-catalogued-{what}"));
    if folder.exists() {
        fs::remove_dir_all(&folder).unwrap();
    }
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// A weights file of this many bytes, under that folder, with this name.
fn a_weights_file(folder: &Path, named: &str, bytes: usize) -> PathBuf {
    let file = folder.join(named);
    fs::write(&file, vec![0u8; bytes]).unwrap();
    file
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(model_words().unwrap())
}

/// This crate's words, with these said in German and German preferred.
fn speaking_german(said: &[(alo_strings::Word, &str)]) -> Strings {
    let vocabulary = model_words().unwrap();
    let german = Language::written("de").unwrap();
    let mut translation = Translation::into_language(german.clone());
    for (word, says) in said {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german]);
    strings
}

/// **A file is offered beside the catalogue, with no entry in it, its size
/// measured off the disk** — and the catalogue's own answer about the name is
/// that it has never heard of it, which is the right answer.
#[test]
fn a_file_is_offered_beside_the_catalogue_with_no_entry_in_it_and_its_size_measured() {
    let folder = a_folder_of_our_own("offered");
    let file = a_weights_file(&folder, "my-finetune.gguf", 300_000);
    let catalogue = Catalogue::built_in().unwrap();

    let weights = Weights::at(&file).unwrap();
    let mut brought = Brought::default();
    brought.add(weights.clone()).unwrap();

    assert_eq!(weights.bytes_on_disk, 300_000);
    assert_eq!(weights.bytes_on_disk, fs::metadata(&file).unwrap().len());
    assert!(catalogue.get(&weights.id).is_none());
    assert!(brought.get("my-finetune.gguf").is_some());
    assert_eq!(weights.drives_verbs, Driving::NotMeasured);

    // Whose licence it is, and that no rendering of the weights claims one.
    let [_, licence, _] = weights.lines(&in_english(), 16.0);
    assert!(licence.text().contains("yours"), "{licence}");
    let rendered = format!("{weights:?}") + &serde_json::to_string(&weights).unwrap();
    assert!(!rendered.to_lowercase().contains("licen"), "{rendered}");
}

/// **Bringing a file changes nothing about the catalogue's own entries.** The
/// catalogue is read before and after; every entry is the same entry, with the
/// same licence, and a file whose name happens to match one is still not it.
#[test]
fn bringing_a_file_changes_nothing_about_the_catalogues_own_entries() {
    let folder = a_folder_of_our_own("unchanged");
    let before = Catalogue::built_in().unwrap();
    // One the catalogue has measured, so the grade it states is one a file
    // could have wrongly inherited.
    let a_catalogued_name = before
        .models
        .iter()
        .find(|model| model.drives_verbs.has_been_measured())
        .unwrap()
        .id
        .clone();
    let file = a_weights_file(&folder, &a_catalogued_name, 12);

    let weights = Weights::at(&file).unwrap();
    let mut brought = Brought::default();
    brought.add(weights.clone()).unwrap();

    let after = Catalogue::built_in().unwrap();
    assert_eq!(after.models.len(), before.models.len());
    for (was, is) in before.models.iter().zip(&after.models) {
        assert_eq!(was.id, is.id);
        assert_eq!(format!("{:?}", was.licence), format!("{:?}", is.licence));
        assert_eq!(was.drives_verbs, is.drives_verbs);
    }
    // The file with the catalogued name is on the person's list, measured
    // off the disk, with the catalogue's grade and licence guessed for it in
    // neither direction.
    assert_eq!(weights.id, a_catalogued_name);
    assert_eq!(weights.bytes_on_disk, 12);
    assert_eq!(weights.drives_verbs, Driving::NotMeasured);
    assert_ne!(
        after.get(&a_catalogued_name).unwrap().drives_verbs,
        Driving::NotMeasured,
        "the catalogue states a grade for its own entry, and the file did not inherit it"
    );
}

/// **The three lines are read in the language the person reads, and none of
/// them nudges.** A German reader who has just pointed alo OS at their own
/// weights is told what it costs, whose terms they are, and that nobody has
/// measured them — and not that they would be better off with something alo
/// OS offers.
#[test]
fn the_three_lines_are_read_in_the_language_the_person_reads_and_none_nudges() {
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
        (
            words::WEIGHTS_NOT_MEASURED,
            "niemand hat gemessen, ob diese Gewichte die Verben des Agenten steuern können, und \
             alo OS hat nicht geraten — sie beantworten Ihre Fragen jetzt, und bekommen eine \
             Agentenrunde, sobald eine Messung sagt, dass sie es können",
        ),
    ]);
    let folder = a_folder_of_our_own("german");
    let weights = Weights::at(&a_weights_file(&folder, "theirs.gguf", 40)).unwrap();

    let [cost, licence, measured] = weights.lines(&strings, 0.000_000_001);
    assert!(cost.is_translated());
    assert!(cost.text().contains("trotzdem"), "{cost}");
    assert!(licence.is_translated());
    assert!(licence.text().contains("nicht gelesen"), "{licence}");
    assert!(measured.is_translated());
    assert!(measured.text().contains("nicht geraten"), "{measured}");

    for word in words::ABOUT_BROUGHT_WEIGHTS {
        let read =
            format!("{} {}", word.says(), word.note().unwrap_or_default()).to_ascii_lowercase();
        for nudge in words::NUDGES {
            assert!(!read.contains(nudge), "{} says \"{nudge}\"", word.named());
        }
    }
}

/// **A path that is not a file is refused in words naming the path**, and the
/// refusal is one sentence in the reader's language with the path carried
/// through untranslated.
#[test]
fn a_path_that_is_not_a_file_is_refused_in_words_naming_the_path() {
    let strings = speaking_german(&[(
        words::WEIGHTS_NO_FILE_THERE,
        "unter {path} liegt keine Datei, also wurde nichts hinzugefügt",
    )]);
    let folder = a_folder_of_our_own("refused");
    let nowhere = folder.join("nicht-da.gguf");

    let refused = Weights::at(&nowhere).unwrap_err();
    assert_eq!(refused, WeightsError::NoFileThere(nowhere.clone()));
    let said = refused.said(&strings);
    assert!(said.is_translated());
    assert!(said.text().contains("nicht-da.gguf"), "{said}");
    assert!(said.text().contains("nichts hinzugefügt"), "{said}");

    let refused = Weights::at(&folder).unwrap_err();
    assert_eq!(refused, WeightsError::NotAFile(folder.clone()));
    assert!(!refused.said(&in_english()).is_a_bug());
}
