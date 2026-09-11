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
thing it was — `ran`, `stopped`, `turned-away`, `answered-here`,
`never-put-anywhere`, `grants-not-read-again`, `left`, `held-back`,
`left-on-its-own` — and the fields under it depend on that tag.
`crates/alo-record` is the shape as working code; ADR 0001 §7 is why each of
them is kept.

**Every entry names whose authority it was under, except two.** `agent` is
present on all of them but these:

- `left-on-its-own`, which is alo OS reaching the network with nobody having
  asked it to: signing somebody in, fetching a model, checking for an update;
- `grants-not-read-again`, which is the machine being told by the person's own
  side that what they granted had changed and not being able to read its list —
  so it went on serving under the list it already had, and `why` is the sentence
  the person was shown.

There is no name in either position and there is not going to be one. Nobody
granted the system permission to sign somebody in, and nobody granted it
permission to hold the person's own list of grants, so a name there would be an
authority the record invented — and it would appear in a *who did what* column
beside agents that really were granted something. A reader looking for what the
machine did with nobody's authority looks for the entries with no `agent`; the
two are told apart by their tags, and only the first of them reached the
network.

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
- **A record whose entries deny its own first line is reported as not being
  what it says it is** — alongside everything that could be read, with the line
  numbers, and never as a refusal of the whole file. Two things count: an entry
  from before the moment `since` names, which no shortening leaves behind, and
  an entry from an earlier moment than the entry before it, which appending as
  things happen never writes. An entry *at* `since` is legitimate — a
  shortening keeps the entry at its own boundary — and two entries in one
  moment are ordinary. This is a **reader's rule, not a field**: nothing about
  the shape above changes, and a file alo OS wrote and shortened itself is one
  this rule is silent about.

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

Both are `alo-agentd`'s rather than this contract's. **Where** is
`record.path` in `docs/contracts/machine-description.md`, typed by whoever
stands the machine up. **When** is once an hour, in the service's own loop,
between turns and never inside one — and only on a machine whose description
sets `record.keeping` to a number of days, because a machine that keeps
everything has nothing for a timer to do.

Two things follow that a reader of one of these files should know. The first
shortening on a machine happens **before it serves anything**, so a machine that
was switched off for longer than its rule catches up before its agent does
anything. And a shortening that could not be made removes nothing at all: a
record whose first line still says it is whole is a record nothing has been
taken out of, whatever else went wrong.

What this contract fixes is the file itself, so that whatever writes it and
whatever reads it later cannot disagree about what it is.
