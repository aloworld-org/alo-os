//! A window whose client named no application, put aside and brought back.
//!
//! **This was impossible until 2026-10-01.** A client may map a toplevel having
//! set neither a class nor a title, and the minimise gesture refused such a
//! window outright: `alo_shell::NotAside::Unnamed`. The owner's direction is
//! that *a missing application identity must not prevent minimization*, and
//! that no identity is to be fabricated to get around it.
//!
//! # Why the absence is the subject rather than an edge case
//!
//! The two repairs that suggest themselves are both worse than the absence, and
//! each is tested against below. A **translated word** — *an application* — is
//! not an identity, so two unnamed windows would become one application
//! wherever `AppId` is grouped by. The **window's own number** is unique and is
//! an identity no person can read. `None` is neither: it cannot be grouped
//! wrongly and it cannot be shown as a name.
//!
//! The panel identifies a window by its **own identity, its title and its
//! preview**, and the application is a line it adds when there is one. So these
//! tests assert the whole journey — aside, held, found, restored to the saved
//! patch — on a window that has no application at all.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — \
              the same exemption `alo-dock`'s own test modules take, with its words"
)]

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::{Patch, Spot, TheView};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_dock::windows::Windows;
use alo_put_aside::putting_aside::put_aside;
use alo_put_aside::restoring::restore;
use alo_put_aside::{Panel, Privacy};

/// The zoom the person was at — **never [`Zoom::LIFE_SIZE`]**.
///
/// Life size is `Zoom`'s own default, so a fixture using it would let these
/// assertions pass for an implementation that saved no zoom and built a default
/// on the way out.
fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// A window with **no application**, called something, sitting somewhere.
fn nameless(id: u64, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        None,
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// A window that does have one, for the tests that need both kinds.
fn named(id: u64, app: &str, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        Some(AppId::named(app).expect("a fixture names its application")),
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// A view of the plane, wide enough to hold a window and then some.
fn view(x: i64) -> TheView {
    TheView::showing(Patch::of(Spot::at(x, -200), 1_920, 1_080).expect("a view has extent"))
}

/// **A window with no application can be put aside at all.**
///
/// The whole of what was refused. Asserted through the panel rather than
/// through a return value alone, because *the call did not error* and *the
/// window is in the panel* are two different claims and only the second is the
/// promise.
#[test]
fn a_window_with_no_application_goes_into_the_panel() {
    let mut windows = Windows::none();
    windows.opened(nameless(1, "Untitled", 0));
    let mut panel = Panel::new();

    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .expect("a window with no application was refused, which is the bug this test exists for");

    assert_eq!(
        panel.previews().len(),
        1,
        "the call succeeded and the panel is empty, so nothing was actually put aside"
    );
    assert_eq!(
        panel
            .previews()
            .first()
            .expect("the length was just asserted to be one")
            .window(),
        WindowId::numbered(1),
        "the panel holds a preview of some other window"
    );
}

/// **Its preview says no application, rather than saying a made-up one.**
///
/// The direction was explicit that no identity is to be substituted, so this
/// asserts the absence itself. A fabricated name would satisfy *the window was
/// put aside* and break the rule that produced it.
#[test]
fn its_preview_carries_no_application_rather_than_an_invented_one() {
    let mut windows = Windows::none();
    windows.opened(nameless(1, "Untitled", 0));
    let mut panel = Panel::new();

    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    assert_eq!(
        panel
            .previews()
            .first()
            .expect("the window was just put aside, so the panel holds it")
            .app(),
        None,
        "an application was invented for a window whose client named none"
    );
}

/// **It comes back to its saved patch, like any other window.**
///
/// Restoring is the half that would be easy to leave broken: a window that goes
/// into the panel and cannot be got out is worse than one that was refused,
/// because the person has already lost sight of it. The view sits at the
/// plane's origin and the window twenty thousand units away, so an
/// implementation answering with the current view would be visibly wrong rather
/// than accidentally right.
#[test]
fn a_window_with_no_application_comes_back_to_where_it_was() {
    let mut windows = Windows::none();
    windows.opened(nameless(1, "Untitled", 20_000));
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
    let travel = restore(&mut windows, &mut panel, WindowId::numbered(1), view(0)).unwrap();

    let there = travel
        .to()
        .expect("the window is off screen, so a travel is needed");
    assert_eq!(
        there.at(),
        was_at,
        "it came back somewhere other than the patch it was put aside from"
    );
    assert_eq!(
        there.zoom(),
        a_zoom(),
        "it came back at a different zoom from the one it was put aside at"
    );
    assert!(
        panel.previews().is_empty(),
        "it was restored and the panel still holds a preview of it"
    );
}

/// **Two windows with no application are two rows, not one.**
///
/// This is the test that fails for the repair nobody should make. Give both a
/// shared translated word and they become one application; group by that and a
/// person who put two windows aside finds one. The identity the panel actually
/// uses is the window's own, which is never absent — so the absence of an
/// application cannot collapse them.
#[test]
fn two_windows_with_no_application_stay_two_windows() {
    let mut windows = Windows::none();
    windows.opened(nameless(1, "Untitled", 0));
    windows.opened(nameless(2, "Untitled", 2_000));
    let mut panel = Panel::new();

    for id in [1, 2] {
        put_aside(
            &mut windows,
            &mut panel,
            WindowId::numbered(id),
            a_zoom(),
            Privacy::Ordinary,
        )
        .unwrap();
    }

    assert_eq!(
        panel.previews().len(),
        2,
        "two windows with no application became one row"
    );
    let mut held: Vec<u64> = panel
        .previews()
        .iter()
        .map(|p| p.window().number())
        .collect();
    held.sort_unstable();
    assert_eq!(
        held,
        [1, 2],
        "the panel holds two rows but not the two windows that were put aside"
    );
}

/// **A named window and an unnamed one sit side by side.**
///
/// The absence must not disturb the windows around it: the panel's existing
/// promise is that it holds windows rather than applications, and a window with
/// no application is the sharpest case of that.
#[test]
fn an_unnamed_window_and_a_named_one_are_both_held() {
    let mut windows = Windows::none();
    windows.opened(named(1, "Browser", "Research", 0));
    windows.opened(nameless(2, "Untitled", 2_000));
    let mut panel = Panel::new();

    for id in [1, 2] {
        put_aside(
            &mut windows,
            &mut panel,
            WindowId::numbered(id),
            a_zoom(),
            Privacy::Ordinary,
        )
        .unwrap();
    }

    assert_eq!(panel.previews().len(), 2, "one of the two was not held");
    let apps: Vec<Option<&str>> = panel
        .previews()
        .iter()
        .map(|p| p.app().map(AppId::name))
        .collect();
    assert!(
        apps.contains(&Some("Browser")),
        "the named window lost its application"
    );
    assert!(
        apps.contains(&None),
        "the unnamed window gained one, which is the substitution the rule forbids"
    );
}
