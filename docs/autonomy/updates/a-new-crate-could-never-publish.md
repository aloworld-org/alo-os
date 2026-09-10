# A new crate could never publish

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **found by the first autonomous task, fixed in one flag.**

The v0.01 loop's first worker did its job: `crates/alo-overlay`, ten files, a
report, and one named test per acceptance criterion. The supervisor refused it —
*these are changed and no task named them: `crates/alo-overlay/`* — parked it on
`parked/task-2-1789028569`, pushed the branch, and carried on. Every safeguard
built this afternoon behaved exactly as designed, around a check that was wrong.

**Porcelain's default collapses a brand-new directory into one entry.** The
handoff named all ten files; the tree said `?? crates/alo-overlay/`; the
comparison could never match. So any task that *created a directory* — which is
any task that adds a crate — was unpublishable however carefully it named its
files. It never showed before today because every earlier task changed files
that already existed.

`--untracked-files=all` lists the files instead of the directory, at both call
sites, so the cleanliness check and the rename check read the tree one way.

The parked work is restored to the tree exactly as the worker left it, and the
loop gates it with the corrected check — through the gates, not around them.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a task that creates a crate publishes; parked work from
`parked/task-2-1789028569` was restored and gated normally.
