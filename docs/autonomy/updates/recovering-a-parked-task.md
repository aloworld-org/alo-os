# Recovering a parked task, as a command rather than as a memory

**Date:** 2026-09-11. **Workstream:** the build loop (`tools/kernel-loop`).
**Contributor:** Claude, in `C:\dev\alo-os-claude`.
**Plan task:** v0.01 delivery plan, *Recovering a parked task, as a command
rather than as a memory* — numbered 28 when it was handed over, 29 in the plan
as published (see *The plan was failing its own test* below).

## What changed

`alo-kernel-loop recover <branch>` puts a parked task's work back in the working
tree, on top of today's `main`, and stops there.

Parking already worked: a task whose gates refuse it is committed to a local
branch of its own, nothing is discarded, and the run carries on. Picking one
back up was a thing somebody had to remember, and on 2026-09-11 what was
remembered was

```
git restore --source=parked/task-21-... -- .
```

which is not *bring that task's work back*. It reverts the **whole working
tree** to that branch, and a parked branch is a photograph of `main` as it was
before whatever has been published since — so two published tasks were silently
undone the first time and two more would have been the second. Both times the
supervisor's *these are changed and no task named them* check stopped it before
anything reached `main`. Neither time did anything in this repository stop it
being done again ten minutes later.

The right recovery is knowable from what the parked branch already carries,
because parking force-adds the task's own handoff onto it and a handoff names
every file the task touched. So:

- a file the task named that **nobody has published over since** comes back
  whole, as the branch has it — including one the task *deleted*, which comes
  back deleted;
- a file **both** touched has the task's own **diff** applied over what is in
  the tree, never the task's version of the file;
- a conflict between them is **reported and left**, markers and all, and the
  command exits non-zero saying which files and that no side was chosen;
- the handoff goes back to `.kernel-loop/handoff.toml`, so the recovered task is
  ready for `verify` and `publish`;
- everything else in the tree is untouched.

Source:

- `tools/kernel-loop/src/recovering.rs` — new, and the whole of the decision:
  which branch names are parked ones, what the branch changed, which files come
  back whole and which are merged, and every refusal.
- `tools/kernel-loop/src/repository.rs` — the seven git commands recovery needs.
  They live here because this file's stated responsibility is *every `git` this
  loop runs, in one file, so that what it can do to a checkout is a list
  somebody can read*, and a second place to run git would be the end of that
  guarantee. Also `git_bytes`, because the existing helper trims and a diff that
  has lost its trailing newline is one `git apply` calls corrupt.
- `tools/kernel-loop/src/handoff.rs` — `Handed::read` is now public so a handoff
  can be read out of a branch rather than only off the disk, and
  `Handed::where_one_waits` names the one place a handoff waits.
- `tools/kernel-loop/src/main.rs` — the `recover <branch>` subcommand, its usage
  line, its output and a paragraph in the crate documentation.

### The refusals

Each is a sentence, and nothing is written to the tree before any of them:

| Asked | Answered |
| --- | --- |
| a name that is not `parked/task-<number>-<moment>` | *is not a parked branch* |
| a parked name nobody has here | *there is no branch … Parked branches are local* |
| a handoff already waiting | *a handoff is already waiting, for `…`* — nothing restored |
| a branch that changes nothing against its base | *there is no work on it to restore* |
| a handoff naming files its own branch never changed | the files are named, and nothing is restored |

### A user-readable change description

The build supervisor can now pick a parked task back up with one command:
`alo-kernel-loop recover <branch>`. It brings back exactly the files that task
said it had touched — whole where nobody has changed them since, merged where
somebody has — and leaves anything published in the meantime alone. Where the
two genuinely disagree it says so and leaves both sides in the file for a person
to settle. It never publishes and never deletes the branch, so a recovery that
goes wrong can simply be done again.

## Decisions taken

Nobody was waiting to be asked, so these were decided here and are written down
rather than left to be inferred.

- **The branch's own handoff is the list of what to restore**, not a diff of the
  branch against its base. The diff is what the branch *contains*, which
  includes whatever happened to be lying in the tree when the task was parked;
  the handoff is what the task *claimed*, which is the thing the whole loop
  already gates on. A path the branch changed and the handoff never named is
  reported as left alone rather than restored — restoring it would be this
  command making the original mistake in smaller print.
- **The base is `git merge-base HEAD <branch>`**, not the branch's parent
  commit. It is right for a park that made no commit, for a branch with more
  than one, and for a `main` that has moved a long way since.
- **Whole-restore versus diff-apply is decided per file**, by asking git whether
  anything has happened to that path on `HEAD` since the base. A file nobody
  touched is restored whole because that is exact; a file somebody touched gets
  the task's diff because its version is stale by definition.
- **`git apply --3way`** does the merging, so a conflict lands as ordinary
  conflict markers in the working tree — the form every person and every editor
  in this workflow already knows. Whether it conflicted is asked of
  `git ls-files --unmerged` rather than read out of the error text, because
  `git apply` exits the same way for *this conflicts* and for *this is not a
  patch*, and only one of those leaves work in the tree.
- **A conflicted recovery exits non-zero.** The work is in the tree either way,
  but a zero exit would read as *ready to publish* and it is not.
- **A handoff already waiting is a refusal, not an overwrite.** Recovery writes
  that file; writing over somebody's would have the loop gate one task's tree
  under another task's name.
- **The branch-name check is the shape, not the prefix.** A branch somebody
  called `parked/task-mine` by hand is refused. This command reads a handoff out
  of a branch and writes files from it; the one thing it has to be sure of is
  that the branch is one the supervisor made.
- **Recovery is a separate module from parking.** They are one story and two
  responsibilities: `repository::parked` decides what a branch is made of, and
  `recovering` decides what comes back out. The file that already had a second
  reason to change is the one this repository's fourth law is about.

## Acceptance criteria, and the test behind each

Seven of the ten tests build a **real git repository** in a temporary directory,
write a task's work into it, park it with `repository::parked` — the same
function the supervisor uses — commit something else on `main` on top, and then
read the tree back. A fake `git` would have proved only that the fake agreed
with whoever wrote it, which is exactly the gap the defect lived in.

| Acceptance criterion | Test (`recovering::tests::…`) |
| --- | --- |
| one subcommand leaves the tree holding that task's work on top of today's `main`, with its handoff back in place | `a_parked_task_comes_back_on_top_of_todays_main_with_its_handoff` |
| a file the task named and nobody else touched comes back whole | `a_file_nobody_else_touched_comes_back_whole` |
| a file both touched is merged by applying the task's own diff | `a_file_both_touched_is_merged_by_the_tasks_own_diff` |
| a conflict is reported and left rather than resolved by preference | `a_conflict_is_reported_and_left_unresolved` |
| a branch whose handoff names files the task did not change is refused in words | `a_handoff_naming_files_the_task_did_not_change_is_refused` |
| a branch that is not a parked branch at all is refused in words | `a_branch_that_was_never_parked_is_refused` |
| nothing may reset, clean or check out the whole tree | `recovery_never_touches_more_than_one_named_path` |

Three more refusals are tested beside them and are not offered as evidence
because they answer no criterion of their own:
`a_parked_branch_that_is_not_here_is_refused`,
`a_recovery_will_not_write_over_a_handoff_that_is_waiting`, and
`only_the_names_parking_writes_are_parked_names`.

The first test is where the defect itself is pinned. A task is parked, another
task publishes `src/somebody-else.rs` on top, the recovery runs, and the
assertion is that **that file is still there afterwards** — which the old
one-line `restore -- .` fails and every other assertion in the test passes.
`a_file_both_touched_is_merged_by_the_tasks_own_diff` is the same defect in
small print: restoring the branch's version of a shared file would satisfy
everything except the published line, so the published line is asserted.

The constraint is held on the source as well as by behaviour, because a
whole-tree command added later for convenience would pass every behavioural test
above on the day it was written and revert somebody's work on the day it was
used. `recovery_never_touches_more_than_one_named_path` reads
`recovering.rs` for `"checkout"`, `"reset"`, `"clean"`, `"restore"` and
`Command::new` — recovery runs no git of its own — and reads the one restore in
`repository.rs` to check it still names exactly one path and takes no pathspec.

## The plan was failing its own test

`plan::tests::every_plan_this_repository_drives_holds_only_tasks` holds every
plan this repository drives to being numbered from one, in order. The v0.01
delivery plan had **two sections numbered 27** — *The sign-in surface's half of
the door* and *What the greeter does* — so that test had been red since the
second of them was written, and with it the whole of
`cargo test -p alo-kernel-loop`, which is a gate this task had to pass.

Renumbered here in one line each: the greeter is 28, this task is 29. No title,
dependency, status or body moved, and the supervisor selects tasks by title
rather than by number, so nothing already handed over means anything different.

While this was being written, `e2ea3ef` published *The gates build where there
is room, not where there is none*, numbered 29 on the assumption that this task
was 28. It is 30, by the same one-line change. Because the plan therefore names
a task after this one, this task writes none.

One idea that came out of the work is recorded here rather than inserted into
the plan, since the plan's next task is the other lane's to keep: **there is no
way to see what is parked.** Recovery takes a branch name and `git branch --list
'parked/*'` gives a number and a second count — not which task it was, not what
it claimed, and not whether it has since been redone and published, which is the
one case where recovering would do a finished task's work twice. The published
plan knows which tasks are done, so a listing could say so. Offered for the
queue, not taken.

## Verification

Run from `C:\dev\alo-os-claude\tools\kernel-loop` on Windows 11, 2026-09-11.

| Command | Result |
| --- | --- |
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test -p alo-kernel-loop` | 62 passed, 0 failed |
| each of the seven evidence tests, run alone with `--exact --include-ignored` | `1 passed` each |

Also exercised against this checkout itself: `alo-kernel-loop recover main` and
`alo-kernel-loop recover parked/task-99-1789000000` both refuse in the words
above and touch nothing.

**Not run here, deliberately:** `cargo test --workspace` for the product
workspace. No product crate was touched — every file in this change is in
`tools/kernel-loop`, which is its own workspace, or is documentation. The
supervisor runs the whole suite after this regardless.

**Not run at all:** no recovery of the parked branch that exists in this
checkout, `parked/task-27-1789139845`. This working tree holds this task's own
work, and recovering into an occupied tree is one of the refusals above. That
branch is still there and is now recoverable by the command this task adds.

## Limitations

- Recovery does not consult the plan. A parked branch whose task has since been
  redone and published will come back like any other, and nothing warns that the
  work is already on `main`. That is the listing idea above, and it is a task
  rather than a line.
- A file that both sides changed is merged with git's three-way apply, which
  resolves by *position*. Two edits that do not overlap textually but contradict
  each other in meaning will merge cleanly and be wrong, exactly as they would
  in any rebase. The gates are what catch that, and they run afterwards.
- Recovery does not verify that the recovered tree still gates. It is not
  supposed to: `verify` does that, and it is one command later.

## Proposed shared-document updates

Not made here — `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` and
`docs/autonomy/STATE.md` belong to the integration owner.

- **CHANGELOG.md**, under the build loop: *the supervisor can pick a parked task
  back up — `alo-kernel-loop recover <branch>` restores exactly the files that
  task named, whole where nothing has changed under them and by applying the
  task's own diff where something has, leaving conflicts for a person and
  everything else alone. It publishes nothing and deletes no branch.*
- **QUEUE.md**: v0.01 delivery-plan task 29 (*Recovering a parked task*) done;
  the greeter task is now 28 and *The gates build where there is room* is 30;
  optionally add the parked-branch listing described above.
- **STATE.md**: reference this report, and note that
  `plan::tests::every_plan_this_repository_drives_holds_only_tasks` had been
  failing on the duplicate task number since the greeter task was written —
  which means the change that introduced it did not pass
  `cargo test -p alo-kernel-loop`, a gate the supervisor runs.

## Status

Ready for integration.
