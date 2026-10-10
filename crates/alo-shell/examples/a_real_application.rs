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
//! # Two programs look like one, and that is a finding rather than a bug here
//!
//! `--run` may be given more than once. Run twice, the compositor reports two
//! windows and draws both — and the picture shows one, because **nothing
//! places a newly mapped window.** Every client lands at the origin and they
//! sit exactly on top of each other.
//!
//! `crate::window_placement::set` exists and is called from
//! `canvas_fixed_controls`; what has no caller is anything that places a
//! window *when it maps*. So this fixture cannot yet show two applications
//! side by side on the canvas, and a picture of it is a true picture of what
//! the compositor does rather than a fault in the fixture.
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
    use alo_access::TurnedOn;
    use alo_appearance::{Appearance, TimeOfDay};
    use alo_dock::Dock;
    use alo_egress::Indicator;
    use alo_indicator::{Drew, Indicating};
    use alo_shell::{
        Cursor, DesktopFrame, DesktopLook, EgressStatus, FillingWindow, Nested, RunningWindow,
        Server, WindowControlLabels,
    };
    use alo_strings::{Direction, Strings};
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };

    let asked = what_was_asked()?;
    let mut saving = asked.saving;

    #[expect(
        clippy::cast_sign_loss,
        reason = "THE_OUTPUT_IS is a positive extent this fixture chose"
    )]
    let wanted = (THE_OUTPUT_IS.0 as u32, THE_OUTPUT_IS.1 as u32);
    let mut nested = Nested::new("alo OS, with a real application", wanted)?;
    if saving.is_some() {
        nested.keep_each_frame(true);
    }
    nested.pump()?;

    // The socket lives in a directory only this user can enter, for the same
    // reason the integration fixture's does: a Wayland socket is a door into
    // this session's surfaces and input.
    // Everything the desktop is drawn from. A machine with nothing open and
    // nobody signed in beyond this fixture: the readings are closed, nothing is
    // in use, nothing is put aside, and the only window is the one the program
    // about to start will map. Midday rather than the evening schedule, because
    // the question here is what an application looks like and the light scheme
    // is the one the design is drawn in.
    let strings = Strings::of(alo_saying::everything_this_machine_can_say()?);
    let appearance = Appearance::shipped();
    let midday = TimeOfDay::checked(12, 0).map_err(|error| format!("{error:?}"))?;
    let reading = Direction::LeftToRight;
    let dock = Dock::shipped();
    // The surface is a background per display and there is no other accessor,
    // so a name is needed; with one display it is only a key.
    let the_one_display = alo_appearance::DisplayId::named("the-one-display")
        .map_err(|error| format!("{error:?}"))?;
    let closed_running = RunningWindow::closed();
    let closed_filling = FillingWindow::closed();
    // **Told that nothing is leaving, because the frame is refused otherwise.**
    // `crate::scene_drawing` will not draw a desktop whose indicator has not
    // been told what is going out — the first law held by the drawing path
    // rather than by a reviewer. *Nothing* still has to be said; silence is
    // the one answer it will not take. The first run of this fixture failed
    // exactly here, with *the egress indicator has not been told what is
    // leaving*, which is the refusal working.
    let mut quiet = EgressStatus::on_an_output();
    if let Drew::Refused(refused) =
        Indicating::nowhere().show(Some(&mut quiet), &Indicator::default())
    {
        return Err(refused.said(&strings).into_text().into());
    }
    let mut labels = WindowControlLabels::new()?;
    // The shell's own faces: Manrope for what alo says, Inter behind it so a
    // script Manrope does not cover still draws.
    let mut faces = cosmic_text::FontSystem::new();
    faces
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    faces
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
    let division = alo_dividing::Division::of(alo_dividing::Area::of(
        alo_dividing::area::Point::at(0, 0),
        alo_dividing::area::Size::of(1366, 768),
    )?);

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

    // Each program is told where to connect and nothing else about us.
    let mut started = Vec::new();
    for program in &asked.programs {
        started.push(
            std::process::Command::new(program)
                .env("WAYLAND_DISPLAY", &socket)
                .env_remove("DISPLAY")
                .spawn()
                .map_err(|why| format!("could not start {program}: {why}"))?,
        );
        println!("started {program}");
    }
    let named = asked.programs.join(" and ");

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
        // **alo's external window edge, which replaces the old tiles.**
        //
        // This drew `WindowControlLayout` until 2026-10-09 — 32 by 32 boxes
        // with borders, inside the window. The owner looked at a terminal
        // wearing them and said they are not the controls we discussed, and
        // `docs/design/the-external-window-edge.md` forbids them by name:
        // *no permanent square button tiles*.
        //
        // The edge is above the window and outside its content, so nothing of
        // alo's covers anything the application drew.
        let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
        // **The host's answer, asked once for this frame.** §5 of
        // `docs/design/the-external-window-edge.md` is the shell's and not a
        // fixture's: `Server::the_edges_were_told_where_the_pointer_is` reads
        // the pointer and advances one `alo_dock::Revealing` per window.
        server.the_edges_were_told_where_the_pointer_is();
        let edges: Vec<_> = roots
            .iter()
            .filter_map(|surface| {
                let window = alo_shell::where_a_window_is(surface);
                // **This window's own answer, not a literal.** This passed
                // `true` until 2026-10-10, under a comment saying a fixture has
                // no pointer to reveal an edge with — true of a fixture and the
                // reason it was written, and it made this the only caller of
                // `edge_of` outside a test and the only one that could not be
                // wrong. A walk that moves the pointer onto an edge now sees it
                // reveal, and one that does not sees the grip, which is what a
                // person with no mouse on that window sees.
                let revealed = server.is_this_windows_edge_revealed(surface);
                let edge = alo_shell::edge_of(window, asked.decorations, revealed);
                let title = server.the_name_of(surface);
                alo_shell::EdgePicture::of(
                    &edge,
                    alo_shell::Pointing::default(),
                    title.its_own_words(),
                    &mut faces,
                    (THE_OUTPUT_IS.0, THE_OUTPUT_IS.1),
                    100,
                )
            })
            .collect();
        drawn += nested
            .submit_with_desktop(
                &roots,
                &[],
                &Cursor::Default,
                // **Every window's edge, not the first.** This passed
                // `edges.first()` until 2026-10-10 — correct for the signature
                // it had, which took one picture because the layer it went into
                // held one of the lock screen, the reader or the sign-in screen.
                // Two windows open meant one edge drawn and the other built and
                // discarded, silently.
                &edges,
                &mut labels,
                DesktopFrame {
                    // **The Dock is told what is open, which is the chain two
                    // lanes built today.** `Server::the_windows_the_dock_sees`
                    // turns the mapped surfaces into what `alo-dock`
                    // understands, `Holding::showing` groups them by
                    // application, and `dock_holds` carries the count to the
                    // drawing - which was handed a literal `0` until 2026-10-09.
                    //
                    // `Holding::nothing()` because nothing stores what a person
                    // pinned yet, so this counts the applications that have a
                    // window open. True, and a Dock of what is running is what
                    // a machine with no saved pins should show.
                    dock_holds: server.how_many_the_dock_holds(&alo_dock::Holding::nothing()),
                    display_scale: 100,
                    dock: &dock,
                    look: DesktopLook::of(
                        &appearance,
                        &TurnedOn::nothing(),
                        midday,
                        reading,
                        &the_one_display,
                    ),
                    strings: &strings,
                    egress: &quiet,
                    running: &closed_running,
                    filling: &closed_filling,
                    in_use: &[],
                    notifications: &[],
                    capturing: None,
                    division: &division,
                    offer: &alo_dividing::Offer::Nothing,
                    windows: &[],
                    put_aside: nothing_put_aside(),
                    panel_is_revealed: true,
                    filling_the_screen: false,
                },
                None,
                None,
            )?
            .len();
        let mapped = server.mapped_surfaces().count();
        // One picture each time the number of windows changes, plus a last one,
        // so a folder of pictures is the moments rather than the frames.
        if mapped != seen {
            println!("{mapped} window(s) from {named}");
            seen = mapped;
            if let (true, Some(saving)) = (mapped > 0, saving.as_mut())
                && let Some(frame) = nested.the_frame_just_drawn()
            {
                saved_at.push(saving.frame(&format!("{named} with {mapped} window"), frame)?);
            }
        }
        std::thread::sleep(Duration::from_millis(16));
    }

    // A last picture of wherever it got to, named so, because the interesting
    // case is often the one where nothing arrived.
    if let Some(saving) = saving.as_mut()
        && let Some(frame) = nested.the_frame_just_drawn()
    {
        saved_at.push(saving.frame(&format!("{named} at the end"), frame)?);
    }

    // Ended rather than left running: this fixture started it, so this fixture
    // is what stops it.
    for mut child in started {
        let _ = child.kill();
        let _ = child.wait();
    }

    println!("{drawn} surfaces drawn, {seen} window(s) at the end");
    for at in &saved_at {
        println!("  {}", at.display());
    }
    if seen == 0 {
        return Err(format!(
            "{named} connected to nothing, or drew nothing, within {} seconds. That is a \
             result rather than a crash, and the pictures show what the compositor had",
            asked.seconds
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
    /// The programs to start, in the order they were named.
    ///
    /// **More than one, because one window cannot show a Dock counting.** The
    /// Dock groups by application, so two windows of one program are one entry
    /// and two programs are two — and a fixture that can only open one can
    /// never tell those apart on a screen.
    programs: Vec<String>,
    /// How long to keep drawing for.
    seconds: u64,
    /// Who draws the window's header.
    ///
    /// **An input, because the contract forbids guessing it** from a
    /// screenshot, an application's name or how its title bar looks. A
    /// fixture cannot read protocol state for a client it has not configured,
    /// so it is named on the command line with `--app-draws-its-own` and the
    /// production host reads it properly.
    decorations: alo_shell::Decorations,
    /// Where the pictures go, if anywhere.
    saving: Option<saving_frames::SaveTo>,
}

/// Read `--run`, `--seconds` and `--save-to` off the arguments.
///
/// `--save-to` is parsed by [`saving_frames::SaveTo`] so that one file owns
/// what that flag means for every fixture that takes it.
#[cfg(target_os = "linux")]
fn what_was_asked() -> Result<Asked, Box<dyn std::error::Error>> {
    let mut programs = Vec::new();
    let mut seconds = 10;
    let mut decorations = alo_shell::Decorations::TheShellDraws;
    let mut rest = Vec::new();
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--run" => programs.push(arguments.next().ok_or("--run needs a program after it")?),
            "--app-draws-its-own" => decorations = alo_shell::Decorations::TheApplicationDraws,
            "--seconds" => {
                seconds = arguments
                    .next()
                    .ok_or("--seconds needs a number after it")?
                    .parse()?;
            }
            other => rest.push(other.to_owned()),
        }
    }
    if programs.is_empty() {
        // One small Wayland terminal, so the fixture has something to run when
        // nobody named anything.
        programs.push("foot".to_owned());
    }
    Ok(Asked {
        programs,
        seconds,
        decorations,
        saving: saving_frames::SaveTo::from_these(rest)?,
    })
}

/// A desk where nothing has been put aside.
///
/// One shared value rather than a temporary at each site, so the borrow does
/// not outlive the panel it names — the same reason `desktop_check` has one.
#[cfg(target_os = "linux")]
fn nothing_put_aside() -> &'static alo_put_aside::Panel {
    static EMPTY: std::sync::OnceLock<alo_put_aside::Panel> = std::sync::OnceLock::new();
    EMPTY.get_or_init(alo_put_aside::Panel::new)
}

/// The extent this fixture opens its nested output at.
///
/// Named once because two things need it and a second literal is how they
/// come to disagree: `Nested::new` is told it, and `WindowControlLayout`
/// validates its origin against it.
#[cfg(target_os = "linux")]
const THE_OUTPUT_IS: (i32, i32) = (1366, 768);
