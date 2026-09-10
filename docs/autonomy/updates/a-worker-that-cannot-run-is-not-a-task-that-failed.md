# A worker that cannot run is not a task that failed

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **the first unattended hour's worst failure, and the diagnosis that
  was thrown away with it.**

Both lanes had just published — the overlay seam and the local account, the
first two clauses of the v0.01 exit gate gaining their code halves. Then three
workers across the two loops exited with `1`, and within ten seconds both runs
reported **the plan has no executable task left**: the sentence the loop uses
for a *finished workstream*. Every task was still there. Not one had been
attempted.

Two defects, and the second is why the first was hard to see.

**Stepping over a task is the wrong answer to a worker that never started.** A
worker that spends ten minutes and fails has attempted the task; one that exits
in two seconds has not — the command is missing, the account is out of quota,
the machine is refusing to run it. From an exit code the two are identical, and
the loop treated both as *nobody could finish this*, consuming the plan at the
speed of the failures. Time told them apart: under a minute is not an attempt,
and **two in a row is the machine**. The run now stops with the plan untouched
— no task given up, none marked — instead of reporting a workstream complete.

**And the supervisor threw away the only account of why.** Both of the worker's
output streams were `Stdio::null()`, so three deaths left nothing to read. A
worker's own words are not evidence of its work — the loop still reads the tree
and the handoff, and always will — but *why a worker died* is not a claim about
the work. It is the thing an operator needs at two in the morning, and it is in
`.kernel-loop/worker.log` now.

The failure text says so, so somebody who meets this at 2am is told where to
look rather than left to discover the log exists.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a worker that cannot run stops the run with the plan
untouched; worker output is kept at `.kernel-loop/worker.log`.
