# A loop that can drive a second workstream

- Date: 2026-09-10
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: A plan the loop can read, and a loop that can read it
- Status: **the supervisor takes a plan as an input, and v0.01 has one.**

## What was in the way

`tools/kernel-loop` works. It has published every task in this workstream: it
selects from a plan, launches a worker, runs the gates, holds each acceptance
criterion to a named test that actually ran, integrates `main`, re-gates the
combined tree and pushes. It refuses to publish work whose evidence names a test
that selects nothing, and it refuses to stage a file no task named — both caught
real mistakes of mine today.

**It could only ever drive one workstream**, because the plan it reads was a
constant: `docs/autonomy/kernel-enforcement-plan.md`. So a second workstream
could only be driven by copying the supervisor — and a copy inherits every
safeguard as a stale duplicate. The gates, the evidence rule, the lock, the
rebase, the bounded retry, the disk reserve, the zero-tests refusal: none of them
is about kernel enforcement. **The only thing that was is the file the tasks come
from.**

So the plan is an input now (`ALO_LOOP_PLAN`), and the two plans share one
supervisor rather than one of them inheriting a fork of it.

## The refusal worth having

Naming **no** plan is refused rather than quietly meaning the default. Somebody
who sets that variable meant to select a workstream; falling back would run one
workstream's tasks under the intention of another's, which is worse than
stopping and saying so.

## And a test that reads both plans

`every_plan_this_repository_drives_holds_only_tasks` opens **both** — the
kernel-enforcement plan and the new v0.01 plan — and holds each to parsing as
tasks numbered from one in order.

That is not decoration. A plan that parsed to nothing would make the loop report
*the plan has no executable task left*, which is the sentence it uses for **a
workstream being finished**. A malformed plan and a completed one would be
indistinguishable, and the loop would announce the second while looking at the
first. That is the audit-heading bug from the other end, and it is worth a test
before it happens rather than after.

## The v0.01 plan

`docs/autonomy/v0-01-delivery-plan.md`, twelve tasks across the six phases
between here and the machine, in the shape the loop reads.

It is written to be honest about three things.

**It is a spine, not a script.** Phases 2 and 3 are months of increments the size
of the ones already published — *client maximize/restore requests*, *native
window placement* — and no document written today enumerates them. What a plan
can carry is the next executable increment in each phase and the rule for adding
the one after: whoever finishes a task writes the `**Done,**` line **and the next
task**, in the same change. A plan that lags its own work is one the loop reads
as *nothing left to do*.

**Task size is a constraint.** The supervisor stops a worker at forty-five
minutes. Anything that cannot be finished and gated inside that is not a task,
it is a phase, and it is split before it is offered.

**Phase 8 is marked `scheduled` so the loop steps over it.** A supervisor cannot
plug in a laptop, and a task it could pick up and fail at forever is worse than
one it knows not to start. That relies on the scheduled-is-not-startable rule
published earlier today, which is why that one came first.

## What the plan actually asks for next

Task 2 is **the agent overlay** — the exit gate's own words are *sign in, **press
the key**, ask an agent to do something*. `alo-shortcuts` exists and is tested;
nothing appears when the key is pressed. The first increment is deliberately the
seam and not the pixels: an action that means *summon the agent*, a surface
request the compositor can honour or refuse, and **a refusal in words when there
is nowhere to show it** — because a key that silently does nothing is the worst
outcome available.

It names a constraint too: nothing in `crates/alo-shell`'s window-control files,
which are the desktop worker's live chain and a different overlay entirely.

## What this does not do

**It does not make the loop able to build a compositor.** The loop selects,
launches, gates and publishes; the work is still work. Six of the eight phases
remain and nearly all of the code in them is compositor.

**It does not shorten v0.01.** `ROADMAP.md`'s exit gate and `docs/features.md`
remain the definition, and a finished task list is a finished task list.

**It changes no gate, lock or safeguard.** The only behavioural change is which
file the tasks are read from.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible.

**ROADMAP.md** — no tick. The plan is a plan.

**docs/autonomy/QUEUE.md** — v0.01 has an executable plan the supervisor can
drive: `ALO_LOOP_PLAN=docs/autonomy/v0-01-delivery-plan.md`.

**docs/autonomy/STATE.md** — `tools/kernel-loop` takes its plan as an input and
drives two workstreams from one supervisor; both plans are held to parsing as
tasks.
