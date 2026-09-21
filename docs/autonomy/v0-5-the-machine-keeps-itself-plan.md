# v0.5 — the machine keeps itself: updates, rollback, and undoing what the agent did

**Workstream:** four `ROADMAP.md` v0.5 lines that are one subject — *updates
that never interrupt*, *atomic updates with rollback*, ★ *undo what the agent
did*, and the decidable half of *recovery and rollback screen*. They belong
together because they are all the same question: **what this machine was
yesterday, and how a person gets back to it.**
**Why it exists:** written 2026-09-14 so that no lane sits idle waiting for a
plan. It is the third in a queue — the applications plan is taken first, this
after it.

**What is inherited and what is ours.** `ROADMAP.md` says it outright:
*atomic updates with rollback — largely **inherited** rather than built, since
a bootc image rolls back with one command (ADR 0011). What is ours is the
policy around it, and ★ undo what the agent did, which is the one agent
capability the base rather than our code provides.* So this plan writes almost
no mechanism. It writes **the decisions around a mechanism somebody else
maintains**, which is exactly what ADR 0011 asks of a rented engine.

**Crates this plan owns:** a new `crates/alo-keeping-up` for what an update
is, when it may happen, and what rolling back means, a new `crates/alo-looking`
for finding out there is one — added by task 6 on 2026-09-19, because the asking
needs a client and a disk and `alo-keeping-up` is held to neither — and — from task 4, taken by
the third PC on 2026-09-19 — the **undo entry in `crates/alo-record`** and the
sentence `crates/alo-recounting` reads it back with. ADR 0045 point 3 asks for
a record entry an undo writes, no existing kind fits one, and the crate is
additive: no plan's header claims `alo-record`, and the plan that was working in
it, `v0-5-the-local-network-plan.md`, is finished at 37 of 37. Taking it is the
rule of 2026-09-18, and this line is the record of the take. **It reads and never
edits** `alo-keeping` (where what the machine did is written),
`alo-capability` (what a verb is), `alo-egress` (an update is an
errand that leaves the machine, and the indicator fires for it), `image/` and
`crates/alo-image` (the installer plan's — the registry an update comes from
is established by its task 1). Nothing in `crates/alo-shell` — the recovery
*screen* is the shell plan's, and what this owes it is the decisions it draws.

**What this plan may not do:** tick anything *on the machine*; edit a crate
another lane owns; patch the base or `bootc` (ADR 0011 — a source patch to an
engine requires an ADR first); or make an update happen without the person's
knowledge. Before writing the next task, `git pull` and read the plan as
published.

**It depends on the installer plan's task 1** having established a registry
and a pinned digest. If that has not landed, the first task here is a finding
rather than a guess.

## Tasks

### 1. What an update is, and what it may never do

**Status:** ready. **Depends on:** nothing.

**Done, 2026-09-14.** `crates/alo-keeping-up`: `Digest` is a whole, lowercase
`sha256:` image digest, refused otherwise and read back through the same check;
`Running` is the build reported running, `Offered` the build offered, and
`Standing::between` answers `UpToDate` or `Ready { running, offered }` — differ,
not newer, and no priority, severity or deadline anywhere. `a_check_at` is
`Errand::CheckingForAnUpdate`, and `Offered::heard` takes the `Underway` only the
indicator makes, refusing one for any other errand (`NotACheck`). `THE_RULE`
refuses `Cause::AnUpdate` for each `Disturbance` — restarting the machine,
closing an application, interrupting the person — and allows `Cause::ThePerson`;
there is no third cause. `WhenItApplies` is `AtTheNextRestart` (the default) or
`NowBecauseThePersonAsked`, with no *never* and no *by itself*. Eight sentences,
collected by `alo-saying`, none naming the machinery. The crate depends on
`alo-egress`, `alo-strings`, `serde` and `thiserror` alone, and a test reads its
manifest and source for a clock, a thread, a socket or a file. The registry it
will check against is still the installer plan's task 1, and `a_check_at` takes
the place as an argument rather than guessing it. Report:
`docs/autonomy/updates/what-an-update-is-and-may-never-do.md`.

*Updates that never interrupt.* Every operating system says this and most mean
*we will interrupt you later instead*. The promise is only keepable if what an
update may do is decided before anything downloads, and the decision is a type
rather than a habit.

- **Acceptance:** `alo-keeping-up` holds what an update is — a digest the
  registry offers, the digest running now, and whether they differ — with
  **no clock, no scheduler and no background fetch decided here**; a check for
  an update is an `alo_egress::Errand`, so it appears on the indicator like
  everything else that leaves (★ *no telemetry* is checkable only if the
  machine's own traffic is visible), and a test holds it to that; the rule
  that an update **never restarts the machine, never closes an application
  and never interrupts what a person is doing** is a value with a test per
  clause, and there is no variant meaning *urgent* that could bypass them,
  because *urgent* is the word every interrupting system used; and what a
  person is told — that an update is ready and will apply when they choose —
  is in the vocabulary `alo-saying` collects.
- **Constraint:** nothing here downloads anything or writes to a disk. It
  decides; the doing is task 2. No setting that turns updates off entirely —
  a machine that can be left unpatched by a checkbox is a fleet's worst
  liability — but equally nothing that applies one unasked: the choice is
  *when*, never *whether to be told*.

### 2. An update applied, and the same machine afterwards

**Status:** ready. **Depends on:** 1.

**Done, 2026-09-15.** The doing is a new crate, `crates/alo-updating`, so
`alo-keeping-up` still names no process, file or clock. `alo-keeping-up` adds
the decisions: `Deployments` reads `bootc status` and refuses half a digest;
`Source` is a repository with no tag inside it; and `Staging::of` produces one
instruction, `switch --enforce-container-sigpolicy <source>@<digest>`, plus
`--apply` only when the person asked to restart now. That instruction is
refused when the machine moved on since the offer, when that build is already
waiting, or when no build can be named. `Since` compares the build last known
with the build booted. `alo-updating::apply` reads the status at the moment of
choosing and runs exactly those arguments with no shell. `running` and
`deployments` ask the base every time. `after_a_restart` writes the new
additive `alo_record::Happened::Updated { from, to }`, with no agent, at the
first start on a different build, and writes it before
`/var/lib/alo/last-known-build` advances. **Measured** by
`tests/an_update_keeps_the_persons_things.rs` in a QEMU/TCG virtual machine
built on the pinned base, in 949 s. The run wrote settings, appearance,
grants, pairings, the record, a file index, the indexed-folder list and two of
the person's files; staged the second build; restarted; and found all of them
byte for byte, with one `updated` entry and none on the next start. Found on
the way (`docs/quirks.md`): `bootc install` reads the policy of the image it
installs. For the installer lane: the shipped image needs a signed
`policy.json` before `apply` stages anything, and a unit calling
`after_a_restart` at boot. Report:
`docs/autonomy/updates/an-update-applied-and-the-same-machine-afterwards.md`.

**What the machine that runs this lane can do, measured 2026-09-15.** It is a
VMware guest running Windows Server 2022, with Ubuntu 24.04 in WSL2 on kernel
`6.18.33.2`. **It has no hardware virtualisation**: `/dev/kvm` exists, but
`qemu-system-x86_64 -accel kvm` answers *failed to initialize kvm: No such
device*, and `/proc/cpuinfo` has no `vmx` flag (`docs/quirks.md`). **Software
emulation works**: QEMU 8.2.2 with `-accel tcg`, 3 CPUs and 4 GB booted Fedora
Cloud 42 to its login prompt in **190 seconds**. So the virtual-machine tests
this task and task 3 ask for can run here, slowly; budget minutes per boot, not
seconds. `podman` 4.9.3 is installed in the WSL box. The owner's signed release
0.0.1 at `ghcr.io/aloworld-org/alo-os` is reachable from it with `skopeo
inspect`. The installer plan's task 1 is still writing the repository's pin for
that release, so read the digest from the pin once it lands rather than from
this note.

The mechanism is `bootc`'s: a new image is staged as a second deployment and
the machine boots into it. What is ours is that the person's own things
survive it, and that the machine can say what changed.

- **Acceptance:** applying an update stages the new digest through `bootc`
  without touching `/var` or the person's own files, and a test in a virtual
  machine writes known files, applies an update, reboots, and finds them
  byte-for-byte — the settings, the grants, the pairings, the record and the
  file indexes each checked by name, because *the person's data survived* is
  a claim that must be made about named things rather than in general; the
  record gains an entry saying the machine updated, from which digest to
  which, as an errand with no agent behind it; and the running digest is
  readable at any moment, so *what am I running* is answerable without
  guessing.
- **Constraint:** `bootc` is rented and unmodified. If it cannot do something
  this task needs, that is a finding and an ADR, never a patch. The update is
  applied when the person chooses it and at no other moment.

### 3. Back to yesterday's machine

**Status:** ready. **Depends on:** 2.

**Done, 2026-09-15.** `alo-keeping-up` decides, still with no clock, file or
process: `Deployments` reads `rollbackQueued`; `Before::of` names the build
before from the base's kept deployment and the record's last `Changed`, and
says whether it is still on the disk; `GoingBack::offered` refuses before
anything is offered — `NothingBefore`, `NoLongerKept`, `AlreadyGoingBack`,
`NotRunningABuild` — and its sentence is what the person approves, including
that an update waiting will not apply; `Returning::of` is the one instruction,
`rollback` plus `--apply` only for *restart now*, refused
(`ChangedSinceItWasOffered`) when the machine is no longer what was offered;
`Since::RolledBack` is a return told apart from an update by what the person
chose. `alo-updating` does it: `yesterday` adds *when it was replaced* from the
record; `go_back` notes the chosen build under `/var/lib/alo/going-back-to`
before telling the base, and `after_a_restart` writes the new additive
`alo_record::Happened::RolledBack { from, to }`, no agent, at the first start on
it. Ten sentences, collected by `alo-saying`. **Measured** by
`tests/back_to_yesterdays_machine.rs` in a QEMU/TCG virtual machine on the
pinned base, in 1587 s: update, restart, the build before named and kept, going
back set and a second request refused, restart, the first build running, every
named thing and a file written after the update byte for byte, one `rolled-back`
entry. **Found** (`docs/quirks.md`): going back does not bring `/etc` as it is
now — accounts, passwords and whole-machine settings changed since the update
stay with the newer build, which the sentence says. Report:
`docs/autonomy/updates/back-to-yesterdays-machine.md`.

*Atomic updates with rollback.* The base keeps the previous deployment and
rolls back with one command; that is the inheritance. What is missing is
everything a person needs for it to be a promise rather than a command they
have never heard of.

- **Acceptance:** the previous deployment is named and readable — what it was,
  when it was replaced, and whether it is still on the disk — and a rollback
  is one act that returns the machine to it, tested in a virtual machine by
  updating, rolling back, and finding the earlier digest running and the
  person's files untouched; **a rollback that cannot be done says so before
  it is offered** rather than failing halfway, and the reason is in the
  vocabulary; the record says the machine rolled back, to which digest and
  when; and what a person is shown is the decisions this task makes, handed
  to the shell plan's recovery screen rather than drawn here.
- **Constraint:** rolling back never touches the person's own files — an
  update that changed them could not be undone by a deployment swap, which is
  precisely why task 2 forbids touching them. Nothing here deletes a
  deployment to make room; that is a policy with its own consequences and is
  a task of its own if it turns out to be needed.

### 4. Undo what the agent did

**Status:** ready — [ADR 0045](../decisions/0045-what-undoing-rewinds-to.md) was
accepted on 2026-09-16, option A (the base's snapshot on btrfs, a subvolume per
home, a snapshot either side of a changing turn), with six terms the owner added:
undo reaches back seven days or fifty changing turns, whichever ends first; disk
pressure removes the oldest first and the record says which turns lost their undo;
a machine too full to snapshot still runs the turn and records that it cannot be
undone; `alo-measuring` counts what undo is holding, by name; only a turn that
changes files is bracketed; and the installer's move to btrfs lands **before the
certified laptop is installed**. Build points 1 to 5 of the decision here, with
every change verb answering *not yet on this machine* until the bracket exists on
it. The filesystem itself is the installer plan's task 11, the home subvolume the
accounts work's, the bracket lane A's `alo-turn`, and the privilege to snapshot
the broker plan's — each named in the decision, none of them this plan's to edit.
**Depends on:** 3.

**Done, 2026-09-18.** Points 1 to 5 of ADR 0045's *what holds whichever option
is taken*, with every change verb answering *not yet on this machine* because
that is what is true here. `alo-keeping-up` decides, still with no clock, file
or process: `WhatWasDone` is what the record says as far as undoing cares and
`could_be_put_back` is the decision's closed table — three verbs have a road
back (`Change::Renamed`, `Moved`, `Archived`) and **every other verb this
machine ships or gains answers never**, each of the fourteen `NotUndoable`
reasons in its own sentence; `AnUndo::offered` asks the second question, what
this machine kept, and refuses before anything is offered —
`NothingKeepsWhatWasThere`, `TheTurnWasNotKept`, `NoLongerKept`,
`ChangedSinceItWasDone`, `NothingLeftToPutBack` — so one name changed since
refuses the whole undo rather than taking away a person's newer work; `AnUndo`'s
sentence names **what** comes back and **when** it was changed, and is the one
sentence in this crate with anything to fill in; `HowFarBack` is the owner's
first term, seven days or fifty changing turns, whichever ends first, refusing a
window that reaches nothing, and `WhatWasKept::forgetting` is his fifth, one act
that says both what it costs and what it frees. `alo-record` gains the additive
`Happened::Undone { undid, what, failed }` — no agent and no field for one,
carrying a **copy** of what it undid rather than a pointer into a file that is
pruned, written only from an entry that **ran**, a failed one counted among the
refusals; `alo-recounting` reads it back as *you put this back* or *it could not
be*, with the change's own sentence on the same line. `alo-updating` is the seam
and the only place the two are put beside each other: `what_was_done` maps every
kind of entry onto the table exhaustively, `what_this_machine_kept` answers
`NothingOnThisMachine` and names the four lanes that would change it, and
`putting_back` is one call from an entry to an honest answer. **No agent verb
undoes and none proposes one**, held against `alo-declared`'s registry.
`docs/contracts/record-file.md` gains the `undone` kind; `format` stays `1`.
Nothing puts a file back: the road is still the installer's task 11, the
accounts lane's home subvolume, lane A's bracket and the broker's privilege.
Report: `docs/autonomy/updates/undo-what-the-agent-did.md`.

**Before it: decided as far as a worker could, 2026-09-15.** The road this task
names did not exist on the machine we install: `crates/alo-installing` formats
the disk `ext4`, which has no subvolume and no snapshot, and the base's rollback
swaps `/usr` and leaves a person's home alone (task 3). Measured in the pinned
base: `bootc 1.15.1` accepts `--filesystem btrfs` and `btrfs-progs 6.19.1` is
there, so a snapshot is a configuration away — but the filesystem is the
installer plan's, the snapshot at turn start is lane A's `alo-turn`, the
privilege is the broker plan's, our own copy contradicts ADR 0011 and this
task's constraint, and undoing nothing narrows a ★ line. So the work handed
over is the decision:
`docs/decisions/0045-what-undoing-rewinds-to.md`, **proposed**, recommending a
btrfs home subvolume with a read-only snapshot either side of a changing turn
(option A), with *nothing can be undone yet, said honestly* (option D) as the
fallback and our own copy (B) and inverses from the record (C) rejected. It
also settles what holds under every option: which entries can never be undone
and the sentence why, that an undo is the person's with no agent verb, that it
never overwrites a later change, and that the record gains an additive `undone`
kind carrying a copy of what it undid. **No code was built in that change**, and
`crates/alo-keeping-up/tests/undoing_is_decided_before_it_is_built.rs` still
fails if an `Undone` entry or a file about undoing appears while the ADR says
*proposed* — it says *accepted*, and what was built above is what the
acceptance asked for. Report:
`docs/autonomy/updates/undo-what-the-agent-did-waits-on-a-snapshot-road.md`.

★ and the sharpest of the four. `ROADMAP.md`: *the one agent capability the
base rather than our code provides.* An agent acted under a grant, a person
approved it, the record says exactly what happened — and the person has
changed their mind. Today nothing can answer that.

- **Acceptance:** for a verb the record says ran, `alo-keeping-up` answers
  whether it can be undone, and **says plainly when it cannot** — a message
  that was sent, a question that was asked of a model, a file handed to
  another machine are gone beyond recall, and a system that implied otherwise
  would be lying at the worst possible moment; what *can* be undone is
  undone through the mechanism that made it undoable — a file written inside
  a granted folder, where the base's own snapshot of that subvolume is the
  road — and never by a second implementation guessing at inverses; undoing
  is itself recorded, with the entry it undid named, so the record remains a
  complete account rather than one with a hole where a change used to be;
  and undoing something requires the same person's approval the doing did,
  because an undo is a change.
- **Constraint:** no inverse is invented. If the record says *sent an email*
  there is no undo, and the honest sentence is the deliverable. Nothing here
  gives an agent the ability to undo its own work unasked — the ★ promise is
  a capability for the **person**, and an agent that could quietly revert
  what somebody approved would be worse than one that could not.

### 5. What a person is told, before and after

**Status:** ready. **Depends on:** 1, 4.

**Done, 2026-09-19.** Nothing was re-decided: every sentence this task holds
existed before it, and what was missing was the proof that they are a
*sequence* rather than forty-five strings in two crates.
`crates/alo-updating/tests/what_a_person_is_told.rs` is the one test file, here
rather than in `alo-keeping-up` because *before and after* needs both halves —
the offer and the promise are `alo-keeping-up`'s, and *this machine started on
an updated version of its system* is `alo-recounting`'s, read off an entry
`alo-record` keeps, and `alo-updating` is the crate that already has both
beside it. `THE_WALK` is the sixteen sentences a person meets from *an update
exists* through *it is applied* to *it is rolled back*, each **produced by
driving the real types** — `Standing::between`, `THE_RULE`, `WhenItApplies`,
`Staging::of`, `Since::between` onto `Entry::updated`, `GoingBack::offered`,
`Returning::of`, `Since::RolledBack` onto `Entry::rolled_back` — and compared
with the table's key *and* its exact English; the same table is printed
verbatim in the report. Changing one word of one sentence was measured to fail
it, twice over. The other four: every one of `alo-keeping-up`'s forty-one
words is in the machine's one vocabulary with a translator's note **and is
reachable from a public `said()`**, which is checked by collecting what every
refusal and offer in both crates renders and comparing the set; each of the
fourteen *this cannot be undone* refusals says **why**, no two alike, with the
ones the plan names checked by name; no sentence in either crate names the
machinery, which now holds `alo-recounting`'s four clauses to the list
`alo-keeping-up` already held its own to, and adds `subvolume`, `btrfs` and
`filesystem` to it for the undo road; and every refusal on the same walk is
said, distinct, and — for the six that follow something the person chose —
says the machine is as it was. One dependency added, `alo-recounting` as a
dev-dependency of `alo-updating`; no `src/` changed anywhere. Report:
`docs/autonomy/updates/what-a-person-is-told-before-and-after.md`.

**Finding, 2026-09-20 — two of the things a person is told have no sentence.**
Handed in by the shell plan's task 13 and by `alo-access`'s tree. *What this
machine is running* and *what it replaced* are drawn on the recovery screen and
named in the accessibility tree — `access.what-is-running` and
`access.what-it-replaced` — and **no crate words either**: `Deployments` has no
`said`, and `Since` has no words at all. So each of those two lines has a name a
screen reader announces and **no sentence to read after it**. Filling it is this
plan's: a drawing crate that wrote those sentences would be deciding what a
person is told, which is what this plan exists to refuse. It is not free either
— this plan's own rule is that a person is told an update is ready and **never
which build it is**, so whatever those two say must say it without naming a
digest or a deployment. Until then the two lines are named and empty, on
purpose, and a reader announces a line with nothing in it rather than a line
nobody knows is there.

The four promises above end in sentences, and the sentences are the product:
*an update is ready*, *this will not interrupt you*, *you are running this,
you were running that*, *this can be undone*, *this cannot*.

- **Acceptance:** every sentence this plan's crates can say is in the
  vocabulary with a translator's note, and a walk from *an update exists* to
  *it is applied* to *it is rolled back* produces the exact sequence a person
  meets, recorded verbatim in the report as a table and held by one test that
  fails if any sentence changes without the table changing; the sentence for
  *this cannot be undone* names **why** — it left the machine, it was sent, it
  was told to somebody — because a refusal without a reason is a system a
  person stops trusting; and no sentence names `bootc`, a deployment, a
  digest or any other word from the machinery (`docs/features.md`: *a person
  never learns the name of anything we rented* — *the system updates; it does
  not "apply an OSTree transaction"*).
- **Constraint:** nothing here re-decides what the sentences describe. If a
  sentence is true and reads badly, the sentence changes; if it reads well
  and is not true, it changes the other way.

### 6. Finding out there is an update

**Status:** ready. **Depends on:** 1, 2.

**Done, 2026-09-19.** A new crate, `crates/alo-looking`, so `alo-keeping-up`
still names no clock, socket or file and `alo-updating` still has no HTTP client
in it. `Place::on_this_machine` is where this machine asks, read through
`alo_image::ThePin` from `image/pinned.toml` — the registry is the repository it
asks and the pinned release is the **oldest** it will be offered, and a test
reads every file of the crate for the address and finds none. `look` is the one
act: it puts `a_check_at` on the indicator before it asks anything and takes it
off after the answer whichever way it ends, asks the place two questions —
which names it holds, and which build one of them is (a `HEAD`, the answer and
never the build) — and hears the `Offered` from the same `Underway`. `Because`
is the whole list of occasions and it is two, *the person asked* and *this
machine started*; there is no thread, timer, clock read or `await` anywhere in
the crate and a test reads its source for each. `NoAnswer` is the closed set of
five, each with a sentence that says the machine is as it was and names no
machinery — four of them this crate's four words, and *the answer could not be
understood* is `alo-keeping-up`'s existing sentence rather than a second
spelling. `SaidOnce` is ADR 0009's rule in `alo-telling`'s shape: a machine with
no way out is told once, a suppressed telling carries nothing a surface could
word anyway, and any answer forgets it. `Kept` is one file under
`/var/lib/alo/an-update-was-found`, written whole or not at all and read
strictly, so a surface reads the answer back without a second line on somebody's
indicator — and a kept answer names the build it was about, so *an update is
ready* on a machine that has moved on is refused with
`CHANGED_SINCE_IT_WAS_FOUND` rather than shown. What is read back is
deliberately **not** an `Offered`: that type still does not deserialise, and
carrying a decided update across a process is ADR 0053's open question.
**Measured against the real registry** on 2026-09-19 in 2.6 s
(`tests/against_the_real_registry.rs`, `#[ignore]`d because no other test in
this suite reaches the internet): `ghcr.io/aloworld-org/alo-os` answered with
six names, the newest release it offers is `0.0.3` at
`sha256:41d43c7e…`, the pinned `0.0.2` is exactly the digest
`image/pinned.toml` holds, the road out was decided by `alo_proxy::the_way` for
`Road::CheckingForAnUpdate`, the indicator carried the check between the two
requests, and three refusals came from the real place — a repository nobody
publishes and a release it does not hold are both *it refused*, a host that does
not exist is *no way out*, and a proxy that is not there turns a place that
answers straight out into one that cannot be reached. **Found:** a machine
running the pinned release is offered `0.0.3`, which the owner has pushed and
signed but this repository has not pinned — task 7.

**That measurement is of 2026-09-19, and the registry has moved twice since.**
`0.0.3` was pinned that evening and **`0.0.4` on 2026-09-20**, which is what
`image/pinned.toml` holds now, at `sha256:48bd5f31…`. The measurement is left
exactly as it was taken — one with a date is worth more than one edited to stay
true — but the impression that a machine today is offered `0.0.3` is not left
standing. Anyone re-running `tests/against_the_real_registry.rs` should expect
the pinned release to be `0.0.4`.

**Refused once and fixed:**
the first attempt's unit tests took their floor from `image/pinned.toml` while
writing their own release names down, so pinning `0.0.3` turned a place holding
`0.0.2` into one offering nothing; a test that means *a release this machine
would take* now says which (`testing::a_place_not_before`), and the two tests
that are about the shipped pin read the release out of the place rather than
writing a number down. Nothing that ships changed. Report:
`docs/autonomy/updates/finding-out-there-is-an-update.md`.

Four fifths of *updates that never interrupt* is built and **nothing on this
machine has ever looked**. Task 1 left it out by name — `alo-keeping-up`'s own
header says *when to check is a later task that builds on these types* — and
the whole promise rests on a machine that finds out there is an update without
watching for one. It is the last piece of that `ROADMAP.md` line this plan can
build; the recovery *screen* stays the shell plan's.

- **Acceptance:** where this machine checks is **read rather than guessed** —
  the registry and release `image/pinned.toml` already pins, through
  `alo-image`, so one repository has one answer about where a build comes from
  and an organisation's own mirror is v1 rather than a second constant here;
  a check is one act that asks that place and is on the indicator for the whole
  of it (`alo_keeping_up::a_check_at`, and an `Offered` can be heard no other
  way), fetching **the answer and never the build** — a test reads the crate
  for a container library and for anything that writes a fetched build to the
  disk; **when** a check happens is somebody's act and never a watcher's, which
  means the two that exist are *the person asked* and *once at a start*, each a
  call something else makes, with no thread, no timer and nothing that checks
  while a person is working; what a check answers is an `Offered` or one of a
  closed set of refusals with a sentence each, and a machine with no way out at
  all says so **once** rather than at every asking
  (`updates/a-machine-that-cannot-reach-a-model-says-so-once.md` is the shape
  to copy); the answer is kept where a surface reads it back without asking
  again, and a kept answer names the build it was about, so *an update is
  ready* on a machine that has moved on since is refused rather than shown
  stale; and the whole of it is **measured against the real registry** that
  `image/pinned.toml` names, from this machine, with the proxy honoured
  (`alo-proxy`), a refusal exercised, and the indicator's own record read
  afterwards to show the check appeared there.
- **Constraint:** nothing downloads a build. Staging is task 2's and is the
  person's choice; a check that fetched the build to be helpful would be
  exactly the traffic law 1 exists to make visible. No setting that turns
  checking off (task 1's rule), and no member meaning *urgent*.
  **`alo-keeping-up` gains no clock, no socket and no file** — the test holding
  it to four dependencies stays exactly as it is, and whatever does the asking
  is `alo-updating`'s or a new crate beside it.

### 7. An offer a person can act on

**Status:** ready. **Depends on:** 6.

**Done, 2026-09-19.** The offer **carries** the doubt rather than hiding the
build: `alo_keeping_up::Vouching` is two members, `Offered::heard` takes one as
an argument (the value a caller would get by forgetting it is the comfortable
one), `Ready` carries it and `Standing::said` says
`keeping-up.ready-not-vouched-for` in place of `keeping-up.ready` — *before*
the person chooses. Narrowing the offer to what the place vouches for was
considered and discarded in writing: it would leave a machine reading *up to
date* while a newer version sat at the place, which is task 1's rule turned
inside out, and would hide the window between pushing and signing from the only
people who could close it. `alo_looking::vouched_for` reads it out of the names
the place already answered with — no third question and no second departure —
and counts only the name **this machine's base would read**. The refusal a
person meets when the policy refuses a build is now
`keeping-up.not-genuine`, its own sentence, told apart from *the update could
not be prepared* by what the base said
(`alo_updating::refused_for_its_signature`, whole clauses only, because the
base prints *Getting image source signatures* on its way to a **success**).
Both new words are in the vocabulary with a translator's note and reachable
from a public `said()`. A kept answer carries the vouching too, and silence in
an older one reads as *nobody did*.

**Measured, on the real place and a real base.** The place answers seven names;
the newest release is `0.0.4` at `sha256:48bd5f31…`; nothing at the place
vouches for it, and a person reads *A new version of this machine's system is
available, but this machine cannot confirm that it came from alo OS.* The
base's own `skopeo` and `containers-common`, out of the image
`image/Containerfile` pins, under a policy requiring `image/signing/alo-os.pub`,
refused **both** `0.0.4` **and** the pinned `0.0.3` with *Source image
rejected: A signature was required, but no signature exists* — before a single
layer was fetched, so the machine is unchanged. **Found:** `cosign` 3 signs as
an OCI 1.1 referrer (`sha256-<build>`, no suffix) and `containers/image` reads
the `.sig` attachment, so **no alo OS release published today can be staged
under `--enforce-container-sigpolicy`**. That is the release process's and so
the installer lane's (ADR 0036): handed over as a finding, never repaired here
by lowering what counts. Two entries in `docs/quirks.md`. Report:
`docs/autonomy/updates/an-offer-a-person-can-act-on.md`.

Task 6 measured something nobody had looked at before, and it is a gap rather
than a detail: a machine running the release `image/pinned.toml` pins is
offered **`0.0.3`**, which the owner pushed and signed and which this
repository has **not** pinned. The check is right to offer what the place
offers — a machine cannot know about a release that did not exist when it was
built, and the pin travels with the build — but *an update is ready* is a
sentence a person acts on, and the act is `alo_updating::apply`, which stages
under `--enforce-container-sigpolicy` (ADR 0036) and refuses a build that
policy will not have. A person told an update is ready and then told it could
not be prepared has been told two true things and learned nothing, at the one
moment a machine sold on sovereignty cannot afford to be vague.

- **Acceptance:** what a machine is offered today and what happens when it
  acts on it is **measured, on the real place and a real base** — the build
  offered, the instruction staged, whether the signature policy accepts it,
  and the exact sentence a person reads if it does not, with the machine shown
  unchanged afterwards; a check does not offer a build this machine has no
  reason to believe it would accept, which is either the offer narrowed to
  what the place can vouch for or the offer carrying that it has not been
  vouched for and saying so **before** the person chooses — whichever is
  chosen is written down with the reason, and so is the one discarded; the
  refusal a person meets when a build is refused by the signature policy is
  its own sentence rather than `keeping-up.not-prepared`, which today says
  both *the download failed* and *what arrived was not a genuine alo OS* and
  so tells a person neither; and every refusal added is reachable from a test
  and is in the vocabulary with a translator's note.
- **Constraint:** nothing here weakens `--enforce-container-sigpolicy` or adds
  any road to staging a build the policy refuses — that is the whole of what
  makes an offer safe to act on. What is **pushed and tagged** at the place is
  the installer plan's release process (ADR 0036): a change wanted there is a
  finding handed to that lane in this task's report, never made here. And no
  setting that turns checking off, and no member meaning *urgent*.

### 8. An update that really stages, under the policy alo OS ships

**Status:** blocked — on a release at the place that this machine's signature
policy accepts, and on the shipped image carrying a `policy.json` at all. Both
are the installer lane's under ADR 0036; task 2 handed over the second and task
7 the first. **Depends on:** 2, 7.

Every crate in this plan is finished and **no alo OS machine can apply an
update today**. Task 7 measured why, on the base's own tooling: the signature
this repository publishes is not the one `containers/image` looks for, so
`--enforce-container-sigpolicy` refuses the pinned release itself. The refusal
is the correct one — a machine that cannot confirm a build refuses it, and says
so in its own sentence — but a promise nobody has ever seen kept end to end is
a promise, not a measurement. `tests/an_update_keeps_the_persons_things.rs`
staged a build from a registry on the host with a policy that trusts it; what
has never happened is **this machine staging a real alo OS release under the
real policy**, which is the only version of the claim a customer's machine
makes.

- **Acceptance:** a virtual machine built on the pinned base, installed with
  the shipped `policy.json` rather than a test's, updates from one signed alo
  OS release to another through `alo_updating::apply` with nothing weakened —
  the instruction unchanged, the signature policy enforced, the person's named
  things byte for byte afterwards and one `updated` entry, as task 2 measured
  with a policy of its own; **and the same machine, offered a build nothing
  vouches for, refuses it and reads `keeping-up.not-genuine`** — so the two
  halves of task 7 are measured on a real boot rather than on the base's
  library alone; a release that the place vouches for is read as vouched for by
  `alo_looking::vouched_for` against the **real** place, which is the same test
  as task 7's with the answer the other way round, and it is what says the
  release process and this machine agree at last; and the time each half took is
  written down, because the lane that runs it next needs to know what it costs.
- **Constraint:** nothing here changes what is pushed, tagged or signed at the
  place, and nothing lowers what counts as vouched for — if the release process
  has not moved, this task is still blocked and saying so is the work. No
  policy of a test's own stands in for the shipped one: that substitution is
  exactly what task 2 did, honestly, and exactly what this task exists to stop
  doing.

### 9. A staging decided from an approval that arrived from elsewhere

**Status:** ready. Written 2026-09-20 for
[`v0-5-the-broker-and-the-disk-plan.md`](v0-5-the-broker-and-the-disk-plan.md)
task 8, which names it as one of its three blockers and **may not write it**:
`alo-keeping-up` is this plan's crate and that plan never edits it. Its other two
blockers are closed — [ADR 0053](../decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
was accepted by the owner on 2026-09-19, option B, and `alo-egress` gained
`Errand::FetchingAnUpdate` the same day. **Depends on:** 1, 2.

**Done, 2026-09-19.** `Staging::approved(from, to, deployments, source)` is the
second door, and there is one instruction behind both: `Staging::of` and
`Staging::approved` each call a single private `decided`, so the two cannot drift
apart by construction — and
`an_approval_and_a_ready_decide_the_same_instruction_element_for_element`
measures it anyway, on one `Ready`, argument by argument. Making the approved
door decide a different `when` was measured to fail that test and the one beside
it, both of them, before the change was put back. **It takes no `WhenItApplies`
and always decides `AtTheNextRestart`**, which was the one thing the task left
open: an approval that arrived from elsewhere carries two builds and nothing
else, *restart now and apply it* is not in it, so it is not decided from it — and
the instruction it writes therefore cannot carry `--apply`, which makes ADR
0053's *no instruction the broker causes restarts the machine* true by
construction here rather than by the unit remembering it. The fourth refusal is
`NotStaged::NotAnUpdate`, with `keeping-up.not-an-update` beside it: its own
sentence in the machine's one vocabulary with a translator's note, reachable from
`NotStaged::said`, and distinct from the other three by a test that reads all
four. It is decided **before** the base's status is read, so an approval naming
one build twice reads as what it is rather than as a machine whose build could
not be named. An approval whose two builds arrived the wrong way round is refused
too, by `TheMachineMovedOn`, which carries both. This crate gained no
dependency, no road to the base, no program name and nothing that starts
anything. **Found, and handed back rather than repaired:** the paragraph above
says ADR 0053 was accepted on 2026-09-19 and
[the decision itself](../decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
still reads *proposed, 2026-09-17*, so
`crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs` still passes for
exactly the reason it was written — the broker plan's task 8 is now blocked on
that one line and on nothing this crate owes. Accepting a decision is the
owner's, so nothing was changed to match it. Report:
`docs/autonomy/updates/a-staging-decided-from-an-approval.md`.
**Cleared 2026-09-20** by the task that was blocked: ADR 0053 now reads
*accepted*, that guard is gone, and the broker plan's task 8 is done. This
crate's second door was walked through for the first time in
`crates/alo-brokerd/src/staging_an_update.rs`, which decides through
`Staging::approved` and assembles nothing of its own — the second way to stage
that this paragraph exists to prevent was not built. Report:
`docs/autonomy/updates/updates-through-the-broker.md`.

[`Staging::of`](../../crates/alo-keeping-up/src/staging.rs) decides the one
instruction from a `Ready`, and a `Ready` exists only inside a check **this**
machine made: it carries the build this machine was running when it looked and
the build the place offered. That is right for the road a person takes through
the shell, and it is the wrong shape for the broker's. What reaches the broker
under ADR 0053 is an approval — a `{from, to}` pair a person approved, carried
over the door, with no `Ready` behind it and nothing of this crate's in it. So
today there is no way to decide a staging from an approval, and the plan that
needs one is not allowed to add it.

**What must not happen is a second way to stage.** The temptation is for the
broker's unit to assemble the base's arguments itself from two digests, and then
there are two places that know what staging an update means and one of them will
drift. The instruction stays this crate's, and gains a second door into it.

- **Acceptance:** a constructor on `Staging` that decides the same instruction
  from an approved `{from, to}` and the base's `Deployments` **read now**, after
  the approval and before the instruction is made, so its refusals are about the
  machine as it is rather than as it was when somebody approved. It refuses,
  each with a sentence of its own: the machine is no longer running `from`
  (`TheMachineMovedOn`, carrying both digests so the person is told what moved);
  `to` is already staged (`AlreadyWaiting`); the base reports no running build;
  and `from` and `to` are the same digest, which is not an update and is the one
  refusal `Staging::of` never needed because a `Ready` could not hold it. **A
  test takes one `Ready`, decides a staging both ways — through `Staging::of`
  and through the new constructor given that `Ready`'s own `{from, to}` — and
  asserts the two `arguments()` are identical, element for element**, which is
  what holds the two doors to one instruction. Every new refusal is in the
  machine's one vocabulary with a translator's note, reachable from a public
  `said()`, and no two of them read alike, as task 5 measured for the rest.
- **Constraint:** nothing here approves anything, and nothing here carries
  anything out — the approval is the person's and the broker's, the unit that
  runs the base is ADR 0053's and lives in that plan. This crate gains no road
  to the base, names no program, and starts nothing. The new constructor takes
  digests and the base's status; it does not read a registry, and it does not
  decide whether a build is vouched for, which is task 7's and stays where it
  is.

### 10. A machine that has never looked

**Status:** ready. **Depends on:** 6, 7.

**Done, 2026-09-19.** `crates/alo-looking-once` is the one caller of
`Because::ThisMachineStarted` on a booted machine, and a test reads every file
of it to count that there is exactly one and that nothing there could check a
second time. **Where it runs and under whose privilege, decided:** a system unit
started once at `multi-user.target`, running as **the person's own login**,
holding no capability at all — because both files a check writes are inside
`/var/lib/alo`, which the image makes `0700 alo alo`, so a login of its own
would need that folder widened, and root is refused on its own merits (a process
that reads what a public registry sent it is the last one to hold privilege,
ADR 0018). Nothing here needed a new privileged component or a widened grant, so
no ADR was owed. A **session hook was considered and discarded in writing**: the
answer a check keeps is one per machine and a session is not, so two sign-ins
would be two departures for one question and a machine nobody signs into would
never look. The record goes to a file of its own,
`/var/lib/alo/checking-for-updates.jsonl`, for `alo-brokerd`'s reason —
`alo_keeping::Writing` has one writer per file and the person's record already
has `alo-agentd` — and it is **opened before the first question**, so no road
here reaches the network without somewhere to account for it. `alo_looking::look`
gained `Noting` in the same change: until then **no check anywhere was ever
written into a record**, which is half of law 1 missing from the one errand alo
OS makes unasked.

**Measured on a booted machine, started twice** (`alo` at uid 1000,
`/var/lib/alo` `0700`, the unit installed byte for byte as this crate hands it
over): each start made exactly one check against the real `ghcr.io`, kept one
answer naming the running build, appended exactly one record entry, and
`systemctl start` on a machine that had already looked did nothing at all.
**Six TCP connections per check**, counted at the network boundary against a
start where the unit was masked — two questions, each answered `401` and asked
again with a token. A machine with no way out said so once, exited `1`, wrote
the refusal into the record and **left the kept answer untouched**.
**Found and fixed inside this task:** `Type=oneshot` put the check on the boot's
critical chain — a unit wanted by a target is implicitly ordered before it, and
a oneshot's start job is not finished until the process exits — so
`multi-user.target` was reached at 4.246 s against 1.9 s, and `graphical.target`
behind it. On a slow network that would have been the whole twenty seconds a
request waits, which is the constraint *a check at a start never delays a
person's sign-in* broken. `Type=exec` takes it off the chain, measured twice
more. **Handed over, not done here:** the image installation — the unit at
`/usr/lib/systemd/system/`, the program at `/usr/libexec/alo-looking-once`,
`systemctl enable`, and a `crates/alo-image` check holding the two to each other
— is the installer lane's, because this plan reads `image/` and never edits it.
Report: `docs/autonomy/updates/a-machine-that-has-never-looked.md`.

Written 2026-09-19 by task 9, because the plan named nothing after it and one
thing it promises is still done by nobody. `alo_looking::look` is built and
measured against the real registry; `Because` is the whole list of occasions and
it is two, *the person asked* and *this machine started*; and **nothing on this
machine calls either of them**. A check nobody performs is a capability rather
than a promise kept: from where the person sits, a machine that is never told an
update exists cannot be told apart from one that has none. *The person asked* is
a surface's and belongs to the shell plan. *This machine started* is nobody's,
and this is it.

The hard half is not the call. It is **where it runs and under whose
privilege**: the answer a check keeps is machine-wide
(`/var/lib/alo/an-update-was-found`) and a person's session is not, so a session
hook and a system unit are two different machines being described, and one of
them may need something no worker may decide.

- **Acceptance:** `Because::ThisMachineStarted` has exactly one caller on a
  booted machine, and it happens **once per start and never again** — no timer,
  no thread, no repetition, with a test that reads the caller's own source for
  each, as task 6's tests read `alo-looking`'s; where it runs and under whose
  privilege is **decided and written down with the reason**, and if the only road
  runs through a new privileged component or a widened grant then the ADR is this
  task's deliverable and the code waits on it (ADR 0001 §2); the check is on the
  indicator for the whole of it and is in the indicator's own record afterwards;
  a machine with no way out at all says so **once** rather than at every start,
  which `SaidOnce` already decides and this must not undo; and the whole of it is
  **measured on a booted virtual machine** — started twice, with the kept answer,
  the indicator's record and the departures counted at the network boundary read
  after each start, and the time it added to a start written down, because a
  check that delays a sign-in is a check that will be turned off.
- **Constraint:** a check at start never delays a person's sign-in, takes focus
  or asks them anything: `THE_RULE` is about an update and this is the thing that
  finds one, so the same promise holds here. Nothing downloads a build (task 6's
  constraint, unchanged), no setting turns checking off (task 1's), and
  **`alo-keeping-up` gains no clock, no socket and no file** — the test holding
  it to four dependencies stays exactly as it is. The unit or the session hook
  that performs the act is installed by the lane that owns the image or the
  session; this plan writes what it calls and hands the installation over, the
  way task 4 handed over the filesystem it needs.

### 11. The check on a machine that is not this one

**Status:** done. **Depends on:** 10.

**Done, 2026-09-20.** Both halves measured on a real bootc machine —
`alo-lane-b-bootc`, the pinned image `ghcr.io/aloworld-org/alo-os:0.0.4` at
`sha256:48bd5f31…` installed to a 20 GiB disk with `bootc install to-disk` and
booted under KVM on the development PC: Fedora 42, kernel 6.19.14, `bootc`
1.15.1.

**The first half answers the way the task feared.** As uid 1000 the real base
refuses: exit 1, nothing on stdout, and on stderr `error: Status: Preparing for
write: Querying root privilege: This command must be executed as the root user`
— for `--format json --format-version 1`, for `--format yaml`, for the human
form and for `--booted` alike, whether asked through `runuser`, through a login
shell, or by systemd with `User=alo`. The whole unit was then run as the login
it actually uses, and said exactly the sentence this plan predicted it would:
*alo-looking-once: this machine did not find out whether there is an update: the
base would not say which build this machine is running … — nothing left this
machine*, exit 1, nothing kept. As root the same program on the same machine
asked `ghcr.io` and kept its answer. **So on a real alo OS machine the unit
task 10 built fails at every boot**, and task 10's decision that the check runs
as the person cannot stand on `bootc status` as its source.

**The fix is decided, and no ADR is owed, because the road that works is not a
privileged one.** The base itself already writes, world-readable, what the check
needs: the booted deployment's origin file
(`/ostree/deploy/default/deploy/<checksum>.0.origin`, `0644 root root`) names
the image reference *with its digest*, and `ostree admin status` run as uid 1000
names which deployment is the booted one — both read as the person on this
machine, and neither asks the base's command for permission to say what the base
has already written down. So `alo_updating::running` gains a road that reads
that, the unit stays the person's, ADR 0018 stands, and **no new privileged
component and no widened grant is owed** (ADR 0001 §2 does not fire). What it
costs is one sentence of ADR 0011's *the base is spoken to through its own
command* — which holds unchanged for every act that changes the machine, and is
narrowed only for the one question the base refuses to answer to the person at
all. Applying it is the next task's: this task is two measurements.

**And the measurement found a second thing nobody was looking for.** On this
machine `status.booted.image.imageDigest` is `sha256:2e7ecd95…` while the image
reference, the spec and the origin file all say `sha256:48bd5f31…`. `Running`
reads the former and `Standing::between` compares it to what the registry
offers, so the kept answer on a machine running exactly the pinned build reads
`{"about":"sha256:2e7ecd95…","offered":"sha256:48bd5f31…"}` and the person is
told *a newer version of this machine's system is available* when there is none.
Both digests were measured for the same image — identical config id
`74a4aa15…`, local container store manifest `2e7ecd95…`, registry manifest
`48bd5f31…` — and this machine was installed from the local store, which is
where its `imageDigest` came from. **Not measured:** whether a machine installed
straight from the registry records the registry's digest instead. That is one
install away and is the next task's first line. The origin file, note, carries
the digest that matches what the registry offers, so the fix above happens to
close this as well.

**The second half went through the proxy and not around it.** With a real proxy
on the machine's own `/etc/alo-proxy/proxy.json` (`manual`, `10.0.2.2:3128`,
`set_by: an-organisation`) the check succeeded, the proxy at the boundary wrote
down six `CONNECT ghcr.io:443` departures — task 10's six, arriving as CONNECTs
— and counters inside the machine read **98 packets through the proxy and 0
straight out to port 443**. With the proxy set to a port nothing listens on the
check refused — *there is no way out of this machine to the place its updates
come from* — kept nothing, and left **exactly one packet, a lone SYN at the dead
proxy, and 0 straight out**: it did not go around. With a proxy file present and
not readable as a setting it refused before asking anything, *nothing left this
machine*, and no counter moved at all. Report:
`docs/autonomy/updates/the-base-answers-only-root.md`.

Written 2026-09-19 by task 10, because the plan named nothing after it and that
task's measurement leaves exactly two things unmeasured — both of them about
machines this one is not.

**The first is the one that could make task 10's central decision wrong.** The
check runs as the person and asks the base what this machine is running
(`alo_updating::running`, `bootc status --format json`). **Nothing in this
repository has ever run `bootc status` as anybody but root**: task 10's machine
had no base at all and answered with a stand-in, and every other caller of that
crate is a test with `/bin/echo` behind it. If the real base refuses an
unprivileged caller, then on a real alo OS machine the check answers *the base
would not say which build this machine is running*, keeps nothing, and the whole
of *updates that never interrupt* is a unit that fails at every boot — and the
answer is **not** to make this component root, which is what the unit's own
comments and ADR 0018 spend their length refusing.

**The second is the company network.** `alo-looking`'s road out is decided by
`alo_proxy::the_way` and handed to the client explicitly, and task 6's report
said plainly that a machine behind a proxy is owed a measurement of its own. It
still is. On a great many company networks there is no other route out, so this
is the difference between a fleet that finds out there are updates and one that
does not.

- **Acceptance:** `bootc status --format json --format-version 1` is run **as
  uid 1000 on a real bootc machine** and what it answers is written down —
  whichever way it goes, because *it refuses an unprivileged caller* is as much
  an answer as *it does not*; if it refuses, the fix is decided and written down
  with the reason, and if the only road runs through a new privileged component
  or a widened grant then the ADR is this task's deliverable and the code waits
  on it (ADR 0001 §2); and a check is made **through a proxy** — a real one on
  the machine's own `/etc/alo-proxy/proxy.json`, not a test's argument — with
  the departures counted at the network boundary to show they went through it
  and not around it, and a proxy that is set and unreachable shown to refuse
  rather than to go straight out.
- **Constraint:** nothing about what a check is changes here; this is two
  measurements and whatever one line each turns out to need. The image
  installation is **not** this task and is not this plan's: the unit, the
  program and the `systemctl enable` are the installer lane's, handed over in
  task 10's report. No setting that turns checking off, no member meaning
  *urgent*, and `alo-keeping-up` still gains no clock, no socket and no file.

### 12. What the base has already written down

**Status:** ready. **Depends on:** 11.

Written 2026-09-20 by task 11, because the plan named nothing after it and its
measurement leaves the unit it was about broken on every real machine. The check
runs as the person and asks `bootc status`; the real base refuses uid 1000 with
*This command must be executed as the root user*; so the unit fails at every
boot and keeps nothing. Task 11 decided the fix and is two measurements, not a
change: this is the change.

**The road is the one the base already leaves open.** Measured as uid 1000 on
`alo-lane-b-bootc`: the booted deployment's origin file is `0644 root root` and
names the image reference with its digest, and `ostree admin status` names which
deployment is booted. Nothing there is privileged, nothing is new on the machine,
and ADR 0018 is untouched. `alo_updating` gains that road for *which build is
this machine running* and keeps `TheBase` for everything that changes the machine
— where being root is correct and is the caller's, not this component's.

**Its first line is a measurement, not code.** Task 11 found
`status.booted.image.imageDigest` (`sha256:2e7ecd95…`) disagreeing with the
image reference, the spec and the origin file (`sha256:48bd5f31…`) on a machine
installed from a local container store, so the kept answer said *a newer version
is available* about the build it was already running. Install one machine
**straight from the registry** and read all four back: that says whether the
disagreement is the install's or the product's, and it decides how much of this
task is a fix and how much is a fix plus a bug.

- **Acceptance:** the check, run **as uid 1000 on a real bootc machine**, keeps
  an answer that names the build that machine is actually running — the same one
  the origin file names — and a machine that is running the newest build says so
  rather than offering itself an update; the digest measurement above is made
  and written down before the code is; what the base is asked is still exactly
  the arguments this repository decided, with no shell between them, for every
  act that changes the machine; and `ADR 0011` carries, in its own text, the one
  sentence narrowing *spoken to through its own command* to the acts that change
  the machine, with the refusal that forced it quoted.
- **Constraint:** nothing becomes root and nothing gains a capability — a fix
  that widens a grant is the fix this plan spent task 10 and task 11 refusing.
  No setting that turns checking off, no member meaning *urgent*, and
  `alo-keeping-up` still gains no clock, no socket and no file. The unit's
  installation remains the installer lane's.
