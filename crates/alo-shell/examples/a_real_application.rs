//! A real application, built by somebody else, on alo OS's own compositor.
//!
//! Every other fixture in this folder draws alo OS's own surfaces, or drives a
//! test client written here to exercise one protocol. **This one starts the
//! compositor, hands its socket to a program nobody on this project wrote, and
//! photographs what arrives.**
//!
//! It exists because the owner asked a question none of the others answer:
//!
//! > Windows will be for apps not just windows so we need to design them and
//! > try installing apps and see how they look on the canvas
//!
//! What the canvas walk proves is that the machinery works — three surfaces
//! open, drag, resize, pan, zoom. What it cannot show is **what an application
//! looks like**, because its three surfaces are test rectangles a few pixels
//! across. A real client answers that and nothing else does.
//!
//! # How to run it
//!
//! ```text
//! cargo run -p alo-shell --example a_real_application -- --save-to <folder>
//! cargo run -p alo-shell --example a_real_application -- --run foot --save-to <folder>
//! ```
//!
//! `--run` names the program; the default is `foot`, a small Wayland terminal.
//! Any Wayland client will do, and one that refuses tells you something too.
//!
//! # What it is not
//!
//! **Not a session.** The program is started by this fixture with the socket in
//! its environment, which is not how alo OS will launch anything — there is no
//! grant, no adapter and no record. It is a developer pointing one program at
//! one socket.
//!
//! **Not a panel.** Same as every fixture here: a nested surface under a
//! Wayland parent, software rendered under WSLg. `support/saving_frames.rs`
//! carries the rest of that, and why a saved frame is not the screenshot
//! promise.

/// Writing a drawn frame out.
#[cfg(target_os = "linux")]
#[path = "support/saving_frames.rs"]
mod saving_frames;

/// Report any failure, including the one where the program will not start.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("a real application: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Start the compositor, start the program, draw what it sends.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::{Nested, Server};
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };

    let asked = what_was_asked()?;
    let mut saving = asked.saving;

    let mut nested = Nested::new("alo OS, with a real application", (1366, 768))?;
    if saving.is_some() {
        nested.keep_each_frame(true);
    }
    nested.pump()?;

    // The socket lives in a directory only this user can enter, for the same
    // reason the integration fixture's does: a Wayland socket is a door into
    // this session's surfaces and input.
    let runtime = tempfile::tempdir()?;
    fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "alo-with-an-application",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..Default::default()
        },
    )?;
    server.enable_pointer()?;
    server.render(&mut nested, 0)?;
    let socket = server.socket_path().to_owned();
    println!("the compositor is listening on {}", socket.display());

    // The program is told where to connect and nothing else about us.
    let mut child = std::process::Command::new(&asked.program)
        .env("WAYLAND_DISPLAY", &socket)
        .env_remove("DISPLAY")
        .spawn()
        .map_err(|why| format!("could not start {}: {why}", asked.program))?;
    println!("started {}", asked.program);

    // Draw until the program has put something on the screen, or until the
    // patience runs out. A deadline rather than a fixed number of frames: how
    // long a program takes to open its first window is its business.
    let deadline = Instant::now() + Duration::from_secs(asked.seconds);
    let mut drawn = 0_usize;
    let mut seen = 0_usize;
    let mut saved_at = Vec::new();
    while Instant::now() < deadline {
        nested.pump_seat(&mut server)?;
        server.dispatch()?;
        // **With whatever chrome this release has**, rather than the bare
        // surface `Server::render` draws. The question being asked here is what
        // an application looks like, and a window drawn with no controls
        // because nobody asked for them would answer it wrongly — it would show
        // a frameless rectangle and let somebody conclude there is no window
        // frame, when what is true is that this fixture did not request one.
        let first = server.mapped_surfaces().next().cloned();
        drawn += match &first {
            // **On the window, not in the corner of the screen.**
            //
            // This passed `(0, 0)` until 2026-10-09 and the controls drew at
            // the screen's top-left while the application sat elsewhere — the
            // owner saw a picture of it and said they belong on the app
            // window. `WindowControlLayout::new` lays its three out starting
            // **at** the origin it is handed, so the origin is the whole of
            // where they go and this fixture was handing it the wrong one.
            //
            // **Top right, which is where the design puts them.** `Glass /
            // Window controls` sits at x=794 of a 936-wide window, inset 8
            // from the top — so the origin is the window's right edge, less
            // the strip, less the inset.
            Some(surface) => {
                let at = where_the_controls_go(surface);
                nested.render_window_controls(
                    &mut server,
                    Some((surface, at)),
                    asked.scheme,
                    drawn_as_time(drawn),
                )?
            }
            None => server.render(&mut nested, drawn_as_time(drawn))?,
        };
        let mapped = server.mapped_surfaces().count();
        // One picture each time the number of windows changes, plus a last one,
        // so a folder of pictures is the moments rather than the frames.
        if mapped != seen {
            println!("{mapped} window(s) from {}", asked.program);
            seen = mapped;
            if let (true, Some(saving)) = (mapped > 0, saving.as_mut())
                && let Some(frame) = nested.the_frame_just_drawn()
            {
                saved_at
                    .push(saving.frame(&format!("{} with {mapped} window", asked.program), frame)?);
            }
        }
        std::thread::sleep(Duration::from_millis(16));
    }

    // A last picture of wherever it got to, named so, because the interesting
    // case is often the one where nothing arrived.
    if let Some(saving) = saving.as_mut()
        && let Some(frame) = nested.the_frame_just_drawn()
    {
        saved_at.push(saving.frame(&format!("{} at the end", asked.program), frame)?);
    }

    // Ended rather than left running: this fixture started it, so this fixture
    // is what stops it.
    let _ = child.kill();
    let _ = child.wait();

    println!("{drawn} surfaces drawn, {seen} window(s) at the end");
    for at in &saved_at {
        println!("  {}", at.display());
    }
    if seen == 0 {
        return Err(format!(
            "{} connected to nothing, or drew nothing, within {} seconds. That is a \
             result rather than a crash, and the pictures show what the compositor had",
            asked.program, asked.seconds
        )
        .into());
    }
    println!(
        "software rendered on a virtual output: what the renderer drew, not what a screen showed"
    );
    Ok(())
}

/// What this run was asked for.
#[cfg(target_os = "linux")]
struct Asked {
    /// The program to start.
    program: String,
    /// How long to keep drawing for.
    seconds: u64,
    /// Light or dark, for the chrome drawn around the window.
    scheme: alo_appearance::Scheme,
    /// Where the pictures go, if anywhere.
    saving: Option<saving_frames::SaveTo>,
}

/// Read `--run`, `--seconds` and `--save-to` off the arguments.
///
/// `--save-to` is parsed by [`saving_frames::SaveTo`] so that one file owns
/// what that flag means for every fixture that takes it.
#[cfg(target_os = "linux")]
fn what_was_asked() -> Result<Asked, Box<dyn std::error::Error>> {
    let mut program = "foot".to_owned();
    let mut seconds = 10;
    let mut rest = Vec::new();
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--run" => program = arguments.next().ok_or("--run needs a program after it")?,
            "--seconds" => {
                seconds = arguments
                    .next()
                    .ok_or("--seconds needs a number after it")?
                    .parse()?;
            }
            other => rest.push(other.to_owned()),
        }
    }
    Ok(Asked {
        program,
        seconds,
        scheme: alo_appearance::Scheme::Light,
        saving: saving_frames::SaveTo::from_these(rest)?,
    })
}

/// A frame counter as the milliseconds a callback wants.
///
/// The clients being drawn here are somebody else's and they are entitled to a
/// time that moves forward; nothing in this fixture depends on it being the
/// wall clock.
#[cfg(target_os = "linux")]
fn drawn_as_time(drawn: usize) -> u32 {
    u32::try_from(drawn.saturating_mul(16) % u32::MAX as usize).unwrap_or(0)
}

/// How wide the shell's three window controls are together.
///
/// `window_controls.rs` lays them out at offsets 0, 36 and 72, each 32 wide,
/// so the strip spans 0..104. **Read off that file rather than guessed**, and
/// it is not the design's 140 — the design's controls are 44 wide where the
/// code's are 32, which `crates/alo-dock`'s own measurement settles as a 32
/// glyph inside a 48 target rather than a disagreement.
#[cfg(target_os = "linux")]
const THE_STRIP_IS_WIDE: i32 = 104;

/// The design's inset of the controls from the window's top and right edges.
#[cfg(target_os = "linux")]
const INSET: i32 = 8;

/// Where this window's controls belong: its own top right, inset.
///
/// **This arithmetic should not be a fixture's.** Where the controls sit on a
/// window is the design's answer and the shell's to apply, and every caller
/// doing it again is every caller getting a chance to do it differently. It is
/// here because `alo-shell` exports `window_buffer_origin` and **nothing
/// public for a window's width** — so no caller outside the crate can place
/// them to the design without reaching past the API, which is what this does.
///
/// Recorded as a gap rather than left as a trick.
#[cfg(target_os = "linux")]
fn where_the_controls_go(
    surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
) -> (i32, i32) {
    // **The window, not the surface.** A client drawing its own decorations
    // puts its shadow in the surface's margin, so the buffer starts above and
    // left of anything a person calls the window's corner. Measured here on
    // 2026-10-09: placed from the buffer, these drew about fifty pixels above
    // the calculator's visible top edge and read as belonging to the screen.
    let window = alo_shell::where_a_window_is(surface);
    // A window narrower than its own controls keeps them at its left edge
    // rather than hanging them off its side.
    let inset = (window.size.w - THE_STRIP_IS_WIDE - INSET).max(0);
    (window.loc.x + inset, window.loc.y + INSET)
}
