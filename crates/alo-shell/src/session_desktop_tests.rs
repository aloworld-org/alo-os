//! What can be checked about a session's desktop on a machine with no display.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use std::os::unix::fs::PermissionsExt;

/// A desktop nothing asks for, since the failure under test happens first.
struct NoDesktop;

impl TheDesktop for NoDesktop {
    fn now(&self) -> crate::DesktopFrame<'_> {
        unreachable!("the display is refused before a frame is ever asked for")
    }
}

/// **A session with no display to stand on says so about the display.**
///
/// The socket binds and the keymap builds before the display is asked for, so
/// what a gate machine reaches is the failure further down the order — which is
/// the check that the order is what this file claims. It is a refusal rather
/// than a panic: a desktop that aborted would leave a person's session being
/// restarted forever by their own manager.
#[test]
fn a_session_with_no_seat_refuses_by_naming_the_seat() {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();

    let refused = stand_the_desktop_up(
        ADisplayToStandOn {
            display: Path::new("/dev/dri/card0"),
            runtime: runtime.path(),
            socket: "a-desktop-with-no-seat",
            layout: "gb",
        },
        &mut NoDesktop,
        || DirectFrame::Stop,
    )
    .err()
    .unwrap();

    assert!(
        matches!(refused, WouldNotStand::Seat(_)),
        "a session with no seat refused for another reason: {refused}"
    );
    assert!(
        !runtime.path().join("a-desktop-with-no-seat").exists(),
        "a desktop that could not start left its socket directory behind"
    );
}
