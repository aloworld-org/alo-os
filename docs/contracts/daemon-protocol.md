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

**A person's shell** may send `approve`, `decline`, `waiting`, `granted`, and
— since the local network — `pair`, `confirm-pairing`, `revoke-pairing`,
`pairings` and `choose-machine-to-answer`; it is told `did`, `waiting`,
`declined`, `granted`, `pairing`, `confirmed`, `revoked`, `pairings`,
`chosen-to-answer` and `refused`.

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

A question may also say **what it wants back**, and nothing else:

```json
{"ask":{"question":"…the person said: list my invoices. Your next request?","answered":"as-the-next-request"}}
```

| `answered` | Means |
|---|---|
| absent, or `"in-words"` | An answer in words. What every `ask` meant before this field existed. |
| `"as-the-next-request"` | The agent is asking a model for its own next request, which must be a line of this protocol. |

Added 2026-09-14, additively: an `ask` without the field reads exactly as it
always did, and one in words is written without it. Any other value — a place,
a schema, anything not one of these two — refuses the whole request.

It decides one thing. When the place the person chose is the model runtime alo
OS pins, a question `as-the-next-request` is put to it **held to the protocol's
envelope** — the version and exactly one of `read`, `propose` and `ask`, with
nothing about what is inside the door (ADR 0032) — and a question in words never
is. A provider, a service somebody runs on this machine and a paired machine are
asked exactly as they are asked in words. The answer comes back as `answered`
either way, in the model's own words: it becomes a request only when the agent
sends it as its next line, read and validated like any other. The record entry
is the same for both; how the model was asked is not kept.

The daemon reads that decision out of the person's own settings file
(`docs/contracts/person-settings.md`), once at the first question of each turn
— so a model chosen in Settings answers the next turn, and two questions in one
turn go to the same place. A turn that asks nothing opens no file and probes no
runtime.

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

### `granted` — what is granted has changed, so read it again

```json
{"granted":{}}
```

**A knock, and never a payload.** It carries no grant, no folder, no reach, no
duration and nothing naming which grant was revoked — there is no field for any
of them, exactly as there is no field for a command. A grant is made by a person
picking a folder (ADR 0001 §3), and the whole of what this message can cause is
the daemon reading **its own file** again, under the same rules about who may
have written it that it applies at start-up. So a message that arrived from
anywhere at all could not widen anything, and there is nothing on the wire for
it to widen anything with.

It is on this door because making and revoking a grant is the person's act. An
agent sending it is refused in words, and unlike every other message on the
wrong door, **the refusal is written down** — an agent choosing the moment its
own reach is recalculated is a thing somebody reviewing a machine wants to find,
where a malformed message is noise.

What comes back is `granted`, saying how many grants are in force after the
reading. **If the file cannot be believed, the daemon keeps the grants it
already had** and answers `refused`: a machine that emptied its list because
somebody chmodded a file would go quiet about what its agent may reach, and the
person would find out by discovering their agent can no longer read their
invoices. A file that is simply *not there* is not that machine — it is a person
who has granted nothing, or revoked the last thing they granted, and it reads as
an empty list.

### Pairing, from the person's door

```json
{"pair":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","may":["models"],"seconds":86400}}
{"confirm-pairing":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","code":"482910"}}
{"revoke-pairing":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}
{"pairings":{}}
```

A pairing is mutual, deliberate, enumerated, revocable in one action and
expiring (ADR 0003), made by two people each on their own machine — so
proposing, confirming, revoking and listing one are on **this** door, and an
agent sending any of the four is refused in the same words as an agent trying
to approve something. What crosses the network for it is
`docs/contracts/local-network-wire.md`; this is only how the person here
reaches it.

`pair` names the other machine by its **identity** — the thirty-two characters
discovery found it by — the enumerated list as the wire spells each arm
(`models`, `workspace`), and the duration in seconds, stated here rather than
hidden in a constant. **It carries nothing that could name a machine discovery
did not measure**: there is no field for an address, a port or a name, so
nothing on this wire can point the machine at anything typed. The daemon looks
for that identity on the local network at the moment, proposes to the machine
that answered at the address it answered from, and is refused in words when no
machine by that identity answers, when the identity is not one, when the list
names something no pairing can permit, or when `alo-nearby` refuses the terms.
What comes back is `pairing` — the proposal waiting, with the code known.

`confirm-pairing` carries the **code** the person was shown, so that what is
confirmed is what was compared with the other person (ADR 0031). It is refused
when nothing is waiting with that machine, when the other machine has not
answered yet so there is no code, and when the code is not the one shown. What
comes back is `confirmed`, saying what became of it: `waiting-for-the-other-person`,
`paired`, or `paired-until-a-restart` — the last when both people confirmed and
this machine could not write the pairing to its file, which is said rather than
hidden and rather than reported as a refusal, because the pairing was made.

`revoke-pairing` takes effect at once on the very next verb and the very next
question from that machine, before the file is written; `revoked` says
`revoked` or `revoked-until-a-restart`. Revoking a machine this one is not
paired with is refused in words rather than reported as a success about
nothing.

`pairings` carries nothing, for `waiting`'s reason, and answers with the whole
list — both halves.

### `choose-machine-to-answer` — a paired machine answers the person's questions

```json
{"choose-machine-to-answer":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}
```

Added 2026-09-14, additively. Where a question is answered is the person's
setting (ADR 0008), and a machine this one is paired with is one of the places.
The request names that machine by its **identity** and nothing else: no address,
for `pair`'s reason, and **no model**, because which model answers there is that
machine's person's setting.

The daemon holds the choice to the pairings it keeps — the very list that
refuses that machine's next verb and next question — and writes it into the
person's own settings file (`docs/contracts/person-settings.md`, `[answers]
machine = "<identity>"`) through the same door a settings panel uses. That file
is the one the next turn's first question reads, so the next turn's question goes
to that machine. It is refused in words, and **nothing is written**, when what is
named is not an identity (before the settings file is opened), when this session
has nowhere to keep settings, when the settings file is there and does not hold
(it is left exactly as it was), and when no pairing with that machine permits
asking its models at the moment — never paired, paired for something else, run
out or revoked, all one sentence naming the machine. An agent sending it is
refused in the same words as an agent trying to approve something: an agent that
could choose where questions are answered would be choosing where its own
questions leave for.

What comes back is `chosen-to-answer`, naming the machine by the identity that
was written down.

A choice is asked of the pairings again at every question
(`docs/contracts/local-network-wire.md`): a pairing revoked or run out after the
choice refuses the question in words, and nothing else is asked instead.

### `name-machine` and `clear-machine-name` — what the person calls a paired machine

```json
{"name-machine":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","called":"the studio machine"}}
{"clear-machine-name":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}
```

Added 2026-09-14, additively. A machine this one is paired with is named by its
**identity**, and the person gives it a name to read: `called` is what the egress
indicator, the record, the list of pairings and `chosen-to-answer` say beside or
instead of the identity from then on. `clear-machine-name` takes the name away,
and the machine is spoken of by its identity again.

**A name decides nothing** (ADR 0003). It is kept on this machine in the person's
own file (`docs/contracts/machine-names-file.md`), it never crosses to the other
machine, and nothing finds, dials or proves a machine by it — every request here
still names a machine by its identity, and a name typed where the identity goes is
refused as not a machine.

Each is refused in words, and **nothing changes**, when what is named is not an
identity; when this machine is not paired with that one at the moment (never
paired, revoked, or run out); and, for `name-machine`, when the name has nothing
in it once trimmed, is longer than 64 characters, holds a line break or another
control character, or is spelt as a machine's identity (bare or as
`machine:<identity>`, in any case) — a name that reads as one machine's identity
would put it on another machine. Each of those is its own sentence. An agent
sending either is refused in the same words as an agent trying to approve
something: an agent that could name a machine could put one machine's name on
another machine's evidence.

What comes back is `machine-named`: the machine by its identity, `called` as it
now stands — trimmed, and absent once there is no name — and `became`, which is
`kept`, or `kept-until-a-restart` when the file could not be written. The name
takes effect at once either way.

**A name goes with its pairing.** Revoking a pairing takes its name away in the
same act, and a pairing kept afresh with a machine starts with no name.

### `workspaces` — the self-hosted workspaces on the network

```json
{"workspaces":{}}
```

Added 2026-09-14, additively. Asks which workspaces discovery finds on the local
network at the moment (`docs/contracts/local-network-wire.md`, *A workspace on the
network*). It **carries nothing** — in particular no address: a workspace is found,
not configured, and there is no request on either door that names an address to be
dialled as a workspace, so a message that carries one is not a request.

The link is asked when the request arrives — who is here, and which workspaces, in
one window of two seconds — and nothing is kept between two asks. **Finding a
workspace confers nothing**: nothing is contacted, paired, proposed, granted or
written in the record because of it. An agent asking is refused in the same words as
an agent trying to approve something: the network around the person is the
person's to be shown.

What comes back is `workspaces`, in the order they answered — an empty `found` when
there are none, which is an answer and not a failure:

```json
{"workspaces":{"found":[{"machine":"aaaabbbbccccddddeeeeffff00001111","answers_at":"192.168.1.20:8443","speaks":"1","called":"the studio machine"}]}}
```

`machine` is which workspace — the identity it was advertised under; `answers_at` is
the address discovery measured off the answer with the port it advertised; `speaks`
is the version. `called` is the name the person gave the paired machine hosting it
(`name-machine`), present **only** when this machine is paired with the machine of
that identity at the moment **and** that machine answered from the same address in
the same look — an advertisement is not proven, and a workspace claiming a named
machine's identity from anywhere else is listed by the identity alone. There is no
field for standing: a workspace found is not trusted, reachable or signed in to.

### `open-workspace` — the person opens a found workspace

```json
{"open-workspace":{"machine":"aaaabbbbccccddddeeeeffff00001111"}}
```

Added 2026-09-14, additively. The person opens a workspace discovery found, **by its
identity and nothing else**. There is no field for an address, a port, a hostname or
a URL, and a message carrying any of them — beside the identity or instead of it — is
not a request. A shell does not send back the `answers_at` it drew from
`workspaces`: that list has aged, and an address remembered from it is a typed
address by another route.

The daemon, in this order, and every refusal contacts nothing and writes nothing:

1. `machine` must be an identity, or the person is told *that is not a workspace on
   this network* — before the link is asked at all.
2. The link is asked at that moment, as for `workspaces`.
3. Exactly one address must have answered as that identity. None is *no workspace
   by that identity answered on this network just now*; two or more different
   addresses is *more than one place on this network answered as that workspace* —
   an advertisement is not proven, and a claim nobody can tell apart is not settled
   by taking whichever answered first. The same address heard twice is one place.
4. The opening is written in the record as `workspace-opened`, naming the identity
   and the address (`docs/contracts/record-file.md`), **before** the answer is sent.
   A record that cannot be written stops the service, and no address is handed over.

What comes back is `workspace-opened`, in the shape one entry of `workspaces` has,
with `called` under the same rule:

```json
{"workspace-opened":{"machine":"aaaabbbbccccddddeeeeffff00001111","answers_at":"192.168.1.20:8443","speaks":"1"}}
```

**The daemon dials nothing.** It hands the address to the person's session; what
connects is the workspace client — `alo-workplace`'s — under the person's own
account, and the workspace's own sign-in answers it. No pairing is required or
consulted, and nothing here is a sign-in. An agent sending the request is refused in
the same words as an agent trying to approve something: an agent that could open a
workspace would be choosing where the person's session connects. Anything this
machine or its agents do *with* a workspace on another alo machine is a different
act, under a pairing permitting `workspace`, and is not this request.

## What comes back

```json
{"pairing":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","side":"asking","may":[{"named":"models","sentence":{"text":"…","came_from":"translation"}}],"seconds":86400,"code":"482910","confirmed":{"here":false,"there":false},"lapses_in":600}}
{"confirmed":{"became":"paired"}}
{"revoked":{"became":"revoked"}}
{"pairings":{"paired":[{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","may":[{"named":"models","sentence":{"text":"…","came_from":"translation"}}],"made_ago":60,"ends_in":86340,"may_answer_questions":true}],"waiting":[]}}
{"chosen-to-answer":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}
{"pairings":{"paired":[{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","may":[…],"made_ago":60,"ends_in":86340,"may_answer_questions":true,"called":"the studio machine"}],"waiting":[]}}
{"chosen-to-answer":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","called":"the studio machine"}}
{"machine-named":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","called":"the studio machine","became":"kept"}}
```

A proposal waiting carries the **code** and the **list** — each arm as the wire
spells it beside the sentence the person reads — because that is what the
person is shown to confirm, and a shell that confirmed without drawing both
would have removed the only check two people have against somebody standing
between their machines. `code` is absent until the other machine has answered.
`side` says which machine this one is in the proposal, `confirmed` where the
two people stand, and `lapses_in` how many seconds are left before it lapses
unanswered. A pairing carries `made_ago` and `ends_in` in seconds — never a
moment — and no address: discovery measured where the machine is, the daemon
dials it, and a shell has no use for an address it could not act on. A machine
is named by its identity throughout, and — since 2026-09-14, additively — a
paired machine the person has named carries `called` beside it on `pairings` and
`chosen-to-answer`: the name the person gave it with `name-machine`, absent
(never empty) when they gave none, and absent in anything written before the
field existed.

`may_answer_questions` (added 2026-09-14, additively) says whether that pairing
lets the person choose the machine to answer their questions at the moment the
list was made — answered by the same question `choose-machine-to-answer` and
every question to that machine are asked — so a shell offers only what can be
chosen. A list without the field reads as `false`.

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
{"granted":{"holding":2}}
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

A change in `waiting` that an agent on a **paired machine** proposed carries
`from`, the name this machine's person gave that machine — because what they
approve is the sentence, and a sentence for a change from the machine down the
corridor has to say so (ADR 0003: a change waits for the *receiving* machine's
person). It is additive: absent, not empty, for every change proposed on this
machine, so a message written before there was such a thing as a remote turn
reads back exactly as it was written. `approve` and `decline` answer such a
change exactly as they answer a local one — the number is found among what is
really waiting, one approval is one execution — with one more thing asked at
the moment of approval: the pairing, so a pairing revoked between the proposal
and the answer stops the change at the moment it would have run.

```json
{"waiting":{"changes":[{"number":7,"sentence":{"text":"…","came_from":"translation"},"lapses_in":300,"from":"the reception machine"}]}}
```

`answered` carries where the answer came from, and there is no shape without it:
`docs/features.md` promises *where the answer came from is said where the answer
appears*, and this is the last boundary at which that could be lost.

`declined` carries nothing about why, because nothing was asked.

`granted` carries a **count and no list**. What is granted is drawn by the
surface that lists it, out of the person's own file; a list here would be a
second copy of it crossing a socket for nobody to check against. The count is
what a shell needs in order to know the knock arrived and was acted on.

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
/run/alo/<uid>/agentd.sock
```

where `<uid>` is the person's login number. Both names are part of this
contract: `alo` for the directory and `agentd.sock` for the socket. A client
finds this machine's daemon by that path and by nothing else — there is no port,
no announcement on the network, and nothing to discover.
`crates/alo-agentd`'s `place.rs` is this paragraph as working code.

**This moved, and it is the only break this document has taken.** It was
`$XDG_RUNTIME_DIR/alo/agentd.sock` until 2026-09-04, and that path cannot work:
`logind` makes `/run/user/<uid>` `0700` and owned by the person, so the agent —
a login of its own, ADR 0001 §5 — was refused by the *parent* directory before
either of the two modes below was consulted. [ADR 0017] is the decision and the
reasoning; the move is taken now, before v0.01, because nothing outside this
repository speaks this protocol yet and after v0.01 the same change costs a
version and a migration.

[ADR 0017]: ../decisions/0017-the-agents-door-is-ours-and-not-in-the-session.md

**Three things own three parts of that path.**

- `/run/alo` is the **image's**, made at boot through `tmpfiles.d`, `0755` and
  owned by root. The daemon never creates it: a machine without it is told which
  directory is missing and who makes it, and starts nothing.
- `/run/alo/<uid>` is the **daemon's**, made when that person's session starts
  and taken away — the socket, then the directory — when it ends. A directory
  with anything else in it is left where it is.
- The socket is the daemon's, and goes with the directory.

**The directory is `0750` and the socket is `0660`**, both owned by the person
and both handed to the group the agent is in. Nobody else on the machine can
reach the socket at all — not to connect, not to see whether it is there. The
daemon makes the directory with that mode from the moment it exists, and refuses
to start rather than use one that is a symbolic link, is not a directory, or
belongs to somebody else: whoever owns the directory a socket lives in can
replace the socket, and every client on the machine would then be talking to
them.

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

**A question can be answered, and only on this machine.** Since item 21n the
daemon reads the person's own `settings.toml`
(`docs/contracts/person-settings.md`) and puts a question to the runtime it
finds here (ADR 0019). What a machine has no list of is **providers**, so
`answered` never yet carries a `came_from` naming somewhere else, and neither
does the egress indicator: a working day on this machine still produces zero
inference egress because there is nowhere else for a question to go.

Three refusals are what a client will meet until that changes, and each is a
different thing to fix rather than one sentence wearing three faces: *nothing on
this machine has been chosen to answer questions* is a person who has picked
neither a model nor a provider, *the model runtime is not reachable* is a person
who picked one and has nothing running, and a sentence naming their settings
file is a file that does not hold.

**No message on this socket makes a grant, and none ever will.** A grant is made
by a person picking a folder (ADR 0001 §3). Where a machine keeps them is
answered — `crates/alo-remembering`, one file the person's own side writes — and
`granted` above is how a running daemon hears that the file has changed. On a
machine where nobody has picked a folder, `read` and `propose` are answered by
the capability model in its own words, which is the capability model running
rather than missing.

**What lists and revokes them is still owed.** *See what is granted* and *revoke
it* are a surface rather than a message, and there is nothing here for a shell
to draw that list from: `granted` answers with a count and no list, deliberately.

**No organisation states a bound.** `docs/contracts/machine-description.md` has
no `SourcePolicy` key, so what an organisation permits is `None` on every
machine — which changes nothing today, because no policy refuses this machine
answering on itself. Whether the description gains one is ADR 0016's subject.
