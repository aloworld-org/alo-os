# A lock the operating system holds

- Date: 2026-09-09
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: Two verifications, and the hole one of them found
- Status: **the stale-lock recovery published in `305cab1` had a hole, and this
  closes it.**

## The disk reserve does measure C:

Asked directly, because *free space* has two answers on this machine and only one
of them matters:

```
checkout . ->  C:\        474G   54G   /mnt/c
ubuntu   / ->  /dev/sde  1007G  894G   /
```

The reserve runs `df` on the filesystem the **checkout** is on, which is the
Windows drive through `/mnt/c` — not Ubuntu's own disk, whose 894 GB is the
figure that would have made the check meaningless. Nothing to change.

## The stale-lock recovery could have taken a live loop's lock

`305cab1` decided *is the owner alive* from the process id written in the lock,
and that is not sound. **A process id is not an identity**: the number is reused.
A lock naming one can read as alive when its owner is long gone and something
unrelated has the number — annoying — and, if the answer ever came back wrong the
other way, a **live** loop could be taken over. Two supervisors on one working
tree is the thing the lock exists to prevent.

So on Windows the lock is now **held open** for the life of the loop with a share
mode that admits no other writer. The handle is the lock. Nobody consults a
number to decide anything: another loop's open fails while the owner lives, and
the operating system closes the handle when the process ends — asked to stop,
killed, or crashed — so the next loop opens it with no recovery step at all.

There is no stale lock to recover from any more. The pid is still written into
the file, and is still only ever read to tell a person which process to look at.

`when_a_number_is_reused` is the case stated: the lock is held by something
running, and the number in the file is one belonging to nothing. Deciding by pid,
that reads as *nobody holds it* and the live owner is taken over. Deciding by the
handle, it is refused — and the test also shows the file cannot even be rewritten
while a loop holds it.

On a host that is not Windows the pid path remains, and says so.

## Two mistakes of mine, both caught by these tests

**Sharing nothing locked out `status`.** The first version used `share_mode(0)`,
which refuses readers as well — so the supervisor's own status command could not
read the lock and would report *no loop is running* while one was. That is
exactly the confusion the liveness work exists to remove. It is
`FILE_SHARE_READ` now: readers welcome, writers refused.

**The refusal arm never fired.** A sharing violation arrives as OS error 32 with
no `ErrorKind` of its own, so matching `PermissionDenied` silently never matched
and a second loop got the generic *could not be taken* sentence instead of the
one naming the cause. Matched on the number now.

Both were found by tests failing, not by reading the code.

## And the distribution is held open for publishing too

`305cab1` gave `run` a helper that keeps Ubuntu from stopping under it. `publish`
and `verify` did not have one, and they gate for minutes — so the distribution
idled down between commands and the readiness check found `/sys/fs/bpf` gone,
twice in a row, for a reason that had nothing to do with either change.

They hold one now, on the same terms: started with the command, killed when it
ends, mounting nothing and restarting nothing.

## Tested, and where

**The gates run in WSL**, so what they can name as evidence is the tests that
exist on Linux — which exercise the **process-id path this change leaves in
place** on hosts that are not Windows.

The exclusive-handle path and `when_a_number_is_reused` are `cfg(windows)`. Run
by name on Linux they select nothing, and the supervisor **refused them as
evidence** for precisely that reason: a name matching no test reports zero passed
and exits successfully, which is why it reads the count rather than the exit
code. That is the zero-tests safeguard catching this change, and it is worth
recording that it caught mine.

So the Windows path is verified on the Windows host instead —
`cargo test` in `tools/kernel-loop`, **34 passed**, including
`when_a_number_is_reused` — and this report says that rather than implying the
gates covered it.

34 in the supervisor. The six from `305cab1` still hold under the new mechanism —
which is the point of their being about behaviour rather than about a pid — plus
the reused-number case above.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — the backend supervisor's lock is held by the
operating system on Windows, so a live loop cannot be taken over and a killed
one leaves nothing to recover; the disk reserve is confirmed to measure C:.

**docs/autonomy/STATE.md** — `alo-kernel-loop`'s lock is an open handle rather
than a process id, `status` still reads it, and the 12 GiB reserve measures the
Windows drive the checkout is on.
