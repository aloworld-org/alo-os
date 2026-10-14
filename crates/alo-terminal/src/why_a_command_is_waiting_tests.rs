//! **Every case of the table, and the two the measurement overturned.**
//!
//! The reading is arithmetic, so it is held to its cases exhaustively rather
//! than sampled. What is worth more than the sweep are the two tests named for
//! what was measured: that a terminal and a socket are told apart although the
//! kernel calls them the same thing, and that *still computing* comes from
//! positive evidence rather than from having found nothing.

use super::*;

/// Evidence for a parked process with this kernel function and this input.
fn parked(kernel_says: Option<&str>, reads_from: WhatItReadsFrom) -> TheEvidence {
    TheEvidence {
        running: false,
        kernel_says: kernel_says.map(ToOwned::to_owned),
        reads_from,
    }
}

/// **A terminal and a socket are told apart**, although the kernel parks both
/// in the same function.
///
/// The measurement this module exists because of: on 7.0.0-31-generic a
/// process reading a terminal and a process reading a socket are **both**
/// `wait_woken`. A table keyed on the kernel function alone answers one of
/// these wrongly every time, and both are answers a person acts on — pressing
/// a key, or going to look at a network.
#[test]
fn a_terminal_and_a_socket_are_told_apart_under_the_same_kernel_function() {
    let the_same_function = Some("wait_woken");
    assert_eq!(
        why_it_is_waiting(&parked(the_same_function, WhatItReadsFrom::ATerminal)),
        WhyItIsWaiting::ForTheKeyboard
    );
    assert_eq!(
        why_it_is_waiting(&parked(the_same_function, WhatItReadsFrom::ASocket)),
        WhyItIsWaiting::ForTheNetwork
    );
}

/// **Still computing is positive evidence**, not the absence of a reason.
///
/// A running process answers it whatever else is known — including when
/// nothing else is known at all, which is the case that separates *it is
/// working* from *we found nothing*. A person told the second will kill
/// something; a person told the first will wait.
#[test]
fn still_computing_comes_from_evidence_and_not_from_an_empty_answer() {
    let running_and_nothing_else = TheEvidence {
        running: true,
        kernel_says: None,
        reads_from: WhatItReadsFrom::SomethingElse,
    };
    assert_eq!(
        why_it_is_waiting(&running_and_nothing_else),
        WhyItIsWaiting::StillComputing
    );

    // And the same evidence without the one positive fact is *not* the same
    // answer, which is what makes the fact load-bearing rather than decorative.
    let parked_and_nothing_else = TheEvidence {
        running: false,
        ..running_and_nothing_else
    };
    assert_eq!(
        why_it_is_waiting(&parked_and_nothing_else),
        WhyItIsWaiting::NobodyCanTell
    );
}

/// **Running beats everything**, including a terminal for input.
///
/// A command that is working happens to have the terminal as its input almost
/// always — it was typed there. Answering *waiting for the keyboard* for a
/// command that is busy would be wrong for nearly every command.
#[test]
fn a_running_command_is_not_waiting_for_the_keyboard_it_happens_to_have() {
    let busy_at_a_terminal = TheEvidence {
        running: true,
        kernel_says: None,
        reads_from: WhatItReadsFrom::ATerminal,
    };
    assert_eq!(
        why_it_is_waiting(&busy_at_a_terminal),
        WhyItIsWaiting::StillComputing
    );
}

/// **A lock is answered as a lock, even at a terminal.**
///
/// The order of the questions, asserted: a command blocked on a lock usually
/// still has the terminal for its input, and *waiting for the keyboard* would
/// send a person to press a key that changes nothing.
#[test]
fn a_lock_is_answered_before_the_terminal_it_also_has() {
    assert_eq!(
        why_it_is_waiting(&parked(
            Some("locks_lock_inode_wait"),
            WhatItReadsFrom::ATerminal
        )),
        WhyItIsWaiting::ForAnotherProcess,
        "a command waiting on a lock was reported as waiting for a keypress"
    );
}

/// **Each kernel function that names what it waits for is read.**
#[test]
fn every_kernel_function_in_the_table_answers_another_process() {
    for known in WAITING_ON_ANOTHER_PROCESS {
        assert_eq!(
            why_it_is_waiting(&parked(Some(known), WhatItReadsFrom::SomethingElse)),
            WhyItIsWaiting::ForAnotherProcess,
            "{known} is in the table and was not read"
        );
    }
}

/// **A kernel function nobody has seen does not become an answer**, whatever
/// the command reads from.
///
/// The fifth state falling out of the mechanism rather than being added for
/// honesty's sake: a symbol is a kernel's own name, any table here is finite,
/// and a kernel that renames one must produce *nobody can tell* rather than the
/// nearest entry.
///
/// Swept across every input as well, because the fault this guards against is
/// precisely the table reaching for the file when the kernel has already said
/// something it does not understand.
#[test]
fn a_kernel_function_nobody_has_seen_answers_that_nobody_can_tell() {
    for unseen in [
        "hrtimer_nanosleep",
        "futex_wait_queue_me",
        "something_a_later_kernel_calls_it",
    ] {
        for reads_from in [
            WhatItReadsFrom::ATerminal,
            WhatItReadsFrom::ASocket,
            WhatItReadsFrom::APipe,
            WhatItReadsFrom::AFile,
            WhatItReadsFrom::SomethingElse,
        ] {
            assert_eq!(
                why_it_is_waiting(&parked(Some(unseen), reads_from)),
                WhyItIsWaiting::NobodyCanTell,
                "{unseen:?} with {reads_from:?} was read as something it is not"
            );
        }
    }
}

/// **A command waiting on a clock is not reported as waiting on anything
/// else**, although its input happens to be a pipe.
///
/// Measured: `sleep 30` is parked in `hrtimer_nanosleep` with a pipe for its
/// input. The first version of this table fell through to the file whenever the
/// kernel function was unrecognised, and answered *waiting for another
/// program* — a confident wrong answer, which a person acts on by going to look
/// for a program that is not there. A clock is none of the five, so the fifth
/// is the true one.
///
/// Both halves are asserted, because the answer was wrong in a way that is not
/// *still computing* and a test for only that would still pass.
#[test]
fn a_command_waiting_on_a_clock_says_nobody_can_tell() {
    let a_sleep = parked(Some("hrtimer_nanosleep"), WhatItReadsFrom::APipe);
    assert_eq!(
        why_it_is_waiting(&a_sleep),
        WhyItIsWaiting::NobodyCanTell,
        "a command waiting on a clock was explained by a file it is not reading"
    );

    // And the same holds at a terminal, which is the case that would send
    // somebody to press a key.
    assert_eq!(
        why_it_is_waiting(&parked(
            Some("hrtimer_nanosleep"),
            WhatItReadsFrom::ATerminal
        )),
        WhyItIsWaiting::NobodyCanTell
    );
}

/// **With no kernel function at all, the file is the evidence.**
///
/// A hardened kernel refuses `wchan`, and a terminal that then said *nobody can
/// tell* about everything would be useless on exactly the machines most likely
/// to run it. The file is still readable and still says what it says.
#[test]
fn with_no_kernel_function_the_file_is_still_evidence() {
    assert_eq!(
        why_it_is_waiting(&parked(None, WhatItReadsFrom::ATerminal)),
        WhyItIsWaiting::ForTheKeyboard
    );
    assert_eq!(
        why_it_is_waiting(&parked(None, WhatItReadsFrom::ASocket)),
        WhyItIsWaiting::ForTheNetwork
    );
}

/// **Every answer has its own sentence**, and no two share a key.
#[test]
fn every_answer_has_a_sentence_of_its_own() {
    let keys: Vec<&str> = WhyItIsWaiting::ALL
        .iter()
        .map(|why| why.the_word().named())
        .collect();
    assert_eq!(keys.len(), 5);
    for (one, key) in keys.iter().enumerate() {
        for (other, against) in keys.iter().enumerate() {
            assert!(
                one == other || key != against,
                "answers {one} and {other} share the key {key}"
            );
        }
    }
}

/// **Every combination of evidence answers something**, and never panics.
///
/// A sweep rather than a sample: the table is small enough to walk entirely,
/// and a reading that could panic on a combination is a terminal that falls
/// over when somebody asks a question.
#[test]
fn every_combination_of_evidence_has_an_answer() {
    let inputs = [
        WhatItReadsFrom::ATerminal,
        WhatItReadsFrom::ASocket,
        WhatItReadsFrom::APipe,
        WhatItReadsFrom::AFile,
        WhatItReadsFrom::SomethingElse,
    ];
    let functions = [
        None,
        Some("wait_woken"),
        Some("do_wait"),
        Some("locks_lock_inode_wait"),
        Some("anon_pipe_read"),
        Some("hrtimer_nanosleep"),
    ];
    let mut seen = Vec::new();
    for running in [true, false] {
        for reads_from in inputs {
            for kernel_says in functions {
                let answer = why_it_is_waiting(&TheEvidence {
                    running,
                    kernel_says: kernel_says.map(ToOwned::to_owned),
                    reads_from,
                });
                if !seen.contains(&answer) {
                    seen.push(answer);
                }
            }
        }
    }
    assert_eq!(
        seen.len(),
        5,
        "the sweep produced {} of the five answers, so one is unreachable: {seen:?}",
        seen.len()
    );
}
