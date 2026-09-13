# An index brought up to date by its name says how old it is

- Date: 2026-09-14
- Workstream: v0.5 the machine, measured, task 7 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

Task 6 made the list the one place that says which folders are indexed and
hands back the index for one. Two things were still missing. A caller holding
a folder's name could not bring its index up to date without reading the
index back, calling `Index::again` and keeping the result — three calls a
daemon and a file manager would each have written, which is the pair of
lists task 6 removed coming back as a pair of refresh loops. And an index did
not say when it was made, so a search over a folder indexed last Tuesday
answered as if it were now and the person had no way to tell. This task
makes the refresh one call by the folder's name, and makes every index and
every answer say when the folder was read.

## What changed

**The moment an index was made, from the caller.**
`crates/alo-finding/src/index.rs`: `Index::of(folder, made)` and
`Index::again(&self, made)` take the moment as a `std::time::SystemTime`
argument, and `Index::made` carries it as an `Option<Moment>` — `Some` for
every index this version makes, `None` only for one read back from a file
written before the moment was kept. `crates/alo-finding/src/indexing.rs`
writes it into the assembled index. Nothing in the crate reads a clock: when
an index is made is the file manager's, the daemon's or the person's
decision, and a crate that looked at the clock itself would be a step from
looking at the disk itself.

**In the file, additively.** `crates/alo-finding/src/format.rs`: the head
gains `made` as `{"secs":…,"nanos":…}`, the shape an entry's `modified`
already has, placed between `of` and `covered`. It is `#[serde(default,
skip_serializing_if = "Option::is_none")]`, so a head written before the
field still reads with no moment rather than an invented one, and an index
with no moment is written with the field left out rather than as `null` —
the file an earlier version wrote, byte for byte. `format` stays `1`: the
contract says a later field is added this way, and a unit test writes a
head of the earlier shape, reads it, and writes it back unchanged. The
existing test that checks an unknown field is ignored used `made` as its
example of an unknown field; it now uses `checked`, because `made` is known.

**An answer says how old it is.** `crates/alo-finding/src/answer.rs`:
`Answer::made` is the moment the index it answered from was made, set in
`searching.rs` from `Index::made`, so a window can put *as of Tuesday*
beside the results — and an empty answer still says when it is from.
`Searched::of` is unchanged, and the answer an agent is given through it is
the same function a person is given, moment included.

**Brought up to date by its name, in one call.**
`crates/alo-finding/src/indexed.rs`: `Indexed::again(&mut self, folder,
made)` reads the kept index through `index_of`, indexes the folder again
through `Index::again` — reading only a file whose size or time changed,
counted by `Index::opened` — and keeps the result whole through `keep`,
which writes the index in its place and leaves the list as it was because
the folder is already on it. A folder not on the list is refused with
`NeverIndexed` **before anything is read** and is not indexed for the first
time: a call meant to refresh an index that made one would be a walk nobody
asked for. A folder that is gone since it was indexed is refused with
`NotWalked` and its index is kept, byte for byte: an unplugged disk is not a
request to forget it, and the kept index still answers about the folder as
it was, saying when that was. No new refusal and no new word: both refusals
already had a sentence, a translator's note and a Polish translation, and
the moment is two numbers rather than a string.

**The shipped-source test, and the one clock.**
`crates/alo-finding/tests/nothing_here_opens_a_socket_or_asks_anybody.rs`
gains sixteen forbidden names — `inotify`, `notify`, `Watcher`, `watch`,
`thread`, `sleep`, `timer`, `Timer`, `interval`, `Interval`, `poll`,
`epoll`, `mpsc`, `channel`, `Receiver`, `recv` — every road to a crate
waking up on its own to read the disk. A new test,
`the_only_clock_is_the_stopwatch_around_a_search`, walks every shipped file
and refuses the identifier `now` anywhere but `searching.rs`, and there only
on a line that also names `Instant` — the monotonic stopwatch task 4 put
around a search so an answer can say how long it took, which decides
nothing — and counts exactly one such line. The test pinning `Index::of`'s
and `Index::again`'s signatures now pins them with the moment: a folder and
a moment, and no caller.

**The contract.** `docs/contracts/file-index.md`: the head table gains
`made`, optional, with why; a paragraph says the moment is the caller's and
not the machine's; the list section describes `Indexed::again` and says
nothing watches; *what changes additively* names `made` as the first field
added that way with `format` still `1`; the example head carries it.

**Front page.** `crates/alo-finding/src/lib.rs` gains *An index says when
it was made, and an answer says how old it is*, describes `Indexed::again`
under *Which folders are indexed*, adds two table rows, and the example
passes the caller's clock in and brings the index up to date later.

**Tests moved to the new shape.** Every `Index::of(&folder)` and `.again()`
in the five existing test files passes a fixed `noon()`; nothing else in
them changed.

## Decisions

- **`Option<Moment>` rather than `Moment`.** The plan requires an index file
  without the moment to still read. A required field would have had to
  invent a moment for such a file — the epoch, which says *as of 1970*, or
  the file's own modification time, which is the machine's clock by another
  door. `None` is the honest shape: the window says nothing rather than
  something false, and the next `Indexed::again` gives the index a moment.
- **The moment is an argument everywhere, including `Index::of`.** The
  alternative — keep `Index::of(folder)` and add a second constructor — would
  have needed `Index::of` either to read a clock, which the plan forbids, or
  to make an index with no moment, which makes the gap the normal case. The
  signature change is inside this repository: `alo-by-hand` and
  `alo-saying` use only the verb declaration and the words, and the
  workspace clippy run confirms nothing else calls it. The contract
  describes the file, not the function signatures.
- **No sentence for *as of Tuesday*.** `alo-strings` formats no dates, by
  its own design (`crates/alo-strings/src/filling.rs`); a sentence with a
  formatted date in it would have meant either a calendar in this crate,
  which is a second responsibility and the wrong time zone, or English in a
  place a translator never sees. The moment is handed to the window as two
  numbers, the way an entry's `modified` already is, and how it is spelled
  for the reader is the window's in the reader's own calendar.
- **`Indexed::again` returns the fresh `Index`.** The caller usually wants
  to search it or show `opened`; returning it costs nothing, since it is
  already in hand, and saves a second read of the file just written.
- **A refresh at a moment earlier than the last one is taken as said.** A
  test brings an index up to date at one o'clock and then again at noon, and
  the file says noon: nothing in the crate has a clock to disagree with the
  caller, and a crate that did would be reading one.
- **The stopwatch stays.** Task 4's `Instant::now()` around a search is a
  measurement of duration, not a reading of the time of day, and an
  `Answer::took` that lied would be worse than a stopwatch. The new test
  holds it to exactly that one line rather than forbidding `now` outright,
  so a `SystemTime::now()` anywhere in the crate fails the suite.
- **Next task.** Task 8 in the plan: one search over every indexed folder,
  one answer per folder in the list's order each saying which folder and
  how old, a torn index file a named refusal beside the other answers rather
  than a silent gap, the query checked once before any file is opened, and
  the verb unchanged because a cross-folder search under one grant would be
  a search of folders nobody granted.

## Acceptance, criterion by criterion

| The plan says | The test |
|---|---|
| the moment is in the first line, passed in by the caller, and noon reads back as noon | `an_index_made_at_noon_says_noon_in_its_first_line_and_in_every_answer`; `format::tests::the_file_is_one_head_line_and_one_line_per_entry_and_comes_back_whole` |
| the moment is in the file additively, and an index file without it still reads | `an_index_file_without_the_moment_still_reads_and_answers_with_none`; `format::tests::a_head_without_the_moment_still_reads_and_is_written_without_it`; `format::tests::a_field_from_a_later_version_is_ignored` |
| an `Answer` carries the moment the index it answered from was made | `an_index_made_at_noon_says_noon_in_its_first_line_and_in_every_answer`; `searching::tests::what_was_not_searched_depends_on_what_was_asked` |
| `Indexed::again` reads the kept index, reads only what changed (by `Index::opened`), and keeps the result whole in one call | `an_index_is_brought_up_to_date_by_its_name_reading_only_what_changed_and_kept_whole`; `indexing::tests::only_what_changed_is_read_and_the_reader_counts`; `a_file_unchanged_since_last_time_is_not_read_again` |
| a folder never indexed is refused with `NeverIndexed` and not indexed for the first time | `a_folder_never_indexed_is_refused_and_is_not_indexed_for_the_first_time` (read count held to the counter's own cost on Linux; no index file; list unchanged; the refusal in words) |
| a folder gone since it was indexed is refused with `NotWalked` and its index is kept | `a_folder_gone_since_it_was_indexed_is_refused_and_its_index_is_kept` (file byte for byte, list, entries, moment, and a later `forget` still works) |
| nothing watches, and the shipped-source test gains the names that would let one in | `nothing_in_the_shipped_source_opens_a_socket_or_asks_anybody` (sixteen new names) |
| nothing reads a clock: the moment is an argument | `the_only_clock_is_the_stopwatch_around_a_search`; `the_index_takes_no_account_of_who_asked`; `the_moment_is_the_callers_and_the_door_is_unchanged` |
| the list is still not a grant, and `Searched::of` is still unchanged | `the_moment_is_the_callers_and_the_door_is_unchanged` pins the door's shape; task 6's `an_indexed_folder_is_not_a_granted_one_and_a_granted_one_is_not_an_indexed_one` still passes unchanged |

## Verification

Executed, foreground, exit codes read:

| Where | Command | Result |
|---|---|---|
| Windows 11, this checkout | `cargo fmt --all` | clean |
| Windows 11 | `cargo clippy -p alo-finding --all-targets -- -D warnings` | clean |
| Windows 11 | `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, exit 0 |
| Windows 11 | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps` | clean, exit 0 |
| Windows 11 | `cargo test -p alo-finding` | 56 unit + 3 + 6 + 5 + 7 + 7 + 3 + 7 integration + 2 doc tests, all passing |
| WSL Ubuntu, `/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1` | `cargo fmt --all -- --check` | clean |
| WSL Ubuntu | `cargo clippy -p alo-finding --all-targets -- -D warnings` | clean |
| WSL Ubuntu | `cargo test -p alo-finding` | 58 unit + 3 + 6 + 5 + 7 + 8 + 3 + 7 integration + 2 doc tests, all passing; the per-thread read-count check runs live here |

Not run here, on purpose and per the task: the full workspace suite, which
the supervisor runs. Nothing in `alo-measuring`, `alo-saying`, `alo-by-hand`
or any other crate was edited; workspace clippy on all targets compiles
every one of them against the changed `alo-finding` and is clean.

One thing found while testing, already known to the crate: on NTFS a
folder's own modification time settles a moment after a write inside it, so
two walks of one folder made back to back can differ in a folder entry's
`modified`. The new tests compare the file entries of two walks, as
`indexing.rs`'s own test does; the index's incremental rule never compared
folders' times, so nothing in the shipped code changed for this.

## Limitations

- `Option<Moment>` is `None` for an index file written before this change.
  It becomes `Some` the next time that folder is indexed or brought up to
  date; nothing rewrites old files unasked, because that would be a read of
  the disk nobody asked for.
- The known gap of the incremental rule is unchanged: a file rewritten with
  the same byte count within the filesystem's time resolution is not seen.
- The moment is the caller's word. A caller that passes a wrong moment
  writes a wrong moment; the crate cannot check it without a clock, and
  having no clock is the point.

## Proposed shared-document updates

For `CHANGELOG.md`, under the unreleased v0.5 section:

> **Search your own files: an index says when it was made, and is brought up
> to date by its folder's name.** Every index now records, in its first
> line, the moment it was made — passed in by whoever made it, never read
> from a clock inside the crate — and every answer from it carries that
> moment, so a window can say *as of Tuesday* beside the results. An index
> file written before this still reads, with no moment rather than an
> invented one. A folder's index can be brought up to date in one call by
> its name, reading only the files that changed; a folder that was never
> indexed is refused rather than indexed by a call meant to refresh one, and
> a folder that is gone since is refused with its index kept, because an
> unplugged disk is not a request to forget it. Nothing watches a folder:
> there is no `inotify`, thread or timer in the crate, and a test reads its
> source to say so.

For `docs/autonomy/QUEUE.md` and `STATE.md`: task 7 of the machine-measured
plan done; task 8 written and ready.
