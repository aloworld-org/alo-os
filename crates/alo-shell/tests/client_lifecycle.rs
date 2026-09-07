//! Real XDG clients exercise buffer lifecycle and isolated protocol refusal.
#![cfg(target_os = "linux")]

mod input;
mod pointer;
mod support;
use support::{Application, Fixture};

#[test]
fn synchronized_children_do_not_become_independent_toplevels() {
    let fixture = Fixture::new();
    let mut app = Application::new(&fixture);
    app.configure();
    let (child, role) = app.child((24, 32));
    app.attach();
    app.sync();
    fixture.wait_for((1, 1));
    role.destroy();
    child.destroy();
    app.surface.commit();
    app.sync();
    fixture.wait_for((1, 1));
}

#[test]
fn output_resize_and_callbacks_follow_successful_submission() {
    let fixture = Fixture::new();
    assert_eq!(fixture.render((320, 200), false, 0).ok(), Some(0));
    let mut app = Application::new(&fixture);
    app.sync();
    assert_eq!(app.events.outputs, 1);
    assert_eq!(app.events.modes.last(), Some(&(320, 200)));
    app.configure();
    app.surface.frame(&app.queue.handle(), ());
    app.attach();
    app.sync();
    assert!(app.events.frames.is_empty());
    assert!(fixture.render((320, 200), true, 10).is_err());
    app.sync();
    assert!(app.events.frames.is_empty());
    assert_eq!(app.events.membership, (0, 0));
    assert_eq!(fixture.render((320, 200), false, 20).ok(), Some(1));
    app.sync();
    assert_eq!(app.events.frames, [20]);
    assert_eq!(app.events.membership, (1, 0));
    assert_eq!(fixture.render((640, 480), false, 30).ok(), Some(1));
    app.sync();
    assert_eq!(app.events.outputs, 1);
    assert_eq!(app.events.modes.last(), Some(&(640, 480)));
    assert_eq!(app.events.frames, [20]);
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(fixture.render((640, 480), false, 40).ok(), Some(0));
    app.sync();
    assert_eq!(app.events.membership, (1, 1));
    drop(app);
    fixture.wait_for((0, 0));
    assert_eq!(fixture.render((640, 480), false, 50).ok(), Some(0));
}

#[test]
fn empty_output_and_unmapped_surface_never_complete_a_frame() {
    let fixture = Fixture::new();
    assert!(matches!(
        fixture.render((0, 200), false, 0),
        Err(alo_shell::RenderError::EmptySize)
    ));
    let mut app = Application::new(&fixture);
    assert_eq!(app.events.outputs, 0);
    app.surface.frame(&app.queue.handle(), ());
    app.configure();
    assert_eq!(fixture.render((320, 200), false, 1).ok(), Some(0));
    app.sync();
    assert!(app.events.frames.is_empty());
    app.attach();
    app.sync();
    assert!(matches!(
        fixture.render((0, 0), false, 2),
        Err(alo_shell::RenderError::EmptySize)
    ));
    app.sync();
    assert!(app.events.frames.is_empty());
    assert_eq!(fixture.render((320, 200), false, u32::MAX).ok(), Some(1));
    app.sync();
    assert_eq!(app.events.frames, [u32::MAX]);
}

#[test]
fn configure_map_unmap_remap_and_orderly_destroy() {
    let fixture = Fixture::new();
    let mut app = Application::new(&fixture);
    assert_eq!(
        app.events.serial, None,
        "configure must wait for the initial commit"
    );
    fixture.wait_for((1, 0));
    let first = app.configure();
    fixture.wait_for((1, 0));
    app.attach();
    app.sync();
    fixture.wait_for((1, 1));
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    fixture.wait_for((1, 0));
    assert_ne!(app.configure(), first);
    assert_eq!(
        app.events.releases, 1,
        "unmapping releases the attached buffer"
    );
    app.attach();
    app.sync();
    fixture.wait_for((1, 1));
    app.toplevel.destroy();
    app.xdg.destroy();
    app.surface.destroy();
    app.sync();
    fixture.wait_for((0, 0));
}

#[test]
fn abrupt_disconnect_removes_mapped_surfaces() {
    let fixture = Fixture::new();
    let mut app = Application::new(&fixture);
    app.configure();
    app.attach();
    app.sync();
    fixture.wait_for((1, 1));
    drop(app);
    fixture.wait_for((0, 0));
    let mut next = Application::new(&fixture);
    next.configure();
    next.attach();
    next.sync();
    fixture.wait_for((1, 1));
}

#[test]
fn premature_buffer_and_invalid_ack_only_disconnect_the_offender() {
    let fixture = Fixture::new();
    let mut good = Application::new(&fixture);
    good.configure();
    good.attach();
    good.sync();
    let mut premature = Application::new(&fixture);
    premature.attach();
    premature.refused();
    fixture.wait_for((1, 1));
    let mut wrong = Application::new(&fixture);
    wrong.xdg.ack_configure(0);
    wrong.refused();
    fixture.wait_for((1, 1));
    good.sync();
}

#[test]
fn unmapping_invalidates_the_previous_handshake() {
    let fixture = Fixture::new();
    let mut app = Application::new(&fixture);
    app.configure();
    app.attach();
    app.sync();
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    fixture.wait_for((1, 0));
    app.attach();
    app.refused();
    fixture.wait_for((0, 0));
}

#[test]
fn receiving_a_configure_without_acknowledging_it_cannot_map() {
    let fixture = Fixture::new();
    let mut app = Application::new(&fixture);
    app.surface.commit();
    app.sync();
    assert!(app.events.serial.is_some());
    app.attach();
    app.refused();
    fixture.wait_for((0, 0));
}

#[test]
fn a_serial_from_before_unmap_cannot_be_acknowledged_again() {
    let fixture = Fixture::new();
    let mut app = Application::new(&fixture);
    let old = app.configure();
    app.attach();
    app.sync();
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    app.surface.commit();
    app.sync();
    app.xdg.ack_configure(old);
    app.refused();
    fixture.wait_for((0, 0));
}

#[test]
fn disconnecting_one_client_preserves_another_clients_buffer() {
    let fixture = Fixture::new();
    let mut first = Application::new(&fixture);
    let mut second = Application::new(&fixture);
    first.configure();
    second.configure();
    first.attach();
    second.attach();
    first.sync();
    second.sync();
    fixture.wait_for((2, 2));
    drop(first);
    fixture.wait_for((1, 1));
    second.sync();
    assert_eq!(second.events.releases, 0);
}

#[test]
fn shutting_down_the_server_disconnects_live_clients_and_removes_the_socket() {
    let fixture = Fixture::new();
    let path = fixture.path.clone();
    let mut app = Application::new(&fixture);
    app.configure();
    app.attach();
    app.sync();
    drop(fixture);
    assert!(!path.exists());
    assert!(app.queue.roundtrip(&mut app.events).is_err());
}
