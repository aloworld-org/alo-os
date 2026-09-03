//! What this crate says, against the vocabulary the code actually uses.
//!
//! The unit tests read the list; this reads what a person would see, in a
//! language that is not the one the code is written in. Greek, because the two
//! sentences here are both instructions — *write the question first*, *choose a
//! model* — and a language that inflects and writes its own script is where an
//! instruction that had been assembled out of English pieces would show it.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_asking::{NotAQuestion, asking_words, declare_into};
use alo_strings::{Key, Language, Strings, Translation, Vocabulary};

/// Greek, and this crate's two strings in it.
fn in_greek() -> Strings {
    let vocabulary = asking_words().expect("this crate's own words");
    let greek = Language::written("el").expect("a language");
    let translation = Translation::into_language(greek.clone())
        .says(
            Key::named("asking.question.nothing").expect("a key"),
            "δεν υπάρχει ακόμη κάτι να ρωτήσετε — γράψτε πρώτα την ερώτηση",
        )
        .says(
            Key::named("asking.question.no-model").expect("a key"),
            "επιλέξτε ένα μοντέλο που θα απαντήσει σε αυτή την ερώτηση",
        );
    let speaking = vocabulary
        .check(translation)
        .expect("a translation of two sentences with no gaps in them");
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).expect("a checked translation");
    strings.prefers(&[greek]);
    strings
}

/// A machine with no translations shows the English, and says so about itself.
#[test]
fn a_machine_with_no_translations_still_says_what_to_do() {
    let strings = Strings::of(asking_words().expect("this crate's own words"));
    for refusal in [NotAQuestion::Nothing, NotAQuestion::NoModel] {
        let said = refusal.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(!said.is_translated(), "{said}");
    }
}

/// And a machine that has them shows those, whole.
#[test]
fn both_sentences_are_read_in_the_language_the_person_reads() {
    let strings = in_greek();
    let said = NotAQuestion::Nothing.said(&strings);
    assert!(said.is_translated(), "{said}");
    assert!(said.text().starts_with("δεν υπάρχει"), "{said}");

    let said = NotAQuestion::NoModel.said(&strings);
    assert!(said.is_translated(), "{said}");
    assert!(said.text().contains("μοντέλο"), "{said}");
}

/// **Everything this crate says is something it declares.** A refusal worded
/// with a key nothing declares reaches a person as the key itself, which no
/// check at declaration time can catch — so every one of them is put through
/// the lookup here, which is the test item 9g says every declaring crate owes.
#[test]
fn everything_this_crate_says_is_something_this_crate_declares() {
    let strings = Strings::of(asking_words().expect("this crate's own words"));
    for refusal in [NotAQuestion::Nothing, NotAQuestion::NoModel] {
        assert!(!refusal.said(&strings).is_a_bug(), "{refusal:?}");
    }

    // And a translator handed this crate is handed exactly these two — no key
    // that nothing declares, and none of them left out of the list.
    assert_eq!(strings.unanswered().len(), alo_asking::EVERY_WORD.len());
    let translated = in_greek();
    assert!(
        translated.unanswered().is_empty(),
        "{:?}",
        translated.unanswered()
    );
}

/// This crate's words go into a vocabulary that already holds four other
/// crates', which is the arrangement a shell actually has: one vocabulary, one
/// area per crate, and nothing quietly replacing anything.
#[test]
fn these_words_live_beside_every_other_crates_in_one_vocabulary() {
    let mut vocabulary = Vocabulary::empty();
    alo_models::declare_into(&mut vocabulary).expect("the model words");
    alo_egress::declare_into(&mut vocabulary).expect("the egress words");
    alo_answering::declare_into(&mut vocabulary).expect("the answering words");
    let before = vocabulary.how_many();

    declare_into(&mut vocabulary).expect("this crate's words beside them");
    assert_eq!(
        vocabulary.how_many(),
        before + alo_asking::EVERY_WORD.len(),
        "something was replaced rather than added"
    );
}
