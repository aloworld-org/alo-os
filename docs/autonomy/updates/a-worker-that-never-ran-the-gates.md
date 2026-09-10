# A worker that never ran the gates

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **the gates were catching what the worker could have caught in seconds.**

Task 7's worker wrote the approval path well and its work was refused anyway.
Not for the approval logic: `alo-saying` collected **seventeen** crates with
vocabulary where eighteen were expected, and that one number failed
**twenty-four** image checks behind it. The approval sentences needed new words;
registering words touches a collection list and an image manifest; the worker
did not know, finished, and handed over.

Twenty minutes later a supervisor told it so — except a supervisor cannot tell a
worker anything. The worker was gone. The task was parked, and the next attempt
starts from nothing knowing none of this.

**The workers had never been told to run the gates.** Nowhere in the prompt.
They wrote, they handed over, and the first anyone saw of a mechanical fault was
after a full workspace run in somebody else's process.

So: `cargo fmt --all`, `cargo clippy --all-targets` with warnings denied, and
`cargo test --workspace`, before the handoff, with the reason stated — *the
commonest failure is not your logic but a registration you did not know about*.
A new crate that has words must be collected. An image manifest must agree. A
rustdoc link must resolve. Every one of those is named in seconds by a gate, and
**the worker is the only one who can fix it before an hour is spent**.

It would have caught tonight's rustdoc link too, and the overlay's crate
registration, and this.

**The supervisor still runs every gate itself.** A worker's word remains
evidence of nothing — the tree and the handoff are what get gated, exactly as
before. What changes is that a worker now sees its own mechanical faults while
it can still fix them, instead of a stranger seeing them when it cannot.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — workers run the gates before handing over; the
supervisor runs them again regardless.
