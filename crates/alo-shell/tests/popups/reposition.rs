//! Explicit reposition transactions use committed geometry for the whole scene.
use super::{fixture, mapped};
use wayland_client::Proxy;

#[test]
fn reposition_tokens_acknowledgements_and_commits_are_independent() {
    let f = fixture();
    let mut app = mapped(&f);
    let (surface, xdg, role) = app.popup(true, 1);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    let position = || f.backend(|s| s.popup_surfaces().first().map(|p| p.geometry.loc.x));
    assert_eq!(position(), Some(7));
    app.events.popups.order.clear();
    app.reposition_popup_to(&role, 40, 91);
    app.sync();
    assert!(app.events.popups.serial.is_some());
    let first = app.events.popups.serial.unwrap_or_default();
    assert_eq!(app.events.popups.order, ["token", "geometry", "surface"]);
    assert_eq!(app.events.popups.geometry.last(), Some(&(34, -6, 16, 16)));
    app.reposition_popup_to(&role, 80, 92);
    app.sync();
    assert!(app.events.popups.serial.is_some());
    let second = app.events.popups.serial.unwrap_or_default();
    assert_eq!(app.events.popups.repositioned, [91, 92]);
    surface.commit();
    app.sync();
    assert_eq!(
        position(),
        Some(7),
        "unacknowledged configure moved the scene"
    );
    xdg.ack_configure(first);
    app.sync();
    assert_eq!(position(), Some(7), "ack without commit moved the scene");
    surface.commit();
    app.sync();
    assert_eq!(position(), Some(34));
    xdg.ack_configure(second);
    surface.commit();
    app.sync();
    assert_eq!(position(), Some(74));
    assert_eq!(app.events.popups.done, 0);
}

#[test]
fn reposition_stale_ack_refuses_only_its_client() {
    let f = fixture();
    let mut healthy = mapped(&f);
    let mut bad = mapped(&f);
    let (surface, xdg, role) = bad.popup(true, 0);
    surface.commit();
    bad.sync();
    bad.ack_popup(&xdg);
    bad.attach_popup(&surface);
    bad.sync();
    bad.reposition_popup_to(&role, 40, 1);
    bad.sync();
    assert!(bad.events.popups.serial.is_some());
    let stale = bad.events.popups.serial.unwrap_or_default();
    bad.reposition_popup_to(&role, 80, 2);
    bad.sync();
    bad.ack_popup(&xdg);
    surface.commit();
    bad.sync();
    xdg.ack_configure(stale);
    bad.refused();
    healthy.sync();
    f.wait_for((1, 1));
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
}

#[test]
fn reposition_moves_descendant_input_and_preserves_failed_frame_callbacks() {
    let f = super::presentation::fixture();
    let mut app = mapped(&f);
    let (surface, xdg, role) = app.popup(true, 1);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    let (child, child_xdg, _) = app.popup_on(Some(&xdg), 1);
    child.commit();
    app.sync();
    app.ack_popup(&child_xdg);
    app.attach_popup(&child);
    app.sync();
    app.reposition_popup_to(&role, 80, 7);
    app.sync();
    app.ack_popup(&xdg);
    surface.commit();
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(82.0, 5.0, 1)).is_ok());
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(child.id().protocol_id(), 1.0, 1.0))
    );
    assert!(
        f.backend(|s| s.render(&mut super::presentation::Target(true), 2))
            .is_err()
    );
    app.sync();
    assert!(app.events.frames.is_empty());
    assert_eq!(
        f.backend(|s| s.render(&mut super::presentation::Target(false), 3))
            .ok(),
        Some(3)
    );
    app.sync();
    assert_eq!(app.events.frames, [3, 3]);
    surface.attach(None, 0, 0);
    surface.commit();
    app.sync();
    app.reposition_popup_to(&role, 0, 8);
    app.sync();
    assert_eq!(app.events.popups.repositioned, [7]);
    assert_eq!(app.events.popups.done, 2);
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
}
