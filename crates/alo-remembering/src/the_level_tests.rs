//! What the level file reads, what it refuses, and the one thing it must never
//! do: answer *the sealed box* to the question *what did this person choose*.

use super::*;

/// Every level survives being written down and read back.
#[test]
fn every_level_goes_down_and_comes_back() {
    for chose in HowCodeRuns::ALL {
        let text = written(chose).expect("a level writes");
        assert_eq!(
            read(&text).expect("what this crate wrote, this crate reads"),
            chose,
            "{chose:?} did not survive {text:?}"
        );
    }
}

/// **The three words are `HowCodeRuns`' own `serde` names**, and this is the
/// check that keeps them so.
///
/// [`the_word_for`] is a `match` written by hand, which is one list a reader
/// can see — and a second list is free to drift from the `kebab-case` names
/// the enum derives. Nothing in the product notices a drift, because this
/// crate reads what this crate wrote and agrees with itself either way. So the
/// agreement is asserted here, against the derived name taken out of `serde`
/// once, in a test, where a quote-splitting parser is cheap rather than load
/// bearing.
#[test]
fn the_word_is_the_one_serde_derives() {
    for chose in HowCodeRuns::ALL {
        #[derive(Serialize)]
        struct One {
            it: HowCodeRuns,
        }
        let table = toml::to_string(&One { it: chose }).expect("a level serialises");
        let derived = table
            .split('"')
            .nth(1)
            .unwrap_or_else(|| panic!("{chose:?} wrote as {table:?}, which has no word in it"));
        assert_eq!(
            the_word_for(chose),
            derived,
            "this crate writes {:?} for {chose:?} where serde derives {derived:?}",
            the_word_for(chose)
        );
    }
}

/// **A file that says no level is refused, and is not read as the default.**
///
/// The whole reason this file is read as an `Option` by its caller. A `chose`
/// somebody deleted by hand must not come back as *in a sealed box*, because
/// that is a default wearing their decision. `SaysNothing` names it.
#[test]
fn a_file_that_says_no_level_is_refused_and_never_defaulted() {
    let said_nothing = format!("format = {THE_LEVEL_FORMAT}\n");
    assert_eq!(read(&said_nothing), Err(NotTheLevel::SaysNothing));

    // And the mistake this is guarding against, stated as a test: if this ever
    // answers `Ok`, it must not be the default.
    if let Ok(level) = read(&said_nothing) {
        assert_ne!(
            level,
            HowCodeRuns::default(),
            "a file saying no level answered with the default, which is a default dressed as a choice"
        );
    }
}

/// **A spelling this does not know is refused, and the refusal names the
/// three.**
///
/// No nearest match: `full-trust` is not read as `at-full-trust`, because
/// guessing is choosing for the person.
#[test]
fn a_spelling_it_does_not_know_is_named_rather_than_guessed() {
    let nearly = format!("format = {THE_LEVEL_FORMAT}\nchose = \"full-trust\"\n");
    let why = read(&nearly).expect_err("`full-trust` is not a level");
    assert_eq!(
        why,
        NotTheLevel::NoSuchLevel {
            said: "full-trust".to_owned()
        }
    );
    let said = why.to_string();
    for word in [IN_A_SEALED_BOX, ASKING_EACH_TIME, AT_FULL_TRUST] {
        assert!(
            said.contains(word),
            "the refusal was {said:?}, which does not name {word}"
        );
    }
}

/// **A newer format is refused before the level is looked at.**
///
/// Asserted as *which* refusal rather than *that one happened*: the file below
/// has a newer format **and** a level this does not know, and only the format
/// may be reported. A reader of a file from a later alo OS is owed *this is
/// not for me*, not a complaint about a word whose meaning that alo OS defines.
#[test]
fn a_newer_format_is_refused_before_the_word_is_read() {
    let later = format!(
        "format = {}\nchose = \"a-level-invented-later\"\n",
        THE_LEVEL_FORMAT + 1
    );
    assert_eq!(
        read(&later),
        Err(NotTheLevel::AnotherFormat {
            format: THE_LEVEL_FORMAT + 1
        })
    );
}

/// Text that is not this file at all is refused as a shape, in the parser's
/// words.
#[test]
fn text_that_is_not_this_file_is_refused_as_a_shape() {
    assert!(matches!(
        read("this is not toml at all ["),
        Err(NotTheLevel::NotTheShape { .. })
    ));
    // A table of the right shape with the wrong type for `format`.
    assert!(matches!(
        read("format = \"one\"\n"),
        Err(NotTheLevel::NotTheShape { .. })
    ));
}

/// **It is kept in words a person can read, not as a number.**
///
/// `docs/contracts/how-code-runs-file.md` is a file a person may edit, over the
/// one question of how much of their machine an agent may use.
#[test]
fn it_is_kept_in_words_a_person_can_read() {
    let text = written(HowCodeRuns::AskingEachTime).expect("it writes");
    assert!(
        text.contains(ASKING_EACH_TIME),
        "it was written as {text:?}, which a person cannot read in their own settings"
    );
    assert!(
        text.contains(&format!("format = {THE_LEVEL_FORMAT}")),
        "it was written as {text:?}, with no format in it"
    );
}

/// The three words are distinct, and none is a prefix of another.
///
/// A prefix would make a hand edit that lost its last characters read as
/// another level rather than as a refusal.
#[test]
fn the_three_words_are_distinct_and_none_is_a_prefix_of_another() {
    let words = [IN_A_SEALED_BOX, ASKING_EACH_TIME, AT_FULL_TRUST];
    for (which, one) in words.iter().enumerate() {
        for (other_which, other) in words.iter().enumerate() {
            if which == other_which {
                continue;
            }
            assert!(
                !one.starts_with(other),
                "{one} starts with {other}, so a truncated edit reads as another level"
            );
        }
    }
    assert_eq!(
        words.len(),
        HowCodeRuns::ALL.len(),
        "there are {} levels and {} words for them",
        HowCodeRuns::ALL.len(),
        words.len()
    );
}
