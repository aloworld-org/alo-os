# A deadline that stopped good work

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: The daemon's environment is the session's (supervisor correction)
- Status: **the worker was not failing. It was still working.**

Lane B's worker was killed at forty-five minutes, mid-task, with the image's
service definitions half-written toward starting `alo-agentd` inside a real
session. Nothing was wrong with the work and nothing was wrong with the worker.
The deadline was wrong.

It exists for a good reason — a worker that has stopped making progress must
not hold a checkout indefinitely, because the checkout is the thing no second
editor may touch. But a deadline that stops work still being done is set for the
supervisor's comfort rather than the work's, and forty-five minutes was chosen
before there was a single measurement to choose it from.

There are two now. The overlay's at-rest states finished in **twenty-six**
minutes. Wiring the daemon into a session was still going at **forty-five**.
Ninety leaves room for the second kind while still catching a worker that has
genuinely stuck.

**It is not a licence to write bigger tasks.** The plan's rule is unchanged: a
task that cannot be finished inside the deadline is not a task, it is a phase,
and it is split before it is offered. Raising this again to fit a task would be
using the deadline to avoid the splitting.

The stopped worker's output is on `parked/lane-b-task-2-timeout-1789046173`,
pushed. The task is offered again from clean rather than on top of it: a fresh
worker cannot ask the previous one what it was in the middle of, and inheriting
half a design it does not understand is not obviously better than starting.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — the worker deadline is ninety minutes, set from measured
task lengths.
