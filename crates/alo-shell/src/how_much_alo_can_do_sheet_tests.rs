//! **The sheet draws its three cards, its one action and nothing else a press
//! could land on**, at every size the first start is drawn at.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::{Scheme, TextScale};

use super::{HowMuchSheet, drawn};
use crate::a_press_on_a_setup_sheet::OnASheet;
use crate::display_scale::DisplayScale;
use crate::how_much_alo_can_do::{HowMuchAloCanDo, ToAsk};
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::sign_in_surface::TheColours;

/// The sheet for a person who has not chosen, and the folder it would write in.
fn the_sheet() -> (tempfile::TempDir, HowMuchAloCanDo) {
    let at = tempfile::tempdir().expect("a temporary directory");
    match HowMuchAloCanDo::to_ask(&at.path().join("how-code-runs.toml")) {
        ToAsk::Ask(it) => (at, it),
        other => panic!("not asked: {other:?}"),
    }
}

/// The sheet drawn on a display of `size` at `per_cent`.
fn drawn_at(
    asking: &HowMuchAloCanDo,
    size: (i32, i32),
    per_cent: u16,
    the_sealed_box_runs_here: bool,
) -> SignInPicture {
    let mut labels = crate::WindowControlLabels::new().expect("the font");
    let words = crate::what_this_machine_can_say().expect("the words");
    drawn(
        &HowMuchSheet {
            asking,
            the_sealed_box_runs_here,
        },
        &mut labels.fonts,
        &words,
        size,
        TheColours::of(crate::Contrast::AsDesigned, Scheme::Light),
        &Measure::at(TextScale::ordinary(), DisplayScale::of_per_cent(per_cent)),
    )
    .unwrap_or_else(|why| panic!("{size:?} at {per_cent}%: {why}"))
}

/// **Three cards, Continue and *Accessibility* — and no way back**, which the
/// owner's *"Continue only"* forbids drawing; cards that never overlap, each at
/// least as tall as an ordinary card.
#[test]
fn three_cards_continue_and_no_way_back_at_every_size_drawn() {
    let (_at, asking) = the_sheet();
    for (size, per_cent) in [
        ((1440, 960), 100),
        ((1280, 800), 100),
        ((1280, 720), 100),
        ((1366, 768), 100),
        ((640, 700), 100),
        ((2880, 1920), 200),
    ] {
        for here in [true, false] {
            let picture = drawn_at(&asking, size, per_cent, here);
            let choices: Vec<_> = picture
                .controls
                .iter()
                .filter(|it| matches!(it.what, OnASheet::Choice(_)))
                .collect();
            assert_eq!(choices.len(), 3, "{size:?}: {:?}", picture.controls);
            for (which, it) in choices.iter().enumerate() {
                assert_eq!(it.what, OnASheet::Choice(which), "{size:?}");
                let at_one = it.area.size.h * 100 / i32::from(per_cent);
                assert!(at_one >= 85, "{size:?}: card {which} is {at_one} tall");
            }
            for pair in choices.windows(2) {
                if let [above, below] = pair {
                    assert!(
                        above.area.loc.y + above.area.size.h <= below.area.loc.y,
                        "{size:?}: two cards overlap"
                    );
                }
            }
            let ways_on = picture
                .controls
                .iter()
                .filter(|it| it.what == OnASheet::WayOn)
                .count();
            assert_eq!(ways_on, 1, "{size:?}");
            assert!(
                !picture
                    .controls
                    .iter()
                    .any(|it| it.what == OnASheet::WayBack),
                "{size:?}: a way back is drawn"
            );
            assert!(
                picture
                    .controls
                    .iter()
                    .any(|it| it.what == OnASheet::Accessibility),
                "{size:?}"
            );
        }
    }
}

/// **The cards grow to their words**: the cost of full access is two lines
/// at the design's size, so its card is taller than the 85 an ordinary card
/// is — the sentence is never cut off.
#[test]
fn the_cards_grow_to_hold_the_cost_of_full_access() {
    let (_at, asking) = the_sheet();
    let picture = drawn_at(&asking, (1440, 960), 100, true);
    let full = picture
        .controls
        .iter()
        .find(|it| it.what == OnASheet::Choice(2))
        .expect("the full access card");
    assert!(full.area.size.h > 85, "{}", full.area.size.h);
}

/// **A screen too small for all three is refused, never drawn with one out of
/// sight.**
#[test]
fn a_screen_too_small_for_all_three_is_refused() {
    let (_at, asking) = the_sheet();
    let mut labels = crate::WindowControlLabels::new().expect("the font");
    let words = crate::what_this_machine_can_say().expect("the words");
    let refused = drawn(
        &HowMuchSheet {
            asking: &asking,
            the_sealed_box_runs_here: true,
        },
        &mut labels.fonts,
        &words,
        (639, 699),
        TheColours::of(crate::Contrast::AsDesigned, Scheme::Light),
        &Measure::at(TextScale::ordinary(), DisplayScale::ONE),
    );
    assert!(refused.is_err());
}
