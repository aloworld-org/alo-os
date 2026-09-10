# Gating is not a step after the handoff

- Date: 2026-09-11
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **told once and ignored twice; bound to the artefact instead.**

Two workers have handed over code that does not compile. The audit's, with
`expect()` in library code. Lane B's task 6, with tests naming an
`alo_keeping::Agreement` and two `Account` methods that do not exist. Both were
told to run the gates. Both wrote the handoff anyway.

Reading the instruction back, it invited exactly that: *run the gates before you
hand over* is a **step**, and a step can be done after the interesting part is
finished, or skipped when the work feels done. The handoff is the artefact a
worker produces; the gates were somewhere else.

So the instruction is bound to the artefact now. **Do not write the handoff until
the gates pass** — not *run them and hand over*, because the handoff is the
worker's statement that the work is finished, and work that does not gate is not
finished. Fix, re-run, and only then write the file.

It also says why, with the count: two workers, both told, both caught, an hour
lost each time. And that the cause is never the logic — it is a registration
nobody could have known about. A new crate that has words must be collected. An
image manifest must agree. A rustdoc link must resolve. **A test must name a
method that exists.** Each is named in seconds by a gate, and only the worker can
act on it: by the time the supervisor runs them, the worker is gone.

Nothing about the supervisor changes. It runs every gate itself and publishes
only what passes — a worker's word is still evidence of nothing. What this buys
is the hour, and on an account with a session limit the hour is the scarce thing.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a worker writes no handoff until the gates pass.
