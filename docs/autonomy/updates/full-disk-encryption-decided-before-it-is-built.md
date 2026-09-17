# Full-disk encryption, decided before it is built

**Date:** 2026-09-17
**Workstream:** `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, task 5
(*Full-disk encryption, decided before it is built*)
**Contributor:** Claude Code, in `C:\dev\alo-os-2`
**Status:** ready for integration. The decision it produces is **proposed** and
waits on the owner; tasks 6 and 7 of the plan wait on that.

## What this task was

Nothing in this repository decided where the disk's key lives. `docs/features.md`
promises *[v0.5] Full-disk encryption, enrolled at install* and *[v1]
Disk-encryption key escrow*, and between those two lines there was no answer to
the question a customer asks first: **if this laptop is stolen, is my disk
stolen with it — and if I forget the password, is the machine gone?**

The task's acceptance is a decision record and the shape it leaves behind in
code. No encryption is enrolled on any real disk by anything here.

## The decision: ADR 0054, proposed

`docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md`.

**The disk is sealed to this machine's own security chip and opened with a PIN;
on a machine whose chip cannot be used it is opened with a passphrase; and
either way the person writes down a recovery key and types it back before the
install finishes.**

It sets the three options the plan named — the chip alone, a passphrase at boot,
the chip with a PIN, each with a recovery key — against the four things a person
actually loses a machine to (a laptop stolen while it was off, an evil maid, a
forgotten password, a replaced motherboard), against Secure Boot as ADR 0033 §4
leaves it, and against updates that change what the chip measures. The table in
the ADR is the argument; the short version is that the chip alone fails the first
column outright on a machine with Secure Boot off, and that a PIN plus the chip's
own lockout is worth more than a passphrase a person will turn off.

### Measured rather than remembered

Everything the ADR claims about the rented tools was measured on 2026-09-17 in
the pinned base image (`quay.io/fedora/fedora-bootc`, local image
`b035260f985f`): `bootc 1.15.1`, `systemd 257.13-1.fc42`, `cryptsetup 2.8.4`,
`clevis 21-10.fc42`, `tpm2-tools 5.7-3`.

1. **`bootc install to-disk --block-setup tpm2-luks` is not the encryption we
   could ship.** Read for everything it names around LUKS, the binary holds
   exactly `luksFormat`, `systemd-cryptenroll` and `--tpm2-device=auto` — no
   `--tpm2-with-pin`, no `--tpm2-pcrs`, **no `--recovery-key`**. So the flag
   fails the task's own acceptance (*each with a recovery key the person writes
   down*) before any threat-model argument begins. Its own help sends anything
   with LUKS in it to `install to-filesystem`, which takes `--root-mount-spec`.
2. **A recovery key was made on a virtual disk and it is a thing a person can
   write down.** `systemd-cryptenroll --recovery-key` on a 64 MiB LUKS2 file
   exited 0 and wrote **72 bytes to stdout**: eight groups of eight characters
   from the alphabet `cbdefghijklnrtuv`, separated by `-`, and a newline. It
   enrolled a keyslot of its own with a `systemd-recovery` token in the header.
   The key opened the volume; one character changed was refused; the original
   passphrase still opened it.
3. **The tool's English goes to stderr and the key does not.** So alo OS reads
   the 72 bytes and says its own sentence in the person's language; the rented
   tool's wording never reaches a screen.
4. **The signed-policy road is not open to us.** `--tpm2-public-key` over PCR 11
   and `systemd-pcrlock` both need a unified kernel image; the base ships no
   `systemd-ukify`, no `systemd-boot` and no `.efi.stub` anywhere, and alo OS
   boots through shim and GRUB. That measurement is what decides the binding: it
   is **PCR 7 and nothing else**, which is also why an alo OS update — a new
   kernel and initramfs, measured into PCRs 4, 5, 8 and 9 — cannot lock anybody
   out.
5. **What a TPM actually holds was not measured**, because there is no TPM in a
   container and this task enrols nothing. PCR values, the chip's lockout
   behaviour and what `--tpm2-with-pin` costs at boot are task 6's, in a virtual
   machine with a software TPM. The ADR says so in its own measurement 7 rather
   than implying it was checked.

Measurements 1 and 2–3 are written into `docs/quirks.md` under *Pinned engines*.

## What was built: `crates/alo-encrypting`

A new crate with **no dependencies at all**, which is the argument rather than an
accident: a crate that holds a recovery key for the length of one screen must not
be able to serialise it, log it, write it or send it, and with nothing in reach it
cannot.

| | |
|---|---|
| `TheChip`, `WhatToAskFor` | Which of the two roads this machine takes. A chip that is ready means a PIN; absent, not ready, or unread means a passphrase — *a chip we cannot see is a chip we will not make the only place the key lives* |
| `Pin` (at least 6 characters), `Passphrase` (at least 12) | The one secret the person is asked for. The two numbers are two different arguments: the chip counts wrong PINs, and nothing counts wrong passphrases |
| `HowItUnlocks` | A struct, not an enum, so *sealed to a chip* cannot be constructed on a machine with no chip to seal to |
| `RecoveryKey` | Read from what the tool printed; **consumed** by `written_back`, so no line after the confirmation can be holding one |
| `WrittenDown` | The proof, with no public maker anywhere in the crate |
| `THE_ROAD`, `Step` | The six steps in the order ADR 0054 decided |
| `Enrolment` | One constructor, and it takes the proof |

### The acceptance, and how it is held

> `alo-encrypting` holds the decided shape as types, with a test that **no road
> enrols encryption without also producing the recovery key and requiring the
> person to confirm they kept it**.

`tests/no_road_enrols_without_a_recovery_key_the_person_kept.rs` holds it twice,
because either half alone would be a green test proving the wrong thing:

- **by walking the road** — the one that works, on a machine with a chip and on
  one without, and every refusal on it (the tool printed something that is not a
  key; the person typed nothing; the person typed one character wrong; the PIN
  was too short; the chip cannot hold a key), each stopping before an
  `Enrolment` exists;
- **by reading the crate's own public surface** — every `pub fn` in `src/`, so
  that *there is no other road* is a measurement rather than a claim about the
  road a test happened to walk. `Enrolment` has one maker and it takes a
  `WrittenDown`; `WrittenDown` has no public maker; the one crate-private maker
  is called from exactly one place, the matching arm of `written_back`;
  `written_back` takes the key **by value**; and the whole list of public
  functions that can produce one of the three is asserted verbatim, so one added
  anywhere fails here.

`tests/the_key_is_never_kept_on_the_disk_it_recovers.rs` holds *shown once, never
stored*: the manifest has no dependencies and no dependency table at all; no
source file names `std::fs`, a `File`, a `Command`, a socket, `std::env`,
`serde`, a `println!` or `unsafe`; every secret has a hand-written silent `Debug`
and no `Display`; and no secret appears in any line the crate can print,
`Enrolment`'s included.

`tests/the_enrolment_waits_on_its_decision.rs` is the pattern
`crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs` set in task 4:
it asserts ADR 0054 still says *proposed*, and **fails the day it says
accepted**, which is task 6's instruction to build rather than a line somebody
has to remember. It also reads the source to show that no rented tool, device or
argument list is named in code anywhere in the crate.

## Decisions a worker made, and why

Nobody was waiting to answer these, so they were decided and are written here.

- **PCR 7 alone.** Not a compromise — measurement 4 closes the alternative, and
  binding to more is what makes updates break unlock. The honest consequence is
  stated in the ADR rather than buried: with Secure Boot off, PCR 7 is what ties
  the key to this chip in this machine, and **the PIN is what protects the
  machine**. The evil maid is not defended against in v0.5, on either road, and
  the ADR says so in as many words.
- **Six characters for a PIN, twelve for a passphrase.** The chip's
  anti-hammering is what an attacker meets on the first road and nothing is what
  they meet on the second. Counted in characters, not bytes, because the machine
  promises 24 languages.
- **The recovery key is made and typed back *before* the way the person unlocks
  is enrolled.** The other order makes *no recovery key* something that can
  happen to a working machine. This one makes an install abandoned at that
  moment abandoned cleanly.
- **Confirming means typing it back, not ticking a box.** Dashes, spacing and
  case are forgiven; the characters are not. A person copying 64 characters onto
  paper and back gets the dashes wrong, and refusing that teaches them to
  photograph the screen — which is the one outcome the whole file exists to
  prevent.
- **The machine chooses the road; the person is not asked.** The question *do you
  want the chip to hold your key or your head* cannot be asked at install without
  explaining a threat model. Recorded in the ADR as a v1 settings question.
- **Full-disk encryption means the root filesystem**, as it does on BitLocker,
  FileVault and every LUKS installation. Written down in the ADR openly, with
  what it leaves unencrypted and what that costs, rather than assumed — it is
  not a narrowing of the `docs/features.md` line, it is what that line has always
  meant, and the unencrypted half is named because it is where the evil maid
  attacks.
- **No words in this crate.** It declares none and says nothing to anybody, the
  way `alo-drives` does; its English is for a service log. The sentences a person
  meets are task 6's to write and task 7's to put in the vocabulary with a
  translator's note, which is what the plan says.
- **`TheChip` is written again rather than borrowed from
  `alo_installer::SecurityChip`.** That crate is the Windows program a person
  runs on the machine they are replacing; a type in it is not a type about the
  machine alo OS is on. The four answers agree deliberately so whoever wires them
  together has nothing to translate.

## What was not done, and why

- **No command sequence.** Task 6's, and its acceptance names it: enrol, reboot,
  unlock, change the PIN, recover after the measurements change, all against a
  virtual disk. `THE_ROAD` is the order it turns into.
- **Nothing in `crates/alo-installing` or `crates/alo-installer`.** The plan
  reads those and never edits them; what the installer plan needs is handed to it
  as types and, at task 6, as the sequence.
- **No escrow.** ADR 0004's v1 line. The ADR records what the shape owes it — the
  key exists as a value at one known moment, in one process — and deliberately
  builds nothing.
- **No verb, broker-side or otherwise.** Enrolment happens at install; nothing
  here is reachable by an agent.

## Verification

Run from `C:\dev\alo-os-2` on Windows Server 2022 (`x86_64-pc-windows-gnu`), and
the same on Linux through WSL2 Ubuntu with
`CARGO_TARGET_DIR=$HOME/alo-builds/this-machine`, which is this lane's own build
directory.

| Command | Where | Result |
|---|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` | Windows | clean |
| `cargo clippy -p alo-encrypting --all-targets -- -D warnings` | Windows | clean |
| `cargo clippy -p alo-encrypting --all-targets -- -D warnings` | Linux | clean |
| `cargo doc -p alo-encrypting --no-deps` | Windows | clean |
| `cargo test -p alo-encrypting` | Windows | 30 unit + 13 integration, all pass |
| `cargo test -p alo-encrypting` | Linux | same, all pass |
| `cargo test -p alo-citing` | Windows | 31 pass — the new ADR, its links and every citation of it land |
| `cargo test -p alo-collected` | Linux | 19 pass (8 + 11), `every_crate_that_declares_words_is_collected_or_named_apart` included: a new workspace member with no `src/words.rs` is not owed a collection |
| `cargo test -p alo-by-hand` | Linux | 40 pass (27 + 13) — the new member does not disturb the verb-by-hand check |

The measurements behind the ADR were taken by running `bootc`, `cryptsetup` and
`systemd-cryptenroll` inside the pinned base image under podman in WSL2, against
a 64 MiB file — a virtual disk, never a real one.

**Not run, deliberately:** the full workspace suite, which the supervisor runs
after this task.

**Could not be run on Windows:** `cargo test -p alo-collected` and
`cargo test -p alo-by-hand` fail to *build* on this host because `ring 0.17.14`
needs a C compiler the `x86_64-pc-windows-gnu` toolchain here does not have. That
was confirmed to be true of the clean tree as well as this one (`git stash -u`,
same failure), so it predates this change; both suites were therefore run on
Linux, where they pass.

**Physical acceptance owed, and not claimed:** nothing here has touched a real
disk, a real TPM or a real machine. Task 6 owes the virtual-machine measurements
with a software TPM; no line of `ROADMAP.md`'s *on the machine* column is earned
by this task.

## Limitations

- The decision is **proposed**. Until the owner accepts it, tasks 6 and 7 are
  blocked, and `the_enrolment_waits_on_its_decision.rs` is what makes that
  visible.
- Option C's evil-maid protection arrives with shim and Secure Boot, which is a
  v1 line. v0.5 ships the honest version and says so.
- Turning Secure Boot on after an install changes PCR 7 and costs the person
  their recovery key once. That is what PCR 7 is for, and it is why the key is
  not optional.

## Proposed updates to the shared documents

For the integration owner; not edited here (`docs/autonomy/SHARED_MAIN.md`).

**`CHANGELOG.md`** — under Unreleased:

> **Full-disk encryption, decided.** Where the key to this machine's disk lives
> is now written down: sealed to the machine's own security chip and released
> when a PIN is typed, or — on a machine whose chip cannot hold a key — opened
> with a passphrase. Either way the person is shown a recovery key once, writes
> it down, and types it back before the install finishes; a machine cannot be
> described as encrypted, in code, without that having happened. The decision is
> ADR 0054 and waits on the owner. Nothing is enrolled on a real disk yet.

**`ROADMAP.md`** — v0.5 *Full-disk encryption* stays unticked. The decision and
the shape are done; enrolment at install, the recovery road and the
on-a-machine evidence are tasks 6 and 7.

**`docs/autonomy/QUEUE.md`** — task 5 of the broker-and-the-disk plan: done,
2026-09-17. Task 6 remains blocked on ADR 0054 being accepted, which is now a
question for the owner rather than for a worker.

**`docs/autonomy/STATE.md`** — reference this report and
`docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md`.
The owner has two proposed decisions from this lane waiting: ADR 0053 (updates
through the broker) and ADR 0054 (the disk's key).

## Files

- `docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md` (new)
- `crates/alo-encrypting/Cargo.toml` (new)
- `crates/alo-encrypting/src/lib.rs`, `chip.rs`, `unlocking.rs`, `pin.rs`,
  `passphrase.rs`, `recovery_key.rs`, `written_down.rs`, `road.rs`,
  `enrolment.rs` (new)
- `crates/alo-encrypting/tests/no_road_enrols_without_a_recovery_key_the_person_kept.rs`,
  `the_key_is_never_kept_on_the_disk_it_recovers.rs`,
  `the_enrolment_waits_on_its_decision.rs` (new)
- `Cargo.toml`, `Cargo.lock` — the new member
- `docs/quirks.md` — two entries under *Pinned engines*
- `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` — task 5 marked done, and
  what task 6 inherits
- this report
