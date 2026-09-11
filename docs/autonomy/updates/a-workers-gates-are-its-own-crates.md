# A worker's gates are its own crates', not the workspace's

- Date: 2026-09-11
- Workstream: the delivery supervisor (`tools/kernel-loop`)
- Contributor: Claude Code
- Status: **the deadline was killing finished work; the instruction was the knife.**

Two finished tasks died at the 90-minute deadline in one night, both the same
way: the work complete, the worker's log saying *waiting on the WSL workspace
suite*, and no handoff — because the prompt demanded `cargo test --workspace`
before the handoff may be written, and under two-lane contention that suite
takes the better part of an hour. Lane B's carry-or-fetch measurement and lane
A's task 23 both had to be recovered by hand from what the dead worker left.

The demand made sense when it was written: it existed to stop workers handing
over code that does not compile. But the repair mechanism now covers that case
for a few minutes' cost — a refused task goes back to a worker holding the
exact gate error — while the workspace demand costs the whole task whenever
the suite outlasts the clock. The expensive guard was bought before the cheap
one existed, and nobody re-priced it.

So the prompt now says: gate what you touched. `cargo fmt --all`, clippy with
warnings denied, and `cargo test -p <crate>` for every crate in the change —
minutes, and they still catch the commonest cause of a dead handover, which
was never the logic but a registration in the worker's own crates. And it says
plainly: DO NOT run the full workspace suite; the supervisor runs it after you
regardless, and a cross-crate break comes back to a worker with the error in
hand. The repair prompt says the same.

Nothing the supervisor trusts changes, again: every gate still runs on every
publish, and nothing reaches `main` that the whole workspace does not pass.
What moved is only who waits on the slow suite — the supervisor, which has no
deadline, instead of the worker, which does.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — workers gate their own crates; the workspace is the
supervisor's.
