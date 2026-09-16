# The boot environment says why an install stopped

**Date:** 2026-09-16
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 9,
*With Secure Boot on, the staged loader starts*, as it stands after its split)
**Contributor:** Claude Code worker in `C:\dev\alo-os`, for the owner, finishing the
code an earlier worker on this checkout left in the tree at the ninety-minute limit
**Status:** ready for integration.

Follows `updates/with-secure-boot-on-the-staged-loader-starts.md`, which found the
page fault and the unread choice and is not edited here.

## What this task ends at

The plan split task 9 on 2026-09-16: it ends at *the environment saying the
installer's own failure, the fault written into `docs/quirks.md` with the run's
console as evidence, and the refusal tests passing*. The install that finishes
and boots to `alo-agentd` with Secure Boot on is task 12. All three are done, and
one thing found on the way is fixed.

## What changed

### The environment keeps why a program failed, off the screen

- `crates/alo-installing/src/complaint.rs` (new) — `Complaint`, the last
  `THE_LAST_LINES` (12) non-blank lines a long program printed to its standard
  error. The end is kept because a program says why last, and a quarter of an hour
  of output is not held in the memory the install needs.
- `crates/alo-installing/src/machine.rs` — `TheMachine::note(line)`: one line of the
  machinery's own words, for the log and the serial lines, never the screen.
- `crates/alo-installing/src/console.rs` — `every_serial_line`: every console the
  kernel lists that is not a virtual terminal (`ttyS0`, `hvc0`, `ttyAMA0`; never
  `tty`, `tty0`, `tty63`).
- `crates/alo-installing/src/running.rs` — a long program's standard error is
  piped, echoed line by line into the journal as it arrives, and its end kept; the
  environment waits at most five seconds for the last words after the program
  exits, in case something it started still holds the pipe.
- `crates/alo-installing/src/sequence.rs` — when the signature check fails (not
  genuine, not reachable) or the write fails, every kept line is noted, prefixed
  with the program's whole path; a failure that printed nothing is noted as
  *failed, and said nothing*, so it cannot be mistaken for a lost line. The
  sentences a person reads are unchanged.

Why the serial lines and not the screen: `docs/features.md` promises that *a
person never learns the name of anything we rented*, and `bootc`, `bwrap` and
`cosign` are exactly those names. A serial line is where a technician, or the
virtual-machine test, reads a machine that has no other log they can reach.

### The fault, recorded

`docs/quirks.md`, *bootc 1.15.1 — in the boot environment, the image deploys and
the bootloader's probe dies in `bwrap`'s `pivot_root`*, with the run's console.
It says what is measured (the chain, the choice, the signature and the deploy
work; `bootupctl`'s probe fails because `bwrap`'s `pivot_root` returns `EINVAL`) and
marks the likely cause — the environment runs from the initramfs's root, where
`pivot_root(2)` documents `EINVAL` — as **a reading, not a measurement**, for
task 12 to prove from a run. `docs/booting.md` says the same and where a failure's
reason now goes.

### Found on the way: the Secure Boot refusal test could never have passed

`a_loader_the_firmware_does_not_trust_is_refused`, written by the previous worker
and never run, changes one byte of the staged loader and expects the firmware to
refuse it. Run for this task, **Linux started**. That looked like a firmware
starting a changed loader under Secure Boot, which would have been the most
serious finding the installer has had. It was not: the log also said *No disk was
chosen*, and the loader had read the choice in every earlier run. The cause was
the test's stager, `staged()`, which copied `/work/environment/.` **whatever
environment it was handed**. The changed copy was never staged; the genuine loader
was, and the firmware rightly started it.

`staged()` now copies the directory it is handed, through `inside_the_stager`,
which refuses a directory the stager's container cannot see rather than swapping
in the one it can. Run again, the firmware answers *Access Denied -- rejected
probably by Secure Boot* and the test passes. The mistaken run's console is kept
at `/root/alo-builds/alo-os-88e6ebddb0cab76e/refused-loader-untampered-staged.log`
in this checkout's WSL.

### Virtual-machine tests remove what they made, pass or fail

The plan's header rule since 2026-09-15 is that every virtual-machine test removes
its disks when it finishes, pass or fail. None of the three did: they removed
their disks when they *started*. `Leftovers` in
`crates/alo-installing/tests/installed_in_a_virtual_machine.rs` is a guard each of
the three now holds, naming its own environment copies, staged partition, disks,
the pre-run copy of the first disk (`windows.before`, 2.3 GB) and its variable
flash; the serial logs are kept. A guard rather than a last line, because a failed
test ends at a `panic!` and unwinding still drops it;
`what_a_test_made_is_removed_pass_or_fail` holds both roads. Both refusal runs
below left only their serial logs.

The previous worker's `Processor` change is kept as it was: the machines use
hardware virtualisation when the host can really start it, and run emulated with
eight times the deadlines when it cannot. This host's cannot.

**User-readable change description (proposed for CHANGELOG):** *When installing
alo OS stops partway, the computer's service line now records why, in the
installer's own words, while the screen keeps saying it plainly — so the reason an
install failed can be found instead of guessed.*

## Decisions

- **Serial lines and journal, never a virtual terminal.** The alternative, a
  *details* line on screen, would put rented names in front of a person and into
  the vocabulary translators receive. A laptop with no serial line keeps the lines
  only in the environment's journal, which is lost at the restart; making that
  reachable (for example, written to the installer's own partition) is a change
  to what the environment writes to disk, which task 8 made a guarantee of, so it
  is not done here and is named below.
- **The last twelve lines.** Enough for the error and what the program was doing
  when it stopped; the measured failure used four.
- **Five seconds for the last words.** A child that outlives its parent can hold
  the pipe open forever; saying how the install ended matters more than the tail.
- **The stager refuses rather than guesses.** A test helper that silently
  substituted a directory is how a Secure Boot test could never pass; one that
  panics naming the directory cannot do that again.
- **Evidence is the in-suite tests.** The two virtual-machine refusals are
  `#[ignore]`d in the suite by design and run by name below; the handoff's evidence
  names tests the supervisor can run on their own in seconds.

## Verification

Platform: Windows Server 2022 checkout at `C:\dev\alo-os`; everything below in WSL
Ubuntu from `/mnt/c/dev/alo-os`, `CARGO_TARGET_DIR=/root/alo-builds/alo-os-88e6ebddb0cab76e`,
QEMU with Fedora's `edk2-ovmf-20250812-21.fc42` taken from the pinned base,
processor emulated (TCG). WSL's disk had 911 GB free and Windows' `C:` 40.5 GB
before each virtual machine, above the plan's 15 GB.

### The gates

```
$ cargo fmt --all                                            # clean
$ cargo clippy --all-targets -p alo-installing -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s)
$ cargo test -p alo-installing
  lib                              38 passed
  installed_in_a_virtual_machine    4 passed, 3 ignored (run by name below)
  the_boot_environment_installs    14 passed
  what_the_environment_carries      5 passed
```

The tests that read the edited documents, run too: `cargo test -p alo-image`
(holds `docs/booting.md` to the code) — every target passed, 245 in its
largest; `cargo test -p alo-citing` — 21 and 10 passed; and in
`tools/kernel-loop`, `cargo test plan::` — 12 passed, reading the plan with task 9
done.

The Windows-only `alo-installer` crate is not touched, so the plan's paste of
`cargo test -p alo-installer` does not apply to this task.

### The refusals in a virtual machine, run by name

```
$ cargo test -p alo-installing --test installed_in_a_virtual_machine -- --exact \
    a_loader_the_firmware_does_not_trust_is_refused --include-ignored --test-threads 1
test a_loader_the_firmware_does_not_trust_is_refused ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 254.85s
```

Its console:

```
BdsDxe: loading Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0)
BdsDxe: failed to load Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0): Access Denied -- rejected probably by Secure Boot
```

```
$ cargo test -p alo-installing --test installed_in_a_virtual_machine -- --exact \
    a_release_signed_by_another_key_writes_nothing_and_says_so --include-ignored --test-threads 1
test a_release_signed_by_another_key_writes_nothing_and_says_so ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 258.84s
```

Its console, the kernel's lines left out — the person's sentences, and the
checker's own words noted between them, on the serial line only:

```
alo OS is being installed on this computer. Each step is written here as it happens
Reading which disk you chose before the restart
Looking for the disk you chose: virtio-alo-target
Checking that virtio-alo-target is safe to install onto
Connecting to the internet
Checking over the internet that this download is a genuine alo OS
/usr/bin/cosign: WARNING: Skipping tlog verification is an insecure practice that lacks transparency and auditability verification for the signature.
/usr/bin/cosign: Error: no matching attestations: failed to verify signature: could not verify envelope: accepted signatures do not match threshold, Found: 0, Expected 1
/usr/bin/cosign: error during command execution: no matching attestations: failed to verify signature: could not verify envelope: accepted signatures do not match threshold, Found: 0, Expected 1
This download is not a genuine alo OS, so nothing was changed
You can turn this computer off or restart it now
```

After both runs the work directory held only `firmware/` (the cached firmware) and
the serial logs; `podman image prune -f` removed the build's unnamed images, leaving
the pinned base and one Ubuntu image.

### The install's console, the evidence in `docs/quirks.md`

From the previous worker's run of
`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
on this checkout with the code in this change (`installing.log`, kept in the work
directory). **That test fails, as expected**: it is task 12's acceptance. It was
not run again here: one emulated run takes about fifty minutes, and it would show
the same step failing.

```
[    0.000000] secureboot: Secure boot enabled
…
This is a genuine alo OS
Installing alo OS onto virtio-alo-target. Everything that was on that disk is being replaced. …
/usr/bin/bootc: mke2fs 1.47.2 (1-Jan-2025)
/usr/bin/bootc: Deploying container image...done (3 minutes)
/usr/bin/bootc: error: Installing to disk: Installing bootloader: Probing bootupd --filesystem support: Subprocess failed: ExitStatus(unix_wait_status(256))
/usr/bin/bootc: bwrap: pivot_root: Invalid argument
alo OS could not be installed onto virtio-alo-target. That disk may now hold part of alo OS; nothing else on this computer was changed. Restart to try again
```

Not run: the workspace suite (the supervisor's); any Hyper-V machine; any physical
machine.

## Remaining limitations

- **The install does not finish.** That is task 12, which starts from the quirk's
  reading that the environment runs from the initramfs's root.
- **A laptop with no serial line loses the reason at the restart.** The lines are in
  the environment's journal, in memory. Keeping them somewhere a person can hand to
  support needs a write the environment does not make today, and task 8 made
  *nothing but the chosen disk is written* a tested guarantee, so it wants a
  decision about where, not a quiet file.
- Nothing here ran on the certified laptop, in Hyper-V, or with hardware
  virtualisation; timings above are emulated.

## Proposed updates for the integration owner

- CHANGELOG: the sentence above.
- STATE: *installer task 9 done: the boot environment notes a failing program's own
  words on the serial lines and never on the screen; the install stops at
  `bootupctl`'s probe with `bwrap: pivot_root: Invalid argument`
  (`docs/quirks.md`), which is task 12. The Secure Boot refusal test could never
  have passed — its stager staged the genuine loader — and now passes;
  virtual-machine tests remove their disks pass or fail.*
- QUEUE/ROADMAP: nothing beyond the plan's own status.
