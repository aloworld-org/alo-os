# Contract — what a client asks `alo-agentd`

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is what goes **in** to the daemon: the messages a shell and an agent put on
its socket. Third parties build against it — a workspace client, a shell that is
not ours, an agent runtime somebody else wrote — so it is a public surface from
the first version rather than from the version somebody outside starts depending
on it.

Read `docs/decisions/0001-the-capability-model.md` first, and
`docs/contracts/agent-verbs.md` beside it: what a verb *is* belongs to those, and
this describes only how one is asked for. `crates/alo-protocol` is this document
as working code.

**What comes back is not here yet.** A read answers with what the machine found,
a proposal with a number and a sentence, a question with an answer — and every
one of those is its own decision (`alo_files::Answer` carries paths, and a path
is not always text). It is written down when it is built.

## The shape

One message is one line of JSON, compact, with no newline inside it. A JSON
string escapes control characters, so one message is always exactly one line.

```
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
{"format":1,"asks":{"approve":{"number":7}}}
```

| Field | Meaning |
|---|---|
| `format` | Which shape the message is in. Required. `1` today. |
| `asks` | The request. One of the names below, and nothing else. |

A field nobody declared — in the envelope, in a request or in an argument — is
**refused rather than ignored**. A client that asked for something this machine
does not do is told so, rather than having the part it cared about dropped.

## The two sides, and why a message cannot cross

There are two kinds of caller and they do not share a list.

**An agent, during a turn**, may ask for `read`, `propose` and `ask`.

**A person's shell** may send `approve` and `decline`.

If one door took both, the side that proposed a change could approve it, and
ADR 0001 §5 — one approval, one execution, given by a person — would be true of
the capability model and false of the socket in front of it. So `approve`
arriving on an agent's connection is refused, in words, and so is `read`
arriving on a person's.

**Which side a connection is on is not decided by the message.** It is peer
credentials on the socket, and it is the daemon's. Nothing a client sends says
who it is: there is no `agent` field, no `as`, and no token that could be
copied.

## What an agent may ask

### `read` — something it was granted, answered inside the turn

```json
{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}
```

### `propose` — a change, put to the person in one sentence

```json
{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"name","is":"march-final.pdf"}]}}
```

Both carry `verb` — a name, looked up against the closed list this machine
offers — and `given`, the arguments.

**Whether something is a read or a change is not settled by which name was
used.** `alo_capability::Authorised::read` refuses a change and
`alo_capability::Proposal::checked` refuses a read, whichever request carried it.

### `ask` — a question for a model

```json
{"ask":{"question":"how many of these are unpaid?"}}
```

The question and nothing else. **Where it is answered is not on the wire**:
ADR 0008 puts that decision with the person, and a request naming a place would
be an agent choosing which machine its question goes to.

## What a person answers

```json
{"approve":{"number":7}}
{"decline":{"number":7}}
```

`number` is the number the change was waiting under. It is **a number and not a
handle**: the daemon finds it among the changes actually waiting, and one naming
nothing is refused. An answer to a stale list fails rather than landing
somewhere it was not aimed.

There is no shape here for approving two things, everything from an agent, or
whatever an agent asks next. An approval is of one sentence (ADR 0001 §5).

## Arguments

```json
{"named":"folder","is":"/home/anna/Invoices"}
```

`is` is **text or a whole number**, and there is no third kind: that is the
whole of what `alo_capability::Given` accepts, because it is the whole of what a
model can produce. `true`, `null`, a list, an object and a fraction are all
refused.

`given` is a **list and not an object**, and that is deliberate. An object has no
duplicates, so a message naming `file` twice would arrive as one `file` with the
JSON reader having silently chosen which — in the one place a person's approval
sentence is built from. As a list, both arrive, and
`alo_capability::CallError::SameArgumentTwice` refuses them.

A name is carried exactly as it was written. Trimming or matching it loosely is
the verb registry's decision to make or refuse (ADR 0001: identities are matched
exactly).

## What is deliberately not a request

**Nothing begins a turn, and nothing ends one.** A turn begins when the person
invokes the agent, and what the invocation offered — the window, the selection,
the open document — is answered by the compositor at that moment (ADR 0001 §4).
A request carrying a context would be an agent handing itself the grant it
wanted.

**Nothing names a turn.** Which turn a message belongs to is answered by the
connection it arrived on. A number for it would be a number an agent could
change.

**Nothing names a moment.** `now` is the machine's clock. A request that named
one could revive a grant that expired an hour ago.

**Nothing carries a command** — no shell line, no path to an executable, no
script, no expression. Not because a check refuses them, but because there is no
field for one to arrive in. That is law 2 at the one place a caller can reach.

## Bounds

A message is at most **1 MiB**, in bytes, and one longer is refused before
anything is parsed. A client that can make a privileged service allocate without
a bound has taken the machine away from its owner without ever being granted
anything.

A message with a line break inside it is refused as more than one message. The
alternative is answering the first and dropping the rest, and a service that
silently does part of what it was asked is worse than one that refuses.

## Versioning

`format` is `1`. Anything that would stop this version reading a message
correctly raises it; anything additive does not.

**The number is read before the message**, out of a shape that tolerates fields
this version has never heard of — so a client from a newer alo OS is told *that
message comes from a newer alo OS than this one* rather than told its message
was gibberish, which would send whoever holds it looking for a bug instead of an
update.

**A new request is additive and does not raise `format`.** An older daemon
cannot parse a name it has never heard of — and what it does is *refuse* that
one message, in words, which is the only thing it could safely do with a request
it does not understand. Raising `format` instead would tie the protocol's
version to the growth of the capability model, so that every client stopped
working the first time alo OS learned to do something new. The record file's
contract makes the same argument about a new kind of entry, for the same reason.

A message whose `format` is lower than this version writes names a format no alo
OS ever wrote, and is refused as that.

## Refusals

A message that is not a request is **refused in the reader's own language, and
never dropped**. A privileged service that answers silence is one nobody can
tell apart from one that has stopped.

There are seven, and `crates/alo-protocol`'s `words.rs` is where their sentences
live: too long, more than one message, from a newer alo OS, not a format
anything wrote, not readable at all, not for an agent, not for a person.

**A refusal never quotes the message back.** What arrived is text nobody has
checked, and repeating it would put it in front of a person — `alo-record`'s
*the arguments of a call that never validated are never kept*, one step earlier.
The numbers a reader might want (how long the message was, what format it
claimed) are carried beside the sentence rather than inside it.

## The transport, and the process

Not here. A Unix socket, its permissions, peer credentials, and a long-lived
service that holds a turn per connection are `alo-agentd`'s, and they need a
Linux host. What this contract fixes is the message, so that whatever writes one
and whatever reads it cannot disagree about what it is.
