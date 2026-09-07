//! Opt-in popup handshake, isolated refusal and terminal parent lifetimes.
use super::{Application, Fixture};
mod presentation;

/// Protocol-only fixture explicitly opts in, independently of nested rendering.
fn fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.backend(|server| server.enable_popup_protocol());
    fixture
}

/// An eligible parent must already own a configured buffer.
fn mapped(fixture: &Fixture) -> Application {
    let mut app = Application::new(fixture);
    app.configure();
    app.attach();
    app.sync();
    app
}

#[test]
fn popup_configure_buffer_unmap_and_old_ack_cannot_revive() {
    let f = fixture();
    let mut app = mapped(&f);
    let (surface, xdg, _role) = app.popup(true, 1);
    app.sync();
    assert!(app.events.popups.geometry.is_empty());
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry, [(7, 10, 16, 16)]);
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    assert_eq!(f.backend(|s| s.popup_surfaces().len()), 1);
    assert_eq!(
        f.backend(|s| s.popup_surfaces().first().map(|popup| popup.geometry.loc)),
        Some((7, 10).into())
    );
    // An older target refuses live popups rather than silently omitting them.
    assert!(f.render((320, 200), false, 10).is_err());
    app.sync();
    assert!(app.events.frames.is_empty());
    surface.attach(None, 0, 0);
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    app.attach_popup(&surface);
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    app.ack_popup(&xdg);
    app.refused();
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
}

#[test]
fn popup_requires_ack_and_refusal_does_not_harm_another_client() {
    for commit_first in [false, true] {
        let f = fixture();
        let mut healthy = mapped(&f);
        let mut bad = mapped(&f);
        let (surface, _xdg, _role) = bad.popup(true, 0);
        if commit_first {
            surface.commit();
            bad.sync();
        }
        bad.attach_popup(&surface);
        bad.refused();
        healthy.sync();
        f.wait_for((1, 1));
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    }
}

#[test]
fn popup_invalid_parent_extreme_placement_and_disabled_backend_dismiss() {
    for (enabled, mapped_parent, parent, offset) in [
        (false, true, true, 0),
        (true, false, true, 0),
        (true, true, false, 0),
        (true, true, true, i32::MAX),
        (true, true, true, i32::MIN),
    ] {
        let f = if enabled { fixture() } else { Fixture::new() };
        let mut app = if mapped_parent {
            mapped(&f)
        } else {
            Application::new(&f)
        };
        let (surface, _, _) = app.popup(parent, offset);
        surface.commit();
        app.sync();
        assert_eq!(app.events.popups.done, 1);
        assert!(app.events.popups.geometry.is_empty());
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    }
}

#[test]
fn popup_parent_loss_role_destruction_and_disconnect_clear_snapshots() {
    for action in 0..4 {
        let f = fixture();
        let mut app = mapped(&f);
        let (surface, xdg, role) = app.popup(true, 0);
        surface.commit();
        app.sync();
        app.ack_popup(&xdg);
        app.attach_popup(&surface);
        app.sync();
        assert_eq!(f.backend(|s| s.popup_surfaces().len()), 1);
        match action {
            0 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
            }
            1 => app.toplevel.destroy(),
            2 => role.destroy(),
            _ => {
                drop(app);
                f.wait_for((0, 0));
                assert!(f.backend(|s| s.popup_surfaces().is_empty()));
                continue;
            }
        }
        app.sync();
        assert!(f.backend(|s| s.popup_surfaces().is_empty()));
        if action < 2 {
            assert_eq!(app.events.popups.done, 1);
            app.sync();
            assert_eq!(app.events.popups.done, 1);
        }
    }
}

#[test]
fn popup_reposition_dismisses_once_and_new_role_can_configure() {
    let f = fixture();
    let mut app = mapped(&f);
    let (surface, xdg, role) = app.popup(true, 0);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    app.reposition_popup(&role);
    app.sync();
    app.reposition_popup(&role);
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    role.destroy();
    xdg.destroy();
    surface.destroy();
    app.sync();
    let (surface, xdg, _role) = app.popup(true, 0);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    assert_eq!(f.backend(|s| s.popup_surfaces().len()), 1);
}
