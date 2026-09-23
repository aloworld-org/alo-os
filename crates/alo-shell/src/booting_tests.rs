//! What can be checked about a machine's morning on a machine that has no
//! display, no login seat and nobody to sign in.
//!
//! Which is less than the task owes and more than nothing: the sentences the
//! screen would need really load from the bundle this crate ships with, and the
//! order the pieces are put in is the order that matters — a machine missing
//! its seat says so about the seat, rather than about the first thing that
//! happened to be checked.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use std::os::unix::fs::PermissionsExt;

/// **A sign-in screen can word what it has to say.**
///
/// The bundle is loaded by the shipped runner rather than by a test fixture, so
/// a machine that shipped without its sentences fails here rather than in front
/// of somebody who cannot tell a wrong password from a broken service.
#[test]
fn this_machine_can_say_what_a_refused_password_is() {
    let words = what_this_machine_can_say().unwrap();
    let said = alo_greeting::Standing::MakeAnAccount.said(&words);

    assert!(
        said.is_some_and(|said| !said.text().is_empty()),
        "this machine cannot say the one sentence a screen with no accounts has to show"
    );
}

/// **A machine with no login seat says so about the seat.**
///
/// The socket binds and the keymap builds before the display is asked for, so
/// the failure a gate machine gets here is the one further down the order —
/// which is the check that the order is what this file claims it is. The
/// refusal names the seat, and it is a refusal rather than a panic: a
/// compositor that aborted would leave a supervisor restarting it forever.
#[test]
fn a_machine_with_no_seat_refuses_by_naming_the_seat() {
    let runtime = tempfile::tempdir().unwrap();
    // A runtime directory is the person's own and nobody else's, which the
    // socket checks for before it binds; a temporary directory is not that
    // until it is made so.
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let refused = stand_the_sign_in_screen_up(
        AMachineToStandOn {
            display: Path::new("/dev/dri/card0"),
            accounts: &runtime.path().join("accounts"),
            person: 1000,
            door: alo_greeting::TheOpenersDoor::at(&runtime.path().join("sign-in.sock")),
            runtime: runtime.path(),
            socket: "a-machine-with-no-seat",
            layout: "gb",
        },
        what_this_machine_can_say().unwrap(),
        || DirectFrame::Stop,
    )
    .err()
    .unwrap();

    assert!(
        matches!(refused, WouldNotStand::Seat(_)),
        "a machine with no seat refused for another reason: {refused}"
    );
    assert!(
        !runtime.path().join("a-machine-with-no-seat").exists(),
        "a compositor that could not start left its socket directory behind"
    );
}
