# An index made whole for a folder larger than one walk

- Date: 2026-09-14
- Workstream: v0.5 the machine, measured, task 11 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

`alo_files::MOST_WALKED` is twenty thousand: the most things one walk looks
at, so that a verb over a granted folder is bounded whatever the folder
holds. Task 3 borrowed that walk for the index and was honest about the
bound — an index of a bigger folder said it was not whole and named the
folders the walk had not entered — and honest was not enough: a photo
library, a source tree with its dependencies or a Documents folder a decade
old is more than twenty thousand things, and for them *search your own
files* was a search box that would never find the rest. This task makes the
index whole for such a folder by walking on — the same walker, asked again
from each folder it named, under the same bound — and measures what that
costs on the development machine.

## What changed

**Walking on.** `crates/alo-finding/src/walking_on.rs`, new, and the one
file in the crate that names `alo_files::Walking`, which the shipped-source
test holds it to. `everything_under(folder, most)` walks the folder once
and then, for every folder that walk left in `not_entered` — the folder it
stopped inside and every folder it had found and not yet entered — walks
again from that folder, under the same bound, until nothing is left
unentered. Each later walk's steps have their `below` respelled from the
folder the person named, so an entry says where a thing is the way it does
for a small folder, and `Step::at` is untouched. The result is a `Gathered`:
the things, the unread and elsewhere folders spelled from the root, the
count of unshowable names, whether it is whole, and the folders no walk
could finish.

**What still cannot be made whole.** The walker lists a folder, sorts its
names and keeps the first `most`; a folder holding more than `most` things
**at one level** stops it in the same place every time. Walking on from
such a folder would be the same walk again, so it is walked once more and,
when that walk stops inside it again, it is left in `not_entered` and the
index is not whole — with the first `most` names in it kept, and everything
under the folders among them. The loop cannot run forever: every folder
taken off the queue is either finished, left, or replaced by folders
strictly deeper than it.

**A folder listed twice is counted once.** The folder a walk stopped inside
was listed whole before anything in it was kept, and walking on from it
lists it again. Its steps already kept are told apart by path and not kept
twice; the unshowable names are only a count, so the count in that one
folder is taken once more — with a walk that keeps nothing, bound zero —
and subtracted. A test on Unix, where such a name can be made, holds the
count to three across six bounds.

**A folder a later walk could not read.** The walker's own refusal for the
root of a later walk becomes an `Unread` in the gathering, spelled from the
folder named, exactly as a folder the first walk stepped over: the
machine's own words when it said no, and for a folder gone between the walk
that found it and the walk on from it, `alo-files`' own sentence for a
thing that is not there any more, through `alo-strings` with the file
words. Nothing here invents an English sentence.

**The index.** `crates/alo-finding/src/indexing.rs` consumes a `Gathered`
instead of a `Walked` and takes the bound as an argument, so that a unit
test can make a folder of a few dozen files take several walks and count
the reads; `Index::of` and `Index::again` pass `alo_files::MOST_WALKED`,
and `Covered::most` still says what one walk's bound is. `Index::again`
walks on the same way, so an index an earlier version cut short at one walk
is made whole by it, reading only the files it had not reached — the
vouching is by path and does not care which walk found a file.

**The words.** `finding.not-whole` said *the index stopped after {most}
things*, which is no longer what not-whole means; it now says *a folder
under it holds more than {most} things at a single level, so the index is
not the whole of the folder*, with the note rewritten, and the Polish test
translation in `tests/what_this_crate_says.rs` follows it. The
`finding.not-searched.not-entered` sentence is still true as written and
only its note changed. `Covered`, `NotSearched` and `Unsearched` say in
their rustdoc what `whole` and `not_entered` mean now.

**The contract.** `docs/contracts/file-index.md`: `whole` and `not_entered`
describe the new meaning, and say that an index written by a version that
stopped at one walk lists there the folders that walk had not entered, and
is made whole the next time it is made again. `format` is still `1` and no
field was added — a test reads the head of a cut-short index back and
counts its fields.

**Nothing else moved.** The verb, `Searched::of`, the list, `Indexed`,
`InHand` and every disk form are unchanged; the walk is `alo-files`' and
`alo-files` is not edited; the shipped-source test still finds one file
that walks, no socket, no clock, no watcher.

### User-readable change description

Searching your own files now finds everything in a folder however large it
is. Before this, a folder of more than twenty thousand things was indexed
up to that many and the search said honestly which folders it had never
entered — and there was no way to ask for the rest. Now the index carries on
from each of those folders until it has been everywhere, and says so; a
folder indexed by an earlier version and cut short is completed the next
time it is brought up to date, reading only the files it had not reached.
The one thing still left out, and said, is a single folder holding more
than twenty thousand things directly inside it. Nothing about what an agent
may search changed.

## Decisions

- **The name.** The plan offered *`Index::of` walking on from each folder in
  `not_entered`*, and that is what it is: no new public call. The walking on
  is a private file, `walking_on.rs`, with one function, `everything_under`,
  because the constitution's fourth law wants the thing that decides *which
  folders to walk again, and what to do with what they found* apart from the
  thing that decides *which files to open*. It is also what keeps the
  shipped-source rule — one file names the walker — true without weakening it.
- **A folder wider than the bound stays not entered, once tried.** The plan
  forbids a second walker and a wider bound, and the walker as it is cannot
  list such a folder to its end. Walking on from it once costs one extra
  listing and finds nothing new; not trying at all would leave a folder that
  merely *looked* stuck. The report says the honest thing: a folder of more
  than twenty thousand things at one level is indexed up to twenty thousand,
  the index says it is not whole, and every answer names the folder. Lifting
  that is a change to `alo-files`, which this plan does not make.
- **The unnamed count is corrected rather than over-counted.** A count that
  is wrong by the number of unshowable names in one folder is a small lie
  above the index; a walk with bound zero is the walker asked its own
  question and costs one listing per walk on. Chosen over a new field on the
  walker, which would edit `alo-files`.
- **The bound is an argument inside the crate.** `indexing::assembled` takes
  `most` so a unit test can walk a folder of fifty things seven at a time
  and count every read; nothing public takes a bound, and `Covered::most`
  is whatever each walk was under, which from the public way in is always
  `MOST_WALKED`.
- **The refusal of a later walk is mapped, not stringified.** `Failed` has
  no `Display` on purpose; the machine's words are carried as they are, and
  the one other case goes through `alo-strings` with `alo_files::file_words`
  rather than through a `format!("{why:?}")`.
- **The sentence changed meaning, so the sentence changed.** Leaving *the
  index stopped after 20000 things* above an index that had been everywhere
  but one folder would be a false sentence in twenty-four languages.

## Acceptance, and the test behind each

| Acceptance | Test |
|---|---|
| an index whole for a folder larger than one walk's bound, every `below` spelled from the folder named, `whole` true, `not_entered` empty, `most` still one walk's bound; built in subfolders, every file in it, timed | `an_index_made_whole_for_a_folder_larger_than_one_walk::a_folder_larger_than_one_walk_is_one_whole_index_with_every_file_in_it_timed` (24,400 things, 24,000 files; `Index::where_is` of every entry is on the disk; a link out of the folder is a link and nothing under it is indexed; written and read back the same) |
| `Index::again` on that folder reads only what changed, checked by `Index::opened` | `an_index_made_whole_for_a_folder_larger_than_one_walk::indexed_again_a_folder_larger_than_one_walk_reads_only_what_changed` (zero, then exactly two: one rewritten past the first walk, one added before it) |
| an index file written by task 3 for a folder cut short still reads, and is made whole by `Index::again`; `format` still `1`, no field added | `an_index_made_whole_for_a_folder_larger_than_one_walk::an_index_cut_short_by_an_earlier_version_still_reads_and_is_made_whole_by_again` (only the files it had not reached are read) |
| a subfolder the machine would not read says so in `Covered::unread` exactly as today, and *nothing matched* is not said about it | `an_index_made_whole_for_a_folder_larger_than_one_walk::a_subfolder_the_machine_would_not_read_past_the_first_walk_is_unread_and_said_so` (past where the first walk stopped; run unprivileged, see below) |
| walking on is the walker asked again from a folder it named, under its own bound, with nothing kept twice and each folder before what is in it | `walking_on::tests::a_folder_larger_than_one_walk_is_gathered_whole_and_nothing_twice` (five bounds, each the same set as one walk the folder fits under) |
| a folder wider than the bound at one level is left, once tried, and said; the walk terminates | `walking_on::tests::a_folder_wider_than_the_bound_at_one_level_is_left_not_entered_and_said` |
| the walking on is counted by the reader: every file once, none again | `indexing::tests::a_folder_larger_than_one_walk_is_indexed_whole_and_again_reads_nothing` |
| a name that cannot be shown is counted once however often its folder is listed | `walking_on::tests::a_name_that_cannot_be_shown_is_counted_once_however_often_its_folder_is_listed` (Unix) |
| a folder a later walk could not read is unread, and the rest is gathered | `walking_on::tests::a_folder_a_later_walk_could_not_read_is_unread_and_the_rest_is_gathered` (Unix, unprivileged) |
| what a walk left is queued in its order, or left not entered | `walking_on::tests::what_a_walk_left_is_queued_in_its_order_or_left_not_entered` |
| one file walks, no socket, no clock, no watcher | `nothing_here_opens_a_socket_or_asks_anybody` (unchanged, still passing) |

## The timings, with the machine named

**Machine:** Dell Latitude 5550, Intel Core Ultra 7 155U, 15 GiB, SK hynix
PVC10 NVMe 512 GB — the machine tasks 4 and 9 were measured on. Windows 11
Pro 10.0.26200; WSL 2 Ubuntu on the same machine, files under `/tmp` on the
Linux filesystem, unoptimised test profile. Three runs of the timed test.

| What | Run 1 | Run 2 | Run 3 |
|---|---|---|---|
| 24,000 files in 24,400 things written | 269 ms | 259 ms | 233 ms |
| `Index::of` over 24,400 things, two walks, 24,000 files opened | 431 ms | 427 ms | 451 ms |
| `Index::again` over the same, unchanged, nothing opened | 104 ms | 167 ms | 164 ms |
| the index file on the disk, 24,401 entries | 5,197,133 bytes | | |
| paths and words held in hand, bytes of string | 1,523,787 bytes | | |

Task 4's single walk of ten thousand files on the same machine indexed in
227 to 335 ms in the same runs, so walking on costs nothing per file beyond
the walk: the second walk here re-lists one folder of a hundred and one
names and no more.

**The memory number the plan asked for.** The index holds every word of
every text file as its own `String`: 1.5 MB of string bytes here cost, at
Rust's 24 bytes per `String` plus one heap allocation each, roughly 40 bytes
of overhead per word, so this index of short letters is on the order of
13 MB in hand. For real documents the words dominate: a text file at the
one-megabyte read bound is some hundred and fifty thousand words, and held
this way is five to eight times its own size. A hundred thousand short
files is a hundred and thirty megabytes; a hundred thousand long documents
is more than a person's machine should give a search. **That is a bound to
argue for in a task of its own** — words kept once per file rather than
once per occurrence, or a cap on words per entry said in `Contents` — and
this report names it rather than trimming an index quietly. No index is
partial because of memory today.

**Not claimed:** anything about a certified machine. These are the
development machine's numbers, as the plan's constraint asks.

## Verification

| Platform | Command | Result |
|---|---|---|
| WSL Ubuntu, `/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1` | `cargo fmt --all` | clean, exit 0, no file changed on the final run |
| WSL Ubuntu | `cargo clippy -p alo-finding --all-targets -- -D warnings` | clean, exit 0 |
| WSL Ubuntu | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps` | clean, exit 0 |
| WSL Ubuntu | `cargo test -p alo-finding` | 70 unit + 3 + 1 + 6 + 4 + 5 + 6 + 7 + 8 + 5 + 3 + 7 integration + 2 doc tests, all passing |
| WSL Ubuntu, as `nobody` | the `an_index_made_whole_for_a_folder_larger_than_one_walk` binary filtered to `a_subfolder_the_machine_would_not_read`, and the lib binary filtered to `walking_on::tests` | 1 and 5 passing: the gates run as root, which no permission stops, so the two unreadable-folder tests take their *this process reads anything* branch there; copied to `/tmp` and run unprivileged they take the refusal branch, and pass |

The workspace suite was not run by this worker, as the brief asks; the
supervisor runs it. Nothing outside `crates/alo-finding` and two documents
changed.

## Limitations

- A folder holding more than `MOST_WALKED` things directly inside it is
  still indexed up to that many and said to be not whole. Lifting that is a
  change to the walker in `alo-files`, outside this plan.
- The memory of a whole index in hand is bounded only by the folder; the
  number is above, and a bound is proposed as a task rather than taken.
- The two unreadable-folder tests are decided at run time by whether
  `read_dir` refuses: under root, or on a host without modes, they check
  the whole-and-nothing-unread branch instead. The unprivileged run above
  is how the refusal branch was shown, and the report says so rather than
  the tests pretending.
- The timing test writes twenty-four thousand files. On Linux that is a
  quarter of a second; on Windows it is tens of seconds of disk before
  anything is timed.

## Proposed shared-document updates

For `CHANGELOG.md`, under the unreleased v0.5 section:

> **Search your own files: an index made whole for a folder larger than one
> walk.** A folder of more than twenty thousand things is now indexed to
> the end: the index walks on from every folder one walk had not entered,
> under the same bound each time, and says it is whole; an index an earlier
> version cut short is completed the next time it is brought up to date,
> reading only the files it had not reached. A single folder holding more
> than twenty thousand things directly inside it is the one thing still
> left, and the index says so. Measured on the development machine: a
> folder of 24,400 things indexed in under half a second on Linux.

For `docs/autonomy/QUEUE.md` and `STATE.md`: task 11 of the machine-measured
plan done; task 12 written and ready — sizes made whole for a folder larger
than one walk, with one walking on for the index and the tree of sizes.
