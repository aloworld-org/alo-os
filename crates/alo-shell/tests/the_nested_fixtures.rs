//! **Every nested fixture, in the gate.**
//!
//! These checks drive real clients through a real GLES context in a real
//! compositor, and until 2026-09-26 no gate ran one of them. That is not a
//! detail: `nested_check --offscreen` asserts how many stages it walked, task 16
//! of the shell plan removed six of those stages and left the number at thirty,
//! and the probe was broken on `main` for a day while all nine gates passed
//! twice over it. **A test no gate runs is not a test; it is a file that
//! compiles.**
//!
//! What made them unrunnable was a belief rather than a limit: that a nested
//! compositor needs a desktop session, so a build machine cannot have one.
//! `weston --backend=headless` is a Wayland parent with no display and no GPU,
//! and Mesa's llvmpipe gives it GLES. `docs/autonomy/COMPOSITOR.md` carries the
//! invocation.
//!
//! # A machine with no parent says so and does not fail
//!
//! ADR 0063 — *a machine that cannot run the engine says so, rather than
//! failing* — is the rule, and
//! `crates/alo-in-use/tests/a_stream_through_the_media_server_is_listed.rs` is
//! the shape: **a skip nobody can see is the same colour as a pass**, so every
//! skip here prints what it skipped and why. A gate only one machine can pass is
//! how these fixtures ended up unrun in the first place, and replacing *nobody
//! runs it* with *one machine runs it* would be the same fault wearing a test's
//! name.
//!
//! The order this looks for a parent, and it never installs anything:
//!
//! 1. a `WAYLAND_DISPLAY` already in the environment — a developer's own
//!    session, or WSLg, which is what the fixtures were written against;
//! 2. failing that, a `weston` on the path, started headless for this run and
//!    stopped again afterwards;
//! 3. failing that, a skip naming which of the two was missing.
//!
//! # `harness = false`, because winit will not start off the main thread
//!
//! This is the reason these checks were examples nobody ran rather than tests,
//! and it is a hard refusal rather than a preference: winit panics with
//! *initializing the event loop outside of the main thread is a significant
//! cross-platform compatibility hazard*, and libtest runs every `#[test]` on a
//! worker thread. So a harnessed test can never construct an
//! `alo_shell::Nested`, however it is written.
//!
//! `harness = false` in `Cargo.toml` gives this target its own `fn main()`,
//! which is the main thread. A zero exit is a pass, as it is for any program.
//!
//! # One process per sub-mode, and why it re-runs itself
//!
//! `Nested` documents that winit's event loop must not be recreated in the same
//! process after a failure, so each sub-mode wants its own process. And a parent
//! this program starts has to reach `Nested` through the environment, which
//! `std::env::set_var` is the obvious way to do and which this workspace
//! forbids: `unsafe_code = "forbid"` covers tests too, and that is the right
//! answer — a process mutating its own environment while threads read it is
//! exactly what the rule is for.
//!
//! Both wants have one answer. It spawns **itself** once per sub-mode, with the
//! parent's variables set on the child rather than on anybody, and
//! `ALO_NESTED_SUBMODE` naming which sub-mode that child is. Where a parent was
//! already in the environment there is nothing to set and the children inherit
//! it.
//!
//! # What this does not prove
//!
//! Nothing here is a physical display. A headless parent composites to nothing:
//! there is no panel, no scanout and no photograph, so every *physical display
//! unverified* these fixtures print is still true and no owed screenshot of a
//! real machine is discharged by a green run.

#[path = "support/application.rs"]
mod application;
#[path = "../examples/support/default_cursor_check.rs"]
mod default_cursor_check;
#[path = "../examples/support/grab_check.rs"]
mod grab_check;
#[path = "../examples/support/interactive_resize_check.rs"]
mod interactive_resize_check;
#[path = "../examples/support/nested_client_check.rs"]
mod nested_client_check;
#[path = "../examples/support/nested_control_frame_check.rs"]
mod nested_control_frame_check;
#[path = "../examples/support/nested_reader_frame_check.rs"]
mod nested_reader_frame_check;
#[path = "../examples/support/offscreen_check.rs"]
mod offscreen_check;
#[path = "../examples/support/offscreen_client.rs"]
mod offscreen_client;
#[path = "../examples/support/popup_check.rs"]
mod popup_check;
#[path = "../examples/support/resize_geometry_check.rs"]
mod resize_geometry_check;
#[path = "../examples/support/window_control_label_check.rs"]
mod window_control_label_check;
#[path = "../examples/support/window_control_scene_check.rs"]
mod window_control_scene_check;
#[path = "../examples/support/window_control_snapshot_check.rs"]
mod window_control_snapshot_check;
#[path = "../examples/support/window_controls_pixels.rs"]
mod window_controls_pixels;
#[path = "../examples/support/window_maximize_check.rs"]
mod window_maximize_check;
#[path = "../examples/support/window_minimize_check.rs"]
mod window_minimize_check;
#[path = "../examples/support/window_placement_check.rs"]
mod window_placement_check;
#[path = "../examples/support/window_raise_check.rs"]
mod window_raise_check;
#[path = "../examples/support/window_size_check.rs"]
mod window_size_check;
#[path = "../examples/support/window_switch_check.rs"]
mod window_switch_check;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Socket location shared with the fixtures, which reach it as `crate::Fixture`.
pub struct Fixture {
    /// Private test display, distinct from the parent socket.
    pub path: PathBuf,
}

/// How long a freshly started parent is given to put its socket down.
const PATIENCE: Duration = Duration::from_secs(10);

/// A parent this run may use, and whether this test started it.
enum Parent {
    /// One that was already in the environment; left exactly as it was.
    AlreadyThere,
    /// One started for this run, to be stopped when it is dropped.
    Started {
        /// The weston process.
        child: Child,
        /// Its private runtime directory, removed with it.
        runtime: tempfile::TempDir,
    },
    /// None, and which of the two reasons.
    None(String),
}

impl Drop for Parent {
    fn drop(&mut self) {
        if let Self::Started { child, .. } = self {
            // A parent this test started is this test's to stop, on the way out
            // of a pass and of a panic alike: one left running would be found by
            // the next run as *already there* and quietly reused.
            drop(child.kill());
            drop(child.wait());
        }
    }
}

/// Find a Wayland parent, or start one, or say why there is none.
fn a_parent() -> Parent {
    if std::env::var_os("WAYLAND_DISPLAY").is_some_and(|it| !it.is_empty()) {
        return Parent::AlreadyThere;
    }
    let Ok(runtime) = tempfile::tempdir() else {
        return Parent::None("no temporary directory for a parent's socket".to_owned());
    };
    // Only the owner may reach a compositor's socket directory, exactly as the
    // shell's own `Server::bind` requires of the runtime directory it is given.
    if std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).is_err() {
        return Parent::None("a parent's socket directory could not be made private".to_owned());
    }
    let started = Command::new("weston")
        .args([
            "--backend=headless",
            "--width=1366",
            "--height=768",
            "--socket=alo-gate-parent",
            "--idle-time=0",
        ])
        .env("XDG_RUNTIME_DIR", runtime.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let Ok(child) = started else {
        return Parent::None(
            "no WAYLAND_DISPLAY in the environment and no `weston` on the path".to_owned(),
        );
    };
    let socket = runtime.path().join("alo-gate-parent");
    let waiting = Instant::now();
    while !socket.exists() {
        if waiting.elapsed() > PATIENCE {
            return Parent::None(format!(
                "a `weston` started and put no socket down within {PATIENCE:?}"
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Parent::Started { child, runtime }
}

use std::os::unix::fs::PermissionsExt as _;

/// Which sub-mode a child process was spawned to run.
///
/// The offscreen probe is first because it is the one that reads pixels back
/// and compares them, so it is the one that fails most usefully.
const EVERY_SUBMODE: [&str; 7] = [
    "offscreen",
    "grabs",
    "pointer-release-grabs",
    "keyboard-grabs",
    "keyboard-release-grabs",
    "client",
    "client-everything",
];

/// The environment variable naming a child's sub-mode.
const SUBMODE: &str = "ALO_NESTED_SUBMODE";

/// Run one sub-mode in this process.
fn the_submode(named: &str) -> Result<(), Box<dyn std::error::Error>> {
    match named {
        "offscreen" => offscreen_check::run(),
        "grabs" => grab_check::run(false, false),
        "pointer-release-grabs" => grab_check::run(false, true),
        "keyboard-grabs" => grab_check::run(true, false),
        "keyboard-release-grabs" => grab_check::run(true, true),
        "client" => nested_client_check::run(nested_client_check::Flags::default()),
        "client-everything" => nested_client_check::run(nested_client_check::Flags::everything()),
        other => Err(format!("no such sub-mode: {other}").into()),
    }
}

/// **Every sub-mode of the nested fixtures, against a real parent.**
///
/// Each one drives real clients through the parent's own GLES.
fn main() -> std::process::ExitCode {
    if !cfg!(target_os = "linux") {
        println!("skipped: the nested fixtures are Linux's, and this is not Linux");
        return std::process::ExitCode::SUCCESS;
    }
    // A child spawned by the run below, told which one check it is.
    if let Some(named) = std::env::var_os(SUBMODE) {
        let named = named.to_string_lossy().into_owned();
        return match the_submode(&named) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("{named}: {why}");
                std::process::ExitCode::FAILURE
            }
        };
    }

    let parent = a_parent();
    let runtime = match &parent {
        Parent::None(why) => {
            println!(
                "skipped: the nested fixtures need a Wayland parent and there is none — {why}"
            );
            return std::process::ExitCode::SUCCESS;
        }
        // Nothing to hand a child: it inherits the parent already here.
        Parent::AlreadyThere => {
            println!(
                "using the WAYLAND_DISPLAY already in this environment rather than starting one"
            );
            None
        }
        Parent::Started { runtime, .. } => Some(runtime.path().to_owned()),
    };

    let mut refused: Vec<&str> = Vec::new();
    for named in EVERY_SUBMODE {
        let Ok(me) = std::env::current_exe() else {
            eprintln!("this program cannot find itself, so it cannot run its own sub-modes");
            return std::process::ExitCode::FAILURE;
        };
        let mut child = Command::new(me);
        child.env(SUBMODE, named);
        if let Some(runtime) = &runtime {
            child
                .env("XDG_RUNTIME_DIR", runtime)
                .env("WAYLAND_DISPLAY", "alo-gate-parent")
                // llvmpipe rather than whatever device happens to be present: a
                // gate that picked one would pass or fail for reasons this
                // repository does not pin.
                .env("LIBGL_ALWAYS_SOFTWARE", "1");
        }
        match child.status() {
            Ok(status) if status.success() => println!("{named}: passed"),
            Ok(_) => refused.push(named),
            Err(why) => {
                eprintln!("{named} could not be started as a child: {why}");
                refused.push(named);
            }
        }
    }
    if refused.is_empty() {
        println!(
            "every nested fixture passed against a real parent; \
             physical display unverified, because a headless parent composites to nothing"
        );
        return std::process::ExitCode::SUCCESS;
    }
    eprintln!(
        "nested fixtures refused: {refused:?} — each ran as its own process above, \
         and its own output says why"
    );
    std::process::ExitCode::FAILURE
}
