# The install finishes on the installer's own road, and the installed system starts

**Date:** 2026-09-22
**Workstream:** the installer plan's task 18
**Contributor:** Claude Code, lane A (`/root/alo-os`, WSL Ubuntu on the
development PC, Intel Core Ultra 7 155U, 15.5 GB)
**Status: done.** Nothing is ticked on a machine. This ran in a virtual machine.

## In one paragraph

On the installer's own road, `bootc` stopped at *Creating rootfs: No such file
or directory*. The cause was that the environment's initramfs had no
`mkfs.btrfs`. The installer asks for btrfs, which task 11 decided, and the base
has the program, but nothing put it in the environment. The list now carries
it, and a test ties the list to the one filesystem the installer names. With
that as the only change, the same road ran on a Windows 11 in QEMU/KVM, with no
boot order given to the machine:
1. The installer ran in Windows and restarted it.
2. The firmware started the environment.
3. The environment installed alo OS onto the chosen disk and restarted.
4. The firmware started the installed system.
5. The installed system reported that its root is btrfs on the disk with serial
   `ALOTARGET1`.

## The cause, and how it was found

- **Reproduced:** on 2026-09-22 the road on Fedora's firmware, with the new
  start-up entry, stopped with exactly the line task 18 records.
- **Located:** *Creating rootfs* is the context bootc gives to making the root
  filesystem, and it starts `mkfs.<filesystem>` by name. A missing program is a
  bare *No such file or directory*, as `fstrim` was on 2026-09-16
  (`docs/quirks.md`). The built environment's initramfs, listed file by file,
  had `usr/bin/mkfs.ext4`, `mkfs.fat`, `mkfs.vfat` and `mkfs.xfs`, plus `btrfs`
  and `fsck.btrfs` from dracut's own module, and **no `mkfs.btrfs`**. The pinned
  base has `/usr/sbin/mkfs.btrfs` (btrfs-progs 6.19.1-1.fc42), and its kernel has
  btrfs built in (`modules.builtin`), so there was no module to add. Task 11
  measured btrfs by running `bootc install` from the release image, which has
  the program. It did not measure btrfs through the environment.
- **Of task 18's candidates** (the release, the filesystem, the disk bus, the
  firmware), it was the filesystem. The change touched only that, and the disk
  stayed SATA `sdb`.

## The change

- `image/installing/alo-installing.conf` installs `/usr/sbin/mkfs.btrfs`, with
  the reason beside it. The initramfs built from it lists `usr/bin/mkfs.btrfs`.
- `crates/alo-installing/tests/what_the_environment_carries.rs` has two new
  tests:
  - `the_initramfs_carries_the_maker_of_the_one_filesystem` holds the list to
    `mkfs.` followed by `alo_image::THE_ONLY_FILESYSTEM`;
  - `a_list_missing_the_maker_of_the_one_filesystem_is_caught` shows that a
    list without it fails, whether the line is dropped or commented out.
- `crates/alo-installer/tests/the_installer_walked_on_a_real_windows.rs` has a
  new test, `the_whole_road_installs_alo_os_and_the_installed_system_starts`,
  and `walking::machine` can start a machine with **no boot order**, so the
  firmware's variables decide every start.
  - The installed system reports itself through a systemd credential in the
    firmware's tables. Nothing on any disk is changed so that it can speak.
  - A unit test holds the reading of sentences that a kernel message broke in
    two. That happened in the first run.

## The run

`the_whole_road_installs_alo_os_and_the_installed_system_starts`, run by name,
detached, from a script file:

```
probe_should_be_7=7
head=891ec5de2003ef66fc2bce17fdfb0cee43c9fbfb
started=2026-09-22T18:33:40+02:00
cargo_test=0 seconds=1846
finished=2026-09-22T19:04:26+02:00
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 1814.68s
probe_should_be_7=7
```

The run began with one uncommitted file, `docs/quirks.md`. The test code is
the head's.

What the machine said:

- **The firmware, in order:**
  1. `Boot0004 "Windows Boot Manager"`;
  2. `Boot000A "alo OS"` from `HD(4,…,0x7C8F800,…)`, the installer's area;
  3. after the install, `Boot000B "Fedora"` from
     `HD(2,…,0x1000,0x100000)/\EFI\fedora\shimx64.efi`.
- **The environment**, every sentence in order, from *alo OS is being installed
  on this computer* to *alo OS is installed. This computer restarts in a few
  seconds*. It said *Still installing* 8 times, one a minute.
- **The installed system:**

  ```
  /dev/sdb3 btrfs
  sdb3
  └─sdb ALOTARGET1
  os-release: Fedora Linux 42
  Id=alo-boundaryd.service  ActiveState=active    SubState=exited
  Id=alo-agentd.service     ActiveState=inactive  SubState=dead
  ```

  `alo-agentd` is a user service. It was not running because nobody had signed
  in. Task 15 measured it running after a sign-in, and this test does not claim
  it.

The first run showed the same thing and failed on the test's own reading of
the sentences. That reading is fixed in `891ec5de`.

## Found on the way

- **A stalled download never ends.** The second run stopped moving after about
  20 minutes and was still saying *Still installing* 65 minutes in:
  - no disk traffic, no network traffic;
  - three TCP connections established and empty in QEMU's own table.

  The host reached `ghcr.io` in 0.19 s. This is the installer plan's
  **task 20**, and the evidence is in `/root/t10/logs/installed-second/`.
- **After the install, the first entry is called *Fedora*.** bootupd adds
  `Boot000B "Fedora"` and puts it first in `BootOrder`, ahead of Windows. The
  installer's staging entry `alo OS` stays, last, and so does its area.
  `docs/quirks.md` records this for **task 4**, which owns what a person sees
  and when staging is cleaned up.

## Machine and costs

QEMU 10.2.1 q35 under KVM with Fedora's `edk2-ovmf` 20250812-21. Secure Boot is
off in that machine only because the Windows-side installer refuses to run with
it on (ADR 0033 §4). The environment's own Secure Boot road is task 15's, and
it passed there. Three runs of about 30 minutes each. The host's C: had 48–57 GB
free throughout. Every destructive step happened inside a machine the test made.
