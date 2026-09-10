# Two loops, two lanes

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`, plans)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it (continued)
- Status: **v0.01 runs as two lanes, from two checkouts, with no task shared.**

## The question, and the gap it exposed

*Can more than one loop run at once?* One per checkout, yes — the OS-held lock
forbids two in one working tree, and it is right to. Separate checkouts are the
approved pattern here already.

The gap: **the supervisor has no task-claiming.** Two loops reading one plan
both select the same next task, launch two workers at it, and produce two
competing implementations racing to publish. Nothing prevented it except there
having only ever been one loop.

## Partition by file is the claiming mechanism

`docs/autonomy/v0-01-lane-b-plan.md` carves the one chain out of the v0.01 plan
that is independent of the overlay work: **accounts and session entry** — the
local account, and the daemon started into a real session. The main plan marks
those two tasks as lane B's, in words the selector already steps over, so the
first loop cannot take them.

The cost is a protocol, written in the lane file so it is not folklore: when
lane B finishes, its final handoff marks the matching tasks done in the main
plan, in the same commit — the image task there depends on session work it can
only see in its own file.

Lane B's plan joins `every_plan_this_repository_drives_holds_only_tasks`, so a
plan that rotted into parsing as nothing — which the loop would report as *the
workstream is finished* — fails a test instead.

## What two loops do and do not buy

**Workers parallelise; gates largely do not.** The two checkouts share one WSL
distribution and one build cache, so gate runs contend and roughly serialise.
Most wall-clock in a task is the worker writing it, so the gain is real — but it
is nearer half again than double, and a third loop on this machine would mostly
queue behind the other two's gates. Two is the ceiling here until the machine
stops being the bottleneck.

**And review debt doubles.** A loop publishes only what passes gates and named
evidence; it cannot tell plausible from right. Two lanes means twice the
published increments somebody should actually read.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — v0.01 runs as two lanes: the main plan in one
checkout, accounts-and-session in another, no task in both.

**docs/autonomy/STATE.md** — two supervisor loops run concurrently from separate
checkouts; partition-by-file is the claiming mechanism.
