# Sizes made whole for a folder larger than one walk, with one walking on

- Date: 2026-09-14
- Workstream: v0.5 the machine, measured, task 12 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

Task 11 made the index whole for a folder larger than one walk by walking on:
the same walker, asked again from every folder one walk had found and not
entered, under the same bound, until nothing was left. *What is filling the
disk* had the same bound and the same honesty — task 2's `Holding::of` walked
once under `alo_files::MOST_WALKED` and said *cut short* where the walk
stopped — and the same *not enough*: the folder a person opens *what is
filling the disk* on is exactly the one with too much in it. This task makes
the tree of sizes whole the same way, and puts the walking on in the one
place both crates can borrow it from.

## What changed

**The walking on lives in `alo-files`.** `crates/alo-files/src/walking_on.rs`,
new: `Walking::throughout(folder)` is `Walking::through` asked again from
every folder one walk named in `not_entered`, under the same policy and the
same bound, until nothing is left unentered, and it answers a `Gathered` —
the things with every `below` respelled from the folder the caller named,
the links and the unshowable names each counted once, whether it is whole,
the unread and elsewhere folders, and the folders no walk can finish. It is
task 11's `walking_on.rs` moved next to the walk, with two additions: it is
defined for both policies, so a later walk's refusal is an `Unread` under
the measuring policy and the error under the searching one, exactly as one
walk treats the same folder; and `Gathered::links` is carried, corrected the
same way the unshowable count is. `walking.rs` gains two crate-private
helpers — whether the policy notes an unreadable folder, and the policy with
a bound of nothing — and a paragraph saying walking on is next door.
`Walking::through` is not changed by a character, so the six verbs and the
archive bound are as they were. `Gathered` is re-exported from the crate.

**`alo-finding`'s `walking_on.rs` is the call.** Thirteen lines of code:
`everything_under(folder, most)` is `Walking::measuring(most).throughout(folder)`
and `Gathered` is `alo_files::Gathered`. It is still the one file in the
crate that names the walker, and the shipped-source test still finds
exactly one. The five unit tests that lived there moved with the code they
test; `indexing.rs` gained one line in a unit test's struct literal, for the
`links` field. Every task 11 integration test passes unchanged.

**`alo-measuring` counts whole.** `Holding::of` calls `throughout` instead of
`through`; `counting::tree_of` takes a `Gathered` and sets `finished` from
`whole`. Nothing else in the arithmetic changed, and that is the point: the
tree is built from one flat list in which a parent is before its children
whichever walk found each, and the look that counts a file's names is over
the whole list, so a file with two names met by two different walks is one
file. `Counted::NotFinished` now means one thing — a folder holding more
than the bound directly inside it, which no walk can list to its end — and
its rustdoc, `Holding::finished`, `Holding::most` and the module say so.

**The words.** `measuring.filling.cut-short` said *the count stopped after
{most} things*, which is no longer what not-finished means; it now says *a
folder under it holds more than {most} things at a single level, so these
sizes are not the whole of it*, the note rewritten, and the Polish test
translation follows it. `measuring.filling.not-finished` keeps its sentence,
which is still true, and its note now says when it is shown and when it is
not. No key was added or removed; the vocabulary is still twenty-five.

### User-readable change description

*What is filling the disk* now counts a folder whole however large it is.
Before this, a folder of more than twenty thousand things was counted up to
that many and the tree said honestly where the count had stopped — and there
was no way to ask for the rest. Now the count carries on from every folder
it had not entered until it has been everywhere, every size is the sum of
what is inside it to the byte, and a file with two names is still counted
once wherever its names are. The one thing still left out, and said, is a
single folder holding more than twenty thousand things directly inside it;
it is shown with exactly that many, and the sentence above the tree names
the limit. Searching your own files is unchanged in what it does: the
walking on it used since this morning now lives beside the walk, where the
disk count borrows it too. Nothing about what an agent may count or search
changed, and nothing here deletes, moves or empties anything.

## Decisions

- **The one place is `alo-files`, additively, as the plan recommended.** The
  alternatives were a copy in `alo-measuring` — two opinions about which
  folder is walked again and what is counted once, the thing the plan borrows
  one walker to avoid — or a third crate holding one function, which is a
  crate for the sake of a rule. Walking on is a fact about the walk: it is
  the walk's own `not_entered` read back to it, and nothing about it is the
  index's or the tree's. The edit is additive — a new file, a new method,
  two crate-private helpers, a re-export, no change to `through`, no wider
  bound, no second walker — and it is the second and last edit to
  `alo-files` this plan makes. Argued here the way task 2's was: the plan's
  preamble says `alo-files` is read and never edited, its task 12 says this
  decision is to be made in the open, and this is it.
- **The name is `throughout`.** `through` is one walk; `throughout` is the
  same walk, everywhere under the folder. A method on `Walking` rather than a
  free function, because the policy is what decides what a later walk's
  refusal means.
- **Both policies, so that the walk's own rules hold for walking on.** A
  measuring walk-on notes a later walk's refusal as an `Unread`, the way a
  measuring walk notes a folder it steps over; a searching walk-on returns
  it, the way a searching walk fails, because "it is not here" about a folder
  nobody looked in is the false answer that policy exists to refuse. No
  caller makes a searching walk-on today; the test shows it behaves.
- **`links` is carried.** Task 11's `Gathered` had no `links`, because the
  index keeps links as steps and can count them. A searching walk-on steps
  over links and would otherwise lose the count `Walked` carries. The
  correction for a folder listed twice is the same one walk with a bound of
  nothing that already corrects the unshowable names, so it costs nothing
  more. It is the one change to task 11's shape, and it is one line in one
  task 11 unit test.
- **A folder wider than the bound is told apart by its children.** After the
  walk on from it, such a folder holds exactly one walk's bound of names —
  the sorted first `most`, kept by the walk that stopped inside it and
  completed by the walk from it — and never fewer, because a walk that
  stopped short of listing a folder's own names stopped inside a deeper
  folder instead. The tests assert that number, and the root-of-the-machine
  test uses it: every `NotFinished` node it finds must hold `most` children,
  which is the difference between *too wide* and *merely large*.
- **Naming `/` is counted whole.** The plan's constraint is that the whole
  disk is never walked *unasked*; a person who names the root of the machine
  has asked, and the honest answer is the whole root filesystem, stopping at
  each mount point. The task 2 test asserted `!finished` on the grounds that
  a machine is more than one walk; that is no longer a reason, and the test
  now asserts what is true — finished unless a folder is too wide, and each
  such folder holds `most` names — and prints what it measured. It walks the
  development machine's root filesystem, which is what it costs.
- **The sentence changed meaning, so the sentence changed.** Leaving *the
  count stopped after 20000 things* above a tree that had been everywhere but
  one folder would be a false sentence in twenty-four languages, as task 11
  found for the index.

## Acceptance, and the test behind each

| Acceptance | Test |
|---|---|
| whole for a folder of more than one walk's bound, every node's size the sum of its children plus its own files, built in subfolders with known bytes and read back to the byte, timed | `alo-measuring` `what_is_filling_is_a_tree_of_sizes::a_folder_larger_than_one_walk_is_counted_whole_to_the_byte_and_timed` (24,240 things, 240,000 bytes, every node whole and every folder's size the sum of its children, a folder found by the last walk where it is) |
| *cut short* said only for a folder holding more than the bound at a single level, never for one merely large | `alo-measuring` `what_is_filling_is_a_tree_of_sizes::a_count_that_reaches_the_bound_says_so_on_the_folder_and_above_the_tree` (one folder of 20,001 files beside one of 22,220 things in subfolders: the first is `NotFinished` with exactly 20,000 children, the second whole to the byte, one such node in the tree, and the sentence says *single level*) |
| hard links counted once per tree across the walks, the task 2 test extended past the bound | `alo-measuring` `what_is_filling_is_a_tree_of_sizes::a_file_with_two_names_is_counted_once_and_the_second_name_says_where` (three folders of 7,000 files; a second name met only by the second walk, and a pair both past the first walk's stop, each counted once) |
| naming `/` still stops at each mount point and says so, and is otherwise whole | `alo-measuring` `what_is_filling_is_a_tree_of_sizes::naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so` |
| the tree's notes are on the node each is about, `finished` from `whole` | `alo-measuring` `lib` `counting::tests::what_the_walk_could_not_read_enter_or_finish_is_marked_where_it_is` |
| the sentences above and beside the tree, in English and in Polish forms | `alo-measuring` `what_this_crate_says::a_machine_with_no_translations_still_says_everything_in_english`, `what_this_crate_says::every_sentence_is_read_in_the_language_the_person_reads` |
| the walking on is in one place: the walk asked again from a folder it named, under its own bound, nothing kept twice, each folder before what is in it, under both policies | `alo-files` `lib` `walking_on::tests::a_folder_larger_than_one_walk_is_gathered_whole_and_nothing_twice` |
| a folder wider than the bound at one level is left, once tried, said, and holds exactly the bound's worth of names | `alo-files` `lib` `walking_on::tests::a_folder_wider_than_the_bound_at_one_level_is_left_not_entered_and_said` |
| a link found by a later walk is a link: never followed, kept or stepped over by policy, counted once | `alo-files` `lib` `walking_on::tests::a_link_found_by_a_later_walk_is_counted_once_and_never_followed` (Unix) |
| a folder a later walk could not read: unread by a measurement, fatal to a search | `alo-files` `lib` `walking_on::tests::a_folder_a_later_walk_could_not_read_is_unread_by_a_measurement_and_fatal_to_a_search` (Unix, unprivileged — see below) |
| a folder gone before it is walked on from is unread, in this crate's words | `alo-files` `lib` `walking_on::tests::a_folder_gone_before_it_is_walked_on_from_is_unread_in_words` |
| a name that cannot be shown is counted once however often its folder is listed | `alo-files` `lib` `walking_on::tests::a_name_that_cannot_be_shown_is_counted_once_however_often_its_folder_is_listed` (Unix) |
| the folder named is refused the way one walk refuses it, and nothing outside it is walked | `alo-files` `lib` `walking_on::tests::the_folder_named_is_refused_the_way_one_walk_refuses_it` |
| what a walk left is queued in its order, or left not entered | `alo-files` `lib` `walking_on::tests::what_a_walk_left_is_queued_in_its_order_or_left_not_entered` |
| `alo-finding` still walks in one file, with `alo-files`, and every task 11 test passes unchanged | `alo-finding` `nothing_here_opens_a_socket_or_asks_anybody::the_walk_is_alo_files_and_nothing_else_is_rented`; `an_index_made_whole_for_a_folder_larger_than_one_walk` (four tests, unchanged); `lib` `indexing::tests::what_the_walk_could_not_reach_is_kept_beside_the_index` |
| `Walking::through` unchanged: the six verbs' walk, its bound and its refusals | `alo-files` `lib` `walking::tests::a_walk_that_stops_early_says_that_it_stopped_and_where` (unchanged, still passing) |

## The timings, with the machine named

**Machine:** Dell Latitude 5550, Intel Core Ultra 7 155U, 15 GiB, SK hynix
PVC10 NVMe 512 GB — the machine tasks 4, 9 and 11 were measured on.
Windows 11 Pro 10.0.26200; WSL 2 Ubuntu on the same machine with six
processors and 7 GiB visible to it, files under `/tmp` on the Linux
filesystem, unoptimised test profile.

| What | Run 1 | Run 2 | Run 3 |
|---|---|---|---|
| 24,000 files in 24,240 things, 240,000 bytes, written | 164 ms | 315 ms | 165 ms |
| `Holding::of` over them: two walks, 24,000 files looked at, whole to the byte | 144 ms | 124 ms | 173 ms |
| `Holding::of("/")`: 394,024 things, 100,485,194,244 bytes, twenty walks, finished, no folder too wide | 12.39 s | 11.89 s | |

Task 2 measured nothing past the bound; a single walk of 24,240 things
would have stopped at 20,000. Walking on costs the second walk's listing of
one folder again and nothing per file beyond the look the tree already
made. The root filesystem is the honest large case: nearly four hundred
thousand things, a hundred gigabytes, most of it the two lanes' build
directories under `/root`, counted in twelve seconds with `/proc`, `/sys`,
`/dev`, `/run` and `/mnt/c` each a node saying it is elsewhere.

**Memory.** A `Node` is a name, a path, a kind, two sizes, a `Counted` and a
`Vec`: about 150 bytes plus the two strings. The root walk holds roughly
four hundred thousand of them, on the order of a hundred and fifty megabytes
in hand while the answer is built, and the whole measuring suite ran in that
budget without incident. It is one node per thing, not one per word, which
is why the index's memory is the number that grows and the tree's is not;
task 13 is written for the index.

**Not claimed:** anything about a certified machine. These are the
development machine's numbers, as the plan's constraint asks.

## Verification

| Platform | Command | Result |
|---|---|---|
| WSL Ubuntu, `/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1` | `cargo fmt --all` | clean; `cargo fmt --all -- --check` exit 0 on the final tree |
| WSL Ubuntu | `cargo clippy -p alo-files -p alo-finding -p alo-measuring --all-targets -- -D warnings` | clean, exit 0 |
| WSL Ubuntu | `cargo clippy --workspace --all-targets -- -D warnings` | clean, exit 0 |
| WSL Ubuntu | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-files -p alo-finding -p alo-measuring --no-deps` | clean, exit 0 |
| WSL Ubuntu | `cargo test -p alo-files` | 106 unit + 6 + 11 + 12 + 4 + 5 + 5 + 2 integration + 1 doc test, all passing |
| WSL Ubuntu | `cargo test -p alo-finding` | 65 unit + 4 + 3 + 1 + 6 + 4 + 5 + 6 + 7 + 8 + 5 + 3 + 7 integration + 2 doc tests, all passing |
| WSL Ubuntu | `cargo test -p alo-measuring` | 60 unit + 4 + 7 + 8 + 5 + 3 integration + 2 doc tests, all passing; the eight disk tests take 49 s, of which the root of the machine is 12 |
| WSL Ubuntu, as `nobody` | the `alo_files` lib test binary copied to `/tmp` and filtered to `walking_on::tests` | 8 passing: the gates run as root, which no permission stops, so the unreadable-folder test takes its *this process reads anything* branch there; run unprivileged it takes the refusal branch for both policies, and passes |

The workspace suite was not run by this worker, as the brief asks; the
supervisor runs it. Nothing outside the three crates and two documents
changed; `Cargo.lock` is untouched because no dependency moved.

## Limitations

- A folder holding more than `MOST_WALKED` things directly inside it is
  counted up to that many and marked; lifting that is a change to the walk
  itself, and this plan's two edits to `alo-files` are spent.
- The root-of-the-machine test now walks the whole root filesystem of the
  machine it runs on: twelve seconds here, longer on a fuller disk. That is
  the product's answer to *what is filling `/`*, measured rather than
  assumed, and the report says what it costs.
- The unreadable-folder test is decided at run time by whether `read_dir`
  refuses: under root, or on a host without modes, it checks the
  whole-and-nothing-unread branch instead. The unprivileged run above is how
  the refusal branch was shown.
- The two tests past the bound write forty-five thousand files between
  them. On Linux that is under a second; on Windows the whole test file is
  compiled out, since `Holding::of` refuses there.

## Proposed shared-document updates

For `CHANGELOG.md`, under the unreleased v0.5 section:

> **What is filling the disk: sizes made whole for a folder larger than one
> walk.** A folder of more than twenty thousand things is now counted to the
> end: the count walks on from every folder one walk had not entered, under
> the same bound each time, every size is the sum of what is inside it to
> the byte, and a file with two names is counted once wherever they are. A
> single folder holding more than twenty thousand things directly inside it
> is the one thing still left, and the tree says so on that folder and once
> above it. The walking on that search and the disk count share now lives
> beside the walk in `alo-files`, additively; the file verbs are unchanged.
> Measured on the development machine: 24,240 things counted whole in under
> two hundred milliseconds, and the whole root filesystem in twelve seconds.

For `docs/autonomy/QUEUE.md` and `STATE.md`: task 12 of the machine-measured
plan done; task 13 written and ready — an index that fits in hand, the words
of a large folder bounded and said, the memory number task 11 measured
argued into a bound.
