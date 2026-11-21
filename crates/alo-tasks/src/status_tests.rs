//! That every status a task can be in is one of these, and each is its own.

use super::TheStatus;

/// **Each status is its own words, and its words find it again.**
#[test]
fn each_status_is_its_own_words_and_they_find_it_again() {
    for status in TheStatus::EVERY {
        assert_eq!(TheStatus::of(status.word()), Some(status), "{status:?}");
    }
    let keys: std::collections::BTreeSet<_> = TheStatus::EVERY
        .iter()
        .map(|status| status.word().named())
        .collect();
    assert_eq!(
        keys.len(),
        TheStatus::EVERY.len(),
        "two statuses share words"
    );
}

/// **And a word that is not a status is none of them**, so a task saying
/// something new is not drawn as though it said something else.
#[test]
fn a_word_that_is_not_a_status_is_none() {
    assert_eq!(TheStatus::of(crate::words::A_TASK), None);
}
