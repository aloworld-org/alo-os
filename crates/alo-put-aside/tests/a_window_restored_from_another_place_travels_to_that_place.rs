//! The clause tasks 1 and 2 waited on all night: **minimising saves which Place**, and
//! restoring a window whose Place is not the one on screen travels to that Place rather than
//! dropping the window on this one.
//!
//! `alo_canvas::Place` landed on 2026-10-01. `where_it_goes_back.rs` was written in advance
//! for it, and said so: *what a person gets back is a view of the plane, not a rectangle, and
//! it is gaining a third member: the Place.*
//!
//! # Why every other test in this crate could not have caught this
//!
//! They all pass **one** Place. A suite in which every window is on the Place being looked at
//! is a suite that cannot tell a restore which checks the Place from one which ignores it — the
//! same reason the travel tests restore from a view that does not contain the window, and the
//! same reason the zoom fixture is 1_750 rather than `LIFE_SIZE`.
//!
//! So these tests put the window on Place 7 and look at Place 2.
//!
//! # The failure being guarded is a confident *nothing to do*
//!
//! `TheView::already_shows` compares rectangles, and **a rectangle does not know which surface
//! it is on**. `(4200, 0)` exists on every Place. So a view showing that patch on Place 2
//! answers *yes, already shown* about a window saved on Place 7, and a caller that asked the
//! geometry first is told no travel is needed — about a window that is not on this surface at
//! all.
//!
//! That is worse than travelling to the wrong place. It is **doing nothing and being told that
//! is correct**, while the screen agrees because something else occupies that rectangle.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — the same \
              exemption `alo-dock`'s own test modules take, with its words"
)]

use alo_canvas::{Place, Zoom};
use alo_dock::on_the_canvas::{Patch, Spot, TheView};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_dock::windows::Windows;
use alo_put_aside::putting_aside::put_aside;
use alo_put_aside::restoring::{Travel, restore, travel_for};
use alo_put_aside::{Panel, Privacy, WhereItGoesBack};

/// Where the window was put aside from.
fn its_place() -> Place {
    Place::numbered(7).expect("7 is not zero, so it is a Place")
}

/// Where the person is looking instead. **Not** `its_place`, and not `FIRST` either.
fn another_place() -> Place {
    Place::numbered(2).expect("2 is not zero, so it is a Place")
}

fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// The patch the window occupies — **the ambiguous one**, which exists on every Place.
fn the_ambiguous_patch() -> Patch {
    Patch::of(Spot::at(4_200, 0), 800, 600).expect("a fixture gives its patch an extent")
}

fn a_desk() -> Windows {
    let mut windows = Windows::none();
    windows.opened(Window::of(
        WindowId::numbered(1),
        // `Some`, because #356 made the application optional: a window whose client
        // named neither class nor title is now put aside carrying none rather than
        // refused. This fixture names one, because what it is testing is the Place and
        // a nameless window would be a second variable.
        Some(AppId::named("Docs").expect("a fixture names its application")),
        "Launch strategy",
        the_ambiguous_patch(),
        HowItSits::OnTheCanvas,
    ));
    windows
}

/// A view that **does** contain the ambiguous patch, so the geometry alone would say
/// *already shown*.
fn a_view_containing_it() -> TheView {
    TheView::showing(Patch::of(Spot::at(4_000, -200), 1_920, 1_080).expect("a view has extent"))
}

/// **The preview saves which Place**, which is the clause tasks 1 and 2 owed.
#[test]
fn a_preview_remembers_the_place_it_was_put_aside_from() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        its_place(),
        Privacy::Ordinary,
    )
    .unwrap();

    let preview = panel.previews().first().expect("one window is put aside");
    assert_eq!(preview.place(), its_place());
    assert_ne!(
        preview.place(),
        Place::FIRST,
        "the fixture's Place is the one a fresh machine looks at, so this test would pass \
         for a panel that saved none"
    );
    assert_eq!(preview.where_it_goes_back().place(), its_place());
}

/// **The whole saved view travels together** — patch, zoom and Place, in one value a caller
/// cannot take a third of.
#[test]
fn the_saved_view_carries_all_three_or_none() {
    let saved = WhereItGoesBack::of(the_ambiguous_patch(), a_zoom(), its_place());
    assert_eq!(saved.at(), the_ambiguous_patch());
    assert_eq!(saved.zoom(), a_zoom());
    assert_eq!(saved.place(), its_place());
    assert!(saved.is_on(its_place()));
    assert!(
        !saved.is_on(another_place()),
        "a view on Place 7 reported itself as being on Place 2"
    );
}

/// **A window on another Place travels to that Place, and the geometry is not consulted
/// first.**
///
/// The test the clause exists for. The view deliberately *does* contain the saved patch, so an
/// implementation that asked `already_shows` before the Place would answer `NotNeeded` — and
/// this asserts it does not.
#[test]
fn a_window_on_another_place_is_not_reported_as_already_shown() {
    let saved = WhereItGoesBack::of(the_ambiguous_patch(), a_zoom(), its_place());

    // The view contains the patch. On the right Place this would be `NotNeeded`.
    assert!(
        a_view_containing_it().already_shows(the_ambiguous_patch()),
        "the fixture must contain the patch, or this test proves nothing"
    );
    assert_eq!(
        travel_for(saved, a_view_containing_it(), its_place()),
        Travel::NotNeeded,
        "on its own Place, a view containing the patch needs no travel"
    );

    // The same geometry, a different Place.
    let what = travel_for(saved, a_view_containing_it(), another_place());
    assert_eq!(
        what,
        Travel::ToAnotherPlace(saved),
        "a window on Place 7 was judged by a rectangle on Place 2"
    );
    assert!(what.leaves_this_place());
    assert!(what.is_needed());
    assert_eq!(
        what.to()
            .expect("a travel that leaves a Place still says where")
            .place(),
        its_place(),
        "the travel does not name the Place to go to"
    );
}

/// **And the three outcomes are three**, so a caller cannot collapse them by accident.
#[test]
fn leaving_a_place_is_not_the_same_answer_as_panning() {
    let saved = WhereItGoesBack::of(the_ambiguous_patch(), a_zoom(), its_place());
    let elsewhere =
        TheView::showing(Patch::of(Spot::at(0, 0), 400, 300).expect("a view has extent"));

    let nothing = travel_for(saved, a_view_containing_it(), its_place());
    let pan = travel_for(saved, elsewhere, its_place());
    let leave = travel_for(saved, a_view_containing_it(), another_place());

    assert_ne!(nothing, pan);
    assert_ne!(pan, leave);
    assert_ne!(nothing, leave);

    assert!(!nothing.is_needed() && !nothing.leaves_this_place());
    assert!(pan.is_needed() && !pan.leaves_this_place());
    assert!(leave.is_needed() && leave.leaves_this_place());
}

/// **Restoring across Places works end to end**, and the window still comes back.
///
/// The owner's *cross-Place restoration*: bringing back a window whose Place is not the one
/// being looked at travels to that Place rather than dropping the window on this one. The
/// window's own state changes either way — it is restored, and the canvas is told where to go.
#[test]
fn restoring_from_another_place_brings_the_window_back_and_names_its_place() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        its_place(),
        Privacy::Ordinary,
    )
    .unwrap();

    let what = restore(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_view_containing_it(),
        another_place(),
    )
    .unwrap();

    assert_eq!(
        what,
        Travel::ToAnotherPlace(WhereItGoesBack::of(
            the_ambiguous_patch(),
            a_zoom(),
            its_place()
        ))
    );
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::OnTheCanvas,
        "the window is still put aside after being restored"
    );
    assert_eq!(panel.holding(), 0);
}

/// **Zero is not a Place**, which matters because of what writes a missing field.
///
/// `alo-arranging` carries `#[serde(default)]` and zero is exactly what serde invents for an
/// absent number. So anything persisted holds the raw number across the disk boundary and comes
/// back through `numbered` — and `numbered` refusing zero is what makes that road safe rather
/// than merely conventional.
#[test]
fn zero_is_refused_so_a_missing_field_cannot_become_a_place() {
    assert_eq!(Place::numbered(0), None);
    assert!(Place::numbered(1).is_some());
    assert_eq!(Place::FIRST.number(), 1);
}
