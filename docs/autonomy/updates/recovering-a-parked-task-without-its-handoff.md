# Recovering a parked task whose worker never wrote a handoff

**Date:** 2026-09-12. **Workstream:** the build loop (`tools/kernel-loop`).
**Contributor:** Claude, in `C:\dev\alo-os-claude`.
**Plan task:** v0.01 delivery plan, task 35, *Recovering a parked task whose
worker never wrote a handoff*.

## What changed

`alo-kernel-loop recover <branch>` can now recover a parked branch that carries
no handoff.

Task 29 made recovery a command, and the command reads the parked branch's own
`.kernel-loop/handoff.toml` to know which files to bring back. On 2026-09-11 it
was used on real parked work three times and refused twice, both times for the
same honest reason: the branch had no handoff. The road there is the ordinary
one. The gates refuse a first worker; the repair path moves that worker's
handoff to `.kernel-loop/refused/<moment>.toml` so a second worker can write
its own; the second worker is killed at the ninety-minute deadline before it
does; and the supervisor parks what is in the tree. The branch then holds
every line of both workers' work and nothing that names it, and the recovery
was done by hand.

Both halves of what a handoff would have said are knowable from what is there:

- **The file list is the branch's own commit** — its diff against the `main`
  it was parked from, read with the same `git diff --name-status --no-renames`
  the existing path already uses. Never `git status`, which describes the tree
  the command is being run in rather than the branch, and never a guess.
- **The handoff is the newest refused one for the same task.** The task's
  number is in the branch's name, because parking writes it there; the plan
  turns the number into the name a handoff carries; and `.kernel-loop/refused/`
  is read newest-first, by the moment in each file's name, for a handoff naming
  that task.

Both are **reported as reconstructed** rather than restored, on the terminal
and in the journal, with the refused file the handoff was taken from. And
because a handoff the first worker wrote does not describe what the second
worker left, the recovery lists the files that handoff names which the branch
never changed and the files the branch changed which the handoff does not
name, and is not ready to gate — exits non-zero, as a conflict does — until a
person has made the handoff say what the branch says. Every file on the
branch's commit comes back, whole or merged by the task's own diff exactly as
before; nothing is left alone, because there is no handoff to have left it
unclaimed.

The existing behaviour for a branch that has a handoff is unchanged, and a test
now says so: a refused entry that happens to match is never consulted when the
branch can speak for itself.

Source:

- `tools/kernel-loop/src/recovering.rs` — the decision. `recover` asks the
  branch whether it carries a handoff before reading one; the reconstructed
  path; `Handoff`, the account of where the waiting handoff came from, on
  `Recovered`; `is_ready_to_gate` now also requires that account to describe
  the branch; the number read out of a parked name; the module documentation;
  and the tests.
- `tools/kernel-loop/src/handoff.rs` — `Handed::newest_refused_for`, which
  reads the refused directory newest-first by the name `put_aside` gave each
  file, and `Refused`, the entry found: its path, its text byte for byte, and
  its reading. `where_refused_ones_are` names the directory so a refusal can.
- `tools/kernel-loop/src/plan.rs` — `plan::numbered`, the one place a task
  number becomes a task.
- `tools/kernel-loop/src/repository.rs` — `has_a_file`, one `git cat-file -e`,
  because `one_file_as` answers *not there* and *git would not run* with the
  same kind of sentence and this path has to tell them apart. Every git this
  loop runs still lives in this one file.
- `tools/kernel-loop/src/main.rs` — the reconstructed account on the terminal
  and in the journal, the two reasons a recovery can be not ready to gate said
  separately, and a paragraph in the crate documentation.
- `docs/autonomy/v0-01-delivery-plan.md` — task 35 marked done, and task 36
  written after it.

### The refusals

Each is a sentence, and nothing is written to the tree before any of them. The
five from task 29 are unchanged; these two are new, and each says what was
looked for and where, because the next thing a person does is look there:

| Asked | Answered |
| --- | --- |
| a branch with no handoff, numbered for a task the plan does not know | *the plan `…` names no task N* — nothing restored; name the plan with `ALO_LOOP_PLAN` if the branch was parked from another |
| a branch with no handoff and no refused handoff for its task | *nothing in `.kernel-loop/refused` is a refused handoff for task N, `name`* — nothing restored; the diff to read is named |

### A user-readable change description

The build supervisor can now pick up a parked task even when the worker that
was stopped never wrote the file that names its work. It reads the list of
files off the branch itself, takes the most recent handoff the gates had
refused for that task, and says plainly that the handoff was reconstructed
rather than restored — listing anything the reconstructed handoff gets wrong
about the branch so a person can correct it before publishing. A branch with
nothing to reconstruct from is refused in words that say where it looked.

## Decisions taken

- **The plan maps number to name, and the plan is the one the run is
  configured with.** A parked branch carries its task's number and a handoff
  its name; nothing else in the repository knows both. `plan::numbered` reads
  the plan `ALO_LOOP_PLAN` names, as `run`, `verify` and `publish` do, so a
  branch parked from another workstream's plan is recovered by naming that
  plan — and the refusal says so.
- **Newest by the file's name, not its modification time.** `put_aside` names
  each entry by the second it was refused. A checkout copied or restored gets
  new times on every file and keeps every name, so the name is the order that
  means something. A file in the directory whose name is not a number sorts as
  older than every one that is.
- **An unreadable entry is passed over, not reported.** Everything `put_aside`
  moves into that directory was read successfully first — the loop gated it —
  so an unreadable file there was not put there by this program.
- **The reconstructed handoff is written back verbatim, and its differences
  from the branch are reported rather than corrected.** Rewriting the file list
  to match the branch would be mechanical, but the handoff is a worker's
  statement and its `evidence` lines name tests in files it lists; a person
  reconciling the two sees the whole thing. The recovery is not ready to gate
  until they have, which is what the exit code says.
- **A reconstructed handoff that does describe the branch is ready to gate.**
  The common case — the second worker finished the same files and died writing
  the handoff — then needs nothing from anybody.
- **The branch's own handoff always wins.** A refused entry for the same task
  is never consulted when the branch carries a handoff, so task 29's behaviour
  is unchanged by this existing, and a test holds that.
- **Nothing is left alone in the reconstructed case.** With the branch's commit
  as the list, every path on it comes back. That is the acceptance criterion,
  and it is also the only honest reading: there is no handoff to have declined
  to claim a file.
- **`has_a_file` is a new git command in `repository.rs`** rather than a
  reading of `one_file_as`'s error, because *not there* and *git would not
  run* must lead to different places and an error string is not a fact.

### Acceptance, criterion by criterion

| Criterion | Test |
| --- | --- |
| recovered from what it has: the file list off the branch's commit, the handoff from the newest refused entry for the task, both reported as reconstructed | `recovering::tests::a_branch_with_no_handoff_is_recovered_from_its_commit_and_the_refused_handoff` — made by the real road: `put_aside`, a second worker writing more and no handoff, the real `parked` |
| the newest refused entry whose `task` matches is the one taken | `recovering::tests::the_newest_refused_handoff_for_the_task_is_the_one_taken`; `handoff::tests::the_newest_refused_handoff_for_a_task_is_found_by_its_moment` |
| a branch with neither is refused in words that say what was looked for | `recovering::tests::a_branch_with_neither_a_handoff_nor_a_refused_one_is_refused_in_words`; `recovering::tests::a_branch_numbered_for_a_task_the_plan_does_not_know_is_refused` |
| the existing behaviour for a branch that has a handoff is unchanged | `recovering::tests::a_handoff_on_the_branch_is_preferred_to_a_refused_one`, and the ten task-29 tests, unchanged but for one asserting the handoff is reported as on the branch |
| the file list never comes from `git status` | `recovering::tests::a_reconstruction_reads_its_file_list_off_the_branch_and_not_the_tree` |
| a reconstructed handoff that describes the branch needs nothing further | `recovering::tests::a_reconstructed_handoff_that_describes_the_branch_is_ready_to_gate` |
| `git checkout -- .` and `git restore -- .` remain refused | `recovering::tests::recovery_never_touches_more_than_one_named_path`, unchanged and still reading the source |

## Verification

Windows 11, PowerShell, from `tools/kernel-loop` (its own workspace):

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

All three ran and passed on 2026-09-12: fmt clean, clippy with zero warnings,
and `92 passed; 0 failed` (81 before this change, eleven added). The
recovering tests build real git repositories, park real branches with the
real `parked`, and read the tree back; they are not mocked.

Not run here, deliberately: the product workspace's full suite. This change
touches only `tools/kernel-loop`, which is its own workspace and not part of
what ships; the supervisor runs the full suite after this regardless.

Not exercised: a recovery on the real parked branches from 2026-09-11. They
were recovered by hand that day and the branches are no longer in this
checkout, so the road was reproduced in a fixture instead, step for step.

## Limitations

- The reconstruction depends on the refused directory of the checkout that
  parked the branch. A parked branch copied to another checkout still cannot
  be reconstructed, because `.kernel-loop` is ignored and the branch carries
  nothing that names it. Task 36, written in the plan, is the upstream fix:
  parking carries the refused handoff on the branch under a distinct name.
- The reconstructed handoff is the first worker's and may not describe the
  branch. The differences are reported and the recovery refuses to call itself
  ready; a person still edits the file.

## Proposed changes to the shared documents

- **CHANGELOG.md:** *The build supervisor's `recover <branch>` now recovers a
  parked branch that carries no handoff, from the branch's own commit and the
  newest handoff the gates refused for that task, and says that it did.*
- **QUEUE.md / STATE.md:** task 35 of the v0.01 delivery plan is done; task 36
  is written and ready.
- **ROADMAP.md:** no change.

## Status

Ready for integration.
