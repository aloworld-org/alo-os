//! The nested compositor driven by a real client, through the parent's GLES.
//!
//! **Moved out of `examples/nested_check.rs` on 2026-09-26 so a gate can run
//! it.** It was the one sub-mode of that fixture with no callable shape — the
//! others were already functions here — and a check a test cannot call is a
//! check only a person running a command by hand ever performs. That is how a
//! stage count left at thirty after six stages were removed shipped through
//! nine gates twice.
//!
//! What it does is unchanged. The five switches it read out of
//! `std::env::args()` are [`Flags`] now, so the fixture passes what a person
//! typed and a test passes what it means to check.

use crate::{
    Fixture, application, nested_control_frame_check, nested_reader_frame_check, popup_check,
};

/// Which of the optional checks this run makes.
///
/// Defaulting every one to *off* keeps the plain run the plain run: a switch
/// added here does not silently join every existing caller.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    /// Drive the default cursor's pixels.
    pub cursor: bool,
    /// Enable the popup protocol and exercise popups.
    pub popups: bool,
    /// Compose and route the native window-control strip.
    pub controls: bool,
    /// Compose the paged reader over that strip.
    pub reader: bool,
    /// Print each submission's timings.
    pub trace: bool,
}

// This module is compiled into two crate roots that want different halves of
// this: the fixture reads a person's command line, the gate names what it means
// to check. Each root therefore sees the other's constructor as dead, and both
// are live where they are used.
#[expect(
    dead_code,
    reason = "one of these two constructors is unused in each of the two roots this module is compiled into"
)]
impl Flags {
    /// The switches this process was started with.
    #[must_use]
    pub fn from_args() -> Self {
        let has = |name: &str| std::env::args().any(|arg| arg == name);
        Self {
            cursor: has("--cursor"),
            popups: has("--popups"),
            controls: has("--controls"),
            reader: has("--reader"),
            trace: has("--trace"),
        }
    }

    /// Every optional check this fixture has, for a caller that wants them all.
    #[must_use]
    pub fn everything() -> Self {
        Self {
            cursor: true,
            popups: true,
            controls: true,
            reader: true,
            trace: false,
        }
    }
}

/// Drive actual client buffers through GLES and require callbacks and teardown.
///
/// # Errors
/// Any graphics initialisation, submission or protocol failure, and a client
/// that has not finished within its deadline.
pub fn run(flags: Flags) -> Result<(), Box<dyn std::error::Error>> {
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
    server.enable_pointer()?;
    server.render(&mut nested, 0)?;
    let path = server.socket_path().to_owned();
    let cursor_check = flags.cursor;
    let popup_check = flags.popups;
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
    let controls_check = flags.controls;
    let mut controls_checked = false;
    let reader_check = flags.reader;
    let mut reader_checked = false;
    let trace = flags.trace;
    while !client.is_finished() {
        if start.elapsed() > Duration::from_secs(10) {
            return Err("client deadline exceeded".into());
        }
        if trace {
            eprintln!("Nested trace {:?}: before pump", start.elapsed());
        }
        if cursor_check {
            nested.pump()?;
            server.pointer_motion(26.0, 35.0, start.elapsed().as_millis() as u32)?;
        } else {
            nested.pump_seat(&mut server)?;
        }
        if trace {
            eprintln!("Nested trace {:?}: after pump", start.elapsed());
        }
        server.dispatch()?;
        if trace {
            eprintln!(
                "Nested trace {:?}: after dispatch, {} roots",
                start.elapsed(),
                server.mapped_surfaces().count()
            );
        }
        if reader_check && !reader_checked && server.mapped_surfaces().next().is_some() {
            nested_reader_frame_check::run(
                &mut server,
                &mut nested,
                start.elapsed().as_millis() as u32,
            )?;
            reader_checked = true;
        }
        if controls_check && !controls_checked && server.mapped_surfaces().next().is_some() {
            nested_control_frame_check::run(
                &mut server,
                &mut nested,
                start.elapsed().as_millis() as u32,
            )?;
            controls_checked = true;
        }
        rendered += server.render(&mut nested, start.elapsed().as_millis() as u32)?;
        if trace {
            eprintln!("Nested trace {:?}: after render", start.elapsed());
        }
        thread::sleep(Duration::from_millis(4));
    }
    client.join().map_err(|_| "client assertion failed")?;
    server.dispatch()?;
    assert_eq!(server.toplevel_count(), 0);
    assert!(server.popup_surfaces().is_empty());
    assert_eq!(server.render(&mut nested, 10000)?, 0);
    assert!(rendered > 0);
    assert!(!controls_check || controls_checked);
    assert!(!reader_check || reader_checked);
    println!(
        "Nested GLES submissions included {rendered} client surfaces; unmap/remap, refusal and disconnect passed; physical display unverified"
    );
    Ok(())
}
