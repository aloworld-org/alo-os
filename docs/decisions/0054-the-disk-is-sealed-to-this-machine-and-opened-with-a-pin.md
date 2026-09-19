# ADR 0054 — The disk is sealed to this machine and opened with a PIN, and the person writes down the key that recovers it

**Status:** **accepted, 2026-09-19, by the owner — option C falling back to B,
as recommended and without amendment.** Written by task 5 of
`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` (*Full-disk encryption,
decided before it is built*), whose whole acceptance is this decision and the
shape it leaves behind in `crates/alo-encrypting`. Tasks 6 and 7 of that plan
waited on it and are now the work to do;
`crates/alo-encrypting/tests/the_enrolment_waits_on_its_decision.rs` begins
failing with this line, which is how a decision gets built rather than
remembered.

**What the owner was told before accepting**, so a later reader knows what it
rested on: that the machine chooses PIN or passphrase rather than asking a
person who should never meet the words TPM or LUKS; that the recovery key is
made *before* the unlock method is enrolled, so an install abandoned at that
moment is abandoned cleanly rather than leaving a disk nobody can open; that
confirming means typing the key back, with dashes, case and spacing forgiven and
the characters not, because a stricter rule teaches people to photograph the
screen; and that the key is consumed by the type holding it rather than trusted
to a comment.

**Left open deliberately:** whether an *install without encryption* road should
exist at all. This decision does not create one, and §6 leaves that to the
installer plan with the owner. The default is right; it should be settled on
purpose later rather than discovered by somebody meeting a machine alo OS
refuses to install onto.
**Date:** 2026-09-17
**Proposed by:** the broker-and-the-disk workstream
**Context:** `docs/features.md` v0.5 *Full-disk encryption, enrolled at install*
and v1 *Disk-encryption key escrow*;
[ADR 0004](0004-the-organisations-machine.md) (a managed machine's organisation
holds a recovery key, and the person is told so);
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (LUKS,
`systemd-cryptenroll` and the TPM stack are rented, configured and never
patched);
[ADR 0023](0023-installed-from-the-machine-it-replaces.md) (the install story);
[ADR 0033](0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
(§4: the shipped installer refuses to proceed with Secure Boot **enabled** until
shim lands, and the certified laptop is certified with Secure Boot **off**, on
the record);
[ADR 0036](0036-the-image-is-signed-by-a-key-a-person-holds.md) (a release is
verified before anything is written); `docs/booting.md`;
`crates/alo-installer`, `crates/alo-installing`, `crates/alo-encrypting`.

## The question in one line

**Where does the key to this machine's disk live — in the machine's security
chip, in the person's head, or both — and what does a person hold in their hand
on the day it will not open?**

Nothing in this repository has answered it. The answer decides whether a stolen
laptop is a stolen disk, and whether a forgotten password is a lost machine.

## What is true today, measured rather than remembered

Measured on 2026-09-17 in the pinned base image (`quay.io/fedora/fedora-bootc`,
local image `b035260f985f`), which is what `image/Containerfile` builds on and
what the installer's boot environment is made from: `bootc 1.15.1`, `systemd
257.13-1.fc42`, `cryptsetup 2.8.4`, `clevis 21-10.fc42`, `tpm2-tools 5.7-3`.

1. **`bootc` will make an encrypted disk, and the encryption it makes is not one
   we could ship.** `bootc install to-disk --block-setup` takes `direct` or
   `tpm2-luks`, the second documented as *Bind unlock of filesystem to presence
   of the default tpm2 device*. Read for everything it names around LUKS, the
   binary holds exactly three things: `luksFormat`, `systemd-cryptenroll` and
   `--tpm2-device=auto`. **No `--tpm2-with-pin`, no `--tpm2-pcrs`, no
   `--recovery-key`.** So the flag on its own is the chip alone, with no PIN, and
   with no key a person could ever recover the disk with — which fails this
   task's acceptance by its own words, *each with a recovery key the person
   writes down*, before any argument about threat models begins.
2. **`install to-disk`'s own help sends anything more complex elsewhere**: *Use
   `install to-filesystem` for anything more complex such as RAID, LVM, LUKS
   etc.* `install to-filesystem` takes `--root-mount-spec`, which is how a root
   filesystem that is already open on a mapped device is handed to it. That is
   the shape of the road, and `crates/alo-installing`'s single `install to-disk`
   call is what changes.
3. **The rented tool has both of the things `bootc`'s flag does not.**
   `systemd-cryptenroll` offers `--recovery-key`, `--tpm2-with-pin=BOOL`,
   `--tpm2-pcrs=`, `--tpm2-public-key`/`--tpm2-public-key-pcrs`, and
   `--tpm2-pcrlock=`. `clevis` is also in the base; its TPM binding has neither a
   PIN nor a recovery key of its own, so either would have to be bolted on beside
   it, and `bootc` calls `systemd-cryptenroll` already. There is no reason to run
   two enrolment tools on one header.
4. **A recovery key was made on a virtual disk, and it is a thing a person can
   write down.** On a 64 MiB file formatted LUKS2 in that base,
   `systemd-cryptenroll --recovery-key` exits 0 and writes **72 bytes to
   stdout**: eight groups of eight characters separated by `-`, every character
   from the sixteen-letter alphabet `cbdefghijklnrtuv`, and a newline. It enrols
   it in a keyslot of its own with a `systemd-recovery` token in the header. The
   key opened the volume; one character changed was refused; the original
   passphrase still opened it. That alphabet is not decoration — it has no `0`
   against `O` and no `1` against `l`, and it types the same on QWERTY, AZERTY
   and QWERTZ, which for a product promising 24 languages is the difference
   between a key somebody can use and a key somebody photographs.
5. **The tool's English never has to reach a person.** In the same measurement
   the explanation went to **stderr** and the key went to **stdout**, and stderr
   held no part of the key. So alo OS reads 72 bytes and says its own sentence in
   the person's own language, and `systemd-cryptenroll`'s wording is a service
   log's problem and nobody else's.
6. **The road systemd offers for a boot chain that changes is not open to us.**
   `--tpm2-public-key`/`--tpm2-public-key-pcrs` (a signed policy over PCR 11) and
   `systemd-pcrlock` both need PCR 11, which is measured by `systemd-stub` — that
   is, by a unified kernel image. The base ships **no** `systemd-ukify`, no
   `systemd-boot`, and no `.efi.stub` anywhere on the filesystem; alo OS boots
   through shim and GRUB with boot-loader-specification entries
   (`docs/booting.md`, and the GRUB entry in `docs/quirks.md`, 2026-09-16).
   `/usr/lib/systemd/systemd-pcrlock` exists and has nothing to lock against.
   **This is the measurement that decides which PCRs we may bind to**, and it is
   the one most likely to change: a UKI is a v1 conversation, and this decision is
   written so that it can be revisited without re-enrolling anybody.
7. **What a TPM actually holds was not measured here.** There is no TPM in a
   container, and this task enrols encryption on no real disk. PCR values, the
   chip's lockout behaviour and what `--tpm2-with-pin` costs at boot are task 6's
   to measure in a virtual machine with a software TPM, and they go into
   `docs/quirks.md` whichever way they answer.

## What is being encrypted, said plainly

The **root filesystem** — everything a person has, every setting, every record
and the model's weights — on the disk alo OS was installed to. The EFI system
partition and `/boot` carry the loader, the kernel and the initramfs, and no data
of the person's. That is what full-disk encryption means on BitLocker, on
FileVault and on every LUKS installation in the world, and it is what
`docs/features.md`'s v0.5 line means here. It is written down rather than
assumed, because the half it leaves unencrypted is exactly the half an evil maid
attacks, and the next section says so instead of hiding it.

## What each option protects from

Four things a person actually loses a machine to.

| | a laptop stolen while it was off | an evil maid — someone with the machine for an hour, who gives it back | a forgotten password | a replaced motherboard, or a chip that cleared |
|---|---|---|---|---|
| **A. The chip alone, no PIN** | **Not protected.** The chip releases the key to whatever reproduces the measurements it was sealed against. With Secure Boot off (ADR 0033 §4) that is *any* operating system the thief boots; with Secure Boot on and PCR 7, it is any signed loader chain-loading the thief's own initramfs. The thief does not need to open the disk — they turn the machine on. | Not protected, and worse: nothing is typed, so there is nothing to phish and nothing to notice. | Nothing to forget. | Key gone. Recovery key or nothing. |
| **B. A passphrase typed at every start** | **Protected.** The disk is bytes; the key is in one person's head. | Not protected: the boot chain is unencrypted and unsigned, so it can be replaced with one that keeps the passphrase. Secure Boot is the answer and it is off. | Recovery key. | Unaffected — the key was never in the machine. |
| **C. The chip, released when a PIN is typed** | **Protected twice.** The disk alone is useless: it is sealed to a chip that did not leave the machine. The machine alone is useless: the PIN was not stolen with it, and the chip's own lockout counts wrong PINs, so a six-character PIN is worth what a far longer passphrase is worth against an offline attack on a header. | Not protected while Secure Boot is off, for the same reason as B. **Protected when shim lands**, because PCR 7 then stops matching for a chain the person's firmware did not accept — and that is the one of these three that gets better on its own. | Recovery key. | Key gone. Recovery key. |

Every row of that table assumes the recovery key, because the acceptance does and
because two of the four columns have no other answer. The key is not an option;
it is the fourth column.

## Secure Boot, honestly (ADR 0033 §4)

Until the shim review lands, the shipped installer refuses to proceed with Secure
Boot enabled, and the certified laptop was certified with it off. So on every
machine this decision can reach today:

- Binding to PCR 7 records *what the firmware's Secure Boot policy was*, which
  with Secure Boot off is a value any other operating system on that machine
  reproduces. **It is therefore not what protects the machine. The PIN is.**
  PCR 7 is what ties the key to *this chip in this machine*, and that is worth
  having on its own — it is what makes a disk pulled out of a laptop unreadable.
- **The evil maid is not defended against in v0.5**, and this decision says so
  rather than implying otherwise. The kernel and the initramfs are on an
  unencrypted, unverified `/boot`; somebody with the machine for an hour can
  replace them and collect the next PIN or passphrase. That is true of option B
  as well; it is true of every unencrypted boot chain. What fixes it is Secure
  Boot with our key, which is a v1 line in `docs/features.md` and is already
  named there.
- **Turning Secure Boot on later changes PCR 7 and the chip will stop releasing
  the key.** That is not a bug and it is exactly what PCR 7 is for. The person
  uses the recovery key once and the machine re-enrols. It is the reason the
  recovery key has to exist on a machine that has never had a problem.

## Updates that change what the chip measures

This is what makes people hate encrypted disks, so it is decided rather than
discovered:

- **An alo OS update does not change PCR 7.** A new deployment is a new kernel, a
  new initramfs and new boot entries; those are measured into PCRs 4, 5, 8 and 9,
  and we bind to none of them. An update therefore does not lock anybody out, and
  task 6's acceptance — *an update that changes boot measurements does not lock a
  person out*, held by a test that updates a virtual machine's boot chain and
  unlocks afterwards — is a test of that claim rather than of a workaround.
- **A firmware update that rotates the signature databases does change PCR 7**,
  as does enabling or disabling Secure Boot. Rare, deliberate, and the recovery
  key is what it costs. Re-enrolment afterwards is a v1 conversation about what
  the machine offers to do for the person; v0.5 owes them a key that works and a
  sentence that tells them what happened.
- **Binding to more than PCR 7 is not available to us** (measurement 6), and
  would be the thing that made updates break unlock if it were. So the narrow
  binding is not a compromise here; it is both the safer and the only road.

## The options

### A. The chip alone, with no PIN, and a recovery key

`bootc install to-disk --block-setup tpm2-luks`, plus a
`systemd-cryptenroll --recovery-key` beside it.

- **For:** one flag and one command. The person types nothing, ever. It is what
  every "transparent encryption" ships.
- **Against:** the first column of the table. On a machine with Secure Boot off —
  which is every machine alo OS can be installed on today — a stolen laptop opens
  itself when the thief presses the power button, and *full-disk encryption* on a
  feature list would then be a sentence that protects a person from having their
  disk read in a different computer and from nothing else. It also hands the whole
  guarantee to a firmware setting the person cannot see. **Rejected.**

### B. A passphrase typed at every start, and a recovery key

LUKS2 with the person's passphrase in a keyslot. No chip involved.

- **For:** it works on every machine, chip or no chip. Nothing depends on
  firmware, measurements, updates or a chip that might clear. It is the simplest
  thing that is honest.
- **Against:** it is the option people turn off. A passphrase strong enough to
  survive an offline attack on a LUKS2 header is long, and it is typed at every
  start, before any keyboard layout the person chose is in effect. Nothing rate
  limits an attacker who has the header: what stands between them and the disk is
  argon2 and the passphrase's own length. **Kept, but as the road for a machine
  with no usable chip rather than as the default.**

### C. Sealed to the chip and released when a PIN is typed, and a recovery key

`cryptsetup luksFormat`, then `systemd-cryptenroll --recovery-key`, then
`systemd-cryptenroll --tpm2-device=auto --tpm2-with-pin=yes --tpm2-pcrs=7`, then
the installer's own first key wiped away — and `bootc install to-filesystem` onto
the opened volume.

- **For:** both halves of the table's first column, and the only one of the three
  that improves when shim lands without anybody re-enrolling. The chip's lockout
  is what makes a short PIN safe, and a short PIN is what makes an encrypted
  machine one a person keeps switched on. The chip binds the key to the machine;
  the PIN binds it to the person; the recovery key binds it to neither, which is
  the point of it.
- **Against:** it needs a chip, so it is not the road on every machine and the
  installer must choose. It puts one more thing between the person and their
  desktop at every start. And it makes a recovery key mandatory rather than
  advisable, which is more install to get through — the thing every option here
  needs anyway.

### D. Offer the person the choice at install

- **Against:** the question is *do you want the chip to hold your key, or your
  head?* — which cannot be asked without explaining a threat model at the one
  moment a person is trying to finish installing an operating system. A machine
  that has a usable chip has one right answer. **Rejected for v0.5**, and noted as
  a settings question for v1 for the person who wants a passphrase on a machine
  that has a chip.

## The recommendation

**C, falling back to B.** In one sentence: **the disk is sealed to this machine's
own security chip and opened with a PIN; on a machine whose chip cannot be used it
is opened with a passphrase; and either way the person writes down a recovery key
and types it back before the install finishes.**

With it:

1. **The machine chooses, not the person.** A chip that is present and ready means
   a PIN. A chip that is absent, not ready, or that we could not read means a
   passphrase — *a chip we cannot see is a chip we will not make the only place
   the key lives*. The person is asked for one secret and is never asked what LUKS
   is, what a TPM is, or which of them is holding anything.
2. **The recovery key is made before the way the person unlocks is enrolled**, so
   that until they have confirmed they kept it, nothing on the disk opens the way
   they will use tomorrow — and an install abandoned at that moment is abandoned
   cleanly. That ordering is the whole of `alo_encrypting::THE_ROAD`, and it is
   what the acceptance's *no road enrols encryption without also producing the
   recovery key* is held to.
3. **Confirming means typing it back.** Not a checkbox saying *I have written this
   down*. Dashes, case and spacing are forgiven, because a person copying 64
   characters off a screen onto paper and back will get those wrong, and refusing
   them teaches people to photograph the screen instead. The characters are not
   forgiven.
4. **The key is shown once and is never stored on the disk it recovers.** It
   exists as a value in one process, for the length of one screen, and is dropped
   when it is confirmed — `RecoveryKey::written_back` consumes it, so no later
   code can hold one. Nothing in `crates/alo-encrypting` can open a file, run a
   program or reach a network, and a test reads its source and its manifest to
   hold that.
5. **PCR 7 and nothing else**, for the reasons measured above, until there is a
   unified kernel image to measure PCR 11 against.
6. **A machine with no usable chip is still encrypted.** There is no *install
   without encryption* road in this decision. Whether one exists at all is the
   installer plan's to decide with the owner; this decision does not create one.

## What a managed machine's escrow will need at v1, without building it now

ADR 0004: a managed machine's organisation *holds a recovery key*, escrowed at
enrolment, and the person is told so at first sign-in. Nothing here escrows
anything, and nothing here reaches a network. What v1 needs from this shape is one
property, and it has it: **the recovery key exists as a value at exactly one known
moment, in one process** — between `RecoveryKey::as_printed` and
`RecoveryKey::written_back`. A managed install takes it at that moment and nowhere
else.

What v1 adds beside `WrittenDown`, and this decision deliberately does not: a
second witness that the key reached the organisation, the departure that carries
it (on the indicator and in the record, like every other departure — ADR 0004 says
management is traffic), and the sentence at first sign-in that says who holds it.
On a personal machine none of that exists, and the only copy of the key is on the
person's paper. That is the difference between the two modes, and it is one type,
not an architecture.

## Consequences if it is accepted

- **Task 6** turns `alo_encrypting::THE_ROAD` into one tested command sequence
  against a virtual disk, with a software TPM: enrol, reboot, unlock, change the
  PIN, and recover with the recovery key after the chip's measurements change. It
  measures what measurement 7 leaves open and writes it into `docs/quirks.md`.
- **The installer plan** gains the sequence and the sentences, and its single
  `install to-disk` becomes a format, two enrolments, a wipe and an
  `install to-filesystem --root-mount-spec`. `crates/alo-installing` is that
  plan's to change; this plan hands it types and a sequence and edits none of it.
- **`crates/alo-encrypting`** stops waiting: its
  `tests/the_enrolment_waits_on_its_decision.rs` fails the day this line says
  *accepted*, and is replaced by the tests of what was built.
- **Task 7** adds the encryption sentences to the vocabulary with a translator's
  note. Until then this crate says nothing to anybody: it declares no words, and
  its refusals carry English for a service log the way `alo-drives`' do.
- **`docs/hardware.md`** gains a fact that matters: whether the certified
  machine's chip is present and ready decides which of the two roads it takes.

## What the code waits on

- This decision, accepted. Tasks 6 and 7 of the plan wait on it, and the test
  named above is what makes that visible rather than remembered.
