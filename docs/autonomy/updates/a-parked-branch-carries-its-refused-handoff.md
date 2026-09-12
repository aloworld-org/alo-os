# A parked branch carries the handoff its gates refused

**Date:** 2026-09-12. **Workstream:** the build loop (`tools/kernel-loop`).
**Contributor:** Claude, in `C:\dev\alo-os-claude`.
**Plan task:** v0.01 delivery plan, task 36, *A parked branch carries the
handoff its gates refused*.

## What changed

Parking now carries the newest refused handoff for the task onto the parked
branch when the worker left none, and `recover` reads it from there.

Task 35 taught `recover <branch>` to reconstruct a handoff-less branch from
`.kernel-loop/refused/`. That works on the checkout that parked the branch and
nowhere else: the refused directory lives inside the ignored `.kernel-loop`,
so a parked branch copied to another checkout — which is the one reason parked
work is a branch rather than a stash — arrives with every line of the work and
no way to name it. The upstream cause was that parking force-added
`.kernel-loop/handoff.toml` and nothing else, and on the road that produces
most parks the repair path had already moved that file into `refused/`.

Now:

- **Parking carries the refused handoff.** When no handoff is waiting and the
  newest refused one for the task being parked exists, parking copies that
  entry to `.kernel-loop/refused-handoff.toml` and force-adds it onto the
  branch beside the handoff's own name. A distinct name, so nothing can read
  it as a handoff the parked worker wrote. The refused entry itself stays in
  `refused/`; the copy is taken back out of the loop's directory whether the
  park succeeded or not, so a failed park leaves nothing for the next one to
  carry under the wrong task.
- **Parking with neither still parks.** A branch with the work and no name is
  still better than no branch. Parking still pushes nothing, and it still
  never writes `.kernel-loop/handoff.toml` itself — that is the worker's word.
- **`recover` prefers, in order,** the branch's own `.kernel-loop/handoff.toml`,
  the `.kernel-loop/refused-handoff.toml` the branch carries, and this
  checkout's `.kernel-loop/refused/` newest-first — and reports which it used,
  on the terminal and in the journal. The two reconstructed cases are reported
  exactly as task 35 reports one: as reconstructed, with the files the handoff
  names that the branch never changed and the files the branch changed that
  the handoff does not name, and not ready to gate until a person has made the
  handoff say what the branch says.
- **The loop's own files never come back into the tree.** With the branch's
  commit as the file list, anything under `.kernel-loop/` is parking's and not
  the task's: the carried handoff stays on the branch and is listed as left
  alone, and the handoff written back is the file's text, byte for byte.
- **A carried handoff that does not read is refused in words**, naming the
  file, rather than passed over for the directory. Parking copied it from a
  handoff the gates had read, so one that no longer reads has been changed
  since, and a recovery that quietly used a different source would hide that.

Source:

- `tools/kernel-loop/src/parking.rs` — new. The step before and after the git
  of parking: `parking::parked` copies the refused handoff for the branch to
  carry, calls `repository::parked`, and takes the copy back. `Parked` says
  which branch and, when one was carried, which refused entry it came from,
  and the loop writes that in its journal.
- `tools/kernel-loop/src/handoff.rs` — `Handed::carried_for_parking`, the copy
  and the two conditions under which nothing is copied; `what_parking_carries`,
  the one list of the two names parking force-adds, so what parking adds and
  what recovery looks for cannot drift; `where_a_carried_one_goes` and
  `the_carried_copy_is_taken_back`.
- `tools/kernel-loop/src/repository.rs` — `parked` force-adds both names from
  that list. Every git this loop runs is still in this one file, and its
  order — abandon a stopped rebase, then switch — is unchanged.
- `tools/kernel-loop/src/recovering.rs` — the three sources in order;
  `TakenFrom` on the reconstructed account, with a `Display` a person reads;
  the file list filtered to the task's work; the refusal for a carried handoff
  that does not read; the module documentation; and the tests.
- `tools/kernel-loop/src/main.rs` — the loop holds the task it is driving
  rather than only its number, so parking can be told the name a refused
  handoff carries; the journal says when a park carried one; the recovery
  output names where the handoff came from; and a paragraph in the crate
  documentation.
- `docs/autonomy/v0-01-delivery-plan.md` — task 36 marked done, and task 37
  written after it.

### The refusals

Nothing is written to the tree before any of them. The seven from tasks 29
and 35 are unchanged in effect; the two that mention where this looked now
also say the branch carries no refused handoff. This one is new:

| Asked | Answered |
| --- | --- |
| a branch whose carried `.kernel-loop/refused-handoff.toml` does not read | *the refused one it carries as … does not read: …* — nothing restored, the directory not consulted, and `git show <branch>:<file>` named as what to look at |

### A user-readable change description

When the build supervisor parks a task whose worker never wrote the file that
names its work, the parked branch now carries the last handoff the gates
refused for that task, under a name that says what it is. Picking the branch
up works on any checkout — not only the one that parked it — and the recovery
says plainly that the handoff was reconstructed and from where. A branch with
nothing to carry is still parked, and parking still publishes nothing.

## Decisions taken

- **A new module, `parking.rs`, rather than more in `repository.rs` or
  `main.rs`.** `repository.rs` is every git this loop can run, as one readable
  list, and deciding what a branch carries is not git. `main.rs` is the
  program's shape. The copy-park-take-back sequence is one responsibility, and
  it is the thing the tests call as *the real `parked`*.
- **The two names are one list.** `Handed::what_parking_carries` is read by
  `repository::parked` to force-add and by `recovering` to look. The directory
  is ignored, so a name that differed by a letter between the two would be a
  branch carrying a file nothing reads, and no test would notice until a
  recovery on another machine.
- **Parking is told the task's name, not asked the plan.** The loop already
  holds the task it chose; it now holds the whole task rather than the number.
  `repository::parked` keeps naming the branch by number, as every parked
  branch has been named, and `carried_for_parking` looks for the name, which
  is what a handoff carries.
- **A failure to copy is a failure to park.** If `.kernel-loop` cannot take a
  small file, the git that follows will not fare better, and a park that
  stopped is the loop stopping with the work still in the tree — the safe
  answer for a checkout that will not do as it is asked. The copy is taken back
  before the error is handed on.
- **The carried handoff beats this checkout's directory, however new the
  directory's entry.** The branch says what it was parked with; the directory
  says what happened on this checkout since, which may be a later attempt at
  the same task. A test holds the order.
- **A carried handoff that does not read is a refusal, not a fall-through.**
  Everything parking copies was read by the gates first. The only way it stops
  reading is somebody editing the branch, and a supervisor that silently took
  a different source would be hiding a tampered file.
- **The task's work excludes `.kernel-loop/`.** The branch's commit is the
  file list, and parking's two force-added files are on it. Bringing the
  carried handoff back would put a file into the ignored directory that the
  next park would carry under whatever task it was for. It stays on the branch
  and is reported as left alone. A branch whose only change is under
  `.kernel-loop/` is refused as changing nothing, which is what it is.
- **Two task-35 tests now park before any handoff is refused.** With parking
  carrying, a fixture that refuses first and parks second walks the carried
  road, and the directory road — every branch parked before this change —
  would have gone untested. Ordering the fixture the other way is exactly how
  such a branch came to be.

### Acceptance, criterion by criterion

| Criterion | Test |
| --- | --- |
| parking with no handoff waiting and a refused one for the task force-adds it as `.kernel-loop/refused-handoff.toml`, a distinct name; the refused entry stays; no `handoff.toml` is written; the copy does not linger | `recovering::tests::parking_carries_the_newest_refused_handoff_when_the_worker_left_none` — the real road: `put_aside`, a second worker, the real `parked`; `handoff::tests::the_newest_refused_handoff_is_copied_for_parking_to_carry`; `handoff::tests::what_parking_carries_is_the_handoff_and_the_refused_one_under_different_names` |
| `recover` prefers the branch's own handoff, then the carried one, then `refused/`, and reports which | `recovering::tests::nothing_is_carried_when_the_worker_left_a_handoff` (own over carried); `recovering::tests::a_carried_refused_handoff_is_preferred_to_this_checkouts_refused_directory` (carried over directory); `recovering::tests::the_newest_refused_handoff_for_the_task_is_the_one_taken` (the directory, on a branch that carries none); `handoff::tests::nothing_is_copied_when_a_handoff_is_waiting` |
| both reconstructed cases are reported exactly as task 35 reports one, differences and all | `recovering::tests::a_carried_refused_handoff_recovers_the_branch_on_another_checkout` — a real `git clone` with no `refused/`; `recovering::tests::a_branch_with_no_handoff_is_recovered_from_its_commit_and_the_refused_handoff` |
| parking with neither still parks | `recovering::tests::parking_with_neither_a_handoff_nor_a_refused_one_still_parks`; `handoff::tests::nothing_is_copied_when_no_refused_handoff_names_the_task` |
| the refusals, beside the recoveries, on branches made by the real `parked` | `recovering::tests::a_carried_refused_handoff_that_does_not_read_is_refused_in_words`; the task-35 refusals `a_branch_with_neither_a_handoff_nor_a_refused_one_is_refused_in_words` and `a_branch_numbered_for_a_task_the_plan_does_not_know_is_refused`, unchanged in effect |
| parking pushes nothing and never writes `handoff.toml` itself | `repository::tests::the_only_branch_this_pushes_is_main`, unchanged; the three parking tests above assert no handoff waits afterwards |
| the file list comes from the branch's own commit, never `git status` | `recovering::tests::a_reconstruction_reads_its_file_list_off_the_branch_and_not_the_tree`, unchanged |
| `git checkout -- .` and `git restore -- .` remain refused | `recovering::tests::recovery_never_touches_more_than_one_named_path`, unchanged and still reading the source |

## Verification

Windows 11, PowerShell, from `tools/kernel-loop` (its own workspace):

```
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test -p alo-kernel-loop
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

All ran and passed on 2026-09-12: fmt clean, clippy with zero warnings,
rustdoc with zero warnings, and `102 passed; 0 failed` (92 before this change,
ten added). The recovering tests build real git repositories, park with the
real `parking::parked`, and one clones the repository into a second checkout
and recovers there; nothing is mocked.

Not run here, deliberately: the product workspace's full suite. This change
touches only `tools/kernel-loop`, which is its own workspace and not part of
what ships; the supervisor runs the full suite after this regardless.

Not exercised: a real parked branch crossing between `C:\dev\alo-os` and
`C:\dev\alo-os-claude`. The crossing is a `git fetch` by hand today, and
making it a command with its own refusals is task 37, written in the plan.

## Limitations

- Branches parked before this change carry no refused handoff and are
  recovered as task 35 recovers them: from this checkout's `refused/`, or
  refused in words that now also say the branch carries none.
- The carried handoff is the first worker's and may not describe the branch.
  As in task 35, the differences are reported and the recovery refuses to call
  itself ready; a person still edits the file.
- Moving a parked branch to another checkout is still a `git fetch` typed by
  hand. Task 37 makes it a command.

## Proposed changes to the shared documents

- **CHANGELOG.md:** *The build supervisor's parking now carries the newest
  refused handoff for the task onto the parked branch when the worker left
  none, so `recover <branch>` can reconstruct the task on any checkout and
  says where the handoff came from.*
- **QUEUE.md / STATE.md:** task 36 of the v0.01 delivery plan is done; task 37
  is written and ready.
- **ROADMAP.md:** no change.

## Status

Ready for integration.
