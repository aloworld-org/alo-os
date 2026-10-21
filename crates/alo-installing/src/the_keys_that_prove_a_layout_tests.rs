//! That the four characters are the design's own, that they differ where the
//! layouts differ, and that the card is never empty.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::the_keys_that_prove_a_layout;

#[test]
fn belgisch_gives_the_four_the_design_frame_shows() {
    // **The anchor for every other assertion in this file.** The adopted frame
    // shows `a z ; m` on the Belgisch card. If the four positions were chosen
    // rather than derived, this is where that shows.
    assert_eq!(
        the_keys_that_prove_a_layout("be").as_deref(),
        Some("a z ; m"),
        "the design's own frame shows `a z ; m` for Belgisch"
    );
}

#[test]
fn an_azerty_and_a_qwerty_keyboard_do_not_look_alike() {
    // The mistake worth catching: a person with an AZERTY keyboard handed a
    // QWERTY guess cannot type their own password. If these two agreed, the
    // card would pass somebody straight into a field they cannot use.
    let azerty = the_keys_that_prove_a_layout("be").expect("Belgisch is offered");
    let qwerty = the_keys_that_prove_a_layout("us").expect("English (US) is offered");
    assert_ne!(
        azerty, qwerty,
        "an AZERTY and a QWERTY keyboard must not show the same four characters"
    );
}

#[test]
fn every_layout_offered_has_four_characters_to_show() {
    // A card that showed nothing would read as *press nothing to check*, which
    // is what the screen did before this existed. Every layout the environment
    // offers must produce four.
    let mut empty = Vec::new();
    for layout in alo_keyboards::what_a_key_gives::EVERY_LAYOUT {
        match the_keys_that_prove_a_layout(layout) {
            None => empty.push(format!("{layout}: no characters at all")),
            Some(shown) => {
                let how_many = shown.split(' ').filter(|s| !s.is_empty()).count();
                if how_many != 4 {
                    empty.push(format!(
                        "{layout}: {how_many} characters, not 4 ({shown:?})"
                    ));
                }
            }
        }
    }
    assert!(empty.is_empty(), "{}", empty.join("\n"));
}

#[test]
fn a_layout_this_environment_does_not_know_is_answered_rather_than_faked() {
    // **The control.** Every assertion above would hold on an implementation
    // that returned four characters for anything at all, including a layout
    // that does not exist — and then the card would prove nothing while
    // looking exactly the same.
    assert_eq!(
        the_keys_that_prove_a_layout("not-a-layout"),
        None,
        "a layout this environment does not know must not produce a check"
    );
}

#[test]
fn the_two_english_layouts_are_the_pair_this_cannot_tell_apart() {
    // **Written as a test rather than only as prose**, because it is a stated
    // limit of the card and a reader should find it failing if it ever stops
    // being true — at which point the sentence in this module's header is the
    // thing to correct.
    let uk = the_keys_that_prove_a_layout("gb").expect("English (UK) is offered");
    let us = the_keys_that_prove_a_layout("us").expect("English (US) is offered");
    assert_eq!(
        uk, us,
        "these two are documented as indistinguishable on this card; if they \
         now differ, the header saying so is wrong"
    );
}
