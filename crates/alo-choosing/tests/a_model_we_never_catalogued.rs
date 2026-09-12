//! Run a model we never catalogued — the catalogue recommends, it does not
//! gate.
//!
//! Three sentences from `docs/features.md`, read as one rule from three sides:
//! *point alo OS at weights you already have and it runs them*; *a model too
//! large for the memory in this laptop is said so plainly, once — and then run
//! anyway*; *what you bring is yours, including its licence, and alo OS does
//! not pretend to have checked it.*
//!
//! Everything here is a real file on a real disk, read back through the one
//! door a daemon reads settings through, and worded in the vocabulary the
//! whole machine loads rather than this crate's own list — because a sentence
//! that is only in one crate's list is a sentence that reaches a person as a
//! key.
//!
//! | The acceptance | The test |
//! |---|---|
//! | a weights file on this machine is named as a model source through this crate's own shapes | [`a_weights_file_on_this_machine_is_named_as_a_model_source_and_read_back`] |
//! | it is offered with no catalogue entry, its licence *yours*, its size measured off the file | [`what_is_offered_has_no_catalogue_entry_a_licence_that_is_theirs_and_a_measured_size`] |
//! | a file larger than the machine's memory is chosen and answers anyway | [`a_file_larger_than_this_machines_memory_is_still_chosen_and_still_asked`] |
//! | a path that is not a weights file is refused naming the path, and nothing is written | [`a_path_that_is_not_a_weights_file_is_refused_and_the_settings_are_byte_for_byte_what_they_were`] |
//! | every sentence is in the vocabulary `alo-saying` collects, and none nudges | [`every_sentence_about_their_own_weights_is_in_the_machines_vocabulary_and_none_nudges`] |
//! | `drives_verbs` for a brought file is not measured, and the sentence says so | [`what_a_brought_file_drives_is_not_measured_and_the_sentence_says_so`] |

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_choosing::{
    Choosing, Chosen, NotWritten, Picked, Settings, THE_FOLDER, THE_SETTINGS, Which,
};
use alo_models::{Catalogue, Driving, InferenceSource, SourcePolicy, WeightsError, words};
use alo_strings::Strings;

/// A folder on this machine's own disk that only this test uses, made fresh.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-choosing-never-catalogued-{what}"));
    if folder.exists() {
        fs::remove_dir_all(&folder).unwrap();
    }
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// Where this person's settings go, under that folder.
fn settings_in(folder: &Path) -> PathBuf {
    folder.join(THE_FOLDER).join(THE_SETTINGS)
}

/// A weights file of this many bytes, under that folder, with this name.
fn a_weights_file(folder: &Path, named: &str, bytes: usize) -> PathBuf {
    let file = folder.join(named);
    fs::write(&file, vec![0u8; bytes]).unwrap();
    file
}

/// Everything a real machine can say, which is what a shell holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// **A weights file on this machine is named as a model source through this
/// crate's own shapes**, and reads back through the door a daemon reads
/// settings through: the id is the file's name, the size is the file's, and
/// where it is, is written down.
#[test]
fn a_weights_file_on_this_machine_is_named_as_a_model_source_and_read_back() {
    let folder = a_folder_of_our_own("named");
    let at = settings_in(&folder);
    let file = a_weights_file(&folder, "my-finetune.Q4_K_M.gguf", 65_536);
    let mut choosing = Choosing::at(&at).unwrap();

    choosing.bringing_a_file(&file).unwrap();
    choosing
        .answered_by(Some(Picked::OnThisMachine(
            Chosen::of(Which::Brought, "my-finetune.Q4_K_M.gguf").unwrap(),
        )))
        .unwrap();

    let read = Settings::at(&at).unwrap();
    let weights = read.weights().unwrap();
    assert_eq!(weights.id, "my-finetune.Q4_K_M.gguf");
    assert_eq!(weights.bytes_on_disk, 65_536);
    assert_eq!(weights.file.as_deref(), Some(file.as_path()));
    // And the file on the disk says where the weights are, in the shape the
    // contract names.
    let text = fs::read_to_string(&at).unwrap();
    assert!(text.contains("[[brought]]"), "{text}");
    assert!(text.contains("file = "), "{text}");
    assert!(text.contains("bytes-on-disk = 65536"), "{text}");
}

/// **What is offered has no catalogue entry, a licence that is the person's,
/// and a size that was measured rather than stated.** The catalogue alo OS
/// ships is exactly what it was before and after, and it has never heard of
/// this file.
#[test]
fn what_is_offered_has_no_catalogue_entry_a_licence_that_is_theirs_and_a_measured_size() {
    let folder = a_folder_of_our_own("offered");
    let at = settings_in(&folder);
    let file = a_weights_file(&folder, "theirs.gguf", 1_024);
    let catalogue_before = Catalogue::built_in().unwrap();
    let mut choosing = Choosing::at(&at).unwrap();

    choosing.bringing_a_file(&file).unwrap();

    let brought = choosing.settings().brought();
    let weights = brought.get("theirs.gguf").unwrap();
    assert_eq!(weights.bytes_on_disk, fs::metadata(&file).unwrap().len());
    // No catalogue entry, before or after, and the catalogue is unchanged.
    assert!(catalogue_before.get("theirs.gguf").is_none());
    let catalogue_after = Catalogue::built_in().unwrap();
    assert!(catalogue_after.get("theirs.gguf").is_none());
    assert_eq!(catalogue_after.models.len(), catalogue_before.models.len());

    // Whose licence it is, in the machine's own vocabulary.
    let strings = what_this_machine_can_say();
    let [cost, licence, measured] = weights.lines(&strings, 16.0);
    assert!(!cost.is_a_bug(), "{cost}");
    assert!(licence.text().contains("yours"), "{licence}");
    assert!(licence.text().contains("has not read"), "{licence}");
    assert!(!measured.is_a_bug(), "{measured}");
}

/// **A file larger than this machine's memory is still chosen and still
/// asked.** The cost is a sentence; the choice is written; the permission to
/// put a question to it is granted on a machine with no bound and on one whose
/// bound is *this machine only* — because it is this machine.
#[test]
fn a_file_larger_than_this_machines_memory_is_still_chosen_and_still_asked() {
    let folder = a_folder_of_our_own("larger");
    let at = settings_in(&folder);
    let file = a_weights_file(&folder, "enormous.gguf", 2_000_000);
    let mut choosing = Choosing::at(&at).unwrap();
    choosing.bringing_a_file(&file).unwrap();
    let weights = choosing
        .settings()
        .brought()
        .get("enormous.gguf")
        .unwrap()
        .clone();

    // A machine with less memory than the file: the case worth saying.
    let cost = weights.costs_on(0.001);
    assert!(cost.larger_than_memory());
    let said = cost.said(&what_this_machine_can_say());
    assert!(said.text().contains("will still run them"), "{said}");

    // Chosen anyway, and written.
    choosing
        .answered_by(Some(Picked::OnThisMachine(
            Chosen::of(Which::Brought, "enormous.gguf").unwrap(),
        )))
        .unwrap();
    let read = Settings::at(&at).unwrap();
    let chosen = read.chosen().unwrap().on_this_machine().unwrap();
    assert_eq!(chosen.model(), "enormous.gguf");

    // And asked: nothing between the choice and the question refuses it.
    for bound in [None, Some(&SourcePolicy::ThisMachineOnly)] {
        let answering = chosen.asking(bound).unwrap();
        assert_eq!(answering.source(), &InferenceSource::ThisMachine);
    }
}

/// **A path that is not a weights file is refused, naming the path, and the
/// settings are byte for byte what they were.** Nothing there, and a folder:
/// the two ways a file picker's answer is not a file.
#[test]
fn a_path_that_is_not_a_weights_file_is_refused_and_the_settings_are_byte_for_byte_what_they_were()
{
    let folder = a_folder_of_our_own("refused");
    let at = settings_in(&folder);
    let mut choosing = Choosing::at(&at).unwrap();
    choosing
        .bringing_a_file(&a_weights_file(&folder, "first.gguf", 8))
        .unwrap();
    let before = fs::read_to_string(&at).unwrap();
    let strings = what_this_machine_can_say();

    let nowhere = folder.join("missing.gguf");
    let refused = choosing.bringing_a_file(&nowhere).unwrap_err();
    assert!(
        matches!(&refused, NotWritten::NotWeights { why: WeightsError::NoFileThere(path), .. } if *path == nowhere),
        "{refused:?}"
    );
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("missing.gguf"), "{said}");
    assert!(said.text().contains("nothing has been added"), "{said}");
    assert_eq!(fs::read_to_string(&at).unwrap(), before);

    let refused = choosing.bringing_a_file(&folder).unwrap_err();
    assert!(
        matches!(&refused, NotWritten::NotWeights { why: WeightsError::NotAFile(path), .. } if *path == folder),
        "{refused:?}"
    );
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("is not a file"), "{said}");
    assert_eq!(fs::read_to_string(&at).unwrap(), before);
    assert_eq!(choosing.settings().brought().weights.len(), 1);
}

/// **Every sentence about the person's own weights is in the vocabulary
/// `alo-saying` collects, and none of them nudges** — toward a catalogued
/// model or away from their own. Read here against the machine's whole
/// vocabulary rather than `alo-models`' list, because a word declared and
/// not collected reaches a person as a key.
#[test]
fn every_sentence_about_their_own_weights_is_in_the_machines_vocabulary_and_none_nudges() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for word in words::ABOUT_BROUGHT_WEIGHTS {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "the machine cannot say {}",
            word.named()
        );
        let read =
            format!("{} {}", word.says(), word.note().unwrap_or_default()).to_ascii_lowercase();
        for nudge in words::NUDGES {
            assert!(!read.contains(nudge), "{} says \"{nudge}\"", word.named());
        }
    }
}

/// **What a brought file drives is not measured, and the sentence says so.**
/// The catalogue's grades are for catalogued entries and are never guessed for
/// a file: the third line under somebody's own weights says nobody measured
/// and nothing was guessed, and the list a turn is chosen from does not offer
/// the file until somebody does.
#[test]
fn what_a_brought_file_drives_is_not_measured_and_the_sentence_says_so() {
    let folder = a_folder_of_our_own("unmeasured");
    let at = settings_in(&folder);
    let file = a_weights_file(&folder, "theirs.gguf", 16);
    let mut choosing = Choosing::at(&at).unwrap();
    choosing.bringing_a_file(&file).unwrap();

    let weights = Settings::at(&at)
        .unwrap()
        .brought()
        .get("theirs.gguf")
        .unwrap()
        .clone();
    assert_eq!(weights.drives_verbs, Driving::NotMeasured);
    assert!(!weights.can_be_the_agent());
    assert!(choosing.settings().brought().for_the_agent().is_empty());

    let [_, _, measured] = weights.lines(&what_this_machine_can_say(), 16.0);
    assert!(!measured.is_a_bug(), "{measured}");
    assert!(
        measured.text().contains("nobody has measured"),
        "{measured}"
    );
    assert!(measured.text().contains("has not guessed"), "{measured}");
    assert!(
        fs::read_to_string(&at)
            .unwrap()
            .contains("drives-verbs = \"not-measured\""),
        "the file states the measurement rather than leaving it blank"
    );
}
