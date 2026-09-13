# v0.5 — the machine, measured, before there is a window to show it in

**Workstream:** three `[v0.5]` promises in `docs/features.md` that are
measurements of this machine rather than pictures of it: *search your own
files, without asking anything*; *what is running, and what it is using*; and
*what is filling the disk*. Each names a window. None of the work that makes the
window truthful needs one.
**Why it exists:** [ADR 0028](../decisions/0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
begins v0.5's screenless work while v0.01 waits on hardware. Lane B's provider
and model partition finished on 2026-09-13; this is its next partition, chosen
because it shares no crate with lane A's local-network plan.

**Crates this plan owns:** two new ones — `crates/alo-finding` for the index
and the search, `crates/alo-measuring` for processes, memory, disk and network
— and `crates/alo-saying`'s three list entries when either declares words.
**It reads `crates/alo-files` and never edits it**: the walker there already
knows which paths are real, and a second walker would be two opinions about a
symlink. (Task 2 made that walker public, additively, as `alo_files::Walking`
with a searching and a measuring policy; task 3 borrows it the same way and
edits nothing there.) **Nothing in `crates/alo-shell`**, nothing in `image/`, and nothing in
`alo-nearby`, `alo-asking`, `alo-record`, `alo-capability`, `alo-turn` or
`alo-egress`, which are lane A's on `v0-5-the-local-network-plan.md`.

**What this plan may not do** (ADR 0028's terms): move any v0.01 box, line or
wording; tick anything *on the machine*; edit a crate another lane owns. Before
writing the next task, `git pull` and read the plan as published — numbers are
a shared space, and two lanes have taken the same one twice.

## Tasks

### 1. What is running, and what it is using — read, not estimated

**Status:** done. **Depends on:** nothing.

**Done, 2026-09-13.** Report:
[`updates/what-is-running-is-read-from-the-kernel.md`](updates/what-is-running-is-read-from-the-kernel.md).
`crates/alo-measuring`: a `Reading` is every total the kernel keeps at one
moment, each number carrying the `/proc` file and field it was read from;
`Reading::since` makes two of them and a caller's interval into rates, a share
of the processor, and a list of what ended in between. Per-process network
bytes are the network namespace's, and the answer says how many other
processes share the count rather than attributing the machine's traffic to a
row — the one place the kernel keeps no per-process number, decided in the
report.

*What is running, and what it is using — processes, memory, disk and network
in a window. The plain answer to "why is it slow?", for the person who cannot
or will not ask.* The window is the desktop lane's. What it shows is this
task's, and the whole of the promise is in the word **plain**: a number the
person can check against the machine, not one the machine derived and hopes is
close.

- **Acceptance:** `alo-measuring` answers *what is running* as a list of
  processes each with its memory, its share of the processor since last asked,
  its open files' bytes read and written, and its bytes sent and received —
  every number **read from the kernel** (`/proc`, and nowhere else) and named
  for the file it came from, so a test can open that file and compare; asking
  twice gives a rate rather than a total, with the interval passed in rather
  than read from the clock; a process that exited between two readings is
  reported as gone rather than as zero; and the list is the same whether an
  agent or a person asked, because it is a list of facts.
- **Constraint:** read-only, and no process is signalled, stopped or reniced
  here — *what is using the machine* is a measurement, and acting on it is a
  verb that would need a grant (ADR 0001). No `sysinfo`-style dependency: the
  numbers this promise rests on are twelve files in `/proc`, and renting a
  crate to read them would be a second author's opinion about what *memory*
  means, unreviewable from here. Linux-only, `cfg`'d the way `alo-agentd` is;
  on any other host the crate compiles to its types and refuses to measure.

### 2. What is filling the disk — sizes a person can open up

**Status:** done. **Depends on:** nothing.

**Done, 2026-09-13.** Report:
[`updates/what-is-filling-the-disk-is-a-tree-of-sizes.md`](updates/what-is-filling-the-disk-is-a-tree-of-sizes.md).
`crates/alo-measuring`: `Holding::of` answers a folder as a tree of `Node`s,
each size the sum of its children plus its own bytes, and each node carrying a
`Counted` that says when the size is not the whole truth — a hard link counted
under its first name, a folder the machine would not read, a folder on another
filesystem, a folder the bound stopped the walk in. The walk is `alo-files`',
made public and given a measuring policy in the same change — the one edit to
that crate this plan makes, decided in the report because the private walker
could not express three of the acceptance's clauses and a second walker was
the worse answer.

*What is filling the disk — shown as sizes you can open up and click through,
not a number in Settings.* The click-through is the shell's; the tree of sizes
underneath it is this task's, and it has to be true to the byte for the one
directory a person opens up rather than roughly right for the whole disk.

- **Acceptance:** `alo-measuring` answers *what is filling this* for a
  directory as a tree of sizes, each node's size **the sum of its children
  plus its own files**, checked by a test that builds a directory of known
  bytes and reads the tree back; hard links are counted once per tree, not once
  per name, with a test naming the difference; a symlink is reported as the
  bytes of the link and never followed, because a link out of the tree is how
  a size becomes a lie; a directory the person may not read is a node saying so
  rather than a zero; the walk is `alo-files`' walker, borrowed and not
  reimplemented; and where that walker's `MOST_WALKED` bound cuts a walk short,
  the tree says it was cut short rather than reporting a partial sum as a
  total — a size that is silently too small is worse than no size.
- **Constraint:** nothing here deletes, moves or empties anything. *What is
  filling the disk* ends at the answer, and the person's next action is theirs
  on a surface this crate never draws. The whole disk is never walked unasked:
  a caller names a directory, and a caller that names `/` is answered with a
  tree that stops at each mount point and says so.

### 3. Search your own files — an index that lives on the machine

**Status:** done. **Depends on:** nothing.

**Done, 2026-09-13.** Report:
[`updates/search-your-own-files-is-an-index-on-the-machine.md`](updates/search-your-own-files-is-an-index-on-the-machine.md).
`crates/alo-finding`: an `Index` is one folder a person named, walked once
with `alo-files`' measuring policy, with an `Entry` for everything under it —
where it is, its `Kind` read from its own bytes, its size, when the filesystem
says it was written, and its `Contents`: the words in it for a kind that is
text, or the reason there are none. `Index::find` answers a `Query` over any
of the four from the index alone; `Index::again` reads only a file whose size
or time changed and says how many it opened; the index lives under
`$XDG_DATA_HOME/alo/finding/` in the JSON-lines format
`docs/contracts/file-index.md` describes, readable by its owner alone. Nothing
in the crate opens a socket, asks a model, or holds anything the record does,
and a test reads the shipped source to say so.

*Search your own files, without asking anything — by name, kind, date and
contents, in the file manager, indexed on the machine.* And the sentence the
promise turns on: *a machine whose only search is a conversation is a machine
somebody locked out of their own documents.* This is the index, the thing the
file manager and the agent will both ask, and it is written first so that the
agent's *"where is that file?"* becomes a nicer road to it rather than the
road.

- **Acceptance:** `alo-finding` indexes a directory a person names — by
  **name**, **kind** (from the file's own bytes, not its extension, with a test
  naming a `.pdf` that is a text file), **date** (modified, from the
  filesystem), and **contents** for kinds that are text — and answers a query
  over any of the four from the index alone, never by walking again, checked
  by a test that removes the directory and asks; the index is **on this
  machine** in a file the person owns, under their own directory, in a format
  `docs/contracts/` describes; and indexing is incremental — a file unchanged
  since last time is not read again, with a test counting reads.
- **Constraint:** contents are indexed, and **contents are never sent
  anywhere** — nothing in this crate opens a socket, and a test reads the
  crate's shipped source to say so. Nothing here is a conversation: no model
  is asked what a file is about, and *contents* means the words in the file.
  The walk is `alo-files`' walker. The index is not the record and never holds
  anything the record does.

### 4. Search answers without asking anything, and says what it did not read

**Status:** done. **Depends on:** 3.

**Done, 2026-09-13.** Report:
[`updates/search-answers-in-time-and-says-what-it-did-not-read.md`](updates/search-answers-in-time-and-says-what-it-did-not-read.md).
`crates/alo-finding`: `Index::answer` is the search a person or an agent is
given — `Index::find` stays the filter under it. An `Answer` is what matched
beside a `NotSearched`: the folder outside which nothing was looked at, the
folders the machine would not read, the folders on another disk, the folders
the index stopped in, the files that could not be opened, the kinds with no
reader and the files too large — the last three only when the query asked
for what those files cannot answer. A query that is not one — nothing asked,
more than `A_SENTENCE` words, a part longer than `A_NAME` — is refused with
a `NotAsked` before anything is searched. The answer carries how long it
took; a test builds ten thousand files and times it, and the numbers are in
the report with the machine named.

The second half of *without asking anything*: a search that answers in the time
a person will wait for one, and is honest about what the index does not hold —
a file it could not read, a kind it cannot open, a directory the person never
asked to index.

- **Acceptance:** a query over an index of ten thousand files answers by name
  and kind in under a tenth of a second and by contents in under a second on
  this development machine, measured by a test that builds the index and times
  the answer rather than one that asserts a number; the answer lists what was
  **not** searched — unreadable files, kinds with no reader, directories
  outside the index — as a separate list beside the results, so an empty
  answer is *nothing matched* and never *nothing was looked at*; and a query
  that is not a query (empty, or longer than a sentence) is refused in words
  `alo-saying` collects rather than answered with everything.
- **Constraint:** the timings are measured on this machine and **published
  with the machine named** in the report, the way `docs/features.md`'s zero-
  egress day is; a number with no machine beside it is a claim. Nothing here
  ranks by anything the person did not ask for.

### 5. The three measurements are asked the way everything else is asked

**Status:** done. **Depends on:** 1, 2, 3.

**Done, 2026-09-13.** Report:
[`updates/the-three-measurements-are-verbs-and-not-the-only-road.md`](updates/the-three-measurements-are-verbs-and-not-the-only-road.md).
`crates/alo-finding` declares `search_files` and `crates/alo-measuring`
declares `what_is_running` and `what_is_filling`, each in `src/verbs.rs`
through a `declare_into` in the shape `alo-files` uses, each a read requiring
a grant over the one folder it reads — for *what is running* that folder is
`/proc`, taken as an argument so the grant names exactly what is read.
`Searched::of` and `Measured::of` are the doors from an `alo_files::Touching`
(the call permitted and the folder made real) to `Index::answer`,
`Reading::since` and `Holding::of`, and each hands the authorisation back for
the record. A test in each crate walks the real verb, grants, resolver and
record through the happy road and every refusal, and shows the same answer
reached with no agent and no grant. `docs/by-hand.md` answers the three,
`docs/contracts/agent-verbs.md` describes them, and `alo-capability` was read
and not edited. What is not done, decided in the report: `alo-turn`'s machine
does not yet offer the three — lane A's crate this week — and the index a
search is handed is chosen by the caller, which task 6 takes up.

The agent's *"where is that file?"* and *"why is it slow?"* are verbs, and a
verb is what ADR 0001 says it is: on a closed list, checked against a grant,
recorded. This task makes the three measurements reachable by an agent under
exactly those terms — and makes them reachable **without** an agent, which is
the half the promise underlines.

- **Acceptance:** three read verbs — *find a file*, *what is running*, *what is
  filling* — are declared in the shape `alo-files` declares its own, with an
  argument each and a sentence each, and every one is a **read** in
  `alo-capability`'s terms so no approval is asked and the record's entry has
  no approval to name; each is evaluated against a grant naming the directory
  it reads, refused outside it, and recorded as having run; and the same three
  answers are reachable by a caller with no agent and no grant at all, checked
  by a test that calls `alo-finding` and `alo-measuring` directly — because a
  person searching their own files is not an agent and is not asking anybody.
- **Constraint:** no new crate. The verbs are declared in the crates that
  answer them, the way `alo-files` and `alo-applications` do, and
  `alo-capability`'s verb list is read and never edited — it is lane A's this
  week. If declaring a verb turns out to need a change there, the honest
  deliverable is the finding in the report and the task stays open, not an
  edit to a crate this plan does not own.

### 6. Which folders are indexed, and the index for a folder found by its name

**Status:** done. **Depends on:** 3, 5.

**Done, 2026-09-13.** Report:
[`updates/which-folders-are-indexed-is-a-list-beside-the-indexes.md`](updates/which-folders-are-indexed-is-a-list-beside-the-indexes.md).
`crates/alo-finding`: `Indexed` is the list of folders a person asked to
have indexed, kept as `folders.list` beside the indexes under
`$XDG_DATA_HOME/alo/finding/` in the shape `docs/contracts/file-index.md`
now describes, and written whole the way an index is. `Indexed::index_of`
hands back a folder's index read from its file or refuses with
`NeverIndexed` in words, and never walks the folder — a test holds it to the
kernel's own count of the thread's reads. `Indexed::keep` writes the index
first and the list second; `Indexed::forget` removes the index file first
and the list second, so a forgotten folder's words are off the disk before
anything else. The list holds folders and no other field, is not a grant —
a test indexes a folder no grant covers and shows the verb still refused —
and `Searched::of` is unchanged. Five new refusals, each with a sentence and
a Polish translation in the test.

Task 5 left one thing to the caller on purpose: `Searched::of` is handed the
index of the granted folder and checks that it is that folder's, but nothing
in `alo-finding` says **which folders have an index** or hands back the index
for one. A daemon carrying `search_files`, and a file manager offering a
search box, both need the same answer — *is this folder indexed, and where is
its index?* — and today each would have to remember the folders it indexed
and call `Index::where_kept` itself, which is two lists of the same fact.

- **Acceptance:** `alo-finding` keeps the list of folders a person asked to
  index, under `$XDG_DATA_HOME/alo/finding/` beside the indexes in a format
  `docs/contracts/file-index.md` describes, written whole or not at all like
  an index is; a caller names a folder and gets back its index read from the
  disk, or a refusal in words saying the folder was never indexed — never a
  walk of the folder, checked by a test that asks about an unindexed folder
  and counts the files opened; a folder removed from the list has its index
  file removed with it, and a test says so; the list answers the same
  whether an agent or a person asked, because it holds folders and nothing
  the record does; and `Searched::of` is unchanged — the caller still hands
  it an index, now one this list gave it.
- **Constraint:** the list is not a grant and grants nothing: a folder being
  indexed says nothing about whether an agent may search it, and the tests
  hold the two apart by indexing a folder no grant covers and showing the
  verb is still refused. Nothing here reads the environment inside the
  crate: `$XDG_DATA_HOME` and `$HOME` are passed in, as `Index::where_kept`
  already takes them. The walk stays `alo-files`'; nothing here opens a
  socket, and the shipped-source test keeps saying so.

### 7. An index brought up to date by its name, and an answer that says how old it is

**Status:** done. **Depends on:** 3, 6.

**Done, 2026-09-14.** Report:
[`updates/an-index-brought-up-to-date-by-its-name-says-how-old-it-is.md`](updates/an-index-brought-up-to-date-by-its-name-says-how-old-it-is.md).
`crates/alo-finding`: `Index::of` and `Index::again` take the moment the
index is made from the caller as a `SystemTime`, and `Index::made` carries
it — written into the file's first line as `made`, additively, with
`format` still `1`; a head without it reads with no moment rather than an
invented one, and `docs/contracts/file-index.md` says so. `Answer::made` is
that moment beside the results, so a window can say *as of Tuesday*; how a
moment is spelled for the reader is the window's, because `alo-strings`
formats no dates by design. `Indexed::again` takes a folder's name and a
moment, reads the kept index, indexes again reading only what changed, and
keeps the result whole — one call; a folder never indexed is `NeverIndexed`
and not indexed for the first time, with the kernel's per-thread read count
held to the counter's own cost; a folder gone since is `NotWalked` and its
index is kept byte for byte. The shipped-source test gains sixteen names
that would let a watcher, a thread, a timer or a channel in, and a new test
holds the one `now` in the crate to the stopwatch around a search.

Task 6 made the list the one place that says which folders are indexed and
hands back the index for one. What nobody can yet do through that list is
bring an index **up to date**: `Index::again` exists and reads only what
changed, but a caller holding a folder's name has to read the index back,
call `again` on it, and keep the result — three calls that a daemon and a
file manager would each write, which is the pair of lists task 6 removed
coming back as a pair of refresh loops. And an index does not say **when it
was made**: a search over a folder indexed last Tuesday answers as if it
were now, and the person has no way to tell.

- **Acceptance:** `alo-finding` records in the index's first line the moment
  the index was made, **passed in by the caller** as a `SystemTime` rather
  than read from a clock inside the crate, so a test can make an index at
  noon and read *noon* back; the moment is in the file additively, as
  `docs/contracts/file-index.md` says a later field is, and an index file
  without it still reads; an `Answer` carries the moment the index it
  answered from was made, so a window can say *as of Tuesday* beside the
  results; and `Indexed::again` takes a folder's name and a moment, reads
  the kept index, indexes again reading only files whose size or time
  changed — checked by the read count `Index::opened` already gives — and
  keeps the result whole, in one call. A folder never indexed is refused
  with `NeverIndexed` and is **not** indexed for the first time by a call
  meant to refresh one; a folder that is gone since it was indexed is
  refused with `NotWalked`, and the index it had is **kept**, not removed —
  an unplugged disk is not a request to forget it.
- **Constraint:** nothing here watches a folder: no `inotify`, no thread,
  no timer. When an index is brought up to date is the caller's decision —
  the file manager's, the daemon's, the person's — and a crate that woke up
  on its own to read the disk would be the background reader `CLAUDE.md`
  calls a bug, whether or not what it read was ever shown to a model. The
  shipped-source test gains the names that would let one in. Nothing here
  reads a clock: the moment is an argument, like the interval in task 1.
  The list is still not a grant, and `Searched::of` is still unchanged.

### 8. One search over every indexed folder, each answer saying which folder and how old

**Status:** ready. **Depends on:** 4, 6, 7.

A person's search box is not a folder's. Task 6 made the list the one place
that says which folders are indexed, and task 7 made every answer say how
old it is; what nobody can yet do is type *contract* once and be answered
from every folder they asked to have indexed. Today a file manager would
read the list, read each index, ask each, and stitch the answers together —
which is the loop task 6 removed for *which folder is indexed* coming back
for *what did I search*. And a search stitched together by each caller is a
search whose honesty depends on the caller: one that skips a folder whose
index file would not read, and says nothing, has answered *nothing matched*
about a folder it never looked at.

- **Acceptance:** `alo-finding` answers one `Query` over every folder on the
  list in one call — `Indexed::answer`, or a name the report argues for —
  as one answer per folder, **in the list's order**, each carrying the
  folder it is of, what matched, what was not searched and the moment its
  index was made, read from each index's file and never by walking; a
  folder whose index file could not be read or is not an index is a
  refusal **beside** the other answers, named, rather than a missing folder
  or a failed search, so a test that tears one index file of three still
  gets two answers and one named refusal; a query that is not one is
  refused once, before any index file is opened, checked by the read count
  the task 6 test uses; and an empty list answers with no folders and no
  refusal, because nothing asked for is nothing to search.
- **Constraint:** nothing ranks across folders — the order is the list's,
  and within a folder the index's own, and nothing the person did not ask
  for moves an entry up or down. The verb is **unchanged**: an agent's
  `search_files` still names one granted folder and takes one index through
  `Searched::of`, because a search across every indexed folder under one
  grant would be a search of folders nobody granted — if a cross-folder
  verb is ever wanted it is a verb with a grant per folder, declared in its
  own task and checked at the door like any other. The list is still not a
  grant. Nothing here opens a socket, reads a clock or watches a folder,
  and the shipped-source test keeps saying so.
