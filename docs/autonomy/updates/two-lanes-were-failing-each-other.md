# Two lanes were failing each other

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **the failures were not in the work, and not in either lane.**

Both lanes parked a finished task tonight for *the workspace's tests did not
pass*. Loop A's folder selection and lane B's session wiring, both refused, both
sent to a branch.

Neither had broken anything. The two failing tests were
`window_controls::name_fallback` — in `alo-shell`, a crate **neither lane
touches**, belonging to a worker who is away until the fifteenth. Run alone on
the same commit, that binary passes **262 of 262**.

**The lanes were failing each other.** They gate at the same time on one
machine, and those tests are sensitive to how much of it they get. The result is
the worst kind of failure an autonomous run can have: good work parked, the task
left unfinished, and a diagnosis pointing at a crate that has nothing to do with
it. Both parks tonight were this, and without looking it would have read as two
workers writing bad code.

**So a gate that fails is run once more before it is believed.** Passing on the
second run is recorded as such — `(on the second run)` — rather than hidden, so
a lane that starts needing the retry often is visible rather than quietly
tolerated.

**It does not make a failure ignorable.** A gate that fails twice fails, and the
message says it ran twice. A real break fails both times; there is no flag
anywhere that turns a gate off, and this adds none. What it removes is the
transient — the one kind of failure that says nothing about the work.

What it does not fix is the cause. Two loops on one machine will keep colliding,
and the honest ceiling stated when the second lane started still holds: **two is
the limit here**, and the second one is bought at the price of this kind of
noise.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a failed gate is run twice before it is believed; the
`alo-shell` name-fallback tests are timing-sensitive under two-lane contention.
