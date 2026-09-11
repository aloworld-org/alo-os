# The four promises with no evidence, and which of them a lane can still reach

**Date:** 2026-09-11
**Workstream:** v0.01 delivery — `docs/autonomy/v0-01-delivery-plan.md`, task 19
**Contributor:** Claude (`C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this task was

`docs/autonomy/v0-01-evidence.md` records **four v0.01 promises with no evidence
at all**, and one sentence beside them says each needs a screen, a decision or a
machine. That sentence had been carried unexamined since it was written: three of
the four — *copy, cut and paste*, *the GPU works on first boot*, *boots on one
certified machine* — were sorted into *needs a machine* by the audit that found
them, in a single pass, while it was finding six things at once.

So this task is a reading, taken one promise at a time against the crates, the
contracts and the decisions this repository actually has, and written down under
the promise it is about — so the next person inherits the argument rather than
the verdict.

## What the reading found

**The count stays at four. Nothing was closed here and nothing was ticked.** What
changed is that one of the four was in the wrong pile.

### Copy, cut and paste — reachable, and it is now task 20

A clipboard is a **protocol before it is a surface**. One client owns the
selection and says which types it can give; another asks for one of those types
and is handed a pipe; the compositor brokers and holds nothing of its own. Every
one of those is a value, and every refusal in it — a type that was never offered,
an offer left over from an owner who has since given the selection up, a paste
with nothing behind it — is decidable with no pixels, exactly as
`crates/alo-overlay`, `crates/alo-approving`, `crates/alo-indicator` and
`crates/alo-recounting` decided their surfaces without drawing one.

Nothing gates it, either, and that is the half the original sorting could not
have known without looking:

- **No agent verb touches the clipboard.** The ten verbs in
  `docs/contracts/agent-verbs.md` and `docs/by-hand.md` are six about files and
  four about applications. ADR 0001's grant model is not in this promise's way:
  copy and paste is a person moving their own text between their own windows.
- **ADR 0005's portal is the other route, and it is v0.5.** The sandboxed
  application's clipboard is a portal interface, and `docs/features.md` schedules
  the portal set at v0.5. What v0.01 promises is the native Wayland selection
  between clients of our own compositor.
- **The protocol is already in the pinned engine.** `crates/alo-shell` enables
  `smithay` 0.7's `wayland_frontend`, which carries the data-device protocol. The
  wiring is the desktop lane's; the value it would wire to is not.

Task 20 of the delivery plan is that increment, written with its own acceptance,
including one thing it must show it does **not** do: the clipboard is not
`alo-context`'s selection. ADR 0001 §4 offers the focused window, the highlighted
text and the open document at the moment of invocation and for that turn; what a
person has copied is neither, and a turn that read it would be the background
reader that ADR calls a bug.

### The GPU works on first boot — not reachable

`docs/hardware.md` already defines this promise in four clauses: the display
comes up at native resolution with no configuration; the card is available to the
model runtime with no driver installation and no CUDA or ROCm archaeology; a
model runs in one command; and an upgrade cannot break that stack, because the
runtime is versioned with the drivers it needs. Three of the four are answers a
machine gives and nothing else does.

The fourth is about what an image contains, and it is not reachable either:
`image/Containerfile` adds two binaries, two units, two directories and one
description to a pinned base and **carries no model runtime and no weights at
all** — which is the gap ADR 0025 found from the other side. There is nothing on
this image for a card to accelerate, so a check that the image names a driver
stack would be a check with nothing to hold, and *which* stack is pinned for the
certified workstation is engine configuration under ADR 0011 with no machine
behind it yet. The machine is task 12, which is scheduled and needs hardware
nobody has plugged in.

### The agents point at the local model by default — not reachable

Re-read, and the reading did not move it.
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md` is
proposed rather than accepted, and its recommendation asks for one line of
`docs/features.md` to be reworded — which is the owner's and nobody else's. A
worker starting code before the answer would be choosing between the four
options rather than building one. **It waits on a person rather than on a task.**

### Boots on one certified machine — not reachable

The one of the four that was honestly waiting rather than unexamined, and both
halves were checked again. The firmware half is a machine. The sign-in half is a
binary that does not exist: `crates/alo-shell` has no `src/main.rs` and no
`[[bin]]`, which is the finding task 10 stopped on, and building one means
choosing between the options in
`docs/decisions/0024-what-a-person-signs-in-at.md`. A machine **and** a decision,
with no increment in between.

## What was built, and why this shape

The reading is the deliverable, and a reading nothing holds is a reading that
rots. So the ledger is now held to one rule it was not held to before: **a
promise with no evidence at all has to say where the work is.**

`crates/alo-reconciling/src/waiting.rs` reads what a promise waits on — a
decision under `docs/decisions/`, or a task in a plan — and follows it to a file
on the disk. Three decisions are written into the shape rather than into prose:

**The plan goes beside the number.** A task is cited as
``task 20 of `docs/autonomy/v0-01-delivery-plan.md` ``, and *task 12 of the
delivery plan* is not a pointer this reads. That is deliberate rather than a gap
in the parser: this repository drives three plans, their tasks are all numbered
from one, and a reader who cannot see which plan is meant has been handed a
number that matches something in every one of them. It is `alo-citing`'s rule
about ADR filenames, applied to the other kind of number a ledger carries — and
it cost two real sentences in the ledger a rewrite, which is the rule doing its
job on the day it was written.

**A task is only a task under `## Tasks`.** `tools/kernel-loop` reads a plan that
way because it learnt the hard way: the first thing that supervisor ever selected
on its own was an audit section called *Implemented and verified*, whose heading
is numbered from one at the same depth as the work. A pointer read any more
loosely would land on it too.

**Being owed is not the finding; being owed and pointing nowhere is.** A promise
shown in part is not held to this rule at all, because the code it already has is
where the next reader goes. A promise shown by nothing leaves them the sentence
and nothing else — and that is the entry whose reasoning gets derived again from
scratch every time somebody opens the file, which is the seven-times-over reading
this ledger exists to end, arriving from the other end.

What it judges is the **pointer**, never the argument. Whether task 20 is really
the increment the clipboard needs is a reader's judgement and nothing mechanical
reaches it; whether task 20 is there at all is arithmetic.

## Files

- `crates/alo-reconciling/src/waiting.rs` — new: what a promise waits on, and
  whether a reader can follow it.
- `crates/alo-reconciling/src/finding.rs` — two findings:
  `APromiseOwedWithNowhereToGo` and `AWaitNobodyCanFollow`.
- `crates/alo-reconciling/src/reconciling.rs` — where they are raised.
- `crates/alo-reconciling/src/lib.rs` — the module, and why it exists.
- `crates/alo-reconciling/tests/every_v0_01_promise_is_reconciled.rs` — the
  measurement against this repository, and the refusals against a fixture.
- `docs/autonomy/v0-01-evidence.md` — the reading, under each of the four
  promises, and what the four add up to.
- `docs/autonomy/v0-01-delivery-plan.md` — task 19 marked done; task 20 written.
- `docs/autonomy/updates/the-four-promises-with-no-evidence.md` — this report.

No crate declares strings here and `alo-saying` does not collect
`alo-reconciling`; it is a repository check like `alo-by-hand`, `alo-collected`
and `alo-citing`. Nothing in `crates/alo-shell` was touched.

## Verification

Windows 11, `C:\dev\alo-os-claude`, product workspace unless stated.

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo test --workspace` | see below |
| `cargo test -p alo-reconciling` | 20 unit + 15 integration, all passing |

**Six pre-existing Windows-only failures in `alo-recounting` remain**, exactly as
tasks 17 and 18 reported them, and they are deliberately not cut to green here:
they are about how a record file is read on this platform and they are not this
task's work. Nothing in this change touches that crate.

### Acceptance, criterion by criterion

| Acceptance | Test |
|---|---|
| Each of the four is read against what the repository has and written down as reachable-with-an-increment or not-with-what-it-waits-on | `each_promise_with_no_evidence_names_where_the_work_is` (`alo-reconciling`) |
| The finding goes in the ledger under the promise it is about, and the reconciling gate passes on the change | `every_v0_01_promise_is_reconciled_against_evidence_that_runs` (`alo-reconciling`) |
| Where one is reachable, the next task in the plan is the increment | `a_promise_waiting_on_a_task_nobody_wrote_is_refused` (`alo-reconciling`) — task 20 is followed to the plan, and a number no plan has is refused |
| The refusal tested as carefully as the answer | `a_promise_with_no_evidence_and_nowhere_to_go_is_refused`, `waiting::tests::a_task_number_with_no_plan_is_not_something_to_follow`, `waiting::tests::a_pointer_nobody_can_follow_says_which_of_the_three_it_is`, `waiting::tests::a_numbered_heading_outside_the_tasks_is_not_a_task` |

## Limitations

- **No promise was closed.** Four v0.01 promises still have no evidence at all,
  and this task deliberately ticks none of them.
- **The clipboard is scheduled, not built.** Task 20 is the increment and the
  compositor wiring stays with the desktop lane.
- The check reads pointers, not arguments: a ledger entry can point at a task
  that exists and is the wrong task, and only a reader catches that.
- Task pointers written without the plan beside the number are invisible to the
  check. That is the rule rather than a gap, and the two sentences in the ledger
  that were written that way have been rewritten to name their plan.

## Proposed shared-document updates

For the integration owner — this contributor does not edit `CHANGELOG.md`,
`ROADMAP.md`, `docs/autonomy/QUEUE.md` or `docs/autonomy/STATE.md`.

**CHANGELOG.md**, under the current release:

> The evidence ledger now has to say where the work is. A v0.01 promise with no
> test and no report behind it names the decision it waits on or the task that is
> the increment, with the plan beside the number, and the pointer is followed to a
> file on the disk — a promise that is missing and points nowhere is one the next
> reader works out again from scratch. The four promises with nothing behind them
> were each read against the repository: one of them, *copy, cut and paste*, had
> been filed as needing hardware and does not, and it is now scheduled.

**ROADMAP.md**: no line moves. Nothing was shown and nothing was ticked.
