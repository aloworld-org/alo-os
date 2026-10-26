//! **A real child process, a real pseudoterminal, and what the screen then
//! says.**
//!
//! These start processes. That is the point — the file they test is the one
//! place in this crate where something outside it exists, and a test of it
//! that mocked the child would be a test of the mock.
//!
//! # Why every test has a deadline
//!
//! A child writes when it is scheduled to, not when a test would like. So each
//! of these reads in a loop until what it is waiting for arrives or a deadline
//! passes — and **the deadline failing is itself an assertion**, with a message
//! saying what the screen held instead. A test that looped forever would hang
//! a gate, which is the worst way to report a failure.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, Instant};

use super::{AShellOnAPseudoterminal, WhatArrived};
use crate::how_big_the_grid_is::HowBig;
use crate::terminal_output_becomes_a_grid::TerminalOutput;
use crate::why_a_command_is_waiting::WhyItIsWaiting;

/// Long enough for a process to start and print on a loaded machine, short
/// enough that a failure is reported rather than waited for.
const PATIENCE: Duration = Duration::from_secs(10);

/// A screen a shell can write onto.
fn a_screen() -> TerminalOutput {
    TerminalOutput::of(HowBig::of(80, 24, 100).expect("eighty by twenty-four is a terminal"))
}

/// Read until the screen says something, or the deadline passes.
///
/// Answers whether the shell finished, so a caller can tell *it ended having
/// said this* from *it is still going and has said this so far*.
fn read_until(
    shell: &mut AShellOnAPseudoterminal,
    screen: &mut TerminalOutput,
    enough: impl Fn(&TerminalOutput) -> bool,
) -> bool {
    let deadline = Instant::now() + PATIENCE;
    let mut finished = false;
    while Instant::now() < deadline {
        match shell.what_it_wrote(screen).expect("the shell answers") {
            WhatArrived::Bytes(_) => {
                if enough(screen) {
                    return finished;
                }
            }
            WhatArrived::NothingYet => std::thread::sleep(Duration::from_millis(5)),
            WhatArrived::ItFinished => {
                finished = true;
                // **Read on after it finished.** Bytes written just before the
                // child exited are still in the kernel's queue, and stopping
                // here would lose the last line of every command that ends
                // quickly — which is most of them.
                if enough(screen) {
                    return true;
                }
                break;
            }
        }
    }
    finished
}

/// Whether any line of the screen contains this.
fn screen_says(screen: &TerminalOutput, what: &str) -> bool {
    screen.lines().iter().any(|line| line.contains(what))
}

#[test]
fn a_program_on_a_pseudoterminal_writes_onto_the_screen() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "echo hello from a real shell"],
    )
    .expect("this machine can start /bin/sh on a pseudoterminal");

    let mut screen = a_screen();
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "hello from a real shell")
    });

    assert!(
        screen_says(&screen, "hello from a real shell"),
        "the screen never showed what the shell printed. It held: {:?}",
        screen.lines()
    );
}

/// **A shell that has gone says so, rather than looking like one that is
/// quiet.** This is the distinction a `Result<usize>` would have lost, and a
/// caller that got it wrong would close a terminal the moment a program paused.
#[test]
fn a_shell_that_has_exited_reports_that_it_finished() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "exit 0"],
    )
    .expect("this machine can start /bin/sh");

    let mut screen = a_screen();
    let deadline = Instant::now() + PATIENCE;
    let mut finished = false;
    while Instant::now() < deadline && !finished {
        match shell.what_it_wrote(&mut screen).expect("the shell answers") {
            WhatArrived::ItFinished => finished = true,
            WhatArrived::NothingYet => std::thread::sleep(Duration::from_millis(5)),
            WhatArrived::Bytes(_) => {}
        }
    }
    assert!(
        finished,
        "a shell told to exit never reported that it had, within {PATIENCE:?}"
    );

    // **And it keeps saying so**, so a caller that reads once more does not
    // see it come back to life.
    assert_eq!(
        shell.what_it_wrote(&mut screen).expect("it still answers"),
        WhatArrived::ItFinished
    );
}

/// **A shell that is alive and quiet is not a shell that has gone.** The
/// control for the test above: if `NothingYet` and `ItFinished` were ever
/// confused, this would be the one that caught it.
#[test]
fn a_shell_that_is_only_quiet_does_not_report_that_it_finished() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "sleep 30"],
    )
    .expect("this machine can start /bin/sh");

    let mut screen = a_screen();
    // Give it long enough to have started and said nothing.
    std::thread::sleep(Duration::from_millis(200));
    for _ in 0..5 {
        let what = shell.what_it_wrote(&mut screen).expect("the shell answers");
        assert_ne!(
            what,
            WhatArrived::ItFinished,
            "a sleeping shell was reported as finished, so a terminal would close under somebody"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// **What a person types reaches the shell, and what it answers comes back.**
/// The round trip, which is the whole point of the file.
#[test]
fn what_a_person_types_reaches_the_shell() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        // No prompt and no echo, so the screen holds the answer and not the
        // question — this test is about the round trip, not about how a shell
        // decorates it.
        &["-s"],
    )
    .expect("this machine can start /bin/sh");

    let mut screen = a_screen();
    shell
        .a_person_typed(b"echo typed-and-answered\n")
        .expect("the shell takes what was typed");

    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "typed-and-answered")
    });

    assert!(
        screen_says(&screen, "typed-and-answered"),
        "what was typed never came back. The screen held: {:?}",
        screen.lines()
    );
}

/// **The shell is told how big its terminal is**, which is how a program knows
/// where to wrap. Asked of the shell itself rather than of our own type,
/// because what matters is what the **child** sees — this crate agreeing with
/// itself would prove nothing.
#[test]
fn the_shell_is_told_the_size_of_its_terminal() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "stty size"],
    )
    .expect("this machine can start /bin/sh");

    let mut screen = a_screen();
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "24 80")
    });

    assert!(
        screen_says(&screen, "24 80"),
        "the child was not told it had 24 lines and 80 columns. `stty size` said: {:?}",
        screen.lines()
    );
}

/// **The child sees the new size, asked of the child.**
///
/// `stty size` runs *inside* the shell and reads `TIOCGWINSZ`, so this is the
/// kernel's own answer rather than this crate agreeing with itself. It is the
/// only witness that matters: the child was the party that was wrong.
#[test]
fn the_child_is_told_when_the_window_changes() {
    let first = HowBig::of(80, 24, 100).expect("a terminal");
    let mut shell = AShellOnAPseudoterminal::running(first, "/bin/sh", &["-s"])
        .expect("this machine can start /bin/sh");
    let mut screen = a_screen();

    // Lay it out again before asking, at a size nothing would arrive at by
    // accident.
    let larger = HowBig::of(117, 31, 100).expect("a terminal");
    shell
        .the_window_changed(
            larger,
            crate::ACellInPixels {
                width: 8,
                height: 22,
            },
        )
        .expect("the pseudoterminal takes a new size");

    shell
        .a_person_typed(b"stty size\n")
        .expect("the shell takes what was typed");
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "31 117")
    });
    assert!(
        screen_says(&screen, "31 117"),
        "the child still believes its old size. `stty size` said: {:?}",
        screen.lines()
    );
    assert!(
        !screen_says(&screen, "24 80"),
        "the child answered its original size as well, so something is reporting both"
    );
}

/// **A program running at the time is told**, which is what `SIGWINCH` is for
/// and the half an `ioctl` alone would not give.
///
/// A shell waiting on `read` is woken, and `$COLUMNS` is re-read. Checked by
/// asking twice across one resize and requiring two different answers — a
/// single answer could be a stale one that happened to be right.
#[test]
fn a_resize_reaches_a_shell_that_is_already_running() {
    let first = HowBig::of(80, 24, 100).expect("a terminal");
    let mut shell = AShellOnAPseudoterminal::running(first, "/bin/sh", &["-s"])
        .expect("this machine can start /bin/sh");
    let mut screen = a_screen();

    shell
        .a_person_typed(b"stty size\n")
        .expect("the shell takes it");
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "24 80")
    });
    assert!(
        screen_says(&screen, "24 80"),
        "the shell did not answer its first size at all, so this test cannot see a change. \
         The screen held: {:?}",
        screen.lines()
    );

    let larger = HowBig::of(132, 43, 100).expect("a terminal");
    shell
        .the_window_changed(larger, crate::ACellInPixels::default())
        .expect("the pseudoterminal takes a new size");

    shell
        .a_person_typed(b"stty size\n")
        .expect("the shell takes it");
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "43 132")
    });
    assert!(
        screen_says(&screen, "43 132"),
        "a shell that was already running never saw the new size. The screen held: {:?}",
        screen.lines()
    );
}

/// **A command reading the terminal is waiting for the keyboard**, and the
/// terminal can now say so.
///
/// `cat` with no argument reads its standard input, which inside a
/// pseudoterminal **is** the terminal. So the honest answer is the keyboard, and
/// a person who was about to kill something learns it is waiting for them.
///
/// This is the three calls joined: which process is in front, what the machine
/// says about it, and what that reads as. Each was tested on its own before
/// this and nothing asked the question.
#[test]
fn a_command_reading_the_terminal_is_waiting_for_the_keyboard() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-s"],
    )
    .expect("a shell starts on a pseudoterminal");
    let mut screen = a_screen();

    // Nothing is running yet, so there is nothing to ask about.
    assert_eq!(
        shell.why_the_command_in_front_is_waiting(),
        None,
        "a person at a prompt has no command waiting for anything"
    );

    shell
        .a_person_typed(b"cat\n")
        .expect("what a person types reaches the shell");

    // Wait until `cat` is really in front **and has settled**, rather than
    // guessing at a delay.
    //
    // **Settled, and not merely in front**, which is the repair of 2026-10-26:
    // this took the first answer, and on a loaded hosted runner the first
    // answer can arrive while `cat` is still starting — `StillComputing`, which
    // is true of that moment. It failed one run in the forty failed runs before
    // it, on a change that touched nothing here. The claim is about the answer
    // once `cat` is blocked reading, so the last answer within the same five
    // seconds is what is judged, and one that never becomes the keyboard still
    // fails below, naming what it said.
    let mut waiting = None;
    for _ in 0..200 {
        drop(shell.what_it_wrote(&mut screen));
        if let Some(what) = shell.why_the_command_in_front_is_waiting() {
            let settled = what.why == WhyItIsWaiting::ForTheKeyboard;
            waiting = Some(what);
            if settled {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }

    let what = waiting.expect("`cat` reaches the front of the terminal");
    assert_ne!(
        what.process,
        shell.the_shell_itself(),
        "the answer is about the shell rather than the command in front of it"
    );
    assert_eq!(
        what.why,
        WhyItIsWaiting::ForTheKeyboard,
        "a command reading the terminal is waiting for the person, and this said {:?}",
        what.why
    );

    // And the sentence a person reads is words rather than a key.
    assert!(
        !what.why.the_word().key().as_str().is_empty(),
        "the answer carries no word for a person to read"
    );
}

/// **And when the command finishes, there is nothing to answer again.**
///
/// The control for the test above: a reader that answered *waiting for the
/// keyboard* about the shell would pass that one and fail this one, because the
/// shell is always reading its terminal.
#[test]
fn when_the_command_finishes_there_is_nothing_waiting_again() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-s"],
    )
    .expect("a shell starts on a pseudoterminal");
    let mut screen = a_screen();

    shell
        .a_person_typed(b"cat\n")
        .expect("what a person types reaches the shell");
    let mut reached = false;
    for _ in 0..200 {
        drop(shell.what_it_wrote(&mut screen));
        if shell.why_the_command_in_front_is_waiting().is_some() {
            reached = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    assert!(
        reached,
        "`cat` never reached the front, so this proves nothing"
    );

    // End of input: `cat` reads it and exits, and the shell is in front again.
    shell
        .a_person_typed(&[0x04])
        .expect("what a person types reaches the shell");

    let mut back = false;
    for _ in 0..200 {
        drop(shell.what_it_wrote(&mut screen));
        if shell.why_the_command_in_front_is_waiting().is_none() {
            back = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    assert!(
        back,
        "the command finished and the terminal still reports something waiting"
    );
}

/// **A shell started here is told what terminal it is on.**
///
/// Until 2026-10-17 it was told nothing, and `clear` answered *TERM environment
/// variable not set.* — so alo's own terminal could not clear its own screen,
/// and `less`, `vim` and `top` would have refused in the same way. It was found
/// by typing `clear` into a terminal on a compositor and reading the screen
/// back, which is step 13 of the boot walk; a unit test of this crate could not
/// have found it, because the fault was an empty map that every type agreed
/// with.
///
/// **What this cannot check** is that the name resolves on the machine the
/// terminal runs on: a terminfo database is a property of an installed system,
/// not of this crate, and a test that read `/usr/share/terminfo` would pass or
/// fail for reasons that have nothing to do with the code. What it holds is
/// that a name is given at all and that it is shaped like one.
#[test]
fn a_shell_is_told_what_terminal_it_is_on() {
    let told = super::what_the_child_is_told();
    let name = told
        .get("TERM")
        .expect("a shell is told what terminal it is on");
    assert!(
        !name.is_empty(),
        "TERM is set to nothing, which curses reads the same way as unset"
    );
    assert!(
        !name.chars().any(char::is_whitespace),
        "TERM is {name:?}, which is not a name a terminfo database can be asked for"
    );
}

/// **A real shell's mark reaches the terminal that read it.**
///
/// The reader has its own fifteen tests and every one of them hands bytes
/// straight to it. **None of them proves the terminal is wired to it** — a
/// constant at the seam would satisfy all fifteen, which is the fault
/// `docs/autonomy/` names as *honest fixtures are silent about wiring*. So this
/// one starts `/bin/sh`, has it `printf` the sequence onto a real
/// pseudoterminal, and asks the terminal what it heard.
///
/// `BEL` terminates rather than `ST`, because `\033\\` inside a shell string
/// inside a Rust string is three levels of escaping to get wrong and both
/// terminators are read.
#[test]
fn a_mark_a_real_shell_printed_reaches_the_terminal() {
    use crate::what_the_shell_says_about_the_prompt::AShellSaid;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;

    let command = "git push";
    let encoded = STANDARD.encode(command.as_bytes());
    let script = format!(
        "printf '\\033]133;A\\007'; printf '\\033]133;alo-typed;{encoded};{}\\007'; echo done",
        command.len()
    );

    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", &script],
    )
    .expect("this machine can start /bin/sh on a pseudoterminal");

    let mut screen = a_screen();
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "done")
    });

    let heard = shell.what_the_shell_said();
    assert_eq!(
        heard,
        vec![
            Ok(AShellSaid::APromptIsAboutToBeDrawn),
            Ok(AShellSaid::TheyTyped(command.to_owned())),
        ],
        "a real shell printed two marks onto a real pseudoterminal and the terminal heard \
         {heard:?}. The screen held: {:?}",
        screen.lines()
    );
}

/// **The marks are drained, so one reaches one caller once.**
///
/// A caller that read without draining would explain the same command on every
/// frame for as long as the person left it on the screen — sixty times a second
/// for a line they typed once.
#[test]
fn asking_twice_what_the_shell_said_does_not_say_it_twice() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "printf '\\033]133;C\\007'; echo done"],
    )
    .expect("this machine can start /bin/sh on a pseudoterminal");

    let mut screen = a_screen();
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "done")
    });

    assert_eq!(
        shell.what_the_shell_said().len(),
        1,
        "the mark arrives once"
    );
    assert!(
        shell.what_the_shell_said().is_empty(),
        "and asking again says nothing, because the first ask drained it"
    );
}

/// **Ordinary output carries no marks**, which is what a shell with no snippet
/// installed produces all day.
///
/// The feature says plainly that without the snippet nothing happens, and this
/// is that sentence as a test: a shell doing ordinary work leaves the reader
/// with nothing to report.
#[test]
fn a_shell_with_no_snippet_says_nothing_about_the_prompt() {
    let mut shell = AShellOnAPseudoterminal::running(
        HowBig::of(80, 24, 100).expect("a terminal"),
        "/bin/sh",
        &["-c", "echo one; echo two; echo done"],
    )
    .expect("this machine can start /bin/sh on a pseudoterminal");

    let mut screen = a_screen();
    read_until(&mut shell, &mut screen, |screen| {
        screen_says(screen, "done")
    });

    assert!(
        shell.what_the_shell_said().is_empty(),
        "a shell that printed three ordinary lines said something about the prompt"
    );
}
