//! Task 3's acceptance, from `docs/autonomy/putting-a-window-aside.md`.
//!
//! One click returns a window to its **saved** position and the canvas travels to show it —
//! *not* to wherever the viewer happens to be.
//!
//! # Every test here restores from somewhere else on purpose
//!
//! *Restore it where it was* and *restore it where I am* agree whenever the window is on
//! screen already, which is most of the time. So the fixtures put the current view
//! **nowhere near** the window, because a test that restored from a view containing it
//! would pass for an implementation that ignored the saved value entirely.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — \
              the same exemption `alo-dock`'s own test modules take, with its words"
)]

use alo_canvas::{Place, Zoom};
use alo_dock::on_the_canvas::{Patch, Spot, TheView};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_dock::windows::Windows;
use alo_put_aside::putting_aside::put_aside;
use alo_put_aside::restoring::{Travel, restore, travel_for};
use alo_put_aside::{NotPutAside, Panel, Privacy, WhereItGoesBack};

/// The zoom the person was at — **never [`Zoom::LIFE_SIZE`]**.
///
/// Life size is `Zoom`'s own `Default`, so a fixture using it would let every assertion
/// here pass for an implementation that saved no zoom and built a default on the way out.
fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// The Place the window was on.
///
/// Deliberately not `Place::FIRST`, because FIRST is what a fresh machine looks at and a
/// fixture using it would pass for a panel that saved no Place and answered with the default.
/// The same reason the zoom fixture is 1_750 rather than LIFE_SIZE.
fn a_place() -> Place {
    Place::numbered(7).expect("7 is not zero, so it is a Place")
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

/// One window, far out on the plane at `x = 20_000`.
fn a_desk() -> Windows {
    let mut windows = Windows::none();
    windows.opened(window(1, "Docs", "Launch strategy", 20_000));
    windows
}

/// A view of the plane, wide enough to hold a window and then some.
fn view(x: i64) -> TheView {
    TheView::showing(Patch::of(Spot::at(x, -200), 1_920, 1_080).expect("a view has extent"))
}

/// **The restore names the saved patch and not the current view.**
///
/// The acceptance's own test, and the current view is at the plane's origin while the
/// window is twenty thousand units away — so an implementation that answered with
/// `showing` would be visibly wrong rather than accidentally right.
#[test]
fn restoring_from_somewhere_else_travels_to_the_saved_patch() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let was_at = windows.window(WindowId::numbered(1)).unwrap().at();

    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        a_place(),
        Privacy::Ordinary,
    )
    .unwrap();
    let travel = restore(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        view(0),
        a_place(),
    )
    .unwrap();

    let there = travel
        .to()
        .expect("the window is off screen, so a travel is needed");
    assert_eq!(
        there.at(),
        was_at,
        "the travel went to the current view rather than to the saved patch"
    );
    assert_eq!(
        there.zoom(),
        a_zoom(),
        "the travel kept the patch and lost the zoom"
    );
    assert_ne!(
        there.zoom(),
        Zoom::LIFE_SIZE,
        "the saved zoom is the default, so this test would pass for a panel saving none"
    );
}

/// **And the window is back on the canvas**, which is the other half of one click.
///
/// A travel decided for a window still marked put aside would be a canvas moving to show
/// something that is not there.
#[test]
fn the_window_is_on_the_canvas_again_and_the_panel_has_let_it_go() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        a_place(),
        Privacy::Ordinary,
    )
    .unwrap();

    restore(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        view(0),
        a_place(),
    )
    .unwrap();

    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::OnTheCanvas,
        "the canvas is travelling to a window that still says it is put aside"
    );
    assert_eq!(panel.holding(), 0, "and the panel is still holding it");
}

/// **A window the view already shows whole is not travelled to.**
///
/// The case a restore that always travels gets wrong: panning to a window already on
/// screen moves the person's view for nothing, and they did not ask for that.
#[test]
fn restoring_a_window_the_view_already_shows_needs_no_travel() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        a_place(),
        Privacy::Ordinary,
    )
    .unwrap();

    // The window sits at (20_000, 0) and is 800 by 600; this view contains it whole.
    let travel = restore(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        view(19_800),
        a_place(),
    )
    .unwrap();

    assert_eq!(travel, Travel::NotNeeded);
    assert!(!travel.is_needed());
    assert_eq!(
        travel.to(),
        None,
        "there is nowhere to go, and None says so where a saved view would not"
    );
}

/// **Partly on screen is travelled to, because partly is not readable.**
///
/// `TheView::already_shows` is *wholly inside* rather than *touching*, and its own file
/// gives the reason. This test exists so that loosening it is a decision somebody makes on
/// purpose rather than a behaviour that quietly changes: the window fits this view
/// horizontally and its **bottom two hundred units fall below it**, which is the case a
/// predicate about overlap would call visible.
///
/// The cost of the strict reading is a travel for a window that is nearly there. The cost
/// of the loose one is a click that appears to do nothing — and a person whose click
/// changes nothing concludes the click was lost, not that the window was already visible.
#[test]
fn a_window_only_partly_shown_is_still_travelled_to() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        a_place(),
        Privacy::Ordinary,
    )
    .unwrap();

    // Showing from x = 19_000 for 1_920 reaches x = 20_920; the window ends at 20_800 —
    // so it fits horizontally. The miss is vertical: this view is only 400 tall.
    let clipped =
        TheView::showing(Patch::of(Spot::at(19_000, 0), 1_920, 400).expect("a view has extent"));
    let travel = travel_for(
        WhereItGoesBack::of(a_patch(), a_zoom(), a_place()),
        clipped,
        a_place(),
    );

    assert!(
        travel.is_needed(),
        "a window with its bottom off the screen is a window a person cannot read"
    );
}

/// The window's own patch, as a value this file can hand to `travel_for` directly.
fn a_patch() -> Patch {
    Patch::of(Spot::at(20_000, 0), 800, 600).expect("a fixture gives its patch an extent")
}

/// **A refusal decides nothing about travelling.**
///
/// `Travel::NotNeeded` is a thing a caller acts on — it means *the window is back and the
/// canvas is right where it should be*. Returning it for a window the panel never held
/// would be a refusal a caller could mistake for a success, which is the whole reason the
/// answer is a `Result` around a `Travel` rather than a `Travel` with a third case.
#[test]
fn restoring_a_window_the_panel_does_not_hold_refuses_and_changes_nothing() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let before: Vec<Window> = windows.each().cloned().collect();

    assert_eq!(
        restore(
            &mut windows,
            &mut panel,
            WindowId::numbered(1),
            view(0),
            a_place()
        ),
        Err(NotPutAside::ItIsNotThere),
        "a window nobody put aside cannot be restored"
    );
    assert_eq!(
        windows.each().cloned().collect::<Vec<_>>(),
        before,
        "the refusal changed the desk, so it was not a refusal"
    );
    assert_eq!(panel.holding(), 0);
}

/// **The decision can be asked without moving anything.**
///
/// `travel_for` is separate from `restore` so that a panel drawing previews can say *this
/// one is over there* before anybody clicks. This checks the two agree, because a second
/// way of asking that answered differently would be worse than not having one.
#[test]
fn asking_whether_a_travel_is_needed_does_not_restore_the_window() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        a_place(),
        Privacy::Ordinary,
    )
    .unwrap();

    let goes_back_to = WhereItGoesBack::of(a_patch(), a_zoom(), a_place());
    let asked = travel_for(goes_back_to, view(0), a_place());

    assert!(asked.is_needed());
    assert_eq!(
        panel.holding(),
        1,
        "asking took the window out of the panel"
    );
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::PutAside,
        "asking put the window back on the canvas"
    );

    // And when it is actually restored, the answer is the same one.
    let done = restore(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        view(0),
        a_place(),
    )
    .unwrap();
    assert_eq!(
        done, asked,
        "asking and doing gave different answers about the same travel"
    );
}
