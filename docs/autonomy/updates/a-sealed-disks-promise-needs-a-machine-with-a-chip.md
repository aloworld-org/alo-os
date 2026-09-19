# A sealed disk's promise needs a machine with a chip

**Date:** 2026-09-19
**Workstream:** `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, task 6 —
*Enrolled at install, and recovered*
**Contributor:** the third PC (`AGAI01`), Windows Server 2022 with WSL 2 Ubuntu
**Status:** **ready for integration as a decision.** The task's code is
**blocked** on that decision being accepted, and the plan now says so.

## What this task was, and what it turned out to be

Task 6 was to turn `alo_encrypting::THE_ROAD` into *one tested command sequence
against a virtual disk — enrol, reboot, unlock, change the PIN, recover with the
recovery key after the TPM's measurements change*, and to hold *an update that
changes boot measurements does not lock a person out* with a test that changes a
virtual machine's boot chain and unlocks afterwards.

The road it was to build is decided and is not in question:
[ADR 0054](../../decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
was accepted by the owner on 2026-09-19, option C falling back to B, and
`crates/alo-encrypting` already holds the shape.

What stopped the task is its **acceptance**, not its decision. Three of the five
things it asks for are facts about a security chip, and **no machine in this
fleet can present a chip to anything.** Measured, not assumed — the measurements
are in the new decision and are summarised below.

So the work handed over is the decision:
[**ADR 0056 — What a virtual disk shows about encryption, and what only a machine
with a chip can**](../../decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md),
proposed. The code waits on it, and the crate's existing guard has been
retargeted so that the waiting is a red suite rather than a forgotten line.

## What was measured, on 2026-09-19

All on the third PC, in the same pinned base ADR 0054 was measured in
(`quay.io/fedora/fedora-bootc`, local image `b035260f985f`).

**There is no chip, and the kernel cannot be given one.**

```
$ ls /dev/tpm*                          → No such file or directory
$ zcat /proc/config.gz | grep VTPM      → # CONFIG_TCG_VTPM_PROXY is not set
$ modprobe tpm_vtpm_proxy               → Module not found
(in the pinned base) systemd-cryptenroll --tpm2-device=list
                                        → No suitable TPM2 devices found.
```

`swtpm --vtpm-proxy` is the one road by which a software chip becomes a device
node a program can open, and it needs `/dev/vtpmx`, which needs that kernel
option. It is not set.

**The other road is a virtual machine, and every virtual machine here is
emulated.**

```
$ qemu-system-x86_64 -tpmdev help       → passthrough, emulator
$ grep -c -E "vmx|svm" /proc/cpuinfo    → 0
$ qemu-system-x86_64 -accel kvm …       → failed to initialize kvm: No such device
```

This host is itself a guest with no nested virtualisation. The one
virtual-machine acceptance this repository already owns ran that way and took
**3779 seconds**
(`the-install-finishes-under-secure-boot-and-the-installed-disk-boots.md`).

**Everything on the road that does not need a chip runs here in seconds.** A
64 MiB LUKS2 virtual disk in the pinned base, with `cryptsetup 2.8.4` and
`systemd-cryptenroll` 257.13:

```
luksFormat with an installer's first key                       rc=0
systemd-cryptenroll --recovery-key                             72 bytes on stdout
  bfiktgnt-vueecglh-nnjvgvkl-dnlnegdk-giidcibj-jbjkifbj-tjerkihi-fjnfvrhk
  (its English on stderr, with the key blanked out of it)
cryptsetup luksAddKey — the person's passphrase                rc=0
cryptsetup luksRemoveKey — the installer's first key            rc=0
header afterwards: slots 1 and 2, token 0: systemd-recovery
open with the person's passphrase                              OPENED
open with the recovery key                                     OPENED
open with the installer's wiped first key                      rc=2, no key available
open with a key that is not this disk's                        rc=2, no key available
the recovery key, searched for in the disk's own bytes          not on the disk
```

That is eight of ADR 0054's own claims held against a real LUKS2 header with no
chip anywhere — including *the key is never stored on the disk it recovers* and
*the installer's own first key opens nothing afterwards*.

**A rented tool behaves unlike its manual, and it changes the sequence.**
`systemd-cryptenroll` has `--unlock-key-file=` for the secret that opens a volume
and **no option for the secret being enrolled**. `--password` asks
interactively through `systemd-ask-password`, and closing standard input does not
make it fail — it waits. A container sat on it for two and a half minutes before
it was killed. So the passphrase road uses `cryptsetup luksAddKey`, and
`systemd-cryptenroll` is used only for the recovery key and for the chip. Written
into `docs/quirks.md` under *Pinned engines*.

## What ADR 0056 asks the owner

Where the line falls between what a virtual disk shows and what only a machine
with a chip can, given that the two roads of ADR 0054 differ by **exactly one
invocation** and it is the only one that needs a chip.

Four options, with the measurements against each: a software TPM in an emulated
virtual machine in the gates (rejected — the most expensive road, and what it
proves about a chip is that a simulator implements the specification, while the
PIN's whole argument rests on a *real* chip's lockout); waiting for the certified
laptop and building nothing (rejected — ADR 0028 settled that shape the other
way); **splitting the acceptance where the chip is** (recommended); and accepting
a simulated chip as evidence of a real one (rejected — option A's cost with the
honesty removed).

The recommendation, in one sentence: **a virtual disk shows everything about the
sequence except the chip, and the chip's three promises are shown on a machine
with a chip — and until there is one, the repository says so.**

## Decisions made in writing it, and why

- **Retargeting the guard rather than deleting it.**
  `crates/alo-encrypting/tests/the_enrolment_waits_on_its_decision.rs` failed the
  moment ADR 0054 stopped saying *proposed*, exactly as task 5 designed it to.
  Deleting it because the decision it named had been answered would have left a
  green suite over a crate that still enrols nothing. It now reads **both**
  decisions: ADR 0054 must still say *accepted* (a crate waiting on *how to
  show* a road nobody decided is waiting on the wrong thing) and ADR 0056 must
  still say *proposed*. The second test holds ADR 0056 to being a decision
  somebody actually wrote — one that cites the road it is about, names both
  halves, names the measurement that forces the split, and carries a
  recommendation an owner can accept.
- **The plan marks task 6 `blocked`, not `Done`.** It is not done. `blocked` is
  also what makes the loop step over it rather than send another worker at it
  (`tools/kernel-loop/src/plan.rs`). The plan keeps its acceptance verbatim and
  says underneath what the decision would split it into; pre-applying a proposed
  recommendation to the plan would be deciding it on the owner's behalf.
- **No new task written into the plan.** Tasks 7 and 8 already follow, so the
  plan names work after this one. The second half — the chip on a certified
  machine — is a task ADR 0056's acceptance creates, and writing it before the
  decision is accepted would be the same pre-application.
- **A separate decision rather than an amendment to ADR 0054.** ADR 0054 is
  accepted and is about *where the key lives*. This is about *what has to be
  shown, and by what*. Reopening an accepted decision to add a paragraph about
  test machinery would make it harder to read and would put the owner's
  acceptance in question over something the acceptance never covered.
- **The quirk is filed under *Pinned engines*, not in the ADR alone.** It is a
  rented tool behaving unlike its manual, which is what that section is for, and
  the next person to script an enrolment will look there rather than in a
  decision record.
- **`docs/features.md` is untouched.** Its v0.5 line is not narrowed, not
  reworded and not ticked. What changes is that the repository now says in a
  decision where that promise's evidence is missing.

## What the code waits on, and what it will be

If ADR 0056 is accepted:

- **Task 6** becomes the sequence and the virtual disk: `alo-encrypting` gains
  the enrolment as closed types — one sequence whose fifth step is either of two
  roads, no free string that becomes a device or an argument — plus the
  sentences the installer plan is handed, and an integration test that runs it
  against a virtual disk in the pinned base with `podman`, holding the eight
  claims measured above and the passphrase road end to end.
- **A new task** becomes the chip's three facts on the certified laptop: that
  the chip releases the key when the PIN is typed, that a new deployment's
  measurements do not stop it, and that a change to what PCR 7 measures does and
  the recovery key answers. Written as an `#[ignore]`d test and named in
  `docs/hardware.md` among what is measured when that machine arrives.
- **Task 7** takes the encryption sentences from the first of those.

If it is rejected in favour of option A, the emulated virtual machine with
`swtpm` is a task of its own, and this report's measurements are what it should
be budgeted against: an hour a run on the only machine in the fleet that can run
it at all.

## Verification

Gates run from this checkout through WSL, the way the supervisor runs them
(`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, `RUSTDOCFLAGS=-D warnings`).
Another checkout's workspace suite held the build cache during part of this;
Cargo's own lock serialised them and no second target directory was made.

```
cargo fmt --all                                                  ran
cargo fmt --all -- --check                                       exit 0
cargo clippy -p alo-encrypting --all-targets -- -D warnings      exit 0
cargo doc -p alo-encrypting --no-deps                            exit 0
cargo test -p alo-encrypting                                     exit 0
  src/lib.rs                                              30 passed
  tests/no_road_enrols_without_a_recovery_key_the_person_kept.rs   8 passed
  tests/the_enrolment_waits_on_its_decision.rs             2 passed
  tests/the_key_is_never_kept_on_the_disk_it_recovers.rs   4 passed
cargo test -p alo-citing                                         exit 0   (the citation check — this change adds a decision)
cargo test        in tools/kernel-loop                           exit 0   (150 passed — the plan checks read every plan)
```

Each evidence test was then run on its own, by name:

```
cargo test -p alo-encrypting --test the_enrolment_waits_on_its_decision -- --exact \
  --include-ignored nothing_is_enrolled_while_how_it_is_shown_is_undecided     1 passed
cargo test -p alo-encrypting --test the_enrolment_waits_on_its_decision -- --exact \
  --include-ignored the_decision_about_showing_it_names_both_halves            1 passed
```

**The refusal paths, shown rather than asserted.** A guard that has never been
seen to fail is a guard nobody has tested, so both were made to fail on purpose
and the decision was then restored byte for byte:

```
ADR 0056's status changed to "accepted" →
  nothing_is_enrolled_while_how_it_is_shown_is_undecided ... FAILED
  "ADR 0056 is no longer proposed (…). Build what it decided — the enrolment
   sequence, the sentences handed to the installer plan, and the virtual-disk
   acceptance of everything that needs no chip — and replace this test with the
   tests of that"

"## The recommendation" renamed in ADR 0056 →
  the_decision_about_showing_it_names_both_halves ... FAILED
  "a decision with no recommendation is a description, and the owner cannot
   accept one"
```

The first of those is the whole point of the guard: the day an owner accepts
this, the suite goes red and names the work. It is also what already happened —
the test failed on ADR 0054's acceptance, which is how this task began.

**Not run here, deliberately:** the whole workspace suite (the supervisor runs
it), and the BPF gates, which this change cannot reach. Gates chosen by
`SHARED_MAIN.md`'s *gate what the change can reach*: a decision, a plan, a quirks
entry, one crate's rustdoc and one crate's tests.

**Not measured, and it is the point of this report:** anything about a security
chip. No PCR value, no `--tpm2-with-pin` cost at boot, no chip lockout behaviour.
ADR 0054's measurement 7 left those to this task and this task cannot take them.

## Proposed changelog entry

> **The decision about how an encrypted disk is shown to work.** alo OS's disk is
> sealed to the machine's security chip and opened with a PIN, and the recovery
> key is the person's paper. Showing that it works splits in two: everything
> about the sequence that is about LUKS is shown against a virtual disk, and the
> three promises that are about a chip are shown on a machine that has one. ADR
> 0056 sets out the options and recommends the split; until it is accepted, no
> enrolment code is written.

## Proposed queue and roadmap updates

- `docs/autonomy/QUEUE.md`: the broker-and-the-disk plan's task 6 is **blocked**
  on ADR 0056 being accepted by the owner. Nothing else in that plan changes;
  tasks 7 and 8 keep the statuses they had.
- `ROADMAP.md`: the v0.5 *Full-disk encryption* line gains no tick and loses
  none. Its evidence needs a machine with a chip, which `docs/hardware.md` says
  does not exist yet.
- `docs/autonomy/STATE.md`: this report, and the decision it hands over.

## Limitations

- The decision is **proposed**. Nothing here is built, and nothing should be
  until an owner answers.
- The virtual-disk measurements were taken by hand in a container, not yet by a
  test in this repository. They are evidence for the decision, not acceptance of
  anything — that test is the first half of what the decision would create.
- `docs/hardware.md` is not edited here. ADR 0056's consequences add the chip to
  what the certified machine is measured for, and that edit belongs in the change
  that creates the task, not in the one that proposes it.
