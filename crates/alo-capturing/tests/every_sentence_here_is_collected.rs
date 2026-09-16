//! The question this crate's own list cannot ask: whether what it says
//! survives being put beside everybody else's.
//!
//! `alo_capturing::words`' tests say the list is well formed. This says it is
//! **collected** — that the one vocabulary a translation is checked against
//! holds these thirteen keys, so a translator's line about a picture of the
//! screen is read as a line rather than as a mistake.
//!
//! It is worth its own file because the failure it catches is silent: a crate
//! that declares words and is not collected compiles, ships and says nothing to
//! anybody in any language. `crates/alo-collected` turned that into arithmetic
//! over the workspace; this is the same question asked from this crate's own
//! side, where whoever adds a word to `words.rs` is standing.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_saying::everything_this_machine_can_say;

/// **Every sentence this crate can say is in the machine's one vocabulary.**
#[test]
fn every_sentence_this_crate_can_say_is_in_the_machines_vocabulary() {
    let Ok(vocabulary) = everything_this_machine_can_say() else {
        panic!("alo OS's own words do not collect");
    };

    for word in alo_capturing::EVERY_WORD {
        let key = word.key();
        assert!(
            vocabulary.phrase(&key).is_some(),
            "{key} is declared here and collected nowhere"
        );
    }
}

/// **And the English in the vocabulary is this crate's English**, rather than a
/// key somebody else happened to declare first. A collision would leave a
/// person reading somebody else's sentence about their own screen.
#[test]
fn the_sentence_collected_is_the_sentence_this_crate_wrote() {
    let Ok(vocabulary) = everything_this_machine_can_say() else {
        panic!("alo OS's own words do not collect");
    };

    for word in alo_capturing::EVERY_WORD {
        let key = word.key();
        let Some(phrase) = vocabulary.phrase(&key) else {
            panic!("{key} is collected nowhere");
        };
        assert_eq!(phrase.source().as_written(), word.says(), "{key}");
    }
}

/// **No sentence this crate says names anything alo OS rents.** The media
/// server the picture comes through and the encoder it comes out of are rented
/// and never written (ADR 0011), and a person told *a picture of your screen
/// was saved* must not be asked to learn what either of them is called. The
/// plan's task 7 asks it of every sentence these crates say; this is the half
/// of it that exists already.
#[test]
fn no_sentence_this_crate_says_names_anything_we_rent() {
    let Ok(vocabulary) = everything_this_machine_can_say() else {
        panic!("alo OS's own words do not collect");
    };

    let overheard = alo_saying::what_a_person_would_have_to_learn(&vocabulary);
    let ours: Vec<String> = overheard
        .iter()
        .filter(|one| one.key().area() == "capturing")
        .map(|one| format!("{one}"))
        .collect();
    assert!(ours.is_empty(), "{ours:?}");
}

/// **What this crate rents is on the list of what alo OS rents**, with a line
/// saying what it is rented for. The rule `alo_saying::rented` states is that
/// adding a rented thing to this product means adding its name there in the
/// same change — and the check above is only worth anything if the name it is
/// looking for is on that list.
#[test]
fn what_this_crate_rents_is_named_as_rented() {
    let named: Vec<&str> = alo_saying::EVERYTHING_WE_RENT
        .iter()
        .map(alo_saying::Rented::name)
        .collect();
    for rented in ["PipeWire", "GStreamer"] {
        assert!(named.contains(&rented), "{rented} is not on {named:?}");
    }
}
