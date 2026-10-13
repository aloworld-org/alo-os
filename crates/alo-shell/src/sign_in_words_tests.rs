//! Every label this screen draws is declared, and declared once.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a vocabulary that will not declare is the failure reported"
)]

use super::{SIGN_IN_WORDS, declare_sign_in_words};
use alo_strings::Vocabulary;

/// **Every word declares, and the keys are its own.**
#[test]
fn every_word_declares_under_this_screens_own_prefix() {
    let mut vocabulary = Vocabulary::empty();
    declare_sign_in_words(&mut vocabulary).expect("this screen's wording declares");
    for word in SIGN_IN_WORDS {
        assert!(
            word.key().as_str().starts_with("shell.sign-in."),
            "{} is not under this screen's prefix",
            word.key().as_str()
        );
        assert!(
            word.phrase()
                .expect("a declared word has a phrase")
                .note()
                .is_some(),
            "{} has no note, so a translator is given no context",
            word.key().as_str()
        );
    }
}

/// **Declaring twice is refused**, and refusing changes nothing.
///
/// The rule `crate::window_control_reader_words` holds: a collision leaves
/// every existing declaration untouched, so a caller that declares into a
/// vocabulary somebody else filled cannot half-apply.
#[test]
fn declaring_the_same_words_twice_is_refused_and_changes_nothing() {
    let mut vocabulary = Vocabulary::empty();
    declare_sign_in_words(&mut vocabulary).expect("the first time declares");
    let before = vocabulary.clone();
    assert!(
        declare_sign_in_words(&mut vocabulary).is_err(),
        "the same keys declared twice were accepted"
    );
    assert_eq!(
        vocabulary.phrases().count(),
        before.phrases().count(),
        "a refused declaration still changed the vocabulary"
    );
}
