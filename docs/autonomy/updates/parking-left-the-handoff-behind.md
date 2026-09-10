# Parking left the handoff behind

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **a successful park stopped the run one task later.**

Task 6 failed its gates and was parked correctly: work committed to
`parked/task-6-1789047518`, pushed, `main` clean, run carrying on. Exactly as
designed. One task later the run stopped dead, and the reason was the park.

**`.kernel-loop` is ignored, so `git add --all` walked straight past the
handoff.** The parked branch got the code and not the one file saying which task
it was, what evidence it claimed and which files it touched — the most useful
thing in the branch to whoever picks it up. And the handoff stayed on `main`,
naming a task nobody was pursuing any more. The next iteration selected task 9,
found a handoff for task 6, and refused because the two disagreed — correctly,
and fatally.

So parking force-adds the handoff onto the branch, and removes it from `main`
afterwards. Safe in that order, and only in that order: it is taken off `main`
because the branch is now carrying it.

**And a clean tree is not a failure to park.** The refusal above changed nothing
in the tree, so parking had nothing to commit — which git calls an error. The
loop read that as *the work could not be put anywhere safe*, the one condition
it stops for, and stopped with nothing whatsoever at risk. Nothing to park is
now nothing to park.

Two failures of the same kind: recovery machinery that had only ever been
exercised on the happy path. Parking a task worked the first time; parking and
then continuing did not.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — parking carries the handoff onto the branch and clears
it from `main`; a clean tree parks without error.
