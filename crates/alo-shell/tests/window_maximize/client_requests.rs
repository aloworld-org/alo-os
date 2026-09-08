//! Wire requests share trusted policy, with explicit refusal and handshake responses.
use super::{Application, Fixture, ack, mapped, origin};
use smithay::backend::input::{ButtonState, KeyState};

#[test]
fn client_maximize_roundtrip_preserves_focus_and_restores_after_repeated_requests()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    let target = root.clone();
    f.backend(move |s| s.place_window(&target, (12, 13)))?;
    f.focus_surface(root.clone())?;
    f.render((33, 32), false, 1)?;
    app.sync();
    other.sync();
    let other_count = other.events.sizes.len();
    let count = app.events.sizes.len();
    app.toplevel.set_maximized();
    app.toplevel.set_maximized();
    app.sync();
    assert_eq!(app.events.sizes.len(), count + 2);
    assert_eq!(app.events.sizes.last(), Some(&(33, 32)));
    assert_eq!(app.events.maximized.last(), Some(&true));
    assert_eq!(app.events.activation.last(), Some(&true));
    ack(&mut app)?;
    assert_eq!(origin(&f, &root), (12.0, 13.0).into());
    f.key(30, KeyState::Pressed)?;
    f.key(30, KeyState::Released)?;
    app.attach_maximized();
    app.sync();
    other.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(other.events.keyboard.keys.is_empty());
    assert_eq!(other.events.sizes.len(), other_count);
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    let count = app.events.sizes.len();
    app.toplevel.unset_maximized();
    app.toplevel.unset_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.sizes.len(), count + 2);
    assert_eq!(app.events.maximized.last(), Some(&false));
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (12.0, 13.0).into());
    Ok(())
}

#[test]
fn client_maximize_premap_declines_at_initial_commit_and_unmap_forgets_intent()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    f.render((33, 32), false, 1)?;
    let mut app = Application::new(&f);
    app.toplevel.set_maximized();
    app.toplevel.unset_maximized();
    app.toplevel.set_maximized();
    app.sync();
    assert!(app.events.serial.is_none());
    app.configure();
    assert_eq!(app.events.wm_capabilities, [vec![2]]);
    assert_eq!(app.events.maximized, [false]);
    assert_eq!(app.events.sizes, [(0, 0)]);
    // After initial configure but before mapping, each request gets a refusal.
    app.toplevel.set_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.maximized, [false, false]);
    app.attach();
    app.sync();
    app.toplevel.set_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.maximized.last(), Some(&true));
    app.attach_maximized();
    app.sync();
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    let count = app.events.sizes.len();
    app.toplevel.set_maximized();
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    app.configure();
    assert_eq!(app.events.wm_capabilities.last(), Some(&vec![2]));
    assert_eq!(app.events.maximized.last(), Some(&false));
    assert_eq!(app.events.sizes.last(), Some(&(0, 0)));
    app.attach();
    app.sync();
    app.toplevel.unset_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.sizes.last(), Some(&(0, 0)));
    app.toplevel.set_maximized();
    app.sync();
    assert_eq!(app.events.maximized.last(), Some(&true));
    // Disconnect with an unacknowledged transaction; it cannot reach a new role.
    drop(app);
    f.wait_for((0, 0));
    let mut replacement = mapped(&f);
    replacement.toplevel.unset_maximized();
    ack(&mut replacement)?;
    assert_eq!(replacement.events.maximized.last(), Some(&false));
    assert_eq!(replacement.events.sizes.last(), Some(&(0, 0)));
    Ok(())
}

#[test]
fn client_maximize_refuses_unavailable_output_and_busy_resize_with_response()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    app.toplevel.set_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.maximized, [false, false]);
    assert_eq!(app.events.sizes.last(), Some(&(0, 0)));
    f.render((33, 32), false, 1)?;
    f.backend(|s| s.pointer_motion(4.0, 5.0, 1))?;
    f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?;
    app.sync();
    app.toplevel.resize(
        app.events.keyboard.seat.as_ref().ok_or("seat missing")?,
        app.events.pointer.button_serial,
        wayland_protocols::xdg::shell::client::xdg_toplevel::ResizeEdge::BottomRight,
    );
    app.sync();
    let count = app.events.sizes.len();
    app.toplevel.set_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.sizes.len(), count + 1);
    assert_eq!(app.events.maximized.last(), Some(&false));
    assert_eq!(app.events.resizing.last(), Some(&true));
    f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))?;
    ack(&mut app)?;
    app.surface.commit();
    app.sync();
    app.toplevel.set_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.maximized.last(), Some(&true));
    Ok(())
}

#[test]
fn client_maximize_refused_restore_preserves_maximized_mode_and_saved_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let target = root.clone();
    f.backend(move |s| s.place_window(&target, (9, 10)))?;
    f.render((33, 32), false, 1)?;
    app.toplevel.set_maximized();
    ack(&mut app)?;
    app.attach_maximized();
    app.sync();
    app.toplevel.set_min_size(1_000_001, 1);
    app.surface.commit();
    app.sync();
    let count = app.events.sizes.len();
    app.toplevel.unset_maximized();
    ack(&mut app)?;
    assert_eq!(app.events.sizes.len(), count + 1);
    assert_eq!(app.events.maximized.last(), Some(&true));
    assert_eq!(app.events.sizes.last(), Some(&(33, 32)));
    app.surface.commit();
    app.sync();
    assert_eq!(origin(&f, &root), (0.0, 0.0).into());
    app.toplevel.set_min_size(0, 0);
    app.surface.commit();
    app.sync();
    app.toplevel.unset_maximized();
    ack(&mut app)?;
    app.attach();
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(16, 16)));
    assert_eq!(origin(&f, &root), (9.0, 10.0).into());
    Ok(())
}
