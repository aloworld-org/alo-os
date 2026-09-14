# The indexes read once and asked many times, and a search over every folder timed

- Date: 2026-09-14
- Workstream: v0.5 the machine, measured, task 9 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

Task 8 made one search over every indexed folder one call, and every call
read every index file from the disk again. A person typing *contract* into
a search box fires a query per keystroke, and three folders of ten thousand
entries are three files parsed per keystroke; nobody had measured what that
costs, and a file manager that decided to keep the indexes between
keystrokes would have written its own set of them, its own refresh of one
by name, and its own memory of which index file would not read. This task
measures the cost on the development machine and offers the set: the
indexes read once and held in hand, asked many times from memory, one
folder brought up to date by its name through the set, and the disk's
refusals held in the same place until the caller reads again.

## What changed

**The set in hand.** `crates/alo-finding/src/in_hand.rs`, new:
`InHand::of(&Indexed)` — and `Indexed::in_hand()`, the same as a method —
reads every folder's index from its file the way `Indexed::index_of` does,
never by walking, and holds one place per folder in the list's order: the
index, or the `NotIndexed` that stood where it would be. `InHand::answer`
puts a query to every held place and hands back an `Everywhere`, the shape
`Indexed::answer` answers in, opening no file; a query that is not one is
refused with `NotAsked` before anything is searched. `InHand::again` is
`Indexed::again` on the list the set was read from — the kept index read,
the folder indexed again reading only what changed, the result kept whole —
with the fresh index put in the set's place for that folder, whether that
place held an index or a refusal; no other folder's file is read, and a
refusal leaves the set as it was, as it leaves the disk. `InHand::index_of`
hands back one folder's held index, or the held refusal, from memory;
`InHand::each`, `InHand::folders` and `InHand::indexed` say what is held
and which list it was read from. The set owns a copy of the list it was
read from, so it is what the disk said at one moment, and reading again is
`Indexed::in_hand()` again — the caller's decision.

**One place answers for a folder, from the disk or from hand.**
`crates/alo-finding/src/everywhere.rs`: the per-folder step of
`Indexed::answer` is now `pub(crate) of_folder(folder, Result<&Index,
&NotIndexed>, query) -> OfFolder`, and both forms go through it, so an
answer from hand is the answer from the disk in every respect but where the
index came from. `Everywhere` and `OfFolder` are now `Clone`, `PartialEq`
and `Eq`.

**A refusal that can be handed back.** `crates/alo-finding/src/refusing.rs`:
`NotIndexed` derives `Clone`, `PartialEq` and `Eq`. Every field already was
— `alo_files::Failed` included — and a set that holds a refusal in a
folder's place has to hand that refusal back beside every answer, the same
each time. Additive: nothing that held a `NotIndexed` before changes.

**Registered and described.** `crates/alo-finding/src/lib.rs` declares the
module, exports `InHand`, gains the section *The indexes read once and
asked many times*, a table row, and the example asks the set between
keystrokes. `crates/alo-finding/src/indexed.rs` gains a section pointing at
the set. `docs/contracts/file-index.md` describes the set under *The list
of indexed folders*: what is read and when, the refusal held in place, the
refresh through the set, and that nothing decides when to read again but
the caller.

**No new word, no new refusal, no new dependency.** The shipped-source test
reads the new file with every other and is unchanged: nothing new names
the capability model, the network, a clock, a thread or a watcher, and the
manifest is as it was.

## Decisions

- **`InHand`, and `Indexed::in_hand`.** The plan's phrase is *held in
  hand*, and `Held` was already taken by task 8 for an answer held apart
  from its index. `Loaded` and `Cached` were the alternatives; *cached*
  says that something decides when the cache is stale, and nothing here
  does — the caller reads again when the caller chooses — so the name that
  makes no such promise is the one used.
- **The set owns a copy of the list, rather than borrowing it.** A set
  that borrowed `&mut Indexed` for its lifetime would keep a file manager
  from touching its own list while it held a search; a set that borrowed
  `&Indexed` could not bring a folder up to date through `Indexed::again`,
  which takes `&mut self`. The list is two vectors of paths, and the
  module documentation on `Indexed` already says that two of these read at
  different moments can disagree the way two readings of any file can. So
  the set is a snapshot of the list and of the indexes, read at one
  moment, with `InHand::indexed()` saying which list that was; a folder
  kept or forgotten on the disk after the set was read is not seen until
  the caller reads again — which task 10, written below in the plan, takes
  up, because *forget* in particular should not leave words in a memory
  the person asked to have gone.
- **`InHand::again` goes through `Indexed::again`, as the plan says, and
  so reads the folder's own index file once.** The alternative — indexing
  again from the held index and keeping the result — would read one file
  fewer, but would refresh from what the set holds rather than from the
  disk's word, so a folder whose file was torn when the set was read could
  never be brought up to date through the set; and it would be a second
  refresh with its own rules beside the disk's. Through `Indexed::again`
  the refusals are the disk's — never indexed, not named from the root,
  the kept file torn, the folder gone — and the test shows each leaving
  the set as it was. The cost is one index file read per refresh, which
  the read-count test shows equals what `Indexed::again` costs on its own.
- **`NotIndexed` is `Clone` and `PartialEq`.** Task 8 chose not to make
  `OfFolder` either, saying a refusal is a thing that happened. It still
  is; a copy of what the machine said is still what the machine said, and
  a set that holds a refusal in a folder's place and answers with it many
  times has to hand out copies. `PartialEq` lets a test say *the same
  refusal each time* with one assertion. Both are additive.
- **The disk form is held to no bound; the held form is held to three
  times task 4's.** The cost of reading the files is the thing being
  measured, and a bound nobody had measured first would be a claim. The
  held form answers from memory and is held to task 4's bounds three
  times over — by name under 300 ms, by contents under 3 s — and the best
  held round may not be slower than the best disk round, because a set in
  hand slower than reading the files would be one nobody should hold.
  Both forms are held to finding the same files.
- **Next task.** Task 10 in the plan: a folder kept or forgotten in hand
  and on the disk in one call through the set, so that a forgotten
  folder's words are gone from memory as they are from the disk without
  the file manager having to remember to drop its set.

## Acceptance, criterion by criterion

| The plan says | The test |
|---|---|
| a test builds three indexes of ten thousand files, times `Indexed::answer` by name and by contents, and the numbers are in the report with the machine named; the held form is timed beside the file-reading one, and both numbers are published | `a_search_over_every_folder_timed::a_search_over_three_folders_of_ten_thousand_files_is_timed_from_the_disk_and_from_hand` (thirty thousand real files, three rounds each way, both forms finding the same files; the numbers below) |
| the list's indexes read from their files once and held in hand, answering any number of queries over every folder from memory in the same shape `Indexed::answer` answers, with a test counting reads that shows the second query opens no file | `the_indexes_read_once_and_asked_many_times::the_second_query_opens_no_file_and_answers_what_the_disk_answered` (reading the set costs at least three reads; three rounds of two queries each cost exactly what reading the counter costs; each equal to `Indexed::answer` but for the timing; with the folders and index files gone the disk refuses and the set still answers) |
| a folder whose index file would not read is the same named refusal, held in the same place beside the others, until the caller reads again | `the_indexes_read_once_and_asked_many_times::a_folder_whose_index_file_would_not_read_is_the_same_refusal_in_the_same_place_until_the_caller_reads_again` (torn → `NotAnIndex` naming the file at the torn folder's place, equal on every query and to the disk's; mended on the disk, the set still refuses while the disk answers; read again, it answers; gone and another folder's held the same way) |
| one folder is brought up to date by its name through the held set in one call — in hand and on the disk, through `Indexed::again` — without the other folders being read again, checked by the read count | `the_indexes_read_once_and_asked_many_times::one_folder_is_brought_up_to_date_by_its_name_without_the_others_being_read_again` (a file added and the other two index files torn after the set was read; the refresh finds the file, in hand and in the file on the disk, `opened == 1`; the other two still answer from hand where read again they would refuse; with nothing changed the refresh through the set costs exactly the reads `Indexed::again` costs alone, and fewer than reading the set) |
| the refusal paths of the refresh through the set | `the_indexes_read_once_and_asked_many_times::a_refusal_to_bring_a_folder_up_to_date_leaves_the_set_as_it_was` (never indexed with no file read; not named from the root; the folder gone; the kept file torn after the set was read — each leaving the set equal to what it was; then mended and refreshed, and a place that held a refusal holding the fresh index) |
| nothing ranks, `Indexed::answer` stays as it is, the verb is unchanged, the list is still not a grant | `the_indexes_read_once_and_asked_many_times::nothing_ranks_the_disk_is_still_asked_every_time_and_the_list_is_still_not_a_grant` (the list's order and the index's own from hand; `Indexed::answer` reading three files on every call; no grant and the set answers while an agent over one of its folders is refused `NotGranted`; four shapes pinned by assignment) |
| a set with nothing to read holds a refusal in every place, and an empty list is an empty set | `in_hand::tests::a_set_with_no_files_to_read_holds_a_refusal_in_every_place`, `in_hand::tests::an_empty_list_is_an_empty_set` |
| nothing opens a socket, reads a clock or watches a folder | `nothing_in_the_shipped_source_opens_a_socket_or_asks_anybody`, `the_only_clock_is_the_stopwatch_around_a_search` and `only_the_verb_and_its_door_name_the_capability_model`, unchanged, now reading the new file |

## The timings, with the machine named

**Machine:** Dell Latitude 5550, Intel Core Ultra 7 155U, 15 GiB, SK hynix
PVC10 NVMe 512 GB — the machine task 4's numbers were measured on. Windows
11 Pro 10.0.26200; WSL 2 Ubuntu on the same machine. Unoptimised test
profile. Three folders of ten thousand files each in a hundred subfolders
(nine thousand text files of about thirty words, one thousand PDFs), each
indexed by `Index::of` and kept on one list by `Indexed::keep` in the test.
*By name* is `letter-0999`, ten per folder, thirty found; *by contents* is
`Anna contract summer`, ninety per folder, two hundred and seventy found.
Each number is the test's clock around the whole call, three rounds, round
one cold.

WSL 2 Ubuntu, files under `/tmp` on the Linux filesystem:

| Search over three folders | Round 1 | Round 2 | Round 3 | Bound |
|---|---|---|---|---|
| from the disk, by name (`Indexed::answer`) | 409 ms | 413 ms | 398 ms | none, measured |
| from the disk, by contents (`Indexed::answer`) | 362 ms | 477 ms | 445 ms | none, measured |
| from hand, by name (`InHand::answer`) | 11.0 ms | 5.4 ms | 5.1 ms | 300 ms |
| from hand, by contents (`InHand::answer`) | 10.7 ms | 12.2 ms | 11.2 ms | 3 s |

The three indexes read into hand once in 454 ms. Ten thousand files
written in 75–102 ms, indexed in 237–275 ms, and kept in 114–151 ms, per
folder.

Windows 11, files under the Windows temporary directory:

| Search over three folders | Round 1 | Round 2 | Round 3 | Bound |
|---|---|---|---|---|
| from the disk, by name | 657 ms | 603 ms | 575 ms | none, measured |
| from the disk, by contents | 647 ms | 679 ms | 536 ms | none, measured |
| from hand, by name | 11.7 ms | 9.1 ms | 9.2 ms | 300 ms |
| from hand, by contents | 13.8 ms | 13.3 ms | 12.0 ms | 3 s |

The three indexes read into hand once in 565 ms. Ten thousand files
written in 10–12 s, indexed in 2.0–2.2 s, and kept in 200–267 ms, per
folder — the disk, not the search, and the test is not timing it.

**What the numbers say.** Reading three index files of ten thousand
entries costs about four tenths of a second on Linux and six on Windows,
every query, and almost all of it is the read and the parse: the search
itself, from hand, is five to thirteen milliseconds over the same three
folders, forty to eighty times less. A search box that read the files per
keystroke would answer a person's eight-letter word in three seconds of
disk on Linux; from hand, in a tenth. The cost of reading the set once is
the cost of one disk query.

**Not claimed:** anything about a certified machine. These are the
development machine's numbers, as the plan's constraint asks. The test
holds the held form to three times task 4's bounds and to being no slower
than the disk form; it holds the disk form to no bound, because its cost
was the thing to be measured.

## Verification

Executed, foreground, exit codes read:

| Where | Command | Result |
|---|---|---|
| Windows 11, this checkout | `cargo fmt --all` | clean |
| Windows 11 | `cargo clippy -p alo-finding --all-targets -- -D warnings` | clean, exit 0 |
| Windows 11 | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps` | clean, exit 0 |
| Windows 11 | `cargo test -p alo-finding` | 60 unit + 3 + 1 + 6 + 5 + 6 + 7 + 7 + 5 + 3 + 7 integration + 2 doc tests, all passing; the read-count clauses skip where the kernel keeps no per-thread count |
| WSL Ubuntu, `/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1` | `cargo fmt --all -- --check` | clean, exit 0 |
| WSL Ubuntu | `cargo clippy -p alo-finding --all-targets -- -D warnings` | clean, exit 0 |
| WSL Ubuntu | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps` | clean, exit 0 — after one private intra-doc link was reworded |
| WSL Ubuntu | `cargo test -p alo-finding` | 62 unit + 3 + 1 + 6 + 5 + 6 + 7 + 8 + 5 + 3 + 7 integration + 2 doc tests, all passing; the per-thread read-count checks run live here |

Not run here, on purpose and per the task: the full workspace suite, which
the supervisor runs. Nothing outside `crates/alo-finding` and the two
documents was edited.

## Limitations

- A set in hand is a snapshot of the list too. A folder kept or forgotten
  through `Indexed` after the set was read is not seen by the set until
  the caller reads again — and for *forget* that means a memory the person
  asked to have gone stays in the caller's hand until then. Task 10, now in
  the plan, gives both a road through the set.
- The set holds every index whole. Three folders of ten thousand entries
  with their words are a few tens of megabytes in memory; that is the size
  of the honest set, and a caller that cannot afford it keeps asking the
  disk through `Indexed::answer`, which is unchanged.
- `InHand::again` reads the folder's own index file from the disk before
  indexing again, so that the refresh is the disk's and a torn file can be
  mended by it; one file read per refresh is the price, shown equal to
  `Indexed::again`'s own.
- The timing test writes thirty thousand files. On Windows that is about
  half a minute of disk before anything is timed; on Linux under a second.

## Proposed shared-document updates

For `CHANGELOG.md`, under the unreleased v0.5 section:

> **Search your own files: the indexes read once and asked many times.** A
> search box that asks on every keystroke no longer has to read every
> index file from the disk each time: the indexes a person asked for can
> be read once and held, and asked any number of times from memory, with
> the same answers folder by folder — a folder whose index file could not
> be read is named in its place on every answer until the caller reads
> again, and one folder can be brought up to date by its name through the
> held set without the others being read. Measured on the development
> machine over three folders of ten thousand files: reading the files per
> query costs about four tenths of a second on Linux; from hand, about ten
> milliseconds. Asking the disk every time is still there, unchanged,
> for a caller that wants it.

For `docs/autonomy/QUEUE.md` and `STATE.md`: task 9 of the machine-measured
plan done; task 10 written and ready.
