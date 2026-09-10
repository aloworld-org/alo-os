# One branch, and it is main

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A rule the owner set — everything goes to `main`
- Status: **rule, code and test.**

The owner opened GitHub and found a shelf of branches: `parked/task-6-…`,
`parked/lane-b-task-2-timeout-…`, five more. Seven in one evening, and **every
one of them a task that had since been redone properly and published to
`main`.** The drafts outnumbered the deliveries, and a stranger reading the
repository would have seen a graveyard of attempts rather than an operating
system.

**The rule: `main` is the only branch this supervisor pushes.** One branch in
the repository, and it is what alo OS is.

Parking still happens and still discards nothing — it is simply **local** now. A
task that fails its gates has its work committed to a branch on this disk, `main`
returns to clean, and the run carries on exactly as before. The branch is a
`git switch` away for anybody who wants it.

**The cost is stated rather than hidden.** A parked branch lives on one machine,
so losing that checkout loses the draft. That is the right trade: parked work is
by definition work that did not pass its gates, and the way to keep work is to
finish the task — which is what the loop does with it next anyway. Every parked
task so far has been redone and published within the hour.

The seven that existed are preserved as local refs and removed from the shared
repository.

**A comment would have been a request**, so the rule is a test:
`the_only_branch_this_pushes_is_main` reads this file, finds every line that
pushes, and fails on any that does not name `MAIN` or that forces. There is
exactly one push in the supervisor and it publishes `main`.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — `main` is the only branch pushed; parked work is local.
