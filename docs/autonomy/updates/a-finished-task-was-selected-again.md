# A finished task was selected again

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **the prompt now demands what the plan's prose only requested.**

Within a minute of publishing `e728fa6` — the overlay seam, the v0.01 loop's
first autonomous publication — the loop selected the same task again and
launched a fresh worker at work that was already on `main`.

The plan's own prose says whoever finishes a task writes the `**Done,**` line in
the same change. Lane B's worker did. Loop A's did not, and nothing had made it:
the worker prompt demanded a handoff, evidence, a report — and never the plan
mark. The loop selects from the published plan, so an unmarked finished task is
selectable forever, and every selection burns a forty-five-minute worker on a
duplicate whose handoff would then collide with the published files.

Two changes. Task 2 is marked done in the plan by hand, naming `e728fa6` and the
omission. And the prompt now says, beside the handoff demand: mark this task
done in the plan named above, with the plan file among your files, and write the
next task there if the plan names none after it — with the prompt test holding
it, so the demand cannot quietly fall out.

A rule that lives only in a document a worker is told to read is a request. The
ones that must hold live in the prompt and in a test.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — workers mark their task done in the plan as part of the
handoff; task 2 (the agent overlay seam) is published as `e728fa6` and marked.
