//! What the terminal's shell is built from, for each answer the manager can
//! give. Reading the manager itself needs a person's session bus, which a test
//! runner does not have; that half is checked on a booted machine.

use std::path::Path;

use alo_terminal::TheSession;

use super::the_session_for_a_terminal;

#[test]
fn the_managers_environment_is_handed_on_with_the_display_laid_over_it() {
    let (session, said) = the_session_for_a_terminal(
        Ok(vec![
            "XDG_DATA_DIRS=/home/anna/.local/share/flatpak/exports/share:/usr/share".to_owned(),
            "WAYLAND_DISPLAY=wayland-stale".to_owned(),
        ]),
        Path::new("/run/alo-desktop/session/wayland"),
    );
    assert_eq!(
        session,
        TheSession::Whole(vec![
            "XDG_DATA_DIRS=/home/anna/.local/share/flatpak/exports/share:/usr/share".to_owned(),
            "WAYLAND_DISPLAY=wayland-stale".to_owned(),
            "WAYLAND_DISPLAY=/run/alo-desktop/session/wayland".to_owned(),
            "XDG_SESSION_TYPE=wayland".to_owned(),
        ]),
        "the display must come after the manager's own, so it is the one that counts"
    );
    assert!(
        said.is_empty(),
        "nothing went wrong and something was said: {said:?}"
    );
}

/// **The refusal**: a manager that cannot be asked leaves the shell the
/// desktop's environment and the display, and the log is told why.
#[test]
fn a_manager_that_cannot_be_asked_is_said_and_the_display_still_goes() {
    let (session, said) = the_session_for_a_terminal(
        Err("no session bus".to_owned()),
        Path::new("/run/alo-desktop/session/wayland"),
    );
    assert_eq!(
        session,
        TheSession::OnTopOfThisProcess(vec![
            "WAYLAND_DISPLAY=/run/alo-desktop/session/wayland".to_owned(),
            "XDG_SESSION_TYPE=wayland".to_owned(),
        ])
    );
    assert_eq!(said.len(), 1);
    assert!(said.iter().all(|line| line.contains("no session bus")));
}

#[test]
fn a_display_that_cannot_be_a_variable_is_said() {
    let (session, said) =
        the_session_for_a_terminal(Ok(vec!["PATH=/usr/bin".to_owned()]), Path::new("relative"));
    assert_eq!(session, TheSession::Whole(vec!["PATH=/usr/bin".to_owned()]));
    assert_eq!(said.len(), 1, "a display nobody could be told went unsaid");
}
