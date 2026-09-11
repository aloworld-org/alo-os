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

**Status:** blocked — on ADR 0024 being accepted, and then on task 13.
**Depends on:** 5, 9, 13.

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

**Status:** blocked — on `docs/decisions/0024-what-a-person-signs-in-at.md`
being accepted. A worker that started this before the answer would be choosing
between the options rather than building one. **Depends on:** 2.
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
