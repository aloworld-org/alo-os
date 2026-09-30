//! Task 5's acceptance, from `docs/autonomy/putting-a-window-aside.md`.
//!
//! > Peek leaves the window's state and the camera both unchanged, asserted separately. A
//! > press meant for the canvas does not reach the peek — tested through the real input path,
//! > and using the shell's existing hit test rather than a second opinion about where clients
//! > are.
//!
//! # Asserted separately, because one assertion would let either half carry the other
//!
//! The window's state is this crate's own data and is checked directly. The camera is held by
//! the shape — `peek_at` and `stop_peeking` take no view — and the check for it reads the
//! module's **source** for a `Travel` it must never build, because a test comparing a view
//! across a call that cannot touch one could not fail.
//!
//! # The clause this file cannot finish
//!
//! *Tested through the real input path, using the shell's existing hit test.* `alo-shell`
//! belongs to another lane. The clause stays in task 5 rather than moving somewhere it could be
//! ticked, and the task stays open — the same division as task 6, where the wiring and the
//! integration evidence are lane B's.
//!
//! The acceptance also names the fault to avoid in advance — *rather than a second opinion
//! about where clients are* — which is the same reason the overlap predicate for task 4 was
//! asked of `alo-dock` instead of being written here.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — the same \
              exemption `alo-dock`'s own test modules take, with its words. `clippy::panic` is \
              for the let-else arms, which name which variant arrived"
)]

use std::path::Path;

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::{Patch, Spot, TheView};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_dock::windows::Windows;
use alo_put_aside::peeking_at_a_preview::{chosen, peek_at, stop_peeking};
use alo_put_aside::putting_aside::put_aside;
use alo_put_aside::restoring::Travel;
use alo_put_aside::restoring_into_a_taken_place::Restored;
use alo_put_aside::{NotPutAside, Panel, Privacy};

/// The zoom the person was at — never [`Zoom::LIFE_SIZE`], which is the default.
fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

fn window(id: u64, app: &str, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        AppId::named(app).expect("a fixture names its application"),
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// Two windows, so that one can be put aside and the other can take its place.
fn a_desk() -> Windows {
    let mut windows = Windows::none();
    windows.opened(window(1, "Docs", "Launch strategy", 4_200));
    windows.opened(window(2, "Mail", "Anna", 9_000));
    windows
}

fn a_view() -> TheView {
    TheView::showing(Patch::of(Spot::at(4_000, -200), 3_000, 1_080).expect("a view has extent"))
}

fn an_offer() -> Patch {
    Patch::of(Spot::at(5_100, 0), 800, 600).expect("an offer has extent")
}

/// A desk with window 1 put aside.
fn a_desk_with_one_put_aside() -> (Windows, Panel) {
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
    (windows, panel)
}

/// **Peeking leaves the window's state alone**, and the whole desk with it.
///
/// The first half of *both unchanged*, asserted on the window itself rather than on the panel's
/// count — a count is the same number for a panel holding the right preview and one holding a
/// preview of something already restored.
#[test]
fn peeking_changes_no_window_and_no_preview() {
    let (windows, panel) = a_desk_with_one_put_aside();
    let before: Vec<Window> = windows.each().cloned().collect();
    let previews_before = panel.previews().to_vec();

    let peeking = peek_at(&panel, WindowId::numbered(1)).unwrap();

    assert!(peeking.is_peeking());
    assert_eq!(peeking.at(), Some(WindowId::numbered(1)));
    assert_eq!(
        windows.each().cloned().collect::<Vec<_>>(),
        before,
        "a look altered what it looked at"
    );
    assert_eq!(panel.previews(), previews_before.as_slice());
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::PutAside,
        "peeking un-minimised the window"
    );
}

/// **Letting go leaves the window minimised**, which is the owner's own sentence.
#[test]
fn letting_go_ends_the_peek_and_the_window_stays_minimised() {
    let (windows, panel) = a_desk_with_one_put_aside();
    let peeking = peek_at(&panel, WindowId::numbered(1)).unwrap();

    let after = stop_peeking(peeking);

    assert!(!after.is_peeking(), "the peek did not end");
    assert_eq!(after.at(), None);
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::PutAside,
        "letting go of a peek brought the window back, and it must stay minimised"
    );
    assert_eq!(panel.holding(), 1, "and the panel let go of the preview");
}

/// **Choosing a peeked preview goes through the restore road**, so a taken place is proposed.
///
/// This is the test the module exists for. `PeekEnded::ByTravellingThere` names a travel, and
/// for a put-aside window the ordinary road is a restore that may find its place occupied. An
/// implementation that followed the variant's name would travel, skip the proposal, and put the
/// window back invisibly behind whatever is there — the one thing task 4 forbids.
#[test]
fn choosing_a_peeked_preview_proposes_when_its_place_is_taken() {
    let (mut windows, mut panel) = a_desk_with_one_put_aside();
    let peeking = peek_at(&panel, WindowId::numbered(1)).unwrap();

    let (what, after) = chosen(
        &mut windows,
        &mut panel,
        peeking,
        a_view(),
        Some(WindowId::numbered(2)),
        an_offer(),
    )
    .unwrap();

    let Restored::Proposed(proposal) = what else {
        panic!(
            "choosing a peeked preview travelled instead of restoring, so the collision \
                was never shown"
        );
    };
    assert_eq!(proposal.window(), WindowId::numbered(1));
    assert_eq!(proposal.taken_by(), WindowId::numbered(2));
    assert!(!after.is_peeking(), "the peek outlived the choice");
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::PutAside,
        "a proposal moved the window, and nothing may move before it is shown"
    );
}

/// **And a free place restores through the same road**, so choosing is not two behaviours.
#[test]
fn choosing_a_peeked_preview_restores_when_its_place_is_free() {
    let (mut windows, mut panel) = a_desk_with_one_put_aside();
    let was_at = Patch::of(Spot::at(4_200, 0), 800, 600).unwrap();
    let peeking = peek_at(&panel, WindowId::numbered(1)).unwrap();

    let (what, after) = chosen(
        &mut windows,
        &mut panel,
        peeking,
        a_view(),
        None,
        an_offer(),
    )
    .unwrap();

    assert_eq!(what, Restored::Travelled(Travel::NotNeeded));
    assert!(!after.is_peeking());
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::OnTheCanvas
    );
    assert_eq!(panel.holding(), 0);
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().at(),
        was_at,
        "it came back somewhere other than where it was"
    );
}

/// **A peek at a window the panel does not hold is refused**, not begun and empty.
///
/// `Peeking::is_peeking` would otherwise answer *yes* about a window nobody can see, which is a
/// state a caller would draw.
#[test]
fn peeking_at_a_window_nobody_put_aside_is_refused() {
    let (_windows, panel) = a_desk_with_one_put_aside();

    assert_eq!(
        peek_at(&panel, WindowId::numbered(2)),
        Err(NotPutAside::ItIsNotThere),
        "a window on the canvas was peeked at as though the panel held it"
    );
    assert_eq!(
        peek_at(&panel, WindowId::numbered(9)),
        Err(NotPutAside::ItIsNotThere),
        "a window that does not exist was peeked at"
    );
}

/// **Choosing when nothing was being peeked at is refused, and changes nothing.**
#[test]
fn choosing_without_a_peek_refuses_and_leaves_the_desk_alone() {
    let (mut windows, mut panel) = a_desk_with_one_put_aside();
    let before: Vec<Window> = windows.each().cloned().collect();

    assert_eq!(
        chosen(
            &mut windows,
            &mut panel,
            alo_dock::Peeking::at_nothing(),
            a_view(),
            None,
            an_offer(),
        ),
        Err(NotPutAside::ItIsNotThere)
    );
    assert_eq!(
        windows.each().cloned().collect::<Vec<_>>(),
        before,
        "the refusal changed the desk, so it was not a refusal"
    );
    assert_eq!(panel.holding(), 1);
}

/// **Nothing in the peek module builds a `Travel`.**
///
/// The camera half of *both unchanged*, and the only version of it that can fail.
///
/// `peek_at` and `stop_peeking` take no view and return none, so the canvas cannot move — a
/// test comparing a view across those calls would pass for ever and prove nothing, which is a
/// shape this crate paid for today. So the check reads the source instead: a `Travel::To` built
/// here would be this crate deciding to move the canvas during a look, and that is the edit
/// that has to fail.
///
/// `chosen` is exempt by construction: it does not build a travel, it returns whatever the
/// restore road decided, which is a different thing and is why the words are `Travel::To` and
/// `Travel::NotNeeded` rather than the type's bare name.
#[test]
fn peeking_never_builds_a_travel_of_its_own() {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("peeking_at_a_preview.rs");
    let text = std::fs::read_to_string(&at).expect("this crate's own source");

    let deciding: Vec<&str> = text
        .lines()
        .filter(|line| {
            let code = only_the_code(line);
            code.contains("Travel::To") || code.contains("Travel::NotNeeded")
        })
        .map(str::trim)
        .collect();

    assert!(
        deciding.is_empty(),
        "the peek module constructs a travel, so looking at a window can move the canvas.\n\n\
         Peek's whole promise is that the canvas stays exactly where it is — alo-dock's \
         Peeking holds no view for that reason, and peek_at and stop_peeking take none. A \
         travel built here is this crate deciding to move during a look.\n\n\
         If a peek genuinely has to move something, that is a change to the acceptance and \
         not to this test.\n\n\
         Found:\n{deciding:#?}"
    );
}

/// A line with its documentation and string literals removed, so that prose naming a travel is
/// not counted as building one. The peek module explains at length why it must not build one.
fn only_the_code(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return String::new();
    }
    let mut code = String::new();
    let mut inside_a_string = false;
    let mut letters = line.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '\\' if inside_a_string => {
                letters.next();
            }
            '"' => inside_a_string = !inside_a_string,
            _ if !inside_a_string => code.push(letter),
            _ => {}
        }
    }
    code
}
