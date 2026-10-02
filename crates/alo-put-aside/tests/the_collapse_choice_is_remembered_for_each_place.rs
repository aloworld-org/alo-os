//! Task 1's owed clause: **the collapse choice persists per Place**.
//!
//! The clause that kept task 1 open after everything else in it was built. The choice was held
//! once for the whole panel, because there was no canvas Place to key it by — and the owner
//! ruled on 2026-09-30 that the canvas owns Place identity, so inventing one here would have
//! been the surface least entitled to define it doing so. `alo_canvas::Place` landed on
//! 2026-10-01 and this is the clause being paid rather than moved again.
//!
//! # Why every test here uses at least two Places
//!
//! A suite in which every assertion is about one Place is a suite that **cannot tell a
//! per-Place choice from a single field**. That is the same reason
//! `a_window_restored_from_another_place_travels_to_that_place.rs` uses Place 7 and looks at
//! Place 2, and the same reason the zoom fixture there is 1_750 rather than `LIFE_SIZE`: a
//! fixture that happens to equal the default proves nothing about whether the default was
//! consulted.
//!
//! So none of these uses `Place::FIRST` alone, and the one that checks the untouched case
//! checks it **while another Place is collapsed**.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — the same \
              exemption this crate's other suites take, with their words"
)]

use alo_canvas::{Place, Zoom};
use alo_dock::on_the_canvas::{Patch, Spot};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_put_aside::showing::{Chosen, HowItShows};
use alo_put_aside::the_collapse_choice_per_place::TheCollapseChoice;
use alo_put_aside::{Panel, Privacy};

/// Where the person is writing.
fn writing_on() -> Place {
    Place::numbered(7).expect("7 is not zero, so it is a Place")
}

/// Where they keep their reading. **Not** `writing_on`, and not `FIRST` either.
fn reading_on() -> Place {
    Place::numbered(2).expect("2 is not zero, so it is a Place")
}

/// A third, for the test that one choice does not spill into every other Place.
fn a_third_place() -> Place {
    Place::numbered(41).expect("41 is not zero, so it is a Place")
}

fn a_window(number: u64) -> Window {
    Window::of(
        WindowId::numbered(number),
        Some(AppId::named("Docs").expect("a fixture names its application")),
        "Launch strategy",
        Patch::of(Spot::at(0, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

fn a_panel_holding(how_many: u64) -> Panel {
    let mut panel = Panel::new();
    for which in 0..how_many {
        panel
            .put_aside(
                &a_window(which + 11),
                Zoom::LIFE_SIZE,
                writing_on(),
                Privacy::Ordinary,
            )
            .unwrap();
    }
    panel
}

/// **Collapsing one Place leaves every other Place expanded.**
///
/// The clause itself. A single field cannot pass this: collapsing would answer `Collapsed`
/// everywhere.
#[test]
fn collapsing_one_place_leaves_the_others_expanded() {
    let mut panel = a_panel_holding(3);
    panel.collapse(writing_on());

    assert_eq!(panel.chosen(writing_on()), Chosen::Collapsed);
    assert_eq!(
        panel.chosen(reading_on()),
        Chosen::Expanded,
        "collapsing one Place collapsed another"
    );
    assert_eq!(
        panel.chosen(a_third_place()),
        Chosen::Expanded,
        "collapsing one Place collapsed every Place"
    );
}

/// **What is drawn follows the choice on the Place being asked about.**
///
/// `chosen` is what the person picked and `showing` is what is painted, and the per-Place
/// keying has to reach the second or the first is a value nothing reads.
#[test]
fn what_is_drawn_differs_between_two_places_at_once() {
    let mut panel = a_panel_holding(3);
    panel.collapse(writing_on());

    assert_eq!(panel.showing(writing_on()), HowItShows::ARailOfIcons);
    assert_eq!(
        panel.showing(reading_on()),
        HowItShows::NamedPreviews,
        "one Place collapsed changed what another Place draws"
    );
}

/// **Two Places can be collapsed and expanded independently, in sequence.**
///
/// A person works on one surface, collapses there, moves, expands here, goes back. The choice
/// they made on the first Place is still theirs.
#[test]
fn a_choice_on_one_place_survives_choices_made_on_another() {
    let mut panel = a_panel_holding(2);

    panel.collapse(writing_on());
    panel.collapse(reading_on());
    panel.expand(reading_on());

    assert_eq!(
        panel.chosen(writing_on()),
        Chosen::Collapsed,
        "expanding one Place expanded another"
    );
    assert_eq!(panel.chosen(reading_on()), Chosen::Expanded);
}

/// **Expanding forgets rather than overwrites**, which is why absent means expanded.
///
/// Asserted through the count, because asking each Place separately cannot tell the two apart:
/// a map storing `Expanded` answers every question identically to a map with no entry at all.
/// The difference only shows in what is held — and it matters for anything that later writes
/// this down, which would otherwise carry a row per Place a person had merely looked at.
#[test]
fn expanding_a_place_forgets_it_rather_than_recording_the_default() {
    let mut panel = a_panel_holding(1);
    assert_eq!(panel.how_many_places_are_collapsed(), 0);

    panel.collapse(writing_on());
    panel.collapse(reading_on());
    assert_eq!(panel.how_many_places_are_collapsed(), 2);

    panel.expand(writing_on());
    assert_eq!(
        panel.how_many_places_are_collapsed(),
        1,
        "expanding stored the default instead of forgetting the Place"
    );

    panel.expand(reading_on());
    assert_eq!(panel.how_many_places_are_collapsed(), 0);
}

/// **A Place nobody has chosen on is expanded**, and that is the same answer a fresh panel
/// gives rather than a default invented for a missing entry.
///
/// Checked **while another Place is collapsed**, so it cannot pass by the map being empty.
#[test]
fn a_place_never_chosen_on_is_expanded_even_when_another_is_collapsed() {
    let mut panel = a_panel_holding(2);
    panel.collapse(writing_on());

    assert_eq!(panel.chosen(a_third_place()), Chosen::Expanded);
    assert_eq!(
        panel.showing(a_third_place()),
        HowItShows::NamedPreviews,
        "an untouched Place drew the collapsed form because another Place was collapsed"
    );
}

/// **Collapsing touches no window**, on any Place.
///
/// The owner's sixth behaviour rule, and the per-Place change must not weaken it. Asserted as
/// the absence — the whole list is unchanged — because a rule about what does *not* happen is
/// only held by a test whose subject is the absence.
#[test]
fn collapsing_any_place_leaves_every_window_alone() {
    let mut panel = a_panel_holding(3);
    let before: Vec<WindowId> = panel.previews().iter().map(|it| it.window()).collect();

    panel.collapse(writing_on());
    panel.collapse(reading_on());
    panel.expand(writing_on());

    let after: Vec<WindowId> = panel.previews().iter().map(|it| it.window()).collect();
    assert_eq!(before, after, "a collapse moved the windows");
    assert_eq!(panel.holding(), 3);
}

/// **The choice itself, asked of the type rather than through the panel.**
///
/// `TheCollapseChoice` is public because it is the thing that would be written down, so its own
/// behaviour is worth asserting without a panel in the way.
#[test]
fn the_choice_on_its_own_answers_expanded_for_anything_it_has_not_been_told() {
    let mut choice = TheCollapseChoice::new();
    assert_eq!(choice.on(writing_on()), Chosen::Expanded);
    assert_eq!(choice.how_many_are_collapsed(), 0);

    choice.collapse(writing_on());
    assert_eq!(choice.on(writing_on()), Chosen::Collapsed);
    assert_eq!(choice.on(reading_on()), Chosen::Expanded);
    assert_eq!(choice.how_many_are_collapsed(), 1);

    choice.expand(writing_on());
    assert_eq!(choice.on(writing_on()), Chosen::Expanded);
    assert_eq!(
        choice.how_many_are_collapsed(),
        0,
        "expanding kept a row for a Place in the state every Place starts in"
    );
}

/// **An empty panel's state is still per Place.**
///
/// A person who has put nothing aside sees the handle, and collapsing on one Place must not
/// change what another Place shows — the empty case is a state rather than a count, and the
/// per-Place keying has to reach it too.
#[test]
fn an_empty_panel_is_the_handle_on_every_place_whatever_was_chosen() {
    let mut panel = Panel::new();
    panel.collapse(writing_on());

    assert_eq!(panel.showing(writing_on()), HowItShows::AnEdgeHandle);
    assert_eq!(
        panel.showing(reading_on()),
        HowItShows::AnEdgeHandle,
        "an empty panel drew something other than the handle"
    );
}
