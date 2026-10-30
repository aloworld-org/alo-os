//! **A sign-in screen that ends with nobody signed in is started again**, so a
//! machine is never left at the console with no way forward.
//!
//! Three facts have to agree for that to be true, and each is read here: the
//! library says that end is not the one the process is for, the binary turns
//! it into a failing exit, and the unit restarts a failing exit. On
//! 2026-10-30 the second was `ExitCode::SUCCESS`, the third did what a success
//! asks, and a booted machine was stranded.
#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

/// A file of this repository, read as text.
fn read(from_the_crate: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(from_the_crate)).unwrap()
}

/// **Nobody signing in is not the end this process is for.**
#[test]
fn nobody_signing_in_is_not_the_end_the_process_is_for() {
    assert!(!alo_shell::Stood::NobodyDid.is_the_end_this_process_is_for());
}

/// **The binary exits failing when nobody signed in**, and only success when
/// somebody did.
#[test]
fn the_binary_fails_when_nobody_signed_in() {
    let source = read("src/bin/alo-compositor.rs");
    let arm = source
        .split("Ok(Stood::NobodyDid) => {")
        .nth(1)
        .and_then(|after| after.split("Err(why)").next())
        .unwrap();
    assert!(arm.contains("ExitCode::FAILURE"), "{arm}");
    assert!(!arm.contains("ExitCode::SUCCESS"), "{arm}");
}

/// **The unit restarts a failing exit**, and does not restart a success,
/// which is what a sign-in opening a session is.
#[test]
fn the_unit_starts_a_failed_screen_again() {
    let unit = read("../../image/usr/lib/systemd/system/alo-compositor.service");
    let said: Vec<&str> = unit
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("Restart="))
        .collect();
    assert_eq!(said, ["Restart=on-failure"]);
}
