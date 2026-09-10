# v0.01 — the executable plan

The eight phases in `docs/autonomy/DELIVERY.md` say *what order*. This says *what
to do next*, in the shape `tools/kernel-loop` can read: a numbered task under
`## Tasks`, a `**Status:**`, a `**Depends on:**`, and a `**Done, <date>.**` line
written by whoever did it.

Point the loop at it with `ALO_LOOP_PLAN=docs/autonomy/v0-01-delivery-plan.md`.

## What this plan is honest about

**A loop cannot invent scope, and this plan does not pretend to hold a whole
compositor.** Phases 2 and 3 are months of increments the size of *client
maximize/restore requests* or *native window placement* — the ones already
published — and no document written today can enumerate them. What a plan can
carry is **the next executable increment in each phase**, and the rule for
adding the one after it.

So this file is a spine, not a script. When a task finishes, whoever finished it
writes the `**Done,**` line **and the next task**, in the same change. A plan
that lags its own work is a plan the loop reads as *nothing left to do* — which
it reports as the workstream being finished, and which would be a lie.

**Task size is a constraint, not a preference.** `tools/kernel-loop` stops a
worker at forty-five minutes. A task that cannot be finished and gated inside
that is not a task, it is a phase, and it must be split before it is offered.

**Two workers share this.** The desktop worker owns `crates/alo-shell`'s
compositor and window-control chain and is away until 2026-09-15. Tasks below
name an owner where it matters, so the two do not arrive in the same file from
two checkouts.

## Rules this plan holds itself to

- **Nothing is ticked from a fixture.** WSLg is a development fixture; a nested
  session proves a client talks to us, never that a machine boots to us.
- **No release verdict from this file.** `ROADMAP.md`'s exit gate and
  `docs/features.md` are the definition of v0.01. A finished task list is a
  finished task list.
- **Phase 8 is not startable here** and is marked so, so the loop steps over it
  rather than launching a worker at a laptop nobody has plugged in.
- Every task names the acceptance that would show it done, before it is done.

## Tasks

### 1. A plan the loop can read, and a loop that can read it

**Status:** ready. **Depends on:** nothing.

`tools/kernel-loop` read one hard-coded plan — the kernel-enforcement
workstream's — so a second workstream could only be driven by copying the
supervisor, and a copy inherits every safeguard as a stale duplicate: the gates,
the evidence rule, the lock, the rebase, the bounded retry. None of those is
about kernel enforcement. The only thing that was is the file the tasks come
from.

- **Acceptance:** the plan is an input (`ALO_LOOP_PLAN`); naming nothing is
  refused rather than silently meaning the default; and **both** plans this
  repository drives parse to tasks numbered from one in order — a plan that
  parsed to nothing would send a worker at *nothing to do* and read as the work
  being finished.

**Done, 2026-09-10.** `tools/kernel-loop/src/plan.rs`, and this file is the
second plan.

### 2. The agent overlay: one key, from anywhere

**Status:** ready. **Depends on:** nothing that is not already built.
**Owner:** Claude, while the desktop worker is away.

v0.01's exit gate is *sign in, **press the key**, ask an agent to do something*.
The key is `alo-shortcuts`, which exists and is tested. What does not exist is
anything that appears when it is pressed.

This is the most product-defining item left in v0.01: an agent one keystroke from
anywhere is what makes this an AI-native operating system rather than a desktop
with an assistant in a window.

Deliberately **not** the whole overlay. The first increment is the seam:

- a shortcut action that means *summon the agent*, declared in `alo-shortcuts`
  with the rest, refusing to clash with a bound key;
- a surface request the compositor can honour or refuse, with no rendering in it;
- and a refusal in words when there is nowhere to show it, rather than nothing
  happening — a key that silently does nothing is the worst outcome available.

- **Acceptance:** the chord resolves to the action; pressing it asks for the
  surface exactly once; a second press while it is open does not ask twice; and
  with no compositor there is a refusal a person could read. No pixels are
  claimed and none are tested.
- **Constraint:** nothing in `crates/alo-shell`'s window-control files, which are
  the desktop worker's live chain and a different overlay entirely.

**Done, 2026-09-10** — `e728fa6`, `crates/alo-overlay`: `Summoning` holds
one-press-one-request in the compiler, the integration test presses the shipped
`Super+A` through `Action::TheAgent`, and nowhere-to-show refuses in words.
Report: `docs/autonomy/updates/agent-overlay-summoning-seam.md`. *Marked done by
the supervisor's operator rather than the worker's handoff — the omission that
taught the prompt to demand it.*

### 3. What the overlay shows when the agent has nothing to say yet

**Status:** ready. **Depends on:** 2.

The overlay's content before a question is asked: what a person sees when they
press the key. The three things `alo-agentd` already answers — what is granted,
what model would answer, and whether anything left the machine — are values this
repository has and no screen has ever shown.

- **Acceptance:** the overlay's state is a value derived from the daemon's own
  answers, with a case for each of *nothing granted*, *nothing chosen* and
  *ready*; every string externalised; and the *nothing chosen* case says what to
  do rather than being empty.

**Done, 2026-09-10.** `crates/alo-overlay`: `AtRest` reads the person's own
settings, the machine's grants and the machine's egress indicator, and
`Standing` derives the three states from them with no constructor that can name
one. Nine strings and two counted ones under the `overlay` area, and
`alo-saying` now collects this crate — it did not, so every sentence task 2
declared would have reached a real shell as a bug. Report:
`docs/autonomy/updates/what-the-overlay-shows-at-rest.md`. The next task (4) was
already written.

### 4. Accounts and session entry — the local account

**Status:** lane B's — scheduled in `v0-01-lane-b-plan.md` as its task 1, and
stepped over here so two loops never take up one task. Lane B's finishing
handoff marks it done here. **Depends on:** nothing.

Phase 4, and the half that needs no identity provider. v0.01's gate opens with
*sign in*, and nothing in this repository signs anybody in. The local account
that needs no tenant is the smaller half and the one the gate actually requires.

- **Acceptance:** a local account is created and authenticated against the
  machine's own store; a wrong password is refused in words and is not
  distinguishable by timing from an unknown user; and the session that results
  carries the uid `alo-agentd` is told about, so `docs/contracts/machine-description.md`
  and the running session cannot disagree about who is signed in.
- **Constraint:** no identity provider, no tenant, no network. Those are the
  other half of phase 4 and are their own task.

**Done, 2026-09-10.** Lane B's task 1: `crates/alo-accounts` — the store, the
evenly-timed refusal, and the session that cannot disagree with the machine
description. The entry surface stays with the compositor lane;
`docs/autonomy/updates/the-local-account-that-needs-no-tenant.md` is the
report.

### 5. The daemon's environment is the session's

**Status:** lane B's — scheduled in `v0-01-lane-b-plan.md` as its task 2, and
stepped over here for the same reason as task 4. **Depends on:** 4.

`alo-agentd` runs as the signed-in person and finds their bus at
`/run/user/<uid>`. That is measured (`a_session_that_really_ended.rs`) and it is
not yet *wired*: nothing starts the daemon into a real session with the right
environment.

- **Acceptance:** the daemon starts under the session created in task 4, reaches
  that session's bus, and stops when the session ends — the three states already
  measured, reached from a sign-in rather than from a test harness.

### 6. Native folder selection, so a grant can be made at all

**Status:** ready. **Depends on:** 2.

ADR 0001 §3: a grant is made by a person picking a folder. `alo-capability` has
grants and `alo-agentd` enforces them, and **nothing on this machine can make
one** — which `crates/alo-agentd/src/starting.rs` says in as many words: every
verb is refused, correctly, because nothing has been granted and nothing can be.

- **Acceptance:** a person picks a folder and a grant exists afterwards that the
  daemon honours; picking nothing grants nothing; and the grant's scope is the
  folder picked rather than its parent.

**Done, 2026-09-10.** `crates/alo-picking`: `Picker` walks a real disk through
one port and can only ever stand somewhere it was shown, `Picked` is sealed so
nothing but a person's pick can become a grant, and `Granting` hands
`alo_capability::Grants` a grant the daemon's own `permits` honours — over the
folder picked and not its parent, with the top of the disk refused in words
(ADR 0001 §3). Eleven strings under the `picking` area, collected by
`alo-saying`. Report: `docs/autonomy/updates/native-folder-selection.md`. The
next task (7) was already written.

### 7. One approval, and the sentence a person approves

**Status:** ready. **Depends on:** 3, 6.

The exit gate's middle: *approve the sentence, see it happen*. `alo-turn` and
`alo-protocol` carry proposals and approvals; no surface has ever shown one.

- **Acceptance:** a proposed change is shown as the sentence `alo-turn` renders,
  approved once, carried out, and refused after its proposal has expired — with
  the record carrying what was approved and by whom.

**Done, 2026-09-10.** `crates/alo-approving`: `Asked` is made from an
`alo_capability::Waiting` and from nothing else — no constructor from text, no
public field, no `From`, no deserialiser, and compile-fail examples that turn
adding one into a failing build — so what a compositor is handed is the sentence
the machine generated from the arguments it validated. `Approving` takes the
question off the surface before the turn is touched and every answer goes
through `Turning::approving`, which is where the grants are asked again and the
record is written: one approval carries one change out, a second answer runs
nothing, and a question that stood too long is neither put up nor carried out —
refused in `alo-capability`'s own words, quoting the change so it can be asked
for again. Nowhere to put the question refuses in words rather than in silence.
Five strings under a new `approving` area, collected by `alo-saying`.
`alo-turn` gained two additive reads (`Turning::proposed`, `Turning::strings`)
so a surface can tell *answered already* from *stood too long* and cannot word a
change in a vocabulary of its own. Report:
`docs/autonomy/updates/one-approval-and-the-sentence-a-person-approves.md`. The
next task (8) was already written. No pixels are claimed and none are tested;
drawing it is the compositor's, and *On the machine* does not move.

### 8. Afterwards, ask what it did

**Status:** ready. **Depends on:** 7.

The exit gate's end: *ask what it did and get an answer from the record*.
`alo-record` and `alo-keeping` hold it; nothing reads it back to a person.

- **Acceptance:** a person asks and is answered from the record on the disk, not
  from memory of the session; a turn that was refused reads back as refused; and
  nothing in the answer is a sentence a model wrote.

### 9. The egress indicator, on a screen

**Status:** ready. **Depends on:** 3.

`alo-egress` decides and is tested; the indicator itself is a compositor surface
and does not exist. The exit gate requires it to have **stayed dark** throughout,
which is a claim about a surface nobody can see yet.

- **Acceptance:** the indicator is drawn from `alo_egress::Indicator` and nothing
  else; a local answer leaves it dark; a provider answer lights it while the
  question is in flight; and it cannot be drawn from a value that was not a
  departure.

**Done, 2026-09-10.** `crates/alo-indicator`: `Lamp` and `Drawn` are made from
`&alo_egress::Indicator` and there is no other constructor — no count, no
`From`, no deserialiser, and compile-fail examples that turn adding one into a
failing build. `Indicating` keeps a compositor in step with the machine (one
change, one redraw), and having nowhere to show it refuses in words rather than
in silence, because a machine that cannot show what is leaving and says nothing
looks exactly like a machine on which nothing is leaving. Four strings under a
new `indicator` area, collected by `alo-saying`. Report:
`docs/autonomy/updates/the-egress-indicator-on-a-screen.md`. The next task (10)
was already written. No pixels are claimed and none are tested; drawing it is
the compositor's, and *On the machine* does not move.

### 10. The image carries the shell, the session and the daemon

**Status:** ready. **Depends on:** 5, 9.

Phase 7. `image/` builds and boots in QEMU with the daemon running; it does not
yet carry a shell to boot *to*, a session to sign in to, or the vocabulary and
palette the shell draws with.

- **Acceptance:** the image boots to a sign-in surface in a VM, a real local-model
  turn runs through approval, execution and its record, and an update rolls back
  cleanly. **A VM is not a machine** and no *On the machine* box moves.

### 11. Reconcile every v0.01 promise against executable evidence

**Status:** ready. **Depends on:** 10.

Phase 8's first half, and the only half that can be done without hardware.
`docs/features.md` is the definition; the roadmap's audit found six promises with
no line at all, one at a time, over seven iterations, and twice believed it had
found the last.

- **Acceptance:** every `[v0.01]` line in `docs/features.md` names the test or the
  report that shows it, or is named as owed. A promise with neither is the
  finding, and it is written down before anything else is.

### 12. Physical acceptance on the two certified machines

**Status:** scheduled — it needs a machine, and no work here substitutes for it.
**Depends on:** 11.

Phase 8's second half. `docs/hardware.md`: an ordinary business laptop with no
discrete graphics first, because it decides whether this has a market, and a GPU
workstation after it. The exit gate, from a cold boot, on the real thing.

**Marked scheduled so the loop steps over it.** A supervisor cannot plug in a
laptop, and a task it could pick up and fail at forever is worse than one it
knows not to start.

- **Acceptance:** `ROADMAP.md`'s v0.01 exit gate passes on a certified machine,
  and `docs/hardware.md`'s table names it, with a date and a person.
