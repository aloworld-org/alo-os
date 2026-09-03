# Contract — the record file

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file `alo-agentd` writes what an agent did into, and what a security
team's tooling reads. `docs/features.md` promises records exported to a SIEM at
v1; that export reads this, so the shape is a public surface from the first
version rather than from the version somebody outside starts depending on it.

Read `docs/decisions/0001-the-capability-model.md` §7 first — what is *in* the
record is that ADR's, and `crates/alo-record` is it as working code. This
describes only how it is written down.

## The shape

One record is one file. The first line says what the file is; every line after
it is one entry, in the order it happened. Every line is JSON, compact, with no
newline inside it — a JSON string escapes control characters, so one entry is
always exactly one line.

```
{"format":1}
{"at":{"secs_since_epoch":1760000000,"nanos_since_epoch":0},"happened":{ … }}
{"at":{"secs_since_epoch":1760000060,"nanos_since_epoch":0},"happened":{ … }}
```

The file is **appended to and never rewritten**, except by a shortening. Three
things follow from that, and they are the reasons for it rather than
consequences somebody noticed later:

- writing an entry does not read the file, so a machine does not spend the day
  rewriting a year;
- a write the machine interrupts costs the entry being written and nothing
  before it;
- a reader that is not alo OS needs a JSON parser and nothing else.

Each entry is flushed and synced before the write answers. An entry sitting in a
page cache when the machine loses power is an entry that never happened, and the
entry lost to a crash is disproportionately likely to be the one an incident is
about.

## The first line

| Field | Meaning |
|---|---|
| `format` | Which shape the file is in. Required. `1` today. |
| `since` | The moment the record now starts at. Present only once something has been removed. |
| `under` | The retention rule that removed it: `"forever"`, or `{"for-days":n}`. |

**`format` is what tells the first line from an entry**, which has no such
field. A file whose first line is not a head is not a record and is never
appended to — a record that had lost its beginning would read as one nothing had
ever been removed from.

**`since` cannot age out.** It is the record's own statement that it does not go
all the way back, and it is in the first line rather than in an entry precisely
so that a later shortening cannot remove it. A record shortened twice still says
it was shortened. Without it, a machine whose evidence has aged out and a
machine that did nothing are the same file.

## What an entry says happened

Every entry carries `at` and `happened`. `happened` is tagged with what kind of
thing it was — `ran`, `stopped`, `turned-away`, `answered-here`, `left`,
`held-back`, `left-on-its-own` — and the fields under it depend on that tag.
`crates/alo-record` is the shape as working code; ADR 0001 §7 is why each of
them is kept.

**Every entry names whose authority it was under, except one.** `agent` is
present on all of them but `left-on-its-own`, which is alo OS reaching the
network with nobody having asked it to: signing somebody in, fetching a model,
checking for an update. There is no name in that position and there is not going
to be one — nobody granted the system anything, so a name there would be an
authority the record invented, and it would appear in a *who did what* column
beside agents that really were granted something. A reader looking for what the
machine did on its own looks for the entries with no `agent`.

## Versioning

`format` is `1`. Anything that would stop this version reading a record
correctly raises it; anything additive does not.

**A record whose `format` is higher than the reader knows is refused, not
appended to.** Adding a line in a shape the writer does not understand would
leave a file neither version can read. A reader that does not recognise a field
inside an entry ignores it, which is what makes additive changes additive.

**A new kind of `happened` is additive, and does not raise `format`.** This is
the one that needs saying, because it looks like the opposite: an older reader
*cannot* parse a tag it has never heard of. What it does is report that line as
one it could not read, with its line number, alongside everything it could —
which is the rule below, and this is the case it exists for. Raising `format`
instead would make the whole file unreadable to that reader rather than one line
of it, and would tie the record's version to the growth of the capability model,
so that a security team's tooling stopped reading a machine's record the first
time alo OS learned to do something new.

Two things make that safe rather than merely tolerable. **An older writer is
never endangered**: the file is appended to and never rewritten, so it goes on
writing its own entries beside ones it cannot read, and loses nothing. And **an
older shortening refuses**, because a record with a line it could not read is
never shortened — so the version that does not understand an entry is also the
version that will not remove it.

## Reading one

A reader that skips what it cannot parse is a reader that can be made to lose an
entry by corrupting one line. So:

- **A line that cannot be read is reported, with its line number**, alongside
  everything that could be read.
- **The last line, unparseable and with no newline after it**, is a write the
  machine interrupted. It is ordinary, and different from the case above.
- **A missing file is not an empty record.** A machine that has done nothing and
  a machine whose record was deleted are not the same thing, and a reader that
  answered *nothing happened* for both would be believed.

## Shortening it

Removing anything takes a rule and a moment, and there is no way to name an
entry, an agent or a day to remove. What is removed is everything that happened
before the rule's window, `since` moves forwards to the window's edge, and the
replacement is written whole and synced beside the record before it is renamed
over it — so a machine that loses power partway leaves the record exactly as
long as it was.

**A record with an unreadable line in it is not shortened.** Rewriting the file
would tidy away the one thing somebody needs to look at.

## Where it lives, and when it is shortened

Not here, and not in `alo-keeping`: the path and the timer are `alo-agentd`'s,
which does not exist yet. What this contract fixes is the file, so that whatever
writes it and whatever reads it later cannot disagree about what it is.
