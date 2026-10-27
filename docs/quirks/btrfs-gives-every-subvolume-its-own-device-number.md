# On btrfs, the device number `stat` reports is not the filesystem's at all

- **Version:** the image's kernel `6.19.14-101.fc42.x86_64` (btrfs built in,
  `CONFIG_BTRFS_FS=y`); the disk of the fourth walk, image built from `main` at
  `dd42a955` plus #969, and read on the development PC under WSL2
  (`6.18.33.2-microsoft-standard-WSL2`).
- **Whose:** the engine's. btrfs's own `getattr` reports a number of its choosing,
  on purpose, and has for many years.
- **Date:** 2026-10-26.

## Behaviour

On the fourth walk, a person granted `Pictures`, asked alo to zip it, and pressed
*Allow once*. The change came back refused:

```text
/var/home/alo/Pictures could not be archived (Permission denied (os error 13))
```

The grant check in the daemon had passed. The refusal came from the kernel
boundary, which refuses an open unless the file, or a folder above it, is one of
the places the turn was given.

The places are written as (device, inode). The daemon takes the device from
`stat`, through `alo_bounding::place_of`. The kernel program compares it with
`super_block->s_dev`. Measured on the walk's own disk, mounted read-only:

```text
filesystem type: btrfs
stat st_dev of Pictures (major:minor): 0:68
stat st_dev of the mount's root:       0:68
mountinfo (the superblock's s_dev):    0:67
subvolume Pictures lives in:           5
```

The two numbers differ even for the top-level subvolume, and the disk has no
other subvolume. btrfs gives **every subvolume its own anonymous device number**
and reports that one from `stat`, while the superblock keeps a separate number
for the whole filesystem. The packing conversion in `quirks.md` (*The device
number `stat` reports is not the one the kernel keeps*) is correct and does not
help, because these are two different devices, not two packings of one device.

So on every alo OS disk, which is always btrfs (`alo_image::THE_ONLY_FILESYSTEM`),
**the boundary refuses every file in every grant**. It fails safe: nothing outside
a grant was reachable. Nothing caught it before a walk because
`a_turn_granted_a_folder_opens_a_file_inside_it` runs on the runner's own
filesystem, which is not btrfs.

## Why the obvious fix is wrong

Sending the superblock's number instead would make the two sides agree, and it
would widen every grant. btrfs numbers inodes **per subvolume**, so the same
inode number exists in each one, and a read-only snapshot repeats every number
of the subvolume it copies. A boundary keyed on (filesystem, inode) cannot tell
`Pictures` from a folder in another subvolume that happens to share its number,
or from `Pictures` in a snapshot. It looks right today only because this disk
has one subvolume, which the undo bracket is designed to change.

## What the kernel offers instead

btrfs reports `BTRFS_I(inode)->root->anon_dev` as `st_dev`, and this kernel's own
type information describes every step of reaching it (read from the image's
`vmlinux` `.BTF` section):

```text
super_block.s_dev at byte 16, type dev_t
super_block.s_magic at byte 96, type long unsigned int
btrfs_inode.root at byte 0, type ptr
btrfs_inode.vfs_inode at byte 400, type inode
btrfs_root.anon_dev at byte 888, type dev_t
```

## Our response

Fixed on 2026-10-26, as the owner chose: on a filesystem whose `s_magic` is
btrfs's, the kernel program reads the subvolume's own number, through
`btrfs_inode.root->anon_dev`, which is the number `stat` reports. The offsets
come from BTF like every other field. A kernel that does not describe btrfs gets
a boundary that refuses every btrfs file rather than one that guesses.

Teaching the walk this ran into a second limit: the kernel allows a program 512
bytes of stack across its calls, and 6.19.14 refused the boundary at 672 and
then 608. The file hooks now read only the places in a grant and never its
destinations (`alo_bounding_map::Places`), and the program loads.

Three tests in `crates/alo-bounding/tests/the_kernel_refuses.rs` hold both
directions on a real loop-mounted btrfs: a granted file opens; the same inode
number in another subvolume is refused; a snapshot of the granted folder is
refused. **They passed once, by hand,** on 2026-10-26, on the image kernel
6.19.14-101.fc42 booted in qemu with SELinux enforcing. CI does not run them:
the kernel tests need root and a kernel that started the BPF LSM, and no
workflow provides one.

The measurements in the sections above were also made once, by hand.
