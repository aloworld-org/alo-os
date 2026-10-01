//! Task 4's acceptance, from `docs/autonomy/putting-a-window-aside.md`.
//!
//! > The intended placement is shown before anything moves, and the original position is
//! > kept in History. Nothing is rearranged without the proposal being visible first.
//!
//! # The clause this file exists for is the last one
//!
//! *Nothing is rearranged without the proposal being visible first* is a rule about what
//! does **not** happen, so the test for it compares the whole desk before and after and
//! insists nothing differs. A test that checked the proposal was correct would pass for an
//! implementation that restored the window first and proposed afterwards — which is
//! invisible stacking with a confirmation dialog's paperwork.
//!
//! # And the clause this file cannot finish
//!
//! *Kept in History* is not built. History is a `[v1]` feature and this panel is `[v0.01]`,
//! and `CLAUDE.md` gates building to what `docs/features.md` says. What is tested instead is
//! that **the original position is not lost** — it survives both roads, accepted and
//! dragged — so History has something true to read when it exists.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — \
              the same exemption `alo-dock`'s own test modules take, with its words. \
              `clippy::panic` is here for the let-else arms: a `Restored` that is not the \
              variant under test is this file's own mistake, and panicking with a sentence \
              says which variant arrived, where a `matches!` assertion would only say that \
              the shape was wrong"
)]

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::{Patch, Spot, TheView};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_dock::windows::Windows;
use alo_put_aside::proposing::Proposal;
use alo_put_aside::putting_aside::put_aside;
use alo_put_aside::restoring::Travel;
use alo_put_aside::restoring_into_a_taken_place::{Restored, accept, ask_for, dragged_to};
use alo_put_aside::shown::Shown;
use alo_put_aside::{NotPutAside, Panel, Privacy};

/// The zoom the person was at — never [`Zoom::LIFE_SIZE`], which is the default.
fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// A window of an application, called something, sitting somewhere.
fn window(id: u64, app: &str, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        Some(AppId::named(app).expect("a fixture names its application")),
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// Two windows: the one that will be put aside, and the one that will take its place.
fn a_desk() -> Windows {
    let mut windows = Windows::none();
    windows.opened(window(1, "Docs", "Launch strategy", 4_200));
    windows.opened(window(2, "Mail", "Anna", 9_000));
    windows
}

/// A patch somewhere else on the plane, which is what a *nearby position* stands in for.
fn an_offer() -> Patch {
    Patch::of(Spot::at(5_100, 0), 800, 600).expect("an offer has extent")
}

/// A view wide enough to show the saved place and the offer together.
fn a_wide_view() -> TheView {
    TheView::showing(Patch::of(Spot::at(4_000, -200), 3_000, 1_080).expect("a view has extent"))
}

/// **A taken place proposes, and the desk is untouched.**
///
/// The acceptance's own clause, and the subject is an absence. Every window is compared
/// whole — patch, state, title, application — and the panel is checked for still holding the
/// preview. An implementation that restored first and proposed second would pass a test
/// about the proposal's contents and fail this one.
#[test]
fn a_taken_place_proposes_and_moves_nothing() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let before: Vec<Window> = windows.each().cloned().collect();
    let previews_before = panel.previews().to_vec();

    let what = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_wide_view(),
        Some(WindowId::numbered(2)),
        an_offer(),
    )
    .unwrap();

    assert!(
        matches!(what, Restored::Proposed(_)),
        "a taken place restored the window instead of proposing"
    );
    assert_eq!(
        windows.each().cloned().collect::<Vec<_>>(),
        before,
        "something moved before the proposal was shown, which is the one thing forbidden"
    );
    assert_eq!(
        panel.previews(),
        previews_before.as_slice(),
        "the panel gave up the preview, so the window is half restored"
    );
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::PutAside,
        "the window says it is on the canvas while nothing has placed it"
    );
}

/// **A free place restores, exactly as before.**
///
/// The regression guard. Adding collisions must not change the uncontested road, and this is
/// the test that says so — it is the one that fails if `ask_for` starts proposing when
/// nothing is in the way.
#[test]
fn a_free_place_restores_and_says_where_to_travel() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let was_at = windows.window(WindowId::numbered(1)).unwrap().at();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    // A view at the plane's origin, nowhere near the window at x = 4_200.
    let elsewhere =
        TheView::showing(Patch::of(Spot::at(0, 0), 1_920, 1_080).expect("a view has extent"));
    let what = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        elsewhere,
        None,
        an_offer(),
    )
    .unwrap();

    let Restored::Travelled(travel) = what else {
        panic!("a free place proposed instead of restoring");
    };
    let there = travel
        .to()
        .expect("the window is off screen, so travel is needed");
    assert_eq!(there.at(), was_at, "it travelled somewhere other than home");
    assert_eq!(there.zoom(), a_zoom(), "and lost the saved zoom");
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::OnTheCanvas
    );
    assert_eq!(panel.holding(), 0);
}

/// **The proposal names both places, and the window in the way.**
///
/// A proposal that offered the position already taken would show a person the thing they
/// cannot have and call it a choice. And *something is here* is less use than *Anna is
/// here*: a named window can be pointed at, a count cannot.
#[test]
fn the_proposal_names_the_saved_place_the_offer_and_what_is_in_the_way() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let was_at = windows.window(WindowId::numbered(1)).unwrap().at();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let what = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_wide_view(),
        Some(WindowId::numbered(2)),
        an_offer(),
    )
    .unwrap();
    let Restored::Proposed(proposal) = what else {
        panic!("expected a proposal");
    };

    assert_eq!(proposal.saved(), was_at, "it forgot where the window lived");
    assert_eq!(proposal.offered(), an_offer());
    assert_eq!(proposal.taken_by(), WindowId::numbered(2));
    assert!(
        proposal.is_worth_showing(),
        "the offer is the saved place, so this proposal asks nothing"
    );
    assert!(
        proposal.can_be_seen_in(a_wide_view()),
        "an offer drawn off screen has not been shown, however carefully it was drawn"
    );
}

/// **An offer identical to the saved place is not worth showing**, and says so.
///
/// Reported rather than refused: what to do about a useless offer belongs to whoever chose
/// it, and a constructor that refused would have to decide what *nearby* means — which needs
/// a screen this crate cannot measure.
#[test]
fn an_offer_that_is_the_saved_place_asks_nothing() {
    let saved = Patch::of(Spot::at(4_200, 0), 800, 600).unwrap();
    let pointless = Proposal::of(WindowId::numbered(1), saved, saved, WindowId::numbered(2));
    assert!(!pointless.is_worth_showing());

    let useful = Proposal::of(
        WindowId::numbered(1),
        saved,
        an_offer(),
        WindowId::numbered(2),
    );
    assert!(useful.is_worth_showing());
}

/// **An offer outside the view has not been shown.**
///
/// The strict *wholly inside* reading again, for the reason `alo-dock` gives: a rectangle
/// with a corner on screen is one a person cannot read. An offer they cannot read is the same
/// failure as not offering.
#[test]
fn an_offer_off_the_screen_cannot_be_seen() {
    let saved = Patch::of(Spot::at(4_200, 0), 800, 600).unwrap();
    let far_away = Patch::of(Spot::at(80_000, 0), 800, 600).unwrap();
    let proposal = Proposal::of(
        WindowId::numbered(1),
        saved,
        far_away,
        WindowId::numbered(2),
    );

    assert!(
        !proposal.can_be_seen_in(a_wide_view()),
        "an offer eighty thousand units away was reported as visible"
    );
}

/// **Accepting places the window at the offer, and only then does its state change.**
///
/// And the saved position comes back with it, because that is the value History is owed and
/// accepting must not be what loses it.
#[test]
fn accepting_places_it_at_the_offer_and_keeps_where_it_was() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let was_at = windows.window(WindowId::numbered(1)).unwrap().at();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let Restored::Proposed(proposal) = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_wide_view(),
        Some(WindowId::numbered(2)),
        an_offer(),
    )
    .unwrap() else {
        panic!("expected a proposal");
    };

    let placed = accept(
        &mut windows,
        &mut panel,
        proposal,
        Shown::in_view(a_wide_view()),
    )
    .unwrap();

    assert_eq!(
        placed.at(),
        an_offer(),
        "it did not go where it was offered"
    );
    assert_eq!(
        placed.was_at(),
        was_at,
        "accepting lost the original position, which is what History is owed"
    );
    assert!(placed.moved(), "it says it did not move, and it did");
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::OnTheCanvas,
        "the window is placed and still says it is put aside"
    );
    assert_eq!(panel.holding(), 0, "and the panel is still holding it");
}

/// **Dragging places it where the person put it, not at the offer.**
///
/// *Let the person accept or drag* is two roads and the second is not a variation of the
/// first. A drag that landed on the offer would make the offer a decision rather than a
/// suggestion — and the original position survives this road too.
#[test]
fn dragging_places_it_where_the_person_said() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let was_at = windows.window(WindowId::numbered(1)).unwrap().at();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let Restored::Proposed(proposal) = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_wide_view(),
        Some(WindowId::numbered(2)),
        an_offer(),
    )
    .unwrap() else {
        panic!("expected a proposal");
    };

    let theirs = Patch::of(Spot::at(6_400, 0), 800, 600).expect("a drop has extent");
    let placed = dragged_to(
        &mut windows,
        &mut panel,
        proposal,
        Shown::in_view(a_wide_view()),
        theirs,
    )
    .unwrap();

    assert_eq!(placed.at(), theirs, "a drag landed on the offer");
    assert_ne!(
        placed.at(),
        an_offer(),
        "the offer decided where the window went, and it is a suggestion"
    );
    assert_eq!(
        placed.was_at(),
        was_at,
        "dragging lost the original position"
    );
}

/// **A refusal proposes nothing and changes nothing.**
///
/// `Restored::Proposed` is something a caller acts on — it means *show this to a person*. A
/// refusal that also handed back a proposal would be a refusal a caller could act on, which
/// is why the answer is a `Result` around `Restored` rather than a third case inside it.
#[test]
fn asking_for_a_window_the_panel_does_not_hold_refuses_and_proposes_nothing() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let before: Vec<Window> = windows.each().cloned().collect();

    assert_eq!(
        ask_for(
            &mut windows,
            &mut panel,
            WindowId::numbered(1),
            a_wide_view(),
            Some(WindowId::numbered(2)),
            an_offer(),
        ),
        Err(NotPutAside::ItIsNotThere)
    );
    assert_eq!(
        windows.each().cloned().collect::<Vec<_>>(),
        before,
        "the refusal changed the desk, so it was not a refusal"
    );
    assert_eq!(panel.holding(), 0);
}

/// **A window that left the panel between the showing and the answer refuses.**
///
/// This happens honestly: a person is shown a proposal, and something else closes or
/// restores that window before they answer. Accepting then has nothing to place, and the
/// refusal is what stops it inventing a window.
#[test]
fn accepting_after_the_window_has_gone_refuses() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let Restored::Proposed(proposal) = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_wide_view(),
        Some(WindowId::numbered(2)),
        an_offer(),
    )
    .unwrap() else {
        panic!("expected a proposal");
    };

    // Somebody else brings it back while the person is still looking at the proposal.
    panel.bring_back(WindowId::numbered(1)).unwrap();

    assert_eq!(
        accept(
            &mut windows,
            &mut panel,
            proposal,
            Shown::in_view(a_wide_view())
        ),
        Err(NotPutAside::ItIsNotThere),
        "it placed a window the panel no longer held"
    );
}

/// **A free place with the window already on screen needs no travel**, so the two outcomes
/// of `Restored::Travelled` are both reachable through this door and not only one.
#[test]
fn a_free_place_already_on_screen_travels_nowhere() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let what = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_wide_view(),
        None,
        an_offer(),
    )
    .unwrap();

    assert_eq!(what, Restored::Travelled(Travel::NotNeeded));
}
