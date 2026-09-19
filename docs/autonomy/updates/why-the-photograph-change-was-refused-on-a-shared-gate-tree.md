# Why the photograph change was refused, and what it was refused for

**Date:** 2026-09-19
**Workstream:** documents and paper (`docs/autonomy/v0-5-documents-and-paper-plan.md`)
**Task:** 6. A `.pages`, a `.heic` and a `.dwg` — recognised, and converted or
explained — second and last attempt.
**Contributor:** the second worker on task 6, on this development PC.
**Status:** ready for integration. Nothing in the code under review was changed
by this attempt; what changed is that the refusal has a cause, and the cause is
not in the change.

This is a follow-up to
`docs/autonomy/updates/a-photograph-is-not-a-film-and-three-formats-wait-on-a-real-file.md`,
which is the first worker's report and stands as written. That report describes
the work; this one describes why the gates refused it and what was done about
that.

## What the gates said

The supervisor ran the nine gates on task 6's tree twice and refused it both
times, on one test, in a crate this task does not touch:

```
thread 'nothing_is_enrolled_while_its_decision_is_proposed' panicked at
crates/alo-encrypting/tests/the_enrolment_waits_on_its_decision.rs:49:5:
ADR 0054 is no longer proposed (**Status:** **accepted, 2026-09-19, by the owner
— option C falling back to B,). Build what it decided — enrolment during
install, and the recovery — and replace this test with the tests of that
```

That test is the encrypting workstream's tripwire on
[ADR 0054](../../decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md):
while the decision says *proposed*, nothing enrols anything, and the day it says
*accepted* the test fails on purpose, so that what it decided gets built.

## Why that could not have been this change

Task 6's diff is `crates/alo-opening` and four documents. It does not touch
`alo-encrypting`, does not touch ADR 0054, and `alo-encrypting` does not depend
on `alo-opening`. And in this checkout ADR 0054 still says what it has said
since it was written:

```
$ grep -m1 '^\*\*Status:\*\*' docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md
**Status:** proposed, 2026-09-17. Written by task 5 of
```

So the tripwire cannot fire here, and does not: `cargo test -p alo-encrypting`
passes in this tree, on the exact target that was reported failing (43 tests,
five targets, all green — see *What was run*).

## Where the accepted ADR 0054 came from

It exists in exactly one place in this repository, and it is not on this branch:

```
$ for r in refs/heads/* refs/remotes/*; do ... grep -m1 '^\*\*Status:\*\*' ...
refs/heads/main                                        proposed, 2026-09-17
refs/remotes/origin/main                               proposed, 2026-09-17
refs/remotes/origin/task/dev-pc/the-disk-is-sealed-and-opened-with-a-pin
                                    **accepted, 2026-09-19, by the owner — …
```

That branch is one docs-only commit (`d8c9745`) on top of current `main`, and it
was made in **another checkout on this same machine**, `C:\dev\alo-os-shell`,
which is the only checkout that has the branch:

```
$ git -C /c/dev/alo-os-shell branch -a | grep sealed
  task/dev-pc/the-disk-is-sealed-and-opened-with-a-pin
  remotes/origin/task/dev-pc/the-disk-is-sealed-and-opened-with-a-pin
```

## What actually happened: two lanes, one gate tree

`tools/kernel-loop/src/gates.rs` does not run the gates against the Windows
checkout. It rsyncs the tree to the Linux side first — `the_copy_for` maps
`$HOME/alo-builds/this-machine` to `$HOME/alo-trees/this-machine` — because
reading `/mnt/c` over 9p costs the gates about fourteen minutes, and its own
rustdoc says why: *One copy for every checkout on the machine, overwritten by
whichever lane has the gate turn, and read only while it holds the turn.*

**Nothing enforces the second half of that sentence across checkouts.**
`tools/kernel-loop/src/lock.rs` takes its lock at `<checkout>/.kernel-loop/lock`
— one supervisor per checkout, which is what its own documentation claims and
all it claims. The build directory and the source copy it reads are machine-wide;
the lock that is supposed to serialise access to them is not. So while this
task's gates were reading `/root/alo-trees/this-machine`, the `alo-os-shell`
lane's gates rsynced *their* tree over it — accepted ADR 0054 and all — and the
workspace suite read a tree that was partly this task's and partly theirs.

The state of that directory now is the same mechanism, still running: it holds
neither this task's work nor the accepted ADR, because a third run has
overwritten it since.

```
$ grep -c iso_media /root/alo-trees/this-machine/crates/alo-opening/src/lib.rs
0
$ ls /root/alo-trees/this-machine/crates/alo-opening/tests/
a_file_that_does_not_hold_together.rs
a_file_this_machine_cannot_open_is_explained.rs
converting_waits_on_its_decision.rs
deciding_never_leaves_the_machine.rs
deciding_reads_only_what_it_needs.rs
making
what_this_machine_can_do_with_a_file.rs
$ grep -m1 '^\*\*Status:\*\*' /root/alo-trees/this-machine/docs/decisions/\
0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md
**Status:** proposed, 2026-09-17. Written by task 5 of
```

*Run twice and refused both times, so this is the work rather than the machine*
is the supervisor's normal and correct reasoning, and here it drew the wrong
conclusion for an ordinary reason: the other lane was gating across both runs, so
the contamination was as reproducible as a real fault. This is the same class of
failure `SHARED_MAIN.md` already records twice — the 2026-09-17 `SleptThrough`
refusal in a tree that had no such case, and the 2026-09-18 CRLF refusal for a
fault that did not exist in the commit. Both were the gates testing something
other than the change. This is the third, and the first one where the wrong bytes
came from another checkout rather than from a timestamp or a line ending.

## What was changed, and what deliberately was not

**Nothing in `crates/alo-encrypting` was touched.** The instruction for this
attempt was to fix what the gates named and to prefer the smallest change, and
the smallest change that makes the named test pass is none: it already passes
here. Three things were considered and refused:

- **Changing or deleting the tripwire.** It is doing exactly its job. ADR 0054
  is now accepted, which obliges the encrypting workstream to build enrolment
  during install and the recovery, and to replace that test with the tests of
  that — tasks 6 and 7 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`.
  A worker on a documents task editing another workstream's tripwire to get a
  green suite would be weakening a gate to pass it, which `CLAUDE.md` forbids in
  the same words.
- **Building what ADR 0054 decided.** That is a workstream, not a fix, and
  batching it onto this branch is what `SHARED_MAIN.md` forbids in *Do not batch
  unrelated tasks into one branch*.
- **Fixing the lock.** This is the real defect and it is named below, but it is
  `tools/kernel-loop`'s own work: putting it in this commit would give this
  branch a second reason to change and would mix a supervisor change into a
  documents task's diff. It is one task for the loop, not a line in this one.

## Owed work this found, for whoever takes it

**The gate's shared tree and build directory need a machine-wide lock.**
`lock.rs` serialises supervisors per checkout; `gates.rs` writes and reads two
paths that every checkout on the machine shares. The fix is a lock beside those
paths rather than beside the checkout — `$HOME/alo-trees/.gate-turn`, taken with
`create_new` and held for the whole gate run, with the waiting lane told who has
it — and a test that a second gate run refuses to start while the first holds it.
`SHARED_MAIN.md`'s *Check running Cargo processes and the machine-wide gate lock
before any build/test* already describes the rule; there is no such lock in the
code, so the rule is prose and this is what prose costs. Until it exists, two
lanes gating at once on this machine can refuse each other's work for faults
neither one contains, and the refusal looks exactly like a real one.

## Why task 6 is still marked blocked rather than done

The first worker marked task 6 **blocked** on ADR 0057 rather than **Done**, and
that is kept. Task 6's acceptance is that all three formats are recognised and
measured against a real file of each; that has not happened and cannot until the
owner answers ADR 0057 and the files arrive. Marking it done would be claiming
unfinished work finished, and it would also fail a test written in this very
change:
`recognising_three_more_formats_waits_on_its_decision.rs`'s
`the_plan_points_at_the_decision_and_steps_over_the_task_while_it_waits`
refuses a task 6 marked `**Done,` while the decision it waits on still says
*proposed*. The loop does not re-offer a blocked task, and the plan now names a
task 7 after it, which is what the loop takes next.

## What was run

On this development PC, in WSL Ubuntu, against `/mnt/c/dev/alo-os-b`, building
into the machine's one build directory `/root/alo-builds/this-machine`. No other
Cargo process was running (`pgrep -a cargo` empty before starting), and the
Linux filesystem had 822 GiB free.

| Gate | Result |
|---|---|
| `cargo fmt --all`, then `cargo fmt --all -- --check` | exit 0, nothing to change |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, zero warnings, 1 m 41 s |
| `cargo test -p alo-opening` | 89 passed, 0 failed, 10 targets |
| `cargo test -p alo-encrypting` | 43 passed, 0 failed, 5 targets — **the crate the refusal named** |
| `cargo test -p alo-saying` | 68 passed, 0 failed |
| `cargo test -p alo-citing` | 31 passed, 0 failed (the citation check, which a new decision reaches) |
| `cargo test -p alo-converting` | 83 passed, 0 failed |
| `cargo test -p alo-printing` | 62 passed, 0 failed, 1 ignored |
| `cargo test -p alo-portals` | 112 passed, 0 failed |
| `cargo test -p alo-sound` | 33 passed, 0 failed |
| `cargo test -p alo-applications` | 86 passed, 0 failed |

`alo-opening` is what the change touches; the other crates are every crate that
depends on it (`alo-applications`, `alo-converting`, `alo-portals`,
`alo-printing`, `alo-saying`, `alo-sound`), the crate that reads the decisions,
and the crate the refusal named. The whole workspace suite was not run here, by
instruction: the supervisor runs it, and it is the better part of an hour.

Each of the fourteen tests named in `.kernel-loop/handoff.toml`'s `evidence`
block was then run again **on its own**, with `--exact`, and each reported
`1 passed`. The five unit tests of `src/iso_media.rs` are new in this handoff's
evidence and were not in the first worker's; they are the rule itself, tested
where it is written.

**The citation check caught this report before anybody read it.** The shell
transcripts above first shortened ADR 0054's filename with an ellipsis through
the middle of it, to keep the line inside eighty columns, and
`crates/alo-citing/tests/every_decision_this_repository_points_at.rs` refused
both lines: a pointer into `docs/decisions/` that does not land, which is exactly
the shape of the renamed ADR that broke `main` for five machines on 2026-09-17.
The names are written out in full now. It is worth recording because it is the
second time in this task that a gate was right and the tree was wrong about
something nobody would have found by reading.

No hardware acceptance is claimed. Nothing in this change reaches a device, a
network or a disk, and nothing was uploaded.

## Proposed shared-document updates

Not made here: `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` and
`docs/autonomy/STATE.md` belong to the integration owner.

**`CHANGELOG.md`** — the first worker's proposal stands, and is the only
user-visible change in this task:

> A photograph from a telephone is no longer described as a video. Files that
> share a container with MP4 — a HEIC photo, an AVIF picture, a camera's raw
> capture — were being reported as *an MP4 video*, which sent people looking for
> something to play them with. alo OS now reads what the file says it is rather
> than only the container it sits in, and where it does not know, it says so.

**`docs/autonomy/QUEUE.md` and `STATE.md`** — task 6 of the documents and paper
plan is blocked on ADR 0057 awaiting the owner's answer, and task 7 is ready.
Two further items this attempt found, for the queue:

1. **A machine-wide gate lock for the shared tree and build directory**
   (`tools/kernel-loop`), described under *Owed work* above. It has refused one
   finished task already.
2. **ADR 0054 is accepted, so its build is owed** — enrolment during install and
   the recovery, tasks 6 and 7 of the broker-and-the-disk plan, which include
   replacing `crates/alo-encrypting/tests/the_enrolment_waits_on_its_decision.rs`
   with the tests of what was decided. Until that lands, every full workspace run
   on a tree carrying `task/dev-pc/the-disk-is-sealed-and-opened-with-a-pin` will
   fail that test — correctly, because that is what the tripwire is for.
