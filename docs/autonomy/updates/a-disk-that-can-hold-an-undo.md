# A disk that can hold an undo

**Date:** 2026-09-21
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 11,
*The disk alo OS is installed onto can hold an undo*)
**Contributor:** Claude Code lane B in `/root/alo-os-lane-b` on the **development PC**,
for the owner
**Status:** **done.** The installer names `btrfs`; what the base makes of it was
measured on the pinned release in a virtual machine rather than assumed; an update
and a return to the build before leave a person's home subvolume and `/var/lib/alo`
byte for byte; and a second filesystem on the road to a person's disk is now a
failing test.

Answers [ADR 0045](../decisions/0045-what-undoing-rewinds-to.md)'s *measured first,
before building*, and its sixth accepted term: the installer's change to `btrfs`
lands **before the certified laptop is installed**, because a filesystem is chosen
at install and cannot be converted afterwards.

## What changed

| File | What it does now |
|---|---|
| `crates/alo-image/src/filesystem.rs` | New. `THE_ONLY_FILESYSTEM` (`btrfs`) is the one value every writer takes, and `TheFilesystem` reads every `--filesystem` in a text so a **second** one cannot appear unnoticed |
| `crates/alo-image/tests/one_filesystem_and_it_can_hold_an_undo.rs` | New. Reads the installer's program and `docs/booting.md` and holds each to naming one filesystem, and to its being one that has snapshots at all |
| `crates/alo-installing/src/writing.rs` | `--filesystem ext4` → `alo_image::THE_ONLY_FILESYSTEM`, with the reason and a test that counts the argument rather than looking it up |
| `docs/booting.md` | Both invocations write `--filesystem btrfs`, with a paragraph saying which argument here cannot be taken back and why |
| `crates/alo-installing/tests/a_btrfs_disk_keeps_what_an_update_passes_over.rs` | New. A btrfs install, known bytes, an update, a rollback, and everything found on the far side |
| `docs/quirks.md` | Four entries, each with the command that showed it |

Nothing was changed in another lane's crates. `crates/alo-keeping-up` already
answers *not yet on this machine* for every undo and goes on doing so: this task
lands the filesystem, not the bracket.

## What the base makes of `--filesystem btrfs`, measured

The pinned release `0.0.5`
(`ghcr.io/aloworld-org/alo-os@sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1`,
built from `97c970c9`) installed onto a 20 GB raw disk, and booted with KVM. Full
entries with their commands are in `docs/quirks.md`; the findings are:

1. **The base creates no subvolume of its own.** One `mkfs.btrfs` and nothing else.
   On the machine it made, `btrfs subvolume list -a -p -u /sysroot` prints nothing,
   and `/boot`, `/etc`, `/sysroot` and `/var` are four bind mounts of four
   directories inside subvolume 5 — `/` itself is a read-only composefs overlay and
   not btrfs at all. So *installed on btrfs* does not mean *has somewhere to
   snapshot*.
2. **A person's home lands in `/var/home`, which is an ordinary directory.** The
   image's login says `/home/alo`, `/home` is `/var/home`, and on a machine nobody
   has signed into it is empty; `btrfs subvolume show /var/home` answers *Not a
   Btrfs subvolume*. Making each home its own subvolume is the accounts lane's, as
   ADR 0045 assigns it. A home *made* a subvolume behaves exactly as the decision
   needs, on the same machine.
3. **Taking a read-only snapshot needs no capability. Removing one needs
   `CAP_SYS_ADMIN`.** With `CAP_SYS_ADMIN` dropped, root still took one
   (`take_without_sys_admin_exit=0`, `ro=true`) and could not remove it
   (`ERROR: Could not destroy subvolume/snapshot: Operation not permitted`); with it
   held, removal succeeded. The person took one into a directory they owned and was
   refused one into a directory root owned. `bootc install` sets no
   `user_subvol_rm_allowed` and we add no mount option of our own.
4. **A snapshot into a destination that already exists is made *inside* it** and
   answers `Read-only file system`, which reads exactly like a mount problem.

### The cost the plan did not anticipate

Finding 3 is worth putting in front of whoever builds the bracket. ADR 0045's
accepted terms assume undo's snapshots can be expired — a seven-day window,
oldest-go-first under disk pressure, and *forgetting it is one act*. **Taking the
bracket is cheap and needs no privilege; expiring it needs a privileged remover**,
and a read-only snapshot cannot even be cleared with `rm -rf`. A design that gave
both halves to the same authority would find out on a machine that had filled with
snapshots nothing could delete — which is the failure the owner's own question was
about. This is not a reason to reopen option A; it is one more line for the broker's
list of fixed verbs.

## The run: a btrfs install boots to `alo-agentd`, takes a snapshot and removes it

The machine was given a measuring unit through the firmware's tables — systemd's
`io.systemd.credential.binary:systemd.extra-unit.…`, the way task 13's test hands one
over — so nothing on the installed disk was changed to make this run. Console,
verbatim:

```
ALO-T11-BEGIN
Id=alo-boundaryd.service
ActiveState=active
SubState=exited

Id=alo-agentd.service
ActiveState=active
SubState=running
--- uname ---
6.19.14-101.fc42.x86_64
--- btrfs version ---
btrfs-progs v6.19.1
--- subvolume list ---
--- subvolume show var home ---
ERROR: Not a Btrfs subvolume: Invalid argument
--- who lives in var home ---
total 0
drwxr-xr-x. 1 root root   0 Sep 21 00:57 .
drwxr-xr-x. 1 root root 266 Sep 21 00:57 ..
alo:x:1000:1000:alo OS:/home/alo:/bin/bash
--- a home made a subvolume ---
Create subvolume '/var/home/probe'
--- read only snapshot of it ---
Create readonly snapshot of '/var/home/probe' in '/var/lib/alo/undo/probe-before'
	Name: 			probe-before
	Parent UUID: 		a53de0e5-4842-914b-a436-95a48ad7df78
	Subvolume ID: 		257
	Flags: 			readonly
kept
rm: cannot remove '/var/lib/alo/undo/probe-before/a-file': Read-only file system
--- removed again ---
Delete subvolume 257 (no-commit): '/var/lib/alo/undo/probe-before'
Delete subvolume 256 (no-commit): '/var/home/probe'
ALO-T11-END
```

`alo-agentd` is `active (running)` on a disk installed with `--filesystem btrfs`;
the read-only snapshot held the file the home held (`kept`), refused a write, and
was removed again leaving nothing behind.

## The run: an update and a return leave the home subvolume and `/var/lib/alo` alone

`crates/alo-installing/tests/a_btrfs_disk_keeps_what_an_update_passes_over.rs`,
run by name on the development PC with KVM:

```
running 1 test
test an_update_and_a_return_leave_the_home_subvolume_and_the_machines_own_alone ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 161.84s
```

Exit 0, and again in 124.69 s and 130.88 s on two further runs. Inside the machine,
three starts, as the console says them:

```
[   22.706415] alo-btrfs-test[992]: test in_the_machine ... alo-btrfs-test: on the first build: passed
[    8.689609] alo-btrfs-test[968]: test in_the_machine ... alo-btrfs-test: on the second build: passed
[    5.566222] alo-btrfs-test[959]: test in_the_machine ... alo-btrfs-test: back on the first build: passed
```

The first start finds `/var` is btrfs, makes `/var/home/ada` a subvolume, writes a
letter and a two-megabyte photograph into it and the grants and the record into
`/var/lib/alo`, takes the read-only snapshot an undo would rewind from, and runs
`bootc switch` to the second build. The second start finds the second build running
and every one of those bytes where it was, the home still a subvolume and the
snapshot still read-only and still holding what it held, and runs `bootc rollback`.
The third start finds the first build running and all of it still true.

## What was verified, and where

| Check | Result |
|---|---|
| `cargo fmt --all --check` | in the gate below |
| `cargo clippy --workspace --all-targets -- -D warnings` | in the gate below |
| `cargo test --workspace` | in the gate below |
| `cargo test -p alo-installing --test a_btrfs_disk_keeps_what_an_update_passes_over -- --ignored --exact …` | **ok, 1 passed, 161.84 s**, exit 0 |
| The measuring boots | five, with KVM, on the pinned release |

The disks were written under `CARGO_TARGET_TMPDIR` and removed when the test ended,
its two images were removed with them, and the measuring disk was deleted by hand:
`fstrim` returned 12.5 GiB on the first pass and 0 on the second. C: was checked
before every machine and never went below 24 GB free.

## Remaining limitations

- **No machine has a home subvolume by default.** The installer's half is landed;
  the accounts lane makes the home a subvolume, `alo-turn` takes the bracket and the
  broker holds the privilege, each named in ADR 0045. Until all four exist on a
  machine, every undo answers *not yet on this machine*, which is true rather than a
  stub.
- **Machines already installed are on `ext4` and stay there.** There is no
  conversion; they answer *not yet on this machine* for good, until reinstalled.
  This is why the argument had to land before the certified laptop.
- **The update and rollback tests in `crates/alo-updating/tests/` still install on
  `ext4`.** They measure the base's promise about `/var` and `/etc`, which this
  change does not touch, and they belong to another lane (ADR 0028). What they now
  measure is a filesystem the product no longer installs; whoever owns them may want
  to follow.
- **Not the certified machine.** Everything here is a virtual machine, as the plan
  requires; Secure Boot was off in these guests, which task 11 does not ask for.
