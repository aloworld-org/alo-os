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
`never-put-anywhere`, `grants-not-read-again`, `paired`, `workspace-opened`,
`not-bounded`, `left`,
`held-back`, `left-on-its-own`, `updated`, `rolled-back`, `brokered`, `slept-through` — and the fields under it depend on
that tag.
`crates/alo-record` is the shape as working code; ADR 0001 §7 is why each of
them is kept.

**Every entry names whose authority it was under, except eight.** `agent` is
present on all of them but these:

- `left-on-its-own`, which is alo OS reaching the network with nobody having
  asked it to: signing somebody in, fetching a model, checking for an update;
- `grants-not-read-again`, which is the machine being told by the person's own
  side that what they granted had changed and not being able to read its list —
  so it went on serving under the list it already had, and `why` is the sentence
  the person was shown;
- `paired`, added 2026-09-14 and additive, which is a pairing with another
  machine being kept on this one (ADR 0003): the second of two people's
  confirmations arrived, and `with` is the other machine's identity as the
  pairing spells it — no person had named it yet, and no name is invented. A
  proposal refused, withdrawn or left to lapse writes nothing, because a
  proposal is not something that happened to the machine.
- `workspace-opened`, added 2026-09-14 and additive, which is the person opening
  a workspace discovery found (ADR 0003): `workspace` is the identity they named
  it by and `answers_at` is the one address it answered from when the machine
  looked, as `address:port`. It is written before that address is handed to the
  person's session, and a request refused — not an identity, nothing answered,
  more than one address answered — writes nothing, because nothing was handed
  anywhere. It is not a departure: the machine's service dialled nothing, and
  what connects is the workspace client under the person's own account. No agent
  can send the request, so no agent is named.
- `updated`, added 2026-09-15 and additive, which is this machine starting on a
  different build of its system than the one it ran before: `from` and `to` are
  the two builds by content digest (`sha256:` and sixty-four lowercase
  hexadecimal characters), as the base reported them. It is written at the first
  start on the new build and never when an update is merely staged, because an
  update nobody restarted into has not happened to the machine. The person chose
  when it applied and no grant was asked, so no agent is named; and it is not a
  departure — fetching the build was its own errand, and starting on it reached
  nothing.
- `rolled-back`, added 2026-09-15 and additive, which is this machine starting
  on the build it ran before because the person asked it to go back: `from` is
  the build it left and `to` the earlier one, spelt as `updated` spells them. It
  is written at the first start on the earlier build, and only when that build
  is the one the person chose to go back to — which the machine notes before the
  base is told anything — so a return is never recorded as an update, nor an
  update as a return. No agent is named and it is not a departure: the earlier
  build was still on the disk, and nothing was fetched.
- `brokered`, added 2026-09-16 and additive, which is the privileged broker
  (ADR 0001 §2, `crates/alo-broker`) answering one request for a system verb —
  printers, the network, updates, storage. It is written by the broker before it
  answers, for every request it receives. `verb` is the broker's own name for the
  verb, and is absent when the request was never read as far as one;
  `from_approval` is the approval the request's token was genuinely issued for,
  and is absent when there was no genuine one; `refused` is absent when the verb
  was handed on to be carried out, and otherwise one of `not-the-agent-service`,
  `not-a-request`, `not-one-of-its-verbs`, `not-approved`, `approval-spent`,
  `approval-lapsed` or `not-carried` — a tag, never a sentence. No agent is named:
  the broker is told an approval and not whose grants it was proposed under, and
  the turn's own entry for the same approval already names that. It is not a
  departure.
- `undone`, added 2026-09-18 and additive, which is the person putting back what
  an agent had changed, or asking for that and being told it could not be done.
  It is set out in full below; what belongs here is why it names nobody. An undo
  is the person's own act, there is no verb that undoes and none that proposes
  one, so there is no agent to name and no field to name one in.

There is no name in any of these positions and there is not going to be one.
Nobody granted the system permission to sign somebody in, nobody granted it
permission to hold the person's own list of grants, two people made a
pairing rather than any agent, a person opened a workspace no agent could
open, and a person put something back that no agent may, so a name there would
be an authority the
record invented — and it would appear in a *who did what* column beside agents
that really were granted something. A reader looking for what the machine did
with nobody's authority looks for the entries with no `agent`; the eight are
told apart by their tags, and only the first of them reached the network.

**`undone` is the person putting back what an agent changed**, added 2026-09-18
and additive ([ADR 0045](../decisions/0045-what-undoing-rewinds-to.md) point 4).
It carries `undid`, the moment of the entry it put back; `what`, that entry's own
account of the call — verb, effect, the sentence the person approved and the
arguments — **copied in rather than pointed at**, because the file is shortened
and a position in it is not a name that lasts, so an undo that pointed at one
would become a line about nothing the day the original was pruned; and `failed`,
absent (`null`) when it happened and otherwise the sentence the person was shown
when it did not. A failed one is a refusal and a reader looking for what was
refused finds it. **No `agent` and no field for one**: an undo is the person's
act, there is no verb that undoes and none that proposes one, and a name in that
position would be an authority the record invented. It is not a departure and it
is not an execution of a verb: what ran was the person's own act on their own
files, and the `what` it carries is what was undone rather than what ran, so a
question about executions never answers with one.

**It is written only from an entry that `ran`.** A change a person declined
carries a `what` as well, and an `undone` written from one would be a record of
putting back something nobody did; a pairing, a departure, an errand and the
machine updating carry nothing of an agent's at all. An `undone` naming one of
those, or naming another `undone`, is a file this repository never writes.
**Whether what ran could be put back is not the record's question**: the closed
table of what can never be undone, and why, is `alo-keeping-up`'s
([ADR 0045](../decisions/0045-what-undoing-rewinds-to.md) point 1), and a reader
of this file learns from an `undone` entry that something was put back, never
that it could have been.

**`slept-through` is a turn the machine went to sleep in the middle of**, added
2026-09-17 and additive. It is written when the machine wakes, through the
turn's own door and before the turn does anything more, so work a person asked
for is never silently lost to a closed lid (`crates/alo-sleeping`). It carries
`agent`, whose turn it was, and `stopped`: absent (`null`) when the turn
carried on, and otherwise the sentence the person was shown when it was stopped
— today, that its time ran out while the machine was asleep. A stopped one is a
refusal and a reader looking for what was refused finds it; neither is a
departure, and neither holds a call.

**`not-bounded` is the machine's own refusal**, added 2026-09-12 and additive.
ADR 0015's *a turn whose boundary cannot be applied does not run* used to leave
no entry, on the argument that nothing had happened; it does now, because a
record that kept every refusal but the machine's own showed a boundary that had
gone as a record that simply stopped. It carries `agent`, `why` — the sentence
the person was shown — and `machine`, what the machine said about its own
boundary in the words whoever administers it reads: which pin was gone, or
which map was not the one the service opened. It holds no call, no verb, no
approval and no grant: a file verb had a call and a question had none, and a
shape with room for one would have a refused question wearing a call's
clothes. A reader looking for what the machine refused of its own accord looks
for this tag.

**`origin` names the machine an entry was caused from**, added 2026-09-13 and
additive. ADR 0003: a verb from a paired machine is evaluated against the
receiving machine's grants and *recorded there, with the origin machine
named*. Any entry may carry `origin`, a string: the name the person on this
machine gave the other machine when they paired with it. It is absent — not
present and empty — on everything caused on this machine, so a record written
before it existed reads back byte for byte, and a reader that has never heard
of it ignores it as the rule below says. It is not an authority: `agent` on
such an entry is still whose grants on **this** machine permitted or refused
the call, which for a verb from a paired machine is that machine's principal,
spelt `machine:` and its identity. A question a paired machine put to this
machine's models carries its origin inside `answered-for-another-machine`, as
it did before, and a reader asking what other machines caused here reads both.

**`told` is what the person was told an execution came to**, added 2026-09-16
and additive. ADR 0039 records a conversion *with what the copy could not
carry*, which is neither the call nor the grant, so an entry may carry `told`, a
list of strings: the sentences the person read, in order and in their language,
each one line as every string here is. It is absent — not present and empty —
when nothing was told beyond the call, so a record written before it existed
reads back byte for byte. It never changes what happened: an entry that ran
and lost something is still `ran`, and what was lost is what it was told.

**`told` is what the person was told an execution came to**, added 2026-09-16
and additive. ADR 0039 records a conversion *with what the copy could not
carry*, which is neither the call nor the grant, so an entry may carry `told`, a
list of strings: the sentences the person read, in order and in their language,
each one line as every string here is. It is absent — not present and empty —
when nothing was told beyond the call, so a record written before it existed
reads back byte for byte. It never changes what happened: an entry that ran
with losses is still `ran`, and what was lost is what it was told.

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
