# ADR 0056 — What a virtual disk shows about encryption, and what only a machine with a chip can

**Status:** **proposed, 2026-09-19.** Written by task 6 of
`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` (*Enrolled at install, and
recovered*), which cannot be finished as its acceptance is written: the
acceptance asks for a software TPM in a virtual machine, and **no machine in
this fleet can present a TPM device to anything**, measured below. The code that
task 6 would write waits on this, and
`crates/alo-encrypting/tests/the_enrolment_waits_on_its_decision.rs` is what
makes the waiting visible rather than remembered — it now fails the day this
line stops saying *proposed*.
**Date:** 2026-09-19
**Proposed by:** the broker-and-the-disk workstream, on the third PC
**Context:** [ADR 0054](0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
(the disk is sealed to this machine's chip and opened with a PIN, with a
recovery key the person writes down — **accepted, and nothing here reopens a
word of it**);
[ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
(work whose evidence needs hardware nobody has yet is ordered around the
hardware, not pretended about);
[ADR 0033](0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
(the certified laptop is where an install is shown to work, and §4 why Secure
Boot is off on it);
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (LUKS,
`systemd-cryptenroll` and the TPM stack are rented and never patched);
`CLAUDE.md` law 3 (*an integration test on real hardware for anything that
touches the machine*); `docs/features.md` v0.5 *Full-disk encryption, enrolled at
install*; `docs/hardware.md`; `crates/alo-encrypting`.

## The question in one line

**ADR 0054 decided that this machine's disk is sealed to its security chip and
opened with a PIN. What has to be shown before that is written down as done —
and where does each half of it run?**

## What is true today, measured rather than remembered

All of it on the third PC (`AGAI01`), Windows Server 2022 with WSL 2 Ubuntu,
2026-09-19, in the same pinned base ADR 0054 was measured in
(`quay.io/fedora/fedora-bootc`, local image `b035260f985f`).

1. **There is no TPM device on this machine, and the kernel cannot be given
   one.** `/dev/tpm*` does not exist. The WSL kernel
   (`6.18.33.2-microsoft-standard-WSL2`) is built with `CONFIG_TCG_TPM=y` and
   **`# CONFIG_TCG_VTPM_PROXY is not set`**, so `/dev/vtpmx` does not exist and
   `modprobe tpm_vtpm_proxy` answers *Module not found*. That is the one road by
   which a software TPM (`swtpm --vtpm-proxy`) becomes a device node a program
   can open, and it is closed. Inside the pinned base,
   `systemd-cryptenroll --tpm2-device=list` answers ***No suitable TPM2 devices
   found*** — which is ADR 0054's own measurement 7, met again from the other
   side.
2. **The remaining road is an emulated virtual machine, and it is emulated.**
   `qemu-system-x86_64` 8.2.2 is here and offers the `emulator` TPM backend, so
   QEMU plus `swtpm` would give a guest a chip. But `/proc/cpuinfo` shows
   **no `vmx` and no `svm`** — this host is itself a guest with no nested
   virtualisation — and `qemu -accel kvm` answers *failed to initialize kvm: No
   such device*. Every guest here is TCG. The one virtual-machine acceptance
   this repository already has ran in exactly that way and took **3779 seconds**
   (`docs/autonomy/updates/the-install-finishes-under-secure-boot-and-the-installed-disk-boots.md`).
3. **An emulated chip is not the chip the decision rests on.** ADR 0054's
   argument for a six-character PIN is *the chip's own lockout counts wrong
   PINs, so a six-character PIN is worth what a far longer passphrase is worth
   against an offline attack*. That lockout is a real chip's implementation of
   dictionary-attack protection — how many wrong answers, how long the
   recovery takes, what the manufacturer shipped. `swtpm` has a lockout because
   the specification says to have one; it is not evidence about the chip in the
   certified laptop. **The most expensive road here is also the one that proves
   the least about the thing that made the decision.**
4. **Everything that does not need a chip runs here in seconds.** Measured on a
   64 MiB virtual disk in the pinned base, `cryptsetup 2.8.4` and
   `systemd-cryptenroll` 257: `luksFormat` with an installer's first key;
   `systemd-cryptenroll --recovery-key`, which again wrote **72 bytes to
   stdout** and its English to stderr; a second unlock secret added; the
   installer's first key removed; and then — the whole point — **the volume
   opened with the person's secret, opened with the recovery key, refused the
   installer's wiped first key (exit 2), and refused a key that was not this
   disk's (exit 2)**. The recovery key was not found anywhere in the disk's
   bytes. That is six of ADR 0054's own claims, held against a real LUKS2
   header, with no chip anywhere.
5. **The two roads differ by one invocation, and it is the only one that needs a
   chip.** ADR 0054's road is: format, enrol the recovery key, show it, take it
   back, **enrol the way the person unlocks**, wipe the first key. Only that
   fifth step differs between the chip road
   (`systemd-cryptenroll --tpm2-device=auto --tpm2-with-pin=yes --tpm2-pcrs=7`)
   and the passphrase road. Five of the six steps are the same commands on both.
6. **`systemd-cryptenroll` cannot be given the new secret without a terminal.**
   It has `--unlock-key-file=` for the secret that opens the volume and **no
   option at all for the secret being enrolled**: `--password` asks
   interactively, and with standard input closed it does not fail, it waits.
   Measured by a container that sat on it for two and a half minutes before it
   was killed. So a scripted passphrase enrolment uses
   `cryptsetup luksAddKey`, and `systemd-cryptenroll` is used for the recovery
   key and the chip and nothing else. Written into `docs/quirks.md`.

## What the two halves are

Taking measurements 4 and 5 together, ADR 0054's promises separate cleanly, and
the separation is not a convenience — it is the line `CLAUDE.md` law 3 already
draws between *the code* and *the machine*, and the one this plan's own header
draws when it says *an encrypted disk that has never booted on certified
hardware is `- [x] The code.`*

**What a virtual disk shows** — every command in the sequence except one, and
every promise that is about LUKS rather than about a chip:

- the six steps happen in ADR 0054's order, and the recovery key is made, shown
  and typed back before anything the person will unlock with exists;
- the recovery key is the shape the tool prints and a person can copy;
- the installer's own first key opens nothing once the install is finished;
- the person's secret opens the volume, the recovery key opens the volume, and
  neither anything else nor a near miss does;
- the recovery key is not on the disk it recovers;
- the passphrase road — ADR 0054's option B, which is what every machine with no
  usable chip gets — is shown **end to end**, because it needs no chip at all.

**What only a machine with a chip shows** — three facts, and every one of them
is a fact about a chip:

- the chip releases the key when the PIN is typed, and the machine reaches its
  desktop;
- a new deployment — a new kernel, a new initramfs, new boot entries — does not
  stop it, which is ADR 0054's claim that an update does not lock anybody out;
- a change to what PCR 7 measures does stop it, and the recovery key is what
  opens the machine then.

## The options

### A. A software TPM in an emulated virtual machine, in this repository's gates

Install `swtpm`, build a guest from the pinned base, boot it under TCG, and run
the whole of task 6's acceptance inside it.

- **For:** it is what task 6's acceptance says, word for word, and it runs
  wherever the gates run.
- **Against:** measurements 2 and 3. It is the most expensive thing this
  repository would own — an hour a run on the only machine that can run it, on
  top of a guest nobody has built yet — and what it would prove about the chip
  is that a simulator implements the specification. It also makes the *cheap*
  half hostage to the expensive one: the passphrase road, the recovery key and
  the wipe would sit behind an hour of emulation that has nothing to do with
  them. **Rejected as the acceptance. Kept as an option for a later release**,
  where it is worth having as a *regression* test on a machine with nested
  virtualisation — which is a thing to buy, not a thing to decide.

### B. Wait for the certified laptop and build nothing until it arrives

- **Against:** ADR 0028 settled the shape of this question already, the other way
  round: work whose evidence needs hardware is *ordered around* the hardware, not
  stopped by it. Five sixths of the sequence needs no chip, and the installer
  plan needs the sequence and the sentences now. **Rejected.**

### C. Split the acceptance where the chip is

The code, the sequence and every promise that is about LUKS are shown on a
virtual disk in the gates. The three facts that are about a chip are shown on a
machine with a chip, named as such, and the repository says plainly that they
are not shown until then.

- **For:** each half is shown by the cheapest thing that can honestly show it,
  and neither half is waiting on the other. It matches what this repository
  already does for the install (ADR 0033) and for the GPU (ADR 0028): the
  code-shaped part is a test, the machine-shaped part is a machine, and which is
  which is written down.
- **Against:** for as long as there is no certified machine, `docs/features.md`'s
  v0.5 *Full-disk encryption, enrolled at install* is not shown to work on any
  machine — and this decision requires that to be said in public rather than
  implied by a green suite. That is a cost, and it is the honest one.

### D. Accept a simulated chip as evidence of a real one

Run option A once and record it as the acceptance.

- **Against:** it is option A's cost with option C's honesty removed. A person
  reading *the disk is sealed to your machine's security chip* would be reading a
  claim whose only evidence is a program that agreed with itself. **Rejected.**

## The recommendation

**C.** In one sentence: **a virtual disk shows everything about the sequence
except the chip, and the chip's three promises are shown on a machine with a
chip — and until there is one, the repository says so.**

With it:

1. **Task 6 becomes two tasks**, and the plan says which acceptance belongs to
   which. The first is the sequence, the types, the sentences the installer plan
   is handed, and the virtual-disk acceptance of measurement 4's six claims plus
   the passphrase road end to end. The second is the chip's three facts on the
   certified laptop, and it is `- [x] The code.` until that machine exists
   (`docs/hardware.md`: nothing is certified yet).
2. **Neither task may tick `docs/features.md`'s v0.5 line on its own.** The
   feature is *enrolled at install*, and an install that has never happened on a
   machine with a chip has not enrolled anything. The reconciliation between
   promises and evidence is `alo-reconciling`'s and the integration owner's; this
   decision only says what the two tasks may claim.
3. **The chip half is written as a test, `#[ignore]`d, and named in
   `docs/hardware.md`** as one of the things measured when the certified machine
   arrives — beside *the GPU works on first boot*. A fact that needs a machine is
   a test somebody runs on that machine, not a paragraph somebody remembers.
4. **`docs/hardware.md` gains the chip as a thing to check on arrival**, which
   ADR 0054's consequences already asked for: whether the machine's chip is
   present and ready decides which of the two roads it takes, and that is now a
   line in the acceptance rather than an assumption about business laptops.
5. **The enrolment is one sequence with one branch.** Because the roads differ
   by exactly one invocation (measurement 5), the crate holds one sequence whose
   fifth step is either of two, rather than two sequences that will drift. The
   virtual-disk acceptance therefore covers every command on the chip road too,
   except the one it cannot run.
6. **`systemd-cryptenroll` is not used to enrol a passphrase** (measurement 6).
   The recovery key and the chip are its; the passphrase is
   `cryptsetup luksAddKey`'s.

## What this decision does not do

It does not reopen ADR 0054. The road, the order, PCR 7, the PIN's length, the
recovery key and its consumption are decided and are not touched here — this is
about what has to be *shown*, and by what.

It does not weaken a gate. Nothing here exempts anything from `CLAUDE.md`'s gate:
it says which tests exist and where they run, and both halves are tests.

It does not create an *install without encryption* road. ADR 0054 left that open
to the installer plan and the owner, and it stays open.

## Consequences if it is accepted

- **`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`** carries task 6 as the
  sequence and the virtual disk, and a new task for the chip on a certified
  machine. Task 7's walk gains the encryption sentences from the first of them.
- **`crates/alo-encrypting`** gains the sequence as closed types with no free
  string that becomes a device or an argument, and the integration test that runs
  it against a virtual disk in the pinned base with `podman`. Its
  `tests/the_enrolment_waits_on_its_decision.rs` — which this change retargets
  from ADR 0054 to this one — is replaced by those tests.
- **`docs/hardware.md`** gains the chip's three facts in what is measured when
  the certified machine arrives.
- **`docs/features.md`'s v0.5 encryption line stays unticked** until a machine
  with a chip has been installed onto and opened. Nothing about that promise is
  narrowed; what changes is that the repository now says where its evidence is
  not.

## What the code waits on

- This decision, accepted. Task 6's code waits on it, and
  `crates/alo-encrypting/tests/the_enrolment_waits_on_its_decision.rs` fails the
  day this line stops saying *proposed*, which is how a decision gets built
  rather than remembered.
