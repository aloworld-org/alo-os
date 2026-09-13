# An exit code is not evidence, and the loop was believing one

- Date: 2026-09-13
- Workstream: the delivery supervisor (`tools/kernel-loop`)
- Contributor: Claude Code
- Status: **the third place the loop concluded something without looking.**

`worker.rs` opens by saying what this supervisor is for:

> A worker's own account of what it did is not evidence. Nothing here reads
> what it said, believes an exit code, or takes a summary as a result. The loop
> looks at **the working tree and the handoff**.

The loop then did the thing that paragraph forbids. On a worker exiting
non-zero it returned *nobody handed over its work* — **without looking at the
handoff.**

On 2026-09-13 a worker finished task 20 entirely: the tenth hook on
`inode_get_acl`, its reproduction flipped, the unwatched-mutation list brought
to twenty-three, the quirks row closed, the plan marked, the report written,
and a handoff naming every one of the sixteen changed files. Then it exited
non-zero, because the account had run out of usage credits. The handoff was on
disk and correct. The journal said nobody had handed anything over, and the
task was stepped over.

A worker can write its handoff and then die of many things that say nothing
about the work: an exhausted account, a killed terminal, a panic on the way
out, a machine going to sleep. The handoff is the artefact; the exit code is
gossip about the program that produced it.

So the `Err` arm looks now, and only says nobody handed over when nobody did.
A worker that exited badly and left a handoff has handed over, and what it left
is gated exactly like anything else — nothing here makes a bad exit
*acceptable*, it makes the tree the thing that decides.

**One case is deliberately untouched.** A worker that died in under a minute is
still *the machine*, before any of this: that check exists so a broken toolchain
cannot consume a plan at the speed of its own failures, and a handoff cannot
appear in a second that never ran a task.

## The pattern, now three deep

| Where | What it concluded without looking |
|---|---|
| `waiting_for` | honoured a stop before looking for a handoff, discarding a finished task |
| `waiting_for`'s bound | waited an hour for a worker that had already exited |
| the `Err` arm | called a handoff missing because the exit code was non-zero |

Each cost real work and each was found the same way: by a task that was done
being reported as not done. **Look, then conclude** is the rule all three
wanted, and it is worth saying once here rather than being rediscovered a
fourth time.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a handoff is read before a worker is blamed for not
writing one.
