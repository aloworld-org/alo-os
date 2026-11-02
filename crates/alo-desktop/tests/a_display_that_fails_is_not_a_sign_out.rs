//! **A desktop whose display failed is started again, never signed out.**
//!
//! Measured on a booted machine on 2026-11-02 (qemu, 1280 × 800): the record
//! window could not draw its Put back confirmation, the display loop ended on
//! `Render(RecordScene)`, and the desktop exited **successfully** — so systemd
//! took `OnSuccess=` to the sign-in screen and the person lost their session
//! and every window in it.
//!
//! Three facts have to agree, and each is read here: the library says a
//! failed display ended on a fault, the binary turns that into a failing exit,
//! and the unit restarts a failing exit while a success goes to the sign-in
//! screen. The library's half is a value; the other two are this repository's
//! own files, read as text, because neither can be run here.
#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_shell::{DirectLoopError, DirectLoopResult, RenderError, SessionError};

/// A display that ended with `outcome`, and nothing else to report.
fn ended(outcome: Result<(), DirectLoopError>) -> DirectLoopResult {
    DirectLoopResult {
        outcome,
        retirement: None,
        rest_retirement: Vec::new(),
        input_cleanup: None,
        input_flush: None,
        flush: None,
    }
}

/// A file of this repository, read as text.
fn read(from_the_repository: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(from_the_repository),
    )
    .unwrap()
}

/// **A frame that could not be drawn is a fault; a clean stop and the seat
/// handing the display on are not.**
#[test]
fn a_frame_that_could_not_be_drawn_is_a_fault() {
    assert!(ended(Err(DirectLoopError::Render(RenderError::RecordScene))).ended_on_a_fault());
    assert!(ended(Err(DirectLoopError::Session(SessionError::Failed))).ended_on_a_fault());
    assert!(!ended(Ok(())).ended_on_a_fault());
    assert!(!ended(Err(DirectLoopError::Session(SessionError::Inactive))).ended_on_a_fault());
}

/// **The binary exits as a failure on a fault**, and the unit starts a
/// failure again while a success goes to the sign-in screen.
#[test]
fn a_fault_is_a_failing_exit_and_a_failing_exit_comes_back() {
    let main = read("crates/alo-desktop/src/main.rs");
    let at = main
        .find("if stood.display.ended_on_a_fault() {")
        .expect("the desktop no longer asks whether its display ended on a fault");
    let after = main.get(at..).unwrap();
    let failure = after.find("ExitCode::FAILURE").unwrap();
    let success = after.find("ExitCode::SUCCESS").unwrap();
    assert!(
        failure < success,
        "a display that ended on a fault must be the failing exit"
    );

    let unit = read("image/usr/lib/systemd/system/alo-desktop@.service");
    let said = |line: &str| unit.lines().any(|it| it.trim() == line);
    assert!(
        said("Restart=on-failure"),
        "a failing desktop is not started again"
    );
    assert!(
        said("OnSuccess=alo-compositor.service"),
        "a success no longer goes to the sign-in screen, so this test's premise moved"
    );
}
