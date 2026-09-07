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
    app.reposition_popup(&role);
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
        "Popup GLES buffer submitted at (8,11) with parent/popup geometry offsets; nested child at (16,23), both callbacks/output enter and descendant dismissal leave passed; offscreen callback withheld"
    );
}
