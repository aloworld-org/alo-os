# Search your own files — an index that lives on the machine

- Date: 2026-09-13
- Workstream: v0.5 the machine, measured, task 3 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

`docs/features.md` promises, at v0.5: *search your own files, without asking
anything — by name, kind, date and contents, in the file manager, indexed on
the machine.* And the sentence the promise turns on: *a machine whose only
search is a conversation is a machine somebody locked out of their own
documents.* This task is the index — the thing the file manager and the
agent will both ask — written first so that the agent's *"where is that
file?"* becomes a nicer road to it rather than the road.

## What changed

**`crates/alo-finding`, a new crate.** An `Index` is one folder a person
named, walked once, with an `Entry` for everything under it: where it is
below the folder, its `Kind` read from its own bytes and never from its
name, its size, when the filesystem says it was last written as a `Moment`,
and its `Contents` — the words in it for a kind that is text, or the reason
there are none. `Index::of(folder)` makes one, reading every file;
`Index::again()` walks the folder again and reads only a file whose size or
time has changed, and `Index::opened` says how many it read so a test can
count. `Index::find(&Query)` answers by name, kind, date and contents — any
part on its own or all of them together — from the index alone, in the
index's own order, and never touches the disk; `Query` is built from one
part and grows by `and_`, so there is no empty query to construct.

**The index lives on this machine, in a file the person owns.**
`Index::where_kept` says where: `$XDG_DATA_HOME/alo/finding/<fnv1a of the
folder's path>.index`, or `$HOME/.local/share/…` when that says nothing —
the two variables handed in rather than read, the way `alo-choosing` takes
them. `Index::kept_at` writes it whole (a sibling, synced, renamed over),
readable by its owner alone on a Unix host; `Index::read_from(at, folder)`
reads it back and refuses a file whose head names another folder. The format
is JSON lines, one head line carrying `format`, the folder and what the walk
could not reach, then one line per entry — the record's shape for the
record's reasons — and **`docs/contracts/file-index.md`** is the public
description of it.

**Kinds from bytes.** `Kind::of_bytes` reads the first eight kibibytes: a
known signature first (PDF, PNG, JPEG, GIF, zip, ELF), then whether what is
there is UTF-8 with no control character beyond tab, newline, carriage
return, form feed and escape, and *bytes of no known kind* otherwise. A file
named `.pdf` holding a shopping list is text and its words are indexed; a
file named `.txt` holding a PNG is a PNG. Folders, links and devices have
their kind from the walk without being opened.

**Contents are the words.** `wording.rs` is the plan's sentence — *contents
means the words in the file* — as code: a word is a run of letters or digits
in any script, kept in lower case, each once, sorted, and the text itself is
never kept. Reads are bounded by `alo_files::MOST_READ`, the megabyte a file
verb reads under, so there is one bound in the repository for how much of a
file is read; a larger file has its kind read and its words left unread, and
the entry says so.

**What the walk did not reach is kept beside the index** as `Covered`: the
folders the machine would not read, the folders on another filesystem it did
not enter, the folders it had not finished when it reached the walk's bound,
and how many things were left out for names that cannot be shown. Task 4
reads it from the head line to say *what was not searched*.

**Words.** Twenty-three phrases and two countable sentences under
`finding.`: the seven refusals, the three reasons a file has no words, the
thirteen kind names, and the two sentences said once above an index. The
Polish test renders every one. `alo-saying` collects the crate: one line in
its manifest, one in `EVERY_LIST`, one `declare` call, one in
`ONE_STRING_EACH`, one in the sum — and `alo-collected`'s workspace walk
found it with nothing to update.

Thirteen source files, one responsibility each: `index.rs` the answer and
its ways in and out; `indexing.rs` turning a walk into entries, reading only
what changed; `reading.rs` the one bounded look inside a file, as a trait so
the reads can be counted; `kind.rs` the bytes; `wording.rs` the words;
`entry.rs`, `covered.rs`, `query.rs` the types; `format.rs` the file's
shape; `keeping.rs` the atomic write; `place.rs` where it lives;
`refusing.rs` the seven refusals; `words.rs` the strings.

## Decisions

**The walk is `alo-files`' measuring policy, not its searching one.** The
plan says the walk is `alo-files`' walker and leaves the policy open. A
searching walk fails whole when one folder on the way cannot be read, which
is right for a verb that must answer *it is not here* truthfully, and wrong
for an index: a person with one subfolder that belongs to somebody else
would have no search over the rest of their documents. The measuring policy
notes that folder in `Covered`, keeps a link as an entry that is a link with
the bytes of the link, and stops at another filesystem — and every one of
those is exactly what task 4's *says what it did not read* needs written
down. Nothing in `alo-files` was edited.

**Contents are the set of words, not the text.** An index that held every
document whole would be a second copy of every document under a different
name in a place nobody granted, and the promise is an index. Words are
lowered so *Contract* and *contract* are one word; a query is lowered the
same way and every word of it must be present. Nothing ranks.

**One bound for reading, and it is `alo-files`'.** `alo_files::MOST_READ`
is what a file verb reads under and what the daemon's message bound is
derived from; a second number here for the same question would be two bounds
that could disagree.

**Unchanged means the same size and the same modification time.** The rule
`make` and every backup tool use, with the same known gap: a file rewritten
with the same byte count within the filesystem's clock resolution is not
seen. A checksum would close it at the cost of reading every file every
time, which is the thing the acceptance says not to do.

**The file's name is a hash of the folder's path, and the head names the
folder.** A path spelled into a file name runs past what a filesystem allows
long before a person's folders do. FNV-1a is written out in eight lines
rather than rented — nothing about it is secret, it is a stable name and
not a signature — and checked against the published test vectors. Because
two folders could in principle share a name, `read_from` takes the folder
and refuses a file whose head names another (`NotIndexed::NotTheSame`);
that refusal was first placed on `Index::again`, where it was unreachable
through the public surface, and moved.

**Under `$XDG_DATA_HOME`, not config and not cache.** An index is not a
choice, and a cache is something a machine may throw away unasked; an index
of ten thousand documents is an afternoon's reading.

**A kind this version does not know reads as `bytes`.** So a later version
that adds a kind still writes a file this one reads; a changed meaning of an
existing field is a new `format`, which this version refuses. `serde`
insists the catch-all be the last variant, so `Kind::Bytes` is declared
last and `Kind::EVERY` carries the order a window would list.

**UTF-16 is not text here.** It begins with a byte-order mark and then
alternates bytes with zeros, and the rule above says *bytes*. Indexing it
would need a second decoder and a guess at the byte order; a guess that
indexed half the letters of every word is worse than a kind that says the
machine cannot read it. Recorded under limitations.

**Two tests exclude a folder's own modification time.** On NTFS, a folder's
time settles a moment after a write inside it, so two indexes of an
unchanged folder made a millisecond apart differ in a folder entry's time —
a fact about the host, not the code, and one the Windows run found. A
folder has no contents to vouch for, so nothing in the incremental rule
depends on it; the two tests compare file entries, and say why.

## Acceptance criteria and the tests that hold them

| Acceptance | Test |
|---|---|
| a directory is indexed by name, kind, date and contents, and a query over any of the four is answered; the date is the filesystem's | `search_your_own_files_from_an_index_on_the_machine::a_folder_is_indexed_by_name_kind_date_and_contents_and_answers_each` |
| kind is from the file's own bytes, with a `.pdf` that is a text file | `search_your_own_files_from_an_index_on_the_machine::a_kind_is_read_from_the_bytes_and_a_pdf_that_is_a_text_file_is_text` |
| the answer is from the index alone, never by walking again, checked by removing the directory and asking | `search_your_own_files_from_an_index_on_the_machine::the_answer_comes_from_the_index_alone_after_the_folder_is_gone` |
| the index is on this machine, in a file the person owns under their own directory, in the contract's format | `search_your_own_files_from_an_index_on_the_machine::the_index_is_kept_in_a_file_under_the_persons_own_directory_in_the_contract_format` |
| a file unchanged since last time is not read again, with reads counted through the public surface | `search_your_own_files_from_an_index_on_the_machine::a_file_unchanged_since_last_time_is_not_read_again` |
| the same, counted by the reader itself | `indexing::tests::only_what_changed_is_read_and_the_reader_counts` |
| a folder that is not there, is a file, or is relative is refused in words | `search_your_own_files_from_an_index_on_the_machine::a_folder_that_is_not_there_or_is_a_file_or_is_relative_is_refused_in_words` |
| a file that is not an index, is not there, is torn, or is another folder's is refused | `search_your_own_files_from_an_index_on_the_machine::a_file_that_is_not_an_index_or_not_there_or_another_folders_is_refused` |
| the same, line by line, with the reason | `format::tests::what_is_not_an_index_is_refused_with_the_reason` |
| a link is an entry that is a link and is never followed | `search_your_own_files_from_an_index_on_the_machine::a_link_is_indexed_as_a_link_and_never_followed` |
| a name that has become a link between the walk and the read is not read | `reading::tests::a_name_that_is_a_link_is_not_read` |
| every signature, text, empty, and bytes of no known kind | `kind::tests::every_kind_is_read_from_the_bytes` |
| what the walk could not read, enter or finish is kept beside the index | `indexing::tests::what_the_walk_could_not_reach_is_kept_beside_the_index` |
| the index file goes down readable by its owner alone | `keeping::tests::an_index_goes_down_readable_by_its_owner_alone` |
| nothing in the shipped source opens a socket, asks a model, runs anything, or reads the record | `nothing_here_opens_a_socket_or_asks_anybody::nothing_in_the_shipped_source_opens_a_socket_or_asks_anybody` |
| the walk is `alo-files`' and nothing else is rented | `nothing_here_opens_a_socket_or_asks_anybody::the_walk_is_alo_files_and_nothing_else_is_rented` |
| the index takes no account of who asked, and an entry holds nothing the record does | `nothing_here_opens_a_socket_or_asks_anybody::the_index_takes_no_account_of_who_asked` |
| every sentence is read in the person's language | `what_this_crate_says::every_sentence_is_read_in_the_language_the_person_reads` |
| the one refusal worded partly by the file half carries its sentence | `what_this_crate_says::the_one_refusal_worded_by_the_file_half_carries_its_sentence` |
| the crate's words are collected into the machine's vocabulary | `alo-saying` `collecting::tests::every_crate_that_says_something_is_in_it` |

## Verified

WSL Ubuntu on this development machine, as root, `CARGO_TARGET_DIR` the
supervisor's own (`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`), foreground,
exit codes read:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (workspace) | clean, 0 warnings |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps` | clean |
| `cargo test -p alo-finding` | 32 unit, 3 + 8 + 3 integration, 1 doctest: all passed |
| `cargo test -p alo-saying` | 63 + 4 + 1 passed |
| `cargo test -p alo-collected` | 8 + 11 passed |
| every evidence line, alone, `--exact` | 1 passed each, 20 lines |

Windows, this development machine: `cargo fmt --all`, `cargo clippy -p
alo-finding -p alo-saying --all-targets -- -D warnings` clean; `cargo test
-p alo-finding` 30 + 3 + 7 + 3 + 1 passed (the two Unix-only tests — the
link that is not read, the file mode — do not exist there); `cargo test -p
alo-saying` 63 + 4 + 1 passed.

**Not run by this worker:** the full workspace suite, per the task's
instruction; the supervisor runs it. **Not claimed:** anything about a
certified machine, or about the time a search takes — that is task 4's, and
it is measured there.

## Remaining limitations

- **UTF-16 text is `bytes`**, and its words are not indexed. Decided above.
- **A file rewritten with the same size within the filesystem's time
  resolution is not re-read** by `Index::again`. The rule every incremental
  tool uses; `Index::of` reads everything when a person wants certainty.
- **Words inside a PDF, an office document or an archive are not indexed.**
  A reader for each is a dependency and a decision of its own; the entry
  says *not a kind of file whose words can be read*, and a search by name,
  kind or date still finds it. Task 4's *kinds with no reader* list is built
  from this.
- **Text past the first eight kibibytes that is not UTF-8** is replaced
  character by character on the way to words rather than failing the file:
  the kind was decided on what was seen, and the words that are there are
  still there.
- **Nothing here is a verb yet.** Task 5 of the plan declares the read
  verbs; `Index::find` is the function *find a file* will call.
- **Nothing here times a search or refuses an empty question.** Task 4.

## Proposed changelog entry

*Search your own files* (v0.5): `alo-finding` indexes a folder a person
names — by name, by kind read from the file's own bytes, by date from the
filesystem, and by the words in every text file — into a JSON-lines file
under the person's own data directory, readable by its owner alone and
described in `docs/contracts/file-index.md`. A query over any of the four
is answered from the index alone; indexing again reads only what changed.
Nothing in the crate opens a socket, asks a model, follows a link, or holds
anything the record does, and a test reads the shipped source to say so.

## Proposed queue and roadmap updates

Task 3 of `v0-5-the-machine-measured-plan.md` is marked done in the plan.
Task 4 is ready and depends on it; task 5 depends on 1, 2 and 3, and now has
all three.
