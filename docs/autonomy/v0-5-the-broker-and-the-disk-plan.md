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
how it is enrolled, where the key is, and how a person recovers). **It reads and
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

### 6. Enrolled at install, and recovered

**Status:** **blocked** — on
[ADR 0056](../decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
being accepted by the owner. Report:
`docs/autonomy/updates/a-sealed-disks-promise-needs-a-machine-with-a-chip.md`.
**Depends on:** 5.

[ADR 0054](../decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
is accepted by the owner, 2026-09-19: option C falling back to B, without
amendment. **The road is settled. What stopped this task is its acceptance, not
its decision.** The acceptance below asks for the sequence to be run *with a
software TPM in a virtual machine*, and the third PC measured on 2026-09-19 that
**no machine in this fleet can present a TPM device to anything**: the Linux the
gates run in is built with `CONFIG_TCG_VTPM_PROXY` unset, so there is no
`/dev/vtpmx` for a software chip to appear through and `modprobe tpm_vtpm_proxy`
finds no module; and the host shows neither `vmx` nor `svm`, so `qemu -accel kvm`
refuses and every guest is emulated — the one virtual-machine acceptance this
repository already owns took 3779 seconds. Measured in the same hour, and the
other half of the same finding: **everything on the road that does not need a
chip runs here in seconds**, on a real LUKS2 virtual disk in the pinned base
(format, the recovery key at 72 bytes on stdout, a second secret added, the
installer's first key wiped, then opened by the person's secret, opened by the
recovery key, and refused both the wiped key and a key that was not this disk's).

So ADR 0056 asks the owner where the line falls: a virtual disk shows the
sequence and every promise that is about LUKS; a machine with a chip shows the
three that are about a chip — that the chip releases the key when the PIN is
typed, that an update's new measurements do not stop it, and that a change to
what PCR 7 measures does and the recovery key answers. If it is accepted this
task becomes the first of those two and a new task becomes the second, whose
acceptance is run on the certified laptop when it exists (`docs/hardware.md`:
nothing is certified yet). Nothing about ADR 0054 is reopened, and
`docs/features.md`'s v0.5 encryption line stays unticked either way.

`crates/alo-encrypting` holds the waiting rather than remembering it: its
`tests/the_enrolment_waits_on_its_decision.rs` now reads both decisions and fails
the day ADR 0056 stops saying *proposed*, exactly as it failed the day ADR 0054
stopped saying so.

- **Acceptance:** what the installer plan needs to enrol encryption during install is
  handed to it as `alo-encrypting`'s types and one tested command sequence against a
  virtual disk — enrol, reboot, unlock, change the PIN, recover with the recovery key
  after the TPM's measurements change; the recovery key is shown once, in a form a person
  can write down and type back, and never stored on the disk it recovers; and **an update
  that changes boot measurements does not lock a person out**, held by a test that updates
  the virtual machine's boot chain and unlocks afterwards.
- **Constraint:** the installer's screens are the installer plan's; this plan hands it the
  decided sequence and the sentences.

### 7. Every sentence, and the walk from a new printer to a recovered disk

**Status:** ready. **Depends on:** 1, 2, 3, 4, and 8 for the update in the walk.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — the agent proposes adding a printer, the person approves,
  the machine joins a network, a USB drive mounts and ejects, an update is applied through
  the broker — produces the exact sequence a person meets, recorded as a table and held by
  one test; the encryption sentences join the table once task 6 lands; no sentence names
  LUKS, TPM, CUPS, NetworkManager, a socket or *root*.
- **Constraint:** nothing here re-decides what the sentences describe.

### 8. Updates, through the broker, as ADR 0053 decides

**Status:** blocked — on ADR 0053 being accepted by the owner; on
`alo-keeping-up`'s owner adding a way to decide a `Staging` from an approved
`{from, to}` checked against the base's status now (the machine-keeps-itself
lane's crate, which this plan never edits); and on `alo-egress`'s owner adding an
errand for fetching an update. **Depends on:** 1, 4.

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
