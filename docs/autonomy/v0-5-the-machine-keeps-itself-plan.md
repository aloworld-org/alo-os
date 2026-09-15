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
is, when it may happen, and what rolling back means. **It reads and never
edits** `alo-record` and `alo-keeping` (what the machine did, and where that
is written), `alo-capability` (what a verb is), `alo-egress` (an update is an
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

**Status:** ready. **Depends on:** 3.

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
