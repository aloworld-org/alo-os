# Which folders are indexed is a list beside the indexes

- Date: 2026-09-13
- Workstream: v0.5 the machine, measured, task 6 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

Task 5 left one thing to the caller on purpose: `Searched::of` is handed the
index of the granted folder and checks that it is that folder's, but nothing
in `alo-finding` said which folders have an index or handed back the index
for one. A daemon carrying `search_files` and a file manager offering a
search box both need the same answer — *is this folder indexed, and where is
its index?* — and each would have had to remember the folders it indexed and
call `Index::where_kept` itself: two lists of the same fact. This task makes
it one list, on the disk, beside the indexes, in the contract.

## What changed

**The list.** `crates/alo-finding/src/indexed.rs`: `Indexed` is the folders
a person asked to have indexed, in the order they asked, and the directory
their indexes are in. `Indexed::read_from(data_home, home)` reads it from
the disk, taking `$XDG_DATA_HOME` and `$HOME` as arguments the way
`Index::where_kept` does; a list that is not there yet is an empty one.
`Indexed::index_of(folder)` hands back the folder's index read from its
file through `Index::read_from`, which still checks the file's head names
that folder — or refuses with `NotIndexed::NeverIndexed` for a folder not on
the list, whether or not the folder exists, and never walks it.
`Indexed::keep(&index)` writes the index to its file and puts its folder on
the list if it was not there; `Indexed::forget(folder)` removes the index
file and takes the folder off the list. `holds`, `folders`, `where_index_of`
and `where_kept` say what is on the list and where things are.

**The file.** `crates/alo-finding/src/listed.rs` is the list as text, the
shape of the index file for the index file's reasons: a first line
`{"format":1}` and one line `{"folder":"…"}` per folder, compact JSON, no
newline inside a line. A torn or foreign file is refused whole with the
reason, because a list read in part would say a folder was never indexed
while its index sat on the disk; a field from a later version is ignored and
a later `format` refused, as `format.rs` does. It goes down through the
existing `keeping.rs` — a sibling written, synced and renamed over the real
file — so it is replaced whole or not at all like an index is.
`crates/alo-finding/src/place.rs` now works out the directory once
(`the_directory`) and names the two files in it (`index_under`,
`list_under`); the list is `folders.list`, a name no sixteen-digit hash can
produce, so a person listing the directory sees which file is the list.
`Index::where_kept` is unchanged in what it answers.

**Five refusals, in words.** `NotIndexed` was `#[non_exhaustive]` and gains
`NeverIndexed`, `NotRemoved`, `ListNotRead`, `NotAList` and `ListNotKept`,
each with a sentence and a translator's note in `words.rs` — the finding
list is now forty-six, and `alo-saying` counts it at run time — and each
with a Polish translation in `tests/what_this_crate_says.rs`. The list of
refusals is now twelve; the front page and the module say so.

**The contract.** `docs/contracts/file-index.md` gains *The list of indexed
folders*: where it is, its shape line by line, that a missing list is an
empty one and a torn one is refused, the order of the two writes, that the
list is the authority over the directory, and that the list is not a grant.

**Front page.** `lib.rs` gains *Which folders are indexed*, the table rows
for `Indexed`, and an example that keeps an index through the list and reads
it back by the folder's name.

## Decisions

**The list is the authority, and the directory is never listed.** The other
design was to answer *which folders are indexed* by reading the directory
and every index head. It was rejected because the head of a ten-thousand-
entry index is the first line of a file that has to be opened to read it,
because the shipped-source test forbids `read_dir` in this crate for the
reason that a walk of anything is `alo-files`' — and because a file in that
directory the list does not name should be nobody's index rather than one
that comes back from the dead when the directory is listed. The list is one
small file, read whole.

**Written whole through `keeping.rs`, as one file per fact.** The list is
not appended to and is not one line per change: it is the set of folders,
replaced whole, so that the file on the disk is always a list this version
reads or the previous one. `keeping.rs` already did exactly this for an
index and was reused unchanged.

**Keep writes the index first; forget removes the index file first.** The
two orders are chosen for what is left if the second step fails. On keep, a
list that named a folder whose index was never written would make
`index_of` say *could not be read* about a folder the person just indexed;
so the index goes down first, and a failed list write leaves the list in
hand and on the disk as they were, which a test shows. On forget, the words
of every text file under the folder are in the index file, and a person who
asked for the folder to be forgotten asked for those to go; so the file goes
first, and a list write that then fails leaves a folder on the list whose
index is gone — which `index_of` reports as *could not be read*, and which
is repaired by forgetting again or indexing again. An index file already
gone is not a refusal to forget.

**Never walked, measured with the kernel's own count.** The acceptance asks
for a test that asks about an unindexed folder *and counts the files
opened*. `Index::opened` counts what an indexing read, but a refusal returns
no index to count on, so the test reads `/proc/thread-self/io`'s `syscr` —
the read calls this thread has made — before and after the question, and
holds the difference to exactly what reading the counter itself costs,
measured in the same test by reading it twice; then walks the same folder
of fifty files and shows the counter move by at least fifty, so the counter
is known to see what it must not see. Per thread and not per process,
because the other tests in the file run beside it on their own threads and
read files of their own; the first run of the per-process form failed for
exactly that reason. On a host without that file the count is skipped and
the rest of the test — *never indexed* for a folder that is there and full,
and the same answer rather than *could not be indexed* for one that is not
there at all — still runs, which is what the Windows run below is.

**The list is not a grant, shown both ways.** The test indexes a folder no
grant covers and shows `Authorised::read` refuse the verb with *not
granted* before the index is asked; grants a folder that is not indexed and
shows the door open with nothing to hand it, the list saying *never
indexed*, and `Searched::of` still refusing the other folder's index for
it; and then both, where the index the list gave is exactly what
`Searched::of` takes. `searched.rs` was not edited; `Searched::of` still
takes `&Index`.

**Two readers can hold stale lists.** A daemon and a file manager each read
the file into an `Indexed`, and a keep or forget in one is not seen by the
other until it reads again. That is the ordinary state of any file two
programs read, and the type's rustdoc says so; a lock, a watch or a socket
between them would each be a thing this crate is not allowed to be. A test
shows a second reader of the same data home seeing what the first kept.

**What is not here.** Bringing an index up to date through the list, and an
index that says when it was made, are task 7 in the plan, written in this
change. `alo-turn` still does not offer `search_files`; that is lane A's
crate and unchanged from task 5.

## Verification

Linux, WSL Ubuntu, the supervisor's target directory
`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`, every command in the
foreground and waited for:

```
cargo fmt --all -- --check                                  clean
cargo clippy --all-targets -- -D warnings                   clean, whole workspace
RUSTDOCFLAGS="-D warnings" cargo doc -p alo-finding --no-deps   clean
cargo test -p alo-finding                                   91 passed with the doctests, five rounds in a row
cargo test -p alo-saying                                    68 passed
```

Windows 11, this checkout:

```
cargo clippy -p alo-finding --all-targets -- -D warnings    clean
cargo test -p alo-finding                                   88 passed
```

The Windows count is lower because the tests that need Unix file modes or
Linux paths do not run there; the read-count clause in the never-walked
test is skipped there and ran on Linux, where `/proc/thread-self/io` exists.
The full workspace suite was not run here; the supervisor runs it.

The existing guard `tests/nothing_here_opens_a_socket_or_asks_anybody.rs`
kept passing unchanged: still six dependencies, still one file that walks,
still no `read_dir` and no environment read anywhere in `src/`.

## Proposed changelog entry

*Search your own files:* alo OS now keeps the list of folders you asked to
have indexed beside the indexes, in a file you can open, so that the file
manager and the agent ask one list rather than each remembering. Asking
about a folder that was never indexed is answered in words and never by
reading the folder; forgetting a folder removes its index with it. A folder
being indexed does not let an agent search it — that is still a grant you
make.

## Proposed queue and roadmap updates

Task 6 of the v0.5 measuring plan is done; task 7 (an index brought up to
date by its name, and an answer that says how old it is) is written and
ready. Nothing on the machine is ticked.
