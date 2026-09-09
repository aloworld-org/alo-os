# A loop that chose an audit heading

- Date: 2026-09-09
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: What the supervisor selects, and what it must not select
- Status: **found by running it.** The first task this loop ever chose on its own
  was not a task.

## What happened

With the plan's own tasks all handed over by a person, `run` selected work for
the first time with nothing waiting, and said:

```
alo-kernel-loop: next in the plan: 1. Implemented and verified
alo-kernel-loop: launching one worker for it
```

*Implemented and verified* is a heading in the plan's **audit** — a section of
five, describing what is already true — not work anybody can do.

## Why it read it that way

`plan.rs` took *a `###` heading beginning with a number* for a task, and its
comment says why it thought that was enough: "The plan's audit sections are
headings at the same depth, and without this the loop would take one of them for
work." The belief was that the audit's headings had no numbers. They do —
`### 1. Implemented and verified` — and they are numbered from one, exactly like
the tasks.

So the first heading in the file won, and it was an audit section.

The numbers collide as well as the shape, which is the sharper half: audit `1`
and task `1` are different sections, so `**Depends on:** 1` could be answered by
whichever came first in the file. A dependency satisfied by something that is
not a task is not a dependency.

**The boundary is now the section.** Headings under `## Tasks` are tasks; every
other `##` closes it, including the *Rules* and *Completion* that follow the last
one. That is what a person reading the plan uses.

## And a scheduled task is not work either

`**Status:** blocked` was stepped over; `**Status:** scheduled` was not, and the
difference matters more than it sounds. A blocked task waits on a **decision**. A
scheduled one waits on a **moment** — a real session to log out of, a maintenance
window on a machine somebody else may be testing on. Task 10 is scheduled for
exactly that reason.

A loop that took one up would not be doing the task. It would be *arranging* it:
changing sessions or lingering, on a shared machine, while other tests run —
which is the one thing this workstream's constraints name over and over. Both
words now mean *not yet*, and the constant that holds them says which is which.

## Testable now, and tested

`plan.rs` had no tests, because every path into it went through `git show`.
Parsing is now a function over text, and the commit-reading stays where it was —
proving that an audit heading is not work should not need a repository with a
history.

Five tests on a plan shaped like this one, and a sixth on **the real plan**: the
synthetic one proves the rule, and the real one proves the rule is about the file
this loop actually opens. That last also holds the plan's tasks to being numbered
from one in order with no repeats, which is what makes a dependency mean one
thing.

## What this does not change

The loop still launches nothing unless `ALO_KERNEL_LOOP_WORKER` names a command,
still reads the **published** plan rather than the working tree, still never
takes a worker's word for anything, and still marks nothing done itself — *done*
is a judgement about evidence and a person makes it.

## What the plan says now

With this published, tasks 1–9 and 11 are done and task 10 is scheduled. **The
backend plan has no executable task left**, which is a true answer rather than an
empty one, and a loop started against it will say so and end rather than
inventing scope to stay busy.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — the supervisor reads tasks only from the plan's
`## Tasks`, and steps over scheduled tasks as well as blocked ones.

**docs/autonomy/STATE.md** — `alo-kernel-loop`'s task selection is tested against
both a synthetic plan and the real one; the backend plan currently has no
executable task left.
