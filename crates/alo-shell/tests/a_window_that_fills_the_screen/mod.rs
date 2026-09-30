//! A window filling the screen: a client asking, a person asking, and the
//! furniture giving way.
//!
//! **Entered by the road that was answered with silence until 2026-09-30.**
//! Smithay's default `fullscreen_request` is empty, so a client that asked
//! received neither a configure nor a refusal — and from the client's side a
//! compositor that advertises nothing and answers nothing is indistinguishable
//! from one that is broken. These drive a real client through the protocol
//! rather than calling the shell's own entry, because the request is the thing
//! that was missing.

use super::{Application, Fixture};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// A mapped application, the way the other client tests make one.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Whether the most recent configure said the window fills the screen.
fn filling(app: &Application) -> bool {
    app.events.fullscreen.last().copied().unwrap_or(false)
}

/// Whether the most recent configure said the window is maximised.
fn maximised(app: &Application) -> bool {
    app.events.maximized.last().copied().unwrap_or(false)
}

/// **A client that asks to fill the screen is told it is filling the screen.**
///
/// The whole of what was missing: before this, the request reached an empty
/// default and nothing came back.
#[test]
fn a_client_asking_to_fill_the_screen_is_answered() -> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);

    app.toplevel.set_fullscreen(None);
    app.sync();
    f.backend(|_| ());
    app.sync();

    assert!(
        filling(&app),
        "a client asked to fill the screen and was not told it had"
    );
    Ok(())
}

/// **Fullscreen alone, never fullscreen and maximised together.**
///
/// A client told both has to decide which it is, and XDG's own answer is that
/// fullscreen supersedes — so saying both says nothing extra and invites two
/// readings of one window.
#[test]
fn a_window_filling_the_screen_is_not_also_told_it_is_maximised()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);

    app.toplevel.set_maximized();
    app.sync();
    f.backend(|_| ());
    app.sync();
    app.toplevel.set_fullscreen(None);
    app.sync();
    f.backend(|_| ());
    app.sync();

    assert!(filling(&app), "not told it fills the screen");
    assert!(
        !maximised(&app),
        "told both fullscreen and maximised, which are two readings of one window"
    );
    Ok(())
}

/// **Leaving full screen returns to normal, not to maximised.**
///
/// However the window arrived. A window a person put into full screen from a
/// normal size and got back maximised has been changed by a journey it did not
/// ask for.
#[test]
fn leaving_full_screen_returns_to_normal() -> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);

    app.toplevel.set_fullscreen(None);
    app.sync();
    f.backend(|_| ());
    app.sync();
    app.toplevel.unset_fullscreen();
    app.sync();
    f.backend(|_| ());
    app.sync();

    assert!(!filling(&app), "still filling the screen after leaving it");
    assert!(
        !maximised(&app),
        "came back maximised from a journey it did not ask for"
    );
    Ok(())
}

/// **The shell knows, which is what the Dock asks before drawing itself.**
///
/// This is the half that makes full screen different from maximised for a
/// person rather than only for a client. Asserted through the trusted entry so
/// that both roads are covered: a client asking above, a person asking here.
#[test]
fn the_shell_knows_a_window_is_filling_the_screen() -> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    let root = f.root();

    let (before, during, after) = {
        let root: WlSurface = root.clone();
        f.backend(move |s| {
            let before = s.a_window_is_filling_the_screen();
            let _ = s.set_window_full_screen(&root, true);
            let during = s.a_window_is_filling_the_screen();
            let _ = s.set_window_full_screen(&root, false);
            let after = s.a_window_is_filling_the_screen();
            (before, during, after)
        })
    };

    assert!(
        !before,
        "a fresh desk already had a window filling the screen"
    );
    assert!(
        during,
        "the shell does not know a window is filling the screen"
    );
    assert!(!after, "the shell still thinks one is, after it stopped");
    Ok(())
}
