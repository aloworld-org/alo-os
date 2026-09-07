//! Scripted popup grab through a real WSLg graphics backend.
use crate::{Fixture, application};

/// Submit a grabbing popup, route input and verify outside-click dismissal.
pub fn run(keyboard: bool) -> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::{Nested, Server};
    use smithay::{
        backend::input::{ButtonState, KeyState},
        input::keyboard::XkbConfig,
    };
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };
    use wayland_client::Proxy;

    let mut nested = Nested::new("alo popup grab integration check", (320, 200))?;
    let runtime = tempfile::tempdir()?;
    fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "popup-grab-check",
        XkbConfig {
            layout: "us",
            ..Default::default()
        },
    )?;
    server.enable_pointer()?;
    server.enable_popup_protocol();
    server.render(&mut nested, 0)?;
    let fixture = Fixture {
        path: server.socket_path().to_owned(),
    };
    let (send, receive) = mpsc::channel::<(bool, mpsc::Sender<()>)>();
    let client = thread::spawn(move || {
        let mut app = application::Application::new(&fixture);
        app.configure();
        app.attach();
        app.sync();
        let step = |outside| {
            let (done, wait) = mpsc::channel();
            assert!(send.send((outside, done)).is_ok());
            assert!(wait.recv_timeout(Duration::from_secs(3)).is_ok());
        };
        step(false);
        app.sync();
        let (surface, xdg, role) = app.popup(true, 20);
        app.grab_popup(
            &role,
            if keyboard {
                app.events.keyboard.key_serial
            } else {
                app.events.pointer.button_serial
            },
        );
        surface.commit();
        app.sync();
        app.ack_popup(&xdg);
        app.attach_popup(&surface);
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.events.frames.is_empty() {
            app.sync();
            assert!(Instant::now() < deadline, "no submitted popup frame");
        }
        assert_eq!(app.events.popups.done, 0);
        assert_eq!(app.events.membership, (2, 0));
        assert_eq!(
            app.events.keyboard.surfaces.last(),
            Some(&surface.id().protocol_id())
        );
        step(true);
        while app.events.membership.1 == 0 {
            app.sync();
            assert!(Instant::now() < deadline, "no popup output leave");
        }
        assert_eq!(app.events.popups.done, 1);
        assert_eq!(
            app.events.keyboard.surfaces.last(),
            Some(&app.surface.id().protocol_id())
        );
        assert_eq!(app.events.keyboard.keys.len(), if keyboard { 4 } else { 2 });
        assert_eq!(
            app.events.pointer.buttons.len(),
            if keyboard { 0 } else { 2 },
            "outside click leaked"
        );
        println!(
            "Grabbed popup GLES callback/output enter; keyboard focus/key/release; outside click consumed, popup_done/output leave and parent focus restored"
        );
    });
    let start = Instant::now();
    while !client.is_finished() {
        if start.elapsed() > Duration::from_secs(10) {
            return Err("client deadline exceeded".into());
        }
        // Deliberately scripted trusted input; physical parent input is not measured.
        nested.pump()?;
        for (outside, done) in receive.try_iter() {
            if outside {
                assert!(server.keyboard_key(30, KeyState::Pressed, 3)?);
                server.pointer_motion(300.0, 190.0, 4)?;
                assert!(!server.pointer_button(0x110, ButtonState::Pressed, 5)?);
                assert!(!server.pointer_button(0x110, ButtonState::Released, 6)?);
            } else if keyboard {
                let root = server.mapped_surfaces().next().cloned();
                server.keyboard_focus(root.as_ref())?;
                assert!(server.keyboard_key(28, KeyState::Pressed, 2)?);
            } else {
                server.pointer_motion(1.0, 1.0, 1)?;
                assert!(server.pointer_button(0x110, ButtonState::Pressed, 2)?);
            }
            done.send(())?;
        }
        server.dispatch()?;
        server.render(&mut nested, start.elapsed().as_millis() as u32)?;
        thread::sleep(Duration::from_millis(4));
    }
    client.join().map_err(|_| "client assertion failed")?;
    server.dispatch()?;
    assert!(server.popup_surfaces().is_empty());
    assert_eq!(server.toplevel_count(), 0);
    assert_eq!(server.render(&mut nested, 10000)?, 0);
    println!("Grab client disconnect cleanup passed; physical display/input unverified");
    Ok(())
}
