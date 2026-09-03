# Contract — what a client asks `alo-agentd`, and what it is told

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is what goes across the daemon's socket in both directions: the messages a
shell and an agent send, and what the daemon says back. Third parties build
against it — a workspace client, a shell that is not ours, an agent runtime
somebody else wrote — so it is a public surface from the first version rather
than from the version somebody outside starts depending on it.

Read `docs/decisions/0001-the-capability-model.md` first, and
`docs/contracts/agent-verbs.md` beside it: what a verb *is* belongs to those, and
this describes only how one is asked for and what it answers with. `crates/alo-protocol` is this document
as working code.

## The shape

One message is one line of JSON, compact, with no newline inside it. A JSON
string escapes control characters, so one message is always exactly one line —
including one carrying a file that has line breaks in it.

```
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
{"format":1,"asks":{"approve":{"number":7}}}
{"format":1,"tells":{"did":{"read":{"text":"March\n4180.00"}}}}
{"format":1,"tells":{"proposed":{"number":7,"sentence":{"text":"rename march.pdf to march-final.pdf","came_from":"the-source"},"lapses_in":300}}}
```

| Field | Meaning |
|---|---|
| `format` | Which shape the message is in. Required. `1` today. |
| `asks` | A request, going to the daemon. One of the names below, and nothing else. |
| `tells` | An answer, coming back. One of the names below, and nothing else. |

**A message names its direction**, which is why the two field names differ. A
client that read an answer as a request would be a client that anything on the
machine able to open a socket could hand one to.

A field nobody declared — in the envelope, in a request or in an argument — is
**refused rather than ignored**. A client that asked for something this machine
does not do is told so, rather than having the part it cared about dropped.

## The two sides, and why a message cannot cross

There are two kinds of caller and they do not share a list.

**An agent, during a turn**, may ask for `read`, `propose` and `ask`, and is
told `did`, `proposed`, `answered` and `refused`.

**A person's shell** may send `approve`, `decline` and `waiting`, and is told
`did`, `waiting`, `declined` and `refused`.

If one door took both, the side that proposed a change could approve it, and
ADR 0001 §5 — one approval, one execution, given by a person — would be true of
the capability model and false of the socket in front of it. So `approve`
arriving on an agent's connection is refused, in words, and so is `read`
arriving on a person's.

**The answers divide the same way, and for a second reason.** What is waiting
is the person's own list, and `alo_turn::Turning::waiting_at` is a method a
daemon holding an agent's connection can call. One answer type would be one
where writing that onto the agent's connection compiles; two make it
impossible. So `waiting` and `declined` arriving at an agent are refused, and so
are `proposed` and `answered` arriving at a person's shell — and a client
meeting either is meeting a fault in alo OS rather than in whatever asked.

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

## What a person sends

```json
{"approve":{"number":7}}
{"decline":{"number":7}}
{"waiting":{}}
```

`number` is the number the change was waiting under. It is **a number and not a
handle**: the daemon finds it among the changes actually waiting, and one naming
nothing is refused. An answer to a stale list fails rather than landing
somewhere it was not aimed.

There is no shape here for approving two things, everything from an agent, or
whatever an agent asks next. An approval is of one sentence (ADR 0001 §5).

`waiting` asks what the person has been asked and has not answered. It **carries
nothing**: what is waiting is what this turn has put to this person, and a field
naming an agent, a number or a moment would be a way to ask about somebody
else's. It is on this door because the list is the person's; an agent asking for
it is refused in the same words as an agent trying to approve something.

## What comes back

```json
{"did":{"listed":{"things":[{"name":"march.pdf","kind":"file","bytes":4180}],"could_not_be_named":0,"cut_short":false}}}
{"did":{"read":{"text":"March, 4180.00"}}}
{"did":{"found":{"files":["/home/anna/Invoices/march.pdf"],"could_not_be_named":0,"cut_short":false}}}
{"did":{"renamed":{"now_at":"/home/anna/Invoices/march-final.pdf"}}}
{"did":{"moved":{"now_at":"/home/anna/Archive/march.pdf"}}}
{"did":{"archived":{"at":"/home/anna/Archive/2026.zip","things":12,"left_out":1,"bytes":40960}}}
{"proposed":{"number":7,"sentence":{"text":"…","came_from":"translation"},"lapses_in":300}}
{"answered":{"text":"Three are unpaid.","came_from":{"text":"by Mistral, in the EU","came_from":"translation"},"model":"mistral-small-latest"}}
{"waiting":{"changes":[{"number":7,"sentence":{"text":"…","came_from":"translation"},"lapses_in":300}]}}
{"declined":{}}
{"refused":{"text":"@files has not been granted the folder /home/anna/Secrets — grants are made by picking a folder, never by asking for one","came_from":"the-source"}}
```

`did` is `alo_files::Answer`'s six shapes: what a read found, and what a change
did once the person approved it. **Every answer that was bounded says it was
bounded** — a listing and a search carry `cut_short`, an archive carries
`left_out` — because a bounded answer that does not say so reads exactly like a
complete one.

`proposed` and `waiting` carry the number **and the sentence it stands on**. A
number alone would let a shell offer *approve change 7*, and what a person
approves is a sentence (ADR 0001 §5). `lapses_in` is seconds, and is absent once
the question has stopped standing.

`answered` carries where the answer came from, and there is no shape without it:
`docs/features.md` promises *where the answer came from is said where the answer
appears*, and this is the last boundary at which that could be lost.

`declined` carries nothing about why, because nothing was asked.

### Every sentence says whether anybody translated it

A sentence crosses as `{"text": …, "came_from": …}`, where `came_from` is
`translation`, `the-source` or `no-sentence`.

The daemon holds the vocabulary, so the daemon renders — and text alone would
have thrown away the one thing `alo-strings` exists for: a Latvian shell shown
English with nothing anywhere knowing it had happened. `the-source` says nobody
has translated this yet; `no-sentence` says the daemon asked for a string
nothing declares, which is a bug in alo OS rather than a translation nobody has
done, and is the one case where a client is shown a key.

### A path that cannot be shown is counted, never dropped

`alo_files::Answer` carries `PathBuf`s, and a path is not always text: it is
bytes on Linux and ill-formed UTF-16 on Windows. A format that assumed
otherwise would fail on somebody's filename rather than on nobody's, and what
that person would see is a read that succeeded arriving as an error.

So a path crosses only if it can be shown — spellable in Unicode, and with
nothing in it that could rewrite the answer around it, which is the rule
`alo_files::Named` already holds a *name* to. What cannot be shown is **counted**
(`could_not_be_named`) or, for a change that names one path, left out while the
change is still reported: the file really was moved, and saying it failed would
be untrue about the disk.

**A file's contents are not held to that rule.** Contents are contents rather
than a name inside a sentence, so a file with a tab, a line break or a terminal
escape in it crosses as it is — JSON escaping is what keeps the message one
line.

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

A request is at most **1 MiB**, in bytes, and one longer is refused before
anything is parsed. A client that can make a privileged service allocate without
a bound has taken the machine away from its owner without ever being granted
anything.

An answer is at most **8 MiB**, and the number is derived rather than chosen: the
largest thing an answer can carry is a file's contents, `docs/contracts/agent-verbs.md`
bounds a read at a megabyte, and JSON writes a control character as six bytes —
so a megabyte of them is six on the wire. One bound for both directions would
have been a bound a legitimate read cannot fit inside, and a verb that succeeded
would have produced a message no client is allowed to read.

Nothing this machine can answer with is longer than that, so an answer that
exceeds it did not come from an alo OS verb.

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

**A new request or a new answer is additive and does not raise `format`.** An older daemon
cannot parse a name it has never heard of — and what it does is *refuse* that
one message, in words, which is the only thing it could safely do with a message
it does not understand. A client meeting an answer it has never heard of does
the same, and what it must not do is guess: an answer nobody can read is a
message that did not arrive, and a client that treated it as *nothing happened*
would be a client that says nothing happened when something did. Raising `format` instead would tie the protocol's
version to the growth of the capability model, so that every client stopped
working the first time alo OS learned to do something new. The record file's
contract makes the same argument about a new kind of entry, for the same reason.

A message whose `format` is lower than this version writes names a format no alo
OS ever wrote, and is refused as that.

## Refusals

A message that is not a request is **refused in the reader's own language, and
never dropped**. A privileged service that answers silence is one nobody can
tell apart from one that has stopped.

There are nine, and `crates/alo-protocol`'s `words.rs` is where their sentences
live. Five are about the envelope and hold in both directions: too long, more
than one message, from a newer alo OS, not a format anything wrote, not readable
at all. Two are about a request on the wrong door — not for an agent, not for a
person — and two about an answer on the wrong one.

**A refusal the daemon makes about a request is not one of these.** A call that
never formed, the grants at the moment of execution, a full disk, a question
nothing answered: all of those are worded by the crate that made them and cross
as `refused`, with the sentence and its provenance and nothing else. Which
refusal it was is deliberately not on the wire — a client that could branch on it
is a client that would, and an agent choosing what to try next from *the grants
said no* is an agent working around the capability model.

**A refusal never quotes the message back.** What arrived is text nobody has
checked, and repeating it would put it in front of a person — `alo-record`'s
*the arguments of a call that never validated are never kept*, one step earlier.
The numbers a reader might want (how long the message was, what format it
claimed) are carried beside the sentence rather than inside it.

## The transport

A **Unix domain socket**, one per signed-in person, at

```
$XDG_RUNTIME_DIR/alo/agentd.sock
```

Both names are part of this contract: `alo` for the directory and `agentd.sock`
for the socket. A client finds this machine's daemon by that path and by nothing
else — there is no port, no announcement on the network, and nothing to
discover. `crates/alo-agentd`'s `place.rs` is this paragraph as working code.

**The directory is `0750` and the socket is `0660`**, both owned by the person
and both handed to the group the agent is in. Nobody else on the machine can
reach the socket at all — not to connect, not to see whether it is there. The
daemon makes the directory itself, with that mode from the moment it exists, and
refuses to start rather than use one that is a symbolic link, is not a
directory, or belongs to somebody else: whoever owns the directory a socket
lives in can replace the socket, and every client on the machine would then be
talking to them.

**A message is a line.** One request or one answer per line, terminated by a
newline, with no newline inside it — which is free, because a JSON string
escapes control characters.

### Which door a connection is on

The two sides above are **two Unix users**, and which one a connection is on is
answered by `SO_PEERCRED` on the accepted socket: the process, user and group
the kernel recorded when the connection was made. Nothing a client sends takes
part in that decision, and there is nothing it could send — the credentials are
the kernel's account of the caller, not the caller's account of itself.

- the **agent's** user gets the agent's door;
- the **person's** user gets the person's;
- anybody else is a stranger, and the connection is **closed with nothing
  written on it**. That is the one place this document's *never dropped* does
  not apply, and the reason is that it is about messages from the two clients
  this machine has doors for. A stranger has sent no message, and an answer
  would tell whoever is knocking that there is an alo OS daemon here and what
  version it is.

A machine on which the person and the agent are **one** login has no socket at
all: the daemon refuses to start, because on such a machine both doors would be
one and the side that proposed a change could approve it. The agent may not be
root, for ADR 0001 §2's reason.

The process id the kernel reports **decides nothing**. It is there for whoever
is reading a service log; a process id is reused, so a door that turned on one
would turn on whatever started next.

## A turn is a connection

**A turn begins when an agent connects and ends when that connection closes.**
Nothing on the wire says either, which is why there is no message for it: a
number naming a turn would be a number an agent could change, and a turn that
outlived the connection that opened it would be a grant nobody could see the end
of. A client that wants a second turn opens a second connection.

**One turn at a time, therefore one agent at a time.** A second agent connecting
while a turn is under way is refused in words and closed, and so is a second
shell on the person's side. Both sentences are `crates/alo-agentd`'s `words.rs`;
they are refusals a client shows its person, so they are translated like every
other.

An agent that connects while the *previous* agent's turn is ending is **not**
refused: it waits for the round that ends the turn and then gets a turn of its
own. A connection is never adopted into a turn somebody else's invocation began,
because the grant that turn holds was made for that invocation.

**A turn is begun with nothing offered.** No document, no window, no selection —
ADR 0001 §4's context arrives at the moment of invocation and there is no
compositor here yet to answer what is in front of the person. An agent gets what
it was granted and nothing more.

**Every answer is at one moment.** The service reads a clock once per round, so
two messages answered in the same round cannot disagree about whether a grant
had expired between them.

## What stops the service

A message that is not a request, a stranger at the door, a caller that hangs up
mid-message: all of these are answered or closed, and the service goes on. Four
things stop it, and each is the machine rather than a client:

- somebody asked it to stop;
- a turn could not be begun at all — a machine that named no agent, or named a
  turn lasting no time;
- the kernel would not let it wait on its own socket;
- **something happened and could not be written down.** `CLAUDE.md` asks that
  every execution and every refusal leaves a record, and a service that went on
  acting once it could not write one would be doing exactly what that sentence
  exists to prevent.

In the first three nothing has been done that is not in the record. In the
fourth, one thing has, and that is what the stop is about.

## What is still owed

There is no `main`. Which directory the socket goes in, which two users this
machine has, which model answers a question, how long a turn and a proposal
last, and the refusal to run as root at all (ADR 0001 §2) are what a machine
says about itself, and nothing in this repository reads that yet. Until it does,
a question put to a model is refused in words: *nothing on this machine has been
chosen to answer questions*.
