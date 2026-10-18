//! **A real interactive `bash`, with the snippet sourced, and a real key
//! pressed.**
//!
//! A test that read the snippet and checked it contained the right words would
//! be a test of the reading. What has to hold is that a shell a person actually
//! runs, with readline actually loaded, turns `Ctrl`+`X` `Ctrl`+`A` into bytes
//! that [`crate::what_the_shell_says_about_the_prompt`] reads back as the line
//! they typed.
//!
//! So these start `bash --norc -i` on a pseudoterminal. `-i` is not optional:
//! `bind -x` needs readline, and readline is only there for an interactive
//! shell. `--norc` keeps somebody's own configuration out of the measurement.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use std::time::{Duration, Instant};

use super::for_the_shell_called;
use crate::a_shell_on_a_pseudoterminal::{AShellOnAPseudoterminal, WhatArrived};
use crate::how_big_the_grid_is::HowBig;
use crate::terminal_output_becomes_a_grid::TerminalOutput;
use crate::what_the_shell_says_about_the_prompt::AShellSaid;

/// Long enough for an interactive bash to start and settle on a loaded machine.
const PATIENCE: Duration = Duration::from_secs(15);

/// `Ctrl`+`X`, then `Ctrl`+`A`.
const THE_KEY: &[u8] = &[0x18, 0x01];

/// An interactive bash with the snippet sourced, and everything it has said.
///
/// Writes the snippet to a file rather than pasting it down the pseudoterminal:
/// a here-document typed into a terminal is interpreted by readline on the way
/// in, and what arrives is not what was sent.
fn a_bash_with_the_snippet() -> (AShellOnAPseudoterminal, TerminalOutput, tempish::Dir) {
    let dir = tempish::Dir::new();
    let snippet = dir.path().join("alo.bash");
    std::fs::write(
        &snippet,
        for_the_shell_called("/bin/bash").expect("alo ships integration for bash"),
    )
    .expect("the snippet is written");

    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/bash",
        &["--norc", "-i"],
    )
    .expect("this machine can start an interactive bash on a pseudoterminal");

    let mut screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));

    // Settle the prompt before sourcing, so the marks that follow belong to the
    // snippet rather than to bash starting up.
    drain(&mut shell, &mut screen, Duration::from_millis(400));
    shell
        .a_person_typed(format!("source {}\n", snippet.display()).as_bytes())
        .expect("the shell takes what was typed");
    drain(&mut shell, &mut screen, Duration::from_millis(800));

    (shell, screen, dir)
}

/// Read for a while, letting the shell write whatever it is going to.
fn drain(shell: &mut AShellOnAPseudoterminal, screen: &mut TerminalOutput, how_long: Duration) {
    let until = Instant::now() + how_long;
    while Instant::now() < until {
        match shell.what_it_wrote(screen) {
            Ok(WhatArrived::Bytes(_)) => {}
            Ok(WhatArrived::NothingYet) => std::thread::sleep(Duration::from_millis(5)),
            Ok(WhatArrived::ItFinished) | Err(_) => return,
        }
    }
}

/// **The snippet makes a real bash tell alo where its prompt is.**
///
/// The four cycle marks are the industry format, and a person who already has
/// shell integration for another terminal gets these without the snippet. This
/// asserts alo OS's own snippet produces them too.
#[test]
fn a_real_bash_with_the_snippet_says_where_its_prompt_is() {
    let (mut shell, mut screen, _dir) = a_bash_with_the_snippet();

    shell
        .a_person_typed(b"echo hello\n")
        .expect("the shell takes what was typed");
    let deadline = Instant::now() + PATIENCE;
    let mut said = Vec::new();
    while Instant::now() < deadline {
        drain(&mut shell, &mut screen, Duration::from_millis(100));
        said.extend(shell.what_the_shell_said());
        if said
            .iter()
            .any(|it| matches!(it, Ok(AShellSaid::TheCommandFinished { .. })))
        {
            break;
        }
    }

    let marks: Vec<_> = said.into_iter().flatten().collect();
    assert!(
        marks
            .iter()
            .any(|it| matches!(it, AShellSaid::APromptIsAboutToBeDrawn)),
        "no prompt-start mark arrived from a real bash with the snippet. It said: {marks:?}"
    );
    assert!(
        marks
            .iter()
            .any(|it| matches!(it, AShellSaid::TheCommandFinished { status: Some(0) })),
        "`echo hello` finished and no mark said so, or it said a status that was not zero. It \
         said: {marks:?}"
    );
}

/// **The key a person presses reaches alo as the line they typed.**
///
/// This is the whole feature's input half in one test: an interactive bash,
/// readline loaded, the binding the snippet made, the key pressed, and the
/// command read back without it ever being run.
#[test]
fn pressing_the_key_sends_the_line_that_has_not_been_run() {
    let (mut shell, mut screen, _dir) = a_bash_with_the_snippet();

    // Typed and **not** followed by Enter: the point is that alo learns a
    // command before it runs.
    shell
        .a_person_typed(b"rm -rf /important")
        .expect("the shell takes what was typed");
    drain(&mut shell, &mut screen, Duration::from_millis(400));
    drop(shell.what_the_shell_said());

    shell.a_person_typed(THE_KEY).expect("the key reaches it");

    let deadline = Instant::now() + PATIENCE;
    let mut typed = None;
    while Instant::now() < deadline && typed.is_none() {
        drain(&mut shell, &mut screen, Duration::from_millis(100));
        for said in shell.what_the_shell_said() {
            if let Ok(AShellSaid::TheyTyped(line)) = said {
                typed = Some(line);
            }
        }
    }

    assert_eq!(
        typed.as_deref(),
        Some("rm -rf /important"),
        "pressing Ctrl+X Ctrl+A in a real bash did not reach alo as the typed line. The screen \
         held: {:?}",
        screen.lines()
    );
}

/// **A command with an accent in it declares its length in bytes.**
///
/// `${#line}` counts characters and `wc -c` counts bytes, and for anything
/// outside ASCII the two differ. A snippet that declared the character count
/// would have every accented command refused by the reader's own length check —
/// and would work perfectly in every ASCII test anybody wrote.
#[test]
fn a_command_that_is_not_ascii_declares_its_length_in_bytes() {
    let (mut shell, mut screen, _dir) = a_bash_with_the_snippet();

    let awkward = "echo café";
    shell
        .a_person_typed(awkward.as_bytes())
        .expect("the shell takes what was typed");
    drain(&mut shell, &mut screen, Duration::from_millis(400));
    drop(shell.what_the_shell_said());

    shell.a_person_typed(THE_KEY).expect("the key reaches it");

    let deadline = Instant::now() + PATIENCE;
    let mut heard = None;
    while Instant::now() < deadline && heard.is_none() {
        drain(&mut shell, &mut screen, Duration::from_millis(100));
        for said in shell.what_the_shell_said() {
            match said {
                Ok(AShellSaid::TheyTyped(line)) => heard = Some(Ok(line)),
                Err(why) => heard = Some(Err(why)),
                Ok(_) => {}
            }
        }
    }

    match heard {
        Some(Ok(line)) => assert_eq!(line, awkward, "the accented command came back changed"),
        Some(Err(why)) => panic!(
            "an accented command was refused, which is what declaring a character count rather \
             than a byte count does: {why}"
        ),
        None => panic!(
            "nothing came back for an accented command. The screen held: {:?}",
            screen.lines()
        ),
    }
}

/// A directory that goes away, without a dependency for it.
mod tempish {
    /// A directory under the system temporary directory, removed on drop.
    pub struct Dir {
        /// Where it is.
        at: std::path::PathBuf,
    }

    impl Dir {
        /// Make one, named for this process and a counter.
        #[expect(
            clippy::expect_used,
            reason = "in a test, a panic naming the failure is the report"
        )]
        pub fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let which = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let at =
                std::env::temp_dir().join(format!("alo-snippet-{}-{which}", std::process::id()));
            std::fs::create_dir_all(&at).expect("a temporary directory can be made");
            Self { at }
        }

        /// Where it is.
        pub fn path(&self) -> &std::path::Path {
            &self.at
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            drop(std::fs::remove_dir_all(&self.at));
        }
    }
}
