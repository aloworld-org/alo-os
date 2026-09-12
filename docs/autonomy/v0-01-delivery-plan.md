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

**Done, 2026-09-10.** Lane B's task 2: `crates/alo-entering` derives the
session's environment from the session a sign-in opens, the agent service is
pulled in by `user@<uid>.service` and bound to it rather than started at boot,
`crates/alo-image` holds those lines to the number the machine description
names, and `alo-agentd` refuses an environment naming another login's session.
Task 10 no longer waits on this. Report:
`docs/autonomy/updates/the-daemons-environment-is-the-sessions.md`.

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

**Done, 2026-09-10.** `crates/alo-recounting`: `Recounting` holds a path and
nothing else — no record, no entries, no answer from last time and no
constructor that takes any of those — so every question re-reads the file
through `alo_keeping::Reading`, and a record that is not there is refused in
`alo-keeping`'s own words rather than drawn as an empty list. `Told` is made
from an `alo_record::Entry` and from nothing else (no constructor from text, no
public field, no `From`, no deserialiser, compile-fail examples), so what
reaches a screen is the sentence the machine generated from validated
arguments; a verb that never became a call has **no** sentence and its text
comes back only from `asked_for`. `Outcome` derives all ten things that can
happen from `alo_record::Happened` by an exhaustive match, keeping the three
refusals three, and `Account` shows the record's own sentence about whether it
goes all the way back beside every answer — so *nothing here* is never read as
*nothing happened*. Thirteen strings under a new `recounting` area, collected
by `alo-saying`. Report:
`docs/autonomy/updates/afterwards-ask-what-it-did.md`. The next task (9) was
already written. No pixels are claimed and none are tested; drawing it is the
compositor's, and *On the machine* does not move.

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

**Status:** blocked — on task 13 alone now. ADR 0024 was accepted on
2026-09-11, so what this waits on is the surface itself rather than the
decision about it. **Depends on:** 5, 9, 13.

Phase 7. `image/` builds and boots in QEMU with the daemon running; it does not
yet carry a shell to boot *to*, a session to sign in to, or the vocabulary and
palette the shell draws with.

- **Acceptance:** the image boots to a sign-in surface in a VM, a real local-model
  turn runs through approval, execution and its record, and an update rolls back
  cleanly. **A VM is not a machine** and no *On the machine* box moves.

**Blocked, 2026-09-10.** Taken up and found unstartable, which is a finding
rather than a failed attempt: **there is nothing to carry.**
`crates/alo-shell` has no binary — no `src/main.rs`, no `[[bin]]` — so no image
can install *the shell*; nothing in this repository authenticates anybody at a
screen, so there is no session to sign in to; and what runs before anybody is
signed in has never been decided. `docs/decisions/0024-what-a-person-signs-in-at.md`
is that decision, written as this task: three options, a recommendation and the
consequences of each, including the one it costs — a second privileged
component beside ADR 0018's loader. **The code waits on it being accepted.**

What was finishable without the decision was done in the same change, because
it is the premise the recommendation rests on: **the image ships no accounts**,
which is the state `alo-accounts` reads as first boot, and it is now a check in
`crates/alo-image` with the fixture that breaks it — a store committed beside
the machine description would look exactly like the file that belongs there and
would hand every holder of the image a login on every machine built from it.
`alo-accounts` grew `place.rs` so the path is readable off a machine as well as
on one. Report: `docs/autonomy/updates/what-a-person-signs-in-at.md`.

### 11. Reconcile every v0.01 promise against executable evidence

**Status:** ready. **Depends on:** nothing.

**It depended on 10 and no longer does, 2026-09-10.** That was an ordering
rather than a need: an audit of `docs/features.md` against the evidence in this
repository reads code and reports, not a booted image, and a promise the image
does not keep yet is exactly the kind of finding this task exists to write
down. Left depending on a blocked task it would have made the plan read as
*nothing left to do*, which the loop reports as the workstream being finished.

Phase 8's first half, and the only half that can be done without hardware.
`docs/features.md` is the definition; the roadmap's audit found six promises with
no line at all, one at a time, over seven iterations, and twice believed it had
found the last.

- **Acceptance:** every `[v0.01]` line in `docs/features.md` names the test or the
  report that shows it, or is named as owed. A promise with neither is the
  finding, and it is written down before anything else is.

**Done, 2026-09-10.** `docs/autonomy/v0-01-evidence.md` is the ledger — all
forty-one v0.01 promises, each naming the test or the report that shows it and
what is still owed — and `crates/alo-reconciling` is what makes it true of
`docs/features.md` rather than a list somebody wrote once: both documents are
parsed, every promise must be answered exactly once, every entry must be about a
promise the definition still makes, and evidence is a test that exists and holds
a `#[test]` or a report under `docs/autonomy/updates/` — an ADR, `ROADMAP.md`
and a file in another repository are each refused by name. **A promise added to
`docs/features.md` and not reconciled now fails the gate in the change that adds
it**, which is the only moment anybody has the knowledge to reconcile it. The
audit's own finding: two promises are shown with nothing owed, thirty-three are
shown in part, and **six have no evidence at all** — *copy, cut and paste*, *the
GPU works on first boot* and *it never nags* have no line anywhere; *anything an
agent verb can do, a person can do by hand* is a standing rule nothing checks;
*the agents point at the local model by default* cannot get a line without a
decision, because ADR 0016 refuses a default nobody chose; and *boots on one
certified machine* is task 12. Report:
`docs/autonomy/updates/every-v0-01-promise-against-evidence.md`. The next task
(14) was written from those findings, because 12 and 13 are both unstartable and
a plan whose remaining tasks are all blocked reads as the workstream being
finished.

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

### 13. A sign-in surface, and what starts it

**Status:** ready — **unblocked 2026-09-11**, when
`docs/decisions/0024-what-a-person-signs-in-at.md` was accepted as Option B
after the measurement it owed was taken: `logind` opens a session for a
privileged caller that is not `pam_systemd` (`Invalid leader PID` from root is
the call being *authorised* and only its contents refused; `Access denied` from
uid 1000 is the boundary). So alo OS has its own sign-in surface, and
`alo-accounts` is the authenticator rather than a PAM module — the exemption
from `unsafe_code = "forbid"` that Option A would have cost is not owed.
**Depends on:** 2.
**Owner:** the desktop worker — it is a binary in `crates/alo-shell`, which is
that lane's, and it is written here so the lane finds it rather than so this
one takes it.

Written by task 10, which found it missing. The first screen a person ever sees
and the only one that runs before anybody is signed in: the compositor started
at boot, `alo-accounts` asked whether the password is right, and the person's
own session opened when it is — the session `alo-agentd.service` is already
bound to and that nothing on the image can currently cause.

- **Acceptance:** a machine with no store shows *make an account* rather than a
  sign-in; a correct password opens the session the machine description names,
  and `alo-agentd` comes up inside it; a wrong one is refused in the words
  `alo-accounts` already has, with no way to tell an unknown name from a wrong
  password; whatever holds a privilege to open the session holds nothing else,
  and says so in a `crates/alo-image` check beside the loader's; and every
  string is in the vocabulary `alo-saying` collects.

### 14. Every verb's by-hand answer, and a check that it has one

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file.

Written by task 11, which found it: `docs/features.md` promises at v0.01 that
**anything an agent verb can do, a person can do by hand** (ADR 0009), and
nothing in this repository checks it. It is the standing rule on every verb
anybody proposes — a machine without a working agent loses *convenience* and
never *capability* — and today it is a sentence in a document, so a verb with no
by-hand answer would arrive and nothing would notice. The one promise of the six
this lane can close without a decision, a screen or a machine.

- **Acceptance:** every verb alo OS ships — read from the crates that declare
  them rather than from a list kept beside them, so a verb added anywhere is a
  verb this check sees — names how a person does the same thing without the
  agent, or is named as owed with the release that owns the answer; a verb with
  neither is refused in a sentence naming it; and the refusal is tested as
  carefully as the answer, including a verb added with nothing said about it.
- **Constraint:** it says nothing to a person and declares no strings — it is a
  repository check like `alo-reconciling`, and `alo-saying` does not collect it.

**Done, 2026-09-11.** `docs/by-hand.md` answers for all ten verbs and
`crates/alo-by-hand` holds the document to them. The verbs come out of the
`alo_capability::Verbs` a daemon enforces, so nothing here holds a verb's name —
and because that alone would only see verbs in crates the check already knows,
the workspace's own member list is walked too: **a crate that declares verbs in
`src/verbs.rs` and was not handed in is a finding naming it**, which is the
failure that really happened one floor down when `alo-overlay`'s nine strings
reached nothing that collected them. An answer is a **promise in
`docs/features.md`, quoted** — ADR 0009's rule is that no surface may be left out
because an agent can do it instead, and the only place a surface is committed to
is the definition — so the release that owns an answer is read off the line
rather than asserted beside it. The second form, *owed at a release the
definition actually ships*, is what one verb needs: nothing promises **making**
an archive by hand, only opening one, and stretching the file-manager line to
cover it would have been the check lying in the first change that used it.
Eleven findings, each shown refusing against a fixture beside the real
measurement. **What the document found and nobody had written down: six of the
ten verbs ship at v0.01 and their plain way — file manager, search, text editor,
terminal — arrives at v0.5**, so a v0.01 machine whose agent is unavailable can
do none of the six by hand. That is a scope fact and the owner's to move, not
this check's to refuse; it is written into `docs/by-hand.md` and
`docs/autonomy/v0-01-evidence.md`, whose entry for this promise is no longer one
of the six with no evidence at all. Rule 7 of *adding a verb* in
`docs/contracts/agent-verbs.md` is where whoever adds the next one meets it.
Report: `docs/autonomy/updates/every-verbs-by-hand-answer.md`. The next task (15)
is written below. No strings are declared and `alo-saying` does not collect this
crate.

### 15. A machine that cannot reach a model says so once

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file and needs no screen.

Written by task 14, from the audit's remaining findings. `docs/features.md`
promises at v0.01: **and it never nags** — *a machine that cannot reach a model
does not follow somebody around asking them to buy credit* — and ADR 0009 says
what that means in full: *a machine that cannot reach a model says so once, where
it happened, and continues.* It is one of the promises with no crate, no test and
no line anywhere, and it is the last of them this lane can close without a
screen, a decision or a machine.

Everything it needs exists. `alo_asking::RanOut` and `alo_answering`'s failures
are the six ways an agent becomes unavailable; `alo-turn` is where a turn ends.
What does not exist is anything that remembers a person has already been told, so
today every turn against an empty balance would produce the same sentence again —
which is the greyed-out panel ADR 0009 refused, once per turn.

- **Acceptance:** the same unavailability told once is told once, and telling it
  again takes something changing — the source, the reason, or the person asking
  again themselves; a *different* reason is a different telling and is not
  suppressed, because a machine that swallowed the second failure would hide the
  one that mattered; nothing is told at all until a turn actually failed, so
  there is no reminder anybody can be followed around by; and every string is in
  the vocabulary `alo-saying` collects.
- **Constraint:** it never asks anybody to buy anything and it never chooses
  another source — ADR 0008's *never a silent fallback* runs in both directions,
  and *we spent your money elsewhere because the first place was empty* is the
  worst available version of this. Nothing in `crates/alo-shell`.

**Done, 2026-09-11.** `crates/alo-telling`: `Telling` is the memory a session
holds, and `Telling::about` is the only door a failure can become something a
person reads through — it **takes the failure by value**, so a repeat is not
suppressed by convention but consumed and dropped, and there is no value left
anywhere that a surface could word instead. That is the difference between a
guarantee and a recommendation, and it is the whole of the design. `Unavailable`
is the identity — where the question was put and what went wrong there, read off
an `alo_answering::Failed` and buildable from nothing else — and it is
deliberately unwordable: what is remembered has no `said`, so nothing this crate
holds could be put back on a screen at a later moment, which is ADR 0009's *where
it happened* as a shape rather than as an instruction. `WhoAsked` has no
`Default`, because both possible defaults are wrong in the direction that
matters: one turns every background retry into a telling and the other silences
somebody who genuinely asked again. Two statuses from one service are two
reasons, because the number is inside the sentence a person reads. The memory is
bounded at sixty-four and forgets the **oldest** first, which is the only
direction it may fail in — a bound may cost a repetition, it may never cost a
telling. `ToldOnce` is four lines in the order they are read: the heading and the
*carry on* are this crate's two strings under a new `telling` area collected by
`alo-saying`, and the two in the middle are `alo-answering`'s own, unchanged,
because a telling that reworded a failure would be a machine with two accounts of
one moment. The offers travel with the telling, unranked and unchosen, so
adopting this crate closes no door ADR 0008 leaves open. `docs/autonomy/v0-01-evidence.md`'s
entry for *and it never nags* is no longer one of the six with no evidence at
all; four are left, and none of them can be closed without a screen, a decision
or a machine. Report:
`docs/autonomy/updates/a-machine-that-cannot-reach-a-model-says-so-once.md`. The
next task (16) is written below. No pixels are claimed and none are tested.

### 16. A default nobody chose, or a promise that says so

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it is a decision and a document, and it touches no
compositor file.

Written by task 15, from the last of the audit's findings this lane can reach.
`docs/features.md` promises at v0.01 that **the agents point at the local model
by default**, and `docs/autonomy/v0-01-evidence.md` records that it *cannot get a
line without a decision*: ADR 0016 says the organisation bounds and **the person
chooses**, and refuses a default nobody chose. Those two sentences cannot both be
kept. One of them is wrong, and no worker may quietly narrow the promise or
contradict the ADR to make a test pass — which is why nothing has been built for
it and why the audit has carried it, untouched, since it was written down.

So the decision *is* the task, in the shape task 10 already used for ADR 0024:
the options, a recommendation, and the consequences of each, written where a
stranger can read the argument rather than infer it from code. The two obvious
readings are not the only ones — *default* may mean **what a machine ships
pointing at**, which is a fact about an image and not a choice made on anybody's
behalf, and that reading may keep both sentences.

- **Acceptance:** an ADR under `docs/decisions/`, numbered next, with the options
  set out fairly, a recommendation, and what each would cost — including what it
  costs `crates/alo-choosing`, `crates/alo-image` and the setup flow ADR 0009
  gave a fourth answer to. `docs/autonomy/v0-01-evidence.md`'s entry for the
  promise names it and says what is still owed, and the reconciling gate passes
  on the change that adds it. **No code follows in the same change**, because
  until it is accepted a worker writing code would be choosing between the
  options rather than building one.
- **Constraint:** it may not narrow `docs/features.md` and it may not contradict
  ADR 0016. If the honest answer is that the definition is wrong, the ADR
  recommends the change and the owner makes it; the plan does not.

**Done, 2026-09-11.**
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md` is the
decision, **proposed rather than accepted**, because its recommendation asks for
one line of `docs/features.md` to be reworded and only the owner may move the
definition. Four options, fairly: *unset means local*, which supersedes ADR 0016
and is argued as such rather than worked around; *the image ships the person's
choice*, which is us writing in the one file ADR 0016 keeps for them and is
impossible anyway, since ADR 0024 ships no accounts and ADR 0007 makes the right
model a function of the hardware; *setup pre-selects local*, which keeps ADR 0016
literally and is persuasion by geometry, taking back the weight ADR 0009 gave the
fourth choice; and the recommended one — **the default is what a machine arrives
able to do**: the image carries the pinned runtime and a model sized for the
machine, setup lists the four with local first and pre-selects nothing, and
nothing is written into anybody's settings before they choose. Ordering is not
weight, and that distinction is the whole of what makes both sentences keepable.

**What the reading found, and nobody had written down: the promise is not blocked
on a settings key at all.** `image/Containerfile` adds two binaries, two units,
two directories and one description to a pinned base and carries **no model
runtime and no weights**, so on the machine this repository builds there is
nothing local to point at — the expensive half of *local by default* is a model
on a disk, not a value in a file, and it is unbuilt in `image/`. The measurement
the implementation owes is named rather than guessed, in ADR 0024's shape:
whether the weights are carried on the certified image or fetched at setup, since
a machine that fetches at setup is not local by default when it is offline at
setup.

`docs/autonomy/v0-01-evidence.md`'s entry names the decision and says what is
owed underneath it, and the count of promises with no evidence at all **stays at
four**: a proposed decision is not evidence that anything was built, which is
what `alo-reconciling` refuses an ADR for in the first place. What is new there
is a check rather than a promise —
`a_promise_that_waits_on_a_decision_names_one_that_is_there` holds the ledger to
the decision it points at, because the one kind of pointer this repository
deliberately refuses as evidence was also the one kind nothing verified, and a
ledger sending a reader to an ADR nobody wrote reads exactly like an answer.
Report: `docs/autonomy/updates/a-default-nobody-chose.md`. The next task (17) is
written below. **No code follows this decision**, per its own acceptance.

### 17. Every crate that declares words, collected — and the one that is not, named

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file, needs no screen and no
machine.

Written by task 16, from a failure this repository has already had. `alo-saying`
is the one vocabulary every word a person reads is collected into, and **what it
collects is a hand-written list**: a `COLLECTED` constant and one `declare` call
per crate, kept beside the crates rather than derived from them. A crate that
declares words and is not on that list compiles, tests, ships — and says nothing
to anybody in any language.

That is not hypothetical. Task 3 records it happening: `alo-overlay` declared
nine strings and `alo-saying` collected nothing, so every sentence task 2 had
declared would have reached a real shell as a bug. It was found by a person
reading, which is the same way `docs/features.md`'s six missing promises were
found six times over — and `alo-by-hand` has already shown the answer's shape for
verbs.

- **Acceptance:** the crates that declare words are read from the workspace's own
  member list rather than from a list kept beside them, so a crate added anywhere
  is a crate this check sees; a crate that declares words and is not collected is
  refused in a sentence naming it; a crate collected that no longer declares is a
  finding too, because a dead entry is how the list stops meaning anything; and
  the refusal is tested as carefully as the answer, including a crate added with
  words nothing collects.
- **Constraint:** it says nothing to a person and declares no strings — it is a
  repository check like `alo-reconciling` and `alo-by-hand`, and `alo-saying`
  does not collect it. Nothing in `crates/alo-shell`.

**Done, 2026-09-11.** `crates/alo-collected`: the crates that declare words are
read out of the workspace's own member list — `src/words.rs` with a `pub fn
declare_into`, which twenty-three crates already follow — and **nothing in the
crate holds a crate's name**, neither the twenty-two collected nor the one that
is not. All three directions are held: a crate that declares words and nothing
collects is the finding this exists for, a name on the list that no longer
declares anything is a finding too, and a crate on both lists or twice on one is
refused because either would make the counts add up while the workspace does
not. The measurement against this repository is `declaring == collected +
apart`, on the disk the test runs on: twenty-three, twenty-two and one.

**What the shape decides, and it is the half no count carries:** the exception
is held to an *argument*, not to a name. `alo_saying::DELIBERATELY_APART` is a
new additive surface pairing the one crate outside the vocabulary — `alo-agentd`,
which is Linux and would make a vocabulary three strings shorter on a host with
no daemon, so one host would refuse a translation file the other accepted — with
the reason, and a name with a shrug beside it is refused at the same forty
characters `alo-by-hand` and `alo-reconciling` use. A standing permission to say
nothing is indistinguishable from the bug this check is for; the sentence is the
whole difference. An exception nobody needs any more is refused as well, because
it would still be standing on the day a crate of that name has words. Every
refusal is shown against a fixture, and one of them is shown against **this
repository** with a real crate taken off the real list, because a check could
pass its fixtures and never look at the disk. `docs/contracts/translations.md`
is where whoever adds the next crate meets the convention, which this makes
load-bearing rather than tidy. Two pre-existing **Windows** clippy failures
(`alo-keeping`, `alo-recounting`) were repaired in the same change because a gate
red for somebody else's reason still stops publication; six pre-existing
Windows-only `alo-recounting` test failures are named in the report and
deliberately not cut to green. Report:
`docs/autonomy/updates/every-crate-that-declares-words-collected.md`. The next
task (18) is written below. No strings are declared and `alo-saying` does not
collect this crate.

### 18. Every decision this repository points at, and the ones nobody can find

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file, needs no screen and no
machine.

Written by task 17, from the failure that family of checks keeps finding in a
new costume. `CLAUDE.md` makes the ADRs binding — *read the ADR before proposing
an alternative* — and this repository points at them constantly: `ADR 0001 §3`
in a crate's documentation, `ADR 0009` in a finding's own sentence, `ADR 0016` in
a promise's evidence. **Nothing checks that any of those pointers lands.**

Task 16 already met one edge of this and fixed exactly one pointer:
`a_promise_that_waits_on_a_decision_names_one_that_is_there` holds
`docs/autonomy/v0-01-evidence.md` to the decision it names, because *waits on a
decision* sending a reader to an ADR nobody wrote reads exactly like an answer.
Every other citation in the repository is unchecked — and an ADR reference is the
one kind of pointer a reader believes without opening, because the number looks
like a fact.

- **Acceptance:** every ADR reference in the repository's Rust and Markdown —
  read as citations rather than from a list kept beside them, so a reference
  added anywhere is one this check sees — names a decision that exists under
  `docs/decisions/`; a number with no file is refused in a sentence naming the
  citation and where it is; two files claiming one number is a finding, and so is
  a decision whose own file does not say what its status is, because an ADR
  nobody recorded as accepted or proposed is one every reader will read as
  settled; and the refusal is tested as carefully as the answer, including a
  citation of a decision nobody wrote.
- **Constraint:** it judges the *pointer*, never the argument — whether an ADR
  says what a citation claims is a reader's job and nothing mechanical reaches
  it. It says nothing to a person and declares no strings; it is a repository
  check like `alo-reconciling`, `alo-by-hand` and `alo-collected`, and
  `alo-saying` does not collect it. Nothing in `crates/alo-shell`.

**Done, 2026-09-11.** `crates/alo-citing`: the citations are read out of this
repository's own Rust and Markdown rather than a list kept beside the decisions,
so **nothing in the crate holds a decision's number or a file's name**. The
measurement on the disk the test runs on: twenty-five decisions, **1,926
references by number** and **195 by filename**, all landing. Both forms are read,
because a filename is the pointer a reader is least likely to check and the only
one that rots on its own.

**Two real pointers did not land, and both were written by people right about
the argument and wrong about the address.** ADR 0019's own `**Status:**` line
linked to a filename carrying ADR 0006's title from before it was renamed — a
true sentence with a dead address, which is the whole argument for this check in
one line. And ADR 0001 line 80 cited two of `alo-workplace`'s decisions as bare
numbers, which nothing here answers for and which a reader would look for here.
Both repaired without touching either decision's argument.

**What the shape decides:** a citation is read as *this* repository's unless the
line names the repository it points into, **before** the number — so a
neighbour's decision is named rather than excused, and the list of neighbours is
held the way `alo-collected` holds its exceptions, with one nothing cites refused
as a place a typo could hide. The unit is the line, which makes the rule
checkable and makes the same demand on writing that a reader makes: keep the name
with the number, because nobody skimming can scroll up for it. Writing
`docs/decisions/README.md` broke that rule in its first draft and was refused by
the check, ten minutes after the check existed. A **section** is deliberately not
resolved — ADR 0019 numbers its headings, ADR 0020 is cited by section and has
none, and `alo-models` cites `ADR 0004 §policy` — because resolving it would mean
imposing a convention this task had no standing to decide or producing findings
nobody could act on. And a pointer **in an example is still a pointer**: a
citation of a decision nobody wrote is refused wherever it is written, including
in a test about dead pointers, so those are assembled from a constant rather than
spelled out; this crate's own tests and three fixtures in `alo-reconciling` do
exactly that, each saying why beside it. Every refusal is shown against a
fixture, and one against **this repository**, with a real decision taken off the
real list. `docs/decisions/README.md` is where whoever writes the next decision or
the next citation meets the convention. Six pre-existing Windows-only
`alo-recounting` test failures are named in the report and deliberately not cut
to green. Report:
`docs/autonomy/updates/every-decision-this-repository-points-at.md`. The next
task (19) is written below. No strings are declared and `alo-saying` does not
collect this crate.

### 19. The four promises with no evidence, and which of them a lane can still reach

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it reads code and reports, and touches no compositor file.

Written by task 18, and it is the first task in this lane's recent run that is
not another repository check. The family is finished: `alo-reconciling` holds the
promises to evidence, `alo-by-hand` holds the verbs to a plain way,
`alo-collected` holds the words to one vocabulary, and `alo-citing` holds the
citations to decisions that exist. What none of them answers is the finding task
11 left standing and tasks 14 and 15 halved: **`docs/autonomy/v0-01-evidence.md`
records four v0.01 promises with no evidence at all**, and the ledger says each
of them needs a screen, a decision or a machine.

That sentence has been carried unexamined since it was written. Three of the four
— *copy, cut and paste*, *the GPU works on first boot*, *boots on one certified
machine* — were sorted into *needs a machine* by the audit that found them, in
one pass, while it was finding six things at once. One of them may not need one:
a promise about text moving between applications is a protocol and a clipboard
before it is a screen, and nothing in this repository has looked.

- **Acceptance:** each of the four promises is read against what this repository
  actually has — the crates, the contracts and the ADRs, named — and is written
  down as either *reachable without a screen, a decision or a machine, and here
  is the increment*, or *not, and here is precisely what it waits on*. The
  finding goes in `docs/autonomy/v0-01-evidence.md` under the promise it is
  about, so the next reader inherits the reasoning rather than the verdict; the
  reconciling gate passes on the change; and **where one is reachable, the next
  task in this plan is the increment**, written with its own acceptance.
- **Constraint:** it may not narrow `docs/features.md`, it may not contradict an
  accepted ADR, and it may not tick a promise it has not shown. If all four are
  genuinely blocked, saying so with the evidence is the finished work — and the
  plan then says what this lane does next rather than leaving the loop to read a
  blocked list as *nothing left to do*.

**Done, 2026-09-11.** The four were read one at a time against the crates, the
contracts and the decisions, and the reading is written under each promise in
`docs/autonomy/v0-01-evidence.md` rather than as a verdict beside it. **The count
stays at four** — nothing was closed and nothing was ticked — and **one of them
was in the wrong pile.**

*Copy, cut and paste* needs no screen, no machine and no decision. It was sorted
into *needs a machine* by the audit that found it, in the same pass that found
five other things, and nothing had looked since. A clipboard is a **protocol
before it is a surface**: an owner, the types it offers, and a transfer somebody
asks for — every value of which is decidable with no pixels, exactly as
`alo-overlay`, `alo-approving`, `alo-indicator` and `alo-recounting` decided
theirs. Nothing gates it either: **no agent verb touches the clipboard** (the ten
in `docs/contracts/agent-verbs.md` are six about files and four about
applications), so ADR 0001's grant model is not in its way, and ADR 0005's portal
is the sandboxed application's route to the same thing, which `docs/features.md`
puts at v0.5. It is task 20 below.

The other three are genuinely waiting, and each says what on: *the GPU works on
first boot* on a machine with a card **and** on an image with something to
accelerate — `docs/hardware.md` defines the promise in four clauses, three of
which only a machine answers, and `image/Containerfile` carries no model runtime
and no weights at all; *the agents point at the local model by default* on the
owner accepting
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md`, whose
recommendation asks for one line of the definition to be reworded; and *boots on
one certified machine* on both a machine and
`docs/decisions/0024-what-a-person-signs-in-at.md`, because `crates/alo-shell`
still has no binary to boot to.

**What the shape decides:** the ledger is now held to *saying where the work is*.
`crates/alo-reconciling/src/waiting.rs` reads what a promise waits on — a
decision under `docs/decisions/`, or a task cited **with the plan beside the
number** — and follows it to a file on the disk; a promise with no evidence at
all and no such pointer is refused, and a promise shown in part is not, because
the code it already has is where the next reader goes. The plan is required
beside the number for the reason `alo-citing` requires a filename: this
repository drives three plans, all numbered from one, and *task 12 of the
delivery plan* matches something in every one of them. A task pointer is read
only under a plan's `## Tasks`, which is how `tools/kernel-loop` reads one — it
learnt that by launching a worker at an audit section called *Implemented and
verified*. Every refusal is shown against a fixture, and the measurement is taken
against this repository: four promises with no evidence, each pointing somewhere
that exists. Report:
`docs/autonomy/updates/the-four-promises-with-no-evidence.md`. The next task (20)
is written below.

### 20. The clipboard, before there is anything to draw

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file, needs no screen and no
machine.

Written by task 19, which found this promise in the wrong pile. `docs/features.md`
promises at v0.01: **copy, cut and paste — text, images and files, across
applications**, and it is the last of the six missing promises with a route that
does not run through a screen, a decision or a machine.

The seam is the selection itself, and deliberately **not** the compositor wiring:
one client owns the selection and says which types it can give; another asks for
one of those types; the broker holds the offer and nothing else. `smithay` 0.7
already carries the protocol, and putting it into `crates/alo-shell` is the
desktop lane's — this is the value that lane would wire to, with its refusals
decided before anybody can see them.

- **Acceptance:** what is pasted is what was copied, in a type the copier
  offered, and a type that was never offered is refused in words rather than
  converted into something the copier did not say it could give; taking the
  selection **retires the previous owner's offer at once**, so a transfer against
  a stale one moves nothing — a clipboard that quietly serves the last owner's
  data is a leak between two applications and reads to everybody like a paste;
  pasting when nothing has been copied is *nothing to copy from* rather than the
  thing before; text, images and files are **offered types rather than three
  mechanisms**, so the promise's three are one shape; and every string is in the
  vocabulary `alo-saying` collects.
- **And one thing it must show it does not do:** the clipboard is not
  `alo-context`'s selection. ADR 0001 §4 offers the focused window, the
  highlighted text and the open document **at the moment of invocation and for
  that turn**; what a person has copied is neither, and a turn that read it would
  be the background reader that ADR calls a bug. A test shows a turn reaching
  nothing here.
- **Constraint:** nothing in `crates/alo-shell`, which is the desktop lane's and
  is where the wiring goes. No pixels are claimed and none are tested, and **no
  verb is added** — copy and paste is a person moving their own text between
  their own windows, and law 2's enumerated verbs are not where it belongs.

**Done, 2026-09-11.** `crates/alo-clipboard`: the broker holds **the offer and
the way back to its owner, and nothing else** — there is nowhere in it to put
anybody's data, so *what is pasted is what was copied* is not a promise kept
carefully but the only thing the crate can do. **Retiring is ownership rather
than a flag**: taking the selection drops the previous owner's `Gives`, so a
transfer against a stale handle moves nothing because there is nobody left to
ask. And a selection is named by a **serial rather than by what it looks like**,
because two identical offers from two applications are two selections and a
clipboard that matched by resemblance would serve the new owner's data to
somebody who asked the old one — a leak between two applications that reads to
everybody involved exactly like a paste. That case is measured with two offers
identical in every visible way.

Text, images and files are **one door**: a form is a media type, an offer is a
list of them, and nothing anywhere switches on which of the three is moving —
`text/uri-list` is read before the `text/` family because it is the one media
type whose own family says the wrong thing. Cut is that door with a flag and **a
cut moves once**, the offer being retired by the transfer that completes it; an
editor's cut is a copy here and the file says why, because treating it otherwise
would make cut text unpastable a second time. The five ways a paste does not
happen are five sentences, not one: *nothing has been copied* and *something
else has been copied since* send a person to two different places, and eight
strings under a new `clipboard` area are collected by `alo-saying`. **Nothing
quotes what was copied** — no preview, no count, no history, and a hand-written
`Debug` that prints the offer and never the bytes — because a clipboard is where
a password manager puts a password.

**What it had to show it does not do is shown twice.** A whole turn runs over a
real context while a password manager owns the selection and the owner is never
asked: that is today. What stops tomorrow is that **neither crate can name the
other** — the test reads both manifests off the disk, and holds this crate to
depending on no daemon, no turn, no record and no grants either. A turn cannot
reach a clipboard because there is no clipboard in scope to reach (ADR 0001 §4).
No verb was added. `docs/autonomy/v0-01-evidence.md` now stands at **three**
promises with no evidence at all, and none of the three can be closed by this
lane: two need a machine and the third needs the owner. Report:
`docs/autonomy/updates/the-clipboard-before-there-is-anything-to-draw.md`. The
next task (21) is written below. No pixels are claimed and none are tested;
drawing it is the compositor's, and *On the machine* does not move.

### 21. A translator's line, held to the rule the English is held to

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file, needs no screen and no
machine.

Written by task 20, out of the one still-owed half in
`docs/autonomy/v0-01-evidence.md` that needs no screen, no decision and no
machine. `docs/features.md` promises at v0.01 that **a person never learns the
name of anything we rented** — *they install an application, not a Flatpak; they
run a model, not Ollama* — and, in the line under it, that **it is enforced
rather than remembered**: no sentence in the one vocabulary may contain the name
of a rented component.

`crates/alo-saying/src/rented.rs` enforces it, and the ledger records exactly
what it does not reach: **a translation passes through nothing.** The check reads
the English declarations and the vocabulary they build, and a translator's file
is neither. That gap is the worse half of the promise rather than the smaller
one: `docs/contracts/translations.md` says a translation is the only file **a
person outside the organisation that owns the machine** types, it arrives in the
image, and it is checked when a process starts — so *Das Flatpak konnte nicht
installiert werden* would reach a person's screen today, in their own language,
past a rule written to stop exactly that sentence.

- **Acceptance:** a translated line naming a rented component is refused with
  the same list and the same argument the English is held to, and the refusal
  names the key and the language so whoever fixes it can find the line; what
  happens to the **rest of the file** is decided and written down, because a
  translation is somebody's donated work and throwing all of it away over one
  line is a different promise being broken; a line that is fine is unaffected,
  measured against a real translation rather than only against a fixture; and
  the refusal is tested as carefully as the answer, including a rented name that
  arrives only in the translation and not in the English beside it.
- **Constraint:** it may not narrow `docs/features.md` and it may not weaken
  `rented.rs`'s existing list to make anything pass. It says nothing new to a
  person unless the decision above needs it to, and where it does, the string is
  declared and collected like any other. Nothing in `crates/alo-shell`.

**Done, 2026-09-11.** `crates/alo-saying/src/translated.rs`: the same question
the English is asked, asked of every translated line at the one moment this
repository holds the file — when a machine loads it. Same list, same matcher
(`rented::names`, made crate-visible so the two checks cannot drift apart), and
the same argument: a finding is *what a person reading it would have to learn*,
never a banned word, and it names the file, the key and the language so whoever
fixes it can find the line. **What happens to the rest of the file is decided
and written down**: the line is left out and the language is kept, deliberately
the same rule a dropped gap is held to and no harsher, because a translation is
somebody's donated work and refusing a language over a sentence would keep one
promise by breaking another — `docs/contracts/translations.md` now says so
where it used to say *nothing here can check it*. The check runs before the
vocabulary check, so a line wrong in both ways is refused for the reason that
matters more; only the translated text is read, because the keys are the
vocabulary's and are held to the rule where they are declared. A rented name
arriving **only** in the translation — the English beside it proven clean in
the same test — is refused, which is the case the whole task is about; a real
translation of real keys from the machine's own vocabulary, on a real disk,
loses nothing. No new string: the refusal travels in `Damage`, which gained the
third kind, and keeps its English for `failing.rs`'s standing reason. The
still-owed half in `docs/autonomy/v0-01-evidence.md` is closed and what remains
owed is named. Report:
`docs/autonomy/updates/a-translators-line-held-to-the-same-rule.md`. The next
task (22) is written below.

### 22. The grants a person can see, before there is anywhere to show them

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file, needs no screen and no
machine.

Written by task 21, from the evidence ledger's entry for *Grants: pick a
folder, see what is granted, revoke it, and it expires*: a grant can be made
(task 6), kept across a sign-out, reached by the daemon and expired — and
***see what is granted* has no surface**, so there is nowhere a person can
look at the list or revoke one by hand. ADR 0001 makes visibility part of the
grant model itself: a grant a person cannot find is a grant they cannot revoke.

The seam is the value, exactly as the overlay at rest (task 3), the sentence
(task 7) and the clipboard (task 20) decided theirs: what a surface would show
is derived from the grants the machine actually keeps, and revoking through it
is the same revocation the daemon already enforces — not a second mechanism
that could disagree with the first.

- **Acceptance:** the list a surface would show is derived from the machine's
  own kept grants and from nothing else — no constructor from text, so nothing
  can show a grant the machine does not hold; revoking through it is the
  revocation `alo-capability` already enforces, shown taking effect on the
  daemon's own `permits` immediately, with the verb refused afterwards; an
  expired grant is never shown as live; *nothing granted* is a sentence rather
  than an empty list; and every string is in the vocabulary `alo-saying`
  collects, with the refusals tested beside the answers.
- **Constraint:** nothing in `crates/alo-shell`, which is where the drawing
  goes and is the desktop lane's. No pixels are claimed and none are tested,
  and no verb is added — a person looking at their own grants is not an agent
  doing something.

**Done, 2026-09-11.** `crates/alo-granted`: `Listing::of` takes the machine's
own `alo_capability::Grants` — the value the daemon's `permits` answers from,
whether granted this session or read back through `Grants::remembered` — and
there is no other door: `Seen` has no public field, no `From`, no
deserialiser and no constructor from text, with compile-fail examples that
turn adding one into a failing build. **Revoking a row is
`alo_capability::Grants::revoke` with the row's own handle and nothing beside
it** — not a second mechanism that could disagree with the first — shown
taking effect on the daemon's own `permits` at the same instant and on a real
verb refused through `Authorised::read`; a row from a list the machine has
moved past lands on nothing, changes nothing byte for byte, and says the list
was out of date. An expired grant cannot become a row at all (`Seen::of` goes
through `Grant::expires_in`), measured with the grant still on the stored
list and the daemon refusing it. *Nothing granted* is a declared sentence
that deliberately reports rather than instructs — the list is where checking
and taking away happen, and a machine asking to be granted things is not this
surface. Four strings under a new `granted` area, collected by `alo-saying`
(twenty-four collected, twenty-five declaring). Times are exposed as values,
for `alo-capability`'s own reason: formatting an expiry hardcodes a calendar
as well as a language. Report:
`docs/autonomy/updates/the-grants-a-person-can-see.md`. The next task (23) is
written below. No pixels are claimed and none are tested; drawing it is the
compositor's, and *On the machine* does not move.

### 23. A person's change to the grants reaches the file the daemon re-reads

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file, needs no screen and no
machine.

Written by task 22, which found the gap while showing revocation immediate.
The daemon's half of *a change reaches the running daemon* exists and is
tested: `alo-agentd/src/rereading.rs` answers a knock
(`alo_protocol::FromAPerson::Granted`, which deliberately carries nothing) by
re-reading `/var/lib/alo/grants.toml` whole, so a grant missing from the file
is a grant revoked. The **person's half has no owner**: nothing outside test
fixtures ever calls `alo_remembering::kept`, so a grant made through
`alo-picking` or a revocation made through `alo-granted` changes a `Grants`
in one process's memory and reaches no daemon and no next sign-in until a
surface hand-writes the composition — write the file, then knock — which is
order-sensitive glue (a knock sent before the write re-reads the old list)
of exactly the kind this repository turns into tested values.

- **Acceptance:** one value composes the person's half: after a grant or a
  revocation made through it, `alo_remembering::remembered` reads the change
  back off the disk, and the knock is sent exactly once, **after** the write
  — with the order held by shape or by test, not by comment; a write that
  fails leaves the file as it was, sends no knock, and is told in words; a
  machine with no daemon to knock is not an error — the change stands and
  applies at the next sign-in, which is the state this repository shipped
  with; and nothing new can write the grants from an agent's door, shown the
  way `rereading.rs` already shows it.
- **Constraint:** additive protocol only — `FromAPerson::Granted` exists and
  is the knock; no new message, no payload added to it. Nothing in
  `crates/alo-shell`, and every string a person reads is in the vocabulary
  `alo-saying` collects.

**Done, 2026-09-11.** `crates/alo-changing`: `Changing` is the composition as
one value, and **the order is held by shape** — both doors end in one private
function, that function is the only caller of `Knocking::knock` in the crate,
and the knock stands on the far side of `alo_remembering::kept`'s `?`, so a
knock cannot overtake the write it announces. It is measured anyway, twice: a
door of the tests' own reads the file at the moment it is knocked on, and over
a real Unix socket a thread standing where the daemon stands re-reads the file
before it answers. The change is applied to a **copy** of the list, the copy
is written whole, and only then does it replace the list the caller holds — so
a refused write leaves all three places a grant lives exactly as they were,
knocks nobody, and is told in one declared sentence whose point is that
nothing moved in either direction. A machine with no daemon is not an error:
every way the conversation at the door can fail to happen is *at the next
sign-in*, because the change already stands on the disk — except a daemon that
answered and could not read its list again, which is the one case with
something left to look at and is carried in the daemon's own sentence rather
than dressed up as either of the others. What the crate cannot do is by
construction: a grant arrives only as a sealed `alo_picking::Chosen`, a
revocation only as an `alo_granted::Seen`, and the agent's side cannot reach
any of it — `alo-agentd` does not name this crate and this crate names no
daemon, no turn and no record, held by manifest tests the way `alo-clipboard`
holds a turn out of the clipboard. No protocol change: the knock is
`FromAPerson::Granted` as it stands, and a test holds it to carrying nothing.
Two strings under a new `changing` area, collected by `alo-saying`
(twenty-five collected, twenty-six declaring). Report:
`docs/autonomy/updates/a-persons-change-reaches-the-file-the-daemon-re-reads.md`.
The next task (24) is written below.

### 24. A person's choice about their model, written where the machine reads it

**Status:** ready. **Depends on:** nothing.
**Owner:** Claude — it touches no compositor file, needs no screen and no
machine.

Written by task 23, which closed the same gap for the grants and found the
settings standing in it. `crates/alo-choosing` is the person's half of
ADR 0016 — which model answers, the weights they brought, the providers they
added, the language they read — and it **only reads**: `Settings::at` reads
the file under `$XDG_CONFIG_HOME/alo`, and nothing in this repository writes
it. The evidence ledger has said so in as many words since task 11: *a
provider is added by writing the person's own settings file by hand*. So every
choice ADR 0016 gives the person is one no surface could carry out — the same
shape task 23 started from, one file over.

The seam is the written change, deliberately not a Settings screen: a change
lands in the person's own file whole, through `alo-choosing`'s own shapes, or
it does not land at all.

- **Acceptance:** a change made through the one new value — a model chosen, a
  provider added, a language picked — reads back through `Settings::at` from
  the person's own file; what can be written is only what `alo-choosing`
  already validates, with no second parser and no constructor from text; a
  write that fails leaves the file as it was and is told in words; a file that
  is not there yet is the first choice's ordinary morning, not an error; and
  every string a person reads is in the vocabulary `alo-saying` collects, with
  the refusals tested beside the answers.
- **Constraint:** it chooses nothing for anybody — ADR 0016 stands and
  ADR 0025's reading stands: no default is written that a person did not
  choose, and nothing pre-selects. It does not touch the organisation's bound
  in `/etc/alo/agentd.toml`, which is ADR 0004's and has an owner. Whether a
  running daemon must also be *told* — the knock question — is answered by
  reading how `alo-agentd` actually takes its settings, and the answer is
  written in the report rather than assumed either way. Nothing in
  `crates/alo-shell`.

**Done, 2026-09-11.** `alo_choosing::Choosing` is the written change, and it is
in `alo-choosing` rather than in a crate of its own **because the shape on the
disk must be declared once**: `crate::written`'s types now carry both
directions, so a key renamed is renamed for the reader and the writer in one
keystroke and there is no second declaration for a release to move one of. That
is what *no second parser* comes to when it is a shape rather than a rule
somebody keeps.

**What the shape decides, and it is the half a round of care would not have
bought: nothing is written that this alo OS does not read back as the same
settings.** `crate::writing` serialises, parses its own output through its own
reader, and refuses unless what comes back is what went in — before a byte
reaches the disk. It exists for a quiet failure rather than a loud one:
`alo_models::Provider` has public fields and this file has nowhere for two of
them, so a writer that simply dropped the list of model names a provider offers,
or a credential kept anywhere but the derived `provider/<name>`, would hand a
settings panel an `Ok(())` for a provider the machine afterwards describes
differently and nobody would ever be told. Both are measured refusing. It costs
one parse of a few hundred bytes at the moment somebody clicks something, and it
makes *reads back through `Settings::at`* a property of the code rather than of
the tests somebody happened to write.

Every door takes a value another crate has already checked and **there is no
door that takes text**; the one rule needing two halves of a file at once —
`Settings::of`'s, that a choice into a list this person keeps must name
something on it — is asked at the moment somebody clicks, so the two ways a
settings file can contradict itself are refused naming what was named rather
than becoming a file the same machine refuses whole at the next question.
Opening settings **writes nothing** — not a file, not a folder, not a `format`
line — which is ADR 0016 and ADR 0025's reading kept where it is easiest to
lose; and settings that are there and do not read are **refused rather than
replaced**, because a surface that read a typo as *nothing chosen* and then
saved would take away the keystroke about to fix it. The folder **is** made
here, unlike `alo-remembering`'s, and the file says why: `/var/lib/alo` is the
image's, `$XDG_CONFIG_HOME/alo` is the person's and the first choice anybody
makes is exactly the moment it does not exist.

**The knock question is answered by reading the daemon, and the answer is that
there is nothing to knock about.** `alo-agentd` reads this file *once a turn, at
the first question of that turn* — `Questions::a_new_turn` forgets,
`what_answers` looks — which it measures itself in
`a_new_turn_reads_the_file_the_person_has_just_written`, so a change written
here is in force for the next question anybody asks. That is the opposite of the
grants, which a daemon holds from start-up and must be told about; a knock here
would be a message telling a service to do what it already does. What this crate
is held to instead is that it *could not* knock, and that the daemon cannot
write: no wire words, no daemon, no turn, no record in the manifest, and
`crates/alo-agentd/src` read off the disk and shown to name `Choosing` nowhere.

Four strings under a new `choosing.change.*` family, collected by `alo-saying`,
and they end in a clause the ten before them deliberately do not: *nothing in
your settings has been changed*, rather than *nothing in the file has been
used*. A person who had just clicked something and read the second would
reasonably conclude their machine had forgotten what they chose last month. Two
of the six refusals are `alo-models`' own, carried rather than reworded, because
the two lists a settings file holds are that crate's and two accounts of one
moment is one too many. No verb was added and none could be. The evidence
ledger's *add your own provider in Settings* no longer owes *added by hand*;
what it owes is a surface. Report:
`docs/autonomy/updates/a-persons-choice-written-where-the-machine-reads-it.md`.
The next task (25) was already written below.

### 25. The image becomes a disk a machine can actually boot

**Status:** ready. **Depends on:** nothing in this lane.

Every promise in `docs/autonomy/v0-01-evidence.md` that says *still owed: no
machine has ever* waits on one missing step, and it is not a machine. It is that
**nothing turns the image into something a machine boots.** `image/Containerfile`
builds a bootc OCI image (ADR 0011) and `crates/alo-image` holds it to every
promise it makes, but there is no path from that image to a disk, so the only
thing anybody has ever seen is a test saying the recipe is right.

The owner's machine is a Windows 11 Pro host with Hyper-V, and the certified
laptop has not been bought. A virtual machine is **not** the hardware acceptance
in phase 8 and this task may not claim to be: a virtual GPU is not *the GPU works
on first boot*, and tame virtual firmware is not *firmware to sign-in*. What it
is, is the difference between a repository that believes it boots and one that
has watched it — and that is worth having before a laptop is bought rather than
after.

- **Acceptance:** one documented command turns the built image into a bootable
  disk image, and `docs/booting.md` says what it produces, what it needs
  installed, and how to attach it to a Hyper-V generation-2 virtual machine;
  the disk is built by a pinned upstream tool the way the runtime is (`bootc
  install to-disk` or `bootc-image-builder` — whichever the pinned base
  supports, named with its version and the reason), never a hand-rolled
  partitioner; `crates/alo-image` holds the recipe's boot-relevant promises the
  way it holds the rest — a check per promise and a twin that breaks one line,
  including that the disk's firmware mode matches what the document tells a
  person to select; and what the disk **cannot** show is written in the same
  document, by name, so nobody quotes a virtual machine as hardware acceptance.
- **Constraint:** no second image recipe — the disk is built from the image
  `image/Containerfile` already produces, and a divergence between them is the
  defect this task exists to prevent. Nothing in `crates/alo-shell`; there is
  still nothing to draw and the machine boots to a text console, which is the
  honest state and is what the document says it will look like. If the pinned
  base cannot produce a disk without an engine change, that is an ADR handed
  over as this task, in the shape ADR 0024 and ADR 0025 used — never a patched
  engine.
- **Owner:** Claude — it touches no compositor file and needs no hardware. The
  *running* of it on the owner's Hyper-V is the owner's, with the document this
  task writes in hand.

**Done, 2026-09-11.** `docs/booting.md`, the three `alo.disk.*` labels in
`image/Containerfile`, and `crates/alo-image`'s `disk.rs` and `booting.rs`. The
tool is the pinned base's own `bootc install to-disk`, so the version of the
partitioner is `THE_BASE`'s digest and there is no second thing to pin;
`crates/alo-image` holds the document and the recipe to each other, firmware
against the generation a person is told to select included. Nobody has run it
yet: the disk is a document and a check, not a machine anybody has watched.
`docs/autonomy/updates/the-image-becomes-a-bootable-disk.md`.

### 26. The one privileged thing that turns a correct password into a session

**Status:** ready. **Depends on:** nothing — ADR 0024's measurement is taken.

Task 13 is the first screen, and it is the desktop lane's because it is drawing.
**This is the half of it that is not drawing**, separated out so the critical
path does not wait for a lane that is away: the small privileged component
ADR 0024 priced and accepted, which takes a uid that `alo-accounts` has
*already* authenticated and asks `systemd-logind` to open that person's session —
the session `alo-agentd.service` is bound to and that nothing on the image can
currently cause.

The measurement is done and is in the ADR: `CreateSession` answers root with
`Invalid leader PID` — the call authorised, only its contents refused — and
uid 1000 with `Access denied`. So the boundary is privilege, not `pam_systemd`,
and no PAM module and no `unsafe` exemption is owed. What remains is to build the
thing and to hold it to the terms ADR 0018's loader is held to.

- **Acceptance:** it opens a session for a uid and can do **nothing else** — no
  password reaches it, no name, no path, and it cannot be asked to open a session
  for a uid the caller has not authenticated; its capability set is a
  `crates/alo-image` check beside the loader's, with a twin that breaks one line;
  a request it refuses is refused in words `alo-saying` collects; and the
  measurement is taken **again on the pinned Fedora base** rather than on the
  development machine's Ubuntu, with the answer written into `docs/quirks.md` —
  ADR 0024 says in as many words that this check is owed before any box is
  ticked, and recording the WSL answer as the image's would be the guessing that
  paragraph exists to prevent.
- **Constraint:** nothing in `crates/alo-shell` — that is task 13's and the
  desktop lane's, and this must not draw, start a compositor, or decide what a
  person sees. It does not authenticate: `alo-accounts` does that and already
  does it, and a component that could do both would be ADR 0018's one privileged
  component argument thrown away. If the pinned base answers differently from the
  development machine, that finding is the deliverable and an ADR is how it is
  recorded — never a workaround.

**Done, 2026-09-11.** `crates/alo-sessiond` is the opener, and **the price ADR
0024 priced came in lower than it was quoted: the second privileged component
holds no capability at all.** The loader holds two and argues for them; this
holds none, because what `logind` decides `CreateSession` on is a **uid**, and
`User=root` with both capability lines present and empty is a process that can
open a session and cannot do the things capabilities buy. Even the one
filesystem thing it does — handing its door to the greeter's group — needs no
`CAP_CHOWN`, because the group it changes to is the group its own unit put it
in. That is a `crates/alo-image` check beside the loader's, with a twin per
promise: seven of them, breaking one line each.

**The measurement was taken again on the pinned base and it found something.**
Fedora 42, systemd 257, the alo OS image itself under `podman --systemd=always`:
root is refused only on the leader PID and uid 1000 is refused outright, so
Option B is buildable on the machine alo OS ships. What was new is that the two
machines *word* it differently — `Leader PID is not valid` against `Invalid
leader PID`, and an unprivileged caller turned away by the **bus policy** rather
than by `logind`, with a completely different sentence and the same D-Bus error
name. So the crate carries the error **name** beside the sentence and decides on
the name; the sentence is for a service log. `docs/quirks.md` has all three
measurements and ADR 0024 records that the check it said it owed is paid.

**What makes *it cannot be asked to open a session for a uid the caller has not
authenticated* true across a process boundary** is that the number on the wire
is checked against the accounts file the surface authenticated against, before
anything else is asked — so the set of numbers this component will ever open a
session for is exactly the set of people this machine has, and a knock from
anything but the greeter's group is refused before even that. No password
reaches it, no name and no path: `Knock` holds one `u32` and a line with a
second thing on it is refused rather than read leniently, which is a test rather
than a rule somebody keeps. The wire is this crate's own and deliberately not
`alo-protocol` — a sign-in on the agent's door would put every message an agent
can send inside a privileged process's parser.

Four strings under a new `signing-in.*` area, collected by `alo-saying`
(twenty-six collected, twenty-seven declaring). The image gains the unit, the
binary in `libexec` beside the loader, and the greeter login the door is handed
to — the surface that will run as it is task 13's, and nothing here draws.
Report:
`docs/autonomy/updates/the-one-privileged-thing-that-opens-a-session.md`.
The next task (27) is written below.

**Refused once before it was published, and not for anything in it.** The
whole-workspace gate reported `no method named numbers found for struct
Accounts` for a method that was in the tree, twice; the same tree checked,
linted and per-crate tested clean. It was a cached `alo-accounts` unit that
Cargo held to be fresh and that predated the method, and `cargo clean -p
alo-accounts` was the whole of the repair — no line of the component changed.
The account of it, and what it means for reading a refusal that arrives twice,
is `docs/autonomy/updates/a-gate-refusal-that-was-not-in-the-change.md` and a
paragraph in `tools/kernel-loop/src/gates.rs`, which is the file that prints
the message.

### 27. The sign-in surface's half of the door

**Status:** ready. **Depends on:** task 26, which is done. Not on task 13: what
is described here is what a screen calls, not the screen.
**Owner:** Claude — it touches no compositor file, needs no screen and no
machine.

Written by task 26, which built the privileged half and found the other half
unowned — the same shape task 22 found for the grants and task 23 closed one
file over. `/run/alo-sessiond/sign-in.sock` is open, `alo_sessiond::Knock` and
`Answered` are the line, `alo-accounts` decides who is here, and **nothing in
this repository composes them.** A surface has to authenticate, then knock, then
render what comes back — which is order-sensitive glue of exactly the kind this
repository turns into a tested value, and a surface that authenticated and
forgot to knock is a screen that takes a correct password and does nothing.

- **Acceptance:** one value composes the person's half: an
  `alo_accounts::Session` — which cannot exist unless a password verified *and*
  the machine description agreed about the number — becomes exactly one knock,
  and a knock cannot be made from anything else, held by shape rather than by
  comment; what comes back is a session or one of `alo-saying`'s sentences,
  looked up and rendered in the person's own language rather than handed on as a
  key; **every way the conversation can fail to happen** — no service listening,
  a connection that drops, an answer that never comes, an answer that is not one
  — is told in words rather than as silence, because somebody standing at a
  sign-in that did nothing cannot tell a refusal from a machine that is not
  running; and the refusals are tested beside the answer, over a real socket.
- **Constraint:** nothing in `crates/alo-shell` — the drawing is task 13's and
  the desktop lane's, and this must not draw. No second authenticator and no
  second reader of the wire: `alo-accounts` decides who is here and
  `alo-sessiond` declares the line. Additive only — no new message and no field
  added to `Knock`, which is the whole of what keeps a password off that wire.
  Every string a person reads is in the vocabulary `alo-saying` collects.

**Done, 2026-09-11 — this is the same seam as task 28 and was built with it.**
Task 26's report wrote this task; a later change wrote task 28 for the same
work under the name the loop then sent a worker at, and for a day this plan
had **two sections numbered 27**, which is a plan
`tools/kernel-loop/src/plan.rs` refuses to read as tasks numbered from one in
order. The numbering is repaired here — this is 27, the greeter is 28, and
recovering a parked task is 29 — and the two sections are answered by one
crate rather than pretended to be two pieces of work. `crates/alo-greeting`
keeps every line of the acceptance above: `Greeting::signs_in` is the one
composition, the one call to `Knock::on_behalf_of` is a private function
taking an `alo_accounts::Session`, what comes back off the wire is looked up
in the reader's own language, and the four ways the conversation can fail to
happen are four values, two sentences and no silence — measured over a real
socket with `alo-sessiond`'s own `Listening` on the far side of it. Report:
`docs/autonomy/updates/what-the-greeter-does.md`.

### 28. What the greeter does, before there is anything to draw

**Status:** ready. **Depends on:** 26.

Task 26 built the door and nothing knocks at it. Task 13 is the screen and it is
the desktop lane's, because drawing is theirs. **Between them is everything the
greeter does that is not drawing**, and this is that: read a name and a password
somebody typed, ask `alo-accounts` whether it is right, and on yes knock at
`alo-sessiond` for that person's uid. It is the same split task 26 used, and for
the same reason — the lane that draws is away until the 16th, and the logic
underneath does not need pixels to be finished or to be tested.

It is a value with no surface, in the shape `crates/alo-approving` and
`crates/alo-overlay` took theirs: what a greeter *is* asked, what it answers,
and every refusal decided — so that when the screen arrives it draws a thing
that already works rather than growing the logic inside a paint routine.

- **Acceptance:** a name and password that match an account produce a knock for
  that account's uid and nothing else; a wrong password and an unknown name are
  refused **identically**, in the words `alo-accounts` already has and in the
  same time, so neither the text nor the clock says which it was; a machine with
  no store answers *make an account* rather than a sign-in, which is ADR 0024's
  first-boot sentence; nothing anywhere in it holds, logs or returns the
  password, tested by a check that reads the crate for it the way
  `alo-saying`'s rented check reads for names; and every string a person would
  read is in the vocabulary `alo-saying` collects.
- **Constraint:** nothing in `crates/alo-shell` and no pixels — no window, no
  font, no layout, and no guess about what the screen looks like. It opens no
  session itself: it knocks, and `alo-sessiond` decides, because a greeter that
  could open one would be the second privileged component growing a third job.
  It authenticates nothing itself either — `alo-accounts` does that and already
  does it.

**Done, 2026-09-11.** `crates/alo-greeting`: `Greeting` is everything the
greeter does that is not drawing, and **the order is held by shape** —
`signs_in` is the only door into the crate, the only place `alo-accounts` is
asked anything, and the only place a door is reached; the one call to
`alo_sessiond::Knock::on_behalf_of` anywhere here is inside a private
`for_whom(session: &Session)`, so a knock cannot be made from a number,
from text, or from a name that did not verify. There is no method that
knocks without authenticating and none that authenticates and forgets to
knock, which is the failure the task exists to prevent: a screen that takes
a correct password and does nothing.

**A wrong password and an unknown name are refused identically, and the
composition is where that promise would have been lost.** Both return
`alo-accounts`' own one refusal, carried rather than reworded, before
anything touches a socket — so neither costs a connection the other does
not, and the timing is measured here as well as one crate down, because
knocking for a known name and not for an unknown one would enumerate this
machine's accounts over a wire whatever the sentence said. A real socket is
shown untouched for a wrong password.

**Nothing anywhere in it holds, logs or returns the password**, and that is
read out of the crate rather than asserted: comments and string literals are
taken out, and in what is left the identifier appears exactly twice — the
parameter of the one method that takes one, and the line that hands it to
`alo-accounts`. Every field the crate declares is held to a list of five,
because a field called `secret` would pass an identifier check and be the
same bug; `Greeting`'s `Debug` is hand-written so a panic on a sign-in
screen cannot print the stored hashes. **A machine with no store answers
*make an account*** (ADR 0024's first-boot sentence), while a store that is
there and will not be believed is neither state but a refusal, because a
password box on that machine asks for a keystroke nothing can check. Four
strings under a new `greeting` area, collected by `alo-saying`
(twenty-seven collected, twenty-eight declaring); everything a person reads
about *who they are* is `alo-accounts`' and `alo-sessiond`'s own, looked up
here in the reader's language because a greeter with its own wording for a
refused sign-in is a machine with two accounts of one moment. Task 27 above
is the same seam and is marked done with this, and the duplicate numbering
that made this plan unreadable to its own loop is repaired. Report:
`docs/autonomy/updates/what-the-greeter-does.md`. The next task (29) was
already written below. No pixels are claimed and none are tested.

### 29. Recovering a parked task, as a command rather than as a memory

**Status:** ready. **Depends on:** nothing.

Parking works: a task that fails its gates goes to a branch, nothing is
discarded, and the run carries on. **Picking one back up does not**, and on
2026-09-11 that cost more than the parking saved. Recovering a task by hand with
`git restore --source=<branch> -- .` reverts the *whole tree* to that branch,
which silently undid two published tasks the first time and would have undone
two more the second. Both times the supervisor's *these are changed and no task
named them* check caught it before anything reached `main`; neither time did
anything in this repository stop the mistake being made again ten minutes later.

The right recovery is knowable and mechanical: restore the files the parked
task's own handoff names, and for a file another task has since changed —
a plan, the evidence ledger — apply that task's **diff** rather than its
**version**.

- **Acceptance:** one subcommand takes a parked branch and leaves the working
  tree holding exactly that task's work on top of today's `main`, with its
  handoff back in place ready to publish; a file the task named and nobody else
  has touched comes back whole; a file both touched is merged by applying the
  task's own diff, and a conflict is **reported and left** rather than resolved
  by preference; a branch whose handoff names files the task did not change, or
  that is not a parked branch at all, is refused in words; and the refusals are
  tested beside the recoveries.
- **Constraint:** it restores and never publishes — gating stays where it is, and
  a recovery that published would be a road around the gates. It deletes no
  branch, so a recovery that goes wrong can be done again. Nothing in it may
  reset, clean or check out the whole tree: that is the defect, and a
  `git checkout -- .` anywhere in the implementation is the thing the test
  should refuse.

**Done, 2026-09-11.** `alo-kernel-loop recover <branch>`, in
`tools/kernel-loop/src/recovering.rs`, with the git it needs added to
`repository.rs` — which is where every git this loop can run already lives, so
that what it can do to a checkout stays one readable list. It reads the parked
branch's **own handoff** to know what to bring back, restores a file nobody has
published over whole, applies the task's **diff** to a file somebody has, leaves
a conflict with its markers in it and says which file, and puts the handoff back
where the loop looks for one. It refuses a branch that is not a parked branch,
a parked name nobody has, a handoff already waiting that it would write over, a
branch with nothing on it, and a handoff naming files its own branch never
changed. Seven of the ten tests build a real repository, park a real task,
publish a real commit over it and read the tree back; the defect's own signature
— *another task's published file is still there afterwards* — is an assertion
rather than a comment. Report:
`docs/autonomy/updates/recovering-a-parked-task.md`. The next task (30) is
written below.

**Renumbered from 28 in the same change, and the plan was failing a test.** Two
sections were numbered 27 — the sign-in surface's half of the door, and the
greeter's logic — so `plan::tests::every_plan_this_repository_drives_holds_only_tasks`,
which holds every plan this repository drives to *numbered from one, in order*,
had been red since the second of them was written, and with it the whole of
`cargo test -p alo-kernel-loop`. The greeter is 28 and this is 29. No task's
title, dependency, status or content moved, and the loop selects by title, so
nothing already handed over means anything different. *The gates build where
there is room* arrived on `main` while this was being written, numbered 29 on
the assumption that this task was 28; it is 30 below, for the same reason and by
the same one-line change. This task writes no task after it, because the plan
already names one.

### 30. The gates build where there is room, not where there is none

**Status:** ready. **Depends on:** nothing.

Every gate builds into `target/` beside the checkout, which is on the **C: drive
of a Windows host that is 98% full**, while the WSL filesystem the gates actually
run in has **868 GB free and holds almost nothing** (measured 2026-09-11: 4.8 GB
in one lane's `target/`, 7.2 GB in the other's, and 32 MB in the WSL-side
directories). So two lanes compete for the scarcest resource on the machine,
for no reason.

It has cost real work. The gates refuse below 12 GiB free — correctly, because a
linker that cannot open a file reads like a broken change and is not one — and
that refusal parked a finished task today. The answer has been to clean a
lane's build directory by hand, which trades an hour of recompilation for space
that was never scarce where the build was running.

- **Acceptance:** the gates build under a directory on the distribution's own
  filesystem, one per checkout so two lanes never share a `target` (sharing is a
  lock, and a lock is a lane waiting); the supervisor says where it is building
  the first time a run starts, because a build directory nobody can find is one
  nobody cleans; the free-space check measures **the filesystem the build will
  actually use** rather than the one the checkout is on, and its sentence names
  that filesystem; and a machine where that directory cannot be made falls back
  to today's behaviour with a line saying so rather than failing.
- **Constraint:** nothing about what the gates *are* changes — same gates, same
  order, same refusals. It may not delete anybody's build directory, including
  the old ones: a supervisor that tidied up could throw away an afternoon of
  compilation belonging to a lane that is merely idle. Say where the old ones
  are and let a person decide.

**Done, 2026-09-11.** `tools/kernel-loop/src/where_it_builds.rs` owns both halves
of the question — *which directory* and *how much room is in it* — because they
were only ever one question answered in two places. Each checkout builds in
`$HOME/alo-builds/<its name>-<fingerprint of its path>`, made once per run and
handed to every gate after that, so two lanes on one machine cannot land on the
same `target` however alike their checkouts are named. The reserve is then asked
of **that** directory rather than of `.`, and its refusal names the filesystem it
measured: on this machine the answer went from *4 GB free on a Windows drive the
build never writes to* to *935 GB free on `/`*, which is the refusal that parked
a finished task. A reading it cannot understand is still a refusal, because a
reserve that passed whenever it failed to measure anything would not be one. A
machine where the directory cannot be made falls back to Cargo's own `target/`
and says so in a line rather than failing. Nothing removes a build directory:
the old ones — `target`, `$HOME/target-claude`, `$HOME/alo-os-target` — are named
when a run starts, with `du -sh` and the sentence that whether any of them goes is
a person's decision, and `nothing_here_can_remove_a_build_directory` reads the
source to keep it that way. The cost paid knowingly is one cold rebuild, because
the directory this lane had was named for the lane rather than for the checkout.
Report: `docs/autonomy/updates/the-gates-build-where-there-is-room.md`. The next
task (31) is written below.

### 31. The weights a machine arrives with

**Status:** ready. **Depends on:** nothing.

ADR 0025 was accepted as Option D on 2026-09-11, and what it took on is heavier
than what it gave up: **a model on the disk of every machine we ship, sized for
that machine** (ADR 0007). Two of the three things that stood in front of that
are now built. The pinned model runtime is on the image — `THE_RUNTIME`, its
digest checked before anything is unpacked, read back by
`alo_image::TheRuntime` — and the setup flow is `crates/alo-setting-up`, four
choices with the local one first and nothing pre-selected. The third is
untouched: `image/Containerfile` carries **no weights at all**, so no machine
this repository builds arrives able to run anything, and
`docs/autonomy/v0-01-evidence.md` records that against *the local model is what
the machine arrives ready to run*.

The open question ADR 0025 left is a decision inside this work rather than a
blocker in front of it: whether the weights ride on the certified image or are
fetched at setup. It is not free either way — a machine that fetches at setup
has not arrived ready when it is offline at setup — and whichever is chosen, the
recipe has to say *which model*, for *which machine*, pinned by digest exactly as
the runtime is.

- **Acceptance:** the recipe declares the weights a certified machine arrives
  with — the model, the quantisation and the digest — sized by what
  `alo_models::Catalogue`'s own measurement says a machine of that class can
  actually drive rather than by a publisher's claim; `alo-image` reads that
  declaration as it reads the runtime's, and `everything_wrong_with` refuses a
  recipe whose weights are absent, unpinned, unchecked before unpacking, or name
  a model the catalogue holds no measurement for, each as a `Wrong` naming the
  decision it breaks; the choice between riding on the image and being fetched at
  setup is **made**, in the recipe and in a sentence in the report saying what it
  costs the other way; and the entry in `docs/autonomy/v0-01-evidence.md` is
  rewritten to say exactly what is now shown and what still waits.
- **Constraint:** it may not tick *arrives ready to run* — that waits on an image
  that boots with the weights on a machine, and this lane has no machine. It may
  not patch the runtime or the base (ADR 0011): a model is configuration here,
  never a source change. It may not name a model `alo-driving` has no grade for,
  because *measured by us, not claimed by the publisher* is the promise directly
  above this one in `docs/features.md`. And it downloads nothing during a gate:
  what is tested is the declaration and its refusals, not a multi-gigabyte fetch
  on a build machine.

**Done, 2026-09-11.** The recipe says which model a machine arrives with —
`phi-3-mini-instruct`, `Q4_K_M`, the publisher's own GGUF at one pinned revision,
a whole sha256 checked in a step of its own **before anything reads the file**,
and the runtime's model store imported at build time and landed at
`/usr/share/alo/models/`. `crates/alo-image/src/weights.rs` reads that
declaration as `runtime.rs` reads the runtime's, on a Containerfile reader both
of them now share (`recipe.rs`), and `everything_wrong_with` refuses eight ways
it can go wrong: weights absent, fetched from a name that moves, unverified or
verified too late, unnamed, a model the catalogue has never heard of, one nobody
put to `alo-driving`, an artefact the catalogue does not state, one larger than
the 16 GB laptop `docs/hardware.md` certifies first, and one whose licence was
not ours to hand on — because **carrying weights in an image is redistributing
them**, which is why the model is MIT and why a conditioned licence is now a
refusal rather than a habit. The carry-or-fetch question ADR 0025 left open is
**decided: carried**, since a machine that fetches at setup has not arrived ready
when it is offline at setup; `docs/quirks.md` has that measurement, including the
half of it that came out negative — *the smallest catalogued model that clears
the verb-driving bar* does not exist, every measured entry is graded `rarely`, so
what a machine arrives able to do is load and answer with a local model rather
than be handed an agent turn. The evidence entry is rewritten to say exactly
that, and *arrives ready to run* stays owed: no machine has booted this image and
nothing yet starts the runtime. Report:
`docs/autonomy/updates/the-weights-a-machine-arrives-with.md`. The next task (32)
is written below.

### 32. The one thing that serves the model, and what it may reach

**Status:** ready. **Depends on:** nothing.

The image now carries a model runtime and the weights it would load, and
**nothing starts either of them**. A machine built from this recipe boots with
2.23 GiB of model on its disk and no process serving it, so `alo-models` reaches
`http://127.0.0.1:11434` and finds nothing there — which is the same answer a
machine with no model at all gives, and *the local model is what the machine
arrives ready to run* cannot be shown by a disk.

It was left out of task 31 deliberately and the reason is the shape of this one:
a unit is not a `COPY` line's worth of decision. **Which login it runs as** —
not the person, whose session comes and goes, and not the agent, which ADR 0001
§2 spends its length keeping authority away from. **Which group may reach its
socket**, which is the whole of who on the machine may ask a model anything.
**What it may reach off the machine**, which is law 1: a model runtime that
fetches a model is an egress an agent caused, and one that phones home on start
is an egress nobody asked for on a machine sold on sovereignty. And **what it is
pointed at**, since the weights are in a directory of ours rather than the
runtime's default.

- **Acceptance:** the image starts the model runtime as a login of its own that
  the image makes, holding no capability and saying so in both lines the way
  `alo-agentd.service` does; its store is the directory the weights landed in and
  nothing else; it answers on the loopback address `alo-models` already spells
  and is reachable by exactly one group, which is neither the person's nor the
  agent's; **it makes no network connection of its own at start** — no update
  check, no telemetry, no registry — and the unit says so in settings a person
  can read rather than in a comment; and `crates/alo-image` checks every one of
  those beside the others, each as a `Wrong` naming the decision it breaks, with
  the twin that breaks one line of a copy of the image and is caught.
- **Constraint:** it may not tick *arrives ready to run* either — that still
  waits on a machine, and this lane has none. It may not give the runtime a
  capability or root to make something work; if the pinned runtime needs one, the
  decision is an ADR rather than a line in a unit. It configures the engine and
  never patches it (ADR 0011): every setting is an environment variable or a
  flag upstream documents. And the egress claim is **tested, not asserted** — a
  unit that merely names a restriction is not the same as a machine that was
  watched making no connection, so say in the report which of the two this is.

**Done, 2026-09-11.** `image/usr/lib/systemd/system/alo-modeld.service` is the
one thing that serves the model, named for what it does rather than for what we
rented to do it. A login and a group of its own — `alo-model`, 60991, asserted in
the build the way the other four numbers are — which is **not the person**, whose
session comes and goes, and **not the agent**, which ADR 0001 §2 and §5 keep
authority and identity away from and which is the last login to lend anything to
a process that reads every question put to this machine. It holds nothing and
both lines say so, because serving a model needs no privilege at all and ADR
0018's argument has to be said in every unit or it is said in none. Its store is
the directory the weights landed in, which matters more than it reads: the
runtime's default is a home directory this image does not make, so a unit that
said nothing would serve nothing off a machine carrying 2.23 GiB of model and
look from the outside exactly like a machine nobody put one on. It answers at the
one address `alo-models` knocks at, **read off `alo_models::ollama::DEFAULT_ENDPOINT`
rather than spelled twice** (ADR 0019), and it reaches nothing off this machine
— `IPAddressDeny=any` under an allow list naming this machine alone, which is a
kernel-side filter on its own control group rather than a comment. Eleven tests,
ten of them breaking one line of a copy of the real image; the two worth naming
are `User=alo`, the edit that looks like tidying up, and one word in
`OLLAMA_HOST` that would offer this machine's model to whatever network it is
plugged into with a dark egress indicator, because nothing left.

**What the acceptance asked for and the kernel cannot give, and that is the half
worth reading.** *Reachable by exactly one group* is not a property a loopback
TCP port can have. Every door alo OS has decided who may knock at so far is a
Unix socket — `/run/alo/<uid>` (ADR 0017), `/run/alo-sessiond` (ADR 0024) — and
both are decided by a `Group=` line and a mode because the filesystem carries an
owner and a mode for a socket and the kernel checks them on `connect(2)`. **A TCP
socket carries neither**, no systemd directive gates a listening port by uid, and
the pinned runtime offers no Unix socket that ADR 0011 would let us add. So
`Group=alo-model` says who **answers** and cannot say who may **ask**, and a
check written as though it did would be a green test standing where a boundary is
not. The unit decides everything a unit can decide and each of those is checked;
`docs/quirks.md` carries the finding so nobody reads that line as the sentence
the other two are; and
`docs/decisions/0027-who-may-ask-the-model-anything.md` is the decision,
**proposed** — what is at stake (not a grant boundary, since no verb touches the
runtime; not an egress, since nothing leaves; the owner's compute today and, at
v0.5, an ADR 0005 sandboxed application reaching the model around the portal by
opening a socket to 127.0.0.1), with a door of ours rejected as a component in
the hottest path bought for an obstacle, a shared network namespace rejected
because whoever joins it has only it and the process that would join is the one
that talks to hosted providers, and a rule in the boundary this machine already
loads named as the real answer and as a second programme on the one privileged
component, so its own ADR at its own time. **The egress claim is a setting read,
not a machine watched** — nothing in this lane has booted this image — and
*arrives ready to run* does not move. `docs/features.md` was not touched. Report:
`docs/autonomy/updates/the-one-thing-that-serves-the-model.md`. The next task
(33) is written below.

### 33. A machine that was watched, rather than a recipe that was read

**Status:** ready. **Depends on:** nothing, on a machine with a container
runtime. **Owner:** Claude — it touches no compositor file and needs no screen.

Written by task 32, from the sentence every image task since 27 has had to write
at the end of its report: **nothing in this lane has ever built this image.**
`crates/alo-image` reads `image/`'s declarations and holds them to every promise
in `docs/` — four units, five logins, two directories, a pinned runtime, pinned
weights, a disk declaration and a document — and every one of those is a
*declaration*. `docs/booting.md` says how the image becomes a disk and
`ROADMAP.md` keeps that half of the line empty, correctly, because *an image that
builds is not an image that boots*.

But there is a step between *nobody read it wrong* and *a machine came up*, and
this lane has never taken it: **`docker build -f image/Containerfile` has not
been run since the weights and the serving unit were added.** The recipe now
fetches 2.23 GiB, runs the pinned runtime inside a build stage to import it, and
asserts five login numbers against a base that allocates downward — three things
that cannot be checked by reading and that fail as a red build rather than as a
machine anybody has to debug. The first one of those already went wrong once:
`systemd-sysusers` quietly put alo OS's agent in the resolver's group, and it was
found by building.

- **Acceptance:** the recipe is built, from this repository, and the result is
  reported honestly — the build's own assertions passing (the five numbers,
  `bootc container lint`, the model store existing), what it cost in time and
  bytes, and what went wrong. **A failure is the finished work**, written into
  `docs/quirks.md` with the version and the date if it is an engine behaving
  unlike its manual, and into the plan as the next task if it is ours. It may
  not be claimed as a boot: the machine half of `ROADMAP.md`'s image line stays
  empty either way, and **no *On the machine* box moves.**
- **Constraint:** it changes no decision to make a build pass. A recipe that
  fails because a promise in `docs/` is expensive is a finding to write down, not
  a line to soften — and if the only way through is an unpinned artefact, a
  capability, a widened grant or a patched engine, the ADR is the work and the
  build waits on it. It downloads what the recipe downloads and nothing else.
  Nothing in `crates/alo-shell`.

**Done, 2026-09-11.** The recipe was built — `podman build -f
image/Containerfile -t alo-os:dev .`, the command `docs/booting.md` gives, on
Ubuntu 26.04 under WSL2 with nothing of ours cached — and it **succeeded**: 27
minutes 29 seconds, 8.38 GiB of image, ≈23 GiB of disk, 2.28 GiB of weights
fetched and held to their digest. The assertion the plan worried most about is
the one that came out cleanest: all five numbers were created exactly as asked,
with no `Suggested user ID … already used` line anywhere, so the move to 60989+
after the resolver-group incident **holds on this base**. `bootc container lint`
said `Checks passed: 13` — and `Checks skipped: 1`, which bootc 1.15.1 will not
name, so the report quotes both counts rather than the flattering one. The model
store is on the image, and the runtime started out of it as `alo-model` lists
`phi3:3.8b-mini-4k-instruct-q4_K_M` off it.

**Two findings, and the second is the one worth the day.** The image carries the
weights **twice** — `ollama create` copies the source GGUF into the store under
its own digest and then writes a second blob of the same length that the manifest
actually names, so 2.23 GiB of the 8.38 GiB is referenced by nothing and sits on
a read-only `/usr` where nothing can ever prune it. And the pinned runtime
**phones home**: two HTTPS requests to `ollama.com` within eight milliseconds of
starting, before anybody asks it anything, retried for as long as it is up. Task
32 wrote `IPAddressDeny=any` on that suspicion; it is now a measurement rather
than a suspicion, and the report says plainly that one of those requests was
*answered* during an inspection run with a network, which is an egress this lane
caused and had said it would not. Both are in `docs/quirks.md` with versions and
dates, with a third — `/root` on an ostree base is a symlink to a `var/roothome`
that does not exist until a boot, so inspecting this image under `podman run`
fails with a sentence that points at the wrong file.

Neither finding was fixed, because the build passed and this task changes no
decision to make one pass: they are task 34. What did change is
`crates/alo-image`, which now holds the recipe's own build-time assertions to the
logins the image declares — building it is what showed those seven `test` lines
are the only thing standing between `systemd-sysusers` taking a different number
and a machine whose description names a login it does not have, and that nothing
held them to the file they are about. **No *On the machine* box moved**,
`ROADMAP.md`'s machine half stays empty and `docs/features.md` was not touched.
Report: `docs/autonomy/updates/the-recipe-built-rather-than-read.md`. The next
task (34) is written below.

### 34. Half an image of dead weight, and a runtime that calls home

**Status:** ready. **Depends on:** nothing, on a machine with a container
runtime. **Owner:** Claude — it touches no compositor file and needs no screen.

Written by task 33, out of the two things building the image measured. Both are
about the same file and both are the shape this plan keeps producing: a promise
in `docs/` that a rented engine quietly makes untrue.

**The weights are carried twice.** `ollama create` leaves the source GGUF in the
store under its own digest and writes a second blob the manifest names instead —
same length, different digest — so `/usr/share/alo/models` is 4.5 GiB for 2.23
GiB of model, the image is 8.38 GiB where the recipe's comment says *2.23 GiB,
carried once*, and `/usr` is read-only on a bootc machine so nothing can ever
prune it. Every machine we ship carries it, over every network it is installed
across.

**And the runtime calls home.** Two HTTPS requests to `ollama.com` in the first
eight milliseconds, with nothing asked of it, retried while the machine is up.
`alo-modeld.service`'s `IPAddressDeny=any` refuses them on a machine — but
nothing in this lane has ever started that unit under systemd, so *the filter
stops it* is exactly the kind of sentence task 33 was written to stop trusting.

- **Acceptance:** the image carries the weights **once** — whichever way is
  chosen, with the cost of the other written down, and with the pinned digest
  still checked before anything reads the file (the check is the pin; a cheaper
  import that dropped it is not cheaper); the recipe's own comment and
  `crates/alo-image` agree with the store that is actually on the image, held
  there by a test rather than by prose; the runtime's own `OLLAMA_NO_CLOUD` is
  **decided** — set in the unit as a second lock or deliberately not set, argued
  either way in one sentence and checked by `crates/alo-image` if it is set; and
  **the unit's filter is watched rather than read**: `alo-modeld.service` started
  by a real `systemd` with the image's own store, and the two requests above
  refused by `IPAddressDeny=` rather than by an absent network, with what was
  seen quoted. A rebuild measures the new image against 9,000,704,537 bytes and
  says what it now is.
- **Constraint:** it may not tick *arrives ready to run* — that waits on a
  machine, and this lane still has none, so **no *On the machine* box moves**. It
  configures the engine and never patches it (ADR 0011): removing a file the
  runtime wrote is the image's business, changing what the runtime writes is not.
  It may not weaken the digest check, widen what the unit may reach, or give it a
  capability to make systemd start it under a container. And if starting the unit
  needs privilege a machine would not need — a container that must be privileged
  to run `systemd` is not evidence about a machine — say so and measure what can
  honestly be measured rather than quietly running it as root.

**Done, 2026-09-12.** **Carried once**: the weights stage removes the runtime's
copy of the checked file after the import — the runtime itself tries to delete
that blob at every start and, on a machine, fails against `/usr` — and asserts
three things before the store leaves the stage: the runtime can still `show` the
model off what is left, there is one manifest, and every blob in the store is
named by it. The digest check before the import is untouched. The other two ways
out are priced in `docs/quirks.md`: importing by hand is a second implementation
of a rented store format, carrying it deliberately is 2.23 GiB on every machine
for a file nothing opens. `crates/alo-image` reads both new lines and refuses a
recipe without either, with the twin tests that break a copy. **The runtime does
not call home**: `OLLAMA_NO_CLOUD=1` is set in the unit as a second lock beside
the filter — measured, it makes neither request — and checked. **And the filter
was watched rather than read**: the unit started by the image's own systemd,
under a container whose init was given three capabilities a machine's init
already holds (the unit unchanged, its process with every capability set empty),
with the image's own store and a working network. Sixteen packets to the
publisher's port 443 attempted by the unit's login, counted inside; **zero** at
the host side of the bridge; a control request from an unfiltered process in the
same container answered `200`. What the runtime logs in that state is
`context deadline exceeded`, which is what a dropped SYN reads like, and the
entry in `docs/quirks.md` says so. **Not a boot** — no firmware, no disk, and no
*On the machine* box moves. The rebuild's size against 9,000,704,537 bytes is in
the report. Report:
`docs/autonomy/updates/the-weights-carried-once-and-a-runtime-that-does-not-call-home.md`.
Task 35 was already written below.

### 35. Recovering a parked task whose worker never wrote a handoff

**Status:** ready. **Depends on:** nothing.

`recover <branch>` was used on real parked work three times on 2026-09-11 and
refused twice — both times honestly, both times for the same reason: **the
parked branch had no handoff.** Parking force-adds `.kernel-loop/handoff.toml`
onto the branch, but on both occasions the gates had refused a first worker,
the repair path had moved that handoff to `.kernel-loop/refused/`, and the
second worker was killed at the 90-minute deadline before writing one. So the
branch held every line of the work and the command could not touch it, and the
recovery was done by hand — the branch's own diff applied onto `main`, and the
handoff copied back out of `refused/`. That is knowable and mechanical, which
is the same argument task 29 made for the command existing at all.

- **Acceptance:** a parked branch with no handoff is recovered from what it
  does have — the file list read off the branch's own commit (its diff against
  its parent), and the handoff taken from the newest `.kernel-loop/refused/`
  entry whose `task` matches the task the branch was parked for; both are
  reported as such, so a person knows the handoff was reconstructed rather than
  restored; a branch with neither a handoff nor a matching refused one is
  refused in words that say what was looked for; the existing behaviour for a
  branch that *has* a handoff is unchanged; and the refusals are tested beside
  the recoveries, on branches made the way parking makes them.
- **Constraint:** it never derives a file list from `git status` or from
  guesswork — only from the branch's own commit — and it still restores and
  never publishes, deletes no branch, and touches nothing outside the paths it
  names. `git checkout -- .` and `git restore -- .` remain the thing the test
  refuses.

**Done, 2026-09-12.** `recover <branch>` now asks the branch whether it carries
`.kernel-loop/handoff.toml` before reading it. When it does, nothing changed:
the handoff is the file list, and a handoff that does not describe its branch
is still refused before anything is written. When it does not, the file list
is **the branch's own commit** — its diff against the `main` it was parked
from, read with the same `git diff --name-status` the existing path uses, never
`git status` — and the handoff is **the newest refused one for the task**:
the number out of the branch's name, the plan asked which task that is, and
`.kernel-loop/refused/` read newest-first by the moment in each file's name for
a handoff naming it. Both are reported as reconstructed, on the terminal and in
the journal, with the file it was taken from; and because a first worker's
handoff does not describe what a second worker left, the recovery lists the
files it names that the branch never changed and the files the branch changed
that it does not name, and is not ready to gate until a person has made the
handoff say what the branch says. A branch with neither is refused naming the
directory, the number and the task looked for; a number the plan does not know
is refused naming the plan. Seven new tests in `recovering.rs` park real
branches the way the supervisor parks them — one by the exact road, with the
real `put_aside` and a second worker that wrote more and no handoff — an eighth
reads the number out of a parked name, and three in `handoff.rs` hold the
newest-by-moment lookup. The whole-tree
commands are still what the source-reading test refuses. Report:
`docs/autonomy/updates/recovering-a-parked-task-without-its-handoff.md`. The
next task (36) is written below.

### 36. A parked branch carries the handoff its gates refused

**Status:** ready. **Depends on:** 35.

Task 35 reconstructs a handoff-less branch from `.kernel-loop/refused/`, which
works on the checkout that parked it and nowhere else: the refused directory is
ignored by git, so a parked branch copied to another checkout — which is the
one reason parked branches exist as branches rather than as stashes — carries
every line of the work and no way to name it. The upstream cause is that
parking force-adds `.kernel-loop/handoff.toml` and nothing else, and on the road
that produces most parks the repair path has already moved that file aside.

- **Acceptance:** when parking finds no handoff waiting and the newest refused
  handoff for the task being parked exists, it force-adds that entry onto the
  branch as `.kernel-loop/refused-handoff.toml` — a distinct name, so nothing
  can mistake it for a handoff the parked worker wrote; `recover` prefers, in
  order, the branch's own handoff, a refused one carried on the branch, and
  the checkout's `.kernel-loop/refused/`, and reports which it used; the two
  reconstructed cases are reported exactly as task 35 reports one, differences
  and all; parking with neither still parks (a branch with the work and no
  handoff is still better than no branch); and the refusals are tested beside
  the recoveries, on branches made by the real `parked`.
- **Constraint:** parking still pushes nothing and still never writes
  `.kernel-loop/handoff.toml` itself; the carried file is never written back as
  the task's own handoff without being reported as reconstructed; the file list
  still comes from the branch's own commit and never from `git status`; and
  `git checkout -- .` and `git restore -- .` remain the thing the test refuses.

**Done, 2026-09-12.** `tools/kernel-loop/src/parking.rs` is the step before and
after the git of parking: when no handoff is waiting and the newest refused one
for the task exists, it copies that entry to `.kernel-loop/refused-handoff.toml`,
and `repository::parked` force-adds it beside the handoff's own name — one list,
`Handed::what_parking_carries`, so what parking adds and what recovery looks for
cannot drift. Parking with neither still parks, still pushes nothing, and still
writes no `handoff.toml`; the copy is taken back whether the park succeeded or
not. `recover` prefers the branch's own handoff, then the one it carries, then
this checkout's `refused/`, and says which — `TakenFrom` on the reconstructed
account, on the terminal and in the journal — with the carried case reported
exactly as task 35 reports the directory one, differences and all. The loop's
own files are never brought back into the tree: the carried handoff stays on
the branch and is listed as left alone. A carried handoff that does not read is
refused naming the file rather than passed over for the directory, because
parking copied it from a handoff the gates had read and one that no longer
reads has been changed since. Ten new tests: four in `handoff.rs` on the copy
and six in `recovering.rs` on branches made by the real `parked` — one of them
cloning the repository into a second checkout with no `refused/` at all, which
is the case this task exists for. Two task-35 tests now park before any handoff
is refused, so that the directory road is still the one they walk. Report:
`docs/autonomy/updates/a-parked-branch-carries-its-refused-handoff.md`. The
next task (37) is written below.

### 37. Recovering a parked task from another checkout, as a command

**Status:** ready. **Depends on:** 36.

Task 36 made a parked branch say what it is wherever it goes. Getting it there
is still a memory: two checkouts share this repository (`SHARED_MAIN.md`),
parked branches are local by decision, and the way one crosses today is
`git fetch <path> <branch>:<branch>` typed by hand — a step with no test, no
refusal, and a refspec a person can get wrong in the direction that overwrites
a branch already here.

- **Acceptance:** `recover <branch> --from <checkout>` fetches exactly that
  branch, by name, from the other checkout's path into this one and then
  recovers it as `recover <branch>` does; it refuses a `--from` that is not a
  git repository, a branch the other checkout does not have, and a branch this
  checkout already has — never overwriting a local branch, even one of the same
  name; the fetch is written in the journal with where it came from; and the
  refusals are tested beside the recovery, on two real repositories.
- **Constraint:** it fetches and never pushes — the other checkout is read and
  not written, and `the_only_branch_this_pushes_is_main` stays true; the fetched
  branch is a local branch here like any parked one, and nothing deletes it;
  the recovery after the fetch is the existing one, unchanged; and
  `git checkout -- .` and `git restore -- .` remain the thing the test refuses.
