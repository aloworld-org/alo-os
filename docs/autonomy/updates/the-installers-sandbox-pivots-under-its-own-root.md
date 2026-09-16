# The installer's sandbox pivots under a root of its own

**Date:** 2026-09-16
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 12,
*With Secure Boot on, the install finishes and boots*, as it stands after its split)
**Contributor:** Claude Code worker in `C:\dev\alo-os`, for the owner
**Status:** ready for integration — for task 12's part. The install run it also
named is task 13, **not run**, and scheduled for a machine that can hold it.

Follows `updates/the-boot-environment-says-why-an-install-stopped.md`, which is not
edited here.

## What this task ends at, and why it was split

Task 12's acceptance had three parts: locate from a run why `bwrap` cannot
`pivot_root` in the boot environment; change how the environment starts so the
installer runs where the base's own sandboxing works; and pass
`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
with Secure Boot on. **The first two are done. The third was not run.**

Before any virtual machine, as the plan's header requires, I measured the drive
holding this machine's WSL disk: `D:\wsl\Ubuntu\ext4.vhdx` (101 GB, fully
allocated), with **5.85 GB free on D:**, and later 5.45 GB. The plan's rule is 15 GB
and *says so and stops if not*: one install run left 12 GB of disks and 11 GB of
images on 2026-09-16 and crashed the distribution twice. The environment also has
to be built first (the cache on this machine holds only the base and one Ubuntu
image), and one emulated install takes about fifty minutes of a ninety-minute worker
limit. So the install run is task 13, scheduled for a machine with the room, and
the plan says so.

The emulated run that *was* made here attaches no disks. It wrote a 150 MB
initramfs and a 52 KB serial log inside the build filesystem, and D:'s free space
read the same before and after each run.

## What was found — from a run

`bootc` 1.15.1 starts its probe as `bwrap --bind <root> / --proc /proc --dev-bind
/dev /dev --tmpfs … bootupctl backend install --help`. Its binary carries those
arguments beside *Running bootupctl via bwrap in* and *Probing bootupd --filesystem
support*. `bwrap` (bubblewrap 0.10.0) always calls `pivot_root(2)`, and the kernel
answers `EINVAL` when the caller's current root mount has no parent, which is true
of the *absolute root*. The environment runs from exactly that root, the kernel's
initial root filesystem, and never switches out of it, because there is no root to
switch to. The pinned kernel (`6.19.14-101.fc42`) has no configuration option that
changes this.

**The run.** It used the base's own kernel and an initramfs made by the base's own
`dracut` (107) inside a throwaway container of the pinned base. The initramfs was
made from the repository's `image/installing/alo-installing.conf`,
`run-alo-installing-root.mount` and `alo-installing.service`, plus the diagnostic
units below. `alo-installing` and `cosign` were stand-ins (`true`), because the run
never starts them. The machine was QEMU q35 with direct kernel boot, TCG, 3 GB of
memory, 4 CPUs and no disks. Its command line was
`console=ttyS0,115200 rd.systemd.unit=diag.target rd.shell=0 rd.emergency=poweroff`.

The two units ran one after the other. Each printed its own
`/proc/self/mountinfo`, then ran `bootc`'s `bwrap` line:

```
ALO-DIAG-bare-BEGIN
1 1 0:2 / / rw shared:1 - rootfs rootfs rw,size=1401480k,nr_inodes=350370,inode64
24 1 0:23 / /proc rw,nosuid,nodev,noexec,relatime shared:2 - proc proc rw
…
bwrap: pivot_root: Invalid argument
ALO-DIAG-bare-END
[  OK  ] Finished diag-bare.service - Diagnostic: bwrap run the way bootc runs it, bare.
         Starting diag-rooted.service - Diagnostic: bwrap run the way bootc runs it, rooted...
ALO-DIAG-rooted-BEGIN
122 110 0:2 / / rw shared:44 master:1 - rootfs rootfs rw,size=1401480k,nr_inodes=350370,inode64
124 122 0:24 / /sys rw,nosuid,nodev,noexec,relatime shared:45 master:3 - sysfs sysfs rw
…
Usage: bootupctl backend install [OPTIONS] <DEST_ROOT>
…
ALO-DIAG-rooted-END
ALO-DIAG-DONE
```

The unit on the initramfs's root has root mount `1`, whose parent is `1`: the
absolute root, refused. The unit with `RootDirectory=/run/alo/installing/root` has
root mount `122`, whose parent is `110`. It holds the same files (`0:2`, rootfs),
and there the same `bwrap` pivots and runs `bootupctl`.

A first run with the two units at the same time gave the same result, with their
output interleaved. It was run again in order so each line has one owner. The log
is kept at `/root/alo-builds/alo-os-88e6ebddb0cab76e/tmp/pivot-diag/serial.log` in
this checkout's WSL. The initramfs and kernel copy were removed, and no container
was left.

The diagnostic, so the run can be repeated. It was built with `dracut
--no-hostonly --reproducible --force --kver <the one kernel>` in the pinned base,
after copying in the repository's three files above and these:

```ini
# diag.target
[Unit]
Description=Diagnostic: where bwrap can pivot
Requires=basic.target diag-done.service
After=basic.target
AllowIsolate=yes

# diag-bare.service
[Unit]
Description=Diagnostic: bwrap run the way bootc runs it, bare
DefaultDependencies=no
After=basic.target
[Service]
Type=oneshot
StandardOutput=file:/dev/ttyS0
StandardError=file:/dev/ttyS0
ExecStartPre=/usr/bin/echo ALO-DIAG-bare-BEGIN
ExecStartPre=/usr/bin/cat /proc/self/mountinfo
ExecStart=-/usr/bin/bwrap --bind / / --proc /proc --dev-bind /dev /dev --tmpfs /tmp -- /usr/bin/bootupctl backend install --help
ExecStartPost=/usr/bin/echo ALO-DIAG-bare-END

# diag-rooted.service
[Unit]
Description=Diagnostic: bwrap run the way bootc runs it, rooted
DefaultDependencies=no
After=basic.target diag-bare.service
Requires=run-alo-installing-root.mount
After=run-alo-installing-root.mount
[Service]
Type=oneshot
RootDirectory=/run/alo/installing/root
StandardOutput=file:/dev/ttyS0
StandardError=file:/dev/ttyS0
ExecStartPre=/usr/bin/echo ALO-DIAG-rooted-BEGIN
ExecStartPre=/usr/bin/cat /proc/self/mountinfo
ExecStart=-/usr/bin/bwrap --bind / / --proc /proc --dev-bind /dev /dev --tmpfs /tmp -- /usr/bin/bootupctl backend install --help
ExecStartPost=/usr/bin/echo ALO-DIAG-rooted-END

# diag-done.service
[Unit]
Description=Diagnostic: done
DefaultDependencies=no
After=diag-bare.service diag-rooted.service
Wants=diag-bare.service diag-rooted.service
[Service]
Type=oneshot
StandardOutput=file:/dev/ttyS0
ExecStart=/usr/bin/echo ALO-DIAG-DONE
ExecStartPost=/usr/bin/systemctl --no-block poweroff

# /etc/dracut.conf.d/zz-diag.conf
install_items+=" /usr/lib/systemd/system/diag.target /usr/lib/systemd/system/diag-bare.service /usr/lib/systemd/system/diag-rooted.service /usr/lib/systemd/system/diag-done.service /usr/bin/echo /usr/bin/cat "
```

## What changed

- `image/installing/run-alo-installing-root.mount` (new): binds `/` to
  `/run/alo/installing/root` with `Options=rbind,rslave`. The bind is recursive,
  so the installer keeps `/dev`, `/proc`, `/sys` and `/run` with the log's,
  D-Bus's and the network manager's sockets. It is a slave, so what `bootc` mounts
  under its root does not spread back into the environment's tree.
- `image/installing/alo-installing.service`: `RootDirectory=/run/alo/installing/root`,
  with `Requires=` and `After=` on that mount. systemd moves the bound tree onto
  `/` in the unit's own mount namespace. The installer's root then has a mount
  above it, and nothing else about the environment changes.
- `image/installing/alo-installing.conf`: carries the mount unit, and
  `/usr/bin/mount`, which the unit is mounted with.
- `image/installing/Containerfile`: copies the unit and sets its mode, and its
  header comment says why.
- `crates/alo-installing/tests/what_the_environment_carries.rs`:
  - `the_installer_runs_under_a_root_its_sandbox_can_pivot_from` holds both units
    to the shape above.
  - `a_root_the_sandbox_cannot_pivot_from_is_caught` is the refusal path. It
    catches a service with no `RootDirectory=` (the fault itself), one set to
    `/` or to another directory, `Wants=` in place of `Requires=`, and no
    `After=`. On the mount side it catches a bind at another path, a bind of
    something other than `/`, a plain `bind` (no devices or sockets), a shared
    bind, and no options.
  - `the_roots_unit_is_built_into_the_environment`: the recipe copies the unit to
    where the initramfs's list takes it from, the list takes it, and a recipe
    without the copy is caught.
  - The list check (`missing_from`) now also requires the mount unit.
- `docs/quirks.md`: the entry *bootc 1.15.1 — in the boot environment, the image
  deploys and the bootloader's probe dies in `bwrap`'s `pivot_root`* went from
  *not yet located* to located, with the run above and our response.
- `docs/booting.md`: says the cause, the change, and that the install run is task 13.
- `docs/autonomy/v0-5-the-installer-plan.md`:
  - Task 12 is marked done for its part, with the split.
  - Task 13, *With Secure Boot on, the install onto the second disk finishes and
    boots to the agent service*, is written and scheduled for a machine that can
    hold the run.
  - Task 11 now depends on 13, because it needs an install that finishes.

**User-readable change description (proposed for CHANGELOG):** *The alo OS installer
no longer stops while setting up the boot loader. That step runs in a sandbox the
installer's environment could not start, and the environment now gives it a place
where it can. Whether an install now runs all the way through is the next thing
measured.*

## Decisions

- **A bound root for one unit, not a switch of the whole environment's root.**
  Switching root into a copy in memory would double the environment's memory on a
  3 GB machine. And systemd's `switch-root` deletes the old initramfs's files,
  which on a bind of the same filesystem risks deleting the new root too. A mount
  unit plus `RootDirectory=` changes only the process tree that needs it. It is
  systemd configuration, with no patch to `bootc`, `bootupd`, `bwrap` or the
  kernel (ADR 0011).
- **`rbind`, not `bind`, and `rslave`.** A plain bind carries no `/dev`, so there is
  no disk to write and no socket to reach. A shared bind would propagate the new
  disk's mounts back into the environment.
- **`/run/alo/installing/root`.** A path with no hyphen in any part, so the unit's
  name is the path with slashes turned to hyphens and needs no `\x2d` escaping in a
  file name or a `dracut` list. The test checks that too. It is not
  `/run/alo-installing`, which is the service's `RuntimeDirectory=`.
- **Not run: the install.** Covered above. Starting it with D: at 5.4 GB would have
  been the plan's documented way to crash the distribution under another lane's
  gates.
- **The diagnostic is not a test in the tree.** It proved a kernel fact once. What
  keeps the fix in place is the configuration test, which runs in the gates in
  milliseconds. Task 13's install run is what shows the whole road.

## Verification

Platform: Windows Server 2022 checkout at `C:\dev\alo-os`. Everything below ran in
WSL Ubuntu from `/mnt/c/dev/alo-os`, with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-88e6ebddb0cab76e`.

```
$ cargo fmt --all                                                        # clean
$ cargo clippy --all-targets -p alo-installing -p alo-image -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 43.66s
$ cargo test -p alo-installing
  lib                              38 passed
  installed_in_a_virtual_machine    4 passed, 3 ignored (virtual machines, run by name; not run)
  the_boot_environment_installs    14 passed
  what_the_environment_carries      8 passed   (5 before, 3 new)
$ cargo test -p alo-image          # reads image/installing/Containerfile and docs/booting.md
  252, 6, 6, 25 passed
$ (tools/kernel-loop) cargo test plan::   # reads the plan with task 12 done, 13 written
  12 passed
```

`alo-installer` (the Windows program) is not touched, so the plan's paste of its
tests does not apply.

**Not run:**
`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`,
which is task 13. Also not run: the two refusal tests in a virtual machine. They
start machines with disks, and neither is affected by the service's root: the
not-genuine refusal happens before `bootc` runs, and the untrusted-loader refusal
happens before Linux starts. No Hyper-V machine and no physical machine was used.

## Remaining limitations

- **No install has yet been seen to finish.** The step that stopped it now works in
  the environment, but what `bootupctl` does next under Secure Boot, and whether
  the disk boots to `alo-agentd`, is task 13.
- At shutdown the rooted service's diagnostic printed *Failed unmounting
  /run/alo/installing/root/run* once, in the first run only; the machine powered
  off either way. It is harmless at power-off. If task 13 shows it holding up the
  restart, it belongs in `docs/quirks.md`.

## Proposed updates for the integration owner

- CHANGELOG: the sentence above.
- STATE: *installer task 12 done for its part. `bwrap` cannot `pivot_root` because
  the environment runs from the kernel's initial root; located from a boot of the
  environment's initramfs (`docs/quirks.md`). The installer's unit now runs under
  the environment bound at `/run/alo/installing/root`, where the same `bwrap`
  pivots. The install run is task 13, scheduled: this machine's WSL disk drive had
  5.4 GB free, under the plan's 15 GB.*
- QUEUE/ROADMAP: nothing beyond the plan's own status. The operator may want to
  know that `D:` on this PC now holds a fully allocated 101 GB WSL disk with about
  5 GB free, which stops every virtual-machine task of this plan here.
