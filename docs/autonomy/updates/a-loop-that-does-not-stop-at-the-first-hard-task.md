# A loop that does not stop at the first hard task

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **a run survives a task it cannot finish.** Nothing is discarded and no
  gate is weakened.

## Three things would have stopped an unattended run in its first hour

The loop was about to be left alone for five days. Started against the v0.01
plan, it would have ended almost immediately, three separate ways.

**It sent every worker to the wrong plan.** The prompt named
`docs/autonomy/kernel-enforcement-plan.md` outright, so a worker on the v0.01
plan was sent to a file its task is not in. It would have found no such heading
and built the task out of its **title** — forty-five minutes of confident work on
something nobody asked for, which is worse than no work at all. Found by reading
the prompt before trusting it, not by watching it happen.

**It told every worker to stop when anything was unclear.** The prompt ended:
*"If the task cannot be completed within the accepted decisions, write no
handoff, leave your work in the tree, and say what decision is needed."* On an
unattended run there is nobody to say it to. The task comes back unstarted, the
loop ends, and a day is lost to a question the worker could have answered.

**And one task failing ended the whole run.** A worker that produced nothing, or
a task whose gates would not pass, broke out of the loop. Every other task —
including ones that would have gated first time — sat untouched until somebody
noticed.

## Deciding is the instruction now

The worker is told what it is building and to build at that standard, and then:

> **DECIDE RATHER THAN STOP.** Where the task leaves something open — a name, a
> shape, which of two reasonable designs — choose it the way a senior engineer
> would, and write what you chose and why in your report.

**With three exceptions, and they are absolute.** A worker told to decide and not
told these is far more dangerous than one told to stop:

- never weaken a gate, take an exemption, widen a grant, or add an unsafe block;
- never claim unfinished work is finished, and never write a handoff for a
  partial task;
- never quietly narrow a promise in `docs/features.md` or contradict an accepted
  ADR.

And a way out that is still work rather than a refusal: **if the only way forward
runs through one of the three, the decision itself is the task.** Write the ADR
with the options, a recommendation and the consequences, hand that over, and say
the code waits on it. That is a finished piece of work — and it is what the next
worker needs in order to build. It is also what happened by hand today with ADR
0021, which had sat proposed for two days while it was the only thing blocking
the release.

## A failed task is stepped over, not slept on

**A worker produced nothing** — the task is noted, added to the run's
*given up on* set, and the loop takes the next one. It stays unfinished in the
plan; nothing is marked done.

**A task would not gate** — its work is in the tree, and nothing may be started
on top of somebody else's uncommitted work; that is the ambiguous authorship this
supervisor exists to prevent. So the work is **parked**: committed to
`parked/task-N-<moment>`, **pushed**, and `main` returns to where it was. Nothing
discarded, nothing reset, and it exists somewhere other than this disk. Whoever
picks the task up finds every line.

Deliberately not a stash — a stash is invisible from any other machine and is the
first thing lost when somebody tidies a checkout.

**Giving up on a task is not finishing it.** Its dependants stay unstartable, and
there is a test for exactly that: a loop that pushed past an abandoned dependency
would build the second floor of a house whose first floor it had abandoned.

## The one failure still worth stopping for

If a task will not gate **and** its work cannot be parked, the run stops. At that
point the work is in the tree, nothing may be built on top of it, and there is
nowhere safe to put it — carrying on would mean either discarding it or building
on it, and both are worse than stopping and saying so.

## Verified

46 supervisor tests. The new ones assert the properties that were wrong an hour
ago: the prompt names the plan **this run is driving**; it says *decide rather
than stop*; it carries all three absolutes; it offers the ADR path; a task given
up on is not chosen again; and giving up does not release what waits on it.

## What this does not change

**No gate, no evidence rule, no lock, no push discipline.** A task is still
published only when every gate passes and every named test ran and passed on its
own. The parked branch is a branch — it reaches `main` when it passes, and not
before.

**It does not make a loop a substitute for judgement.** It makes an unattended
run survive its first hard task, which is a different and smaller claim.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible.

**ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — the supervisor steps over a task it cannot finish,
parking unfinished work on a pushed branch, and workers are told to decide rather
than hand a task back.

**docs/autonomy/STATE.md** — `tools/kernel-loop` survives a failed task; workers
are sent to the plan the run is driving.
