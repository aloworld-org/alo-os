# WSL mounts the root filesystem a second time, with the same device id *and* the same inode

- **Version:** WSL2, kernel `6.18.33.2-microsoft-standard-WSL2`, Ubuntu distro on
  `D:\wsl\Ubuntu`, measured on the development PC's lane-3 checkout.
- **Whose:** the platform's. Nothing in alo OS causes it and nothing in alo OS can
  change it.
- **Date:** 2026-10-02.

## Behaviour

`/mnt/wslg/distro` is the root filesystem again. Not a copy and not a similar tree —
the same directory:

```text
st_dev of /                  2096  (830 hex)
st_dev of /mnt/wslg/distro   2096  (830 hex)
inode of /                   2
inode of /mnt/wslg/distro    2
```

`findmnt` shows why: both are `ext4 /dev/sdd`.

```text
/                     ext4    /dev/sdd
/mnt/wslg/distro      ext4    /dev/sdd
/mnt/wsl              tmpfs   none
/mnt/wslg             tmpfs   none
/mnt/c                9p      C:\
```

## What this means for a walk, and the answer is *not* the obvious one

A walk that stops where `st_dev` changes **cannot** tell `/mnt/wslg/distro` from an
ordinary directory. The device id is identical, so the usual test for *am I crossing
into another filesystem* answers no.

The obvious conclusion is that such a walk re-enters the root and counts the machine
twice. **That conclusion is wrong, and it was measured rather than reasoned:**

```text
find / -xdev                                      993,006 entries
find / -xdev -path /mnt/wslg -prune -o -print     993,007 entries
find /mnt/wslg/distro -xdev                       993,046 entries
```

Pruning the duplicate changes the total by one entry — the pruned directory itself. So
the walk never reached it.

The reason is the mount *above* it. `/mnt/wslg` is **tmpfs**, a genuinely different
device, so a device-id walk stops there and never descends to `distro` underneath. The
duplicate is real, it is reachable by a path, and it is screened by an unrelated mount
that happens to sit in front of it.

**So the accommodation is: none, and that is the useful part.**
`alo_measuring::Holding::of` is correct here by accident of layout rather than by
design. Nothing needs changing today, and that is worth knowing precisely because the
next person will form the double-counting theory from the `stat` output above — it is
the natural reading — and spend an afternoon on a bug that is not happening.

**What would break it:** anything that walks from a path *inside* `/mnt/wslg`, or any
future WSL layout where the second root is not behind a tmpfs. A walk that must be
correct regardless should compare `(st_dev, st_ino)` against the directories already
visited rather than `st_dev` against the parent — the inode above shows device id alone
cannot answer the question. This is recorded as the remedy if it is ever needed, not as
a change made on a hypothesis.

## Related, and measured on the same machine

`Holding::of("/")` took **2.5 hours** here and passed. `find / -xdev` alone takes
**314 seconds**. The cause is not this quirk: `/tmp` held **718,892 entries across
90,427 directories**, 72% of the filesystem, left by test suites across the workspace
that create a scratch directory named after themselves and their process id and do not
remove it. A fresh CI runner starts with an empty `/tmp`, which is why the same suite
takes seconds there and why the local gate and the runner disagree about the cost of
one test.

That is our own test hygiene rather than a platform quirk, and it is recorded where the
lane that found it works. It appears here only so that a reader who arrives at the slow
walk is not left with this quirk as the explanation — **it is not the explanation.**
