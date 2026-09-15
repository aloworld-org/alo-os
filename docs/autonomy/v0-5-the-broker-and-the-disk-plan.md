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
never edits** `alo-capability`, `alo-turn` and `alo-protocol` (a system verb is a
verb, proposed and approved in a turn — if the verb list or the door must change,
that is a finding and a decision), `alo-printing` (the documents plan's — what a
printer is), `alo-keeping-up` (the machine-keeps-itself plan's — what an update is),
`alo-software` (the software plan's — an application install is not a system verb),
`alo-boundaryd` and `alo-sessiond` (the pattern a privileged component follows),
`alo-record`, `alo-egress`, `image/`, `alo-image` and `alo-installer` (the installer
plan's — enrolment happens during install, and what this plan decides is handed to
it), and `alo-saying`. **Nothing in `crates/alo-shell`.**

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

**Status:** ready. **Depends on:** nothing.

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

**Status:** ready. **Depends on:** 1.

- **Acceptance:** the broker's printer verbs — add a printer `alo-printing` found, remove
  one, set the default — take `alo-printing`'s own types, and configure the rented
  print system without a free-form URI or driver name; the agent's *printers, solved*
  (★) reaches configuration only through these verbs, each proposed and approved; and a
  person does the same by hand in Settings through the same verbs (ADR 0009).
- **Constraint:** `alo-printing` decides what a printer is and what is wrong with it; the
  broker only carries out a decided change.

### 3. Network, through the broker

**Status:** blocked — on `v0-5-software-and-the-web-plan.md` task 4, whose
`alo-proxy` holds the proxy this task's verbs set. **Depends on:** 1.

- **Acceptance:** the broker's network verbs — join a network the machine can see, forget
  a network, turn the radio on or off, set the proxy `alo-proxy` holds — take closed
  types (a network by the identity the rented network manager reported, never a typed
  name); **a Wi-Fi password never passes through the agent** — joining a protected
  network asks the person for it in a surface the agent cannot read, held by a test
  that the verb's arguments have no password field; and a network change that would
  cut a turn's own connection says so before it is approved.
- **Constraint:** NetworkManager is rented. No VPN configuration verb in v0.5; its
  absence is recorded.

### 4. Updates and storage, through the broker

**Status:** ready. **Depends on:** 1.

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

**Status:** ready. **Depends on:** nothing.

Nothing in this repository decides where the disk's key lives, and the answer decides
whether a stolen laptop is a stolen disk and whether a forgotten password is a lost
machine.

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

### 6. Enrolled at install, and recovered

**Status:** blocked — on task 5's decision being accepted. **Depends on:** 5.

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

**Status:** ready. **Depends on:** 1, 2, 3, 4.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — the agent proposes adding a printer, the person approves,
  the machine joins a network, a USB drive mounts and ejects, an update is applied through
  the broker — produces the exact sequence a person meets, recorded as a table and held by
  one test; the encryption sentences join the table once task 6 lands; no sentence names
  LUKS, TPM, CUPS, NetworkManager, a socket or *root*.
- **Constraint:** nothing here re-decides what the sentences describe.
