//! A real client's window put aside, and brought back to where it was.
//!
//! **Entered by the road a person's gesture will take**, not by handing a
//! `Panel` a `Preview` somebody constructed. A real client maps through the
//! protocol, the shell reads its own name and number off the surface, and the
//! trusted entry point is the one a key binding will call. The panel plan's own
//! rule is that *a test that enters by the road a person uses is the only one
//! that can tell you the road exists*, and the fault it names — a band of tests
//! calling the compositor directly while no mouse could reach the thing — is
//! exactly what a `panel.put_aside(&window, …)` test here would repeat.
//!
//! What is still not the road: the key binding itself. When there is one, these
//! go through it.

use super::{Application, Fixture};
use alo_canvas::Zoom;
use alo_dock::{Patch, Spot};
use alo_put_aside::Panel;
use alo_put_aside::whether_it_is_private::Privacy;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// A mapped application, the way the other client tests make one.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    // A real application names itself, and these tests are about what happens
    // to a window rather than about what an unnamed one is called. The window
    // that names nothing has a test of its own below.
    app.toplevel.set_app_id("Docs".to_owned());
    app.toplevel.set_title("Launch strategy".to_owned());
    app.configure();
    app.attach();
    app.sync();
    app
}

/// A mapped application that never named itself, which is a real client.
fn mapped_nameless(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Where a window sits on the canvas. Handed in, as the shell hands it in.
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
fn somewhere() -> Patch {
    Patch::of(Spot::at(4_000, -200), 800, 600).expect("a window has extent")
}

/// **A window that named nothing is refused, and stays where it is.**
///
/// This is a hole rather than a policy, and the test says so rather than
/// blessing it. `alo_dock::Window` needs an `AppId`, an `AppId` is an identity,
/// and the machine's own word for an unnamed window is a *translated string* —
/// so two windows that named nothing would become one application if the word
/// were used as the identity. The design has an answer for what such a window
/// is **called**; the model has none for what it **is**.
///
/// What matters until that is settled is the second assertion: a refused
/// request leaves the window on the canvas. A window hidden by a request that
/// failed is one a person can neither see nor get back, which is the single
/// outcome this whole surface exists to prevent.
#[test]
fn a_window_that_named_nothing_is_refused_and_stays_on_the_canvas()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let _app = mapped_nameless(&f);
    let root = f.root();

    let (refused, still_shown) = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            let mut panel = Panel::new();
            let put = s.put_this_window_aside(
                &mut panel,
                &root,
                somewhere(),
                Zoom::LIFE_SIZE,
                Privacy::Ordinary,
            );
            let shown = !s.minimized_surfaces().any(|it| it == &root);
            (put.is_err() && panel.holding() == 0, shown)
        })
    };

    assert!(refused, "a window with no name at all was put aside");
    assert!(
        still_shown,
        "a refused request hid the window anyway, which is the one outcome this prevents"
    );
    Ok(())
}

/// **A window put aside leaves the canvas and the panel holds it.**
///
/// Both halves asserted, because either alone is a different bug: a window
/// hidden with nothing holding its preview is one a person cannot get back,
/// and a preview for a window still on the canvas is the same window twice.
#[test]
fn a_window_put_aside_leaves_the_canvas_and_the_panel_holds_it()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();

    let (held, hidden) = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            let mut panel = Panel::new();
            let put = s.put_this_window_aside(
                &mut panel,
                &root,
                somewhere(),
                Zoom::LIFE_SIZE,
                Privacy::Ordinary,
            );
            let held = put.is_ok() && panel.holding() == 1;
            let hidden = s.minimized_surfaces().any(|it| it == &root);
            (held, hidden)
        })
    };

    assert!(
        held,
        "the panel is not holding the window that was put aside"
    );
    assert!(
        hidden,
        "the window was put aside and is still on the canvas"
    );
    Ok(())
}

/// **The preview carries where the window was**, which is what a person gets
/// back. A preview without it would restore a window to wherever the canvas
/// happens to be rather than to its own place.
#[test]
fn the_preview_remembers_the_place_the_window_left() -> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();

    let at = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            let mut panel = Panel::new();
            s.put_this_window_aside(
                &mut panel,
                &root,
                somewhere(),
                Zoom::LIFE_SIZE,
                Privacy::Ordinary,
            )
            .ok()?;
            let back = s.bring_this_window_back(&mut panel, &root).ok()?;
            Some(back.at())
        })
    };

    assert_eq!(
        at,
        Some(somewhere()),
        "the window did not come back to the place it left"
    );
    Ok(())
}

/// **Bringing it back puts it on the canvas again and empties the panel.**
#[test]
fn bringing_it_back_returns_it_to_the_canvas() -> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();

    let (emptied, shown) = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            let mut panel = Panel::new();
            let _ = s.put_this_window_aside(
                &mut panel,
                &root,
                somewhere(),
                Zoom::LIFE_SIZE,
                Privacy::Ordinary,
            );
            let _ = s.bring_this_window_back(&mut panel, &root);
            let emptied = panel.holding() == 0;
            let shown = !s.minimized_surfaces().any(|it| it == &root);
            (emptied, shown)
        })
    };

    assert!(emptied, "the panel still holds a window that came back");
    assert!(shown, "the window came back and is still off the canvas");
    Ok(())
}

/// **The same window cannot be put aside twice**, and the refusal leaves it on
/// the canvas rather than hiding it for a panel that did not take it.
///
/// This is the ordering `alo_put_aside::putting_aside` exists to protect and
/// the reason the shell hides *after* the panel accepts: a window hidden by a
/// refused request is one a person cannot see and cannot get back.
#[test]
fn a_refused_second_putting_aside_leaves_the_window_where_it_is()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();

    let (refused, still_held_once) = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            let mut panel = Panel::new();
            let _ = s.put_this_window_aside(
                &mut panel,
                &root,
                somewhere(),
                Zoom::LIFE_SIZE,
                Privacy::Ordinary,
            );
            let again = s.put_this_window_aside(
                &mut panel,
                &root,
                somewhere(),
                Zoom::LIFE_SIZE,
                Privacy::Ordinary,
            );
            (again.is_err(), panel.holding() == 1)
        })
    };

    assert!(refused, "the same window was put aside twice");
    assert!(
        still_held_once,
        "a refused second request changed what the panel holds"
    );
    Ok(())
}
