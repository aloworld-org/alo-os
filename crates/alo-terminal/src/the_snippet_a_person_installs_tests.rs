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

/// An interactive bash with the snippet sourced and a fake `alo-try` on its
/// path, so that `try` is defined.
///
/// The fake is a shell script that prints what it was given. Nothing here runs
/// a real box — what is being tested is whether the **word** reaches a command
/// at all, which is the half the snippet decides.
fn a_bash_that_can_find_alo_try() -> (AShellOnAPseudoterminal, TerminalOutput, tempish::Dir) {
    let dir = tempish::Dir::new();
    let snippet = dir.path().join("alo.bash");
    std::fs::write(
        &snippet,
        for_the_shell_called("/bin/bash").expect("alo ships integration for bash"),
    )
    .expect("the snippet is written");

    let fake = dir.path().join("alo-try");
    std::fs::write(&fake, b"#!/bin/sh\necho \"a box would have run: $*\"\n")
        .expect("the stand-in is written");
    {
        use std::os::unix::fs::PermissionsExt;
        let mut how = std::fs::metadata(&fake)
            .expect("the stand-in is there")
            .permissions();
        how.set_mode(0o755);
        std::fs::set_permissions(&fake, how).expect("the stand-in can be made runnable");
    }

    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/bash",
        &["--norc", "-i"],
    )
    .expect("this machine can start an interactive bash on a pseudoterminal");
    let mut screen = TerminalOutput::of(HowBig::of(80, 24, 100).expect("a terminal"));

    drain(&mut shell, &mut screen, Duration::from_millis(400));
    // The folder holding the stand-in goes on the path **before** the snippet
    // is sourced, because the snippet asks once whether the command is there.
    shell
        .a_person_typed(format!("export PATH={}:$PATH\n", dir.path().display()).as_bytes())
        .expect("the shell takes what was typed");
    drain(&mut shell, &mut screen, Duration::from_millis(200));
    shell
        .a_person_typed(format!("source {}\n", snippet.display()).as_bytes())
        .expect("the shell takes what was typed");
    drain(&mut shell, &mut screen, Duration::from_millis(800));

    (shell, screen, dir)
}

/// **`try` reaches `alo-try`, and the word is a function.**
///
/// The caller-side half of the sealed box: the owner chose *a word before the
/// command* on 2026-10-18, and this is the evidence that typing the word
/// reaches the command rather than the shell saying *command not found*.
#[test]
fn the_word_try_reaches_the_command_when_it_is_installed() {
    let (mut shell, mut screen, _dir) = a_bash_that_can_find_alo_try();

    shell
        .a_person_typed(b"type -t try\n")
        .expect("the shell takes what was typed");
    drain(&mut shell, &mut screen, Duration::from_millis(600));

    let said = screen.lines().join("\n");
    assert!(
        said.contains("function"),
        "`try` is not a function, so the word reaches nothing: {said}"
    );

    shell
        .a_person_typed(b"try ./install.sh\n")
        .expect("the shell takes what was typed");
    drain(&mut shell, &mut screen, Duration::from_millis(600));

    let said = screen.lines().join("\n");
    assert!(
        said.contains("a box would have run: ./install.sh"),
        "the word did not carry the command to `alo-try`: {said}"
    );
}

/// **And `try` is not defined at all when `alo-try` is not installed.**
///
/// The sharper half. A `try` defined unconditionally would answer *alo-try:
/// command not found* from inside itself, which tells a person their shell is
/// broken when what is true is that the feature is not installed. The snippet
/// asks before it defines.
#[test]
fn the_word_try_is_absent_when_the_command_is_not_installed() {
    let (mut shell, mut screen, _dir) = a_bash_with_the_snippet();

    shell
        .a_person_typed(b"type -t try; echo done-asking\n")
        .expect("the shell takes what was typed");
    drain(&mut shell, &mut screen, Duration::from_millis(600));

    let said = screen.lines().join("\n");
    assert!(
        said.contains("done-asking"),
        "the question never finished, so the answer below means nothing: {said}"
    );
    assert!(
        !said.contains("function"),
        "`try` is defined on a machine with no `alo-try`, so typing it would report a command \
         not found from inside itself: {said}"
    );
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

// ---------------------------------------------------------------------------
// The one-press way: adding the snippet to a startup file.
//
// These do not start a shell. What the snippet *does* once sourced is measured
// above, by a real bash; what is measured here is the file edit, and the thing
// worth holding is that pressing twice cannot leave two copies.
// ---------------------------------------------------------------------------

use std::path::Path;

use super::{Added, NotAdded, add_to, already_in, the_line_it_begins_with};

/// How many copies of the snippet a startup file holds.
///
/// Counts the snippet's **own first line** rather than a string written here,
/// for the reason `the_line_it_begins_with` gives: a marker spelled twice is
/// two sources of truth.
fn copies_in(at: &Path) -> usize {
    let held = std::fs::read_to_string(at).unwrap_or_default();
    let marker = the_line_it_begins_with(
        super::for_the_shell_called("bash").expect("alo ships integration for bash"),
    );
    held.matches(marker).count()
}

/// **A person who has never edited a startup file still gets the integration.**
#[test]
fn a_startup_file_that_is_not_there_yet_is_made() {
    let dir = tempish::Dir::new();
    let at = dir.path().join(".bashrc");
    assert!(!at.exists(), "the premise: nothing is there yet");

    assert_eq!(add_to(&at, "/bin/bash"), Ok(Added::Appended));
    assert_eq!(copies_in(&at), 1, "one copy, in a file that did not exist");
}

/// **Pressing twice leaves one copy**, which is the whole point of the check.
///
/// Two copies would bind `Ctrl`+`X` `Ctrl`+`A` twice and prepend the prompt
/// marks twice — a prompt measured wrong wraps in the wrong place, which the
/// snippet's own comments call a worse bug than no integration.
#[test]
fn adding_it_twice_leaves_one_copy() {
    let dir = tempish::Dir::new();
    let at = dir.path().join(".bashrc");

    assert_eq!(add_to(&at, "bash"), Ok(Added::Appended));
    let after_once = std::fs::read_to_string(&at).expect("the file the first add wrote");

    assert_eq!(
        add_to(&at, "bash"),
        Ok(Added::AlreadyThere),
        "the second press is told nothing was written, not that it failed"
    );
    assert_eq!(copies_in(&at), 1, "still one copy after two presses");
    assert_eq!(
        std::fs::read_to_string(&at).expect("the file after the second add"),
        after_once,
        "the second press wrote nothing at all, byte for byte"
    );
}

/// **A shell alo ships nothing for is refused, and nothing is written.**
#[test]
fn a_shell_alo_ships_nothing_for_is_refused_and_nothing_is_written() {
    let dir = tempish::Dir::new();
    let at = dir.path().join(".zshrc");

    assert_eq!(
        add_to(&at, "/usr/bin/zsh"),
        Err(NotAdded::NoIntegrationFor("/usr/bin/zsh".to_owned())),
        "the shell is named back, so a person reads which one we have nothing for"
    );
    assert!(
        !at.exists(),
        "a refusal that created the file would have written a bash snippet for a zsh user"
    );
}

/// **What was already in the startup file is kept, and stays first.**
#[test]
fn what_was_already_in_the_startup_file_is_kept() {
    let dir = tempish::Dir::new();
    let at = dir.path().join(".bashrc");
    let theirs = "export EDITOR=vi\nalias ll='ls -l'";
    std::fs::write(&at, theirs).expect("a startup file of their own");

    assert_eq!(add_to(&at, "bash"), Ok(Added::Appended));

    let after = std::fs::read_to_string(&at).expect("the file after");
    assert!(
        after.starts_with(theirs),
        "their own lines are still at the top, unchanged: {after:?}"
    );
    assert_eq!(copies_in(&at), 1);
    assert!(
        after.contains("\n\n#"),
        "a blank line separates their lines from the snippet rather than running on"
    );
}

/// **A startup file with no final newline does not get the snippet welded onto
/// its last line.**
///
/// That would be a syntax error in their shell, on every new terminal, caused
/// by us.
#[test]
fn a_file_with_no_final_newline_is_not_run_onto() {
    let dir = tempish::Dir::new();
    let at = dir.path().join(".bashrc");
    std::fs::write(&at, "alias ll='ls -l'").expect("a startup file with no final newline");

    assert_eq!(add_to(&at, "bash"), Ok(Added::Appended));

    let after = std::fs::read_to_string(&at).expect("the file after");
    assert!(
        after.starts_with("alias ll='ls -l'\n"),
        "their last line ends before anything of ours begins: {after:?}"
    );
}

/// **A read-only startup file never reports a write that did not happen.**
///
/// Asserts the **invariant that holds either way** rather than branching on
/// whether this process is root: a read-only file refuses the append as the
/// person and takes it as root, and a test that needed to know which would
/// need a dependency to ask. What must never happen is either half lying —
/// `Appended` with nothing written, or a refusal that wrote something.
#[test]
fn a_read_only_startup_file_never_reports_a_write_that_did_not_happen() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempish::Dir::new();
    let at = dir.path().join(".bashrc");
    std::fs::write(&at, "alias ll='ls -l'\n").expect("a startup file of their own");
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o444))
        .expect("making it read-only");

    match add_to(&at, "bash") {
        Ok(Added::Appended) => assert_eq!(
            copies_in(&at),
            1,
            "it said it appended, so exactly one copy has to be there"
        ),
        Ok(Added::AlreadyThere) => {
            panic!("nothing of ours was in the file, so AlreadyThere is wrong")
        }
        Err(NotAdded::WouldNotWrite { at: named, why }) => {
            assert!(
                named.ends_with(".bashrc"),
                "the refusal names the file: {named}"
            );
            assert!(!why.is_empty(), "the refusal says what the machine said");
            assert_eq!(copies_in(&at), 0, "it refused, so nothing was written");
        }
        other => panic!("a read-only startup file gives one of those three, got {other:?}"),
    }
    // Either way their own line survived, which is the thing an append protects.
    let after = std::fs::read_to_string(&at).expect("the file is still readable");
    assert!(after.starts_with("alias ll='ls -l'"), "{after:?}");
}

/// **`already_in` tells *not there* apart from *we ship nothing for it*.**
///
/// Collapsing the two is how a `zsh` user gets offered a `bash` snippet.
#[test]
fn already_in_tells_not_there_apart_from_no_snippet_at_all() {
    assert_eq!(already_in("", "bash"), Some(false), "bash, nothing there");
    assert_eq!(already_in("", "zsh"), None, "no snippet for zsh at all");

    let snippet = super::for_the_shell_called("bash").expect("alo ships integration for bash");
    assert_eq!(
        already_in(snippet, "bash"),
        Some(true),
        "the snippet's own text is recognised as itself"
    );
}

/// **The marker is the snippet's own first line**, not a copy of it.
///
/// This is the guard on the two-sources-of-truth hazard the function's comment
/// names: if somebody edits `alo.bash`'s first line, the marker follows it, and
/// a startup file written before the edit is still recognised only if the line
/// is unchanged — which is why the line is a comment nobody has a reason to
/// reword.
#[test]
fn the_marker_is_the_snippets_own_first_line() {
    let snippet = super::for_the_shell_called("bash").expect("alo ships integration for bash");
    let marker = the_line_it_begins_with(snippet);

    assert!(
        snippet.starts_with(marker),
        "the marker is where the snippet begins"
    );
    assert!(
        marker.starts_with('#'),
        "the marker is a comment, so a shell sourcing it does nothing with it: {marker:?}"
    );
    assert!(
        marker.len() > 20,
        "a short marker would match something a person wrote: {marker:?}"
    );
}
