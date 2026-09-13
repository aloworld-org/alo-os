# Search answers without asking anything, and says what it did not read

- Date: 2026-09-13
- Workstream: v0.5 the machine, measured, task 4 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

The second half of *search your own files, without asking anything*: a
search that answers in the time a person will wait for one, and is honest
about what the index does not hold — a file it could not read, a kind it
cannot open, a directory the person never asked to index. Task 3 built the
index; this task builds the answer on top of it, and measures it.

## What changed

**`Index::answer` is the search a person or an agent is given.**
`Index::find` stays exactly what it was — the filter, in the index's own
order — and `Index::answer(&Query)` sits in front of it: it refuses a query
that is not one, times the search, filters, and comes back with an `Answer`
that is what matched **beside** what was not searched. Nothing touches the
disk; every list in the answer is read from the index, including what the
walk never reached, which task 3 wrote into the index's head line for this.

**An `Answer` is `found`, `not_searched` and `took`.** `NotSearched` holds,
in this order: `outside`, the folder the index is of, so nothing beyond it
is mistaken for searched; `folders_unread`, the folders the machine would
not let the index read, each with what it said; `elsewhere`, the folders on
another filesystem the index did not enter; `not_entered`, the folders the
index stopped in at its bound; `unnamed`, how many things were left out for
names a screen cannot show; and three lists of files — `files_unread`,
`no_reader`, `too_big` — that depend on what was asked. `is_nothing` says
whether everything was searched, so an empty answer over an index that
reached everything is *nothing matched*, and over one that did not is
*nothing matched, and here is what was not looked at*. `said(&Strings)`
turns the whole of it into sentences a window shows beside the results, in
the person's language: the folder first, then one sentence per folder and
per unread file, then one count each for the kinds with no reader and the
files too large.

**A query that is not a query is refused before anything is searched.**
`NotAsked::Nothing` for a query no part of which asks for anything — an
empty name, no words, and no kind or date either; `MoreThanASentence` for
more than `A_SENTENCE` words; `LongerThanAName` for a name part or a single
word longer than `A_NAME` characters. Each is a sentence in `words.rs`, and
the refusal is the whole of the answer rather than everything in the index
under a warning.

**Words.** Ten new strings under `finding.`: the three refusals, five
sentences for what was not searched, and two countable sentences for the
files not searched by their words. The Polish test renders every one at
one, a few and many. `alo-saying` needed nothing: it counts each crate's
words at run time, and its suite is green with the thirty-five.

**Three new files, one responsibility each.** `asking.rs` — the two bounds
and the refusal; `answer.rs` — the `Answer` and `NotSearched` types and
their sentences; `searching.rs` — the one pass over the entries that sorts
each into found, not found, or not searched. `query.rs` gained three
accessors (`name_asked`, `kind_asked`, `asks_kind_or_date`) so the check can
read what was asked without the parts becoming public fields. `index.rs`
gained `answer`; `lib.rs` the exports and a section of the crate's
front page.

## Decisions

**What counts as *not searched* depends on what was asked.** A name and a
date come from the walk, not from the bytes: a PDF searched by name *was*
searched, and listing it as unsearched would make every answer over a
folder of photographs say most of the folder was skipped. So the three file
lists are filled only when the query has a part those files cannot answer:
`no_reader` and `too_big` for a search by words, `files_unread` for a search
by words or by kind, because a file whose bytes were never seen can answer
neither *is it a PDF* nor *does it say contract*. What the walk never
reached is listed whatever was asked, because nothing was known about it at
all. One exception, decided here: a query for `Kind::Unread` is asking for
exactly the unread files, so they are found rather than listed as
unsearched for that one kind.

**A file set aside as not searched is not also matched.** The pass either
holds every part of the query against an entry or sets it aside; an entry
never appears in both lists, so a person reading the two lists reads each
file once.

**The bounds are a sentence and a name.** `A_SENTENCE` is thirty-two words:
more than anybody types to find a file, fewer than a paragraph pasted into
the wrong box. `A_NAME` is two hundred and fifty-five characters, the
longest name every filesystem this crate walks allows, so a part longer
than that is in no name; it applies to the name part and to each word, and
is counted in characters rather than bytes so that a Polish word is not
shorter than an English one. Neither is read from the environment.

**The same words twice are the same words.** A query's words are the set
task 3 defined — each once, lowered — so *the the the …* is one word and is
not refused as a paragraph. The bound is on what would be looked up.

**The answer carries its own time, and the test does not trust it.** `took`
is `Instant`-measured around the search alone, so a window can show it and
a report can publish it. The timing test also times the call from outside
and holds the slower of the two to the bound, so the answer cannot flatter
itself.

**No `--release`, no warm-up.** The numbers below are from the unoptimised
test profile the suite runs in, and round one of each run is the cold one.
A released build is faster, not slower; publishing the slower number is the
honest one.

**`Index::find` is kept.** Task 5 declares the verbs and may want the bare
filter; and removing a public function a sibling crate could already call
is a break, which the contract rules forbid without versioning.

## Acceptance criteria and the tests that hold them

| Acceptance | Test |
|---|---|
| a query over ten thousand files answers by name and kind in under a tenth of a second and by contents in under a second, measured by building the index and timing the answer | `a_search_answers_in_time_and_says_what_it_did_not_read::a_query_over_ten_thousand_files_answers_in_the_time_a_person_will_wait` |
| the answer lists what was not searched as a separate list beside the results, read from the index and not the disk, with the folder outside which nothing was searched | `a_search_answers_in_time_and_says_what_it_did_not_read::the_answer_lists_what_it_did_not_search_beside_what_it_found` |
| a query that is not a query — empty, or longer than a sentence — is refused in words, and nothing is searched | `a_search_answers_in_time_and_says_what_it_did_not_read::a_query_that_is_not_a_query_is_refused_in_words` |
| what was not searched depends on what was asked: by name nothing, by words the kinds with no reader and the files too large, by kind the files never opened; what the walk never reached, always | `searching::tests::what_was_not_searched_depends_on_what_was_asked` |
| an empty answer is *nothing matched* and never *nothing was looked at* | `searching::tests::an_empty_answer_is_nothing_matched_and_never_nothing_was_looked_at` |
| the refusal is the whole of the answer, not everything under a warning | `searching::tests::a_query_that_is_not_one_is_refused_and_not_answered_with_everything` |
| nothing asked is refused, however the query was emptied; a kind or a date alone is a question | `asking::tests::a_query_that_asks_nothing_is_refused_and_one_with_a_kind_or_date_is_not` |
| more than a sentence is refused at exactly the bound | `asking::tests::more_than_a_sentence_is_refused_at_the_bound` |
| longer than a name is refused in the name part and in a word, in characters | `asking::tests::longer_than_a_name_is_refused_in_a_name_part_and_in_a_word` |
| every sentence beside an answer is said as a path a person can open, or a count | `answer::tests::what_was_not_searched_is_said_as_paths_and_counts` |
| every new sentence is read in the person's language, with Polish's three forms | `what_this_crate_says::every_sentence_is_read_in_the_language_the_person_reads` |
| and in English on a machine with no translations, with nothing a key | `what_this_crate_says::a_machine_with_no_translations_still_says_everything_in_english` |
| the query says what was asked, in the form it is looked up in | `query::tests::each_part_says_whether_it_was_asked` |
| the crate's list is thirty-five strings, each declared once | `words::tests::the_list_declares_into_a_vocabulary_once` |

Nothing ranks: `searching.rs` is one pass in index order, and the existing
`nothing_here_opens_a_socket_or_asks_anybody` test still reads the shipped
source — the three new files included — and finds no socket, no model, no
record and no environment.

## The timings, with the machine named

**Machine:** Dell Latitude 5550, Intel Core Ultra 7 155U, 15 GiB, SK hynix
PVC10 NVMe 512 GB. Windows 11 Pro 10.0.26200; WSL 2 Ubuntu on the same
machine. Unoptimised test profile, ten thousand files in a hundred folders
(nine thousand text files of about thirty words, one thousand PDFs), the
index built from the disk by `Index::of` in the test, three rounds, round
one cold. Each number is the slower of the answer's own clock and the
test's clock around the call.

WSL 2 Ubuntu, files under `/tmp` on the Linux filesystem:

| Query | Round 1 | Round 2 | Round 3 | Bound |
|---|---|---|---|---|
| by name (`letter-0999`, 10 found) | 4.4 ms | 3.7 ms | 6.4 ms | 100 ms |
| by kind (PDF, 1000 found) | 0.14 ms | 0.12 ms | 0.17 ms | 100 ms |
| by contents (`Anna contract summer`, 90 found) | 6.9 ms | 10.6 ms | 11.9 ms | 1 s |
| by name, kind and contents (9000 found) | 6.2 ms | 29.1 ms | 9.0 ms | 1 s |

Ten thousand files written in 116 ms and indexed in 440 ms.

Windows 11, files under the Windows temporary directory:

| Query | Round 1 | Round 2 | Round 3 | Bound |
|---|---|---|---|---|
| by name (10 found) | 4.2 ms | 3.2 ms | 2.8 ms | 100 ms |
| by kind (1000 found) | 0.25 ms | 0.18 ms | 0.12 ms | 100 ms |
| by contents (90 found) | 7.9 ms | 5.1 ms | 4.8 ms | 1 s |
| by name, kind and contents (9000 found) | 9.2 ms | 5.7 ms | 10.3 ms | 1 s |

Ten thousand files written in 10.4 s and indexed in 2.5 s — the disk, not
the search, and the test is not timing it.

**Not claimed:** anything about a certified machine. These are the
development machine's numbers, as the plan's constraint asks, and they are
between ten and a hundred times inside the bound, which is why the test
holds the bound rather than a tighter one.

## Verified

WSL Ubuntu on this development machine, as root, `CARGO_TARGET_DIR` the
supervisor's own (`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`), foreground,
exit codes read:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (workspace) | clean, 0 warnings |
| `cargo clippy -p alo-finding -p alo-saying --all-targets -- -D warnings` | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps` | clean |
| `cargo test -p alo-finding` | 42 unit, 3 + 3 + 8 + 3 integration, 1 doctest: all passed |
| `cargo test -p alo-saying` | 63 + 4 + 1 passed |
| every evidence line, alone, `--exact` | 1 passed each, 14 lines |

Windows, this development machine: `cargo fmt --all`; `cargo clippy -p
alo-finding --all-targets -- -D warnings` clean; `cargo test -p alo-finding`
40 + 3 + 3 + 7 + 3 + 1 passed (the two Unix-only tests do not exist there).

**Not run by this worker:** the full workspace suite, per the task's
instruction; the supervisor runs it.

## Remaining limitations

- **The search is a scan.** Ten thousand entries answer in milliseconds
  without an inverted index, and a structure that answered a million would
  be a second format to describe in the contract. The bound the plan set is
  met a hundred times over; when a folder that is not is found, that is the
  change, and this is the measurement that says when.
- **A word inside a PDF or an office document is still not searched**, and
  the answer now says so — one counted sentence per search by words. A
  reader for each is task 3's open limitation, unchanged.
- **The folder's own time is not a search axis** — the two NTFS-aware tests
  from task 3 stand.
- **Nothing here is a verb.** Task 5 declares *find a file* and will call
  `Index::answer`.

## Proposed changelog entry

*Search your own files* (v0.5): a search over the index now answers with
what matched **beside what it did not search** — the folder it was confined
to, the folders the machine would not read, the folders on another disk,
the files it could not open, the kinds whose words it cannot read, the
files too large — so an empty answer is *nothing matched* and never
*nothing was looked at*. A query that asks nothing, or runs past a sentence,
is refused in words before anything is searched. Ten thousand files answer
by name and kind in under ten milliseconds and by contents in under thirty
on the development machine named in the report, against bounds of a tenth
of a second and a second.

## Proposed queue and roadmap updates

Task 4 of `v0-5-the-machine-measured-plan.md` is marked done in the plan.
Task 5 is ready, depends on 1, 2 and 3, and now has `Index::answer` to call.
