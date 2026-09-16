# The install finishes under Secure Boot, and the installed disk boots

**Date:** 2026-09-16
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 13,
*With Secure Boot on, the install onto the second disk finishes and boots to the
agent service*)
**Contributor:** Claude Code worker in `C:\dev\alo-os`, for the owner
**Status:** ready for integration — for task 13 as split. **The install onto the
second disk finishes with Secure Boot on, and that disk boots under Secure Boot;
`alo-agentd` then failed on it**, so the named virtual-machine test does not yet
pass as a whole. That is task 14, written into the plan in this change.

Follows `updates/the-installers-sandbox-pivots-under-its-own-root.md` (task 12),
which is not edited here.

## What the run before this one said

The supervisor's run of 2026-09-16 (`C:\dev\setup\task13-install-run.log` on this
PC) was the first with the boot loader's sandbox pivoting. Under Secure Boot the
environment read the choice, found the release genuine, and `bootc` deployed it —
and then:

```
/usr/bin/bootc: mke2fs 1.47.2 (1-Jan-2025)
/usr/bin/bootc: Deploying container image...done (3 minutes)
/usr/bin/bootc: error: Installing to disk: No such file or directory (os error 2)
alo OS could not be installed onto virtio-alo-target. That disk may now hold part of alo OS; nothing else on this computer was changed. Restart to try again
You can turn this computer off or restart it now
```

No program, no path. The test then sat 71 minutes more waiting for a power-off
that an environment which could not install never does.

## What was found

**The missing program was `fstrim`.** Read from bootc 1.15.1's own source (the
tag the pinned base ships, `rpm -q bootc` → `bootc-1.15.1-1.fc42`), and then
shown by the run below:

- bootc names each step with an error context, and anyhow prints every context in
  the chain. The error carried only `install_to_disk`'s own, *Installing to disk*,
  so it came from a call inside that adds none. *Installing bootloader* (the
  `bootupd` step, including the `bwrap` run), *Querying for bootupd*, *Writing
  aleph version* and *Opening deployment dir* are all ruled out by that.
- What is left after the deploy is the bound images (the release has none), the
  image store's SELinux labels (off in the environment), `install_finalize`
  (an ostree load, whose errors are glib's), and **`finalize_filesystem`**:
  `fstrim --quiet-unsupported -v .`, `mount -o remount,ro .`, `fsfreeze -f`/`-u`
  for each file system bootc made; then `umount -R` in `install_to_disk`.
- `Task::run` starts its program with a bare `cmd.spawn()?`. A program that is not
  there is exactly `No such file or directory (os error 2)` and nothing else.
- `alo-installing.conf` named `mount` and `fsfreeze`; dracut's `base` module
  brings `umount`; **nothing brought `fstrim`**. In the pinned base, `grep -rlw
  fstrim /usr/lib/dracut/` finds nothing (exit 1). After the change, `lsinitrd`
  of the rebuilt initramfs lists `usr/bin/fstrim`, `usr/bin/mount`,
  `usr/bin/fsfreeze` and `usr/bin/umount`.

The whole reasoning and both consoles are in `docs/quirks.md`, *`bootc install`
ends "No such file or directory" when the environment lacks `fstrim`*.

## What changed

- `image/installing/alo-installing.conf` — carries `/usr/sbin/fstrim`, and names
  `/usr/bin/umount` rather than leaving it to a dracut module, with the reason and
  the run beside them. Configuration only; bootc, bootupd and dracut are the base's,
  unmodified (ADR 0011).
- `crates/alo-installing/tests/what_the_environment_carries.rs` —
  `WHAT_THE_INSTALLER_FINISHES_WITH` (`fstrim`, `mount`, `fsfreeze`, `umount`);
  `the_initramfs_carries_what_the_installer_finishes_with`, and its refusal
  `a_list_missing_what_the_installer_finishes_with_is_caught`, which drops each
  one on its own and comments out the new line.
- `crates/alo-installing/tests/installed_in_a_virtual_machine.rs` — the install's
  wait ends **as soon as the environment says** *You can turn this computer off or
  restart it now*, said on a line of its own, and fails with the serial line's end
  (`waits_for_a_person`). The installed disk's boot keeps waiting for power-off,
  because that machine powers itself off. Held by
  `an_install_that_could_not_finish_ends_the_wait_when_it_says_so`, against the
  previous run's own lines, a run still installing, a run that finished, and a line
  that only quotes the sentence.
- The same file — the installed disk's watching unit also prints `systemctl
  status` and the boot's journal for `alo-boundaryd`, `alo-agentd` and
  `user@1000.service` between `ALO-WHY-BEGIN` and `ALO-WHY-END` (each line
  `-`-prefixed, so a diagnostic that fails never stops the report), and a service
  that is not running fails the test quoting them (`why_it_said`). Held by
  `a_service_that_is_not_running_is_reported_with_why`. Added after the run below
  reported `alo-agentd` *failed* and nothing more.
- `docs/quirks.md` — the entry above.
- `docs/autonomy/v0-5-the-installer-plan.md` — task 13 marked done for its part,
  task 14 written, task 11's dependency moved to 14.

**User-readable change:** the installer's boot environment now carries the one
tool the install needs to finish the disk it has just written, so an install
under Secure Boot no longer stops after downloading alo OS with an error that
names nothing.

## Decisions

- **Carry `umount` by name as well**, though dracut's `base` brings it: the list is
  what `crates/alo-installing` holds to what the install runs, and a program the
  install needs is safer named there than inherited from a module that could be
  omitted.
- **Held in the test crate, not in `src/program.rs`.** `EVERY_PROGRAM` is what
  *this* crate runs; `fstrim` and the others are what bootc runs in turn, like the
  rest of the list's second half, so the constant lives with the test that reads
  the list.
- **The test ends on the environment's last sentence, not on *not-installed*.**
  Every refusal and failure of the environment ends with *You can turn this
  computer off or restart it now* and then waits; ending on that one line covers
  all of them, and a success never says it.

## Verification

All on the third PC (`AGAI01`), Windows Server 2022 with WSL 2 Ubuntu, no hardware
virtualisation behind `/dev/kvm` (the probe says *failed to initialize kvm: No
such device*), so every machine was emulated (TCG).

**Gates, executed, in WSL** (`CARGO_TARGET_DIR=/root/alo-builds/alo-os-88e6ebddb0cab76e`),
after the last change:

```
cargo fmt --all -- --check                                   FMT-OK
cargo clippy -p alo-installing --all-targets -- -D warnings  exit 0
cargo test -p alo-installing                                 exit 0
  src/lib.rs                          38 passed
  tests/installed_in_a_virtual_machine.rs   6 passed, 3 ignored (the machines, run by name)
    an_install_that_could_not_finish_ends_the_wait_when_it_says_so ... ok
    a_service_that_is_not_running_is_reported_with_why ... ok
  tests/the_boot_environment_installs.rs   14 passed
  tests/what_the_environment_carries.rs    10 passed
    a_list_missing_what_the_installer_finishes_with_is_caught ... ok
    the_initramfs_carries_what_the_installer_finishes_with ... ok
```

`cargo fmt --all` was also run from the Windows checkout. No other crate was
touched; the workspace suite was not run here (the supervisor runs it).

**The virtual-machine run, executed once**, before the watcher's *why* lines were
added (they change only what the second boot prints after it has reported):

```
free before the run: 159 GB
cargo test -p alo-installing --test installed_in_a_virtual_machine -- --exact \
  --include-ignored the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service --nocapture
```

The install, `installing.log` (escape sequences and blank lines removed; repeated
lines counted):

```
[    0.000000] secureboot: Secure boot enabled
alo OS is being installed on this computer. Each step is written here as it happens
Reading which disk you chose before the restart
Looking for the disk you chose: virtio-alo-target
Checking that virtio-alo-target is safe to install onto
Connecting to the internet
Checking over the internet that this download is a genuine alo OS
This is a genuine alo OS
Installing alo OS onto virtio-alo-target. Everything that was on that disk is being replaced. This takes a while, and this screen will say when it is done
[   69.079264]  vdb: vdb1 vdb2 vdb3
[   71.936408] EXT4-fs (vdb3): mounted filesystem 70b9e529-263a-46bb-89ec-4afd00216072 r/w with ordered data mode. Quota mode: none.
Still installing alo OS. Leave the computer on            (44 times)
[ 2777.778647] EXT4-fs (vdb3): re-mounted 70b9e529-263a-46bb-89ec-4afd00216072 ro.
alo OS is installed. This computer restarts in a few seconds
[ 2795.317208] EXT4-fs (vdb3): unmounting filesystem 70b9e529-263a-46bb-89ec-4afd00216072.
```

The test's checks after the install **passed**: every step said in order, no
refusal said, **the first disk hashed the same as before the install**, and the
second disk was written.

The installed disk, started on its own with the same Secure Boot firmware and
Microsoft's variables, `installed.log`:

```
BdsDxe: starting Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0)
GRUB version 2.12      *Fedora Linux 42 (Adams) (ostree:0)
Booting `Fedora Linux 42 (Adams) (ostree:0)'
PageFaultExitBoot: Page fault fixups needed (NX: 0, RW: 1).
PageFaultExitBoot: The guest OS boot chain is not NX clean.
PageFaultExitBoot: Applying global page table fixup (shim is older than v16).
Id=alo-boundaryd.service
ActiveState=active
SubState=exited

Id=alo-agentd.service
ActiveState=failed
SubState=failed
```

```
test the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service ... FAILED
panicked at crates/alo-installing/tests/installed_in_a_virtual_machine.rs:1056:9:
alo-agentd.service is not running on the installed machine:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 3779.29s
EXIT=101 SECONDS=3797
free after the run: 153 GB
```

So the installed disk was started by the firmware through the base's signed shim
and loader with Secure Boot enforcing (the *PageFaultExitBoot* lines are the same
firmware's, under the same shim, as `docs/quirks.md` records for task 9), its
systemd ran the test's watching unit, and the boundary was loaded. `alo-agentd`
failed, and the watching unit of that run said nothing about why. The assertion
*the first disk is unchanged when alo OS booted* comes after the service check
and was not reached.

**After the run:** the test's guard removed `environment`, `staged.img`,
`windows.raw`, `windows.before`, `target.raw` and both variable files; the work
directory holds only the firmware taken from the base and the two serial logs.
`podman image prune -f` ran; `podman system df` shows the two pinned images and
no containers. `/root/bootc-src`, where bootc 1.15.1's source was unpacked to read
it, was removed. The 6 GB between *before* and *after* is the build directory's
compiled test and gate artefacts on the same filesystem, not disks.

**The earlier supervisor run** is quoted from `C:\dev\setup\task13-install-run.log`,
not re-run.

## Remaining limitations

- **`alo-agentd` is not running on the installed disk**, so
  `the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
  does not pass, and *the first disk unchanged when alo OS booted* was not reached.
  Task 14 is written for it. Its first run prints the unit's status and journal
  (`ALO-WHY-BEGIN` … `ALO-WHY-END`), and the failure quotes them.
- The cause of the *No such file or directory* was located by reading bootc's
  source for the only context-free spawns after the deploy and confirmed by the
  run that passed that point, not by a run with `fstrim` alone removed again.
  `mount`, `fsfreeze` and `umount` were already present, so `fstrim` is the only
  change between the two runs' environments.
- No new run of `a_release_signed_by_another_key_writes_nothing_and_says_so` or
  `a_loader_the_firmware_does_not_trust_is_refused`. Neither's path changed: the
  first ends before bootc runs, the second before a kernel starts, and neither
  waits in `powered_off`.
- Emulated only. Timings are not measurements (46 minutes to install; the task 2
  report measured about twelve with hardware virtualisation).

## Proposed updates for the integration owner

- **CHANGELOG.md:** *The installer's boot environment carries the tool it needs to
  finish the disk it has just written. With Secure Boot on, an install onto a
  second disk now completes, and that disk boots.*
- **ROADMAP.md:** no box moves. Nothing here ran on the laptop, and the install
  does not yet reach `alo-agentd`.
- **QUEUE.md / STATE.md:** installer plan task 13 done for its part, as split;
  task 14 (*The disk installed under Secure Boot runs the agent service*) ready,
  on a machine that holds one emulated run of about an hour; task 11 now depends
  on 14 rather than 13.
