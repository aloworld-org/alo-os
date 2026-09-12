# ADR 0029 — What the kernel writes down about a turn, and what the daemon's account becomes

**Status:** proposed — the owner or the delegate decides. Nothing is built in
the change that adds this: the programme keeps its two maps, the record keeps
its shape, and
`crates/alo-bounding/tests/the_records_source_is_decided_before_it_is_built.rs`
fails the day a third map appears while this line still says *proposed*.
**Date:** 2026-09-12
**Proposed by:** the kernel-enforcement workstream, as task 16 of
`docs/autonomy/kernel-enforcement-plan.md`
**Context:** [ADR 0001](0001-the-capability-model.md) §7 (everything executed is
a record with an origin), [ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md)
(*the record becomes an observation, not a claim*),
[ADR 0015](0015-the-kernel-learns-what-a-turn-is.md) (*the LSM decides and
forgets*), [ADR 0018](0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
(one privileged component, taking no input),
`docs/contracts/record-file.md` (additive only), `docs/features.md` (three v0.5
lines quoted below), `crates/alo-bounding-kernel`, `crates/alo-bounding`,
`crates/alo-boundaryd`, `crates/alo-record`, `crates/alo-turn`,
`crates/alo-recounting`

## The question in one line

**When the record says what a turn touched, whose sentence is that — the
daemon's, the kernel's, or both, and shown as which?** Two accepted decisions
and one test answer it three different ways, and until one answer is chosen no
worker may build any of them.

## What is true today, verified rather than remembered

Read off the tree this was written in, because the argument rests on it.

- **The record is the daemon's account, written outside the boundary.**
  `crates/alo-turn/src/carrying.rs` runs the verb's work inside the turn's
  control group and *then* writes `Entry::ran` or `Entry::refused` from what the
  capability model and the verb produced. The kernel contributes one thing to
  that account: `EACCES`. A read, a write, a rename, an attribute change or a
  connection the boundary refused arrives in `alo-files` as the same
  `std::io::Error` and is written down as one sentence — *the machine refused
  this* — beside the path the daemon says it tried
  (`a_read_the_kernel_refused_is_the_sentence_a_refused_open_is`). That is the
  kernel's decision in the daemon's words.
- **Since task 15 the record also holds the machine's own refusal**:
  `not-bounded`, when there was no boundary to run a turn inside. It is the
  daemon reporting the kernel's *absence*, and it is the first entry whose
  subject is the boundary rather than a verb.
- **The programme has exactly two maps and writes nothing.** `BOUNDS`, which
  the daemon fills for the length of a turn and empties when it ends, and
  `FIELDS`, which the loader fills once. `the_boundary_decides_and_forgets.rs`
  counts them on a running kernel, asserts nothing appears in the map of turns
  while no turn runs, and asserts the kernel's trace buffer did not move; its
  header says in as many words that **a third map is the finding**. The
  strongest property in this repository is therefore structural: the programme
  does not refrain from keeping a note, it has *nowhere to keep one*.
- **The programme sees everything by construction.** Twelve hooks, called for
  every open, read, write, rename, removal, link, attribute change, connection
  and message on the machine, by every program, for as long as it is attached.
  ADR 0015 calls this the most dangerous thing in the repository, and the only
  thing between enforcement and surveillance is the sentence above.
- **The loader takes no input and leaves pins.** ADR 0018's one privileged
  component loads one programme, pins its maps and links under
  `/sys/fs/bpf/alo`, gives the agent's group mode `0660` on the map of turns
  and nothing on the map of fields, and stops. What the daemon may do to the
  kernel is a mode on a file, decided in `crates/alo-bounding/src/pinned.rs`.
- **The record file is additive, one line per entry, synced per entry.**
  `docs/contracts/record-file.md`: a new kind of `happened` does not raise
  `format`; every entry is flushed and synced before the write answers; the
  file is appended to and never rewritten. A shape that wrote one line per
  syscall would make a listing of a folder cost a thousand syncs.
- **`docs/features.md` promises three things at v0.5, and they are quoted here
  exactly** because the decision is which of them bends:
  1. *So the record stops being anybody's account of themselves. What a turn
     touched is what the kernel watched it touch — the difference between an
     audit log and a guarantee.*
  2. *And it forgets everything that was not an agent. A syscall outside a turn
     is checked and leaves no trace: no log line, no counter, no timestamp. The
     mechanism that could watch everything is the one place this promise is
     proved by a test rather than stated.*
  3. *So the record stops being a claim and becomes an observation. Today it is
     `alo-agentd`'s honest account of itself, which is an audit log; with the
     boundary in place, what a turn touched is what the kernel watched it
     touch. What did the agent do is answered by the machine rather than by the
     program being asked about.*

## The contradiction, stated exactly

Promise 1 needs the kernel to write something down. Promise 2's test forbids
the kernel having anywhere to write. Both are accepted, both are right, and
**they are not actually in conflict about what is watched — only about
where the discipline lives.** Promise 2 is about syscalls *outside* a turn; a
place the programme writes into only when the control group it looked up is a
turn keeps every word of it. What changes is the kind of promise it is: today
*nothing outside a turn is written down* is true because it **cannot** be, and
under any option that gives the kernel a pen it is true because the programme
**does not** — a property of code, held by a test, rather than a property of
the shape of the thing. That downgrade is the real price of promise 1, it is
paid once, and it is the reason this is a decision rather than a task.

A second fact narrows the options more than the first. **The kernel cannot
write the record.** ADR 0001 §7 asks for four answers — what ran, under whose
authority, from which approval, against which grant — and the kernel knows
none of them. It knows a control group, a hook, a decision, and the identity of
a file or the address of a socket. So *the kernel's observation replaces the
daemon's account* is not an option anybody can build; every option below keeps
the daemon's account for the four answers and decides what the kernel adds to
it and how a person is shown the two.

## What may not be done, whichever option is taken

- **Nothing outside a turn is ever written anywhere**, by any option, and the
  test that proves it stays on a real kernel and is strengthened rather than
  loosened. This is ADR 0015's rule and ADR 0001 §4's *a background reader is a
  bug*, and it is the sentence a customer is sold.
- **What was asked is never kept.** No option puts a question, a selection or
  a document's contents anywhere; `alo-record` already has nowhere for them.
- **The record file changes additively.** A new kind of `happened` or a new
  field, and `format` stays `1`, per its contract.
- **The daemon gains no capability**, and the loader gains no input. Whatever
  the kernel writes, the daemon reaches it through a mode on a pinned file, as
  it reaches the map of turns today.
- **The three promises are not narrowed quietly.** Option D below rewords one of
  them, and says so in the heading.
- **A record that could lose an observation must say it lost one.** A place the
  kernel writes into is bounded; the day it is full is the day a security team
  is looking, and *nothing observed* and *nothing kept* must not be the same
  line.

## The options

### Option A — the kernel emits, the daemon appends

A third map, a ring buffer, written by the programme on every decision it makes
for a control group that is a turn: cgroup id, hook, decision, and either the
path (through `bpf_d_path`, where the kernel permits that helper on the hook)
or the file's identity. The daemon consumes the ring and appends what it reads
to the record.

- **What it buys:** the most literal reading of promise 1. Every syscall the
  kernel decided is in the record in the order it happened, and a security
  team's export at v1 gets the kernel's stream.
- **What it costs a person reading *what did the agent do*:** everything. A
  read of one file is an open, several reads and a close; a listing is one
  entry per name; an archive is thousands. The record stops being one sentence
  per verb and becomes a syscall trace, and the thing that folds it back into
  sentences is the daemon — which reintroduces the daemon's judgement at the
  moment the option claimed to remove it. Kept raw, nobody reads it; folded,
  it is an account again.
- **What it costs the record file:** either one line per syscall, each synced,
  or a batch per turn, which is Option C's shape arriving anyway. And a ring
  buffer overflows: an observation the daemon was too slow to consume is gone,
  so every turn's entry needs a *lost* count and the reader has to be told the
  record is a sample.
- **What it costs the loader:** a fourth pin, a mode giving the agent's group
  the ring, and a consumer problem the loader cannot solve without input:
  reading a ring buffer is consuming it, one daemon runs per signed-in person,
  and two daemons on one machine would each read half of the other's turns.
  A ring per daemon would need the loader to be told who, and ADR 0018 forbids
  telling it anything.
- **What it costs promise 2:** the structural property, for the behavioural one.
  Named above; paid by C as well.
- **Where the path comes from is a measurement this ADR does not have:** the
  helper that renders one is permitted only from a list of hooks the kernel
  keeps, and whether all twelve of ours are on it is a fact about this kernel
  nobody here has measured. An option that assumes it is an option that may
  not build.
- **Not recommended.** It is Option C with the folding done in the daemon and
  the ordering paid for in overflow.

### Option B — the daemon's account kept, with the kernel's refusals added beside it

No third map. The value of a turn's entry in `BOUNDS` — which the daemon
writes at the start of a turn and removes at the end, and which exists only
while a turn does — gains words the programme adds to: one refusal count per
hook. At the end of the turn, before removing the entry, the daemon reads the
counts and writes them into the turn's record entries as additive fields: *the
kernel refused this turn four opens and one connection*. Every refusal is
already in the record in the daemon's words; this puts the kernel's own count
beside them.

- **What it buys:** the cheapest cross-check there is, and a real one. A daemon
  whose account says nothing was refused, beside a kernel that counted three
  refusals, is a daemon whose account is wrong — which is ADR 0013's exact
  question, *what happens when the recorder is wrong*, answered with a number.
  The map count stays two; `the_program_has_nowhere_to_write_what_it_sees`
  keeps its assertion word for word; a word the programme adds to inside an
  entry the daemon owns and removes is written only where a turn already is,
  so promise 2 keeps its structural form: outside a turn there is still
  nowhere to write, because there is no entry.
- **What it costs a person:** nothing to read. One sentence per turn, only when
  the counts disagree with the account.
- **What it costs the record file:** an additive field on `ran` and `stopped`;
  `format` stays `1`.
- **What it costs the loader:** nothing. Same maps, same pins, same modes. The
  layout of a turn's words in `alo-bounding-map` gains a region the daemon
  writes as zero and the programme increments.
- **What it does not deliver:** promise 1 and promise 3 as written. A count of
  refusals says how often the daemon was stopped, never what it touched. A
  compromised daemon that stays inside its grant and lies about *which* granted
  file it read is caught by nothing here. The sentence *what a turn touched is
  what the kernel watched it touch* is not made true by B, and taking B alone
  would need Option D's rewording beside it.

### Option C — the kernel's observation is held beside the daemon's account, and the account is shown as a claim *(recommended)*

A third map, but **a table rather than a stream**: a hash map keyed by
`(cgroup id, hook, device, inode)` for files and `(cgroup id, hook, address)`
for departures, with a count as its value, written by the programme only after
the lookup that says *this control group is a turn* has succeeded, and holding
**identity, never a name**: no path, no string, no timestamp. At the end of a
turn the daemon reads every key carrying its cgroup id, removes them, and
compares them with its own account: for each path a `ran` entry names, the
identity the daemon gets from `stat` of that path; for each destination, the
address it resolved. It then writes **one additive entry per turn**, `watched`,
saying one of three things — *the kernel saw exactly what this account names*,
*the kernel saw N things this account does not name*, with their identities, or
*the kernel could not keep every observation* — and `alo-recounting` shows the
account's lines as they are today with that one sentence under them.

- **What it buys:** promises 1 and 3 in their strongest buildable form. The
  paths a person reads stay in the daemon's words, because only the daemon has
  words; but every one of them is either confirmed or contradicted by what the
  kernel saw, and **a thing the kernel saw that the account does not name is
  in the record with an identity an administrator can find** (`find -inum`).
  *What did the agent do* is still answered in a sentence, and the machine can
  now say the sentence is wrong. A verb with a bug, an argument mis-parsed, a
  daemon compromised — each leaves the disagreement ADR 0013 was written to
  make visible.
- **What it costs a person reading:** one sentence per turn, and only one. The
  record does not grow by syscalls; the daemon folds identities against its own
  claims and writes the result. Nobody is shown an inode number unless the
  kernel saw something the account did not name, which is the case they should
  be shown.
- **What it costs the record file:** one new kind, `watched`, additive; one line
  per turn; no `format` change. A turn whose observations overflowed the table
  writes the third sentence, never silence.
- **What it costs the loader:** a fourth pin, with the same `0660` and group as
  the map of turns, since the daemon must read and remove its own turn's keys.
  No input: the table is one, shared by every daemon, and keys carry the cgroup
  id, so each daemon takes only what is its own — which is exactly how the map
  of turns is shared today. Two daemons do not consume each other's rows,
  because a table is read, not drained. A daemon that dies mid-turn leaves rows
  keyed by a control group that no longer exists; they are an agent's and not a
  person's, so promise 2 is not broken by them, but the table fills, and the
  implementation owes a sweep at daemon start of rows whose cgroup id names no
  control group under the service's own subtree.
- **What it costs promise 2:** the downgrade named above, and this is where it
  is paid. `the_boundary_decides_and_forgets.rs` stops asserting *two maps* and
  asserts *three, and the third holds nothing after an ordinary day of opens,
  reads, writes, changes and datagrams by programs that are not a turn*, beside
  the assertions it already makes. The test that proved the programme *cannot*
  becomes the test that proves it *does not*, and this ADR says so rather than
  letting the header say *nowhere* when there is somewhere.
- **What the programme does when it cannot write:** a full table is a write
  that fails, and a decision does not change because of it — the boundary still
  refuses what it refuses. The failure sets one word in the turn's `BOUNDS`
  entry, the daemon reads it at the end of the turn, and the record says the
  observations were incomplete. The refusal path is the one that must be
  tested first.
- **What it does not put in the kernel's hands:** a path. `bpf_d_path` is not
  used, so the measurement Option A owes is not owed here, and the strongest
  reading of ADR 0015's warning — that the mechanism which could watch
  everything must never be handed a way to write down *names* — survives:
  the kernel writes numbers about turns, and only a program that already knew
  the names can turn them back into any.

### Option D — the record stays the daemon's account of what it did inside a boundary, and the promise is reworded to say so

No map, no field. What makes today's record more than an audit log is the
boundary itself: every path a `ran` entry names is one the kernel would have
refused outside the grant, every refusal is the kernel's, and since task 15 a
turn with no boundary is written down as one. The two v0.5 lines that promise
an observation are reworded by the owner to promise what this delivers — *the
record is the daemon's account, and the kernel guarantees that nothing in it
was outside the grant and nothing outside the grant was reached*.

- **What it buys:** nothing is built, and the structural form of promise 2 is
  untouched.
- **What it costs a person reading, the record file and the loader:** nothing;
  each stays exactly as it is today.
- **What it costs:** the promise. *What a turn touched is what the kernel
  watched it touch* becomes *what a turn touched was inside what the kernel
  allowed*, which is a weaker sentence to a security team: it answers *could
  it have reached the key* and not *did it read the invoice it said it read*.
  This is a narrowing, it is named as one, and it is the owner's to make or
  refuse; a worker may not take it.
- **Not recommended**, and listed because it is the honest name for doing
  nothing, which is what has happened since ADR 0015 was accepted.

## The recommendation

**Option C**, in the table shape, built in this order and no faster:

1. The third map and its pin, in `crates/alo-bounding-kernel` and
   `crates/alo-boundaryd`, with `the_boundary_decides_and_forgets.rs` rewritten
   in the same change to count three maps and assert the third is empty after
   an ordinary day. The change that adds the map is the change that proves it
   is not used outside a turn; there is no gap between the two.
2. The programme writing identity and count, only after a successful lookup,
   with the full-table word set in the turn's entry. Its refusal path — the
   table full, the boundary still deciding, the word set — tested on a real
   kernel before its happy path.
3. The daemon reading and removing its turn's rows at the end of the turn, the
   comparison against its own account, the `watched` entry in `alo-record`
   (additive; `docs/contracts/record-file.md` gains the paragraph), and
   `alo-recounting` showing the sentence. A turn whose rows could not be read
   writes the third sentence rather than the first.
4. The sweep at daemon start of rows whose control group is gone.

**This is not narrowing the promise.** The reworded sentence Option D would
need is not needed here: the record's paths remain the daemon's words, which
was always going to be true because the kernel has no words, and each of them
is now held to what the kernel saw. What the recommendation gives up is the
structural form of promise 2, for the behavioural form, and that is the cost
this ADR exists to put in front of the owner in one sentence: **after this, the
programme has somewhere to write, and a test rather than the absence of a
place is what keeps it writing only about turns.** If the owner judges that
cost too high, Option B is the fallback that keeps the structure and delivers
the count, and Option D is the honest wording to put beside it.

## Consequences if it is accepted

- **`crates/alo-bounding-kernel`** gains one map and one write per decision
  inside a turn; nothing outside a turn changes, and a decision never waits on
  a write.
- **`crates/alo-boundaryd`** pins a fourth object with the map of turns' mode
  and group; its *three pins* become four in `pinned.rs` and in ADR 0018's
  interface paragraph, additively.
- **`crates/alo-bounding`** reads and removes rows by cgroup id at the end of
  `Turns::doing`, and `the_boundary_decides_and_forgets.rs` says three where it
  said two, with the emptiness assertion beside it.
- **`crates/alo-record`** gains `Happened::Watched`;
  **`docs/contracts/record-file.md`** gains its paragraph; `format` stays `1`.
- **`crates/alo-turn`** compares and writes; **`crates/alo-recounting`** reads
  it back as one sentence under the turn's lines, in every language the
  machine speaks.
- **`docs/features.md`** is unchanged. The audit in
  `docs/autonomy/kernel-enforcement-plan.md` moves *kernel-sourced enforcement
  records* from *needs a decision* to a task with acceptance criteria, and the
  first of them is the refusal path.
- **`docs/hardware.md`** gains nothing: a hash map needs no kernel feature the
  two existing maps do not.

## Consequences if it is rejected in favour of Option B

- No new map, no new pin; `alo-bounding-map` gains the counter words; `ran` and
  `stopped` gain an additive field; the forget test is unchanged.
- **The two observation promises are reworded by the owner**, because B does
  not deliver them, and the rewording is Option D's sentence with B's count
  beside it. That is a narrowing and must be made in the open.

## What this does not decide

- **Whether the kernel ever renders a path.** That is Option A's measurement
  and a later ADR if anybody wants it; C is built so as not to need it.
- **Undo.** ADR 0013's snapshot at turn start is a different mechanism with a
  different release, and the table here is not a journal to rewind from.
- **Export.** v1's SIEM export reads the record file; a `watched` entry is one
  more kind for it to carry and nothing about the transport.
- **Anything about applications an agent drives.** ADR 0013's *boundary of the
  boundary* stands: what an application does under its own sandbox is not
  observed by this programme and is not claimed to be.

## What the code waits on

**Nothing is built in the change that adds this ADR**, and the test named in
the status line holds the programme to two maps until the line changes. A
worker who finds this file still saying *proposed* and a task asking for the
third map has found a plan that got ahead of its decision, and should stop at
the plan rather than at the map.
