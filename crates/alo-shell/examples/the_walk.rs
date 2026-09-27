//! **Every new surface, walked** — one person's way through this shell, run as a
//! fixture somebody can start by hand.
//!
//! The walk itself is `support/the_walk_check.rs`, and it is there rather than
//! here for one reason: **the gate runs it.**
//! `crates/alo-shell/tests/the_nested_fixtures.rs` has the same module, so the
//! zoom this walk measures in pixels is measured on every run of the ordinary
//! suite rather than on the days somebody remembers to type this command.
//!
//! That move was made the day the walk earned it. It caught two faults in the
//! canvas's first landing that every other test in the workspace passed — a
//! camera the backend was never given, and then a zoom applied to each frame's
//! size but not to where the frame was — and a check that finds faults like
//! those while running nowhere is the definition of the rot
//! `the_nested_fixtures.rs`'s own header is about.
//!
//! This file remains because a fixture a person can run on its own, and watch,
//! is worth keeping: it prints its nine steps and what each one painted.

#[cfg(target_os = "linux")]
#[path = "../tests/support/application.rs"]
#[expect(
    dead_code,
    reason = "this shared client fixture carries more than one caller uses"
)]
mod application;

#[cfg(target_os = "linux")]
#[path = "support/the_walk_check.rs"]
mod the_walk_check;

/// Socket location the walk's own clients connect to, reached by
/// `tests/support/application.rs` as `crate::Fixture`.
#[cfg(target_os = "linux")]
pub struct Fixture {
    /// The walk's private display, distinct from the parent's socket.
    pub path: std::path::PathBuf,
}

/// Main-thread graphics initialisation is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("the walk failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// The walk, which lives beside the other nested fixtures.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    the_walk_check::run()
}
