//! Every gate run, and every gate accounted for — for a caller that has to say
//! what it did not run.
//!
//! [`crate::gates::all_of_them`] stops at the first gate that refuses, which is
//! right for a loop deciding whether to publish and wrong for continuous
//! integration. **Measured on GitHub's runner until 2026-11-07:** the tests gate
//! always refused there, on the one test `.github/the-tests-a-hosted-runner-cannot-run.txt`
//! declares, and the workflow accepted that refusal — so the six gates after it
//! (the supervisor's three, rustdoc, and the BPF target's two) were never run on
//! any pull request or any queue build, while the check stayed green. A broken
//! rustdoc link reached `main` that way in #1107 and was found by hand in #1114.
//!
//! So this runs [`crate::gates::judged`] for **each** gate whatever the one
//! before it said, and prints one line per gate in a shape the workflow reads:
//!
//! ```text
//! GATE defined | <name>      once per gate, before any runs
//! GATE passed | <name>       or
//! GATE refused | <name>      once per gate, as each finishes
//! ```
//!
//! The workflow then checks that every `defined` line has exactly one answer.
//! A gate that was defined and never answered fails the job, which is the
//! check that would have caught this — and the only one that can, because a
//! gate that does not run prints nothing at all.

use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use crate::gates::{self, Gate};
use crate::where_it_builds;

/// What one gate came to.
#[derive(Debug, PartialEq, Eq)]
pub enum Answer {
    /// It passed, and this is what it said of itself.
    Passed(String),
    /// It refused, and this is why.
    Refused(String),
}

/// The line that introduces each gate, before any of them runs.
pub const DEFINED: &str = "GATE defined | ";
/// The line a passing gate is answered with.
pub const PASSED: &str = "GATE passed | ";
/// The line a refusing gate is answered with.
pub const REFUSED: &str = "GATE refused | ";

/// Run `judge` on every gate in `every`, in order, **whatever the one before it
/// answered**, and hand each answer to `told` as soon as it exists.
///
/// The judging is a parameter so that the one property this file exists for —
/// a refusal does not stop the gates after it — is a test rather than a
/// sentence. Nothing here is skipped and nothing is retried; both are
/// [`gates::judged`]'s business.
pub fn each<'a>(
    every: &'a [Gate],
    mut judge: impl FnMut(&Gate) -> Result<String, String>,
    mut told: impl FnMut(&'a str, &Answer),
) -> Vec<(&'a str, Answer)> {
    let mut answers = Vec::with_capacity(every.len());
    for gate in every {
        let answer = match judge(gate) {
            Ok(said) => Answer::Passed(said),
            Err(why) => Answer::Refused(why),
        };
        told(gate.named, &answer);
        answers.push((gate.named, answer));
    }
    answers
}

/// The line that answers for one gate.
pub fn line(named: &str, answer: &Answer) -> String {
    match answer {
        Answer::Passed(_) => format!("{PASSED}{named}"),
        Answer::Refused(_) => format!("{REFUSED}{named}"),
    }
}

/// `alo-kernel-loop gates`: every gate, each one answered, and a failure if
/// any refused.
///
/// **It exists so the gate list keeps one definition** — see
/// `Asked::Gates` in `main.rs` — and so a workflow can tell a gate that passed
/// from a gate that was never reached, which the stop-at-the-first form cannot
/// tell it.
pub fn run(at: &Path) -> ExitCode {
    // **Where it builds, said once at the start, as `run` and `publish` say it.**
    // A hosted runner has a log and no journal, and the steps after this one
    // read the directory from this line rather than guessing it.
    println!("{}", where_it_builds::chosen(at).because);
    println!(
        "alo-kernel-loop: running all {} gates, each one whatever the one before it said. \
         Each is answered as it finishes.",
        gates::EVERY_GATE.len()
    );
    for gate in gates::EVERY_GATE {
        println!("{DEFINED}{}", gate.named);
    }
    if let Err(why) = gates::prepared(at, &[]) {
        eprintln!("alo-kernel-loop: no gate was run, because {why}");
        return ExitCode::FAILURE;
    }
    let began = Instant::now();
    let answers = each(
        gates::EVERY_GATE,
        |gate| gates::judged(gate, at),
        |named, answer| {
            println!("{}", line(named, answer));
            match answer {
                Answer::Passed(said) => println!("PASS {said}"),
                Answer::Refused(why) => println!("alo-kernel-loop: {why}"),
            }
        },
    );
    let refused: Vec<&str> = answers
        .iter()
        .filter(|(_, answer)| matches!(answer, Answer::Refused(_)))
        .map(|(named, _)| *named)
        .collect();
    println!(
        "alo-kernel-loop: {} of {} gates passed in {:?}.",
        answers.len() - refused.len(),
        gates::EVERY_GATE.len(),
        began.elapsed()
    );
    // **The count is checked, not trusted**, as it was when this stopped at
    // the first: an answer list shorter than the definition is a run that
    // skipped something, whatever else it says.
    if answers.len() != gates::EVERY_GATE.len() {
        eprintln!(
            "alo-kernel-loop: {} gates answered of the {} this repository lists. Nothing here \
             may be read as green.",
            answers.len(),
            gates::EVERY_GATE.len()
        );
        return ExitCode::FAILURE;
    }
    if refused.is_empty() {
        ExitCode::SUCCESS
    } else {
        eprintln!("alo-kernel-loop: refused: {}", refused.join("; "));
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THREE: &[Gate] = &[
        Gate {
            named: "first",
            program: "true",
            args: &[],
            within: ".",
        },
        Gate {
            named: "second",
            program: "true",
            args: &[],
            within: ".",
        },
        Gate {
            named: "third",
            program: "true",
            args: &[],
            within: ".",
        },
    ];

    /// **The fault this file exists for.** A refusal in the middle does not
    /// stop the gate after it, and every gate is answered once.
    #[test]
    fn a_refusal_does_not_stop_the_gates_after_it() {
        let mut asked = Vec::new();
        let answers = each(
            THREE,
            |gate| {
                asked.push(gate.named);
                if gate.named == "second" {
                    Err("refused".to_owned())
                } else {
                    Ok(gate.named.to_owned())
                }
            },
            |_, _| {},
        );
        assert_eq!(asked, ["first", "second", "third"]);
        assert_eq!(
            answers,
            [
                ("first", Answer::Passed("first".to_owned())),
                ("second", Answer::Refused("refused".to_owned())),
                ("third", Answer::Passed("third".to_owned())),
            ]
        );
    }

    /// And a refusal of the *first* gate, the case most like the one that hid
    /// six gates, still runs all the others.
    #[test]
    fn a_first_gate_refusing_still_runs_every_other() {
        let mut told = Vec::new();
        let answers = each(THREE, |_| Err("no".to_owned()), |named, _| told.push(named));
        assert_eq!(told, ["first", "second", "third"]);
        let every_one_refused = answers
            .iter()
            .all(|(_, answer)| matches!(answer, Answer::Refused(_)));
        assert!(every_one_refused, "{answers:?}");
    }

    /// The lines the workflow reads are the lines this prints.
    ///
    /// `.github/workflows/gate.yml` matches on these three prefixes. Changing
    /// one here without changing it there would make the workflow find no
    /// answers at all and fail every run — loud, but for the wrong reason — so
    /// the workflow's spelling is checked from here.
    #[test]
    fn the_workflow_reads_the_lines_this_prints() {
        let workflow = include_str!("../../../.github/workflows/gate.yml");
        for prefix in [DEFINED, PASSED, REFUSED] {
            assert!(
                workflow.contains(prefix.trim_end()),
                "gate.yml does not read `{prefix}`"
            );
        }
        assert_eq!(line("x", &Answer::Passed(String::new())), "GATE passed | x");
        assert_eq!(
            line("x", &Answer::Refused(String::new())),
            "GATE refused | x"
        );
    }

    /// A name with the separator in it would be read as two fields.
    #[test]
    fn no_gate_name_holds_the_separator_or_its_own_line_end() {
        for gate in gates::EVERY_GATE {
            assert!(
                !gate.named.contains('|') && !gate.named.contains('\n'),
                "`{}` cannot be read back out of a GATE line",
                gate.named
            );
            assert_eq!(
                gate.named,
                gate.named.trim(),
                "`{}` has edge spaces",
                gate.named
            );
        }
    }
}
