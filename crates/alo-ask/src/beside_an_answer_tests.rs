//! **The order is what these pin**, because the order is what made the fault of
//! 2026-10-18 invisible: *on this machine*, alone, reads as confirmation.

use super::*;
use alo_choosing::{Chosen, Which};

/// Everything this command can say, with nothing translated — so what comes
/// back is each word's source text and the assertion pins which word was
/// chosen rather than which key.
fn in_english() -> Strings {
    Strings::of(crate::what_it_can_say().expect("the four crates' lists declare"))
}

/// A provider the person chose, and the model they asked it for.
fn a_provider() -> Picked {
    Picked::FromAProvider {
        provider: "Mistral".to_owned(),
        model: "mistral-small-latest".to_owned(),
    }
}

/// **A person who chose a model here is told where it came from, and nothing
/// else.**
///
/// A caveat on every answer is a caveat on none of them.
#[test]
fn a_model_on_this_machine_gets_one_line_and_it_says_where() {
    let chosen = Picked::OnThisMachine(
        Chosen::of(Which::Catalogue, "smollm2-1.7b-instruct").expect("a model"),
    );
    let lines = beside_an_answer(&InferenceSource::ThisMachine, Some(&chosen), &in_english());

    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0].text(), "on this machine");
}

/// **A person who chose nothing gets one line too.**
///
/// They have not asked for anywhere, so nothing was substituted.
#[test]
fn a_person_who_chose_nothing_gets_one_line() {
    let lines = beside_an_answer(&InferenceSource::ThisMachine, None, &in_english());
    assert_eq!(lines.len(), 1, "{lines:?}");
}

/// **A person who chose a provider gets two lines, where first and then that
/// where is not what they asked for.**
///
/// This is the fault measured on 2026-10-18 turned into a test: settings
/// naming `mistral-small-latest` from Mistral, a local model answering, and
/// *on this machine* as the only hint.
#[test]
fn a_provider_that_was_not_used_adds_a_second_line_after_the_first() {
    let lines = beside_an_answer(
        &InferenceSource::ThisMachine,
        Some(&a_provider()),
        &in_english(),
    );

    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(
        lines[0].text(),
        "on this machine",
        "where it came from is said first"
    );
    assert_eq!(
        lines[1].text(),
        "you chose mistral-small-latest from Mistral, and this answer did not come from there"
    );
}

/// **Every line is a sentence the machine can say**, which is the failure that
/// reaches a person as a marked key.
#[test]
fn nothing_said_beside_an_answer_is_a_key() {
    for chosen in [None, Some(a_provider())] {
        for line in beside_an_answer(
            &InferenceSource::ThisMachine,
            chosen.as_ref(),
            &in_english(),
        ) {
            assert!(!line.is_a_bug(), "{line}");
            assert!(line.unfilled().is_empty(), "{line}");
        }
    }
}

/// **A person whose chosen provider really answered gets one line.**
///
/// The pairing that would be wrong the other way. Writing this test is what
/// caught the first version of the composition deciding the second line from
/// *what was chosen* alone, which would have told a person whose provider
/// answered them perfectly well that their choice was not used — the same
/// class of false statement as the silence it was written to fix.
///
/// Unreachable from this command today, which only ever builds
/// `InferenceSource::ThisMachine`, and asserted anyway: the composition is
/// correct for the day the provider road exists instead of becoming wrong on
/// it.
#[test]
fn a_provider_that_really_answered_gets_one_line_and_no_caveat() {
    let from_mistral = InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        // Which region does not matter here: what is compared is the
        // provider's name, and the variant carries a region.
        region: alo_models::Region::Unknown,
    };
    let lines = beside_an_answer(&from_mistral, Some(&a_provider()), &in_english());

    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        !lines[0].is_a_bug(),
        "where a hosted answer came from is a sentence: {}",
        lines[0]
    );
}
