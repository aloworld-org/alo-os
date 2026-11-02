//! **Close ends an idle terminal at once, and asks first while a command is
//! running** — with a real shell on a real pseudoterminal, because *running*
//! is the kernel's answer and a mocked one would test the mock.
//!
//! The refusal path is held as carefully as the other: Cancel, Escape and a
//! stray key each leave the terminal open and the command running.

use std::time::Duration;

use alo_terminal::IsACommandRunning;
use alo_terminal::what_a_person_typed::{ANamedKey, Held, WhatWasTyped};

use super::{OnTheQuestion, WhatClosingDid};
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;

/// A server with alo's terminal open on it.
fn with_a_terminal(named: &str) -> (tempfile::TempDir, crate::Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = crate::Server::bind(directory.path(), named).expect("a bound socket");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    (directory, server)
}

/// Pump the terminal until it reads `wanted`, or ten seconds pass.
fn until_it_reads(server: &mut crate::Server, wanted: IsACommandRunning) -> bool {
    for _ in 0..400 {
        let _ = server.the_terminal_was_written_to();
        if server
            .alos_own_terminal()
            .is_some_and(|terminal| terminal.is_a_command_running() == wanted)
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

/// Type `text` and Enter at the terminal, through the keyboard's own road.
fn typed(server: &mut crate::Server, text: &str) {
    for character in text.chars() {
        let _ = server
            .a_key_reached_alos_own_terminal(WhatWasTyped::Character(character), Held::default());
    }
    let _ = server
        .a_key_reached_alos_own_terminal(WhatWasTyped::Named(ANamedKey::Enter), Held::default());
}

/// A server whose terminal has `cat` holding it.
fn with_a_command_running(named: &str) -> (tempfile::TempDir, crate::Server) {
    let (directory, mut server) = with_a_terminal(named);
    assert!(
        until_it_reads(&mut server, IsACommandRunning::No),
        "the shell never reached its prompt"
    );
    typed(&mut server, "cat");
    assert!(
        until_it_reads(&mut server, IsACommandRunning::Yes),
        "`cat` never held the terminal, so this proves nothing"
    );
    (directory, server)
}

/// Whether a process is gone, waiting up to two seconds for it to go.
fn it_goes(process: u32) -> bool {
    for _ in 0..80 {
        if !std::path::Path::new(&format!("/proc/{process}")).exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

/// **Idle, Close ends it at once**: no question, the window leaves the canvas,
/// and the shell is gone.
#[test]
fn an_idle_terminal_closes_at_once() {
    let (_directory, mut server) = with_a_terminal("alo-closing-idle");
    assert!(until_it_reads(&mut server, IsACommandRunning::No));
    let shell = server
        .alos_own_terminal()
        .expect("a terminal")
        .the_shells_process();
    assert_eq!(
        server.the_person_closes_the_terminal(),
        WhatClosingDid::Closed
    );
    assert!(server.alos_own_terminal().is_none());
    assert!(
        server.the_question_closing_asks().is_none(),
        "nothing was asked"
    );
    assert!(server.where_the_terminal_is_on_the_plane().is_none());
    assert!(it_goes(shell), "the shell outlived its window");
}

/// **Running, Close asks first, with the keyboard on Cancel**, and the
/// terminal is still open and the command still running while it asks.
#[test]
fn a_running_command_is_asked_about_first() {
    let (_directory, mut server) = with_a_command_running("alo-closing-asks");
    assert_eq!(
        server.the_person_closes_the_terminal(),
        WhatClosingDid::Asked
    );
    assert_eq!(
        server.the_question_closing_asks(),
        Some(OnTheQuestion::Cancel)
    );
    assert!(server.alos_own_terminal().is_some());
    assert!(
        server.the_terminal_is_taking_the_keys(),
        "the question cannot be answered with the keyboard"
    );
}

/// **The refusal path: Cancel leaves everything running.** Pressed as a
/// button, as Escape, and after a stray key — the terminal stays, its command
/// still holds it, and the shell is the same process.
#[test]
fn cancel_and_escape_leave_everything_running() {
    let (_directory, mut server) = with_a_command_running("alo-closing-cancel");
    let shell = server
        .alos_own_terminal()
        .expect("a terminal")
        .the_shells_process();

    assert_eq!(
        server.the_person_closes_the_terminal(),
        WhatClosingDid::Asked
    );
    server.the_question_is_answered(OnTheQuestion::Cancel);
    assert!(server.the_question_closing_asks().is_none());

    assert_eq!(
        server.the_person_closes_the_terminal(),
        WhatClosingDid::Asked
    );
    // **A key that is not one of the question's is taken and does nothing**:
    // it does not reach `cat`, and it does not answer.
    assert!(server.a_key_reached_alos_own_terminal(WhatWasTyped::Character('q'), Held::default()));
    assert!(server.the_question_closing_asks().is_some());
    assert!(
        server.a_key_reached_alos_own_terminal(
            WhatWasTyped::Named(ANamedKey::Escape),
            Held::default()
        )
    );
    assert!(
        server.the_question_closing_asks().is_none(),
        "Escape is Cancel"
    );

    let terminal = server
        .alos_own_terminal()
        .expect("the terminal is still open");
    assert_eq!(
        terminal.the_shells_process(),
        shell,
        "the shell was replaced"
    );
    assert_eq!(
        terminal.is_a_command_running(),
        IsACommandRunning::Yes,
        "Cancel stopped the command"
    );
    assert!(std::path::Path::new(&format!("/proc/{shell}")).exists());
}

/// **Close anyway ends it**: chosen with Tab and pressed with Enter, the
/// keyboard's whole road, and the shell and its command are gone.
#[test]
fn close_anyway_ends_the_terminal_and_its_command() {
    let (_directory, mut server) = with_a_command_running("alo-closing-anyway");
    let shell = server
        .alos_own_terminal()
        .expect("a terminal")
        .the_shells_process();
    assert_eq!(
        server.the_person_closes_the_terminal(),
        WhatClosingDid::Asked
    );
    let _ = server
        .a_key_reached_alos_own_terminal(WhatWasTyped::Named(ANamedKey::Tab), Held::default());
    assert_eq!(
        server.the_question_closing_asks(),
        Some(OnTheQuestion::CloseAnyway)
    );
    let _ = server
        .a_key_reached_alos_own_terminal(WhatWasTyped::Named(ANamedKey::Enter), Held::default());
    assert!(server.alos_own_terminal().is_none());
    assert!(server.the_question_closing_asks().is_none());
    assert!(it_goes(shell), "the shell outlived Close anyway");
}

/// **Nothing to close is nothing done**, and the next terminal opens fresh.
#[test]
fn closing_nothing_does_nothing_and_a_new_terminal_opens_after() {
    let (_directory, mut server) = with_a_terminal("alo-closing-nothing");
    server.the_terminal_ends();
    assert_eq!(
        server.the_person_closes_the_terminal(),
        WhatClosingDid::NothingWasOpen
    );
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    assert!(server.alos_own_terminal().is_some());
}
