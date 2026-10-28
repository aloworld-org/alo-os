//! **Real processes on a real kernel**, which is the only thing that can say
//! whether this reads `/proc` correctly.
//!
//! `CLAUDE.md` asks for an integration test on real hardware for anything that
//! touches the machine, and asks what would have to be true for a green check
//! to pass while the product is broken. For a `/proc` reader the answer is
//! *the fields moved* — so every case here puts a process into a known state
//! and reads it, rather than parsing a captured string that cannot go stale.
//!
//! Each case carries its control, because a reader that answered the same
//! thing about everything would pass every one of these taken alone.

use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::*;
use crate::why_a_command_is_waiting::{WhyItIsWaiting, why_it_is_waiting};

/// How long to let a process settle into the state it was started for.
///
/// Polled rather than slept through: a fixed sleep is either slow or flaky, and
/// on a loaded builder it is both.
const LONG_ENOUGH: Duration = Duration::from_secs(5);

/// Start this command with its input coming from a pipe nobody writes to.
fn started(program: &str, args: &[&str]) -> Child {
    Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the test's own command starts")
}

/// Wait until this process's evidence satisfies the question, or give up.
///
/// Answers the evidence that satisfied it, so a test asserts on the thing it
/// waited for rather than reading again and hoping it has not moved.
///
/// # Wait for the state the test is about, not for the first parked moment
///
/// **A process is parked long before it is parked in the thing it was started
/// to do.** It parks while loading, while mapping libraries, while the shell
/// that spawned it hands over. Waiting for `!running` and then asserting the
/// answer reads whichever of those states happened to be current.
///
/// That is not hypothetical: `a_process_waiting_on_an_empty_pipe_reads_as_parked`
/// waited for `!running` and answered `NobodyCanTell` on a loaded builder,
/// because the `wchan` it caught was a startup function in neither table. The
/// test was right about the product and wrong about when to look.
///
/// So a caller waits for the **answer**, and the timeout is the failure. A
/// reader that was genuinely broken never reaches the state and the test fails
/// with the timeout rather than passing by luck.
fn settles_into(process: u32, question: impl Fn(&TheEvidence) -> bool) -> Option<TheEvidence> {
    let until = Instant::now() + LONG_ENOUGH;
    loop {
        if let Some(evidence) = what_can_be_found_out_about(process)
            && question(&evidence)
        {
            return Some(evidence);
        }
        if Instant::now() >= until {
            return what_can_be_found_out_about(process).filter(|evidence| question(evidence));
        }
        sleep(Duration::from_millis(25));
    }
}

/// **A process that is working reads as running**, and that is the control for
/// everything else here.
///
/// Without it, a reader that said *parked* about everything would pass every
/// other test in this file.
#[test]
fn a_busy_process_reads_as_running() {
    let mut busy = started("sh", &["-c", "while :; do :; done"]);
    let evidence = settles_into(busy.id(), |evidence| evidence.running);
    let _ = busy.kill();
    let _ = busy.wait();

    let evidence = evidence.expect("a busy loop is on a processor within five seconds");
    assert!(evidence.running, "a busy loop did not read as running");
    assert_eq!(
        why_it_is_waiting(&evidence),
        WhyItIsWaiting::StillComputing,
        "a busy loop was not reported as working"
    );
}

/// **A process parked on an empty pipe reads as parked**, which is the other
/// half of the control above.
#[test]
fn a_process_waiting_on_an_empty_pipe_reads_as_parked() {
    let mut waiting = started("cat", &[]);
    // **The answer, not merely `!running`** — see `settles_into`. A process
    // that has not yet reached its own read is parked in its loader, and what
    // it is parked in then says nothing about what it is waiting for.
    let evidence = settles_into(waiting.id(), |evidence| {
        !evidence.running && why_it_is_waiting(evidence) == WhyItIsWaiting::ForAnotherProcess
    });
    let _ = waiting.kill();
    let _ = waiting.wait();

    let evidence =
        evidence.expect("a cat on an idle pipe is explained by the pipe within five seconds");
    assert!(!evidence.running, "a parked process read as running");
    assert_eq!(
        evidence.reads_from,
        WhatItReadsFrom::APipe,
        "a pipe was not read as a pipe"
    );
    assert_eq!(
        why_it_is_waiting(&evidence),
        WhyItIsWaiting::ForAnotherProcess
    );
}

/// **A process reading a terminal is told apart from one reading a pipe**, and
/// the two are the same kernel function.
///
/// The measurement this whole module rests on, as a running check: the terminal
/// comes from `alo-terminal`'s own pseudoterminal, so this is the real
/// arrangement and not an approximation of one.
#[test]
fn a_command_on_a_terminal_is_waiting_for_the_keyboard() {
    let shell = crate::AShellOnAPseudoterminal::running(
        crate::HowBig::of(80, 24, 100).expect("a grid"),
        "cat",
        &[],
    )
    .expect("a pseudoterminal with cat on it");

    let in_front = shell
        .the_command_in_front()
        .expect("the terminal says who is in front");
    // The same rule as the pipe below: wait for the answer rather than for the
    // first parked moment, because a process parked in its loader has a
    // `wchan` that describes loading.
    let evidence = settles_into(in_front, |evidence| {
        !evidence.running
            && evidence.reads_from == WhatItReadsFrom::ATerminal
            && why_it_is_waiting(evidence) == WhyItIsWaiting::ForTheKeyboard
    })
    .expect("cat on a terminal is explained by the keyboard within five seconds");

    assert_eq!(evidence.reads_from, WhatItReadsFrom::ATerminal);
    assert_eq!(
        why_it_is_waiting(&evidence),
        WhyItIsWaiting::ForTheKeyboard,
        "a command waiting for somebody to type was not reported as such"
    );
}

/// **The process in front is not the shell**, which is the distinction the
/// whole question turns on.
///
/// A shell sits reading its own terminal whenever a person is at a prompt, so a
/// reader that asked about the shell would answer *waiting for the keyboard*
/// every time — including while a command runs, which is the one case anybody
/// asks about. Asserted as two different numbers rather than described.
///
/// **It uses the person's own shell and types into it**, because that is the
/// only arrangement where the question has the answer the product needs. A
/// shell started as `sh -c "sleep 30"` is the wrong test twice over: a shell
/// given one command `exec`s it, so the pid does not change; and a
/// non-interactive shell has no job control, so it never moves the terminal's
/// foreground group at all. Both were measured, by this test failing.
#[test]
fn the_command_in_front_is_not_the_shell_running_it() {
    let mut shell = crate::AShellOnAPseudoterminal::the_persons_own(
        crate::HowBig::of(80, 24, 100).expect("a grid"),
        None,
    )
    .expect("the person's own shell on a pseudoterminal");
    let itself = shell.the_shell_itself();

    // Let the shell reach its prompt before typing, or the bytes land in a
    // shell that is still starting and are lost.
    sleep(Duration::from_millis(400));
    shell
        .a_person_typed(b"sleep 30\n")
        .expect("a person can type into their own shell");

    let until = Instant::now() + LONG_ENOUGH;
    let mut in_front = shell.the_command_in_front();
    while in_front == Some(itself) && Instant::now() < until {
        sleep(Duration::from_millis(50));
        in_front = shell.the_command_in_front();
    }

    let in_front = in_front.expect("the terminal says which group is in front");
    assert_ne!(
        in_front, itself,
        "the group in front is the shell itself while a command is running, so asking about it \
         would answer about the shell rather than about the command a person is waiting on"
    );

    // And the command really is the one that was typed: it is parked, which a
    // shell at a prompt also is, so the stronger check is that the answer is
    // about a *different* process that exists.
    assert!(
        what_can_be_found_out_about(in_front).is_some(),
        "the group in front is not a process this machine knows about"
    );
}

/// **With nothing running, the shell is in front**, which is how a terminal
/// knows a person is at a prompt rather than waiting on something.
///
/// The control for the test above: without it, a reader that always answered
/// *some other number* would pass that one.
#[test]
fn with_nothing_running_the_shell_itself_is_in_front() {
    let shell = crate::AShellOnAPseudoterminal::the_persons_own(
        crate::HowBig::of(80, 24, 100).expect("a grid"),
        None,
    )
    .expect("the person's own shell on a pseudoterminal");
    let itself = shell.the_shell_itself();

    let until = Instant::now() + LONG_ENOUGH;
    let mut in_front = shell.the_command_in_front();
    while in_front != Some(itself) && Instant::now() < until {
        sleep(Duration::from_millis(50));
        in_front = shell.the_command_in_front();
    }
    assert_eq!(
        in_front,
        Some(itself),
        "a shell with nothing running is not in front of its own terminal"
    );
}

/// **A process that has gone has nothing to say, and that is not an error.**
///
/// `/proc/PID` disappears the moment a process exits, and a terminal that
/// treated an ordinary race as a fault would report one every time a command
/// finished while somebody was asking about it.
#[test]
fn a_process_that_has_gone_answers_nothing() {
    let mut brief = started("true", &[]);
    let _ = brief.wait();
    let gone = brief.id();

    // A pid this high is not in use either, which is the second half: the
    // answer must be the same for *finished* and for *never existed*, since a
    // reader cannot tell them apart and must not pretend to.
    for process in [gone, 4_000_000] {
        assert!(
            what_can_be_found_out_about(process).is_none(),
            "process {process} is not there and something was reported about it"
        );
    }
}

/// **A name with a space in it does not move the field that is read.**
///
/// `/proc/PID/stat` holds the executable's name in parentheses as its second
/// field, so counting fields from the left finds the wrong one for any program
/// whose name has a space. Read from the last `)` instead, and this is that,
/// measured against a real process with a real awkward name.
///
/// **The process is a busy one deliberately.** A parked one would prove
/// nothing: a wrong parse reads some other token, which is not `R` either, so
/// *not running* is what both a correct reader and a broken one answer. Only
/// `running == true` separates them.
///
/// *The first version of this test used `exec -a` to set the name, which is a
/// bashism — `/bin/sh` here is dash, the command never ran, and the test failed
/// against a reader that was correct all along.*
#[test]
fn a_program_whose_name_has_a_space_is_still_read_correctly() {
    // A copy of a real shell under a name with a space and parentheses in it,
    // which is what the kernel records and what the parse has to survive.
    //
    // **The name carries this process's own number**, so two runs cannot
    // collide over one path. A fixed name failed with `ExecutableFileBusy` on
    // a second run: a copy still held open by the run before it cannot be
    // written over, and the test that was meant to be about parsing a name
    // failed about a file instead.
    let awkward = std::env::temp_dir().join(format!("a dash ({}) ", std::process::id()));
    std::fs::copy("/bin/dash", &awkward).expect("a shell can be copied to an awkward name");

    let mut odd = Command::new(&awkward)
        .args(["-c", "while :; do :; done"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the oddly named command starts");

    let evidence = settles_into(odd.id(), |evidence| evidence.running);
    let _ = odd.kill();
    let _ = odd.wait();
    let _ = std::fs::remove_file(&awkward);

    // The name really did have a space in it, or this tested nothing.
    assert!(
        awkward.to_string_lossy().contains(' '),
        "the test's own name has no space in it"
    );
    assert!(
        evidence.is_some_and(|evidence| evidence.running),
        "a running process whose name holds a space did not read as running, so the state field \
         was found by counting from the left"
    );
}
