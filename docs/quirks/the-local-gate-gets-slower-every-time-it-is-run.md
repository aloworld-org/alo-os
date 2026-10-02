# The local gate gets slower every time it is run, and CI never sees it

- **Version:** the nine gates as `tools/kernel-loop/src/gates.rs` runs them, on the
  development PC's lane-3 checkout (WSL2, Ubuntu on `D:\wsl\Ubuntu`).
- **Whose:** ours. Not the platform's, not an engine's.
- **Date:** 2026-10-02.

## Behaviour

`cargo test --workspace` took **10,363 seconds** — two hours fifty-two minutes. Almost all
of it was one test:

```text
test naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so
  has been running for over 60 seconds
```

It **passed**. It was not hung and nothing was broken. After the remedy below, the same
test does not even trip the 60-second warning and the suite takes **2,069 seconds** — a
five-fold difference in the whole workspace's tests, from changing nothing in any crate.

## Cause

`alo-measuring`'s test calls `Holding::of(Path::new("/"))`, which walks the root
filesystem. What it had to walk:

```text
entries on /                993,000
entries under /tmp          718,892   (72% of the filesystem)
direct children of /tmp      90,427
```

Test suites across the workspace create a scratch directory under the temp directory named
after themselves and their process id, and **almost none remove it**. Counted by name
prefix, the directories belonged to `alo-converting`, `alo-approving`, `alo-printing`,
`alo-agentd`, `alo-overlay`, `alo-access`, `alo-applications`, `alo-dividing`,
`alo-keeping`, `alo-image` and `alo-looking`, among others.

So every gate run on a machine makes the next gate run on that machine slower, without
bound. **A fresh CI runner starts with an empty `/tmp`**, which is why `alo/gates-on-a-runner`
has never shown this and why the local gate and the runner disagree about what one suite
costs.

The crate that pays is the one crate in that list which *does* clean up after itself:
zero `alo-measuring-*` directories were present. Its test is the only one whose cost is
proportional to the whole machine, so it pays for everybody else.

## Our response

**Sweep the scratch of processes that have exited, and only those.** By `kill -0` on the
process id in the directory's name — never by age. Several lanes share a development
machine, and deleting a live run's scratch fails somebody else's test for a reason they
would never find. Of 90,427 directories, 89,371 belonged to dead processes and 573 to live
ones, which were left. `/tmp` went to 12,576 entries.

**What is not fixed, and is the real repair:** the suites still leak. A sweep is a remedy
applied to a machine; the cause is a missing cleanup in a dozen crates, each owned by a
different plan.

Two shapes were considered and one of them does not work:

- **Removing your own scratch on exit** — a `Drop` or a trap — is a promise made by a
  process that may not live to keep it. A killed run keeps nothing.
- **Clearing your own path on entry** does not help either **when the name carries the
  process id**, which is the convention here: the next run has a different pid, so
  `remove_dir_all` is handed a path that has never existed. `alo-keyring-fixture` does
  exactly this and it looks like a cleanup; what actually keeps that crate tidy is its
  `Drop`.
- **What works is sweeping the family on entry:** every sibling matching your own name
  pattern whose process is gone. Then a killed run is cleaned by the next run of any
  process, which is the property the other two only appear to have.

## Why this is written where a stranger will look

*The suite is slow* and *the suite is slow on a machine holding 700,000 stale temporary
files* lead to completely different work. The first invites somebody to make
`Holding::of` faster, or to mark the test `#[ignore]`, or to conclude that walking a
filesystem in a test was a mistake. None of those is the repair, and the test is correct:
measuring the machine is what that crate is for, and `docs/features.md:289` promises *"what
is filling my disk?"* as something the agent can actually answer.

**And one theory about this was measured and discarded**, recorded in
`the-root-filesystem-is-mounted-twice-on-wsl-with-the-same-inode.md`: `/mnt/wslg/distro`
shares `/`'s device id *and* inode, which looks exactly like a walk counting the machine
twice. It is not — a device-id walk stops at `/mnt/wslg`, which is tmpfs. The filesystem
genuinely holds a million entries. That entry exists so the next reader does not spend an
afternoon on a double-count that is not happening.
