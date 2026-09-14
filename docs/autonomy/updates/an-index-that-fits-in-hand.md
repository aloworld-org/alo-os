# An index that fits in hand: the words of a large folder bounded and said

- Date: 2026-09-14
- Workstream: v0.5 the machine, measured, task 13 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

Task 9 holds every index in hand for as long as a file manager is open, and
task 11 measured what that costs and named a bound to argue for rather than
trimming an index quietly. This task argues it, takes it, and says it
wherever it applies: in the entry, beside the file, and in every search
answer that could otherwise have said *nothing matched* about words nobody
kept.

## What changed

**Words once per file, in hand as well as on the disk.** The set of a file's
words was already deduplicated, but `wording::words_of` collected every
occurrence into one `Vec<String>`, allocating a `String` for each, then sorted
and deduplicated in place — so the list kept the capacity of every occurrence
it was collected from, twenty-four bytes each, for as long as the index was
held. A megabyte of prose is about a hundred and fifty thousand occurrences:
some three and a half megabytes of places held for twenty-five thousand
words. `wording::kept_of` now gathers a set while it reads: a word already in
lower case is borrowed from the text rather than allocated, a word said again
is one lookup, and the list handed back is built at exactly its length with
every word at exactly its own. `format::read` shrinks the list of entries it
collected, and `Contents` shrinks every list of words it reads, so an index
read back from its file holds what one just made holds — the test checks
both.

**At most fifty thousand different words per file, and the first ones.**
`alo_finding::MOST_WORDS` is `50_000`. A file that says more keeps the first
fifty thousand different words in the order it says them and counts the
rest. *First in the file* rather than *first alphabetically*, because an
alphabetical bound is a search that can never find any word beginning with
*z* in any large file, where *the beginning of the file was kept* is a thing
a person can be told and can act on.

**Said in `Contents`.** `crates/alo-finding/src/contents.rs`, new: `Contents`
moved out of `entry.rs` (which re-exports it, so `alo_finding::entry::Contents`
still names it) because it gained a second reason to change — how it is
spelled in the file is now its own code rather than a derive. It has a new
variant, `Contents::NotAllKept { words, unkept }`, and `Contents::all_kept`.
`Contents::words` and `Contents::say` answer from the kept words for both
kinds of text. `Contents::said` beside such a file is the new
`finding.contents.not-all-kept`: *More different words than the 50000 an
index keeps, so only the first 50000 were kept.*

**Said in the answer.** `NotSearched::not_all_kept`, and `Unsearched`'s copy
of it: for a search by words, a file whose words were not all kept is held
against the words it kept. Found, it is in `found`. Not found **and matching
everything else the query asked** — name, kind, dates — it is listed here,
and `NotSearched::is_nothing` is false; `said` adds the counted sentence
`finding.not-searched.not-all-kept` (*3 files hold more different words than
an index keeps, so not all of their words were searched*). A bounded file
that fails the query on its name is not listed, because its name was
searched whole; a search by name, kind or date alone lists nothing, because
none of those depends on words. The split needed `Query::matches_apart_from_words`,
public beside `Query::matches`, which is now that and the words.

**The file: `format` still `1`, one field added the way `made` was.** A file
whose words were not all kept is still `"were":"read"`, with its kept words
and `"unkept":N` after them. `unkept` is left out when it is zero, so an
entry whose words were all kept is written byte for byte as before and an
index of prose is the same file it was. A reader from before ignores the
field it does not know and reads the kept words — which is true as far as
it goes, and is what the contract says a later field does. A new `were` value
was the other design and was refused: a reader from before refuses a tag it
does not know, so it would refuse the whole index. `docs/contracts/file-index.md`
says all of this, and a count that is not a whole number of zero or more is
refused whole like any torn line.

**Words for translators.** Two, each with a note: the phrase beside a file
and the counted sentence beside an answer. The Polish test translation has
both, in one, few and many.

### A user-readable change description

> **Search your own files: an index that stays small in memory.** A file's
> words are now held once each while it is indexed as well as after, so an
> index held open by the file manager no longer carries room for every time
> a word was repeated. A text file with more than fifty thousand different
> words — a log or a table a program wrote; no letter, note or book in any
> language comes near it — keeps the first fifty thousand and says so beside
> the file, and a search by words that does not find it says that not all
> of its words were searched instead of saying nothing matched. Index files
> written before still read, and an index of ordinary documents is written
> exactly as it was.

## Decisions

- **The bound, fifty thousand.** The question is what prose says at the
  megabyte an index reads. A whole English novel of about that size — *Moby
  Dick* is 1.2 MB — says some seventeen thousand different lower-cased words.
  Languages that build words from many endings, as Finnish and Hungarian do,
  say more different forms for the same length, and the published
  type-token ratios put that at roughly two to three times English for text
  of this length; **that multiple is an estimate from the literature, not a
  measurement made here**, and there is no Finnish corpus on the development
  machine to measure it with. Fifty thousand is above the top of that
  range, so a person's writing in every language this machine is for is kept
  whole, and text a program wrote is what reaches it. A bound that trimmed
  Finnish letters and not English ones would be the bug CLAUDE.md names,
  wearing a memory budget.
- **What the bound buys, and what it does not.** It caps an entry at
  `MOST_WORDS` places (1.2 MB) plus about the file's own bytes of words — a
  little more only where a lower-case letter is longer than its capital, as a
  dotted capital I is — so an entry holds a couple of megabytes at most by
  the index's own count, whatever a file says. It does not make prose smaller: a person's letters are under
  the bound and were already held once per word. What makes prose smaller in
  hand is the capacity fix above, and the next saving is the representation,
  written as task 14 rather than taken here, because it changes the public
  `Contents::words` and deserves its own measurement.
- **The first words of the file, not the most frequent.** Frequency would
  need a count per word while reading — more memory at exactly the moment
  the file is large — and would keep *the*, *and*, *of* over the one rare
  identifier a person searches a log for. The first words are cheap, exact,
  and sayable.
- **`unkept` counts distinct words, exactly.** Counting them needs the set of
  every different word while one file is read — transient, bounded by the
  megabyte the file can be, gone before the next file — and a sentence
  saying *some* where it could say how many is less than the truth.
- **A bounded file that matched is not listed as not all searched.** It
  matched; the answer about it is complete, and its `Contents::said` still
  says its words were bounded to anyone who looks at it.
- **A word longer than a query may ask for is still kept.** `A_NAME` refuses a
  query word over 255 characters, so such a word can never be asked; dropping
  it would save nothing the megabyte bound does not already cap, and would
  make a file say *not all kept* over words nobody can type.

## Acceptance criteria

| Criterion | Evidence |
|---|---|
| A whole index of a large folder in a bounded amount of memory per entry; words once per file rather than once per occurrence; a bound on distinct words per entry, said in `Contents` | `tests/an_index_that_fits_in_hand.rs` `a_folder_of_long_text_files_is_held_in_hand_in_a_bounded_number_of_bytes_measured`; unit tests `wording::tests::the_first_words_are_kept_and_the_rest_are_counted`, `wording::tests::what_is_kept_has_no_room_to_spare`, `indexing::tests::a_look_at_more_words_than_are_kept_keeps_the_first_and_counts_the_rest`, `contents::tests::words_not_all_kept_answer_from_what_was_kept_and_say_so` |
| A test indexes a folder of long text files and checks bytes held in hand against the files' bytes, measured, machine named | the same integration test; numbers below |
| A search over a bounded file finds every kept word and says the words were not all kept, never *nothing matched* quietly | `a_search_over_a_file_whose_words_were_bounded_finds_every_kept_word_and_says_the_rest` (through an `Index`, an `InHand` set and `Index::again`); `what_this_crate_says` in English and Polish |
| `format` still `1`; the field added the way `made` was; a reader from before still reads | `the_index_file_is_format_one_and_a_reader_from_before_still_reads_it` (every entry line parsed with the pre-change shape); `contents::tests::contents_are_tagged_in_the_file`, `contents::tests::contents_come_back_as_they_went_and_nothing_unkept_is_all_read` |
| Refusals: a count that is not one | `contents::tests::an_unkept_that_is_not_a_count_is_refused`; the integration test's torn `unkept` refused whole as `NotIndexed::NotAnIndex` |
| Every task 9 and task 11 test still passes | `the_indexes_read_once_and_asked_many_times` (5) and `an_index_made_whole_for_a_folder_larger_than_one_walk` (4), unchanged files, passing |

No approval is required: nothing here is a grant, a verb, or a v0.01 box.

## The measurement

**Machine:** the development machine — Windows 11 Pro 10.0.26200 host, tests
run in WSL 2 Ubuntu as root, against the Linux temporary directory, debug
build, in the supervisor's target directory. Not a certified machine.

The folder: six letters of about 900 kB each, prose-shaped — words drawn from
a 30,000-word vocabulary, common words far more often than rare, sentences
beginning with a capital — and three logs of about 900 kB each, each saying
*begins log N*, a hundred thousand different identifiers, and *finale*.

| | On the disk | Held in hand (index's own count) | Words kept |
|---|---|---|---|
| Six letters | 5,400,018 bytes | 4,747,808 bytes (0.88×) | 151,455 (all of them) |
| Three logs, bounded | 2,700,060 bytes | 4,800,249 bytes (1.78×) | 150,000 (50,000 each; 50,004 not kept each) |
| One log, every word held | 900,020 bytes | about 3,200,112 bytes (3.56×) | 100,004 |
| The index file of all nine | 3,218,101 bytes | — | — |

The same numbers, to the byte, for the index read back from its file. Three
runs; `Index::of` over the nine files took 1.07 to 1.41 s in a debug build.

*Held in hand* is counted from the index: the size of each `Entry`, the
capacity of its path, the capacity of its list of words times twenty-four,
and the capacity of every word. The allocator's own rounding is not in it —
a test cannot count that without replacing the global allocator, which needs
an `unsafe` block. **As an estimate:** glibc on x86-64 gives an allocation of
up to 24 bytes a 32-byte chunk, so a word of the letters' seven or eight bytes
costs about 25 bytes more than counted — roughly another 3.7 MB for the six
letters (about 1.5× the files in all) and 1.2 MB for the three bounded logs
(about 2.2× in all, where every word held would be about 6×). That overhead
is per word rather than per byte, and it is what task 14 is written to
remove.

## Verification

| Platform | Command | Result |
|---|---|---|
| WSL Ubuntu, `/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1` | `cargo fmt --all` | clean, exit 0 |
| WSL Ubuntu | `cargo clippy -p alo-finding -p alo-saying --all-targets -- -D warnings` | clean, exit 0 |
| WSL Ubuntu | `cargo clippy --all-targets -- -D warnings` (workspace) | clean, exit 0 |
| WSL Ubuntu | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps` | clean, exit 0 |
| WSL Ubuntu | `cargo test -p alo-finding` | 72 unit, 4 + 3 + 1 + 6 + 4 + 3 + 5 + 6 + 7 + 8 + 5 + 3 + 7 integration, 2 doc tests — all passing |
| WSL Ubuntu | `cargo test -p alo-saying` | 63 + 4 + 1, all passing (it collects this crate's words) |
| WSL Ubuntu | `cargo test -p alo-finding --test an_index_that_fits_in_hand -- --nocapture --test-threads=1`, three times | 3 passing each time; the numbers above |

**Second attempt, 2026-09-14.** The supervisor's first check of this tree
refused before any gate ran: its disk-room probe, `df` on
`$HOME/alo-builds/alo-os-b-72aa4fda7f7d8fc1` through WSL, exited non-zero with
nothing on stderr, 31 seconds after the check began. That is the WSL bridge
failing, not this change — the previous two tasks passed the same probe, and
asked again it answered at once with about 873 GiB free. No code was changed
for it. The gates above were run again in the foreground by a second worker —
`cargo fmt --all` (no changes), workspace clippy with warnings denied,
`cargo test -p alo-finding`, `cargo test -p alo-saying`, rustdoc with warnings
denied — all clean, and each of the fourteen evidence tests was run on its
own and passed.

The workspace test suite was not run by this worker,
as the brief asks; the supervisor runs them. No test here depends on a
permission refusal, so nothing needed an unprivileged run.

## Remaining limitations

- The multiple for morphologically rich languages behind the choice of fifty
  thousand is an estimate from published type-token figures, not a
  measurement on this machine.
- An entry's memory is bounded; a folder's is still the sum of its entries,
  so a folder of a hundred thousand long documents is still a large index in
  hand — about 1.5× its text by the estimate above. Task 14 is the next
  saving.
- A file whose *later* words matter — the end of a long log — is exactly the
  one the bound leaves out. It is said, not solved; the file's name, kind and
  date are still searched whole.
- `unkept` is decided when a file is read. An index made again reads an
  unchanged file's contents from the earlier index, so a later version with
  a different bound applies it only to files that change.

## Proposed shared-document updates

For `CHANGELOG.md`, under the unreleased v0.5 section: the user-readable
change description above.

For `docs/autonomy/QUEUE.md` and `STATE.md`: task 13 of the machine-measured
plan done with this report; task 14, *Words held in one piece: an index in
hand the size of its words*, written in the plan and ready.

`ROADMAP.md`: no change.
