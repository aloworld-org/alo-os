//! Explicit WSLg integration fixture; never installed as the person's session.

#[cfg(target_os = "linux")]
#[path = "../tests/support/application.rs"]
mod application;

#[cfg(target_os = "linux")]
#[path = "support/popup_check.rs"]
mod popup_check;

#[cfg(target_os = "linux")]
#[path = "support/grab_check.rs"]
mod grab_check;

#[cfg(target_os = "linux")]
#[path = "support/offscreen_check.rs"]
mod offscreen_check;

#[cfg(target_os = "linux")]
#[path = "support/offscreen_client.rs"]
mod offscreen_client;

#[cfg(target_os = "linux")]
#[path = "support/default_cursor_check.rs"]
mod default_cursor_check;

#[cfg(target_os = "linux")]
#[path = "support/window_raise_check.rs"]
mod window_raise_check;

#[cfg(target_os = "linux")]
#[path = "support/interactive_resize_check.rs"]
mod interactive_resize_check;
#[cfg(target_os = "linux")]
#[path = "support/resize_geometry_check.rs"]
mod resize_geometry_check;
#[cfg(target_os = "linux")]
#[path = "support/window_placement_check.rs"]
mod window_placement_check;
#[cfg(target_os = "linux")]
#[path = "support/window_size_check.rs"]
mod window_size_check;
#[cfg(target_os = "linux")]
#[path = "support/window_switch_check.rs"]
mod window_switch_check;

/// Socket location shared with the real protocol-client fixture.
#[cfg(target_os = "linux")]
pub struct Fixture {
    /// Private test display, distinct from the parent WSLg socket.
    pub path: std::path::PathBuf,
}

/// Main-thread graphics initialization is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nested compositor check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Drive actual client buffers through GLES and require callbacks and teardown.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::{Nested, Server};
    if std::env::args().any(|arg| arg == "--offscreen") {
        return offscreen_check::run();
    }
    if std::env::args().any(|arg| arg == "--pointer-release-grabs") {
        return grab_check::run(false, true);
    }
    if std::env::args().any(|arg| arg == "--grabs") {
        return grab_check::run(false, false);
    }
    if std::env::args().any(|arg| arg == "--keyboard-grabs") {
        return grab_check::run(true, false);
    }
    if std::env::args().any(|arg| arg == "--keyboard-release-grabs") {
        return grab_check::run(true, true);
    }
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        thread,
        time::{Duration, Instant},
    };
    let mut nested = Nested::new("alo nested integration check", (320, 200))?;
    nested.pump()?;
    let runtime = tempfile::tempdir()?;
    fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "nested-check",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..Default::default()
        },
    )?;
    server.enable_pointer()?;
    server.render(&mut nested, 0)?;
    let path = server.socket_path().to_owned();
    let cursor_check = std::env::args().any(|arg| arg == "--cursor");
    let popup_check = std::env::args().any(|arg| arg == "--popups");
    if popup_check {
        server.enable_popup_protocol();
    }
    let client = thread::spawn(move || {
        let fixture = Fixture { path };
        let mut app = application::Application::new(&fixture);
        app.sync();
        assert_eq!(app.events.outputs, 1);
        assert!(app.events.keyboard.keymap.starts_with("xkb_keymap"));
        assert_eq!(app.events.keyboard.repeat, Some((25, 600)));
        assert_eq!(
            app.events.keyboard.capabilities,
            Some(
                wayland_client::protocol::wl_seat::Capability::Keyboard
                    | wayland_client::protocol::wl_seat::Capability::Pointer
            )
        );
        assert_eq!(app.events.modes.last(), Some(&(320, 200)));
        use wayland_client::protocol::wl_output;
        assert!(app.events.output_events.iter().any(|event| matches!(event,
            wl_output::Event::Name { name } if name == "alo-nested")));
        assert!(app.events.output_events.iter().any(|event| matches!(event,
            wl_output::Event::Geometry { physical_width: 0, physical_height: 0, make, model, .. }
            if make == "alo" && model == "nested")));
        assert!(app.events.output_events.iter().any(|event| matches!(
            event,
            wl_output::Event::Mode {
                width: 320,
                height: 200,
                refresh: 0,
                ..
            }
        )));
        println!("Nested output wire name, unknown physical size and refresh verified");
        app.configure();
        // Input regions must not suppress graphical buffer submission.
        app.empty_input();
        let (child, child_role) = app.child((24, 32));
        let (hidden, hidden_role) = app.child((1000, 1000));
        app.surface.frame(&app.queue.handle(), ());
        app.attach();
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.events.frames.len() < 2 {
            app.sync();
            assert!(
                Instant::now() < deadline,
                "no callback after GLES submission"
            );
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(
            app.events.frames.len(),
            2,
            "offscreen child must not receive a callback"
        );
        assert_eq!(app.events.membership, (2, 0));
        println!(
            "SHM client rendered: 16x16 ARGB; output 320x200; callback {:?}",
            app.events.frames
        );
        if popup_check {
            popup_check::check(&mut app);
        }
        if cursor_check {
            let prior_popups = 4 * usize::from(popup_check);
            let prior_popup_frames = 6 * usize::from(popup_check);
            while app.events.pointer.enters.is_empty() {
                app.sync();
                assert!(Instant::now() < deadline, "no scripted pointer enter");
            }
            let cursor = app.cursor((i32::MIN, i32::MAX));
            for _ in 0..5 {
                app.sync();
                thread::sleep(Duration::from_millis(5));
            }
            assert_eq!(
                app.events.frames.len(),
                2 + prior_popup_frames,
                "offscreen cursor callback"
            );
            app.set_cursor(app.events.pointer.serial, Some(&cursor), (2, 3));
            while app.events.frames.len() < 3 + prior_popup_frames {
                app.sync();
                assert!(Instant::now() < deadline, "no GLES cursor callback");
            }
            assert_eq!(app.events.membership, (3 + prior_popups, prior_popups));
            cursor.attach(None, 0, 0);
            cursor.commit();
            while app.events.membership.1 < 1 + prior_popups {
                app.sync();
                assert!(Instant::now() < deadline, "cursor unmap not presented");
            }
            cursor.destroy();
            app.sync();
            println!(
                "Cursor SHM tree submitted at scripted pointer (26,35), hotspot (2,3); callback and output leave on unmap passed"
            );
        }
        app.surface.attach(None, 0, 0);
        app.surface.commit();
        while app.events.membership.1 < 2 + usize::from(cursor_check) + 4 * usize::from(popup_check)
        {
            app.sync();
            assert!(Instant::now() < deadline, "no output leave on unmap");
            thread::sleep(Duration::from_millis(2));
        }
        // Children retain their buffer until explicitly destroyed.
        child_role.destroy();
        child.destroy();
        hidden_role.destroy();
        hidden.destroy();
        app.sync();
        assert!(app.events.releases > 0);
        app.configure();
        app.attach();
        app.sync();
        app.toplevel.destroy();
        app.xdg.destroy();
        app.surface.destroy();
        app.sync();
        // A malformed client is disconnected while this same backend keeps running.
        let mut invalid = application::Application::new(&fixture);
        invalid.attach();
        invalid.refused();
        let mut abrupt = application::Application::new(&fixture);
        abrupt.configure();
        abrupt.attach();
        abrupt.sync();
        drop(abrupt);
    });
    let start = Instant::now();
    let mut rendered = 0;
    while !client.is_finished() {
        if start.elapsed() > Duration::from_secs(10) {
            return Err("client deadline exceeded".into());
        }
        if cursor_check {
            nested.pump()?;
            server.pointer_motion(26.0, 35.0, start.elapsed().as_millis() as u32)?;
        } else {
            nested.pump_seat(&mut server)?;
        }
        server.dispatch()?;
        rendered += server.render(&mut nested, start.elapsed().as_millis() as u32)?;
        thread::sleep(Duration::from_millis(4));
    }
    client.join().map_err(|_| "client assertion failed")?;
    server.dispatch()?;
    assert_eq!(server.toplevel_count(), 0);
    assert!(server.popup_surfaces().is_empty());
    assert_eq!(server.render(&mut nested, 10000)?, 0);
    assert!(rendered > 0);
    println!(
        "Nested GLES submissions included {rendered} client surfaces; unmap/remap, refusal and disconnect passed; physical display unverified"
    );
    Ok(())
}
