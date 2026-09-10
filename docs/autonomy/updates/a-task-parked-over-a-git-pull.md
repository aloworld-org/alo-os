# A task parked over a git pull

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **the third time this task was parked, and the third unrelated
  reason.**

Lane B's session wiring has now been written three times. Parked once at a
forty-five-minute deadline that was too short. Parked once for `alo-shell` tests
that fail under two-lane contention and pass alone. And parked a third time
tonight — for `git pull`.

The worker had finished. The handoff was there, the files named, the gates not
yet run. Between the worker starting and finishing, **the other lane published**,
so `origin/main` had moved; the loop pulls before gating, and git will not
fast-forward over a tree with uncommitted files in it. *Please commit your
changes or stash them before you merge. Aborting.*

With one lane this could not happen — nothing else published while a worker
wrote. **With two lanes it is the ordinary case**, and every finished task is
exposed to it.

So the pull is skipped when a task's work is in the tree, exactly as it is
already skipped when the checkout has unpublished commits, and for the same
reason: that is not a state a fast-forward applies to. **Nothing that matters is
skipped.** What arrived is integrated a few steps later by the publish path,
which commits first and then *rebases* onto `origin/main` — the operation that
works over a tree with changes in it, and the one that was always going to do
the real integration. The pre-pull was an optimisation that only ever worked
when nothing else was happening.

**And one sentence was lying.** The park message still said the work was
*pushed*; since `main` became the only branch this pushes, it is not. It says
local now. A supervisor whose log describes something it stopped doing is worse
than one that says nothing, because somebody will go looking on GitHub for a
branch that was never there.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — the pre-gate pull is skipped over a dirty tree; parked
work is local and the log says so.
