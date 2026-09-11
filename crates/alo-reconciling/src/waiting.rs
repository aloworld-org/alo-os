//! What a promise with nothing behind it says it is waiting on, and whether a
//! reader can follow it.
//!
//! Four v0.01 promises have no evidence at all. For those, *what is still owed*
//! is the only thing anybody inherits, and the sentence has one job beyond being
//! honest: **it has to say where the work is.** A promise recorded as missing and
//! nowhere to go is a promise the next reader re-derives from scratch — which is
//! the seven-times-over reading this crate exists to end, arriving from the other
//! end.
//!
//! So a wholly owed promise names a **decision** under `docs/decisions/` or a
//! **task** in a plan under `docs/autonomy/`, and both are followed. A pointer at
//! a decision nobody wrote, or at a task number a plan does not have, reads
//! exactly like an answer: the number looks like a fact. `alo-citing` makes that
//! argument about ADR citations across the whole repository; this is the same
//! argument about the one place a promise comes to rest.
//!
//! # What it judges, and what it deliberately does not
//!
//! The **pointer**, never the argument. Whether task 20 is really the increment
//! *copy, cut and paste* needs is a reader's judgement and nothing mechanical
//! reaches it. What is mechanical is whether task 20 is there at all.
//!
//! # Keep the plan with the number
//!
//! A task is cited as ``task 20 of `docs/autonomy/v0-01-delivery-plan.md` ``, with
//! the plan named beside the number. *Task 12 of the delivery plan* is not a
//! pointer this reads, and that is deliberate rather than a gap in the parser:
//! this repository drives three plans, their tasks are all numbered from one, and
//! a reader who cannot see which plan is meant has been handed a number that
//! matches something in every one of them. It is `alo-citing`'s rule about ADR
//! filenames, applied to the other kind of number a ledger carries.

/// Where the settled arguments live.
const THE_DECISIONS: &str = "docs/decisions/";

/// What a markdown document is called.
const A_DOCUMENT: &str = ".md";

/// How a citation opens a task.
const A_TASK: &str = "task ";

/// The same, at the start of a sentence.
const A_TASK_CAPITALISED: &str = "Task ";

/// What has to sit between a task's number and the plan it is in.
const OF: &str = "of ";

/// The one section of a plan that holds work, as `tools/kernel-loop` reads it.
const THE_TASKS: &str = "## Tasks";

/// How a plan heads one task.
const A_HEADING: &str = "### ";

/// Something a promise with no evidence says it is waiting on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Waiting {
    /// A decision under `docs/decisions/`, named by its file.
    ADecision(String),

    /// A numbered task in a plan, named with the plan beside the number.
    ATask {
        /// Which task.
        number: u32,
        /// The plan it is in, by its repository-relative path.
        plan: String,
    },
}

/// Why something a promise waits on cannot be followed.
///
/// Three cases rather than one sentence, because they are three different
/// repairs: a decision to write, a plan path to correct, and a task to add to a
/// plan that is otherwise right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NoSuchWait {
    /// No decision of that name is in this repository.
    #[error(
        "there is no such decision in this repository, so the promise waits on nothing anybody wrote"
    )]
    NoSuchDecision,

    /// No plan of that name is in this repository.
    #[error("there is no such plan in this repository, so the task number points into nothing")]
    NoSuchPlan,

    /// The plan is there, and has no task with that number.
    #[error(
        "that plan has no task with that number under `## Tasks`, so the work is not scheduled anywhere a loop or a reader would look"
    )]
    NoSuchTask,
}

impl Waiting {
    /// Everything a sentence says it is waiting on: the decisions it names,
    /// then the tasks.
    ///
    /// Decisions first and tasks after, rather than in the order they are
    /// written, because the two are read by different scans and an interleaved
    /// order would be an accident of the parser rather than a fact about the
    /// sentence.
    #[must_use]
    pub fn named_in(sentence: &str) -> Vec<Self> {
        let flat = sentence.split_whitespace().collect::<Vec<&str>>().join(" ");
        let mut found: Vec<Self> = decisions_in(&flat).map(Self::ADecision).collect();
        found.extend(tasks_in(&flat));
        found
    }

    /// How it is written, so a finding quotes the pointer rather than describing
    /// it.
    #[must_use]
    pub fn said(&self) -> String {
        match self {
            Self::ADecision(at) => at.clone(),
            Self::ATask { number, plan } => format!("task {number} of {plan}"),
        }
    }

    /// Whether a reader following it arrives somewhere.
    ///
    /// `reading` answers with the text of a file named by its repository-relative
    /// path, or [`None`] where there is no such file. Nothing here opens
    /// anything, for the reason the whole crate does not: every refusal below has
    /// to be showable against a fixture.
    ///
    /// # Errors
    ///
    /// [`NoSuchWait`], saying which of the three it is.
    pub fn whether_it_is_there(
        &self,
        reading: &dyn Fn(&str) -> Option<String>,
    ) -> Result<(), NoSuchWait> {
        match self {
            Self::ADecision(at) => reading(at).map(|_| ()).ok_or(NoSuchWait::NoSuchDecision),
            Self::ATask { number, plan } => {
                let written = reading(plan).ok_or(NoSuchWait::NoSuchPlan)?;
                if holds_task(&written, *number) {
                    Ok(())
                } else {
                    Err(NoSuchWait::NoSuchTask)
                }
            }
        }
    }
}

/// Every decision a flattened sentence names, in the order it names them.
fn decisions_in(flat: &str) -> impl Iterator<Item = String> + '_ {
    flat.split(|it: char| it.is_whitespace() || "`(),;:*".contains(it))
        .filter(|word| word.starts_with(THE_DECISIONS) && word.ends_with(A_DOCUMENT))
        .map(str::to_owned)
}

/// Every task a flattened sentence names, in the order it names them.
///
/// The two spellings are scanned separately and the results put back in the
/// order they appear, so a sentence that opens with *Task 20* and names another
/// further along does not report them backwards.
fn tasks_in(flat: &str) -> Vec<Waiting> {
    let mut found: Vec<(usize, Waiting)> = flat
        .match_indices(A_TASK)
        .chain(flat.match_indices(A_TASK_CAPITALISED))
        .filter_map(|(at, word)| a_task_at(&flat[at + word.len()..]).map(|waiting| (at, waiting)))
        .collect();
    found.sort_by_key(|(at, _)| *at);
    found.into_iter().map(|(_, waiting)| waiting).collect()
}

/// One task citation, read from just after the word `task`.
///
/// `20 of `docs/autonomy/v0-01-delivery-plan.md`` and nothing looser: a number
/// with no plan beside it is not a pointer, and neither is a plan mentioned in
/// the same paragraph as a number.
fn a_task_at(after: &str) -> Option<Waiting> {
    let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
    let number = digits.parse().ok()?;
    let rest = after.get(digits.len()..)?.trim_start();
    let rest = rest.strip_prefix(OF)?.trim_start();
    let plan = rest.strip_prefix('`')?.split('`').next()?;
    (plan.ends_with(A_DOCUMENT)).then(|| Waiting::ATask {
        number,
        plan: plan.to_owned(),
    })
}

/// Whether a plan has a task with that number under its `## Tasks`.
///
/// The section is what makes a heading a task, and not the number alone —
/// `tools/kernel-loop` learnt that by launching a worker at an audit section
/// called *Implemented and verified*, whose heading is numbered from one at the
/// same depth as the work. A pointer read any more loosely would land on it too.
fn holds_task(plan: &str, number: u32) -> bool {
    let wanted = format!("{A_HEADING}{number}.");
    let mut in_the_tasks = false;
    for line in plan.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            in_the_tasks = format!("## {}", heading.trim()) == THE_TASKS;
            continue;
        }
        if in_the_tasks && line.starts_with(&wanted) {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A plan in the shape both of this repository's are written in, with an
    /// audit section numbered from one above the work.
    const A_PLAN: &str = "\
# A plan

## Audit

### 1. Implemented and verified

### 2. Remaining gaps

## Tasks

### 1. Something finished

### 20. The increment this promise waits on

## Rules

### 3. Not a task either
";

    /// The repository those pointers are about.
    fn a_repository(named: &str) -> Option<String> {
        match named {
            "docs/autonomy/a-plan.md" => Some(A_PLAN.to_owned()),
            "docs/decisions/0007-the-cpu-is-the-default.md" => {
                Some("# A decision\n\n**Status:** accepted".to_owned())
            }
            _ => None,
        }
    }

    /// **Both kinds are read**, out of the sentence a ledger really writes —
    /// wrapped across lines, which is how markdown hands one over.
    #[test]
    fn a_sentence_says_what_it_waits_on_in_both_kinds() {
        let owed = "all of it. The argument is settled in\n\
                    `docs/decisions/0007-the-cpu-is-the-default.md`, and the\n\
                    increment is task 20 of `docs/autonomy/a-plan.md`.";
        assert_eq!(
            Waiting::named_in(owed),
            [
                Waiting::ADecision("docs/decisions/0007-the-cpu-is-the-default.md".to_owned()),
                Waiting::ATask {
                    number: 20,
                    plan: "docs/autonomy/a-plan.md".to_owned(),
                },
            ],
            "a pointer that wrapped onto the next line was lost, which is how \
             every sentence in the real ledger is written"
        );
    }

    /// **A number with no plan beside it is not a pointer**, because three plans
    /// in this repository number their tasks from one and a reader cannot tell
    /// which was meant.
    #[test]
    fn a_task_number_with_no_plan_is_not_something_to_follow() {
        assert_eq!(
            Waiting::named_in("it is task 12 of the delivery plan, which needs a machine"),
            Vec::new(),
            "a bare task number was accepted as somewhere a reader could go"
        );
        assert_eq!(
            Waiting::named_in("task 20 of the plan at `docs/autonomy/a-plan.md`"),
            Vec::new(),
            "a plan mentioned further along the sentence was attached to a \
             number it was not written beside"
        );
    }

    /// A pointer that lands, and the three that do not.
    #[test]
    fn a_pointer_nobody_can_follow_says_which_of_the_three_it_is() {
        let there = Waiting::ATask {
            number: 20,
            plan: "docs/autonomy/a-plan.md".to_owned(),
        };
        assert_eq!(there.whether_it_is_there(&a_repository), Ok(()));

        let unwritten = Waiting::ATask {
            number: 44,
            plan: "docs/autonomy/a-plan.md".to_owned(),
        };
        assert_eq!(
            unwritten.whether_it_is_there(&a_repository),
            Err(NoSuchWait::NoSuchTask),
            "a promise was sent to a task number the plan does not have"
        );

        let elsewhere = Waiting::ATask {
            number: 20,
            plan: "docs/autonomy/a-plan-nobody-wrote.md".to_owned(),
        };
        assert_eq!(
            elsewhere.whether_it_is_there(&a_repository),
            Err(NoSuchWait::NoSuchPlan)
        );

        // Assembled rather than spelled out: `alo-citing` reads this file too,
        // and a decision filename nobody wrote is a dead pointer wherever it is
        // written, including in a test about dead pointers.
        let nowhere = Waiting::ADecision(format!("{THE_DECISIONS}{}-a-decision.md", "0099"));
        assert_eq!(
            nowhere.whether_it_is_there(&a_repository),
            Err(NoSuchWait::NoSuchDecision)
        );
    }

    /// **An audit heading is not a task**, however it is numbered — the failure
    /// `tools/kernel-loop` really had, arriving here as a pointer that would
    /// land on *Implemented and verified*.
    #[test]
    fn a_numbered_heading_outside_the_tasks_is_not_a_task() {
        assert!(holds_task(A_PLAN, 20));
        assert!(
            !holds_task(A_PLAN, 2),
            "an audit section numbered two was read as task two"
        );
        assert!(
            !holds_task(A_PLAN, 3),
            "a heading after the tasks was read as a task"
        );
    }

    /// And the pointer reads back as it was written, because a finding quotes it
    /// rather than describing it.
    #[test]
    fn a_pointer_is_quoted_as_it_was_written() {
        let waiting = Waiting::named_in("task 20 of `docs/autonomy/a-plan.md`")
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(waiting.said(), "task 20 of docs/autonomy/a-plan.md");
    }
}
