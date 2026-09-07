//! Explicit WSLg integration fixture; never installed as the person's session.

#[cfg(target_os = "linux")]
#[path = "../tests/support/application.rs"]
mod application;

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
    server.render(&mut nested, 0)?;
    let path = server.socket_path().to_owned();
    let client = thread::spawn(move || {
        let fixture = Fixture { path };
        let mut app = application::Application::new(&fixture);
        app.sync();
        assert_eq!(app.events.outputs, 1);
        assert!(app.events.keyboard.keymap.starts_with("xkb_keymap"));
        assert_eq!(app.events.keyboard.repeat, Some((25, 600)));
        assert_eq!(app.events.modes.last(), Some(&(320, 200)));
        app.configure();
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
        app.surface.attach(None, 0, 0);
        app.surface.commit();
        while app.events.membership.1 < 2 {
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
        nested.pump_keyboard(&mut server)?;
        server.dispatch()?;
        rendered += server.render(&mut nested, start.elapsed().as_millis() as u32)?;
        thread::sleep(Duration::from_millis(4));
    }
    client.join().map_err(|_| "client assertion failed")?;
    server.dispatch()?;
    assert_eq!(server.toplevel_count(), 0);
    assert_eq!(server.render(&mut nested, 10000)?, 0);
    assert!(rendered > 0);
    println!(
        "Nested GLES submissions included {rendered} client surfaces; unmap/remap, refusal and disconnect passed; physical display unverified"
    );
    Ok(())
}
