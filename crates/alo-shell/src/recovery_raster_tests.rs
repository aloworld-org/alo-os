//! The recovery screen drawn: the whole sentence or no frame at all, two
//! moments one above the other with neither preselected, and a selection told
//! apart by two shapes rather than a colour.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::Token;

use super::*;
use crate::recovery_keys::RecoveryKey;
use crate::recovery_screen::{RecoveryChosen, RecoveryScreen};
use crate::recovery_testing::{
    a_light_look, a_machine_that_can_go_back, a_machine_with_nothing_before, words,
};

/// An ordinary laptop output.
const A_LAPTOP: (i32, i32) = (1366, 768);

/// The screen for a machine that can go back, with `moves` presses of Tab
/// already made.
fn a_screen(strings: &alo_strings::Strings, moves: usize) -> RecoveryScreen {
    let mut screen = RecoveryScreen::of(&a_machine_that_can_go_back(), None, strings);
    for _ in 0..moves {
        screen = match screen.pressed(RecoveryKey::Next) {
            RecoveryChosen::Still(screen) => *screen,
            RecoveryChosen::GoBack { .. } => panic!("moving chose"),
        };
    }
    screen
}

/// That screen drawn on an output of `size`.
fn drawn(screen: &RecoveryScreen, size: (i32, i32)) -> Result<RecoveryPicture, RenderError> {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(screen.shows(), &mut labels, size, a_light_look())
}

/// **The sentence is drawn whole, byte for byte, or the frame is refused.**
/// Half a sentence about replacing the operating system is a different claim
/// from the whole of it, so an output that cannot hold it draws nothing at all.
#[test]
fn the_sentence_is_drawn_whole_or_the_frame_is_refused() {
    let strings = words();
    let screen = a_screen(&strings, 0);
    let RecoveryShows::Offered { offer, .. } = screen.shows() else {
        panic!("this machine can go back");
    };

    let picture = drawn(&screen, A_LAPTOP).unwrap();
    assert_eq!(picture.sentence.0, offer.text());
    assert_eq!(picture.size, A_LAPTOP);
    assert!(picture.panel.size.w > 0 && picture.panel.size.h > 0);
    assert!(picture.sentence.1.size.h > 0);
    assert!(!picture.inked.is_empty());

    for too_small in [(1366, 40), (120, 768), (20_000, 768), (1366, 20_000)] {
        assert!(
            matches!(drawn(&screen, too_small), Err(RenderError::RecoveryScene)),
            "{too_small:?} was drawn anyway"
        );
    }
}

/// **Both moments are drawn, one above the other, neither selected.** They are
/// the same width and the same height, so neither looks like the answer the
/// machine would prefer, and they are in the order a person Tabs through them.
#[test]
fn both_moments_are_drawn_one_above_the_other_with_neither_selected() {
    let strings = words();
    let picture = drawn(&a_screen(&strings, 0), A_LAPTOP).unwrap();
    assert_eq!(picture.moments.len(), 2);
    let first = picture.moments.first().unwrap();
    let second = picture.moments.last().unwrap();

    assert_eq!(first.area.size, second.area.size);
    assert_eq!(first.area.loc.x, second.area.loc.x);
    assert!(
        second.area.loc.y >= first.area.loc.y + first.area.size.h,
        "the two moments overlap"
    );
    assert!(!first.selected && !second.selected);
    let order = crate::recovery_screen::THE_TWO_MOMENTS;
    assert_eq!(Some(first.when), order.first().copied());
    assert_eq!(Some(second.when), order.last().copied());
    assert!(!first.words.is_empty() && !second.words.is_empty());
    assert_ne!(first.words, second.words);
}

/// **A selected moment is told apart by two shapes, never by a colour.** The
/// panel draws in exactly the same two colours whether something is selected or
/// not; what changes is a thicker edge **and** a bar under the words — which is
/// what EN 301 549 asks for and what a person who cannot tell one hue from
/// another needs.
#[test]
fn a_selected_moment_is_told_apart_by_two_shapes_and_not_by_a_colour() {
    let strings = words();
    let nothing_selected = drawn(&a_screen(&strings, 0), A_LAPTOP).unwrap();
    let first_selected = drawn(&a_screen(&strings, 1), A_LAPTOP).unwrap();

    let colours = |picture: &RecoveryPicture| {
        let mut seen: Vec<[u8; 3]> = picture.solids.iter().map(|solid| solid.colour).collect();
        seen.sort_unstable();
        seen.dedup();
        seen
    };
    assert_eq!(
        colours(&nothing_selected),
        colours(&first_selected),
        "the selection introduced a colour of its own"
    );

    assert!(first_selected.moments.first().unwrap().selected);
    assert!(!first_selected.moments.last().unwrap().selected);
    assert!(
        first_selected.solids.len() > nothing_selected.solids.len(),
        "a selected moment added no shape"
    );
}

/// **A machine that cannot go back draws its sentence and no moment at all.**
/// Not a greyed-out offer: a control that cannot be used is a promise nobody
/// made.
#[test]
fn a_machine_that_cannot_go_back_draws_no_moment_at_all() {
    let strings = words();
    let screen = RecoveryScreen::of(&a_machine_with_nothing_before(), None, &strings);
    let RecoveryShows::CannotGoBack(said) = screen.shows() else {
        panic!("this machine has nothing to go back to");
    };
    let picture = drawn(&screen, A_LAPTOP).unwrap();
    assert_eq!(picture.sentence.0, said.text());
    assert!(picture.moments.is_empty());
    assert!(!picture.inked.is_empty());
}

/// **Not one pixel of the recovery screen is terracotta.** Terracotta means the
/// agent, and no agent is involved in a person getting their own machine back.
#[test]
fn not_one_pixel_of_the_recovery_screen_is_terracotta() {
    let strings = words();
    let terracotta = Token::Terracotta.colour();
    let terracotta = [terracotta.red(), terracotta.green(), terracotta.blue()];
    for scheme in [alo_appearance::Scheme::Light, alo_appearance::Scheme::Dark] {
        for moves in 0..3 {
            let screen = a_screen(&strings, moves);
            let mut labels = WindowControlLabels::new().unwrap();
            let look = RecoveryLook {
                contrast: Contrast::AsDesigned,
                scheme,
                ..a_light_look()
            };
            let picture = picture(screen.shows(), &mut labels, A_LAPTOP, look).unwrap();
            for solid in &picture.solids {
                assert_ne!(solid.colour, terracotta, "{scheme:?}");
            }
            for inked in &picture.inked {
                assert!(!inked.pixels.contains(&terracotta), "{scheme:?}");
            }
        }
    }
}
