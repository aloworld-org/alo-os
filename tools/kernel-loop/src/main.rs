//! The kernel-enforcement workstream's supervisor: one completed task at a
//! time, gated, published, and written down.
//!
//! # What this does, and the thing it deliberately does not
//!
//! It **verifies and publishes**. It runs the gates the repository requires,
//! commits exactly the files a task names, integrates whatever arrived on
//! `main` while the work was being done, reruns the gates on the combined tree,
//! pushes, and records what happened.
//!
//! **It does not write code.** A supervisor that claimed to would be a
//! placeholder with a loop around it, and `CLAUDE.md` has a word for that. The
//! implementing is done by the contributor between iterations; what this
//! removes is the part that is mechanical, forgettable and dangerous to get
//! wrong — the pull, the rebase, the rerun, the bounded retry, the log.
//!
//! That is the same division `tools/dev-loop` has, and this is a second program
//! rather than a change to it: the compositor worker's supervisor must keep
//! running whatever happens here, and two workstreams sharing one supervisor
//! would be two workstreams sharing one lock.
//!
//! # It chooses the task; a person does the writing
//!
//! Each iteration reads `docs/autonomy/kernel-enforcement-plan.md`, takes the
//! **next task that is not done and whose dependencies are**, and says which
//! one it is. Then it waits for that task's work to appear as
//! `.kernel-loop/handoff.toml` — the files it touched, the commit it becomes,
//! the report it published — gates it, publishes it, and goes round for the
//! next one. A handoff naming a *different* task than the one selected is
//! refused, because a supervisor that took whatever it was given would be one
//! whose plan is decoration.
//!
//! **What passes is not what a worker says passed.** The gates are the
//! repository's state, which is the state of everything *except* the thing just
//! written — so a handoff also names the test behind each acceptance criterion,
//! and `crate::evidence` runs each one on its own and refuses a task whose
//! evidence is a test that was already green. A blocked or partial worker
//! leaves no handoff and publishes nothing.
//!
//! **The writing is a person's.** That step cannot be automated by this program
//! and pretending otherwise would put a placeholder at the centre of the thing
//! that publishes. What the loop removes is everything around it: choosing what
//! is next, the pull, the gates, the rebase, the re-gate, the bounded retry,
//! and the log.
//!
//! When `ALO_KERNEL_LOOP_WORKER` names a command, the loop **launches one** for
//! the task it chose rather than waiting for a person, and then does exactly
//! what it would have done anyway: looks at the working tree and the handoff,
//! gates them, and publishes only what passes. A worker's own account of what it
//! did is never read. Without that setting it launches nothing, because making
//! an agent run is not a supervisor's decision to take for whoever started it.
//!
//! So a run is: *select, work, gate, publish, select…* until the plan has no
//! executable task left, nobody produces the work for the one selected, or
//! somebody stops it. A task the plan marks **blocked** is stepped over, so one
//! question awaiting an answer does not hold up work that has none.
//!
//! # Parking, and picking one back up
//!
//! A task whose gates refuse it is committed to a local branch of its own and
//! the run carries on. `recover <branch>` is the other half of that: it puts
//! **exactly the files that task's handoff names** back on top of today's
//! `main` — whole where nobody else has touched them, by applying the task's own
//! diff where somebody has — and leaves its handoff waiting. It publishes
//! nothing, and it deletes no branch. See `crate::recovering` for what it
//! replaces, which was somebody remembering a command that reverted the whole
//! tree.
//!
//! # Stopping
//!
//! `stop` writes a file. The loop finishes what it is doing — it never abandons
//! a half-published task — and does not begin another. Nothing is discarded and
//! nothing is rolled back, because a supervisor that tidied up after itself
//! would be a supervisor that can throw work away.

mod evidence;
mod gates;
mod handoff;
mod journal;
mod keeping_ubuntu_up;
mod lock;
mod plan;
mod publishing;
mod recovering;
mod repository;
mod where_it_builds;
mod worker;

use std::path::Path;
use std::process::ExitCode;
use std::time::{Duration, Instant};

/// Where the loop keeps its lock, its log and the file that stops it.
const ITS_OWN: &str = ".kernel-loop";

/// What the loop was asked to do.
enum Asked {
    /// Run iterations until there is nothing to publish or somebody stops it.
    Run,

    /// Say what is happening, and what happened last.
    Status,

    /// Ask the running loop to finish and not begin again.
    Stop,

    /// Gate and publish the handoff that is waiting, choosing nothing and
    /// launching nothing.
    Publish,

    /// Run every gate and every piece of the waiting handoff's evidence, and
    /// publish nothing.
    Verify,

    /// Put a parked task's work back in the tree, on top of today's `main`.
    Recover(String),
}

impl Asked {
    /// What the arguments say, or [`None`] if they say nothing this understands.
    fn from(mut args: impl Iterator<Item = String>) -> Option<Self> {
        let _ = args.next();
        match args.next().as_deref() {
            Some("run") => Some(Self::Run),
            Some("status") => Some(Self::Status),
            Some("stop") => Some(Self::Stop),
            Some("publish") => Some(Self::Publish),
            Some("verify") => Some(Self::Verify),
            Some("recover") => args.next().map(Self::Recover),
            _ => None,
        }
    }
}

fn main() -> ExitCode {
    let Some(asked) = Asked::from(std::env::args()) else {
        eprintln!(
            "alo-kernel-loop — the kernel-enforcement workstream's supervisor\n\
             \n\
             usage:\n\
             \x20 alo-kernel-loop run      gate and publish ready tasks until told to stop\n\
             \x20 alo-kernel-loop status   what is happening, and what happened last\n\
             \x20 alo-kernel-loop stop     finish the current task and begin no other\n\
             \x20 alo-kernel-loop verify   run every gate and the waiting handoff's evidence\n\
             \x20 alo-kernel-loop publish  gate, commit, integrate and push the waiting handoff\n\
             \x20 alo-kernel-loop recover <branch>\n\
             \x20\x20\x20\x20\x20\x20\x20\x20\x20 put a parked task's work back in the tree, on \
             top of today's main\n"
        );
        return ExitCode::FAILURE;
    };

    let at = match std::env::current_dir() {
        Ok(here) => here,
        Err(why) => {
            eprintln!("alo-kernel-loop: it cannot tell where it is being run: {why}");
            return ExitCode::FAILURE;
        }
    };
    let ours = at.join(ITS_OWN);

    match asked {
        Asked::Run => run(&at, &ours),
        Asked::Publish => publish(&at, &ours),
        Asked::Verify => verify(&at, &ours),
        Asked::Recover(branch) => recover(&at, &ours, &branch),
        Asked::Status => {
            // **Whether anything is running, before what last happened.** A
            // journal's last line reads exactly the same whether the loop is
            // still working or was killed an hour ago mid-sentence, and
            // somebody asking `status` is usually asking the first question.
            match lock::what_is_running(&ours) {
                lock::Running::ALoop(pid) => {
                    println!("alo-kernel-loop: a loop is running here, process {pid}.");
                }
                lock::Running::Nothing => {
                    println!(
                        "alo-kernel-loop: no loop is running here. What follows is what happened \
                         last, not what is happening."
                    );
                }
                lock::Running::ALockNobodyHolds(gone) => {
                    println!(
                        "alo-kernel-loop: no loop is running here — process {gone} left a lock \
                         behind, so it was killed rather than asked to stop. `run` takes that \
                         over. What follows is what happened last."
                    );
                }
            }
            match journal::said(&ours) {
                Ok(said) => {
                    println!("{said}");
                    ExitCode::SUCCESS
                }
                Err(why) => {
                    eprintln!("alo-kernel-loop: {why}");
                    ExitCode::FAILURE
                }
            }
        }
        Asked::Stop => match journal::asked_to_stop(&ours) {
            Ok(()) => {
                println!(
                    "alo-kernel-loop: asked to stop. It finishes what it is doing and begins \
                     nothing else; nothing is discarded."
                );
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("alo-kernel-loop: it could not be asked to stop: {why}");
                ExitCode::FAILURE
            }
        },
    }
}

/// Iterations, until there is nothing to publish or somebody stops it.
fn run(at: &Path, ours: &Path) -> ExitCode {
    if let Err(why) = std::fs::create_dir_all(ours) {
        eprintln!("alo-kernel-loop: it has nowhere to keep its own files: {why}");
        return ExitCode::FAILURE;
    }

    // **One at a time, and the lock is the whole of that.** Two supervisors in
    // one checkout are two editors, which `CLAUDE.md` forbids outright: the
    // second would rebase over the first's half-made commit.
    let held = match lock::Held::taken(ours) {
        Ok(held) => held,
        Err(why) => {
            eprintln!("alo-kernel-loop: {why}");
            return ExitCode::FAILURE;
        }
    };
    // A restart after a kill is written down rather than passed over, because a
    // loop that took a lock from a process nobody stopped is a fact about the
    // last run that somebody reading this journal will want.
    if let Some(gone) = held.took_over_from() {
        journal::note(
            ours,
            &format!("took the lock over from process {gone}, which is gone"),
        );
    }

    // **Ubuntu is kept up for as long as this loop runs, and no longer.** The
    // gates run inside it and it stops itself when nothing is using it, taking
    // `/sys/fs/bpf` with it — which then fails the readiness check halfway
    // through a run for a reason that has nothing to do with the work. This
    // holds one process open there and kills it on the way out. It mounts
    // nothing, restarts nothing, and changes no shared service.
    let _ubuntu = keeping_ubuntu_up::Awake::started();

    // **Where it builds, said once and written down.** A build directory
    // nobody can find is one nobody cleans, and this run is about to put
    // several gigabytes in one.
    said_where_it_builds(at, ours);

    // A stop asked for before this run began is not this run's to obey — it
    // belonged to the loop that has already finished. Cleared here so that
    // `stop` always means *the loop that is running now*.
    journal::the_stop_is_cleared(ours);
    journal::note(ours, "the loop began");

    // **Tasks this run has failed at, stepped over rather than retried forever.**
    // A supervisor meant to be left alone for days cannot end its run because
    // one task did not come off: nobody is waiting to be told, and every task it
    // could have finished would sit untouched until somebody noticed. What it
    // must not do instead is discard the work or pretend the task is done, and
    // it does neither.
    let mut given_up_on = std::collections::BTreeSet::new();

    // Workers that died before they could attempt anything, in a row. Reset by
    // any worker that actually ran, so a single hiccup does not accumulate
    // across an evening of good work.
    let mut cannot_run = 0_u32;

    let ending = loop {
        // Which task the iteration took up, so that a failure inside it can be
        // attributed to one and stepped over. Set by `one_iteration` through
        // this, because the error it returns is a sentence rather than a task.
        let mut holding = None;

        match one_iteration(at, ours, &given_up_on, &mut holding) {
            Ok(journal::Went::Published(sha, task)) => {
                cannot_run = 0;
                journal::note(ours, &format!("published {sha} — {task}"));
                if journal::was_asked_to_stop(ours) {
                    break "asked to stop after publishing".to_owned();
                }
            }
            Ok(journal::Went::PlanIsFinished) => {
                // **Not "the workstream is complete".** Every task being done is
                // a list being empty, and the plan's own last section says what
                // that is and is not: implementation, with hardware acceptance
                // still owed and the v0.5 items still unbuilt.
                break "the plan has no executable task left, which is a list being empty                        rather than a workstream being finished"
                    .to_owned();
            }
            Ok(journal::Went::NobodyWroteIt(number, task)) => {
                // **Not the end of the run.** A task one worker could not finish
                // is not a reason to leave every other task untouched. Stepped
                // over, loudly, and it stays unfinished in the plan.
                given_up_on.insert(number);
                journal::note(
                    ours,
                    &format!(
                        "giving up on `{task}` for this run — nobody handed over its work. \
                         Stepping over it: it is still unfinished in the plan, and anything \
                         depending on it still waits."
                    ),
                );
                if journal::was_asked_to_stop(ours) {
                    break "asked to stop".to_owned();
                }
            }
            Ok(journal::Went::TheWorkerCannotRun(why)) => {
                // One is a hiccup and is tried again. Two in a row is the
                // machine, and the tasks are left alone rather than consumed.
                cannot_run += 1;
                journal::note(
                    ours,
                    &format!("the worker did not start ({cannot_run} in a row): {why}"),
                );
                if worker::is_the_machine(cannot_run) {
                    break format!(
                        "the worker cannot run on this machine — {cannot_run} in a row died \
                         before they could attempt anything. No task was given up and none was \
                         marked: the plan is exactly as it was. What the last one said is in \
                         .kernel-loop/worker.log. Last: {why}"
                    );
                }
                if journal::was_asked_to_stop(ours) {
                    break "asked to stop".to_owned();
                }
            }
            Ok(journal::Went::Stopped) => break "asked to stop".to_owned(),
            Err(why) => {
                // A task that will not gate leaves its work in the tree, and
                // nothing may be started on top of it. So it is parked on a
                // branch of its own and pushed — nothing discarded, nothing
                // reset — and the run carries on.
                let Some(number) = holding else {
                    journal::note(ours, &format!("STOPPED: {why}"));
                    eprintln!("alo-kernel-loop: {why}");
                    drop(held);
                    return ExitCode::FAILURE;
                };
                given_up_on.insert(number);
                match repository::parked(at, number, &why) {
                    Ok(branch) => journal::note(
                        ours,
                        &format!(
                            "task {number} did not pass its gates, so its work is parked on the \
                             local branch `{branch}` and the run carries on. Nothing was \
                             discarded, and nothing was pushed — `main` is the only branch this \
                             publishes. The gates said: {why}"
                        ),
                    ),
                    Err(refused) => {
                        // The work could not be put anywhere safe, and starting
                        // another task would build on top of it. The one failure
                        // still worth stopping for.
                        journal::note(
                            ours,
                            &format!(
                                "STOPPED: task {number} did not gate and its work could not be \
                                 parked, so nothing may be started on top of it. The work is \
                                 still in the tree. Gates said: {why}. Parking said: {refused}"
                            ),
                        );
                        eprintln!("alo-kernel-loop: {refused}");
                        drop(held);
                        return ExitCode::FAILURE;
                    }
                }
                if journal::was_asked_to_stop(ours) {
                    break "asked to stop".to_owned();
                }
            }
        }
    };

    journal::note(ours, &format!("the loop ended: {ending}"));
    println!("alo-kernel-loop: {ending}.");
    drop(held);
    ExitCode::SUCCESS
}

/// Say where this checkout builds, and where the build directories from before
/// it are.
///
/// Once, at the start of anything that runs the gates. Two things belong in it
/// and neither is decoration: **where it builds**, because a directory holding
/// several gigabytes that nobody can name is one nobody ever cleans; and
/// **which older ones are still here**, because the answer to a full disk has
/// been to delete a build directory by hand and the person doing it deserves to
/// know which ones exist. Nothing here removes any of them: one of them may be
/// an afternoon of compilation belonging to a lane that is merely idle.
fn said_where_it_builds(at: &Path, ours: &Path) {
    // Through the journal rather than around it: `note` already says a line on
    // the way past, and a run's own log is where somebody looks a week later to
    // find out which directory filled up.
    journal::note(ours, &where_it_builds::chosen(at).because);

    let old = where_it_builds::the_old_ones(at);
    if old.is_empty() {
        return;
    }
    let said = format!(
        "build directories from before this one are still here, and nothing in this loop removes \
         them: {}. `du -sh` says how much each holds, and whether any of them goes is a person's \
         decision — one of them may belong to a lane that is merely idle.",
        old.join(", ")
    );
    journal::note(ours, &said);
}

/// How long the loop waits for a selected task's work before it stops.
///
/// Bounded on purpose. A supervisor that waited forever would sit on a lock
/// nobody could see the reason for; one that gave up in a minute would be
/// useless for work that takes an afternoon. When it runs out the loop stops
/// and says which task it was waiting for, which is a thing a person can act
/// on.
const WAITING_AT_MOST: Duration = Duration::from_secs(60 * 60);

/// How often it looks for the work while waiting, and how often it notices a
/// stop.
const LOOKING_EVERY: Duration = Duration::from_secs(10);

/// One task: choose it, wait for its work, check the checkout, gate, publish.
///
/// Every road out of this either published one task or explains why it did not.
/// Nothing here removes a file, resets a branch or discards a change: a
/// supervisor that tidied up would be one that can throw work away.
fn one_iteration(
    at: &Path,
    ours: &Path,
    given_up_on: &std::collections::BTreeSet<u32>,
    holding: &mut Option<u32>,
) -> Result<journal::Went, String> {
    if journal::was_asked_to_stop(ours) {
        return Ok(journal::Went::Stopped);
    }

    let Some(chosen) = plan::next_executable(at, given_up_on)? else {
        return Ok(journal::Went::PlanIsFinished);
    };
    // Whose failure it is, if this iteration fails. Set before anything can go
    // wrong, so an error carries a task rather than only a sentence.
    *holding = Some(chosen.number);
    journal::note(
        ours,
        &format!("next in the plan: {}. {}", chosen.number, chosen.named),
    );

    // **The worker writes it; the loop gates it.** Nothing the worker says is
    // read — what is inspected is the working tree and the handoff, through the
    // repository's own checks, exactly as for a change a person wrote.
    if worker::is_configured() && handoff::Handed::waiting(ours)?.is_none() {
        journal::note(ours, "launching one worker for it");
        match worker::ran_on(at, &chosen) {
            Ok(_took) => journal::note(ours, "the worker finished; inspecting what it left"),
            Err((why, took)) => {
                journal::note(ours, &format!("the worker did not finish: {why}"));
                // **A worker that died in seconds did not attempt the task.**
                // Stepping over it would consume the plan at the speed of the
                // failures — which is what happened the first night: three
                // workers exited at once, every remaining task was marked given
                // up inside ten seconds, and the run reported *no executable
                // task left*, the sentence it uses for a finished workstream.
                if worker::was_too_fast_to_have_tried(took) {
                    return Ok(journal::Went::TheWorkerCannotRun(why));
                }
                return Ok(journal::Went::NobodyWroteIt(chosen.number, chosen.named));
            }
        }
    }

    let Some(task) = waiting_for(ours, &chosen.named)? else {
        return Ok(journal::Went::NobodyWroteIt(chosen.number, chosen.named));
    };
    journal::note(ours, &format!("taking up: {}", task.task));

    repository::on_main_and_clean_but_for(at, &task.files)?;
    repository::pulled(at)?;

    let mut steps = publishing::OnThisMachine::publishing(at, ours, &task);
    let said = match publishing::gated_and_pushed(&mut steps) {
        Ok(sha) => return Ok(journal::Went::Published(sha, task.task.clone())),
        Err(said) => said,
    };

    // **A failed gate is the most actionable sentence in the run, and until now
    // nobody read it back to anybody.** Three workers in a row handed over code
    // that did not compile; the instruction not to was written, strengthened and
    // ignored each time. So the lever is not the sentence — it is this: one more
    // worker, on the same task, holding what the gates said, over work that is
    // still in the tree. Parking spends a finished afternoon to punish a missing
    // `cargo test`; this spends a few minutes to collect it.
    //
    // Once, never twice. A second failure is the work rather than an oversight,
    // and parking is what that is for.
    if !worker::is_configured() {
        return Err(said);
    }
    journal::note(
        ours,
        &format!(
            "the gates refused task {}; its work is still in the tree, so one worker is \
             launched on it again with what they said. This is the second and last attempt.",
            chosen.number
        ),
    );
    handoff::Handed::put_aside(ours)?;
    match worker::repairing(at, &chosen, &said) {
        Ok(_took) => journal::note(ours, "the second worker finished; inspecting what it left"),
        Err((why, _took)) => {
            // The repair never ran, so what stands is the original refusal —
            // that is what the work has to answer for, not a worker that died.
            journal::note(ours, &format!("the second worker did not finish: {why}"));
            return Err(said);
        }
    }

    let Some(again) = waiting_for(ours, &chosen.named)? else {
        return Err(said);
    };
    journal::note(ours, &format!("taking up again: {}", again.task));
    repository::on_main_and_clean_but_for(at, &again.files)?;

    let mut steps = publishing::OnThisMachine::publishing(at, ours, &again);
    publishing::gated_and_pushed(&mut steps)
        .map(|sha| journal::Went::Published(sha, again.task.clone()))
}

/// Gate and publish whatever handoff is waiting, choosing no task and launching
/// nothing.
///
/// **The manual recovery path, and the reason there is no other one.** A person
/// finishing a task by hand — or picking up after a loop that stopped — walks
/// through the same checks in the same order as the loop, because it is the same
/// code. What this replaces is a shell line with a `git push` on the end of it,
/// which is how a publication went out over failed gates on 2026-09-07: the two
/// commands were joined by a newline rather than by a check of the first one's
/// result.
///
/// It does not consult the plan. Task order is the loop's business, and somebody
/// recovering from a stopped run already knows which task they are holding.
fn publish(at: &Path, ours: &Path) -> ExitCode {
    // **Gating takes minutes, and WSL stops a distribution nothing is using.**
    // `run` has held one open since it gained the helper; this did not, and the
    // symptom is a publish that fails its readiness check because
    // `/sys/fs/bpf` went away between one command and the next. It happened
    // twice in a row before this line existed.
    let _ubuntu = keeping_ubuntu_up::Awake::started();
    said_where_it_builds(at, ours);
    let waiting = match handoff::Handed::waiting(ours) {
        Ok(Some(waiting)) => waiting,
        Ok(None) => {
            eprintln!(
                "alo-kernel-loop: there is no handoff waiting, and this publishes one. Write \
                 .kernel-loop/handoff.toml first — tools/kernel-loop/src/handoff.rs documents \
                 the format, including the acceptance evidence every task shows."
            );
            return ExitCode::FAILURE;
        }
        Err(why) => {
            eprintln!("alo-kernel-loop: {why}");
            return ExitCode::FAILURE;
        }
    };

    let published = repository::on_main_and_clean_but_for(at, &waiting.files)
        .and_then(|()| repository::pulled(at))
        .and_then(|()| {
            let mut steps = publishing::OnThisMachine::publishing(at, ours, &waiting);
            publishing::gated_and_pushed(&mut steps)
        });
    match published {
        Ok(sha) => {
            journal::note(ours, &format!("published {sha} — {}", waiting.task));
            println!("alo-kernel-loop: published {sha} — {}", waiting.task);
            ExitCode::SUCCESS
        }
        Err(why) => {
            journal::note(ours, &format!("NOTHING PUBLISHED: {why}"));
            eprintln!("alo-kernel-loop: nothing was published. {why}");
            ExitCode::FAILURE
        }
    }
}

/// Every gate and every piece of a waiting handoff's evidence, and nothing else.
///
/// For looking before publishing, and for a machine somebody has just changed
/// something on. It stages nothing, commits nothing and pushes nothing, so a
/// green answer here is *the checks pass* and never *the work is published*.
///
/// It refuses when no handoff is waiting rather than running the gates alone:
/// the gates are the state of the repository, and an answer that said `ok` while
/// no acceptance evidence had been looked at is the exact answer this program
/// exists not to give.
fn verify(at: &Path, ours: &Path) -> ExitCode {
    // As `publish`: the gates take minutes and the distribution they run in
    // stops when nothing is using it.
    let _ubuntu = keeping_ubuntu_up::Awake::started();
    said_where_it_builds(at, ours);
    let waiting = match handoff::Handed::waiting(ours) {
        Ok(waiting) => waiting,
        Err(why) => {
            eprintln!("alo-kernel-loop: {why}");
            return ExitCode::FAILURE;
        }
    };
    let Some(waiting) = waiting else {
        eprintln!(
            "alo-kernel-loop: there is no handoff waiting, so the gates could be run but no \
             acceptance evidence could be checked. Write the handoff first; \
             tools/kernel-loop/src/handoff.rs documents the format."
        );
        return ExitCode::FAILURE;
    };

    let mut steps = publishing::OnThisMachine::publishing(at, ours, &waiting);
    match publishing::Steps::check(&mut steps, "this task's tree") {
        Ok(()) => {
            println!(
                "alo-kernel-loop: every gate passed and the evidence for `{}` stood up. Nothing \
                 was staged, committed or pushed — `publish` does that.",
                waiting.task
            );
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("alo-kernel-loop: {why}");
            ExitCode::FAILURE
        }
    }
}

/// Put a parked task's work back in the tree, and stop there.
///
/// **The other half of parking, and until now it was a thing people
/// remembered.** What they remembered on 2026-09-11 was `git restore
/// --source=<branch> -- .`, which reverts the whole tree to that branch and so
/// undid two published tasks the first time and would have undone two more the
/// second. Both times something else caught it; neither time did anything here
/// stop it happening again.
///
/// This restores and never publishes. The gates stay where they are, `verify`
/// and `publish` are what run them, and a recovery that published would be a
/// road around the thing this program is. It deletes no branch either, so a
/// recovery that goes wrong is a recovery that can be done again.
fn recover(at: &Path, ours: &Path, branch: &str) -> ExitCode {
    let back = match recovering::recover(at, ours, branch) {
        Ok(back) => back,
        Err(why) => {
            eprintln!("alo-kernel-loop: nothing was restored. {why}");
            return ExitCode::FAILURE;
        }
    };

    println!("alo-kernel-loop: `{}` is back in the tree.", back.task);
    for (what, these) in [
        ("came back whole", &back.whole),
        ("merged by applying this task's own diff", &back.merged),
        (
            "changed on that branch and named by no task",
            &back.left_alone,
        ),
    ] {
        if !these.is_empty() {
            println!("  {what}: {}", these.join(", "));
        }
    }
    journal::note(
        ours,
        &format!("recovered `{}` from {}", back.task, back.branch),
    );

    if back.is_ready_to_gate() {
        println!(
            "Its handoff is waiting again. Nothing was gated and nothing was published — \
             `verify` looks, `publish` publishes. `{}` is still there.",
            back.branch
        );
        return ExitCode::SUCCESS;
    }
    eprintln!(
        "These were changed both by this task and on `main` since, and the two disagree line for \
         line: {}\nThe markers are in the files and no side was chosen. Resolve them by hand, \
         then `verify`. `{}` still has the task exactly as it was parked.",
        back.conflicted.join(", "),
        back.branch
    );
    ExitCode::FAILURE
}

/// The handoff for the task that was chosen, once somebody writes it.
///
/// A handoff for a **different** task is refused rather than published: the
/// plan's order is the plan's, and a supervisor that published whatever it was
/// handed would be one whose choosing meant nothing.
///
/// # Errors
/// A sentence when a handoff is there and names another task, or is missing
/// something a commit needs.
fn waiting_for(ours: &Path, chosen: &str) -> Result<Option<handoff::Handed>, String> {
    let until = Instant::now() + WAITING_AT_MOST;
    let mut said_so = false;
    loop {
        if journal::was_asked_to_stop(ours) {
            return Ok(None);
        }
        if let Some(handed) = handoff::Handed::waiting(ours)? {
            if handed.task != chosen {
                return Err(format!(
                    "the handoff is for `{}` and the plan's next task is `{chosen}`. Nothing was \
                     published: either finish the chosen task, or mark it done in the plan if it \
                     is.",
                    handed.task
                ));
            }
            return Ok(Some(handed));
        }
        if !said_so {
            journal::note(ours, "waiting for its work to be handed over");
            said_so = true;
        }
        if Instant::now() >= until {
            return Ok(None);
        }
        std::thread::sleep(LOOKING_EVERY);
    }
}

#[cfg(test)]
mod tests {
    /// **Everything that runs the gates says where it is building, first.**
    ///
    /// A build directory nobody can name is one nobody cleans, and the answer to
    /// a full disk on this machine has been to delete one by hand. So the three
    /// commands that compile anything — `run`, `publish` and `verify` — each say
    /// which directory they are about to put several gigabytes in, before they
    /// put any there.
    ///
    /// Held on the source because there is nothing else to hold it on: the
    /// saying is a line on a terminal and in a log, and a command that quietly
    /// stopped saying it would pass every other test in this crate.
    #[test]
    fn every_command_that_builds_says_where_it_builds_before_it_does() {
        let whole = include_str!("main.rs");
        for command in ["fn run(", "fn publish(", "fn verify("] {
            let body = whole
                .split(command)
                .nth(1)
                .and_then(|rest| rest.split("\n}\n").next())
                .unwrap_or_default();
            assert!(
                !body.is_empty(),
                "`{command}` is no longer in this file, so this test reads nothing"
            );
            assert!(
                body.contains("said_where_it_builds("),
                "`{command}` runs the gates without saying where it builds"
            );
        }
    }
}
