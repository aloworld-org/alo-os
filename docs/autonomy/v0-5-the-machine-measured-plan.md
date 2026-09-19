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

**Status:** done. **Depends on:** 4, 6, 7.

**Done, 2026-09-14.** Report:
[`updates/one-search-over-every-indexed-folder.md`](updates/one-search-over-every-indexed-folder.md).
`crates/alo-finding`: `Indexed::answer` puts one `Query` to every folder
on the list in one call and hands back an `Everywhere` — one `OfFolder`
per folder, in the list's order, each carrying the folder and either a
`Held` (what matched, what was not searched, the moment its index was
made, held apart from the index it came from) or the `NotIndexed` that
stood where the answer would be: the index file could not be read, is
not an index, or is another folder's. A test tears one index file of
three and gets two answers and one refusal naming the file; the query is
checked once before any index file is opened, held to the kernel's
per-thread read count; an empty list answers with no folders and no
refusal. `Held` is the owned shape of an `Answer`, decided in the report
because an `Answer` borrows an index that is gone when the call returns,
and its borrowing view keeps every sentence said by one piece of code.
The verb, the door and the list are unchanged; the shipped-source test
reads the two new files and still says so.

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

### 9. The indexes read once and asked many times, and a search over every folder timed

**Status:** done. **Depends on:** 7, 8.

**Done, 2026-09-14.** Report:
[`updates/the-indexes-read-once-and-asked-many-times.md`](updates/the-indexes-read-once-and-asked-many-times.md).
`crates/alo-finding`: `Indexed::in_hand` reads every index on the list from
its file once and hands back an `InHand` — one place per folder, in the
list's order, holding the index or the refusal that stood where it would
be — and `InHand::answer` answers any number of queries from memory in the
shape `Indexed::answer` answers in, with a test holding the second query to
the kernel's per-thread read count: no file opened. A torn index file is
the same named refusal in the same place on every query until the caller
reads again. `InHand::again` is `Indexed::again` through the set, the fresh
index put in the set's place for that folder, and a test shows the other
folders — their index files torn on the disk after the set was read —
still answering from hand, and the refresh costing exactly the reads the
disk's own refresh costs. `Indexed::answer` is unchanged. Three folders of
ten thousand files timed on the development machine, WSL: a query over the
index files from the disk 400 ms by name and 360 ms by contents at best;
from hand 5 ms and 11 ms; the numbers are in the report with the machine
named. `NotIndexed` became `Clone` and `PartialEq` so a held refusal can be
handed back, decided in the report.

Task 8 made one search over every indexed folder one call. What that call
does on every query is read every index file from the disk again: a person
typing *contract* into a search box fires a query per keystroke, and three
folders of ten thousand entries are three files parsed per keystroke. Nobody
has measured what that costs. Task 4 timed the answer from an index already
in hand; the read of the file before it was never timed, and a number nobody
measured is a claim. And a file manager that decided to keep the indexes in
hand between keystrokes would today write its own set of them, its own
refresh of one by name, and its own memory of which index file would not
read — three of the loops this plan has been removing one by one.

- **Acceptance:** a test builds three indexes of ten thousand files, times
  `Indexed::answer` by name and by contents, and the numbers are in the
  report **with the machine named**, as task 4's are; `alo-finding` offers
  the list's indexes read from their files once and held in hand — a name
  the report argues for — answering any number of queries over every folder
  from memory in the same shape `Indexed::answer` answers, with a test
  counting reads that shows the second query opens no file; a folder whose
  index file would not read is the same named refusal, held in the same
  place beside the others, until the caller reads again; and one folder is
  brought up to date by its name through the held set in one call — in hand
  and on the disk, through `Indexed::again` — without the other folders
  being read again, checked by the read count. The held form is timed
  beside the file-reading one, and both numbers are published.
- **Constraint:** holding the indexes in hand is the caller's choice for
  the caller's lifetime: nothing here caches across processes, writes
  anything new to the disk, or decides when to read again — no watcher, no
  thread, no timer, no clock, and the shipped-source test keeps saying so.
  Nothing ranks. The verb is unchanged, the list is still not a grant, and
  `Indexed::answer` stays as it is for a caller that wants the disk's word
  every time.

### 10. A folder kept or forgotten in hand and on the disk in one call

**Status:** done. **Depends on:** 6, 9.

**Done, 2026-09-14.** Report:
[`updates/a-folder-kept-or-forgotten-in-hand-and-on-the-disk.md`](updates/a-folder-kept-or-forgotten-in-hand-and-on-the-disk.md).
`crates/alo-finding`: `InHand::keep` and `InHand::forget` are
`Indexed::keep` and `Indexed::forget` on the list the set was read from,
and then in hand what the disk's change means — a kept folder's index at
the end of the set or in its place if it was already held, a forgotten
folder's place gone with everything it held, so the next answer has no
folder for it and a test holding the set's own account of itself to a word
in that folder alone finds it gone. Both cost exactly the reads the disk's
own call costs, checked by the per-thread read count with every other
index file torn after the set was read. Every refusal leaves the set as it
was, as it leaves the disk, but the one in which the disk changed — the
index file removed and then the list not written — where the set holds the
disk's own refusal in that place and none of the words, decided in the
report. The order is the list's, a forgotten folder kept again is last,
and the disk forms, the verb and the list-is-not-a-grant are unchanged.

Task 9 gave a file manager the indexes in hand, and `InHand::again` keeps
one folder's place in the set true to the disk when the person asks for it
to be brought up to date. The other two things a person does to the list
have no road through the set yet: *index this folder* and *forget this
folder* are `Indexed::keep` and `Indexed::forget`, on the disk and on the
list the set was read from — and a set in hand does not see either until
the caller reads it all again. The second of these is the one that
matters: task 6 promised that the words of a folder a person asked to have
forgotten are off the disk before anything else, and a set in hand that
went on answering about that folder from memory would be keeping what the
person asked to have gone, for as long as the caller held it. That is not
a background reader, but it is a memory nobody asked for, and it is the
loop task 9 removed coming back for *forget*: a file manager would have to
remember to drop its held set every time it forgot a folder.

- **Acceptance:** `alo-finding` offers, through the held set, a folder kept
  and a folder forgotten in one call each — `InHand::keep` and
  `InHand::forget`, or names the report argues for — each doing on the disk
  and the list exactly what `Indexed::keep` and `Indexed::forget` do, and
  then in hand what the disk's change means: a kept folder's index in the
  set at the end of the list, or in its place if it was already there; a
  forgotten folder gone from the set, its place and its entries with it, so
  that the next `InHand::answer` has no folder for it and nothing of its
  words is held, checked by a test that reads the set's own list of what
  it holds; a refusal — a folder never indexed, a folder not named from the
  root, an index file that could not be written or removed — leaves the set
  as it was, as it leaves the disk; the list's order is unchanged by any of
  this; and neither call reads any other folder's index file, checked by the
  read count the task 9 test uses.
- **Constraint:** nothing here reads the disk that `Indexed::keep` and
  `Indexed::forget` do not read, and nothing writes what they do not
  write. No watcher, no thread, no timer, no clock, and the shipped-source
  test keeps saying so. Nothing ranks. The verb is unchanged, the list is
  still not a grant, and `Indexed::answer`, `Indexed::keep` and
  `Indexed::forget` stay as they are for a caller that holds nothing in
  hand.

### 11. An index made whole for a folder larger than one walk

**Status:** done. **Depends on:** 3, 7.

**Done, 2026-09-14.** Report:
[`updates/an-index-made-whole-for-a-folder-larger-than-one-walk.md`](updates/an-index-made-whole-for-a-folder-larger-than-one-walk.md).
`crates/alo-finding`: `Index::of` and `Index::again` walk on — the same
walker, asked again from every folder one walk left in `not_entered`, under
the same bound each time, in the new `walking_on.rs`, the one file that
names the walker — until nothing is left unentered, every entry's `below`
spelled from the folder the person named; `Covered::whole` is then true,
`not_entered` empty and `most` still one walk's bound. The one folder no
walk can finish is one holding more than the bound at a single level: it is
tried once more, left in `not_entered`, and the sentence above the index
now says so rather than *stopped after 20000 things*. A folder a later walk
could not read is `unread` exactly as one the first walk stepped over; the
folder a walk stopped inside is listed twice and its steps and its unnamed
names are counted once; an index cut short by task 3 reads, and `again`
makes it whole reading only what it had not reached, `format` still `1`
and no field added. Timed on the development machine, WSL: 24,400 things
indexed whole in 427 to 451 ms, and the memory a whole index takes in hand
— five to eight times its words, held one `String` each — is the number
the constraint asked for, with a bound proposed as a task rather than
taken.

`alo_files::MOST_WALKED` is twenty thousand: the most things one walk
looks at, so that a verb over a granted folder is bounded in time and
memory whatever the folder holds. Task 3 borrowed that walk for the index
and was honest about the bound — an index of a bigger folder says it is
not `whole`, names in `Covered::not_entered` the folders the walk had
found and not yet entered, and every answer says they were not searched.
Honest, and not enough: a person's photo library, a source tree with its
dependencies, or a Documents folder a decade old is more than twenty
thousand things, and for them *search your own files* is a search box
that will never find the rest, with no way to ask it to. `Index::again`
does not help, because it walks the same folder under the same bound and
stops in the same place.

- **Acceptance:** `alo-finding` makes an index **whole** for a folder
  larger than one walk's bound — `Index::of` walking on from each folder
  in `not_entered` until nothing is left unentered, or a name the report
  argues for — with every entry's `below` spelled from the folder the
  person named, so an answer says where a thing is the way it does for a
  small folder; `Covered::whole` is true at the end, `not_entered` is
  empty, and `Covered::most` still says what one walk's bound is; a test
  builds a folder of more than the bound, in subfolders, and gets one
  index with every file in it, timed and the number in the report **with
  the machine named**; `Index::again` on that folder reads only what
  changed, checked by `Index::opened`; and an index that still cannot be
  whole — a subfolder the machine would not read — says so in
  `Covered::unread` exactly as today, so that *nothing matched* is never
  said about a folder nobody looked at. An index file written by task 3
  for a folder that was cut short still reads, and is made whole by
  `Index::again`.
- **Constraint:** the walk is still `alo-files`' and `alo-files` is not
  edited: walking on is that walker asked again from a folder it named,
  under its own bound each time, never a second walker and never a wider
  bound — and nothing is walked that is not below the folder the person
  named. A link is still never followed, another filesystem is still noted
  and not entered. The index's format is unchanged and `format` is still
  `1`; if a field has to be added it is added the way `made` was, so a
  reader from before still reads. Nothing here opens a socket, reads a
  clock or watches a folder, and the shipped-source test keeps saying so.
  The verb is unchanged, the list is still not a grant. If a whole index
  of a very large folder turns out to need more memory than a person's
  machine should give a search, the honest deliverable is that number in
  the report and a bound argued for there, not a quiet partial index.

### 12. Sizes made whole for a folder larger than one walk, with one walking on

**Status:** done. **Depends on:** 2, 11.

**Done, 2026-09-14.** Report:
[`updates/sizes-made-whole-for-a-folder-larger-than-one-walk.md`](updates/sizes-made-whole-for-a-folder-larger-than-one-walk.md).
The walking on lives in one place: `crates/alo-files/src/walking_on.rs`,
additively, as `Walking::throughout` — the walk asked again from every
folder one walk named and did not enter, under the same policy and the same
bound, answering a `Gathered` — with `Walking::through` and the six verbs
untouched, the second and last edit to that crate this plan makes.
`alo-finding`'s `walking_on.rs` is reduced to that call and every task 11
test passes unchanged; `alo-measuring`'s `Holding::of` walks on the same
way, so *what is filling this* is whole for a folder of more than one walk's
bound, every size the sum of its children plus its own files to the byte,
hard links counted once across the walks, and *cut short* said only for a
folder holding more than the bound at a single level, which is marked with
exactly one walk's worth of names. Timed on the development machine, WSL:
24,240 things of known bytes counted whole, to the byte, in 124 to 173 ms,
and the whole root filesystem — 394 thousand things, a hundred gigabytes,
twenty walks — in twelve seconds; the numbers are in the report.

Task 11 made the index whole for a folder larger than one walk. *What is
filling the disk* has the same bound and the same honesty: task 2's
`Holding::of` walks once under `MOST_WALKED`, and where the walk stops the
tree says a folder was cut short rather than reporting a partial sum as a
total. Honest, and the same *not enough*: the folder a person opens *what
is filling the disk* on is exactly the one with too much in it, and a tree
that says *cut short* about it is a measurement that stops where it was
most wanted. The walking on that fixes it exists now, in
`crates/alo-finding/src/walking_on.rs` — and a second copy in
`alo-measuring` would be two opinions about which folder is walked again
and what is counted once, which is the reason this plan borrows one walker
instead of writing another. Task 2 made the walker public in `alo-files`
additively, as the one edit to that crate this plan makes, because the
private walker could not express three of the acceptance's clauses; this is
the same shape of decision, and the task is to make it in the open.

- **Acceptance:** `alo-measuring` answers *what is filling this* whole for
  a folder of more than one walk's bound — every node's size the sum of its
  children plus its own files, checked by a test that builds a folder of
  more than the bound in subfolders with known bytes and reads the total
  back to the byte, timed and the number in the report **with the machine
  named**; the tree's *cut short* is said only for a folder holding more
  than the bound at a single level, exactly as the index's `not_entered` is,
  and never for a folder that was merely large; hard links are still
  counted once per tree across the walks, with the task 2 test extended
  past the bound; **the walking on lives in exactly one place** — the
  report decides where, and the recommendation is `alo-files`, additively,
  beside `Walking` as a call that walks on under the walker's own bound and
  returns what task 11's `Gathered` returns, with `alo-finding`'s
  `walking_on.rs` reduced to using it and every task 11 test still passing
  unchanged; and `alo_files::Walking::through` is unchanged, so `alo-files`'
  six verbs and their archive bound are as they were.
- **Constraint:** if the one place is `alo-files`, the edit is additive
  and argued in the report the way task 2's was — a new call, no change to
  `through`, no wider bound, no second walker — and it is the second and
  last edit to that crate this plan makes. Nothing here deletes, moves or
  empties anything, and the whole disk is still never walked unasked. The
  index's format, the verb and the list are untouched. A link is still
  never followed, another filesystem is still noted and not entered, and
  the shipped-source tests of both crates keep saying what they say.

### 13. An index that fits in hand: the words of a large folder bounded and said

**Status:** done. **Depends on:** 9, 11.

**Done, 2026-09-14.** Report:
[`updates/an-index-that-fits-in-hand.md`](updates/an-index-that-fits-in-hand.md).
`crates/alo-finding`: a file's words are gathered as a set while it is
read — never once per occurrence, not even while gathering — and held with
no room to spare, whether the index was just made or read back from its
file (the list of words used to keep the capacity of every occurrence it
was collected from, and that is gone). At most `MOST_WORDS`, fifty
thousand, different words are kept per file, the first ones it says; a
file with more is the new `Contents::NotAllKept`, carrying how many it did
not keep, with a sentence beside the file, and a search by words that does
not find it among the kept words lists it in the new
`NotSearched::not_all_kept` with a counted sentence rather than answering
*nothing matched*. In the file it is still `"were":"read"` with one field,
`unkept`, left out when nothing was — `format` still `1`, an entry whose
words were all kept byte for byte as before, and a reader with the old
shape still reads every entry, tested. Measured on the development
machine, WSL: six long letters, 5.4 MB, held in 4.7 MB by the index's own
count; three long logs of a hundred thousand identifiers each, 2.7 MB,
held in 4.8 MB with the bound where every word would have been 9.6 MB.
Every task 9 and task 11 test passes unchanged.

Task 11 measured what a whole index takes in hand: every word of every text
file held as its own `String`, five to eight times the file's own size, so
that a hundred thousand long documents is more than a person's machine
should give a search — and it named that as a bound to argue for rather
than trimming an index quietly. Task 9 reads the indexes once and asks them
many times, so what an index holds in hand is what the shell holds for as
long as it is open. Task 12 made the tree of sizes whole the same way the
index is, and the tree holds one node per thing; the index holds every word,
which is the number that grows.

- **Acceptance:** `alo-finding` holds a whole index of a large folder in a
  bounded amount of memory per entry — words kept once per file rather than
  once per occurrence, and a bound on how many distinct words one entry
  keeps, said in `Contents` so that an answer can say a file's words were
  not all kept — with a test that indexes a folder of long text files and
  checks the bytes held in hand against the bytes of the files, measured and
  the number in the report **with the machine named**; a search over a file
  whose words were bounded still finds every word that was kept and says the
  words were not all kept, never *nothing matched* quietly; the index file's
  `format` is still `1` if the change is additive, and if a field is added it
  is added the way `made` was, so a reader from before still reads; and every
  task 9 and task 11 test still passes.
- **Constraint:** nothing here is a quiet partial index: what is left out is
  said in the entry and in the answer. No edit to `alo-files` — this plan's
  two edits to that crate are spent. Nothing opens a socket, reads a clock or
  watches a folder; the verb, the list and the record are untouched. If the
  honest deliverable turns out to be that no bound is needed at the sizes a
  person's machine holds, that is the number in the report and the task ends
  there, with the reasoning written down.

### 14. Words held in one piece: an index in hand the size of its words

**Status:** ready. **Depends on:** 13.

**Done, 2026-09-20.** Report:
[`updates/words-held-in-one-piece.md`](updates/words-held-in-one-piece.md).
`crates/alo-finding`: a file's kept words are **two allocations** — every word
joined end to end in one piece, and where each begins beside it — instead of one
allocation per word. Measured on an **Apple M3 with 8 GB**, in the Lima VM
(Ubuntu 24.04.4 aarch64, 6 CPUs, 4 GB), against task 13's own folder of six long
letters and three long logs: **9 548 201 bytes held as separate words, 4 724 993
in one piece — 49 per cent** — with **301 455 separate allocations** gone. Both
numbers are from one run on one machine, side by side, because task 13's were
taken on another and comparing them would measure the machines; and both are the
index's own count, which cannot see the allocator's rounding on those 301 455
requests — roughly nine megabytes more, named as an estimate as task 13 named
it. A search is still a binary search; the index file is byte for byte
unchanged, 3 218 102 bytes before and after, and one written in the older shape
still reads. `Contents::kept()` is the new way to ask; `Contents::words()` is
kept and deprecated and still answers, and the variants keep the field name, so
`Contents::Read { words }` still pattern-matches. A place is a `usize` rather
than a `u32` on purpose: four bytes a word would need a branch for a file larger
than a `u32` can index, and that branch cannot be reached, so it cannot be
tested. Every task 9, 11 and 13 test answers exactly as before.

Task 13 bounded what one file's words may hold and measured what they do
hold — and the measurement says where the rest of the memory goes. A word
is held as its own `String`: twenty-four bytes of place in the list, and an
allocation of its own that the allocator rounds up to thirty-two bytes on
the development machine, for a word that is seven or eight bytes long. The
letters in task 13's test held 4.7 MB in hand by the index's own count for
1.1 MB of words, and the allocator's rounding — which that count cannot
see — is roughly another 3.6 MB on top, so an index of a person's prose is
held at about one and a half times the size of the files, most of it the
bookkeeping of a hundred and fifty thousand small strings rather than the
words. The index file on the disk is smaller than what it holds in hand.

- **Acceptance:** `alo-finding` holds one file's kept words in a constant
  number of allocations — the words sorted and joined in one piece with
  where each begins beside them, or a shape the report argues for — so
  that the bytes an entry holds are its words' own bytes and a small,
  fixed cost per word, **measured with the machine named** against task
  13's folder of long letters and logs and published beside task 13's
  numbers; `Contents::say` still answers by a binary search over the kept
  words, and every search, every `NotSearched` sentence and every task 9,
  11 and 13 test answers exactly as before; the index file is unchanged,
  byte for byte, and an index written before still reads; and a public
  surface that has to change — `Contents::words` hands out `&[String]` —
  changes additively, with the old way kept and marked deprecated rather
  than removed, because other crates are allowed to have read it.
- **Constraint:** no `unsafe` block — the one piece is indexed by byte
  offsets checked like any other slice, and a measurement that would need
  a counting allocator is instead the index's own count plus the
  allocator's rounding named as an estimate, as task 13 did. No edit to
  `alo-files`. Nothing opens a socket, reads a clock or watches a folder;
  the verb, the list, the record, `MOST_WORDS` and the words a person
  reads are untouched. If the honest deliverable is that the saving is
  not worth a changed public surface at the sizes a person's machine
  holds, that is the number in the report and the task ends there.
