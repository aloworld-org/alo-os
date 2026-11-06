//! **An application asks for a cursor by name**, over a real Unix-socket
//! connection, and what alo does with each request — including every request it
//! refuses.
//!
//! The refusals are Smithay's (`crate::cursor_shapes` in the shell says which),
//! and they are tested here from the outside because *it was refused* is the
//! promise: one application cannot change the pointer over another's window, a
//! shape the protocol does not name changes nothing, and a shape does not
//! outlive the application that asked for it.
use super::support::{Application, Fixture};
use alo_shell::Cursor;
use wayland_client::{
    Proxy,
    backend::protocol::{Argument, Message},
};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape;

/// A fixture with a pointer over one application's window.
fn setup() -> (Fixture, Application) {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(8.0, 9.0, 1)).is_ok());
    app.sync();
    (f, app)
}

/// What the shell would draw now, as the name it is drawing, if it is one.
fn named(f: &Fixture) -> Option<alo_cursors::CursorIcon> {
    match f.backend(|s| s.cursor()) {
        Cursor::Named { shape, .. } => Some(shape),
        _ => None,
    }
}

/// **A name asked for is the name drawn**, at the pointer's position, and the
/// arrow by name is the arrow.
#[test]
fn a_shape_asked_for_by_name_is_the_one_alo_draws() {
    let (f, mut app) = setup();
    let Some(device) = app.a_cursor_shape_device() else {
        unreachable!("the compositor advertises no wp_cursor_shape_manager_v1");
    };
    let serial = app.events.pointer.serial;
    for (asked, drawn) in [
        (Shape::Text, alo_cursors::CursorIcon::Text),
        (Shape::Pointer, alo_cursors::CursorIcon::Pointer),
        (Shape::EwResize, alo_cursors::CursorIcon::EwResize),
        (Shape::Grabbing, alo_cursors::CursorIcon::Grabbing),
    ] {
        device.set_shape(serial, asked);
        app.sync();
        assert_eq!(named(&f), Some(drawn), "{asked:?}");
    }
    assert!(matches!(
        f.backend(|s| s.cursor()),
        Cursor::Named { location, .. } if location == (8.0, 9.0).into()
    ));
    device.set_shape(serial, Shape::Default);
    app.sync();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Arrow { .. }));
}

/// **A shape wins over a stale surface, and a surface set after a shape is the
/// application's business** — whichever it asked for last is what shows.
#[test]
fn the_latest_request_wins_between_a_shape_and_a_surface() {
    let (f, mut app) = setup();
    let Some(device) = app.a_cursor_shape_device() else {
        unreachable!("the compositor advertises no wp_cursor_shape_manager_v1");
    };
    let _surface = app.cursor((1, 1));
    app.sync();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Surface { .. }));
    device.set_shape(app.events.pointer.serial, Shape::Text);
    app.sync();
    assert_eq!(named(&f), Some(alo_cursors::CursorIcon::Text));
    let _again = app.cursor((0, 0));
    app.sync();
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Surface { .. }));
}

/// **An application without the pointer cannot change it**, and neither can
/// the one that has it with a serial from an earlier enter.
#[test]
fn a_client_without_the_pointer_is_refused() {
    let (f, mut app) = setup();
    let Some(mine) = app.a_cursor_shape_device() else {
        unreachable!("the compositor advertises no wp_cursor_shape_manager_v1");
    };
    mine.set_shape(app.events.pointer.serial, Shape::Text);
    app.sync();
    let mut other = Application::new(&f);
    other.configure();
    other.attach();
    other.sync();
    let Some(theirs) = other.a_cursor_shape_device() else {
        unreachable!("the compositor advertises no wp_cursor_shape_manager_v1");
    };
    // The other application has never had the pointer, so it holds no serial
    // of its own; it uses the focused one's, which is the most it could know.
    theirs.set_shape(app.events.pointer.serial, Shape::NotAllowed);
    other.sync();
    assert_eq!(
        named(&f),
        Some(alo_cursors::CursorIcon::Text),
        "an application without the pointer changed it"
    );
    mine.set_shape(app.events.pointer.serial.wrapping_sub(1), Shape::Wait);
    app.sync();
    assert_eq!(
        named(&f),
        Some(alo_cursors::CursorIcon::Text),
        "a stale serial changed the pointer"
    );
}

/// **A shape number the protocol does not name changes nothing** and does not
/// take the application down.
///
/// The protocol says this is an `invalid_shape` error; Smithay 0.7 ignores it
/// instead (`docs/quirks/an-unknown-cursor-shape-is-ignored-rather-than-refused.md`).
/// This holds what is true — nothing changes, and the application keeps
/// working — so the day Smithay raises the error, this says so.
#[test]
fn an_unknown_shape_changes_nothing() {
    let (f, mut app) = setup();
    let Some(device) = app.a_cursor_shape_device() else {
        unreachable!("the compositor advertises no wp_cursor_shape_manager_v1");
    };
    device.set_shape(app.events.pointer.serial, Shape::Text);
    app.sync();
    // `set_shape` is request 1 of the device; a typed request cannot carry a
    // number outside the enumeration, so this one is written by hand.
    let sent = app.connection.backend().send_request(
        Message {
            sender_id: device.id(),
            opcode: 1,
            args: vec![
                Argument::Uint(app.events.pointer.serial),
                Argument::Uint(9_999),
            ]
            .into(),
        },
        None,
        None,
    );
    assert!(sent.is_ok(), "the request could not be written: {sent:?}");
    app.sync();
    assert_eq!(named(&f), Some(alo_cursors::CursorIcon::Text));
    assert!(app.connection.protocol_error().is_none());
}

/// **A shape does not outlive the application that asked for it**: when its
/// connection is gone, the pointer is alo's arrow again rather than a cursor
/// nobody is answering for.
#[test]
fn a_dead_applications_shape_goes_with_it() {
    let (f, mut app) = setup();
    let Some(device) = app.a_cursor_shape_device() else {
        unreachable!("the compositor advertises no wp_cursor_shape_manager_v1");
    };
    device.set_shape(app.events.pointer.serial, Shape::Grab);
    app.sync();
    assert_eq!(named(&f), Some(alo_cursors::CursorIcon::Grab));
    drop(device);
    drop(app);
    f.wait_for((0, 0));
    assert!(matches!(f.backend(|s| s.cursor()), Cursor::Arrow { .. }));
}

/// **A destroyed device is inert**: the shape it set stays until something
/// changes it, and nothing it could still send reaches the pointer.
#[test]
fn a_destroyed_device_changes_nothing_more() {
    let (f, mut app) = setup();
    let Some(device) = app.a_cursor_shape_device() else {
        unreachable!("the compositor advertises no wp_cursor_shape_manager_v1");
    };
    device.set_shape(app.events.pointer.serial, Shape::Crosshair);
    app.sync();
    device.destroy();
    app.sync();
    assert_eq!(named(&f), Some(alo_cursors::CursorIcon::Crosshair));
    // A request on a destroyed object is not sent by the client library at all;
    // the pointer is unchanged and the connection is unharmed.
    device.set_shape(app.events.pointer.serial, Shape::Wait);
    app.sync();
    assert_eq!(named(&f), Some(alo_cursors::CursorIcon::Crosshair));
    assert!(app.connection.protocol_error().is_none());
}
