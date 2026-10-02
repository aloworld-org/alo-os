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

/// **A window that named nothing is put aside like any other.**
///
/// **This test asserted the opposite until 2026-10-01**, and the reasoning it
/// carried was *this is a hole rather than a policy*: `alo_dock::Window`
/// required an `AppId`, an `AppId` is an identity, and the machine's own word
/// for an unnamed window is a translated string — so two windows that named
/// nothing would become one application if that word were the identity. The
/// hole was real and the remedy was wrong. The owner's direction of 2026-10-01
/// is that **a missing application identity must not prevent minimization**,
/// and that no identity is to be fabricated.
///
/// The answer was neither the word nor the number but the absence: `app` is an
/// `Option<AppId>`, so there is nothing to collapse two windows together and
/// nothing unreadable to show. The premise that a refused window would
/// otherwise be unreadable was also wrong — a preview is headed with the
/// **window's own** name, so what is lost is the application line, and the row
/// is one line shorter rather than blank.
///
/// Both halves are asserted, as the refusing version asserted both: the window
/// is held, **and** it actually left the canvas. A window hidden with nothing
/// holding its preview is one a person can neither see nor get back, which is
/// the single outcome this whole surface exists to prevent — and it is now the
/// failure this test would catch, rather than the refusal.
#[test]
fn a_window_that_named_nothing_is_put_aside_and_leaves_the_canvas()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let _app = mapped_nameless(&f);
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
            let hidden = s.minimized_surfaces().any(|it| it == &root);
            (put.is_ok() && panel.holding() == 1, hidden)
        })
    };

    assert!(
        held,
        "a window whose client named neither a class nor a title was refused, \
         which is the refusal the owner removed"
    );
    assert!(
        hidden,
        "it was accepted into the panel and left showing on the canvas, \
         which is the same window twice"
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

/// **A person asking for a window reaches the panel, not the primitive.**
///
/// The test task 5's *routed* clause and task 2's evidence both wanted. It enters by
/// `the_person_asked_to_put_aside`, which is what `Action::MinimiseWindow` and the button on a
/// window's own controls both call, and then performs the ask exactly where
/// `crate::direct_desktop` performs it.
///
/// Before this road existed, both of those callers reached `set_window_minimized` directly —
/// so a window was **hidden** and no preview appeared, while `docs/features.md` promised a
/// preview in the panel.
#[test]
fn a_person_asking_puts_the_window_in_the_panel_and_hides_it() {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();

    let (asked_before, put, held, hidden) = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            let mut panel = Panel::new();
            s.the_person_asked_to_put_aside(&root);
            let asked_before = s.how_many_asked_to_be_put_aside();
            let put = s.put_aside_what_was_asked_for(Some(&mut panel));
            let held = panel.holding();
            let hidden = s.minimized_surfaces().any(|it| it == &root);
            (asked_before, put, held, hidden)
        })
    };

    assert_eq!(asked_before, 1, "the ask was not recorded");
    assert_eq!(put, 1, "the ask was recorded and never met");
    assert_eq!(held, 1, "the window did not reach the panel");
    assert!(
        hidden,
        "the road composes set_window_minimized, so the window is hidden after the panel took it"
    );
}

/// **The asks are drained, so one keypress does not put a window aside for ever.**
///
/// A list that was read and not emptied would put the same window aside on every input batch —
/// and the second attempt would be refused as already-there, which looks like nothing
/// happening while the panel is asked sixty times a second.
#[test]
fn an_ask_is_met_once_and_then_forgotten() {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();

    let (first, left, second) = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            let mut panel = Panel::new();
            s.the_person_asked_to_put_aside(&root);
            let first = s.put_aside_what_was_asked_for(Some(&mut panel));
            let left = s.how_many_asked_to_be_put_aside();
            let second = s.put_aside_what_was_asked_for(Some(&mut panel));
            (first, left, second)
        })
    };

    assert_eq!(first, 1);
    assert_eq!(left, 0, "the ask was met and not forgotten");
    assert_eq!(second, 0, "a drained list put a window aside twice");
}

/// **A desktop with no panel loses nothing and puts nothing aside.**
///
/// The trait defaults `the_panel` to `None` because a desktop with no panel is a real one.
/// This asserts the road is a no-op there rather than a panic or a hidden window — a window
/// hidden with no preview to bring it back from is the outcome the whole surface prevents.
#[test]
fn a_desktop_with_no_panel_hides_nothing() {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();

    let (put, hidden) = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            s.the_person_asked_to_put_aside(&root);
            let put = s.put_aside_what_was_asked_for(None);
            let hidden = s.minimized_surfaces().any(|it| it == &root);
            (put, hidden)
        })
    };

    assert_eq!(put, 0);
    assert!(
        !hidden,
        "a window was hidden on a desktop with no panel to bring it back from"
    );
}
