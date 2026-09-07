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
//! Without `ALO_KERNEL_LOOP_WORKER` naming a command, the loop does not launch
//! anything and says so: it waits for work to be handed over by a person, which
//! is the arrangement every task so far has used. Making an agent run by
//! default is not something a supervisor should decide for whoever started it.

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
pub fn ran_on(at: &Path, task: &Task) -> Result<(), String> {
    let named = std::env::var(THE_WORKER)
        .map_err(|_| format!("no worker is configured; set {THE_WORKER} to a command"))?;
    let named = named.trim().to_owned();

    let mut child = Command::new(&named)
        .arg("-p")
        .arg(asked_of_it(task))
        .current_dir(at)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|why| format!("the worker `{named}` could not be started: {why}"))?;

    let until = Instant::now() + AT_MOST;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {}
            Err(why) => return Err(format!("the worker could not be waited on: {why}")),
        }
        if Instant::now() >= until {
            // Killed rather than left: it holds this checkout, and a second
            // iteration beside it would be two editors on one working tree.
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "the worker `{named}` was still running after {} minutes and was stopped. \
                 Whatever it had written is still in the working tree; nothing was published \
                 and nothing was discarded.",
                AT_MOST.as_secs() / 60
            ));
        }
        std::thread::sleep(LOOKING_EVERY);
    }
}

/// What the worker is asked to do.
///
/// The task, where the rules are, and the one thing it has to produce for the
/// loop to be able to publish anything. Deliberately short on instructions
/// about *how*: the repository's own documents say that at length, and a prompt
/// that restated them would be a second copy to drift.
fn asked_of_it(task: &Task) -> String {
    format!(
        "You are the kernel-enforcement workstream's development worker in this checkout.\n\
         \n\
         Task {} — {}\n\
         \n\
         It is described under that heading in docs/autonomy/kernel-enforcement-plan.md.\n\
         Read CLAUDE.md, docs/autonomy/SHARED_MAIN.md, docs/autonomy/updates/README.md and\n\
         that plan before writing anything.\n\
         \n\
         Implement it completely, with tests that cover the refusal paths beside the\n\
         legitimate ones. Do not weaken a gate, add an unsafe exemption, widen a grant, or\n\
         mark unfinished work complete. Do not edit CHANGELOG.md, ROADMAP.md,\n\
         docs/autonomy/QUEUE.md or docs/autonomy/STATE.md. Publish your own report under\n\
         docs/autonomy/updates/ with a descriptive name.\n\
         \n\
         Do not commit and do not push: a supervisor gates and publishes this. When the\n\
         work is finished, write .kernel-loop/handoff.toml naming the task exactly as\n\
         above, the report path, a conventional commit subject, a body, every file you\n\
         touched, and an `evidence` block. tools/kernel-loop/src/handoff.rs documents the\n\
         format.\n\
         \n\
         The evidence is one line per acceptance criterion in the plan — the crate, the\n\
         test target and the test's full name — and each is run on its own before anything\n\
         is published. A test whose file is not among the files you list is refused: the\n\
         existing suite passing is the state of the repository, not proof of what you\n\
         wrote.\n\
         \n\
         If the task cannot be completed within the accepted decisions, write no handoff,\n\
         leave your work in the tree, and say what decision is needed. A partial task with\n\
         a handoff is worse than no handoff at all.",
        task.number, task.named
    )
}
