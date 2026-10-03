# A new machine becomes a lane

> **Current workflow, owner-approved 2026-09-18:**
> [SHARED_MAIN.md](SHARED_MAIN.md) supersedes direct-to-main, main-only,
> per-checkout build-cache and concurrent-build instructions below. Use one
> task branch and draft PR per task; progress pushes are allowed. Either PC
> may hold the shared integration turn and squash-merge its own task after all
> nine gates and acceptance pass on the exact combined tree. Existing direct-to-main publishers remain paused;
> the historical runner recipes below do not implement the new workflow.

Three more machines join the work on 2026-09-14: two spare PCs and the
certified laptop, which builds until the installer is ready to put alo OS on it
([ADR 0033](../decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)).
The development PC here is at its ceiling — two lanes gating at once leaves it
under a gigabyte of free memory, and a third made WSL refuse new sessions
outright. Every lane added *there* costs the lanes already running. A lane on
another machine costs nothing.

This document is what a person pastes into `claude` on each of them. The setup
is the same on all three; **only the plan differs**, and the plan is the whole
reason they do not collide.

## Who takes what

| Machine | Plan | Crates it owns |
|---|---|---|
| Spare PC one | `docs/autonomy/applications-and-what-they-expect-plan.md` | `alo-portals` (new), `alo-granted`, `alo-applications`, `alo-secrets` |
| Spare PC two | `docs/autonomy/the-machine-keeps-itself-plan.md` | `alo-keeping-up` (new) |
| The laptop | `docs/autonomy/documents-and-paper-plan.md` | `alo-printing` (new), `alo-opening` (new) |

**Reassigned 2026-09-15.** The machine that ran documents and paper
(`C:\dev\alo-os`, `admin.disan`) published tasks 1 and 3 and stopped with every
remaining task blocked on three real office documents from the owner. It now
runs **the machine keeps itself** as well, since spare PC two had not started
it, and still owns `alo-printing` and `alo-opening` for when documents and
paper unblocks. Nobody else takes `alo-keeping-up`.

## Every v0.5 plan, and who has it — as of 2026-09-15

*Lane B's checkout was named `alo-os-b` in this table until 2026-09-30. It is
`alo-os-lane-b`. The Panel lane went looking for `alo-dock`'s owner here, found
a checkout that does not exist, and reasonably concluded the lane was dormant —
while it was landing in that crate the same afternoon. **A crate can look
abandoned while its owner is active one directory over**, and this table is the
one place somebody checks before deciding that.*

## The ten plans that had no row, and who assigned them

**The owner assigned these rows to the Mac lane directly on 2026-10-03, in these
words:**

> The charter rows are yours. I am assigning them to you directly, on 2026-10-03, in
> these words. Write the rows for the ten plan documents that have no owning machine.
> […] Record in the change that I assigned this, on this date, in these words — so
> your own row is auditable by somebody reading in a month, which is the objection you
> raised and were right to raise.

**Quoted rather than summarised, and here rather than in a commit message, because of
what one of these rows is.** One of the ten is the lane writing them — a lane editing
the table it is judged against cannot be checked afterwards, and *this change was
assigned* is the one claim a change cannot support from its own contents. Two relays
of the owner's instruction arrived before this one and were held for that reason; what
made the difference is the owner saying it to this lane in their own words, with a
date, so that a reader in a month has the authority in the repository rather than in
somebody's account of a conversation.

### How the machine in each row was decided, and where the method is weak

#### A method that cannot reach the conclusion it was used for, 2026-10-03

**Resolving landed changes to a branch prefix tells you which prefix, never which
machine.** The `putting-a-window-aside.md` row counted eight on `task/panel/…`
and five on `task/dev-pc/…` — both counts correct — and then concluded *which
are one machine under two branch prefixes*. They are two machines. The honest
cell was *two prefixes, and this method cannot say whether they are one machine*.

**It had the consequence this table warns about.** The development PC read that
sentence, believed it, and told the owner twice that the third PC was idle and
had nothing to do. It had been working the panel plan throughout.

The same weakness is already recorded two sections below, where `crates/alo-dock`
is named **probable rather than measured** because a branch outside the
`task/<machine>/…` form cannot be resolved to a machine. Twice in one file is a
convention failing rather than two accidents, which is why the naming section
below exists.

**What to do instead.** Where the evidence reaches only the prefix, write the
prefix. Where a lane's own checkout would settle it, ask that lane — a machine
knows its own branches and no amount of history reading substitutes for it.


**Mentions were not used, and the first attempt at this used them.** Counting how
often a plan says *the Mac* makes `the-executable-plan.md` the Mac's forty-three
times over, when the word appears there because that plan **cites** every lane. A
count of references is not a measure of ownership — *count the thing, not the lines
that mention it*.

What was used, in this order of weight:

1. **An explicit statement of ownership**, in the plan or in a lane document.
   `a-loop-on-a-mac.md` says outright which plan a Mac lane runs;
   `kernel-enforcement-plan.md` says *who can take it*.
2. **Which machine has actually landed changes to the plan**, measured by resolving
   every pull request in its `git log --follow` to the branch that carried it, since
   branch names name the machine.
3. **The plan's former filename**, where a rename removed a lane code this lane
   itself removed in #396.

**And the measurement had to be corrected for a change of this lane's own.** #407
renamed twenty plan documents and touched 956 references, so it appears in the history
of nearly every plan here and inflated this lane's apparent claim on all of them.
#198 and #313 are fleet-wide status sweeps with the same effect. All three are excluded.
A measurement polluted by the measurer's own cross-cutting change is the shape of
fault this repository spent 2026-10-02 cataloguing, and it would have read as evidence.

**Where the evidence is thin it says so in the row.** Two of these rest on a single
landed change, and a row asserting an owner more confidently than its basis warrants
is worse than an empty cell: this table is *the one place somebody checks* before
deciding a crate is abandoned, and the note above this one exists because a lane read
a row and told the owner something false.

| Plan | Machine | Crates it owns |
|---|---|---|
| `the-models-measured-plan.md` | **the Mac** — stated outright: `a-loop-on-a-mac.md` says *the plan a Mac lane runs is `docs/autonomy/the-models-measured-plan.md`* | `alo-models`, `alo-driving`, `alo-choosing`, `alo-answering`, `alo-telling`, `alo-asking`'s hosted and served doors |
| `the-smallest-canvas-worth-showing.md` | **the Mac** — **closed 2026-09-27**, eleven of its thirteen landed changes from this machine | `alo-canvas`, and the one-Place half of `alo-shell` it rests on — see *shared ground* below |
| `the-canvas-and-its-places.md` | **shared: the Mac and the development PC** — seven landed changes from the Mac (tasks 1 to 6), and the plan names task 6a as the panel lane's and task 7 as the desktop lane's in their own status lines | `alo-canvas`, `alo-arranging`, and `alo-shell`'s canvas files — see *shared ground* below |
| `putting-a-window-aside.md` | **the third PC, with the development PC in it** — eight landed changes on `task/panel/…`, which is the third PC, and five on `task/dev-pc/…`, which is the development PC: `#406`, `#420` and `#426` are the agent-crossing work. **This cell read *one machine under two branch prefixes* until 2026-10-03** and that conclusion was wrong — see *A method that cannot reach the conclusion it was used for* below | `alo-put-aside`, and `alo-shell`'s panel files — see *shared ground* below |
| `applications-people-already-use.md` | **the development PC** — six of its seven landed changes | `alo-software`, `alo-convertd`, and the application-facing half of `alo-portals` |
| `kernel-enforcement-plan.md` | **the development PC** — stated in the plan: *who can take it — the development PC, inside a KVM guest with a real login* | `alo-bounding`, `alo-boundaryd`, `alo-agentd`'s enforcement path. **Also `tools/kernel-loop`'s default plan** — `plan.rs`'s `THE_PLAN` names this file |
| `accounts-and-session-entry-plan.md` | **this PC, lane B** — the plan says *this is lane B's*, and it was named `v0.01, lane B — accounts and session entry` until #396 | `alo-accounts`, `alo-entering`, `alo-greeting` |
| `providers-and-models-plan.md` | **this PC, lane B** — named `v0.5, lane B — providers and models` until #396. **Thin: one landed change.** Its task 4's successor *belongs to whichever loop takes it*, by the plan's own words | `alo-hosted`, and the provider half of `alo-choosing` |
| `the-machine-measured-plan.md` | **this PC, lane B** — its crates are named as lane B's where the Mac lane is told not to edit them. **Thin: two landed changes, and no statement of ownership in the plan itself.** Worth confirming before a lane relies on it | `alo-measuring`, `alo-finding`, `alo-files`' measuring path |
| `the-executable-plan.md` | **every machine, per task** — this is the one plan with no single owner by design: it is `tools/kernel-loop`'s input through `ALO_LOOP_PLAN`, and **its tasks carry their own owners** in their own lines (*Owner: Claude, while the desktop worker is away*; *lane B's*). A machine named here would be wrong for most of its tasks | none of its own; its tasks name the crates they touch |
| `the-local-network-plan.md` | this PC, lane A (`alo-os-claude`) | `alo-nearby`, parts of `alo-agentd`/`alo-turn`/`alo-egress`/`alo-bounding*` for pairing |
| `the-installer-plan.md` | **third PC, first loop, from 2026-09-16** — it needs 50 GB free for the three virtual-machine tasks, which the development PC has not | `alo-installer`, `alo-installing`, `image/`, `alo-image`, `.github/workflows/` **except `gate.yml`** — see the exception below |
| `where-a-persons-settings-are-kept-plan.md` | **this PC, lane B (`alo-os-lane-b`), active from 2026-09-18** - task 7, while hands-on tasks 2 and 7 wait on display identities | `alo-appearance`, `alo-dock`, `alo-shortcuts`, `alo-choosing`, `alo-changing`, `alo-kept` |
| `applications-and-what-they-expect-plan.md` | **the Mac** | `alo-portals`, `alo-granted`, `alo-applications`, `alo-secrets`, and ADR 0040's change to `alo-capability`/`alo-remembering` |
| `the-machine-keeps-itself-plan.md` | third PC, behind the installer plan — undo waits on the installer's task 11 for a filesystem that can snapshot | `alo-keeping-up` |
| `documents-and-paper-plan.md` | **this PC (`alo-os-shell` checkout), from 2026-09-16** — the owner's three documents arrived, and this plan needs no virtual machine | `alo-printing`, `alo-opening`, `alo-converting` (new) |
| `the-shell-plan.md` (tasks 7-14) | **this PC, lane A (`alo-os-claude`), from 2026-09-18** - assigned by the owner after the local-network plan finished; starts with task 8, whose lock-state dependency is done. Other tasks retain their dependencies | `alo-shell`, `tools/graphics-check` |
| `access-and-language-plan.md` | **the Mac**, after applications | `alo-access`, `alo-conforming`, `alo-formats` (new), the answering-language clause of `alo-instructing` |
| `models-a-person-adapts-and-subscribes-to-plan.md` | **the Mac**, after access and language | `alo-adapting`, `alo-hosted` (new) |
| `software-and-the-web-plan.md` | **third PC, second loop** (`C:\dev\alo-os-2`) | `alo-software`, `alo-proxy`, `alo-adapters` (new) |
| `the-broker-and-the-disk-plan.md` | third PC, second loop, after software and the web | `alo-broker`, `alo-encrypting` (new) |
| `capture-and-the-room-plan.md` | **the Mac, from 2026-09-17**, tasks 3 to 7 — tasks 1 and 2 were published and the plan then sat untouched for twenty-six hours with no machine holding it. Its tasks 4, 5 and 7 waited on the devices plan's codec decision, which the same lane then took and wrote as ADR 0051 | `alo-capturing`, `alo-in-use` |
| `the-session-and-the-displays-plan.md` | **third PC, first loop, from 2026-09-16** — it needs no virtual machine, and that loop waits on the installer plan's signed release and a machine with hardware virtualisation | `alo-locking`, `alo-sleeping`, `alo-displays`, `alo-notifying` (new) |
| `hands-on-the-desktop-plan.md` | **this PC, lane B (`alo-os-lane-b`), from 2026-09-17** — taken ahead of its queue because `alo-keyboards` is what the Mac's access-and-language tasks 3 and 4 wait on | `alo-dividing`, `alo-desktops`, `alo-keyboards` (new) |
| `devices-and-media-plan.md` | **the Mac, from 2026-09-17** — taken for its task 1, the codec decision, which was blocking capture tasks 4, 5 and 7 on the same machine | `alo-sound`, `alo-bluetooth`, `alo-playing`, `alo-power`, `alo-cameras`, `alo-media-server` (all new) |

**`gate.yml` belongs to the desktop lane, assigned by the owner 2026-10-01.**
## One machine per crate, from 2026-10-03

**The owner gave the canvas to the Mac end to end** — `crates/alo-shell`,
`crates/alo-canvas` and `crates/alo-arranging` — and with it
`applications-people-already-use.md` task 3, the one task of another plan that
edits the shell. The third PC keeps `putting-a-window-aside.md` and `alo-put-aside`. The
development PC takes three v0.5 promises chosen so that it edits no crate either
of the others owns, verified by reading dependencies rather than by intending it:
`alo-appearance` reaches the propose-then-approve road through `alo-saying`, so
it consumes the Mac's `alo-asking` and opens it never.

### *A panel out of view costs nothing* is the Mac's, and it was assigned twice
by one misread word

It sat with the development PC because `alo-put-aside` looked like where the
work was, then moved to the third PC on the owner's rule that we pick the ways
which do not block each other. **Both readings rested on the word *panel*
matching the name of a crate**, and the promise does not mean that panel.

Read rather than matched, all three in `## The interface (ADR 0065)`:

| | |
|---|---|
| `:441` `[v0.01]` | objects **and applications alike open as panels on it** |
| `:493` `[v0.5]` | **a panel** out of view is a still picture until it is reached, so a Place holding forty things **is not forty programs running** |
| `:494` `[v0.5]` | every canvas answers as a list — **its panels** in order |

A *panel* there is a frame on a Place. **Forty things is not forty programs
running settles it**: the put-aside surface holds previews of windows somebody
set aside, never forty running programs. So the work is `alo-canvas` and
`alo-shell`, which are the Mac's.

**The reassignment made to remove a collision would have created one** — the
third PC writing a task into their own plan and then needing two of the Mac's
crates to do it. The third PC found this before writing that task.

**The general fault, which this fleet has now hit three times:** a name correct
in two vocabularies cannot be told apart by a query, only by reading the sentence
it sits in. `alo-arranging` keeps where a person left their canvas and
`alo-displays` keeps the monitor arrangement — both are *arrangement*. *Panel*
is the same word twice, and the sentence says *programs*.

**Why a crate needs an owner at all, when a plan already has one.** `#432` made
it fail if a plan with tasks has no owning machine. Nothing of the kind existed
for crates — and crates are what two machines actually collide in. Measured on
2026-10-03, **six crates named here carried more than one machine and one carried
none**:

| Crate | Was | Is | How settled |
|---|---|---|---|
| `alo-shell` | the Mac, shared, the development PC | **the Mac** | the owner's decision |
| `alo-canvas` | the Mac, shared | **the Mac** | the owner's decision |
| `alo-portals` | the Mac, the development PC | **the third PC** | it holds the USB-portal promise |
| `alo-software` | the development PC, the third PC | **the third PC** | it holds the unsandboxed-installation promise |
| `alo-choosing` | the Mac, the development PC | **the Mac** — *thin* | no evidence either way |
| `alo-hosted` | the Mac, the development PC | **the Mac** — *thin* | no evidence either way |
| `alo-saying` | **nobody** | **nobody — it has a rule instead** | see below; an owner would make every lane's ordinary work a trespass |

**The two marked *thin* are marked, not hidden.** Neither has evidence behind it,
and a confident cell that sends somebody to the wrong lane is worse than one that
says it does not know — which is the lesson of the row corrected above.

### `alo-saying` has a rule and no owner, and the first attempt gave it one

**It was assigned to the development PC and marked *thin*, and that was wrong.**
The reason given was that this is the only lane with work reaching through it
today — which is a reason to touch a crate, not a reason to own one, and marking
a cell thin does not make a wrong cell right.

Measured instead. Every change to it since `#247` came from work in a different
subject: the canvas, access, measuring, another machine's windows, the Dock and
the installer. **Twelve changes, twelve subjects, each one a side-effect of work
somewhere else.** About fifty crates declare it, and its own header says it is
*who loads them* and that neither of the two things it does is a decision about
words.

**So it is not a crate anybody owns. It is a crate everybody appends to**, and
the rule is:

> A crate adds its own words to `alo-saying` as part of the change that needed
> them. That is not shared ground and needs no announcement — adding a key is
> how a crate speaks. **Changing how loading works is different**, and whoever
> does it says so first, because every other crate's words arrive through it.

This is the argument the Mac made about `alo-shell`, and it was right until
phase 2 finished and the claim went dormant. **`alo-saying` never goes dormant**,
because adding words is how every crate speaks. A rule costs nothing; an owner
costs every lane a question it should not have to ask.

## What the shared-ground argument said, and why it was superseded

**The section below argued that `crates/alo-shell` should have no single owner,
and the argument was sound on the evidence it had.** Two lanes edited it
correctly under the owner's direction in the same hours on 2026-10-02, and a
compositor holding the canvas, the Dock's bounds, the panel, the status area and
every window road is a crate several plans must reach into; naming one owner
would have made three lanes' assigned work a trespass.

**The owner decided otherwise on 2026-10-03**, and what changed is that phase 2's
plans finished — `the-shell-plan.md` at 17 of 17 and
`the-session-and-the-displays-plan.md` at 14 of 14. `alo-shell` was the only
crate claimed by two phases, and with phase 2 complete the claim is dormant, so
one owner costs nothing it would have cost a day earlier.

**The argument is kept rather than deleted** because somebody will propose
sharing a crate again and should find what was actually weighed, not a blank.

## Shared ground: the crates more than one lane may edit

**`crates/alo-shell` is in no lane's row, and that is not an omission to be fixed by
giving it to one.** Two lanes edited it correctly under the owner's direction in the
same hours on 2026-10-02 — the panel lane's grouping and the laptop lane's
persistence — and a compositor that holds the canvas, the Dock's bounds, the panel,
the status area and every window road is a crate several plans must reach into. Naming
one owner would make three lanes' assigned work a trespass.

Measured rather than assumed, by resolving the pull requests against each crate since
2026-09-25 to the machine that carried them:

| Crate | Machines that have landed in it | How |
|---|---|---|
| `crates/alo-shell` | the Mac, the development PC | canvas files, panel files, the draw, the fixed controls |
| `crates/alo-desktop` | the Mac, the development PC | the binary that stands a desktop up; **it has zero `#[test]`**, which is a finding two lanes hit in one day and is with the owner |
| `crates/alo-reconciling` | the Mac, the development PC | the gate reads every plan, so a change to any plan's shape reaches it |
| `crates/alo-dock` | the development PC, and one branch this method could not attribute | named here as **probable** rather than measured, because a branch outside the `task/<machine>/…` form cannot be resolved to a machine |

**`alo-canvas`, `alo-arranging` and `alo-put-aside` are *not* shared** on the same
measurement — one machine each — and are left in their plans' rows. Shared ground is
what the evidence shows, not every crate two plans mention.

### The rule, which is the one those two lanes were already following

The owner's words, 2026-10-03: **say which files before you start, pull before you
branch, and tell the other lane when it lands.**

It is written down because it worked by convention through a night when two lanes were
inside one crate, and a convention nobody has written is one the next machine cannot
follow. What each part is actually for:

- **Say which files before you start.** Not *which crate* — two lanes were in
  `alo-shell` all night and never collided, because one was in the panel's files and
  one in the canvas's. Crate-level announcements would have made them refuse work they
  could safely do.
- **Pull before you branch.** Three plan-document citations went stale inside one hour
  on 2026-10-02 while a branch was gating, and two pull requests were ejected from the
  merge queue as `DIRTY` because `main` moved under them. Branching from a fetched
  `main` costs one command; a rebase after an ejection costs a full verification run.
- **Tell the other lane when it lands.** The expensive case is not a conflict, it is a
  lane building on a fact that stopped being true: a close was drafted saying the
  status area *cannot* join the fixed-control set twenty minutes before another lane
  made it join, and was caught only because that lane sent a message. A conflict git
  will find for you. A stale premise it will not.

**And what the rule does not do:** it does not make a boundary negotiable between
lanes. Shared ground means several lanes may edit these crates *under the owner's
direction*; it does not mean a lane may take another lane's task in one of them
because it could. A peer saying work is yours is not the owner saying so — which is
why the rows above carry the owner's own words and a date.

The installer row above carries `.github/workflows/` as a whole, and that stopped
being true some days before this line was written: `.github/workflows/gate.yml`
has been the desktop lane's, with the owner's knowledge, through `#285`, `#287`,
`#308` and `#314`. The owner's words: **desktop owns
`.github/workflows/gate.yml` and the gate automation directly supporting it;
installer retains its installer and image workflows.**

It is recorded as a **specific-file exception** rather than by moving the whole
directory, because the two halves have genuinely different owners: the image and
installer workflows belong with `image/` and `alo-installer`, and the gate
workflow belongs with whoever is answerable for what a gate means.

*What makes this worth more than a corrected cell.* The dev-PC lane read this row,
concluded those four CI changes were the installer lane's, and told the owner that
the configuration change in [ADR 0081](../decisions/0081-the-runner-posts-the-check-main-requires.md)
should go to the lane already furthest into it — **which was the wrong machine.**
The row was the only evidence it had, and that is not an accident of this row:

**Commit authorship must never be used to infer lane ownership.** Every commit on
`main` reads the owner as author and GitHub as committer, because the merge queue
squashes. *No lane's work in this repository can be told apart from another lane's
by its commit metadata.* So this table and each lane's own account of itself are
the only records there are — which is why a stale row does not merely mislead, it
propagates into conclusions nobody can check against the history.

**Narrow printer producer contribution, authorized 2026-09-18.** The owner told
the third PC, "no you should do all the blockers by yourself so no need to lean
on the other pc". For **broker task 2, Printers, through the broker**, this
releases the preserved `alo-printing` additions: `printers_set_up`,
`Found::as_reported`, `Printer::as_reported`, `remove`, `make_default`,
`CannotChange`, their re-exports, contract comments and producer tests, plus the
measured broker socket authentication fix and its real-CUPS acceptance.
The documents plan records the exact released files for this task alone. The third PC integrates that producer contribution with its receiving
broker task publication; the report is
`docs/autonomy/updates/printers-through-the-broker.md`. The documents plan and
its completed work retain their owner, as do all shell and settings work. This
does not transfer the documents plan or authorize any other producer API.
Worker validation is deferred to the supervisor's nine gates and named task
acceptance checks; the ownership release changes none of those requirements.

**`alo-media-server` is owned here and read elsewhere.** It holds reaching the
rented media server and reading what it says, for the four crates that were each
doing it their own way — `alo-in-use` and `alo-capturing` (the capture plan's),
`alo-sound` and `alo-cameras` (this one's). It is listed here so that nobody adopts
it later as unowned: **the devices and media plan owns it, and a change to it is that
plan's lane's**. A crate that wants to read the media server reads this one rather
than starting a fifth copy.

**Why this division.** The Mac holds the small model and runs no virtual machine
well, so it takes the plans that need a model and no hardware: access and language
(the agent answering in each language is measured on the small model) and models a
person adapts. The third PC has the memory for two loops and already runs a virtual
machine, so its second loop takes software and the broker, whose tests use virtual
disks. This PC stays on the installer, which is the critical path, and inherits the
shell plan because it already gates the compositor. Capture goes first on spare PC
two because its task 1 is what the session plan's notifications and the devices
plan's camera wait on. **If spare PC two is not started, its four plans are the queue
for whichever machine empties its own list first.**

The eight plans written on 2026-09-15 own **new crates only** (plus one clause of
`alo-instructing`, whose plan had finished), so any of them can go to any free
machine without meeting a lane. The one exception is the shell plan: exactly one
machine may hold it at a time. **Assigning a plan means editing its row here in the
same commit that starts the lane**, so that the table is always the truth.

No two of those touch one crate, and none of them touches `alo-shell`,
`alo-nearby`, `alo-asking`, `alo-agentd`, `image/` or `alo-image`, which have
lanes on them. **Two lanes in one crate corrupted a task on 2026-09-11; that is
why the table exists.**

**`alo-asking`'s hosted door and `alo-agentd`'s question road, taken
2026-09-20.** The software-and-the-web plan's task 11 carries the machine's
proxy to the question a turn puts, and the road it has to reach runs through
`alo_asking::Hosted` and `alo_asking::openai` — the **models-measured plan's**
crates, and that plan records itself *Finished, 2026-09-15*. Under *a machine
unblocks itself* below, a blocker in a plan that has finished is taken rather
than waited on, so the software-and-the-web plan holds `alo-asking/src/hosted.rs`
and `alo-asking/src/openai.rs` from that date. **Not `alo-asking/src/corridor.rs`
or `src/held_to.rs`**, which are the local-network plan's and which task 11 read
and did not change. `alo-agentd` was already this plan's to edit for the machine
description (`machine_wide_proxy.rs`, task 9); task 11 adds `the_road_out.rs`
beside it.

## A machine unblocks itself

**Changed 2026-09-18, by the owner.** A machine no longer reports that it is
waiting on another machine and stops. It takes the thing blocking it.

- **A blocker in a plan nobody holds, or in one that has finished** — take it,
  and edit its row in the table above in the same commit, so the fleet can see
  who holds it now.
- **A blocker that is a decision** — write the ADR. You are the machine that
  understands why it matters; waiting for one with less context to decide it is
  worse rather than safer. Two exceptions: what needs the owner personally, and
  what needs a lawyer.
- **A blocker inside a crate another machine's lane is working right now** —
  the one case to coordinate. Say so, and take your next ready task meanwhile.
- **Never idle.** Stopping while work is available is the one outcome that is
  always wrong. If a lane empties its plan it says so *and takes the next thing*
  rather than waiting to be told.

**Why this is safe now and was not before.** Crate ownership existed because two
lanes editing one crate on a shared `main` corrupted a task on 2026-09-11. Under
`docs/autonomy/SHARED_MAIN.md` each task has its own branch and its own pull
request, so two machines in one crate now meet at merge time as a conflict
somebody can see and resolve. The table remains the record of who holds what; it
is no longer a reason to sit still.

**It works.** On 2026-09-17 the Mac was blocked on a codec decision and on the
media kinds `alo-playing` needed. It took both and published inside the hour,
while three other machines were idle waiting on each other.

When a machine takes a blocker, its report says **whose plan it came from and
why nobody was on it** — so a plan being quietly abandoned is visible, rather
than inferred later from a count.

## Before the prompt: what the machine needs

The full list is `docs/autonomy/the-laptop-as-a-development-machine.md`, which
applies to any Windows machine, not only the laptop. In short: WSL2 with Ubuntu
24.04; inside it, stable Rust and the pinned nightly with `rust-src`, LLVM 22
and `bpf-linker 0.11.0`, the graphics and udev development packages
`docs/autonomy/GRAPHICS.md` lists, `gnome-keyring` and `dbus`, and `bpffs` at
`/sys/fs/bpf` in `/etc/fstab`; gates run as root. On Windows: git with push
rights, the `claude` CLI, and Rust to build the supervisor. Then a `.wslconfig`
sized to that machine and one `wsl --shutdown`.

A Mac runs a lane too, but it cannot gate natively — the supervisor refuses to
on macOS and drives a Linux machine instead. That is
`docs/autonomy/LOOP.md`'s bridge, not this document.

### Sizing that machine, so the gates are not the slow part

A lane spends most of its wall-clock inside the nine gates, and on 2026-09-16
this PC was found to have been running them on under half of itself: the
`.wslconfig` said `processors=6`, above a comment claiming that was "six of the
eight cores", on a machine with fourteen. Check the real number rather than the
one in the file — `nproc` inside the guest against the host's own count —
because that arithmetic quietly taxes every task the machine ever runs.

Two settings, kept apart on purpose:

- **Cores** (`processors` in `.wslconfig`) go to all but two, leaving the host
  git, the editor and the supervisors themselves. Cores are cheap: the ones
  above the job count are used *inside* each `rustc` and by the linker.
- **Concurrent compiles** (`CARGO_BUILD_JOBS` in the environment a gate runs
  in) are what set peak memory, and memory is what actually breaks a machine —
  `Wsl/Service/0x8007274c`, the guest paging, a finished task losing its lane.
  Pin it, at about half the cores where two lanes share one guest.

Raising the jobs along with the cores raises both, and that is the change that
ran this machine out of memory before. Raise the cores; pin the jobs.

Install **mold** (`apt install mold`) and give the gates
`RUSTFLAGS="-C link-arg=-fuse-ld=mold"`. Linking is the serial tail of every
crate and very nearly the whole of a rebuild that changed one line. Unset
`RUSTFLAGS` for the two BPF gates: that target is not linked by anything of
ours, and the flag would be handed to a linker that is not there.

### What a checkout and a branch are called

`CLAUDE.md` says **names are for strangers: files, commit subjects and branches
describe the subject matter.** It does not reach checkouts, and the names in this
file show why it should:

```
alo-os-lane-b    a lane letter
alo-os-b         a letter
alo-os-claude    a tool, which dates
C:\dev\alo-os-2   a number
alo-os-shell     the work it holds   <- the only one a stranger can read
```

**A number tells nobody anything.** But unreadability is the smaller half, and
naming it as the whole would send the next person looking for a directory to
rename instead of a distinction to stop relying on.

**The real fault is that a path identifies a checkout and this table used it to
identify a lane.** A machine holds as many checkouts as it needs; the third PC
reported six beside each other on 2026-10-03, of which `C:\dev\alo-os-2` — the
one this table gives `software-and-the-web-plan.md` and `alo-software` to — is
**dormant rather than absent**: real, on `main`, tree clean, untouched for eleven
days. So the cell is not wrong about a path. It is wrong about what a path can
tell you.

**There is no one-to-one between a checkout and a lane**, and no spelling of the
directories would have created one. A machine identifies a lane; a checkout says
only where some work happens to sit.

So, from 2026-10-03:

- **A lane is identified by its machine, never by a checkout path.** This is
  the rule the rest follow from, and the reason the paragraph above exists.
- **A machine's own checkout is `alo-os`.**
- **A second worktree is named for the work it holds** — `alo-os-shell` is the
  example that was already right. Never a number, a letter, or a tool.
- **A branch prefix is always the machine**, and the rest of the branch describes
  the subject: `task/<machine>/<what-it-does>`. `task/panel/…` breaks this by
  putting work where the machine belongs, and that single exception is what made
  two prefixes read as one machine. **A convention that holds everywhere except
  once is worse than none**, because the exception is invisible to somebody
  reasoning from the pattern.

**Nothing is renamed by this entry.** A dozen scripts hardcode a checkout path
and branches were in the merge queue when it was written; renaming three
machines' directories at once buys a broken gate runner and no correctness. Each
lane moves its own when it is quiet, and `task/panel/…` stops being created from
today.

### Where the checkout lives, which is worth more than every other setting here

**Do not gate a checkout that lives under `/mnt/c`.** WSL reads it across the
bridge to the Windows filesystem, file by file, in every gate. Copy it onto the
Linux filesystem instead — `rsync -a --exclude target /mnt/c/dev/<checkout>/
/root/<checkout>/`, twenty-one seconds for a 65 MB repository — and gate there.
Measured on the development PC on 2026-09-17, the same nine gates on the same
commit, with the same cores, linker and job count:

| Gate | under `/mnt/c` | on the Linux filesystem |
|---|---|---|
| fmt | 75s | 3s |
| clippy, warnings denied | 61s | 3s |
| the workspace's tests | 1664s | 988s |
| rustdoc | 88s | 56s |
| the BPF target's formatting | 72s | 6s |
| **all nine** | **1907s** | **1085s** |

Thirty-two minutes to eighteen, and the part that is not actually running tests
fell from about four minutes to ninety seconds. What is left is the
network-namespace tests, which take real time wherever the files sit.

The wall-clock is not the point. **A lane whose gate takes half an hour cannot
win a push race** against a fleet that pushes every ten to twenty minutes: it
gates, `main` moves, it rebases and gates again. One task lost four races that
way in a single morning and published nothing, while nothing at all was wrong
with it. Halving the gate is what makes a lane able to finish.

### The disk only grows unless you tell it not to

`ext4.vhdx` never shrinks by itself. Deleting files inside returns nothing to
Windows, and `diskpart compact vdisk` reclaims nothing at all on a disk that is
not sparse — measured at 155.5 GB before and after. A lane machine therefore
fills up and stops, and every symptom looks like something else: a worker that
died mid-task, a supervisor that could not even write its own lock file, gates
blaming the work twice in a row.

Do this once, before it happens: `wsl --shutdown`, then
`wsl --manage <distro> --set-sparse true --allow-unsafe`, then `fstrim -av`
inside the guest. Microsoft marks the conversion unsafe; with the distro shut
down and nothing inside it but build directories that rebuild, it is worth
taking. On this PC it turned **2.8 GB free into 53.5 GB** once a stale 65 GB
`target` was deleted — space that had been unreachable the day before.

Cargo target directories are what fills the disk: one per checkout, tens of
gigabytes each. A machine that changes where it builds should delete the
directory it stopped building in.

## The prompt

Paste this whole thing, with the one line marked below replaced by that
machine's plan from the table.

---

You are joining a team already building alo OS, a sovereign AI-native
operating system meant to replace Windows. Several other agents are working in
this same repository right now, on other machines. Read that sentence twice:
**the repository moves under you while you work**, and everything below follows
from it.

**Your authority.** You do not need permission for anything. Make decisions,
write code, write ADRs, change what needs changing, and push it. Do not stop to
ask whether something is allowed. If a decision is big enough to need
recording, record it as an ADR in `docs/decisions/` and carry on — that is what
they are for. The only things to raise rather than do are the ones a person has
to be physically present for, or that cannot be undone.

**Set yourself up:**

```
git clone https://github.com/aloworld-org/alo-os C:\dev\alo-os
cd C:\dev\alo-os\tools\kernel-loop
cargo build --release
```

**Your plan — replace this line with your machine's row from the table:**

```
$env:ALO_LOOP_PLAN = "docs/autonomy/applications-and-what-they-expect-plan.md"
```

**Then run the loop:**

```
cd C:\dev\alo-os
$env:ALO_KERNEL_LOOP_WORKER = "$env:APPDATA\npm\claude.cmd --dangerously-skip-permissions -p"
.\tools\kernel-loop\target\release\alo-kernel-loop.exe run
```

`status` says what it is doing, `stop` asks it to finish the task in hand and
exit. Never start a second loop in one checkout — a second lane on this machine
is a second clone in its own directory with its own plan.

**Read these before you write anything**, in this order: `CLAUDE.md` — it is
absolute and it is short; your plan document, all of it, including the header
above the tasks, which says what your plan may not do; `docs/autonomy/LOOP.md`;
and the ADRs your plan cites. `docs/features.md` is the only list of what gets
built and `ROADMAP.md` the only order.

**The five laws you are held to.** Nothing leaves the machine silently. No code
runs unless the person chose it. Done means the machine still works — on real
hardware, not in a test. One file, one responsibility. The person chooses, and
alo never takes the choice away. A change that breaks one
of these is wrong however well it is written.

**Git, and the others.**

- **One branch: `main`.** There are no feature branches and no pull requests.
- **Pull before you start a task and again before you push.** A refused push
  means `main` moved while you worked: rebase onto it, **gate the combined tree
  again**, and push. Never resolve it by discarding somebody else's work, and
  never force-push.
- **Push every finished task.** Work that sits unpushed is work the other
  machines will collide with.
- Commit as the git user already configured, with **no `Co-Authored-By`
  trailer**. Commit subjects describe the subject matter — `type(scope):
  subject` — and never mention task numbers or plan names.
- **Before you take an ADR number or a task number, `git pull` and look.**
  Numbers are shared across machines and have collided three times.

**What done means, and what it does not.** All nine gates pass: formatting;
clippy across the workspace with warnings denied; the workspace's tests; the
supervisor's formatting, clippy and tests; rustdoc with warnings denied; the
BPF target's formatting and clippy. `unsafe_code` is forbidden workspace-wide,
and `unwrap`, `expect`, `panic` and slice indexing are denied outside tests. No
`todo!()`, no stub, no half-path. **When time is short, cut scope — never
depth**: a narrow thing that fully works ships; a wide thing that half-works
does not.

And the line that matters most on a new machine: **nothing you measure here is
a certification.** A green suite, a timing, a boot in a virtual machine is a
fact about the thing measured, never a fact about alo OS on certified hardware.
Tick `- [x] The code.` and leave `- [ ] On the machine.` alone — only a person
standing at the certified machine writes into `docs/autonomy/evidence-it-boots-and-the-agent-acts.md`.
A plan that tells you to stop and report a finding means it: an open task with
an honest finding is worth more than a closed one with a guess.

---

## What to expect in the first hour

The first gate run builds the whole workspace and takes a while; later ones are
minutes. The supervisor keeps its build directory apart from the checkout, on
the filesystem the gates run on. If it says it could not make one and fell back
to the checkout's own `target/`, the machine is out of memory or WSL is not
answering — fix that before letting it build, because the fallback is slow
enough to look like a hang.

### Narrow wallpaper handoff, 2026-09-18

The owner assigned the approved Quiet Horizon wallpaper installation to this
PC together with shell task 8. Lane A may add the default-picture COPY mapping,
its alo-image acceptance test and the shared wallpaper lookup contract. All
other installer/image work remains with the third PC. The source artwork was
already published in c69a044; this handoff does not choose new artwork or
change the default appearance.
