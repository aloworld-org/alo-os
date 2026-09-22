# v0.5 — the broker and the disk: the machine's own authority, and its encryption

**Workstream:** two `ROADMAP.md` v0.5 lines that are the two places alo OS holds
authority above the signed-in person — ★ *System verbs through the privileged
broker* (*printers, network, updates, storage*); and *Full-disk encryption*
(`docs/features.md`: *enrolled at install*). They belong together because each is
**a key to the whole machine**, and each is only safe if it is small, closed and
decided before it is built.
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan.
ADR 0001 §2 already decided the broker's shape; nothing has decided the disk's.

**Crates this plan owns, all new:** `crates/alo-broker` (the privileged broker: its
fixed verb list, its door, and nothing else — the third privileged component after
`alo-boundaryd` and `alo-sessiond`, and held to their rule of doing one kind of
thing) and `crates/alo-encrypting` (what full-disk encryption is on this machine:
how it is enrolled, where the key is, and how a person recovers) — and, added by
task 6 on 2026-09-20, `crates/alo-enrolling` (what a person is told on that road:
one sentence per refusal, with a translator's note), and by task 7 on 2026-09-20,
`crates/alo-changing-drives` and `crates/alo-changing-updates` (what a person
reads when the broker's two storage verbs and its two update verbs answer — the
surfaces the printers and the network already had and these two verb families did
not). `alo-encrypting` and `alo-enrolling` are separate
crates because `alo-encrypting` depends on nothing and must keep depending on
nothing: it holds a recovery key for the length of one screen, and a vocabulary
would bring a serialiser into reach of it. **It reads and
never edits** `alo-printing` (except the exact owner release below), `alo-capability`, `alo-turn` and `alo-protocol` (a system verb is a
verb, proposed and approved in a turn — if the verb list or the door must change,
that is a finding and a decision), `alo-keeping-up` (the machine-keeps-itself
plan's — what an update is),
`alo-software` (the software plan's — an application install is not a system verb),
`alo-boundaryd` and `alo-sessiond` (the pattern a privileged component follows),
`alo-record`, `alo-egress`, `image/`, `alo-image` and `alo-installer` (the installer
plan's — enrolment happens during install, and what this plan decides is handed to
it), and `alo-saying`. **Nothing in `crates/alo-shell`.**

**Printing producer scope, owner-authorized 2026-09-18:** the documents plan
retains alo-printing. Its owner-release header names the exact files released
to this task: the preserved producer API, the kernel-verified broker socket
credential, and their tests. No other producer file or task is released. Both
supervisor scope checks still refuse everything outside that exact record.
The receiving task's report and the roster record the owner's authorization.
**What this plan may not do:** tick anything *on the machine* — an encrypted disk that
has never booted on certified hardware is `- [x] The code.`; add a free-form
parameter, a shell, a path argument or a *run as root* verb to the broker, ever (ADR
0001 §1–2); write disk-encryption, TPM or network-configuration code of our own (LUKS,
`systemd-cryptenroll`, the TPM stack, NetworkManager and CUPS are rented, configured
and never patched, ADR 0011); or make any verb that could lock a person out of their
own disk without the recovery road task 5 decides. Before writing the next task,
`git pull` and read the plan as published.

**One verb released to the installer plan, owner-authorized 2026-09-22.** The
owner instructed the third PC to build the alo OS side of installing alongside
Windows, and said of it: *"`SystemVerb` is a closed enum: adding a verb is
deliberate, so give it its name, its words and the same tests the other verbs
have."* That verb is `RestartIntoWindows`, which sets the firmware's next start
for **one** start and leaves the default untouched. It belongs in this plan's
crate, and this plan is unfinished — its task 9 waits on a certified machine —
so the supervisor's ownership check refuses it, correctly. This block is the
record that the owner authorized it, and it releases **one file and no others**.

Nothing else about `alo-broker` moves: the enum stays closed, the verb takes an
`Identity` the firmware reported rather than a path or a string, and it gets the
same tests every other verb has. ADR 0045's seventh term is untouched — no name
on the list begins `undo.`, and the test that holds that is unchanged.

```owner-release
plan = docs/autonomy/v0-5-the-installer-plan.md
task = 16
files =
  crates/alo-broker/src/verbs.rs
```

## Tasks

### 1. The broker: a closed list, a door, and nothing else

**Status:** **Done, 2026-09-16.** **Depends on:** nothing. Report:
`docs/autonomy/updates/the-brokers-closed-list-and-door.md`.

*The broker is small enough to be audited in an afternoon, and that is a constraint on
its design rather than a hope about its future* (`docs/contracts/agent-verbs.md`).

- **Acceptance:** `alo-broker` holds its verb list as a closed enum, each verb's
  arguments as closed types with **no string that becomes a path, a command, a
  configuration line or a device name** — held by a test that walks every argument
  type and fails on a free `String` or `PathBuf`; it listens on one socket, accepts a
  request only from `alo-agentd`'s own credentials checked by the kernel (the peer's
  credentials, not a claim in the message), and **only for a verb a person approved
  in a turn**, which it verifies by a token the turn issues rather than trusting the
  asker; every request, permitted or refused, is recorded through `alo-record` before
  it answers; and a line count and a dependency list are held by a test so the crate
  cannot quietly grow past an afternoon's audit — the numbers are named in the report
  and changing them is a visible decision.
- **Constraint:** no verb is implemented in this task; it is the door and the list's
  shape. The broker does not decide *whether* — the turn did; it decides only *that
  this is one of its verbs, exactly*.

### 2. Printers, through the broker

**Status:** **Done, 2026-09-18.** The recovered carrier and producer pass the
real-CUPS runtime acceptance; publication requires the supervisor's nine gates
and every named acceptance check. This is code and WSL service evidence, not
physical-printer or desktop integration certification. **Depends on:** 1. Report:
`docs/autonomy/updates/printers-through-the-broker.md`.

**Built, then held back, 2026-09-16.** A worker built the broker's printer verbs, and
with them nine files of `alo-printing`, a new `alo-brokerd` and
`alo-changing-printers`, and the `alo-by-hand` and `alo-saying` registrations. The
acceptance below asks for the verbs to take `alo-printing`'s own types. But the lane
table gives `alo-printing` to the development PC's documents-and-paper plan, which
is running there, and two lanes in one crate is the collision the table exists to
prevent. So the supervising machine kept the change unpublished, on its local
branch `held/broker-task-2-edits-alo-printing`, with the handoff in
`.kernel-loop/refused/`. **For the owner:** whether this lane may make the
`alo-printing` change, or whether the documents lane exposes what the broker needs
first. **Resolved by the owner's 2026-09-18 authorization above:** the third PC
now integrates exactly that narrow contribution with this receiving task.
The held branch and original refused handoff remain preserved.

- **Acceptance:** the broker's printer verbs — add a printer `alo-printing` found, remove
  one, set the default — take `alo-printing`'s own types, and configure the rented
  print system without a free-form URI or driver name; the agent's *printers, solved*
  (★) reaches configuration only through these verbs, each proposed and approved; and a
  person does the same by hand in Settings through the same verbs (ADR 0009).
- **Constraint:** `alo-printing` decides what a printer is and what is wrong with it; the
  broker only carries out a decided change.
- **Inherits from 1** (its report has the reasoning): the first verb brings the broker's
  process — its binary, its unit, the machine's record it writes to, and how its
  approving key reaches the turn that issues tokens — because task 1 shipped no root
  service that could carry nothing out. `alo-printing`'s types reach the broker as an
  `alo_broker::Identity` of what the print service reported, never as its text.
- **Recovery integration, 2026-09-18:** task 3 already published the common
  process, record, key hand-over and transport, and task 4 added Storage. Keep
  those implementations and their later fixes. Printers join Network, Proxy
  and Storage through additive `Carriers::with_printers`; the published
  `Carriers::of(network, proxy, storage)` remains available. Updates still
  refuse pending ADR 0053. Software task 10 published the shared verb registry
  at `959ad33c15769618373ed50e0d06c5d9cf087d41`; add this task's declaration to
  `alo-declared/src/shipped.rs` and its dependency, never to copied lists in
  other consumers. Historical results do not validate this combined tree.

### 3. Network, through the broker

**Status:** **Done, 2026-09-16.** **Depends on:** 1. Report:
`docs/autonomy/updates/network-through-the-broker.md`; decision: ADR 0049.

- **Acceptance:** the broker's network verbs — join a network the machine can see, forget
  a network, turn the radio on or off, set the proxy `alo-proxy` holds — take closed
  types (a network by the identity the rented network manager reported, never a typed
  name); **a Wi-Fi password never passes through the agent** — joining a protected
  network asks the person for it in a surface the agent cannot read, held by a test
  that the verb's arguments have no password field; and a network change that would
  cut a turn's own connection says so before it is approved.
- **Constraint:** NetworkManager is rented. No VPN configuration verb in v0.5; its
  absence is recorded.
- **What tasks 2 and 4 inherit from 3** (its report has the reasoning): the broker's
  process now exists — `crates/alo-brokerd`, its unit beside it, its record, the
  key hand-over and the asking side in `alo-broker`, taken from the held task 2
  branch unchanged where they do not touch printers. A task carrying verbs out adds
  them to `alo_brokerd::Carriers`, whose match already names every verb it does not
  carry. Task 2's held branch therefore conflicts with this in `alo-brokerd` and
  `alo-broker` when it is published, and the conflict is the same files with the
  printers' carrier beside the network's.

### 4. Updates and storage, through the broker

**Status:** **Done, 2026-09-17** — storage built, and the updates decided as far
as a worker may and moved to task 8. **Depends on:** 1. Report:
`docs/autonomy/updates/storage-through-the-broker-and-the-update-decision.md`;
decision: ADR 0053, proposed.

**Storage, built.** `crates/alo-drives` (new) is what a drive is as udisks2
reports it, and the client over the system bus. A drive is named by the disk
service's `Id`, a filesystem by that and its UUID, and a device name is never an
identity. The client can call five methods and no other: read everything, mount,
unmount, power off, eject. A test reads the source for every udisks2 method that
formats, repartitions, erases, relabels, repairs, unlocks or tests a disk, and
finds none. `alo-brokerd`'s `Storage` carries `storage.mount` and `storage.eject`
out against what the service reports now: exactly one match, a drive a person
plugged in and never one that is part of the machine, a filesystem not already
mounted, mounted *as* the person by the login `/etc/passwd` gives the door's user,
and ejected by unmounting without force before switching the drive off. Mounting
makes no grant; neither crate depends on anything that could make one. Health is
not a broker verb: it is a read the disk service answers to anybody, in
`alo_drives::Drives::now`.

**Updates, not built, and why.** Measured in the pinned base: `bootc` changes the
machine only for root holding `CAP_SYS_ADMIN`, and the broker holds no capability,
by test. `alo_keeping_up::Ready`, which `Staging` needs, cannot cross into another
process. Every road forward either widens a privileged component, adds one, or
edits `alo-keeping-up`, which this plan never edits. So ADR 0053 sets out the
options and recommends a unit per update verb that the broker starts through
systemd. Both update verbs answer `not-carried` in the record until it is accepted,
and `crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs` fails once it
is. The update half of the acceptance below is task 8's, word for word.

- **Acceptance:** the broker's update verbs — apply a staged update, roll back — carry
  out exactly what `alo-keeping-up` decided and nothing it did not, with the
  machine-keeps-itself plan's rule that an update never interrupts intact; the storage
  verbs — mount a removable drive a person plugged in, eject it, check a disk's health —
  take the drive by its stable identity, **never format, repartition or erase anything**
  (those are v1 and deliberate, if ever), and a removable drive mounts for the signed-in
  person only, with no grant made to an agent by plugging it in.
- **Constraint:** no verb here writes to a partition table. *USB drives that appear when
  plugged in* is the desktop's; mounting is the broker's; granting its contents is
  `alo-picking`'s.

### 5. Full-disk encryption, decided before it is built

**Status:** **Done, 2026-09-17.** **Depends on:** nothing. Report:
`docs/autonomy/updates/full-disk-encryption-decided-before-it-is-built.md`;
decision: ADR 0054, proposed.

Nothing in this repository decided where the disk's key lives, and the answer decides
whether a stolen laptop is a stolen disk and whether a forgotten password is a lost
machine.

**Decided: the chip, with a PIN — and the recovery key is not optional.** ADR 0054
sets out the three options against a stolen powered-off laptop, an evil maid, a
forgotten password and a replaced motherboard, and recommends the key sealed to the
machine's security chip and released when a PIN is typed, falling back to a
passphrase on a machine whose chip cannot hold one. Measured in the pinned base and
on a virtual disk: `bootc install to-disk --block-setup tpm2-luks` calls
`systemd-cryptenroll --tpm2-device=auto` and nothing else — no PIN, no PCRs and **no
recovery key** — so the flag fails this acceptance by its own words and is not used;
`systemd-cryptenroll --recovery-key` prints 72 bytes on stdout, its English on
stderr, and the key it made opened the volume while one character changed did not;
and the signed-policy road for a boot chain that changes (PCR 11, `systemd-pcrlock`)
needs a unified kernel image the base does not build, which is why the binding is
PCR 7 and why an alo OS update cannot lock anybody out. Both measurements are in
`docs/quirks.md`.

`crates/alo-encrypting` (new, no dependencies at all) holds the shape:
`TheChip` and `WhatToAskFor` choose the road; `Pin` (six characters, because the
chip counts wrong answers) and `Passphrase` (twelve, because nothing counts them);
`RecoveryKey`, read from what the tool printed and **consumed** by
`RecoveryKey::written_back`; `WrittenDown`, which has no public maker anywhere;
`THE_ROAD`, whose order puts the typed-back key before anything the person will
unlock with; and `Enrolment`, whose one constructor takes a `WrittenDown`. What
holds the acceptance is that last chain, read off the crate's own public surface by
`tests/no_road_enrols_without_a_recovery_key_the_person_kept.rs` rather than
asserted about one road a test walked.

- **Acceptance:** a decision record, numbered after `git pull` when it is written, sets
  out the options — the key sealed to the TPM and released at boot with a PIN; a
  passphrase typed at boot; the TPM alone with no PIN; each **with a recovery key the
  person writes down** — against what each protects from (a stolen powered-off laptop, an
  evil maid, a forgotten password, a replaced motherboard), how each interacts with
  Secure Boot (ADR 0033 §4) and with updates that change what the TPM measures, how the
  installer plan enrols it during install without a person understanding LUKS, and what a
  managed machine's escrow will need at v1 (ADR 0004) without building it now; and it
  recommends one. `alo-encrypting` holds the decided shape as types, with a test that
  **no road enrols encryption without also producing the recovery key and requiring the
  person to confirm they kept it**.
- **Constraint:** proposed, and marked so; tasks 6 and 7 wait on it. No encryption is
  enrolled on any real disk by any test; everything is a virtual disk.
- **What task 6 inherits from 5** (its report has the reasoning):
  `alo_encrypting::THE_ROAD` is the sequence to turn into commands, in that order, and
  `crates/alo-encrypting/tests/the_enrolment_waits_on_its_decision.rs` **failed the day
  ADR 0054 stopped saying *proposed***, so a decision accepted and not built is a red
  suite rather than a forgotten line. It did, on 2026-09-19, and task 6 retargeted it at
  [ADR 0056](../decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
  — the same guard, on the question that stopped the building. The recovery key comes off the tool's **stdout**
  and its English off stderr, which is how the sentences stay task 7's.

### 6. Enrolled at install, and recovered — everything a virtual disk can show

**Status:** **Done, 2026-09-20.** The sequence, the types the installer plan is
handed, the sentences, and the virtual-disk acceptance — run against a real
LUKS2 volume in the pinned base under `podman`, not a simulation of one. This is
code and container evidence, **not** an install onto a machine with a chip:
`docs/features.md`'s v0.5 encryption line stays unticked and the three promises
that need a chip are task 9's. Report:
`docs/autonomy/updates/the-enrolment-sequence-against-a-virtual-disk.md`;
decision: [ADR 0056](../decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md),
**accepted, option C**, moved to *accepted* in the same change that built it. The
earlier report
`docs/autonomy/updates/a-sealed-disks-promise-needs-a-machine-with-a-chip.md`
recorded the finding that led to the decision. **Depends on:** 5.

[ADR 0054](../decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
is accepted by the owner, 2026-09-19: option C falling back to B, without
amendment. **The road is settled. What stopped this task is its acceptance, not
its decision.** The acceptance below asked for the sequence to be run *with a
software TPM in a virtual machine*, and the third PC measured on 2026-09-19 that
it could not present a TPM device to anything: the Linux the gates run in is
built with `CONFIG_TCG_VTPM_PROXY` unset, so there is no `/dev/vtpmx` for a
software chip to appear through and `modprobe tpm_vtpm_proxy` finds no module;
and the host shows neither `vmx` nor `svm`, so `qemu -accel kvm` refuses and
every guest is emulated — the one virtual-machine acceptance this repository
already owns took 3779 seconds.

> **Correction, 2026-09-20.** This paragraph said **no machine in this fleet can
> present a TPM device to anything**. That was measured on `AGAI01` and is true
> of `AGAI01`; it is **false of the development PC**, where a guest was given a
> TPM 2.0 through QEMU's `emulator` backend the same day and answered
> `/dev/tpmrm0`, `MSFT0101:00`, `tpm_tis`. The module this task found missing was
> never the obstacle: `tpm_vtpm_proxy` exposes a software chip to the **host**,
> not to a guest. **Option C stands, and the correction strengthens it rather
> than weakening it** — measurement 8 of ADR 0056 is why: the chip a guest gets
> reports its manufacturer as *IBM / SW* and carries the simulator's own lockout
> defaults, so an emulated chip is still not the chip ADR 0054's PIN argument
> rests on, and that is now evidenced from inside a guest rather than inferred
> from a missing module.

The other half of the same finding stands unchanged and is what this task is
built on: **everything on the road that does not need a chip runs here in
seconds**, on a real LUKS2 virtual disk in the pinned base (format, the recovery
key at 72 bytes on stdout, a second secret added, the installer's first key
wiped, then opened by the person's secret, opened by the recovery key, and
refused both the wiped key and a key that was not this disk's).

So the line falls where ADR 0056 puts it: a virtual disk shows the sequence and
every promise that is about LUKS, and **task 9 below** shows the three that are
about a chip, on the certified laptop when there is one (`docs/hardware.md`:
nothing is certified yet). Nothing about ADR 0054 is reopened. **Neither half
ticks `docs/features.md`'s v0.5 encryption line on its own** — ADR 0056 point 2,
accepted most deliberately of all: *enrolled at install* that has never been
installed onto a machine with a chip is not done, and a green suite here must not
be allowed to imply otherwise.

`crates/alo-encrypting` held the waiting rather than remembering it: its
`tests/the_enrolment_waits_on_its_decision.rs` read both decisions and failed the
day ADR 0056 stopped saying *proposed*, exactly as it failed the day ADR 0054
stopped saying so. It did, and this task replaced it with the tests of what it
built.

**What was built, 2026-09-20.** `crates/alo-encrypting` gained the sequence as
closed types beside the shape task 5 gave it: `TheVolume` (a partition of a disk
by the identity udev gave it — no free string becomes a device), `ASecretOnItsWay`
(the four secrets an enrolment moves, each in one file under `/run`, which is
memory and not the disk), `TheSequence` and `Run` (which rented tool with which
arguments, for enrolling, opening, closing and changing what the person unlocks
with), and `TheDiskRefused` (what the disk refused, read off what the tool
answered). **The two roads differ by exactly one run**, which is ADR 0056 point 5
held by a test rather than by a paragraph. The crate still depends on nothing,
opens no file and starts no program: it builds the runs and takes none of them.

**`crates/alo-enrolling` is new, and small**: one sentence for every refusal on
the road, with a translator's note, plus the five the road itself asks for. It
exists rather than living in `alo-encrypting` because that crate holds a recovery
key for the length of one screen and may not be able to serialise, log or send
anything — a vocabulary is a dependency that brings a serialiser with it, and the
guard that says so is a test somebody wrote deliberately. `alo-saying` collects
it. Each refusal maps to its sentence in an exhaustive match, so a variant added
to any refusal stops the crate compiling until somebody has written its sentence.

**The acceptance ran**, on a 64 MiB LUKS2 volume in the pinned base under
`podman`: the six steps in order; the recovery key at 72 bytes on stdout and its
English on stderr; a person typing it back making the `WrittenDown` that makes
the `Enrolment`; the person's secret and the recovery key each opening the
volume; the installer's wiped first key, a stranger's secret and a recovery key
one character wrong each refused with exit 2; the secret changed, the new one
opening it and the old one no longer doing so; and the recovery key nowhere in
the disk's 64 MiB. **What is not claimed:** nothing about a chip, and nothing
about a machine.

**What task 9 inherits from 6** (its report has the reasoning): the sequence is
one with a branch, so the chip road's five other runs are the runs this task
ran; what task 9 adds is the sixth —
`systemd-cryptenroll --tpm2-device=auto --tpm2-with-pin=yes --tpm2-pcrs=7` — and
the three facts around it. `alo_encrypting::TheSequence::enrolling_at_install`
already builds it for `WhatToAskFor::APin`, and
`changing_what_the_person_unlocks_with` already builds the one run that changes a
PIN; neither has been run anywhere, and that is exactly what task 9 is.

**How an accepted decision with a waiting guard is landed, in two commits.** This
is a general rule, not a fact about ADR 0056, and the next task to accept a
decision guarded this way should not have to rediscover it. The instinct is to
move the ADR's status line, the guard test and the plan together in one commit,
so that the guard never lies. That deadlocks: **the supervisor reads the plan as
`HEAD` has it**, so it will not take up a task whose status still says *blocked*,
and a single commit means no worker ever starts. The split that keeps the same
property is:

1. **A preparatory commit that touches the plan alone** — *blocked* becomes
   *ready*, recording the acceptance and what it decided. The ADR is untouched,
   so the guard is still telling the truth: the decision it reads still says
   *proposed*, and it still passes.
2. **The work commit** — the ADR's *proposed* becomes *accepted*, the guard is
   replaced by the tests of what was built, and the task is marked done, all
   together. The guard never reads *accepted* while still guarding.

- **Acceptance:** what the installer plan needs to enrol encryption during install is
  handed to it as `alo-encrypting`'s types and one tested command sequence against a
  virtual disk — enrol, unlock, change the PIN, and recover with the recovery key; the
  recovery key is shown once, in a form a person can write down and type back, and never
  stored on the disk it recovers; every refusal on the road is a sentence in the
  vocabulary with a translator's note; and `the_enrolment_waits_on_its_decision.rs` is
  replaced by the tests of what this built, in the same commit that moves ADR 0056's
  status line to *accepted*. **What is not claimed here is named here:** the three
  promises that need a chip are task 9's, and this task's report says so rather than
  leaving a reader to infer it from a suite that passes.
- **Constraint:** the installer's screens are the installer plan's; this plan hands it the
  decided sequence and the sentences. **No emulated chip stands in for a real one** — ADR
  0056 rejected that as option D, and a guest's *IBM / SW* chip is evidence about a
  simulator rather than about a machine. Nothing here ticks the v0.5 encryption line.

### 7. Every sentence, and the walk from a new printer to a recovered disk

**Status:** **Done, 2026-09-20.** The audit, the walk, the table it is held to,
and the two surfaces the walk found missing. This is code and vocabulary
evidence, **not** a printer, a network, a drive, a machine that has been
updated or a disk on certified hardware: every sentence was produced by the
value that really produces it, out of the machine's one assembled vocabulary,
and nothing here touched a rented service. Report:
`docs/autonomy/updates/every-sentence-and-the-walk-from-a-new-printer-to-a-recovered-disk.md`.
**Depends on:** 1, 2, 3, 4, and 8 for the update in the walk
— **task 8 landed on 2026-09-20**, so the update in the walk is takeable: what
a person meets is the broker's one-word answer worded by the surface that
asked, and the broker itself adds no sentence to the vocabulary.

**What the walk found, 2026-09-20.** *Worded by the surface that asked* was true
of two verb families and not of four. The printers had `alo-changing-printers`
and the network had `alo-changing-network`; **storage and updates had nobody**.
`alo_brokerd::Storage` opens and ejects a drive and the two update units apply
a build and go back, and each of them answered *carried*, *not kept* or
*refused* into a process with no person in front of it — so a drive could be
opened and finished with and a person would read nothing either way, and an
update could be refused at the door with nothing anywhere to tell them their
machine was unchanged. A vocabulary audit cannot find that, because there is no
sentence to audit; a walk finds it, which is why the walk is the acceptance and
not a nicety. So this task built the two missing surfaces as siblings of the two
that exist: `crates/alo-changing-drives` (thirteen sentences, the choosing that
goes before them, and the map from the door's word to what a person reads) and
`crates/alo-changing-updates` (**four** sentences — everything else a person
reads about an update was already written in `alo_keeping_up::words` and is
*said* here rather than written again, so that one fact reads as one line
however it reached them). Neither declares an agent verb, opens a socket or
assembles an instruction for the base.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — the agent proposes adding a printer, the person approves,
  the machine joins a network, a USB drive mounts and ejects, an update is applied through
  the broker — produces the exact sequence a person meets, recorded as a table and held by
  one test; the encryption sentences join the table once task 6 lands — **it landed on
  2026-09-20 and they are `alo-enrolling`'s `EVERY_WORD`**, twenty-four of them, already
  collected by `alo-saying` and already held to this rule by that crate's own
  `tests/what_this_crate_says.rs`; no sentence names
  LUKS, TPM, CUPS, NetworkManager, a socket or *root*.
- **Constraint:** nothing here re-decides what the sentences describe.
- **What the desktop lane inherits from 7** (the report has the reasoning): a
  person reads a drive's name as `alo_drives::Drive::identifier()`, which is the
  identifier the disk service keeps across plugging in and out — the drive's own
  name, stable, and not quite what a person would write. Giving a drive a shown
  name is a change to `alo-drives` and to what the disk service is asked for, and
  it belongs to whoever builds *USB drives that appear when plugged in*. It was
  recorded rather than taken, because widening this task into task 4's crate
  would have put two lanes in one crate for a hyphen.

### 8. Updates, through the broker, as ADR 0053 decides

**Status:** **Done, 2026-09-20.** The units, their programs, the broker's
carrier, the handed-over file and its contract, and the refusals beside the
carried case. This is code and unit-file evidence, **not** a machine that has
been updated: no real deployment was changed and no virtual machine was
staged into, which is the image lane's under ADR 0053's own consequences and
is said again in the report rather than left to be inferred from a green
suite. Report: `docs/autonomy/updates/updates-through-the-broker.md`; decision:
[ADR 0053](../decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md),
**accepted, option B**, moved to *accepted* in the same change that built it —
the two-commit shape task 6 sets out.

It was **accepted by the owner on 2026-09-19, option B** — *an update is carried out
by a unit the broker starts, never by the broker*. It is the only option that
leaves the broker holding **no capability**, which is ADR 0001 §2 and not
negotiable, and it puts the long networked part where systemd can bound it. All
three blockers are now cleared, the other two by the lanes that owed
them: `alo-egress` gained `Errand::FetchingAnUpdate`, and `alo-keeping-up` gained
`Staging::approved(from, to, deployments, source)`, which decides `Staging::of`'s
instruction element for element from an approved `{from, to}` and the base's
status read now, refuses with `NotRunningABuild`, `TheMachineMovedOn`,
`AlreadyWaiting` and the new `NotAnUpdate`, and can never carry `--apply`
(machine-keeps-itself plan task 9, whose report is
`updates/a-staging-decided-from-an-approval.md`). **Depends on:** 1, 4.

**The two-commit shape task 6 sets out was followed.** Until the work commit the
decision file still read *proposed, 2026-09-17*, and
`crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs` read that line
and passed; the work commit moved ADR 0053 to *accepted*, replaced that guard
with `tests/the_updates_are_carried_by_a_unit.rs` and
`tests/only_the_update_approved_is_carried_out.rs`, and marked this task done,
together. The guard never read *accepted* while still guarding.

Task 4 carried storage out and found that the updates could not be carried out
without a decision: the base's program asks for `CAP_SYS_ADMIN`, the broker holds
no capability, and an update `alo-keeping-up` decided cannot cross into the
broker. Its report and ADR 0053 have the measurements.

- **Acceptance** (task 4's, for the updates, unchanged): the broker's update verbs
  — apply a staged update, roll back — carry out exactly what `alo-keeping-up`
  decided and nothing it did not, with the machine-keeps-itself plan's rule that
  an update never interrupts intact. And, from ADR 0053 if accepted as proposed:
  no instruction the broker causes carries `--apply`; the broker still holds no
  capability; what runs the base is a unit with a fixed command line, its
  capabilities named line by line and held by a test; each refusal ADR 0053 lists
  is a test beside the carried case; and
  `crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs` is replaced by
  those tests.
- **Constraint:** `bootc` is rented and never patched (ADR 0011). What the image
  installs is the installer plan's; this task hands it the units and the measured
  capability set. No test changes a real machine's deployments; the virtual
  machine is where a staged update and a return are shown.
- **What was handed to the image lane, 2026-09-20** (the report has the
  reasoning): two units beside `crates/alo-brokerd/alo-brokerd.service` —
  `alo-applying-an-update.service` and `alo-going-back.service` — and their
  programs, `/usr/libexec/alo-applying-an-update` and
  `/usr/libexec/alo-going-back`, which are this crate's second and third
  binaries. The capability set in them is **derived, not measured on a booted
  machine**: what the base's own program states it requires (`CAP_SYS_ADMIN`,
  the only capability named in its binary — `docs/quirks.md`, 2026-09-20) plus
  what writing an ostree deployment touches. Narrowing it is the image lane's
  measurement, which ADR 0053's consequences already own, and the list is
  enumerated one per line so that narrowing is a visible edit. The virtual
  machine staging a real update and returning from it is the same lane's, and
  **nothing here claims it**.
### 9. The three promises only a chip can keep

**Status:** blocked — on a certified machine existing (`docs/hardware.md` lists
none). Split from task 6 on 2026-09-20 by
[ADR 0056](../decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md),
accepted option C. **Depends on:** 6.

Three facts about the sequence are about the chip rather than about LUKS, and no
virtual disk can show them: that **the chip releases the key when the PIN is
typed**, that **an update's new measurements do not stop it**, and that **a change
to what PCR 7 measures does stop it and the recovery key answers**. An emulated
chip cannot stand in — it reports *IBM / SW* and the simulator's own lockout
defaults (ADR 0056, measurement 8), so a run against one would be a program
agreeing with itself, which is the option ADR 0056 rejected by name.

- **Acceptance:** the three promises are held by an `#[ignore]`d test that names
  the certified machine it is run on, run by a person at that machine and pasted
  into the report with what they saw; and `docs/hardware.md` gains the line saying
  which machine showed them and when. Until that run exists, the repository says
  in public that the v0.5 encryption line is **not shown to work on any machine**
  — ADR 0056's own *Against*, accepted with the option.
- **Constraint:** the `#[ignore]` is never removed to make a suite look complete,
  and no emulated chip is ever recorded as having shown any of the three.

