# One search over every indexed folder, each answer saying which folder and how old

- Date: 2026-09-14
- Workstream: v0.5 the machine, measured, task 8 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

A person's search box is not a folder's. Task 6 made the list the one
place that says which folders are indexed, and task 7 made every answer
say how old it is; what nobody could yet do was type *contract* once and be
answered from every folder they asked to have indexed. A file manager would
have read the list, read each index, asked each, and stitched the answers
together — and a search stitched together by each caller is a search whose
honesty depends on the caller: one that skipped a folder whose index file
would not read, and said nothing, has answered *nothing matched* about a
folder it never looked at. This task makes that search one call, with the
folder that would not answer a named refusal beside the others.

## What changed

**The search over every folder.** `crates/alo-finding/src/everywhere.rs`:
`Indexed::answer(&self, query)` checks the query once, then for every
folder on the list in the list's order reads its index from its file the
way `Indexed::index_of` does — never by walking the folder — and puts the
query to it. It hands back an `Everywhere`: one `OfFolder` per folder,
each carrying the folder as it is on the list and `answered: Result<Held,
NotIndexed>` — the answer held apart from its index, or the refusal that
stood where the answer would be. `Everywhere::answered()` and
`Everywhere::refused()` walk the two halves in the list's order, and
`every_folder_answered()` says whether there was a refusal at all. An empty
list answers with no folders and no refusal. Nothing ranks: across folders
the order is the list's, within one the index's own.

**An answer that outlives its index.** `crates/alo-finding/src/held.rs`:
`Held` is what an `Answer` says — what matched, what was not searched, the
moment the index was made, how long it took — owned, with `Unsearched` the
owned shape of `NotSearched`. `Held::of(&Answer)` copies the borrowed
entries out, and `Held::answer()` and `Unsearched::borrowed()` give the
borrowing view back, so that `NotSearched::said` and
`NotSearched::is_nothing` stay the one piece of code that says and answers
those things. A unit test holds the round trip to equality and the
sentences to the same sentences.

**The search pass, apart from the check.**
`crates/alo-finding/src/searching.rs`: `answered` is now `checked` followed
by a new `pub(crate) searched(index, query) -> Answer`, so that the search
over every folder checks the query once and puts it to each index without
a second refusal that could never happen having to be handled anyway.
`Index::answer` is unchanged in what it does.

**Registered and described.** `crates/alo-finding/src/lib.rs` declares the
two modules, exports `Everywhere`, `OfFolder`, `Held` and `Unsearched`,
gains the section *One search over every indexed folder*, two table rows,
and the example searches every folder. `crates/alo-finding/src/indexed.rs`
gains the method and a section in its module documentation.
`docs/contracts/file-index.md` describes the search under *The list of
indexed folders*: the shape of the answer, the refusal beside the others,
the check before the first file, the empty list, the order, and that it is
not a verb.

**No new word, no new refusal.** Every refusal the search can put beside an
answer — `NotOpened`, `NotAnIndex`, `NotTheSame` — already had a sentence
and a Polish translation; the query refusals are `NotAsked`'s three. The
shipped-source test reads the two new files with every other and is
unchanged: nothing new names the capability model, the network, a clock, a
thread or a watcher.

## Decisions

- **`Indexed::answer`, as the plan suggested.** It is the name the crate
  already uses for *put a query to this*: `Index::answer` for one folder,
  `Indexed::answer` for every folder on the list. The result type is
  `Everywhere` rather than a `Vec`, so that the two halves — what answered
  and what refused — can be walked without every caller writing the same
  `filter_map`.
- **An owned answer rather than a self-borrowing one.** `Answer<'a>`
  borrows the `Index` it came from, which is the right shape for a caller
  holding an index; here each index is read inside the call and gone when
  it returns, so what comes back has to stand on its own. The alternatives
  were a self-referential struct, which needs a crate this crate's
  dependency list is held to exactly six without, or handing the caller
  the indexes and the loop back, which is the loop the task removes. `Held`
  copies the matched entries and the unsearched lists; the copy is of what
  answered, not of the index, and the borrowing view keeps every sentence
  in one place.
- **`OfFolder` is neither `Clone` nor `PartialEq`**, because `NotIndexed`
  is neither: it carries what the machine said, which is a thing that
  happened. `Held` and `Unsearched` are both, so a test can compare two
  answers field by field — and does, with `took` left out, because two
  searches timed separately are two measurements.
- **The refusal names the file, not the folder's absence.** A torn index
  file is `NotAnIndex { at }` with the file's path, a missing one is
  `NotOpened { at }`, and another folder's index copied over it is
  `NotTheSame { asked, indexed }` — the three `Index::read_from` already
  refuses with. The folder stays on the list and in the answers, at its
  place, so a window shows the folder with its refusal beside the folders
  with their results.
- **The query is checked before the list is looked at**, so a query that is
  not one is refused over an empty list too. The check is the same
  `asking::checked` that `Index::answer` uses.
- **Not a verb, and no new door.** An agent's `search_files` still names
  one granted folder and takes one index through `Searched::of`; a test
  shows the same three indexed folders answering with no grant at all
  while an agent's call over one of them is refused at the door. If a
  cross-folder verb is ever wanted it is a verb with a grant per folder,
  declared in its own task.
- **Next task.** Task 9 in the plan: the cost of reading every index file
  on every query measured with the machine named, and the indexes read
  once and held in hand for a caller — a file manager between keystrokes —
  that asks many times, brought up to date one folder at a time by name,
  with the same refusal held in the same place.

## Acceptance, criterion by criterion

| The plan says | The test |
|---|---|
| one `Query` over every folder on the list in one call, one answer per folder in the list's order, each carrying the folder, what matched, what was not searched and the moment its index was made, read from each index's file and never by walking | `one_query_is_answered_from_every_folder_in_the_lists_order_each_saying_which_and_how_old` (three folders kept out of alphabetical order at noon, one and two o'clock; each answer compared to the folder's own `Index::answer`; the folders removed and the answers unchanged) |
| a folder whose index file could not be read or is not an index is a named refusal beside the other answers: one torn of three still gives two answers and one named refusal | `a_folder_whose_index_file_would_not_read_is_a_named_refusal_beside_the_other_answers` (torn → `NotAnIndex` naming the file, in words; gone → `NotOpened`; another folder's → `NotTheSame`; the other two answers as before; the list untouched) |
| a query that is not one is refused once, before any index file is opened, checked by the read count | `a_query_that_is_not_one_is_refused_once_before_any_index_file_is_opened` (four non-queries, the per-thread read count held to the counter's own cost on Linux; a query that is one shown to read the three files) |
| an empty list answers with no folders and no refusal | `an_empty_list_answers_with_no_folders_and_no_refusal` |
| nothing ranks across folders, and within a folder the order is the index's own | `nothing_ranks_across_folders_or_within_one` (fewest matches first on the list, most last, alphabet disagreeing with both; a refresh does not move a folder; within a folder equal to `Index::find`) |
| the verb is unchanged, and the list is still not a grant | `the_search_over_every_folder_is_not_a_verb_and_the_list_is_still_not_a_grant` (no grant, every folder answers; an agent over one of them refused `NotGranted`; `Searched::of` and `Indexed::answer` pinned by shape) |
| a held answer is the answer it was made from | `held::tests::a_held_answer_is_the_answer_it_was_made_from`; `held::tests::a_held_answer_over_everything_still_says_nothing_was_left_out` |
| nothing opens a socket, reads a clock or watches a folder | `nothing_in_the_shipped_source_opens_a_socket_or_asks_anybody`, `the_only_clock_is_the_stopwatch_around_a_search` and `only_the_verb_and_its_door_name_the_capability_model`, unchanged, now reading the two new files |

## Verification

Executed, foreground, exit codes read:

| Where | Command | Result |
|---|---|---|
| Windows 11, this checkout | `cargo fmt --all` | clean |
| Windows 11 | `cargo clippy -p alo-finding --all-targets -- -D warnings` | clean, exit 0 |
| Windows 11 | `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, exit 0 |
| Windows 11 | `cargo test -p alo-finding` | 58 unit + 3 + 6 + 5 + 6 + 7 + 7 + 3 + 7 integration + 2 doc tests, all passing |
| WSL Ubuntu, `/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1` | `cargo fmt --all -- --check` | clean, exit 0 |
| WSL Ubuntu | `cargo clippy -p alo-finding --all-targets -- -D warnings` | clean, exit 0 |
| WSL Ubuntu | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps` | clean, exit 0 |
| WSL Ubuntu | `cargo test -p alo-finding` | 60 unit + 3 + 6 + 5 + 6 + 7 + 8 + 3 + 7 integration + 2 doc tests, all passing; the per-thread read-count checks run live here |

Not run here, on purpose and per the task: the full workspace suite, which
the supervisor runs. Nothing outside `crates/alo-finding` and the two
documents was edited; workspace clippy on all targets compiles every crate
against the changed one and is clean.

## Limitations

- Every call of `Indexed::answer` reads every index file from the disk
  again. That is the honest default — the disk's word, every time — and
  its cost is unmeasured; task 9 measures it and offers the indexes held
  in hand for a caller that asks many times.
- `Held` copies the matched entries. For a query by contents over a folder
  of ten thousand PDFs the `no_reader` list is ten thousand copied entries;
  that is the size of the honest answer, not an overhead beside it, but a
  window that only wants the count has the count in `len()`.
- Two folders on the list, one inside the other, are two indexes and answer
  twice about the files under both. The list does not refuse a nested
  folder, and neither did task 6; whether it should is a decision for the
  surface that offers *index this folder*, not for the search.

## Proposed shared-document updates

For `CHANGELOG.md`, under the unreleased v0.5 section:

> **Search your own files: one search over every indexed folder.** A
> query is now put to every folder a person asked to have indexed in one
> call, and answered folder by folder in the order they asked for them —
> each answer saying which folder it is of, what matched, what was not
> searched, and when that folder's index was made. A folder whose index
> file could not be read is named beside the other answers rather than
> silently left out, so *nothing matched* is never said about a folder
> nobody looked at. The query is checked before any file is opened, an
> empty list answers with nothing to search, and nothing ranks. An agent's
> `search_files` is unchanged: it still names one granted folder.

For `docs/autonomy/QUEUE.md` and `STATE.md`: task 8 of the machine-measured
plan done; task 9 written and ready.
