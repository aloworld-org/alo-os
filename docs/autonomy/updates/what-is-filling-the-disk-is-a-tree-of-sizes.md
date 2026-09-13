# What is filling the disk — a tree of sizes a person can open up

- Date: 2026-09-13
- Workstream: v0.5 the machine, measured, task 2 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

`docs/features.md` promises, at v0.5: *what is filling the disk — shown as
sizes you can open up and click through, not a number in Settings.* The
click-through is the shell's. This task is the tree of sizes underneath it,
and the plan's own bar for it: *true to the byte for the one directory a
person opens up rather than roughly right for the whole disk*, and *a size
that is silently too small is worse than no size.*

## What changed

**`crates/alo-measuring`: `Holding::of(folder)` answers a folder as a tree.**
A `Holding` is the folder asked about as a `Node`, and every `Node` is a name,
a path, what it is (`alo_files::Kind`, re-exported), its own bytes, its size —
**its own bytes plus the sizes of its children** — and a `Counted` saying
whether that size is the whole truth. `Counted::Whole` says nothing; the other
four arms each carry a sentence in the reader's language for the window to
show beside the size:

- `Elsewhere { at }` — a file with more than one name on the disk, counted
  under the first name the walk met; this name has no bytes of its own and
  says which name they are under. Two names for a hundred bytes make a
  folder of a hundred bytes, not two hundred.
- `NotRead { why }` — a folder the machine would not read, or a file that
  went away between the walk and the look at it. Its size is what could be
  seen, which for a folder is nothing, and the node says so: the difference
  between *unknown* and *empty*.
- `OnAnotherFilesystem` — a folder where another filesystem is attached, not
  entered. Naming the root of the machine is answered this way: a tree that
  stops at each mount point, with `/proc` a node saying why.
- `NotFinished` — a folder the walk had not finished when it reached its
  bound. Its size is the size of what was seen.

Above the tree, `Holding::finished` and `Holding::not_the_whole` say once
when the count stopped at the walk's bound of twenty thousand things, and
`Holding::unnamed` and `Holding::left_unnamed` say once how many things were
left out because their names cannot be shown safely. A link is a `Kind::Link`
node whose bytes are the length of the link itself, and nothing behind it is
ever walked.

Three new files, one responsibility each: `holding.rs` is the answer and its
sentences; `counting.rs` turns a walk into the tree — an arena assembled from
the last step to the first, so nothing recurses and a folder nested two
thousand levels deep is a tree like any other, with a `Drop` that lets go of
levels rather than recursing too; `looking.rs` is the one look at each file
after the walk, which is where how many names it has is read. On any host but
Linux `Holding::of` refuses with `NotMeasured::NotOnThisHost`, the way
`Reading::now` does: the arithmetic is portable and tested everywhere against
a walk a test wrote out, but whether a file has a second name is a question
only a Linux host answers here, and a count that could not tell a second name
from a second file would be a size that is silently wrong.

**`NotMeasured::NotCounted { at, why }`** is the one refusal of the whole
question: the folder named is not there, is a file, or could not be read.
`why` is `alo_files::Failed`, and the sentence carries the file half's own
inside it, composed with `Filling::and_said`, so a shell that collected both
crates' words shows one sentence and a shell that forgot `alo-files`' shows
the inner half as a marked key rather than as untranslated English.

**Words.** Five phrases and two countable sentences under `measuring.filling.`,
collected through `alo-saying`'s existing entry for this crate. The Polish
test renders every one, and checks the *few* form of *N things whose names
cannot be shown* at three.

**`crates/alo-files`: the walker is public, with two policies.** See the
decision below. `walking.rs` gains `Walking::searching(most)`, which is
exactly the walk `find_in_folder` and `archive_folder` made before — the
`pub(crate) fn walk` they call is now a one-line wrapper over it — and
`Walking::measuring(most)`, which differs in three things: a link is kept as
a `Step` whose bytes are the link's own, a folder the machine will not read
is noted in `Walked::unread` with what the machine said and stepped over
rather than ending the walk, and a folder on another filesystem is noted in
`Walked::elsewhere` and not entered. Both policies now also fill
`Walked::not_entered` — the folder being listed and every folder found and
not yet entered — when a walk is cut short, so that a measurement can mark
those folders rather than report a half-listed folder as whole. `Step`,
`Walked`, `Unread`, `Walking` and `MOST_WALKED` are re-exported from the crate
root. Nothing about how a link or a name is decided changed, and the existing
walker tests pass unchanged except for one that now also checks
`not_entered`.

## Decisions

**The walker in `alo-files` was edited, additively, against the plan's
"reads and never edits."** The plan's acceptance says the walk *is
`alo-files`' walker, borrowed and not reimplemented*, and gives the reason:
a second walker would be two opinions about a symlink. The walker was
`pub(crate)`, so it could not be borrowed without a change there at all; and
as written it could not express three of the acceptance's clauses — it
discards links after counting them, so a link's bytes could not be reported;
it fails the whole walk on an unreadable folder, so such a folder could never
be *a node saying so*; and it knows nothing of filesystems, so a tree could
not stop at a mount point. The choice was between an additive change to the
walker and a second walker in `alo-measuring`. The second walker is what the
plan's reason argues against, and it would fail the acceptance criterion as
written; the additive change honours the reason and the criterion at the
cost of one sentence in the plan's preamble. `alo-files` is not a crate lane A
owns this week, the change leaves both existing callers' behaviour identical,
and it is covered by five walker tests. The plan's preamble now carries a
parenthesis saying the walker is public and task 3 borrows it the same way.
If the owner would rather the walker had stayed private, the alternative is
written above and this report is the place it is argued.

**A hard link is counted under the first name the walk met.** The walk is
breadth-first and each folder's contents are sorted, so *first* is
deterministic and a person can predict it: the shallower name, then the
earlier in the alphabet. The alternative — counting it under none and listing
all names — would have made every folder's size smaller than what `du` says
by the bytes of every linked file, which is a size that is silently too small.

**A file is looked at once more, after the walk.** The walk read a size for
every file as it went, from the folder's entry for it. `counting.rs` looks at
each file itself through a function the caller hands in, for two reasons:
only that look says how many names the file has, and the look is after the
walk, so a file that became a link in between is seen as a link — its own
bytes, nothing followed — rather than as the file the walk thought it was.
The function is handed in so the tree is tested on every host against a walk
a test wrote out, the way task 1's `Kernel` trait is.

**The unreadable folder in the tests is made by depth, not by permissions.**
The supervisor's gates run as the administrator, whom permissions do not
refuse, so a test that took a folder's permissions away would pass without
ever reaching the refusal. A folder nested until its path is longer than the
kernel will open is refused for the administrator too, by the same `read_dir`
call, and reaches the walker's noting branch by the same line. `alo-files`'
unit test makes it with `mkdirat` against an open handle (it has `rustix`);
`alo-measuring`'s integration test, which may add no dependency, makes it
one level at a time from inside the previous one and restores the working
directory after — the only test in that binary that touches it, and every
other path in the binary is absolute.

**Mount points stop every count, not only a count of `/`.** The plan names
`/` as the case; the mechanism is the same for any folder, and a folder's size
that included a drive plugged in underneath it would not be *what is filling
the disk*. On a host that cannot say which filesystem a folder is on the
measuring policy enters everything, and `Holding::of` refuses on such hosts
anyway.

**The type is `Holding`, not `Filling`.** `alo_strings::Filling` is imported
in most of this crate's files, and a public type that shares a name with a
dependency's most-used type would be a paper cut in every consumer.

## Acceptance criteria and the tests that hold them

| Acceptance | Test |
|---|---|
| each node's size is its children plus its own files, to the byte, read back from a folder of known bytes | `what_is_filling_is_a_tree_of_sizes::every_size_is_the_sum_of_its_children_plus_its_own_files_to_the_byte` |
| hard links are counted once per tree, not once per name, and the test names the difference (101, not 301) | `what_is_filling_is_a_tree_of_sizes::a_file_with_two_names_is_counted_once_and_the_second_name_says_where` |
| a link is the bytes of the link and is never followed | `what_is_filling_is_a_tree_of_sizes::a_link_is_the_bytes_of_the_link_and_is_never_followed` |
| a folder the person may not read is a node saying so rather than a zero | `what_is_filling_is_a_tree_of_sizes::a_folder_that_cannot_be_read_is_a_node_saying_so_rather_than_a_zero` |
| a walk cut short at `MOST_WALKED` says so rather than reporting a partial sum as a total | `what_is_filling_is_a_tree_of_sizes::a_count_that_reaches_the_bound_says_so_on_the_folder_and_above_the_tree` |
| naming `/` is answered with a tree that stops at each mount point and says so | `what_is_filling_is_a_tree_of_sizes::naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so` |
| the folder itself missing, or a file, is refused in words naming it | `what_is_filling_is_a_tree_of_sizes::a_folder_that_is_not_there_or_not_a_folder_is_refused_in_words` |
| what the walk could not read, enter or finish is marked on the node it is about, the root included | `counting::tests::what_the_walk_could_not_read_enter_or_finish_is_marked_where_it_is` |
| a folder nested thousands deep is a tree, without recursion | `counting::tests::a_folder_nested_thousands_deep_is_a_tree_without_recursion` |
| the tree takes no account of who asked | `nothing_here_acts_or_asks_who_is_asking::the_list_takes_no_account_of_who_asked` |
| nothing here deletes, moves, empties, signals or opens a socket | `nothing_here_acts_or_asks_who_is_asking::nothing_in_the_shipped_source_signals_stops_renices_or_writes` |
| the walk is `alo-files`' and nothing else is rented | `nothing_here_acts_or_asks_who_is_asking::the_numbers_come_from_proc_and_from_no_rented_crate` |
| every sentence is read in the person's language | `what_this_crate_says::every_sentence_is_read_in_the_language_the_person_reads` |
| the one refusal worded partly by the file half carries its sentence | `what_this_crate_says::the_one_refusal_worded_by_the_file_half_carries_its_sentence` |
| a measuring walk keeps a link as its own bytes and never follows it | `alo-files` `walking::tests::a_measuring_walk_keeps_a_link_as_its_own_bytes_and_never_follows_it` |
| an unreadable folder ends a search and is noted by a measurement | `alo-files` `walking::tests::a_folder_that_cannot_be_read_ends_a_search_and_is_noted_by_a_measurement` |
| a measuring walk does not enter another filesystem and says which | `alo-files` `walking::tests::a_measuring_walk_does_not_enter_another_filesystem_and_says_which` |
| a walk that stops early says where it had not finished | `alo-files` `walking::tests::a_walk_that_stops_early_says_that_it_stopped_and_where` |
| a searching walk still steps over links exactly as before | `alo-files` `walking::tests::a_link_found_on_the_way_is_counted_and_not_followed` |

The non-Linux refusal, `holding::tests::on_any_other_host_nothing_is_counted`,
is `cfg(not(target_os = "linux"))` and ran on Windows; it is not in the
handoff's evidence because the supervisor's evidence run is on Linux, where
it does not exist.

## Verified

WSL Ubuntu on this development machine, as root, `CARGO_TARGET_DIR` the
supervisor's own (`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`), foreground,
exit codes read:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (workspace) | clean, 0 warnings |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-files -p alo-measuring --no-deps` | clean |
| `cargo test -p alo-files` | 98 unit + 6 + 11 + 12 + 4 + 5 + 5 + 2 integration + 1 doctest: all passed |
| `cargo test -p alo-measuring` | 50 unit, 3 + 7 + 5 + 3 integration, 1 doctest: all passed |
| `cargo test -p alo-saying` | 63 + 4 + 1 passed (the vocabulary count is summed, not fixed) |
| every evidence line, alone, `--exact` | 1 passed each, 19 lines |

Windows, this development machine: `cargo fmt --all`, `cargo clippy -p
alo-files -p alo-measuring --all-targets -- -D warnings` clean; `cargo test
-p alo-measuring` 51 + 3 + 3 + 1 passed, including the non-Linux refusal;
`cargo test -p alo-files` 88 + 11 + 3 + 5 + 5 + 2 + 1 passed (the four Linux
walker tests do not exist there).

**Not run by this worker:** the full workspace suite, per the task's
instruction; the supervisor runs it. **Not claimed:** anything about a
certified machine. The test that names `/` ran on WSL2's root filesystem,
where `/proc`, `/sys`, `/dev`, `/run` and `/mnt/c` are separate filesystems
and the tree stops at each; a physical alo OS machine has the same layout
under `/`, and the test is the same test.

## Remaining limitations

- **Things whose names cannot be shown are left out of every size**, and
  the count says how many. The walker drops them before reading a size, and
  keeping their bytes without their names would need a per-folder count in
  the walker that nothing else wants yet. On a machine where somebody has a
  file with a control character in its name, that folder's size is smaller
  than `du` says by that file, and the sentence above the tree says so.
- **A file on another filesystem reached by a bind mount of a single file**
  is counted as a file of this filesystem; only folders are checked for
  which filesystem they are on. Rare, and the safe direction: a size that is
  too large by one file rather than a folder silently skipped.
- **Nothing here is a verb yet.** Task 5 of the plan declares the read
  verbs; `Holding::of` is the function it will call.

## Proposed changelog entry

*What is filling the disk* (v0.5): `alo-measuring` answers a folder as a
tree of sizes, each node the sum of its children plus its own bytes, with a
hard link counted once and its other names saying where, a link counted as
the bytes of the link and never followed, a folder the machine would not
read shown as unknown rather than empty, a mount point a node the tree stops
at, and a count that reached its bound saying so on every folder it had not
finished. The walk is `alo-files`', now public with a searching and a
measuring policy. Read-only, Linux-only with a refusal elsewhere.

## Proposed queue and roadmap updates

Task 2 of `v0-5-the-machine-measured-plan.md` is marked done in the plan.
Tasks 3 and 4 are ready; task 5 depends on 1, 2 and 3, and now has two of the
three.
