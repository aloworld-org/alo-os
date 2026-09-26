//! **One process-heavy fixture in this crate at a time.**
//!
//! Two test targets here start small fleets of processes. `the_nested_fixtures`
//! starts a headless `weston` and then seven children, each building a winit
//! event loop, a GLES context and real Wayland clients.
//! `the_tree_a_reader_finds` starts a `dbus-daemon`, at-spi2's own bus launcher
//! and its registry. `cargo test --workspace` runs test binaries in parallel, so
//! the two overlap — and on 2026-09-26 the gate on `main` went red with
//! *dbus-daemon printed no address*: not a wrong answer from the accessibility
//! tree, but a session bus that never came up.
//!
//! # Why this exists even though the failure was never attributed
//!
//! It was seen once, on a tree that had passed the same gate twice, and running
//! the two targets together on their own passes. **One observation is not
//! attribution**, and this file does not claim to have found the cause.
//!
//! It exists because the arithmetic does not need the cause. Weston plus seven
//! compositors beside a bus launcher will contend on any machine slower than the
//! one that saw it, and the fleet has slower machines. The cost of serialising
//! them and being wrong is a slower suite. The cost of leaving it is a `main`
//! that is red often enough that people learn to ignore it, and then a real
//! regression walks through behind the noise.
//!
//! # Why a file and not a mutex
//!
//! These are separate processes — separate test binaries, and in one case
//! separate children of one. A `Mutex` reaches none of them. What crosses a
//! process boundary without `unsafe` and without a dependency is a directory:
//! `create_dir` is atomic on every filesystem this repository runs on, and it
//! fails rather than succeeding twice.
//!
//! A holder that died without cleaning up would otherwise block the suite
//! forever, so the holder writes its process id in and a lock whose owner is
//! gone is taken rather than obeyed — the same shape, and for the same reason,
//! as `gates.sh`'s own lock on the shared build directory. The thing being
//! guarded is a machine's spare capacity, not anything a person loses.

use std::path::PathBuf;
use std::time::{Duration, Instant};

/// How long to wait for another heavy fixture before going ahead anyway.
///
/// Going ahead rather than failing: this serialises to be kind to a loaded
/// machine, and a fixture that refused to run because another was slow would
/// have turned a courtesy into a gate nobody can pass.
const PATIENCE: Duration = Duration::from_secs(300);

/// Held while a process-heavy fixture runs; released when dropped.
pub struct TheOnlyHeavyFixture {
    /// The directory whose existence is the lock.
    at: PathBuf,
}

impl TheOnlyHeavyFixture {
    /// Wait for any other heavy fixture in this crate to finish, then hold.
    ///
    /// Takes a lock left by a process that is gone, because the alternative is a
    /// suite that one killed test run blocks until somebody notices by hand.
    #[must_use]
    pub fn held() -> Self {
        // Beside the build rather than in `/tmp`: two checkouts of this
        // repository on one machine are two suites, and they have no reason to
        // wait for each other. `CARGO_TARGET_DIR` is what distinguishes them.
        let at = PathBuf::from(
            std::env::var_os("CARGO_TARGET_DIR")
                .unwrap_or_else(|| std::ffi::OsString::from(std::env::temp_dir())),
        )
        .join("alo-shell-one-heavy-fixture.lock");
        let waiting = Instant::now();
        loop {
            if std::fs::create_dir(&at).is_ok() {
                drop(std::fs::write(
                    at.join("pid"),
                    std::process::id().to_string(),
                ));
                return Self { at };
            }
            if the_holder_is_gone(&at) {
                eprintln!(
                    "taking a heavy-fixture lock whose holder is gone: {}",
                    at.display()
                );
                drop(std::fs::remove_dir_all(&at));
                continue;
            }
            if waiting.elapsed() > PATIENCE {
                eprintln!(
                    "waited {PATIENCE:?} for another heavy fixture and going ahead anyway; \
                     both are now running and either may be slow"
                );
                return Self { at: PathBuf::new() };
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }
}

impl Drop for TheOnlyHeavyFixture {
    fn drop(&mut self) {
        if self.at.as_os_str().is_empty() {
            // Went ahead without the lock; somebody else owns it.
            return;
        }
        drop(std::fs::remove_dir_all(&self.at));
    }
}

/// Whether the process that took this lock has gone away.
///
/// A lock with no readable pid is treated as gone: it was left by something that
/// could not finish writing one, which is exactly the case this recovers from.
fn the_holder_is_gone(at: &std::path::Path) -> bool {
    let Ok(written) = std::fs::read_to_string(at.join("pid")) else {
        return true;
    };
    let Ok(held) = written.trim().parse::<i32>() else {
        return true;
    };
    // `/proc/<pid>` rather than a signal, because sending one needs `libc` and
    // this workspace forbids `unsafe`. On Linux, which is all this crate builds
    // for, the directory is the answer.
    !PathBuf::from(format!("/proc/{held}")).exists()
}
