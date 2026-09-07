//! Real popup buffers through the nested GLES backend, including clipping.

use crate::application::Application;
use std::{
    thread,
    time::{Duration, Instant},
};

/// Run on the protocol-client thread while the main thread drives the backend.
pub fn check(app: &mut Application) {
    let deadline = Instant::now() + Duration::from_secs(5);
    let (hidden, xdg, role) = app.popup(true, 10000);
    hidden.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&hidden);
    for _ in 0..5 {
        app.sync();
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(app.events.frames.len(), 2, "offscreen popup callback");
    role.destroy();
    xdg.destroy();
    hidden.destroy();
    app.sync();

    app.xdg.set_window_geometry(2, 3, 12, 12);
    app.surface.commit();
    let (surface, xdg, role) = app.popup(true, 1);
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(7, 10, 16, 16)));
    app.ack_popup(&xdg);
    app.reposition_popup_to(&role, 40, 93);
    app.sync();
    assert_eq!(app.events.popups.repositioned, [93]);
    assert_eq!(app.events.popups.geometry.last(), Some(&(34, -6, 16, 16)));
    app.ack_popup(&xdg);
    xdg.set_window_geometry(1, 2, 12, 12);
    let (far_child, far_role) = app.child_on(&surface, (i32::MAX, i32::MAX));
    app.attach_popup(&surface);
    while app.events.frames.len() < 3 {
        app.sync();
        assert!(Instant::now() < deadline, "no GLES popup callback");
    }
    assert_eq!(
        app.events.frames.len(),
        3,
        "extreme offscreen child callback"
    );
    assert_eq!(app.events.membership, (3, 0));
    let (nested, nested_xdg, nested_role) = app.popup_on(Some(&xdg), 1);
    nested.commit();
    app.sync();
    app.ack_popup(&nested_xdg);
    app.attach_popup(&nested);
    while app.events.frames.len() < 4 {
        app.sync();
        assert!(Instant::now() < deadline, "no nested GLES popup callback");
    }
    assert_eq!(app.events.membership, (4, 0));
    app.refuse_popup_position(&role);
    while app.events.membership.1 < 2 {
        app.sync();
        assert!(Instant::now() < deadline, "no popup leave on dismissal");
    }
    assert_eq!(app.events.popups.done, 2);
    nested_role.destroy();
    nested_xdg.destroy();
    nested.destroy();
    far_role.destroy();
    far_child.destroy();
    role.destroy();
    xdg.destroy();
    surface.destroy();
    app.sync();
    println!(
        "Repositioned popup GLES buffer submitted at (35,-5) with parent/popup geometry offsets; nested child at (43,7), both callbacks/output enter and descendant dismissal leave passed; offscreen callback withheld"
    );
    constrained(app);
    reactive(app);
}

/// Submit a reactive menu after a committed change to its parent's window origin.
fn reactive(app: &mut Application) {
    use wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment as Adjust;
    let deadline = Instant::now() + Duration::from_secs(5);
    let frames = app.events.frames.len();
    let (surface, xdg, role) =
        app.popup_reactive(Some(&app.xdg), 10000, Adjust::SlideX | Adjust::SlideY, true);
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(302, 10, 16, 16)));
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    while app.events.frames.len() < frames + 1 {
        app.sync();
        assert!(
            Instant::now() < deadline,
            "no reactive initial GLES callback"
        );
    }
    let configs = app.events.popups.geometry.len();
    app.xdg.set_window_geometry(4, 5, 12, 12);
    app.surface.commit();
    while app.events.popups.geometry.len() == configs {
        app.sync();
        assert!(Instant::now() < deadline, "no reactive parent configure");
    }
    assert_eq!(app.events.popups.geometry.last(), Some(&(300, 10, 16, 16)));
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    while app.events.frames.len() < frames + 2 {
        app.sync();
        assert!(Instant::now() < deadline, "no reactive moved GLES callback");
    }
    assert_eq!(app.events.popups.geometry.len(), configs + 1);
    let leaves = app.events.membership.1;
    surface.attach(None, 0, 0);
    surface.commit();
    while app.events.membership.1 < leaves + 1 {
        app.sync();
        assert!(Instant::now() < deadline, "no reactive popup output leave");
    }
    role.destroy();
    xdg.destroy();
    surface.destroy();
    app.xdg.set_window_geometry(2, 3, 12, 12);
    app.surface.commit();
    app.sync();
    println!(
        "Reactive popup GLES buffers submitted at (304,13) then (304,15); parent-commit configure, callbacks and output leave passed"
    );
}

/// Submit both edges using client-authorized sliding in parent window coordinates.
fn constrained(app: &mut Application) {
    use wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment as Adjust;
    let deadline = Instant::now() + Duration::from_secs(5);
    let frames = app.events.frames.len();
    let enters = app.events.membership.0;
    let (surface, xdg, role) =
        app.popup_adjusted(Some(&app.xdg), 10000, Adjust::SlideX | Adjust::SlideY);
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(302, 10, 16, 16)));
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    while app.events.frames.len() < frames + 1 {
        app.sync();
        assert!(Instant::now() < deadline, "no constrained GLES callback");
    }
    assert_eq!(app.events.membership.0, enters + 1);
    app.reposition_adjusted(&role, -10000, 94, Adjust::SlideX | Adjust::SlideY);
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(-2, -3, 16, 16)));
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    while app.events.frames.len() < frames + 2 {
        app.sync();
        assert!(
            Instant::now() < deadline,
            "no constrained reposition GLES callback"
        );
    }
    let leaves = app.events.membership.1;
    surface.attach(None, 0, 0);
    surface.commit();
    while app.events.membership.1 < leaves + 1 {
        app.sync();
        assert!(
            Instant::now() < deadline,
            "no constrained popup output leave"
        );
    }
    role.destroy();
    xdg.destroy();
    surface.destroy();
    app.sync();
    println!(
        "Constrained popup GLES buffers submitted at (304,13) then (0,0); both callbacks and output enter passed"
    );
}
