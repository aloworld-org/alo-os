//! What a shell starts with, decided without starting one. The real shell is
//! held to it in `a_shell_on_a_pseudoterminal_tests.rs`, by the kernel.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{ENV, TheSession, WhyNotKeptOut, what_a_shell_starts_with};

/// A person's session as their manager might hold it.
fn a_session() -> Vec<String> {
    [
        "SHELL=/bin/bash",
        "HOME=/home/anna",
        "USER=anna",
        "PATH=/usr/bin",
        "XDG_DATA_DIRS=/home/anna/.local/share/flatpak/exports/share:/usr/share",
        "ACCESSIBILITY_ENABLED=1",
        "WAYLAND_DISPLAY=/run/alo-desktop/session/wayland",
        "TERM=dumb",
        "not a variable",
        "=no-name",
    ]
    .map(str::to_owned)
    .to_vec()
}

/// The names the desktop process has.
fn the_desktops_names() -> Vec<String> {
    [
        "PATH",
        "HOME",
        "ALO_AGENT",
        "ALO_KEYBOARD",
        "NOTIFY_SOCKET",
        "TERM",
    ]
    .map(str::to_owned)
    .to_vec()
}

#[test]
fn the_whole_session_is_handed_on_and_the_desktops_own_names_are_removed() {
    let starts = what_a_shell_starts_with(
        &TheSession::Whole(a_session()),
        "xterm-256color",
        &the_desktops_names(),
        Some("/bin/sh"),
        true,
    );

    assert_eq!(
        starts.env.get("XDG_DATA_DIRS").map(String::as_str),
        Some("/home/anna/.local/share/flatpak/exports/share:/usr/share"),
        "Flatpak's folders from the session did not reach the shell"
    );
    assert_eq!(
        starts.env.get("ACCESSIBILITY_ENABLED").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        starts.env.get("TERM").map(String::as_str),
        Some("xterm-256color"),
        "the session changed what the terminal claims to understand"
    );
    assert_eq!(
        starts.env.len(),
        8,
        "something that is not a variable became one: {:?}",
        starts.env
    );
    let (program, arguments) = starts.program.expect("the desktop's names are kept out");
    assert_eq!(program, ENV);
    assert_eq!(
        arguments,
        [
            "-u",
            "ALACRITTY_WINDOW_ID",
            "-u",
            "ALO_AGENT",
            "-u",
            "ALO_KEYBOARD",
            "-u",
            "NOTIFY_SOCKET",
            "-u",
            "WINDOWID",
            "--",
            "/bin/bash",
        ],
        "a name the session holds was removed, or one it lacks was kept, or the session's own \
         shell was not the one started"
    );
    assert_eq!(starts.not_kept_out, None);
}

/// **No value is ever on the command line**, which any account can read.
#[test]
fn only_names_go_on_the_command_line() {
    let starts = what_a_shell_starts_with(
        &TheSession::Whole(vec![
            "SHELL=/bin/bash".to_owned(),
            "A_TOKEN=secret-value".to_owned(),
        ]),
        "xterm-256color",
        &["A_DESKTOP_TOKEN".to_owned()],
        None,
        true,
    );
    let (_, arguments) = starts.program.expect("the desktop's names are kept out");
    assert!(
        arguments
            .iter()
            .all(|argument| !argument.contains("secret-value") && !argument.contains('=')),
        "a value reached the command line: {arguments:?}"
    );
}

#[test]
fn the_desktops_shell_is_used_when_the_session_names_none() {
    let starts = what_a_shell_starts_with(
        &TheSession::Whole(vec!["PATH=/usr/bin".to_owned()]),
        "xterm-256color",
        &[],
        Some("/bin/zsh"),
        true,
    );
    let (_, arguments) = starts.program.expect("a shell was named by this process");
    assert_eq!(arguments.last().map(String::as_str), Some("/bin/zsh"));
}

/// **The refusals**: with no shell to name, or no `env`, the shell still
/// starts — inheriting this process — and the caller is told why.
#[test]
fn with_no_shell_named_the_desktops_names_cannot_be_kept_out_and_it_says_so() {
    let starts = what_a_shell_starts_with(
        &TheSession::Whole(vec!["PATH=/usr/bin".to_owned()]),
        "xterm-256color",
        &the_desktops_names(),
        None,
        true,
    );
    assert_eq!(starts.program, None);
    assert_eq!(starts.not_kept_out, Some(WhyNotKeptOut::NoShellNamed));
    assert_eq!(
        starts.env.get("PATH").map(String::as_str),
        Some("/usr/bin"),
        "the session is still handed on top"
    );
}

#[test]
fn with_no_env_on_the_machine_it_says_so() {
    let starts = what_a_shell_starts_with(
        &TheSession::Whole(a_session()),
        "xterm-256color",
        &the_desktops_names(),
        Some("/bin/sh"),
        false,
    );
    assert_eq!(starts.program, None);
    assert_eq!(starts.not_kept_out, Some(WhyNotKeptOut::NoEnv));
}

/// **A session that could not be asked** is handed on top of this process, as
/// before, and that is not a fault this file reports: the caller already knows
/// it could not ask.
#[test]
fn a_session_that_could_not_be_asked_is_handed_on_top() {
    let starts = what_a_shell_starts_with(
        &TheSession::OnTopOfThisProcess(vec!["WAYLAND_DISPLAY=/run/x/wayland".to_owned()]),
        "xterm-256color",
        &the_desktops_names(),
        Some("/bin/sh"),
        true,
    );
    assert_eq!(starts.program, None);
    assert_eq!(starts.not_kept_out, None);
    assert_eq!(
        starts.env.get("WAYLAND_DISPLAY").map(String::as_str),
        Some("/run/x/wayland")
    );
    assert_eq!(starts.env.len(), 2);
}
