# A failed gate buys one more worker, not a parked afternoon

- Date: 2026-09-11
- Workstream: the delivery supervisor (`tools/kernel-loop`)
- Contributor: Claude Code
- Status: **the instruction was the wrong lever; the loop's answer to failure is the right one.**

Three workers in a row handed over code that did not compile. The first was told
to run the gates. The second was told in stronger words. The third was told that
the handoff may not be written until they pass, with the count of who had ignored
it before and why it costs an hour. It handed over tests naming
`alo_keeping::Reading::disagreement`, a method nobody wrote.

That is enough evidence to stop rewriting the sentence. **A worker cannot be made
to gate by being told to**, and every hour spent proving that again is an hour the
release does not get. So the change is not to the instruction — it is to what the
loop does with a refusal.

Until now a failed gate meant parking: the work moved to a local branch, the task
was given up on for the run, and a person had to come and read the error. But the
gate output is the most actionable sentence in the entire run — it names the
missing method, the unregistered crate, the unresolved link — and **nobody was
reading it back to anybody.** The loop had the diagnosis and threw it away.

Now a refusal launches **one more worker on the same task**, holding what the
gates said, over work that is still in the tree. It is told plainly that the work
is already there and is to be finished rather than started again, told to read the
diff against `main` first, warned against deleting a test that names something
that does not exist (that usually means the thing was meant to exist), and told it
is the second and last attempt. The refused handoff is moved to
`.kernel-loop/refused/` — kept, like every other one, because the pair of the
refused and the repaired one is how anybody later sees what the gates caught.

**Once, never twice.** A second failure is the work rather than an oversight, and
parking is what that is for. Nothing about what the supervisor trusts has changed:
it reads the tree and the handoff, runs every gate itself, and publishes only what
passes. A worker's account of its own work is still evidence of nothing.

What this buys is the afternoon. On an account with a session limit, an hour of
finished work parked over a missing `cargo test` is the most expensive thing that
can happen in a run, and it was happening roughly one task in four.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a refused task gets one repair attempt before it parks.
