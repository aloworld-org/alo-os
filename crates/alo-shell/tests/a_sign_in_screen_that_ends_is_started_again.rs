//! **A sign-in screen that ends with nobody signed in is started again**, so a
//! machine is never left at the console with no way forward.
//!
//! The seat handing the display to another session is the one other end that
//! must not restart it: that is the person's desktop arriving.
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

/// **The binary exits failing when nobody signed in**, unless a person's
/// session has the screen — and both failing ends ask.
///
/// Until 2026-10-31 the arm was `ExitCode::FAILURE` alone. A booted machine
/// showed the person's desktop taking the card first, this screen's next frame
/// refused, the failure restarted it, and as root it took the display back
/// from the desktop. Both failing ends now go through one function, which
/// fails unless `logind` says a person's session is in front.
#[test]
fn the_binary_fails_when_nobody_signed_in_unless_a_person_has_the_screen() {
    let source = read("src/bin/alo-compositor.rs");
    // Both failing ends of the screen — nobody signed in, and the screen
    // failing — ask, and the function is defined once.
    assert_eq!(
        source
            .matches("failing_unless_a_person_has_the_screen()")
            .count(),
        3,
        "the two failing ends no longer both ask whose session has the screen"
    );
    let arm = source
        .split("Ok(Stood::NobodyDid) => {")
        .nth(1)
        .and_then(|after| after.split("Err(why)").next())
        .unwrap();
    assert!(
        arm.contains("failing_unless_a_person_has_the_screen()"),
        "{arm}"
    );
    assert!(!arm.contains("ExitCode::SUCCESS"), "{arm}");

    let deciding = source
        .split("fn failing_unless_a_person_has_the_screen() -> ExitCode {")
        .nth(1)
        .and_then(|after| after.split("\n    }\n").next())
        .unwrap();
    assert!(
        deciding.contains("a_person_has_the_screen(&Seat0)"),
        "{deciding}"
    );
    let otherwise = deciding.split("} else {").nth(1).unwrap_or_default();
    assert!(
        otherwise.contains("ExitCode::FAILURE") && !otherwise.contains("ExitCode::SUCCESS"),
        "the end with no person's session in front no longer fails, so a stranded screen is \
         not started again: {deciding}"
    );
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

/// **The seat handing the display on is the end this process is for**, so a
/// sign-in screen paused for the person's desktop is not started again to take
/// the display back.
#[test]
fn the_seat_handing_the_display_on_is_not_restarted() {
    assert!(alo_shell::Stood::TheSeatHandedItOn.is_the_end_this_process_is_for());
    let source = read("src/bin/alo-compositor.rs");
    let arm = source
        .split("Ok(Stood::TheSeatHandedItOn) => {")
        .nth(1)
        .and_then(|after| after.split("Ok(Stood::NobodyDid)").next())
        .unwrap();
    assert!(arm.contains("ExitCode::SUCCESS"), "{arm}");
    assert!(!arm.contains("ExitCode::FAILURE"), "{arm}");
}

/// **A paused seat is a handover, however it arrives**: directly, or through
/// the input context, which carries the seat's error inside an `io::Error`.
/// This is the shape a booted machine logged on 2026-10-30.
#[test]
fn a_paused_seat_is_read_as_a_handover() {
    use alo_shell::{DirectLoopError, InputDispatchError, SessionError};

    assert!(DirectLoopError::Session(SessionError::Inactive).is_the_seat_handing_the_display_on());
    let through_input = DirectLoopError::Input(InputDispatchError {
        source: std::io::Error::other(SessionError::Inactive),
        cleanup: None,
    });
    assert_eq!(
        through_input.to_string(),
        "input context: display session is inactive; cleanup: None"
    );
    assert!(through_input.is_the_seat_handing_the_display_on());
}

/// **And a display that failed is not**, so the screen still comes back after
/// a frame it could not draw — the end the restart exists for.
#[test]
fn a_failed_display_is_not_a_handover() {
    use alo_shell::{DirectLoopError, InputDispatchError, RenderError, SessionError};

    for failed in [
        DirectLoopError::Render(RenderError::LockScene),
        DirectLoopError::Session(SessionError::Failed),
        DirectLoopError::Input(InputDispatchError {
            source: std::io::Error::other(SessionError::Failed),
            cleanup: None,
        }),
        DirectLoopError::Input(InputDispatchError {
            source: std::io::Error::other("assign input seat"),
            cleanup: None,
        }),
        DirectLoopError::Dispatch(std::io::Error::other("a client went away")),
    ] {
        assert!(!failed.is_the_seat_handing_the_display_on(), "{failed}");
    }
}

/// A seat that answers with these sessions, or cannot be asked.
struct Answering(Result<Vec<alo_shell::whose_screen_it_is::ASessionOnTheSeat>, String>);

impl alo_shell::whose_screen_it_is::TheSeat for Answering {
    fn sessions(&self) -> Result<Vec<alo_shell::whose_screen_it_is::ASessionOnTheSeat>, String> {
        self.0.clone()
    }
}

/// One session on the seat.
fn on_the_seat(class: &str, state: &str) -> alo_shell::whose_screen_it_is::ASessionOnTheSeat {
    alo_shell::whose_screen_it_is::ASessionOnTheSeat {
        active: true,
        class: class.to_owned(),
        state: state.to_owned(),
    }
}

/// **A refused frame while a person's session is in front is the hand-over.**
///
/// The booted machine of 2026-10-31: the screen ended with *atomic TEST_ONLY
/// failed: Permission denied*, which is not a paused seat, so it was read as
/// nobody signing in — and the answer that decides it is `logind`'s.
#[test]
fn a_refused_frame_while_a_persons_session_is_in_front_is_the_hand_over() {
    use alo_shell::whose_screen_it_is::a_person_has_the_screen;

    // The error shape alone still does not say so, which is why the seat is
    // asked: a refused frame is a failed display to `is_the_seat_handing_…`.
    let refused = alo_shell::DirectLoopError::Dispatch(std::io::Error::from(
        std::io::ErrorKind::PermissionDenied,
    ));
    assert!(!refused.is_the_seat_handing_the_display_on());

    assert!(a_person_has_the_screen(&Answering(Ok(vec![
        on_the_seat("greeter", "online"),
        on_the_seat("user", "active"),
    ]))));
}

/// **The same end with no person's session in front starts the screen again**
/// (#1026's purpose).
#[test]
fn the_same_end_with_no_persons_session_starts_the_screen_again() {
    use alo_shell::whose_screen_it_is::a_person_has_the_screen;

    assert!(!a_person_has_the_screen(&Answering(Ok(vec![on_the_seat(
        "greeter", "active"
    )]))));
    assert!(!a_person_has_the_screen(&Answering(Ok(vec![]))));
}

/// **A logind that cannot be asked starts the screen again**, which is what
/// happened before the seat was asked at all.
#[test]
fn a_logind_that_cannot_be_asked_starts_the_screen_again() {
    use alo_shell::whose_screen_it_is::a_person_has_the_screen;

    assert!(!a_person_has_the_screen(&Answering(Err(
        "org.freedesktop.DBus.Error.ServiceUnknown".to_owned()
    ))));
}
