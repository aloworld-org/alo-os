//! Every sentence the three crates of documents and paper can say is in the
//! machine's one vocabulary, with a note for whoever translates it.
//!
//! Task 5 of `docs/autonomy/v0-5-documents-and-paper-plan.md`. `alo-opening`,
//! `alo-converting` and `alo-printing` each hold their own list in their own
//! tests. What this holds is the promise across all three: every sentence a
//! person meets when a file arrives, is converted, is printed or cannot be
//! opened reaches them in their own language — so each one is **collected** by
//! `alo-saying`, exactly as its crate declares it, and each one tells a
//! translator what it is for and what every gap in it holds. A translator handed
//! *This is {what}, and this machine opens it as it is* with no word about
//! `{what}` is a translator who may render it as a noun of their own.
//!
//! It asks the vocabulary `alo-saying` collects, because that is the one a
//! translation is checked against and the one every sentence a person reads
//! comes out of.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;

use alo_strings::{Vocabulary, Word};

/// The three crates, each with its area and everything it declares.
const THE_THREE: [(&str, &[Word]); 3] = [
    ("opening", &alo_opening::EVERY_WORD),
    ("converting", &alo_converting::EVERY_WORD),
    ("printing", &alo_printing::EVERY_WORD),
];

/// The fewest words a note can have and still say what a sentence is for.
const A_NOTE_THAT_SAYS_SOMETHING: usize = 8;

/// The machine's one vocabulary.
fn the_machines() -> Vocabulary {
    alo_saying::everything_this_machine_can_say().unwrap()
}

/// Every phrase the machine collected under this area, as its source and note
/// by key.
fn collected_under(vocabulary: &Vocabulary, area: &str) -> BTreeMap<String, (String, String)> {
    vocabulary
        .phrases()
        .filter(|phrase| phrase.key().area() == area)
        .map(|phrase| {
            (
                phrase.key().to_string(),
                (
                    phrase.source().as_written().to_owned(),
                    phrase.note().unwrap_or_default().to_owned(),
                ),
            )
        })
        .collect()
}

/// Every word a crate declares, as its source and note by key.
fn declared(words: &[Word]) -> BTreeMap<String, (String, String)> {
    words
        .iter()
        .map(|word| {
            (
                word.named().to_owned(),
                (
                    word.says().to_owned(),
                    word.note().unwrap_or_default().to_owned(),
                ),
            )
        })
        .collect()
}

/// Why this note does not do its job for this source, or `None` when it does.
fn what_the_note_lacks(source: &str, note: &str) -> Option<String> {
    if note.split_whitespace().count() < A_NOTE_THAT_SAYS_SOMETHING {
        return Some(format!(
            "a note of fewer than {A_NOTE_THAT_SAYS_SOMETHING} words"
        ));
    }
    let gaps: Vec<&str> = source
        .split('{')
        .skip(1)
        .filter_map(|after| after.split_once('}').map(|(gap, _)| gap))
        .collect();
    gaps.into_iter()
        .find(|gap| !note.contains(&format!("{{{gap}}}")))
        .map(|gap| format!("nothing in the note says what {{{gap}}} holds"))
}

/// **Everything the three crates declare is collected by the machine, exactly
/// as declared** — no sentence a crate can say is missing from the vocabulary a
/// translation is checked against, and nothing is collected under their names
/// that they do not declare.
#[test]
fn every_sentence_the_three_crates_declare_is_the_machines() {
    let vocabulary = the_machines();
    for (area, words) in THE_THREE {
        assert!(!words.is_empty(), "{area} declares nothing");
        assert_eq!(
            collected_under(&vocabulary, area),
            declared(words),
            "{area}: what the machine collected is not what the crate declares"
        );
        assert_eq!(
            vocabulary
                .counted()
                .filter(|plural| plural.key().area() == area)
                .count(),
            0,
            "{area} counts something no test here reads a note for"
        );
    }
}

/// **Every sentence carries a note that says something**, and every gap in a
/// sentence is named in its note, so a translator knows what will stand there.
#[test]
fn every_sentence_the_three_crates_say_carries_a_note_naming_its_gaps() {
    let vocabulary = the_machines();
    let mut lacking = Vec::new();
    for (area, _) in THE_THREE {
        for (key, (source, note)) in collected_under(&vocabulary, area) {
            if let Some(lacks) = what_the_note_lacks(&source, &note) {
                lacking.push(format!("{key}: {lacks}"));
            }
        }
    }
    assert!(
        lacking.is_empty(),
        "sentences a translator is handed without enough to go on: {lacking:#?}"
    );
}

/// **A note that says nothing, or leaves a gap unexplained, is found.** Held
/// against text, so the rule is shown refusing without planting a bad note in
/// the vocabulary.
#[test]
fn a_note_that_says_nothing_or_leaves_a_gap_unexplained_is_found() {
    assert!(what_the_note_lacks("The printer is ready", "").is_some());
    assert!(what_the_note_lacks("The printer is ready", "A status line.").is_some());
    assert_eq!(
        what_the_note_lacks(
            "This is {what}, and it opens",
            "Said when a file opens as it is, straight after it arrives on this machine."
        ),
        Some("nothing in the note says what {what} holds".to_owned())
    );
    assert_eq!(
        what_the_note_lacks(
            "This is {what}, and it opens",
            "Said when a file opens as it is. {what} is the name of the kind of file."
        ),
        None
    );
    assert_eq!(
        what_the_note_lacks(
            "print {document} on {printer}",
            "The sentence a person approves. {document} is where the file is on this machine."
        ),
        Some("nothing in the note says what {printer} holds".to_owned())
    );
}
