//! Client requests cannot retain pre-map intent or target another connection.
use super::{Application, Fixture, mapped, minimize};
use smithay::backend::input::KeyState;

#[test]
fn client_minimize_premap_refusal_and_remap_do_not_retain_intent()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let mut app = Application::new(&f);
    app.toplevel.set_minimized();
    app.sync();
    assert!(app.events.serial.is_none());
    app.configure();
    crate::support::assert_window_capabilities(&app.events.wm_capabilities, 1);
    let count = app.events.sizes.len();
    app.toplevel.set_minimized();
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    app.attach();
    app.sync();
    assert_eq!(f.backend(|s| s.mapped_surfaces().count()), 1);
    app.toplevel.set_minimized();
    app.sync();
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 1);
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 0);
    app.toplevel.set_minimized();
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    app.configure();
    crate::support::assert_window_capabilities(&app.events.wm_capabilities, 2);
    app.attach();
    app.sync();
    assert_eq!(f.backend(|s| s.mapped_surfaces().count()), 1);
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 0);
    Ok(())
}

#[test]
fn client_minimize_duplicates_commits_and_disconnect_preserve_other_client()
-> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    f.focus_surface(root.clone())?;
    assert!(f.key(30, KeyState::Pressed)?);
    app.toplevel.set_minimized();
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 1);
    assert_eq!(app.events.activation.last(), Some(&false));
    assert!(matches!(
        f.focus_surface(root.clone()),
        Err(alo_shell::InputError::Unmapped)
    ));
    f.focus(Some(0))?;
    other.sync();
    let other_count = other.events.sizes.len();
    let count = app.events.sizes.len();
    app.toplevel.set_minimized();
    app.toplevel.set_minimized();
    app.surface.frame(&app.queue.handle(), ());
    app.attach_resized();
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(app.events.keyboard.leaves, 1);
    assert_eq!(f.render((80, 80), false, 10)?, 1);
    app.sync();
    assert!(app.events.frames.is_empty());
    assert!(!f.key(30, KeyState::Released)?);
    assert!(f.key(31, KeyState::Pressed)?);
    assert!(f.key(31, KeyState::Released)?);
    other.sync();
    assert_eq!(other.events.keyboard.keys.len(), 2);
    assert_eq!(other.events.sizes.len(), other_count);
    assert!(minimize(&f, &root, false)?);
    assert_eq!(f.render((80, 80), false, 11)?, 2);
    app.sync();
    assert_eq!(app.events.frames, [11]);
    assert_eq!(app.events.activation.last(), Some(&false));
    app.toplevel.set_minimized();
    app.sync();
    drop(app);
    f.wait_for((1, 1));
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 0);
    let _replacement = mapped(&f);
    assert_eq!(f.backend(|s| s.mapped_surfaces().count()), 2);
    Ok(())
}
