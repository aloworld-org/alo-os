//! Launching one development worker for one task, and never a second.
//!
//! The loop chooses the task and gates the result; this is the part in the
//! middle that writes the code, and it is somebody else's program. On this
//! machine that is `claude`; the other worker's supervisor launches `codex` the
//! same way, and neither loop knows anything about the other's.
//!
//! # What the loop does not do is trust it
//!
//! A worker's own account of what it did is not evidence. Nothing here reads
//! what it said, believes an exit code, or takes a summary as a result. The
//! loop looks at **the working tree and the handoff**, gates them with the
//! repository's own checks, and publishes only what passes — which is exactly
//! what it would do for a change a person wrote. A worker that claimed success
//! and changed nothing produces the same outcome as one that crashed: no
//! handoff, so nothing published.
//!
//! Nor is a green suite evidence. The gates are the state of everything except
//! the thing just written, so the handoff names the test behind each acceptance
//! criterion and `crate::evidence` runs each on its own, refusing any whose file
//! is not part of the change. A worker that was blocked, or that finished half
//! of it, cannot produce that — and half a task published as a whole one is the
//! failure this arrangement exists to make impossible.
//!
//! # One at a time, and bounded
//!
//! One worker per checkout, held by the same lock the loop is. It gets a
//! deadline, and a worker still running at the end of it is killed rather than
//! waited on — a supervisor that waited indefinitely for a program that had
//! stopped making progress would be a supervisor nobody could tell was stuck.
//!
//! # It is opt-in, and its absence is not a failure
//!
//! `ALO_KERNEL_LOOP_WORKER` names the **whole command**, and the task is
//! written to the worker's standard input — `claude
//! --dangerously-skip-permissions -p` on this machine. Without it the loop
//! launches nothing and says so: it waits for work to be handed over by a person, which
//! is the arrangement every task so far has used. Making an agent run by
//! default is not something a supervisor should decide for whoever started it.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::plan::Task;

/// What names the program that writes the code.
const THE_WORKER: &str = "ALO_KERNEL_LOOP_WORKER";

/// How long one worker may take before it is stopped.
const AT_MOST: Duration = Duration::from_secs(45 * 60);

/// How often it is checked on.
const LOOKING_EVERY: Duration = Duration::from_secs(5);

/// Whether a worker is configured at all.
#[must_use]
pub fn is_configured() -> bool {
    std::env::var(THE_WORKER).is_ok_and(|named| !named.trim().is_empty())
}

/// Run one worker on this task, and wait for it to finish or run out of time.
///
/// What comes back is only whether the worker *ran*. Whether it did anything
/// useful is the handoff's question and the gates', and neither of those asks
/// this function.
///
/// # Errors
/// A sentence when no worker is configured, when it could not be started, or
/// when it was still running at its deadline.
pub fn ran_on(at: &Path, task: &Task) -> Result<Duration, (String, Duration)> {
    // How long it took is part of the answer, not a detail: a worker that fails
    // in a second has not attempted the task, and the loop must not treat that
    // as a task nobody can finish. Everything that goes wrong *before* the
    // worker starts is `ZERO` and therefore counts as the machine, which is
    // what a missing command or an unusable log genuinely is.
    let nothing = Duration::ZERO;
    let named = std::env::var(THE_WORKER).map_err(|_| {
        (
            format!("no worker is configured; set {THE_WORKER} to a command"),
            nothing,
        )
    })?;
    let (program, args) = as_a_command(&named).ok_or_else(|| {
        (
            format!("{THE_WORKER} is set to nothing a program could be run from"),
            nothing,
        )
    })?;

    // **Kept, not discarded.** Both of these were `Stdio::null()`, and the cost
    // came due the first night: three workers across two loops exited with 1
    // and there was no way to learn why — the supervisor had thrown away the
    // only account of it. A worker's own words are not evidence of anything,
    // which is why the loop still reads the tree and the handoff rather than
    // this file; but *why a worker died* is not a claim about the work, it is
    // the thing an operator needs at two in the morning.
    let log = at.join(".kernel-loop").join("worker.log");
    let keeping = std::fs::File::create(&log).map_err(|why| {
        (
            format!(
                "the worker's log at {} could not be opened: {why}",
                log.display()
            ),
            nothing,
        )
    })?;
    let also = keeping.try_clone().map_err(|why| {
        (
            format!("the worker's log could not be shared with its error stream: {why}"),
            nothing,
        )
    })?;

    let mut child = Command::new(&program)
        .args(&args)
        .current_dir(at)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(keeping))
        .stderr(Stdio::from(also))
        .spawn()
        .map_err(|why| {
            (
                format!("the worker `{program}` could not be started: {why}"),
                nothing,
            )
        })?;

    // **The task goes in on stdin, not as an argument**, and it is not a style
    // choice. Since Rust 1.77 a `.cmd` or `.bat` — which is what an npm-shipped
    // agent is on Windows — refuses arguments it cannot quote safely, and a
    // prompt with newlines in it is exactly that; the failure reads `batch file
    // arguments are invalid` and looks like a broken setting. `tools/dev-loop`
    // feeds its worker the same way for the same reason. The pipe is closed
    // straight afterwards, because a worker waiting on a handle nobody will
    // write to again is a worker that runs until its deadline.
    match child.stdin.take() {
        Some(mut asking) => {
            if let Err(why) = asking.write_all(asked_of_it(task).as_bytes()) {
                drop(child.kill());
                drop(child.wait());
                return Err((
                    format!("the worker `{program}` could not be told what to do: {why}"),
                    nothing,
                ));
            }
        }
        None => {
            drop(child.kill());
            drop(child.wait());
            return Err((
                format!("the worker `{program}` has no stdin to be told anything on"),
                nothing,
            ));
        }
    }

    let began = Instant::now();
    let until = began + AT_MOST;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return whether_it_finished(&program, status.success(), status.code())
                    .map(|()| began.elapsed())
                    .map_err(|why| (why, began.elapsed()));
            }
            Ok(None) => {}
            Err(why) => {
                return Err((
                    format!("the worker could not be waited on: {why}"),
                    began.elapsed(),
                ));
            }
        }
        if Instant::now() >= until {
            // Killed rather than left: it holds this checkout, and a second
            // iteration beside it would be two editors on one working tree.
            let _ = child.kill();
            let _ = child.wait();
            return Err((
                format!(
                    "the worker `{program}` was still running after {} minutes and was stopped. \
                     Whatever it had written is still in the working tree; nothing was published \
                     and nothing was discarded.",
                    AT_MOST.as_secs() / 60
                ),
                began.elapsed(),
            ));
        }
        std::thread::sleep(LOOKING_EVERY);
    }
}

/// The setting, read as a program and the arguments that go before the prompt.
///
/// **Split on whitespace, because the command that runs an agent
/// non-interactively is never one word.** The other supervisor in this
/// repository runs `codex exec --sandbox danger-full-access`; this one is set to
/// something like `claude --dangerously-skip-permissions -p`. A setting that
/// could only name a bare program would force a wrapper script between the loop
/// and what it really runs, which is one more file to drift and one more place
/// for a flag nobody reviewed.
///
/// The task itself is written to the worker's standard input rather than
/// appended here, so whatever makes the program read one from there belongs in
/// the setting.
///
/// [`None`] for a setting that is empty or only spaces.
fn as_a_command(named: &str) -> Option<(String, Vec<String>)> {
    let mut words = named.split_whitespace().map(str::to_owned);
    let program = words.next()?;
    Some((program, words.collect()))
}

/// What a worker's exit means for the task it was given.
///
/// **A worker that stopped unsuccessfully did not finish the task**, whatever it
/// left behind. The loop still never reads what a worker *said*; this reads the
/// one thing an operating system is willing to state about a process, and takes
/// it the safe way round. A worker that was blocked, killed, or that gave up
/// part way through reports it here, and a task with a handoff beside a failed
/// exit is a contradiction that must not be resolved in favour of publishing.
///
/// The cost of being wrong in this direction is a task nobody published, which
/// somebody notices. The cost of the other is half a task on `main`.
///
/// # Errors
/// A sentence naming the worker and what it exited with.
fn whether_it_finished(named: &str, success: bool, code: Option<i32>) -> Result<(), String> {
    if success {
        return Ok(());
    }
    let said = code.map_or_else(
        || "it was stopped by a signal".to_owned(),
        |code| format!("it exited with {code}"),
    );
    Err(format!(
        "the worker `{named}` did not finish successfully — {said}. Whatever it wrote is still          in the working tree; nothing was published and nothing was discarded. A task it could          not complete is a task nobody has done, even if it left a handoff behind. What it          said is in .kernel-loop/worker.log."
    ))
}

/// How quickly a worker has to die for its death to be about the worker rather
/// than about the task.
///
/// A worker that spends ten minutes and fails has attempted the task. One that
/// exits inside this has not started it: the command is missing, the account is
/// out of quota, the machine is refusing to run it. The two are indistinguishable
/// from an exit code and are completely different situations.
const TOO_FAST_TO_HAVE_TRIED: Duration = Duration::from_secs(60);

/// How many workers may die that fast in a row before the run stops.
///
/// **The night this was written, three did.** Each was stepped over as a task
/// nobody could finish, the loop consumed its whole plan in ten seconds, and
/// reported *the plan has no executable task left* — the sentence it uses for a
/// finished workstream. Every task was still there and nothing had been tried.
///
/// One fast failure is a hiccup and is stepped over. Two in a row is the
/// environment, and stepping over tasks is the wrong response to it: they are
/// left alone, unattempted, for whoever comes back.
const BEFORE_IT_IS_THE_MACHINE: u32 = 2;

/// Whether a worker died so fast, so often, that the machine is what is wrong.
#[must_use]
pub const fn is_the_machine(consecutive_fast_failures: u32) -> bool {
    consecutive_fast_failures >= BEFORE_IT_IS_THE_MACHINE
}

/// Whether this attempt was too quick to have been an attempt.
#[must_use]
pub fn was_too_fast_to_have_tried(took: Duration) -> bool {
    took < TOO_FAST_TO_HAVE_TRIED
}

/// What the worker is asked to do.
///
/// The task, where the rules are, and the one thing it has to produce for the
/// loop to be able to publish anything. Deliberately short on instructions
/// about *how*: the repository's own documents say that at length, and a prompt
/// that restated them would be a second copy to drift.
///
/// Written to the worker's standard input by [`ran_on`].
fn asked_of_it(task: &Task) -> String {
    // **The plan this run is driving, not a constant.** The prompt named the
    // kernel-enforcement plan outright, so a worker on any other plan was sent
    // to a file its task is not in — it would have found no such heading and
    // built the task out of its title.
    let plan = crate::plan::the_plan().unwrap_or_else(|_| "the workstream's plan".to_owned());

    format!(
        "You are a development worker in this checkout, building alo OS: a sovereign,\n\
         AI-native operating system meant to be the best in the world and to run on the\n\
         ordinary machines people already own. Build at that standard.\n\
         \n\
         Task {} — {}\n\
         \n\
         It is described under that heading in {plan}.\n\
         Read CLAUDE.md, docs/autonomy/SHARED_MAIN.md, docs/autonomy/updates/README.md and\n\
         that plan before writing anything.\n\
         \n\
         Implement it completely, with tests that cover the refusal paths beside the\n\
         legitimate ones. Do not edit CHANGELOG.md, ROADMAP.md, docs/autonomy/QUEUE.md or\n\
         docs/autonomy/STATE.md. Publish your own report under docs/autonomy/updates/ with\n\
         a descriptive name.\n\
         \n\
         Do not commit and do not push: a supervisor gates and publishes this. When the\n\
         work is finished, write .kernel-loop/handoff.toml naming the task exactly as\n\
         above, the report path, a conventional commit subject, a body, every file you\n\
         touched, and an `evidence` block. tools/kernel-loop/src/handoff.rs documents the\n\
         format.\n\
         \n\
         IN THE SAME CHANGE, mark this task `**Done, <date>**` in the plan named above —\n\
         with the plan file among the files you list — and write the next task there if\n\
         the plan names none after it. The loop selects from the published plan: a task\n\
         finished and not marked is a task it selects again, and the next worker is sent\n\
         at work that is already done.\n\
         \n\
         The evidence is one line per acceptance criterion in the plan — the workspace\n\
         (`.` for the product's), the crate, the test target and the test's full name —\n\
         and each is run on its own before anything is published. A test whose file is not\n\
         among the files you list is refused: the existing suite passing is the state of\n\
         the repository, not proof of what you wrote.\n\
         \n\
         DECIDE RATHER THAN STOP. Where the task leaves something open — a name, a shape,\n\
         which of two reasonable designs — choose it the way a senior engineer would, and\n\
         write what you chose and why in your report. Nobody is waiting to answer you, and\n\
         a task handed back unstarted over a question you could have answered yourself is\n\
         a day of the release lost.\n\
         \n\
         Three things you may never decide, and these are absolute:\n\
         - Never weaken a gate, take an exemption, widen a grant, or add an unsafe block.\n\
         - Never claim unfinished work is finished, and never write a handoff for a\n\
           partial task. A partial task with a handoff is worse than no handoff at all.\n\
         - Never quietly narrow a promise in docs/features.md, and never contradict an\n\
           accepted ADR in docs/decisions/.\n\
         \n\
         If the only way forward runs through one of those three, then the decision itself\n\
         is the work: write the ADR under docs/decisions/ with the options, a\n\
         recommendation and the consequences, hand that over as this task, and say in the\n\
         report that the code waits on it. That is a finished piece of work rather than a\n\
         failure, and it is what the next worker needs in order to build.",
        task.number, task.named
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A worker that exited unsuccessfully publishes nothing**, and the
    /// sentence says the work is still there.
    ///
    /// This is the blocked-or-partial case. A worker that ran out of road,
    /// was killed, or decided it could not proceed leaves the tree as it is —
    /// and a handoff it may have written beside a failed exit does not turn
    /// half a task into a whole one.
    #[test]
    fn a_worker_that_did_not_finish_is_not_a_completed_task() {
        let stopped = whether_it_finished("claude", false, Some(1));
        assert!(stopped.is_err_and(|why| {
            why.contains("did not finish successfully")
                && why.contains("exited with 1")
                && why.contains("nothing was discarded")
        }));

        let killed = whether_it_finished("claude", false, None);
        assert!(killed.is_err_and(|why| why.contains("stopped by a signal")));
    }

    /// And a worker that finished is simply that — which is not the same claim
    /// as the task being done. The gates and the evidence decide that, and
    /// neither of them asks this function anything.
    #[test]
    fn a_worker_that_finished_is_not_by_itself_evidence_of_anything() {
        assert_eq!(whether_it_finished("claude", true, Some(0)), Ok(()));
    }

    /// **The task a worker is given names the task and demands a handoff.**
    ///
    /// Short, because everything about *how* is in documents the worker is
    /// told to read and a prompt that restated them would be a second copy to
    /// drift. But these four things cannot be left to drift: which task it is,
    /// that it publishes nothing itself, that it produces a handoff, and that
    /// the handoff carries evidence. A prompt missing any of them produces work
    /// the loop then refuses, after forty-five minutes.
    #[test]
    fn the_task_a_worker_is_given_names_it_and_demands_a_handoff() {
        let asked = asked_of_it(&Task {
            number: 5,
            named: "Documenting the filesystem mutations that remain unwatched".to_owned(),
            done: false,
            blocked: false,
            after: Vec::new(),
        });

        assert!(asked.contains("Task 5"), "{asked}");
        assert!(
            asked.contains("Documenting the filesystem mutations that remain unwatched"),
            "{asked}"
        );
        assert!(asked.contains("Do not commit and do not push"), "{asked}");
        assert!(asked.contains(".kernel-loop/handoff.toml"), "{asked}");
        assert!(asked.contains("evidence"), "{asked}");
        assert!(
            asked.contains("never write a handoff for a"),
            "a worker not told this hands over half a task: {asked}"
        );
        // The loop selects from the published plan, so a finished task that is
        // not marked done there is selected again and the next worker is sent
        // at work that is already done. The first overlay worker did exactly
        // that: it finished, its handoff never touched the plan, and the loop
        // launched a second worker at the published task within a minute.
        assert!(
            asked.contains("mark this task `**Done, <date>**`"),
            "a worker not told to mark the plan leaves the task selectable forever: {asked}"
        );
    }

    /// **The worker is told to decide rather than hand the task back**, and told
    /// the three things it may never decide.
    ///
    /// The prompt used to end *write no handoff, leave your work in the tree,
    /// and say what decision is needed*. On an unattended run there is nobody to
    /// say it to: the task came back unstarted, the loop ended, and a day was
    /// lost to a question the worker could have answered. Deciding is the
    /// instruction now — with the three exceptions absolute, because a worker
    /// that decided *those* for itself is the failure this whole supervisor
    /// exists to prevent.
    #[test]
    fn a_worker_is_told_to_decide_and_what_it_may_never_decide() {
        let asked = asked_of_it(&Task {
            number: 2,
            named: "The agent overlay: one key, from anywhere".to_owned(),
            done: false,
            blocked: false,
            after: Vec::new(),
        });

        assert!(asked.contains("DECIDE RATHER THAN STOP"), "{asked}");
        for absolute in [
            "Never weaken a gate",
            "Never claim unfinished work is finished",
            "Never quietly narrow a promise",
        ] {
            assert!(
                asked.contains(absolute),
                "a worker told to decide and not told `{absolute}` is worse than one told to \
                 stop: {asked}"
            );
        }
        // And the way out that is still work rather than a refusal.
        assert!(asked.contains("docs/decisions/"), "{asked}");
    }

    /// **The worker is sent to the plan this run is driving**, not to a
    /// hard-coded one.
    ///
    /// The prompt named `docs/autonomy/kernel-enforcement-plan.md` outright. A
    /// worker on any other plan was therefore sent to a file its task is not in
    /// — it would have found no such heading and built the task out of its
    /// title, which is the one way to get forty-five minutes of confident work
    /// on the wrong thing.
    #[test]
    fn a_worker_is_sent_to_the_plan_this_run_is_driving() {
        let asked = asked_of_it(&Task {
            number: 1,
            named: "Anything".to_owned(),
            done: false,
            blocked: false,
            after: Vec::new(),
        });
        let plan = crate::plan::the_plan().unwrap();
        assert!(
            asked.contains(&plan),
            "the worker was not told where its task is described: {asked}"
        );
    }

    /// **A worker that died in seconds did not attempt the task**, and two in a
    /// row is the machine rather than the plan.
    ///
    /// The first night this loop ran unattended, three workers exited within
    /// seconds of each other. Each was stepped over as *a task nobody could
    /// finish*, the plan was consumed inside ten seconds, and the run reported
    /// *no executable task left* — the sentence it uses for a **finished
    /// workstream**. Every task was still there and not one had been attempted.
    ///
    /// So the two are told apart by how long it took, and the response differs:
    /// a task that was tried and failed is stepped over; a worker that cannot
    /// run stops the run with the plan untouched.
    #[test]
    fn a_worker_that_died_in_seconds_is_the_machine_and_not_the_task() {
        assert!(was_too_fast_to_have_tried(Duration::from_secs(2)));
        assert!(was_too_fast_to_have_tried(Duration::ZERO));
        // Ten minutes of work that failed is an attempt, and belongs to the
        // task rather than to the machine.
        assert!(!was_too_fast_to_have_tried(Duration::from_secs(600)));

        // One is a hiccup; the run tries the next task. Two in a row is not.
        assert!(!is_the_machine(1));
        assert!(is_the_machine(2));
        assert!(is_the_machine(7));
    }

    /// **The setting carries the flags, not just the program**, because no
    /// agent runs non-interactively without them and a wrapper script would be
    /// one more file between the loop and what it runs.
    #[test]
    fn the_worker_setting_is_a_whole_command() {
        assert_eq!(
            as_a_command("  claude --dangerously-skip-permissions -p  "),
            Some((
                "claude".to_owned(),
                vec!["--dangerously-skip-permissions".to_owned(), "-p".to_owned(),],
            ))
        );
        assert_eq!(
            as_a_command("claude"),
            Some(("claude".to_owned(), Vec::new()))
        );
        assert_eq!(as_a_command("   "), None);
    }
}
