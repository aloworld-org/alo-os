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
//! 1. a `weston` on the path, started headless for this run and stopped again
//!    afterwards — **the default, so that the parent is the same parent on every
//!    machine**;
//! 2. only when `ALO_NESTED_INHERIT_DISPLAY` is set, a `WAYLAND_DISPLAY` whose
//!    socket is **really there** — a developer's own session, or WSLg.
//!    `a_display_that_is_really_there` says why the socket and not the variable:
//!    WSL names one for root that only the default user has;
//! 3. failing both, a skip naming what was missing.
//!
//! ## Why inheriting a display is opt-in, having been the default
//!
//! **A display this process did not start belongs to whatever else is running on
//! the machine, so attaching to it measures that compositor rather than this one.**
//! Measured on 2026-09-28 by the laptop lane: the same binary on the same tree
//! passed when run alone and failed inside the full gate, and the cause was in the
//! first line of its own stderr — it had inherited WSLg's compositor, which has no
//! usable GPU for it, and `keyboard-grabs` then died on a bad buffer pool after
//! `ZINK: failed to choose pdev`. The Mac lane's VM has no `WAYLAND_DISPLAY` at
//! all, so it always started its own and always passed. **Same code, two verdicts,
//! decided by what else was on the machine** — which for a fixture whose whole
//! claim is *against a real parent* is the thing that most needs not to be true.
//!
//! It nearly went down as contention: constraining threads moved the time from 72s
//! to 47s and did not move the verdict, which fitted that story and was not it.
//!
//! Reuse was deliberate — these were written against WSLg — so it is kept rather
//! than removed, and made a decision somebody makes instead of one the environment
//! makes for them. A machine with no `weston` and a display it did not start is
//! told the variable's name rather than quietly given the wrong answer.
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
#[path = "../examples/support/offscreen_check.rs"]
mod offscreen_check;
#[path = "../examples/support/offscreen_client.rs"]
mod offscreen_client;
#[path = "../examples/support/offscreen_division_check.rs"]
mod offscreen_division_check;
#[path = "../examples/support/offscreen_stages.rs"]
mod offscreen_stages;
#[path = "support/one_heavy_fixture_at_a_time.rs"]
mod one_heavy_fixture_at_a_time;
#[path = "../examples/support/popup_check.rs"]
mod popup_check;
#[path = "../examples/support/resize_geometry_check.rs"]
mod resize_geometry_check;
#[path = "../examples/support/the_canvas_walk_check.rs"]
mod the_canvas_walk_check;
#[path = "../examples/support/the_canvas_walk_moments.rs"]
mod the_canvas_walk_moments;
#[path = "../examples/support/the_walk_check.rs"]
mod the_walk_check;
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
    // **Only when asked for**, and the header says why: a display this process did
    // not start belongs to whatever else is running on the machine, and a fixture
    // that attaches to it is measuring that compositor rather than this one.
    if std::env::var_os(INHERIT).is_some_and(|it| !it.is_empty())
        && a_display_that_is_really_there().is_some()
    {
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
        // No `weston`. If there is a display here it is somebody else's, and
        // using it is a decision rather than a fallback — so this says how to
        // make that decision instead of quietly making it.
        return Parent::None(match a_display_that_is_really_there() {
            Some(named) => format!(
                "no `weston` on the path to start a parent, and the {named:?} already here \
                 belongs to whatever else is running on this machine: set {INHERIT}=1 to \
                 measure against that compositor, knowing the result is then partly its"
            ),
            None => match std::env::var_os("WAYLAND_DISPLAY") {
                Some(named) if !named.is_empty() => format!(
                    "WAYLAND_DISPLAY is {named:?} and there is no socket of that name to reach, \
                     and no `weston` on the path to start one"
                ),
                _ => "no WAYLAND_DISPLAY in the environment and no `weston` on the path".to_owned(),
            },
        });
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

/// The socket `WAYLAND_DISPLAY` names, if there is really one there.
///
/// **A variable is a claim; a socket is a fact.** This asked only whether
/// `WAYLAND_DISPLAY` was set and non-empty until 2026-09-27, and on the machine
/// that gates this repository that cost every one of the eight sub-modes:
///
/// - WSL puts `WAYLAND_DISPLAY=wayland-0` and `XDG_RUNTIME_DIR=/run/user/0` into
///   every shell, whoever it is;
/// - WSLg's socket is at `/mnt/wslg/runtime-dir/wayland-0`, and WSL links it into
///   `/run/user/1000` for the default user — **never into `/run/user/0`**;
/// - the gate runs as root.
///
/// So the name pointed at nothing, this returned *already there*, weston was
/// never looked for, and all eight sub-modes failed with *nested graphics
/// initialization: No such file or directory*. Not a skip and not a machine
/// fault: a fixture that believed an environment variable. With this, the same
/// machine starts weston and seven of the eight passed on the first run — nine
/// steps of the shell read back off the frames they drew, on llvmpipe, with no
/// GPU.
///
/// Resolved the way a Wayland client resolves it: an absolute `WAYLAND_DISPLAY`
/// is the path, anything else is relative to `XDG_RUNTIME_DIR`, and with no
/// runtime directory there is nowhere for a relative name to be.
fn a_display_that_is_really_there() -> Option<PathBuf> {
    let named = std::env::var_os("WAYLAND_DISPLAY").filter(|it| !it.is_empty())?;
    let named = Path::new(&named);
    let at = if named.is_absolute() {
        named.to_path_buf()
    } else {
        Path::new(&std::env::var_os("XDG_RUNTIME_DIR")?).join(named)
    };
    std::fs::metadata(&at)
        .is_ok_and(|it| it.file_type().is_socket())
        .then_some(at)
}

use std::os::unix::fs::FileTypeExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

/// Which sub-mode a child process was spawned to run.
///
/// The offscreen probe is first because it is the one that reads pixels back
/// and compares them, so it is the one that fails most usefully. The walk is
/// second for the same reason and a sharper one: it is the only check anywhere
/// that measures what the canvas's zoom actually drew, and it found two faults
/// in the canvas's first landing that the whole rest of the workspace passed.
const EVERY_SUBMODE: [&str; 9] = [
    "offscreen",
    "walk",
    "canvas-walk",
    "grabs",
    "pointer-release-grabs",
    "keyboard-grabs",
    "keyboard-release-grabs",
    "client",
    "client-everything",
];

/// The environment variable naming a child's sub-mode.
const SUBMODE: &str = "ALO_NESTED_SUBMODE";

/// Set this to measure against a compositor this process did not start.
///
/// Off by default: see the header. It is the difference between *these fixtures
/// passed* and *these fixtures passed on a machine that happened to be running
/// something usable*, and only one of those is a gate.
const INHERIT: &str = "ALO_NESTED_INHERIT_DISPLAY";

/// What this binary calls itself when something asks it to list its tests.
///
/// One name for all eight sub-modes, because that is honestly what this is: the
/// sub-modes are not separately runnable by a filter — they are children this
/// binary spawns, under one parent compositor and one heavy-fixture lock, and a
/// tool that ran them apart would be running something else.
const THE_ONE_TEST: &str = "every_nested_fixture_against_a_real_parent";

/// Run one sub-mode in this process.
fn the_submode(named: &str) -> Result<(), Box<dyn std::error::Error>> {
    match named {
        "offscreen" => offscreen_check::run(),
        "walk" => the_walk_check::run(),
        "canvas-walk" => the_canvas_walk_check::run(),
        "grabs" => grab_check::run(false, false),
        "pointer-release-grabs" => grab_check::run(false, true),
        "keyboard-grabs" => grab_check::run(true, false),
        "keyboard-release-grabs" => grab_check::run(true, true),
        "client" => nested_client_check::run(nested_client_check::Flags::default()),
        "client-everything" => nested_client_check::run(nested_client_check::Flags::everything()),
        other => Err(format!("no such sub-mode: {other}").into()),
    }
}

/// The answer to `--list`, which is two different questions.
///
/// **A runner lists twice**: once plainly, for every test there is, and once with
/// `--ignored`, to learn which of them are ignored. A test that appears in the
/// second listing *is* an ignored test as far as the runner is concerned, so a
/// binary that answers both the same way marks its own test skipped and is never
/// run again. Measured on nextest 0.9.146 by the laptop lane against the first
/// version of this function, which did exactly that: listed, and `(skipped)`.
///
/// This test is not ignored. It belongs in the first listing and not the second.
///
/// **And a format this cannot speak is refused rather than answered wrongly.**
/// `--format json` asked for JSON and would have been handed the terse format,
/// which nextest happens not to ask for; the next tool to meet it would have got
/// a lie instead of an error. Saying so costs one branch.
fn the_listing(arguments: Vec<String>) -> std::process::ExitCode {
    // `--format terse` and `--format=terse` are the same request written two ways.
    let asked_for = |name: &str| -> Option<String> {
        if let Some(value) = arguments
            .iter()
            .find_map(|argument| argument.strip_prefix(&format!("{name}=")))
        {
            return Some(value.to_owned());
        }
        let at = arguments.iter().position(|argument| argument == name)?;
        arguments.get(at.checked_add(1)?).cloned()
    };
    if arguments.iter().any(|argument| argument == "--ignored") {
        return std::process::ExitCode::SUCCESS;
    }
    if let Some(format) = asked_for("--format")
        && format != "terse"
    {
        eprintln!(
            "these fixtures list in the terse format only, and were asked for {format}; \
             answering in terse would say {format} and mean terse"
        );
        return std::process::ExitCode::FAILURE;
    }
    println!("{THE_ONE_TEST}: test");
    std::process::ExitCode::SUCCESS
}

/// **Every sub-mode of the nested fixtures, against a real parent.**
///
/// Each one drives real clients through the parent's own GLES.
fn main() -> std::process::ExitCode {
    // **`--list` lists; it does not run.** This binary is `harness = false`, so
    // nothing gives it the listing protocol that every tool parsing test output
    // expects: one `name: test` line per test on stdout, and nothing else.
    //
    // Without this it ignored the flag and ran, which is worse than the parse
    // error it was reported as. `cargo nextest list` over the workspace failed on
    // this binary — *did not end with ": test"* — but the listing had by then
    // started a headless compositor and seven children each holding a GLES
    // context, to answer a question about names. Answering it costs nothing now.
    if std::env::args().any(|argument| argument == "--list") {
        return the_listing(std::env::args().skip(1).collect());
    }
    if !cfg!(target_os = "linux") {
        // Diagnostics go to stderr, all of them, because stdout is where the
        // line above lives and a tool reading it cannot tell prose from a name.
        eprintln!("skipped: the nested fixtures are Linux's, and this is not Linux");
        return std::process::ExitCode::SUCCESS;
    }
    // A child spawned by the run below, told which one check it is. A child does
    // not take the lock: the parent below holds it for all seven of them, and a
    // child waiting for a lock its own parent holds would wait out the patience
    // and then run anyway, which is the deadlock with extra steps.
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

    // Held for the whole run: a headless compositor and seven children each
    // with a GLES context is the heaviest thing this crate does, and
    // `one_heavy_fixture_at_a_time` says what it is being kept away from.
    let _heavy = one_heavy_fixture_at_a_time::TheOnlyHeavyFixture::held();

    let parent = a_parent();
    let runtime = match &parent {
        Parent::None(why) => {
            eprintln!(
                "skipped: the nested fixtures need a Wayland parent and there is none — {why}"
            );
            return std::process::ExitCode::SUCCESS;
        }
        // Nothing to hand a child: it inherits the parent already here.
        Parent::AlreadyThere => {
            eprintln!(
                "{INHERIT} is set, so this ran against the compositor already in this \
                 environment rather than a headless parent of its own — what passed or \
                 failed here is partly that compositor's"
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
            Ok(status) if status.success() => eprintln!("{named}: passed"),
            Ok(_) => refused.push(named),
            Err(why) => {
                eprintln!("{named} could not be started as a child: {why}");
                refused.push(named);
            }
        }
    }
    if refused.is_empty() {
        eprintln!(
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
