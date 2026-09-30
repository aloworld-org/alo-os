//! Task 2's acceptance, from `docs/autonomy/putting-a-window-aside.md`.
//!
//! Putting a window aside changes that window and the panel, and **nothing else**. A
//! round trip returns the window to the patch it left, because it is the same patch
//! rather than two calculations that agreed.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — \
              the same exemption `alo-dock`'s own test modules take, with its words"
)]

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::{Patch, Spot};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_dock::windows::Windows;
use alo_put_aside::putting_aside::{bring_back, put_aside};
use alo_put_aside::{NotPutAside, Panel};

/// The zoom a person happened to be at — **deliberately not life size**.
///
/// [`Zoom::LIFE_SIZE`] is the default, so a test that saved it would pass for a panel that
/// stored no zoom at all and built a default on the way out. The value has to be one
/// nothing else in the system would produce by accident.
fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// A window of an application, called something, sitting somewhere.
fn window(id: u64, app: &str, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        AppId::named(app).expect("a fixture names its application"),
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// Three windows on a canvas, spaced apart.
fn a_desk() -> Windows {
    let mut windows = Windows::none();
    windows.opened(window(1, "Docs", "Launch strategy", 0));
    windows.opened(window(2, "Mail", "Anna", 1_000));
    windows.opened(window(3, "Browser", "The wiki", 2_000));
    windows
}

/// Every window except the named one, as a comparable list.
fn the_others(windows: &Windows, except: u64) -> Vec<Window> {
    windows
        .each()
        .filter(|w| w.id() != WindowId::numbered(except))
        .cloned()
        .collect()
}

/// **The other windows do not move to fill the gap.**
///
/// The owner's sentence, and the subject of this test is an **absence**. It compares
/// every other window whole — patch, state, title, application — rather than checking
/// that the canvas looks reasonable afterwards, because the second passes for an
/// implementation that moved something and moved it back.
///
/// A canvas that closed up after a minimise would move things a person had placed, and
/// placing is the whole of what a canvas is for.
#[test]
fn putting_one_aside_moves_no_other_window() {
    let mut windows = a_desk();
    let mut panel = Panel::new();

    let before = the_others(&windows, 2);

    put_aside(&mut windows, &mut panel, WindowId::numbered(2), a_zoom()).unwrap();

    assert_eq!(
        the_others(&windows, 2),
        before,
        "a window that was not put aside changed, and none may"
    );
    assert_eq!(windows.how_many(), 3, "and none was lost or gained");
}

/// **The window that was put aside says so, and the panel holds it.**
///
/// Two halves of one change, asserted separately: a window marked put aside with no
/// preview is one a person cannot see and cannot get back, and a preview whose window
/// still says it is on the canvas is a window in two places.
#[test]
fn the_window_leaves_the_canvas_and_the_panel_holds_it() {
    let mut windows = a_desk();
    let mut panel = Panel::new();

    put_aside(&mut windows, &mut panel, WindowId::numbered(2), a_zoom()).unwrap();

    let it = windows.window(WindowId::numbered(2)).unwrap();
    assert_eq!(
        it.sits(),
        HowItSits::PutAside,
        "the window still says it is on the canvas"
    );
    assert!(
        panel.holds(WindowId::numbered(2)),
        "the panel does not hold the window that just left the canvas"
    );
    assert_eq!(panel.holding(), 1);
}

/// **A round trip lands on the view it left, byte for byte.**
///
/// Not *near* it and not *recomputed to* it. The panel gives back the patch and the zoom
/// it was handed, so this passes because they are the same values rather than because two
/// calculations agreed — which is the only version of this that stays true when the canvas
/// learns to do something else.
///
/// The zoom is asserted separately from the patch, and against life size as well. A window
/// restored to the right rectangle at the wrong zoom is the failure that looks like a bug
/// in the window and is a bug in what was saved; one combined assertion would let either
/// half carry the other.
#[test]
fn a_window_comes_back_to_the_view_it_left() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let was_at = windows.window(WindowId::numbered(2)).unwrap().at();

    put_aside(&mut windows, &mut panel, WindowId::numbered(2), a_zoom()).unwrap();
    let goes_to = bring_back(&mut windows, &mut panel, WindowId::numbered(2)).unwrap();

    assert_eq!(goes_to.at(), was_at, "it came back somewhere else");
    assert_eq!(
        goes_to.zoom(),
        a_zoom(),
        "it came back at a different zoom from the one it was put away at"
    );
    assert_ne!(
        goes_to.zoom(),
        Zoom::LIFE_SIZE,
        "the fixture's zoom is life size, so this test would also pass for a panel that \
         saved no zoom and built a default on the way out"
    );
    assert_eq!(
        windows.window(WindowId::numbered(2)).unwrap().sits(),
        HowItSits::OnTheCanvas,
        "it is back and still says it was put aside"
    );
    assert_eq!(panel.holding(), 0);
}

/// **Minimising discards nothing**, and the whole desk says so.
///
/// Every window is compared before and after a full round trip of one of them. Nothing
/// about the other two may differ, and the one that travelled must be identical to what
/// it was — including its patch, which is the part *never discards work* is about.
#[test]
fn a_round_trip_leaves_the_whole_desk_as_it_was() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let before: Vec<Window> = windows.each().cloned().collect();

    put_aside(&mut windows, &mut panel, WindowId::numbered(3), a_zoom()).unwrap();
    bring_back(&mut windows, &mut panel, WindowId::numbered(3)).unwrap();

    let after: Vec<Window> = windows.each().cloned().collect();
    assert_eq!(
        after, before,
        "a window came back different from how it went away"
    );
}

/// **A window that does not exist and a window the panel does not hold are different
/// mistakes.**
///
/// Both are `ItIsNotThere` today, and the two calls are what distinguish them: asking to
/// put aside something that was never opened, against asking to bring back something
/// nobody put away.
#[test]
fn the_two_ways_of_asking_about_a_window_that_is_not_there() {
    let mut windows = a_desk();
    let mut panel = Panel::new();

    assert_eq!(
        put_aside(&mut windows, &mut panel, WindowId::numbered(9), a_zoom()),
        Err(NotPutAside::ItIsNotThere),
        "putting aside a window nobody opened"
    );
    assert_eq!(
        bring_back(&mut windows, &mut panel, WindowId::numbered(1)),
        Err(NotPutAside::ItIsNotThere),
        "bringing back a window that was never put aside"
    );
    assert_eq!(windows.how_many(), 3, "and neither touched the desk");
}

/// **A window is not put aside twice, and the refusal leaves the state alone.**
///
/// The refusal is not the evidence; the state is. A refusal that had already changed
/// `HowItSits` would be the fault wearing a refusal's words.
#[test]
fn the_second_attempt_refuses_and_changes_nothing() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(&mut windows, &mut panel, WindowId::numbered(1), a_zoom()).unwrap();

    let before: Vec<Window> = windows.each().cloned().collect();
    assert_eq!(
        put_aside(&mut windows, &mut panel, WindowId::numbered(1), a_zoom()),
        Err(NotPutAside::ItIsAlreadyThere)
    );

    assert_eq!(
        windows.each().cloned().collect::<Vec<_>>(),
        before,
        "the refusal changed the desk, so it was not a refusal"
    );
    assert_eq!(panel.holding(), 1, "and did not add a second preview");
}
