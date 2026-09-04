//! What this crate says, against the vocabulary the code actually uses.
//!
//! There are two sentences, so this file is mostly about the other side of the
//! bargain: that everything *else* a turn hands a person is somebody else's
//! string, read out of the one vocabulary a shell has.
//!
//! Finnish, because the sentence is a statement about something that has
//! already happened and Finnish builds it with cases rather than with the word
//! order English uses — a translation assembled out of English pieces would
//! show here.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Key, Language, Strings, Translation};
use alo_turn::{EVERY_WORD, NoBoundary, NotDone, declare_into, turn_words};

/// The turn that could not be bounded, as a caller meets it.
fn not_bounded() -> NotDone {
    NotDone::NotBounded(NoBoundary::because(
        "the kernel would not attach the boundary to file_open".to_owned(),
    ))
}

/// Finnish, and this crate's two strings in it.
fn in_finnish() -> Strings {
    let vocabulary = turn_words().expect("this crate's own words");
    let finnish = Language::written("fi").expect("a language");
    let translation = Translation::into_language(finnish.clone())
        .says(
            Key::named("turn.closed").expect("a key"),
            "tämä vuoro pysäytettiin, koska sen tapahtumia ei voitu kirjata muistiin",
        )
        .says(
            Key::named("turn.not-bounded").expect("a key"),
            "mitään ei tehty: tämä kone ei voi rajata agenttia siihen, minkä olet sille antanut",
        );
    let speaking = vocabulary
        .check(translation)
        .expect("a translation of two sentences with no gaps in them");
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).expect("a checked translation");
    strings.prefers(&[finnish]);
    strings
}

/// A machine with no translations shows the English, and says so about itself.
#[test]
fn a_machine_with_no_translations_still_says_what_has_happened() {
    let strings = Strings::of(turn_words().expect("this crate's own words"));
    let said = NotDone::TurnClosed.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(!said.is_translated(), "{said}");
    assert!(said.text().contains("could not be written down"), "{said}");
}

/// And a machine that has them shows those, whole.
#[test]
fn the_sentence_is_read_in_the_language_the_person_reads() {
    let said = NotDone::TurnClosed.said(&in_finnish());
    assert!(said.is_translated(), "{said}");
    assert!(said.text().starts_with("tämä vuoro"), "{said}");
}

/// **A turn nothing could be bounded to says so in the person's own language**,
/// which is the whole reason that sentence moved here: it lived in a Linux-only
/// crate whose words nothing put into a machine's vocabulary, so a person on a
/// machine that could not bound anything would have been handed a key.
///
/// The reason itself is not in it. It is a fact about a kernel, in English, and
/// it travels beside the sentence rather than inside it.
#[test]
fn a_turn_that_could_not_be_bounded_says_so_in_the_persons_own_language() {
    let strings = in_finnish();
    let not_bounded = not_bounded();
    let said = not_bounded.said(&strings);
    assert!(said.is_translated(), "{said}");
    assert!(said.text().starts_with("mitään ei tehty"), "{said}");
    assert!(
        !said.text().contains("file_open"),
        "the administrator's sentence reached the person: {said}"
    );
    assert_ne!(
        NotDone::TurnClosed.said(&strings).text(),
        said.text(),
        "a turn that stopped and a turn that was never bounded say the same thing"
    );
}

/// **Everything this crate says is something it declares.** A refusal worded
/// with a key nothing declares reaches a person as the key itself, which no
/// check at declaration time can catch — so it is put through the lookup here,
/// which is the test item 9g says every declaring crate owes.
#[test]
fn everything_this_crate_says_is_something_this_crate_declares() {
    let strings = Strings::of(turn_words().expect("this crate's own words"));
    assert!(!NotDone::TurnClosed.said(&strings).is_a_bug());
    assert!(!not_bounded().said(&strings).is_a_bug());

    // And a translator handed this crate is handed exactly this one — no key
    // that nothing declares, and none left out of the list.
    assert_eq!(strings.unanswered().len(), EVERY_WORD.len());
    let translated = in_finnish();
    assert!(
        translated.unanswered().is_empty(),
        "{:?}",
        translated.unanswered()
    );
}

/// This crate's word goes into a vocabulary that already holds three other
/// crates', which is the arrangement a shell actually has: one vocabulary, one
/// area per crate, and nothing quietly replacing anything.
#[test]
fn this_word_lives_beside_every_other_crates_in_one_vocabulary() {
    let mut vocabulary = alo_files::file_words().expect("the file words");
    alo_capability::declare_into(&mut vocabulary).expect("the capability words");
    alo_keeping::declare_into(&mut vocabulary).expect("the record-keeping words");
    let before = vocabulary.how_many();

    declare_into(&mut vocabulary).expect("this crate's word beside them");
    assert_eq!(
        vocabulary.how_many(),
        before + EVERY_WORD.len(),
        "something was replaced rather than added"
    );
}

/// **A turn is the last crate in the chain and says the least of any of them.**
/// One string against three lists that between them are what a person actually
/// reads during a turn — the sentence they approve, the refusal they are given,
/// and the reason their machine stopped keeping evidence.
#[test]
fn a_turn_says_less_than_any_of_the_crates_it_joins() {
    let ours = turn_words().expect("this crate's own words").how_many();
    for (whose, how_many) in [
        (
            "alo-capability",
            alo_capability::capability_words()
                .expect("the capability words")
                .how_many(),
        ),
        (
            "alo-files",
            alo_files::file_words().expect("the file words").how_many(),
        ),
        (
            "alo-keeping",
            alo_keeping::keeping_words()
                .expect("the record-keeping words")
                .how_many(),
        ),
    ] {
        assert!(
            ours < how_many,
            "this crate now says as much as {whose}: {ours} against {how_many}"
        );
    }
}
