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
symlink. **Nothing in `crates/alo-shell`**, nothing in `image/`, and nothing in
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

**Status:** ready. **Depends on:** nothing.

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

**Status:** ready. **Depends on:** nothing.

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

**Status:** ready. **Depends on:** 3.

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

**Status:** ready. **Depends on:** 1, 2, 3.

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
