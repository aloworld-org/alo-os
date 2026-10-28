# A restored copy built from the broken one

**Concluded:** *the second deliberate break fails the test, so the test catches
it* — and, an hour later, *the clean tree fails five unit tests in
`crate::watchers`*. Both from a build directory fed by `git archive`, which is
how a checkout is copied into WSL here so that a long build cannot see a branch
switch.

**What was true.** The source was restored and the program was not. Each run
removed everything but `target`, unpacked the same tar, and changed one file:

```
fresh → break alo-egress/src/indicator.rs → test   (rebuilt: the break is new)
fresh → break alo-agentd/src/starting.rs  → test   (indicator.rs restored, NOT rebuilt)
fresh                                     → suite  (both restored, neither rebuilt)
```

`git archive` stamps every file with the **commit's** time. A file put back
from the tar is therefore *older* than the object Cargo built from the broken
version, and Cargo decides freshness by comparing times. So it kept the broken
`alo-egress`, the second break was measured on top of the first, and the
"clean" suite ran with the indicator still broken. The fault announced itself
only because five unit tests that had nothing to do with the change failed.

## The mechanism

**Restoring a file to an older time is invisible to a build that asks *is
anything newer than what I built?*** The bytes were right; the question Cargo
asks is not about bytes. A build directory reused across extracts is the normal
way to keep this machine's builds to minutes, so the trap is set every time.

## The cure

Unpack with **`tar -m`**, which stamps each file with the time it was
unpacked, so everything restored is newer than anything built before it. It
costs a rebuild of whatever the tar holds and buys a build of what is on the
disk. A script that unpacks without it is safe only while nobody edits the
copy between runs — which a deliberate-failure check always does. A deliberate break is measured on its own fresh
unpack, never after another one.

*Found 2026-10-28 by the lane writing
`crates/alo-agentd/tests/nothing_leaves_before_the_desktop_is_told.rs`, and
caught by failures elsewhere in the suite rather than by any check aimed at it.*
