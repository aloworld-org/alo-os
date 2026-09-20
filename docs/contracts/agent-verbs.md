# Contract — the agent verbs

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is what an agent may ask the machine to do. It is a **closed list**: if a
capability is not written here, `alo-agentd` does not have it. Read
`docs/decisions/0001-the-capability-model.md` before proposing an addition — the
rules below are consequences of that ADR, not preferences.

## The shape of a verb

Every verb declares:

| Field | Meaning |
|---|---|
| `name` | Stable identifier. Never reused for a different meaning. |
| `purpose` | One sentence, in the words a person would use. A declared string, so it can be translated. |
| `effect` | `read` or `change`. Decides whether it runs in the turn or waits for approval. |
| `args` | Typed, each with a purpose. Validated at the boundary before anything runs. |
| `requires` | Which grant must be held for this call to be possible at all. |
| `sentence` | The approval sentence, with a gap per argument, filled **from the validated arguments**. A declared string. |

Two rules that are easy to state and easy to violate:

1. **No argument is ever passed to an interpreter.** Not a path fragment, not a
   filter expression, not a "script" field. A verb that needs to run something
   builds it internally from typed arguments.
2. **The approval sentence is generated, not written by the model.** It is
   derived from the validated arguments. A `change` verb whose sentence cannot
   be generated from its arguments is refused, because an approval a person
   cannot understand is not an approval.

## The words are the declaration's, and the declaration can be translated

A verb's `purpose`, its arguments' purposes and its `sentence` are read by a
person, so **a verb is declared from `alo_strings::Word`s** — a key, the English
beside it, and the note a translator needs. There is no way to declare one from a
bare string, and that is the point: the string a translator is handed is
necessarily the string the declaration was checked against, rather than a copy
of it that a test has to keep equal. An adapter declares its own words in its own
`alo-strings` vocabulary, keyed under its own area, and hands the same constants
to `Verb::checked`.

The rule above survives that, and does not have to be re-checked in each
language: a declaration is refused if its sentence leaves an argument out, and
a translation is refused if it drops a gap the source has or invents one it
does not. Those are the same rule enforced by two crates on **one string**.
`alo-files`' `words` module is the worked example.

**Nothing renders a sentence until somebody reads it.** A `Call` carries what
names its sentence and the validated values that fill it; `Call::sentence`,
`Verb::purpose` and `Arg::purpose` each take the vocabulary the person in front
of the machine reads and answer with something that says whether anybody
translated it. The screen, the approval and the record therefore render **one
value** — never two accounts of one moment, and never a record in a language
nobody was shown.

One thing an adapter has to get right that no check can reach: every word a verb
is declared with must be in the vocabulary the adapter declares. A constant left
out of that list compiles and reaches a person as a key, in the place where the
sentence they are approving belongs. `alo-files`
(`verbs::everything_the_six_say_is_something_this_crate_declares`) is the
worked example of the test that closes it.

## Being told no

Every refusal the capability model makes is a **value**, not a sentence.
`Grants::permitting` answers with the grant that said yes or with a
`NotGranted` that carries what was asked for and, where there was one, the
grant that has run out; `GrantError`, `ArgError`, `CallError`, `ProposalError`,
`AnswerError` and `NotAuthorised` are the same shape. None of them has a
`Display`. The road to words is `said(&Strings)`, which answers with something
that says whether anybody translated it.

Two things follow, and an adapter can rely on both.

**Deciding never depends on a vocabulary.** A machine whose shell declared no
words at all refuses exactly the same things and says so with the key of the
sentence, marked. Nothing is permitted because a string table was missing.

**The screen and the record cannot disagree.** They render the same value with
the same strings, so a refusal cannot be English in one place and the reader's
language in the other. `Entry::refused` takes the strings rather than words for
that reason: there is no way to write down a sentence about something else.

A refusal this crate cannot make itself — *this path really leads somewhere
nobody granted* — is worded by whatever could ask, and arrives already said
(`Refused::worded_elsewhere`). That is the same rule seen from the other side:
the words are made once, where the question was answered.

Three errors keep their English and are `std::error::Error`: `VerbError`,
`VerbsError` and `SentenceError`. They refuse a **declaration**, so their reader
is whoever is writing an adapter at the moment their own declaration fails its
tests — a sentence in whichever language the machine happened to load is not
what that person needs. An adapter's own declaration-time errors should do the
same, and everything a person meets should not.

## What an argument can be

A closed list, like the verbs themselves. An argument is one of:

| Kind | What it takes |
|---|---|
| **path** | A full path, with no `..` in it. What a grant is usually over. |
| **application** | An installed application, by its identifier. |
| **name** | One name — a file's, a folder's, or what is being searched for — of at most the length the verb declares. One name, never a path. |
| **count** | A whole number inside a range the verb declares, both ends included. |
| **choice** | One of a list of options the verb wrote down, matched exactly by name. Each option is a name a model sends and a word a person reads — see *an option is a name a model sends and a word a person reads*, below. |

**None of them is free text**, and that is the point: a model choosing
arguments cannot compose anything, because there is no shape a composition
could arrive in. Adding a kind is a change to what an agent can express and
belongs in ADR 0001 before it belongs in an implementation.

Two consequences worth stating outright:

- **Every argument is required.** There is no optional argument. An argument
  that may or may not be there makes the approval sentence conditional, and a
  conditional sentence describes less than what will happen. A verb that needs
  to behave two ways declares a **choice** and says so in its sentence, or it is
  two verbs.
- **A value that cannot be read in a sentence is refused** — a control
  character, an escape sequence, a newline — including inside a path, where an
  operating system would allow it. The person approves the sentence, so a value
  able to rewrite what the sentence appears to say never becomes one.

## `effect: read` — runs inside the turn

Reads answer. They execute under the run's budget without a tap, exactly as in
the workspace (`alo-workplace` ADR 0047), because making a question wait for
approval is the difference between a colleague and a form.

A read still requires its grant. "Read inside the turn" is about *approval*,
never about *reach*.

## `effect: change` — waits for one approval

A change is proposed with its generated sentence and waits. What the person
approves is that sentence, and the approval covers exactly one execution of
exactly those arguments.

**An approval is never a session.** There is no "remember this", no "allow for
10 minutes", no "always allow for this application". Durable permission is a
*grant*, made deliberately, visible and revocable — not something that
accumulates from clicking through dialogs.

## Grants

A grant is the durable thing. Verbs are what may be done; grants are what they
may be done to.

- **Enumerated** — a list a person can read, not a rule they must reason about.
- **Deliberate** — created by picking a folder, or by the document offered at
  invocation. Never inferred, never widened by use.
- **Visible** — findable without hunting, showing what is granted to whom and
  until when.
- **Revocable** — in one action, taking effect immediately.
- **Expiring** — by default. A grant that outlives its reason is a bug.

There is no grant to `/`.

### And no agent reaches a terminal

A terminal is the whole machine by another road: whatever is typed into it runs.
So an agent is never granted one, and no call from an agent may name one
([ADR 0043](../decisions/0043-the-terminal-is-a-persons-and-never-an-agents.md)).
`alo_capability::A_PERSONS_OWN` is the closed list of those applications, and
three refusals hold it, additively and for an agent only:

- `Grant::checked_for` refuses the grant with `GrantError::APersonsOwn`;
- `Grants::permitting` refuses the ask with `NotGranted::Never`, whatever the
  list holds — a grant written into the grants file by hand permits nothing;
- `Call::permitting` refuses a call naming one in **any** argument with
  `NotGranted::Never`, including a verb that declared `Requires::Nothing`.

An adapter author needs to do nothing to inherit this, and cannot opt out of
it: a verb that takes an application is refused over a terminal before its own
grant is asked about. An application may still be allowed one through a portal.

### A machine may have no agent at all, and then there is no list

ADR 0009 gives setup a fourth answer — *not at all* — and it is not a mode an
adapter has to check for. The grants **live inside** that answer
(`alo_capability::Agent`), so a machine where the person declined holds no
`Grants` at all rather than an empty one: there is no `&mut Grants` to add a
grant to, and `Agent::permitting` refuses with `NotGranted::NoAgent`.

Two things an adapter can rely on. **Turning the agent off ends every grant on
the machine at once**, including the one an invocation's document made, with the
immediacy a single revoke has always had — and turning it on again brings back
an agent with nothing granted, never the folders that ended. **The record and
the egress indicator are unaffected**, because neither is an AI feature: a
machine with no agent still writes down what it did on its own, and
`alo_record::Only::ByAnAgent` is how somebody asks whether anything in their
record has an agent's name on it at all.

`NotGranted::NoAgent` is a new variant rather than a narrower
`NotGranted::Never`, and the reason is what each sentence tells a person to do:
on a machine that declined, the grants panel is **absent** rather than greyed
out, so *grants are made by picking a folder* would send somebody somewhere
their machine does not have.

### A grant is over a place, so the path is resolved before reach is decided

A grant is compared against a path **lexically** — component by component,
touching no disk, so that it means the same thing whether or not the file
exists. That leaves one thing undone, and it is the thing an attacker reaches
for: a link inside a granted folder can point outside it.

So **whatever executes a verb resolves every path the call names and asks the
grants again about where it really leads**, before it opens anything. The three
questions are asked in this order, and the order is part of the contract:

1. do the grants permit the path as it was written? If not, that is the refusal
   and **nothing is looked for on the disk** — otherwise a refusal would tell an
   agent whether a file it may not touch exists, and the verb list would have a
   side channel in it;
2. where does the path really lead;
3. do the grants permit *that*? A link out of a granted folder is refused here,
   as a refusal by the grants, and it is recorded like any other.

Two consequences an adapter author should know. **Every path a call names is
asked about**, not only the ones the verb declared its grant is over — a verb
that forgot one should not be a verb that reaches a disk. And **a grant is made
over a resolved path**: a person picking a folder grants the real one, so a
grant over a link would otherwise be a grant over wherever it points today.

What this cannot do is close the gap between the check and the open — a link
swapped in afterwards, or a hard link, which is a second real name for a file
that also lives elsewhere and which no amount of resolving reveals. Both are in
`docs/quirks.md`, and closing the first belongs to the code that opens the file.

**It is closed there, on Linux.** A file is opened by a single call that refuses
a symbolic link at *every* component of its path, so a folder on the way cannot
be exchanged after the questions above were answered. Moving a name is one call
that refuses or moves rather than a check followed by an act, and it is made
from **handles** on the two folders rather than from their names — so the folder
a file leaves and the folder it arrives in cannot be exchanged either.

**A hard link is answered by counting rather than by resolving.** It is a second
*real* name for one file, so no comparison of paths can see it and the granted
name genuinely is a name for that file. So a file is asked how many names it
has, of the handle that was opened, and **a file with more than one name is not
read** — by any verb that reads bytes, which is `read_file` and
`archive_folder`. A caller sees a failure saying the file has other names, that
it was not read, and what to do instead. It cannot be told *where* the other
names are, because a file does not know; a second name harmlessly beside the
first is refused as well, and `docs/quirks.md` records that cost. On a machine
whose `std` cannot count names this check does not apply.

The machines with no such calls keep the behaviour the paragraph above
describes. Apart from the new failure, none of this changes what an adapter
author writes or what a caller sees — the same refusals, in the same words.

A refusal at question 1 is the grants' own and travels as the value they made.
A refusal at question 2 or 3 is worded by whatever executes the verb, because
only that code knows where the path really leads — so **it is worded in the
language the person reads**, once, and that one rendering is what the record
keeps. Both roads are in *Being told no* above, and neither of them lets a
missing string table change what is refused.

### A grant covers where a file goes, not only where it comes from

The three questions above are about the paths a call **names**. A change also
creates one: `rename_file` invents a name, `move_file` and `archive_folder`
invent a full path inside a folder. A grant can be over a single file — the
document offered at invocation (§4) — and under one of those, renaming would
put a file at a name nobody granted.

So **whatever executes a change asks the grants one more question before it
touches anything: may this be created?** A no is a refusal by the grants like
any other, recorded like any other, and nothing has happened when it is
answered.

### Nothing is replaced that was not named

A person approves *move march.pdf into Archive*. They do not approve
*and overwrite the march.pdf that is already there*, which is what renaming
over an existing file silently does on most systems. So a change whose
destination already holds anything — a file, a folder, or a link, including one
that leads nowhere — is refused and says the name is taken. This is a rule about
the sentence, not about filesystems: what was approved is what happens, and
nothing else is.

On Linux the refusing and the moving are **one call**, so there is no moment
between them in which a destination could appear; a filesystem that cannot
promise that refuses the move rather than replacing. Elsewhere it is a check
followed by a move, and the gap is in `docs/quirks.md`. Either way the answer a
caller gets is the same one.

### Underneath the grants there is a floor, and it does not cover everything

On a machine whose kernel can impose one, a verb's work runs inside a boundary
the kernel holds: the places that call named, and nothing else. It is not a
second capability check and it decides nothing about authority — **inside the
bound is never the same as authorised**, `alo-capability` is what permits a call,
and the floor is what a verb with a bug in it meets. What it covers is opening a
file, moving one, removing a name and giving a file a second name: outside the
call's own places, each of those is refused by the machine with the ordinary
permission failure rather than by our code.

**It does not cover every way a filesystem changes, and an adapter author should
not read it as if it did.** *A turn cannot change anything outside its grant*
is not a sentence this contract makes. What it **does** cover, since
2026-09-12, is what a file *is*: a bounded turn cannot change the size, mode,
owner, times, extended attributes, access list or inode flags of a file
outside the call's own places. And since 2026-09-13 it covers what a turn
**makes**: a bounded turn cannot make a file, a directory or a symbolic link
in a folder outside the call's own places, and cannot remove a directory
there — each refused with the same permission failure, and each landing
inside the call's own places, which is where an archive is written. And since
the same day it covers what a turn **learns about** a file: a bounded turn
cannot ask the size, mode, owner or times of a file outside the call's own
places, by name or through a descriptor it held before the turn began; cannot
read or list its extended attributes; cannot read its access list, which the
kernel routes to a hook of its own; and cannot read where a symbolic link
there points. **An adapter may no longer assume that a path it was not
granted can be checked from inside a turn** — not for its existence by
`stat`, not for its size, not for who may read it — and a check of a path
the call *did* name is answered as it always was, because every such path is
among the turn's places. One thing a turn can still learn is named rather
than closed in `docs/quirks.md` under *What a turn reads about a file is
inside the grant*: whether a name exists, by `access(2)`. What a
bounded turn can still do on a filesystem is read a file it was handed
through a memory mapping, which is named in `docs/quirks.md` under *A
descriptor opened before a turn began is decided about on every use*; the
list of what was once unwatched and when each row closed is under *Four hooks
are not a filesystem*, and what closed is under *Attributes, ownership and
size are inside the grant*, *What a turn makes is inside the grant* and *What
a turn reads about a file is inside the grant* beside it.

Nothing here changes what a caller sees. Every refusal an adapter can receive is
still one of the ones in *Being told no*; the floor exists so that a refusal
nobody wrote still happens.

## Context on invocation

An agent is handed three things at the moment it is invoked, and only then: the
**focused window**, the **selection**, and the **open document** (ADR 0001 §4).
There is no verb that asks for any of them, and there will not be one — a verb
that could ask is a background reader with an approval dialogue in front of it.

**Only the document grants anything.** ADR 0001 §3 names two deliberate acts
that make a grant, and the document offered at invocation is one of them; a
window somebody happened to be looking at and text they happened to have
highlighted are not. So:

- the document becomes a grant over **that file** — not the folder it sits in,
  and not the files beside it;
- the focused window is **told, not granted**. An agent that knows Blender is in
  front of the person still cannot open, focus, arrange or close it until
  somebody grants it;
- the selection is text, whatever it says. A selection reading `/etc/shadow`
  reaches nothing.

**The grant a context makes is a grant like any other.** It goes into the same
list, where a person sees it beside the folder they picked on Monday and revokes
it in one action. It runs from the moment of the invocation and expires when the
turn is over, and whatever holds the turn revokes it when the turn ends — so a
turn that finishes early does not leave the document reachable for the rest of
its allotted time. A grant kept in a list of its own would satisfy none of the
five words above while still deciding what an agent may touch.

**A context is made, never read back.** It has no serialised form: something
that could be read off a disk would be a context existing without an invocation,
which is the whole of what §4 forbids. For the same reason **nothing about a
context is recorded** — what the record keeps is what the agent then *did*, and
the grant it did it under. An entry per invocation saying what was on somebody's
screen would build the watched-context log this rule exists to prevent, one
entry at a time.

**What is offered is shown to the person**, one row per part, and one row saying
nothing was offered when there was nothing. A rule nobody can check is a
promise: somebody who cannot see what they are offering has no way to tell a
system that reads three things at invocation from one that watches everything
all day.

**A selection is bounded and says when it was cut.** What is offered is at most
200,000 characters, and a selection longer than that comes with a sentence
saying how many characters were left out — because a bounded answer that does
not say it was bounded reads exactly like a complete one. Characters that cannot
be seen are removed silently, since nothing a person selected is lost with them;
the marks a right-to-left language needs are **not**, because removing those
would corrupt the text of the readers alo OS says it serves.

## The file verbs

The six `docs/features.md` promises at v0.01, over granted paths only. Every one
of them requires a grant over **every** path it names, and every path it names
is something that already exists — a new name is a **name**, never a path.

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `list_folder` | read | `folder` (path) | list what is in {folder} |
| `read_file` | read | `file` (path) | read what is in {file} |
| `find_in_folder` | read | `folder` (path), `named` (name), `most` (count 1–1000) | find up to {most} files in {folder} whose name contains {named} |
| `rename_file` | change | `file` (path), `name` (name) | rename {file} to {name} |
| `move_file` | change | `file` (path), `into` (path) | move {file} into {into} |
| `archive_folder` | change | `folder` (path), `into` (path), `name` (name) | make an archive of {folder} called {name}, in {into} |

**"Archive" means make an archive**, not move something to an archive folder.
The second is `move_file` under another name, and a closed list with two names
for one action is a list a model picks from at random. **An archive is a zip
with nothing compressed**, and `name` therefore ends in `.zip`: a name that says
otherwise is refused rather than corrected, because a file whose name lies about
what is in it and a file whose name a person did not approve are both worse than
being told to ask again.

**Every answer is bounded, and every answer says when it was bounded.** A
listing carries at most 1000 things, a read at most a megabyte, a search looks
at at most 20,000 things, and an archive holds at most 20,000 things and two
gigabytes. The first three answer with what they have *and a flag saying there
is more*; an archive refuses instead, because an archive missing the half nobody
mentioned is a file somebody keeps and finds out about later. A bounded answer
that did not say it was bounded would read exactly like a complete one, so an
adapter answering a question of its own is expected to do the same.

**A name that cannot be shown is counted, not shown.** Filenames are not written
by us: a file called `march.pdf\nran: deleted everything` would make an answer
that shows one thing and says another. A listing leaves those out and says how
many it left out — and nothing is lost that could have been acted on, because a
name with a control character in it cannot arrive as an argument either.

**There is no search expression.** `find_in_folder` takes one name and builds
the search inside itself, which is §1 at the place somebody would most
reasonably ask for a pattern language. There is no delete verb either: nothing
on this list destroys anything, and adding one goes through the scope gate like
anything else.

## The application verbs

`docs/features.md` promises four at v0.01 — open, focus, arrange, close — over
granted applications only. **All four are declared.**

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `open_application` | change | `application` (application) | open {application} |
| `focus_application` | change | `application` (application) | bring {application} to the front |
| `close_application` | change | `application` (application) | ask {application} to close |
| `arrange_application` | change | `application` (application), `where` (choice) | put {application} {where} |

`arrange_application` offers three arrangements at v0.01: `left_half`,
`right_half` and `whole_screen`. Two windows on opposite halves is *tile*, and
the whole screen is *maximise*; **quarters are v0.5** and are not offered.
Minimising is not on this list at all — this verb says where a window goes, and
*out of the way* is not a place.

**All four are changes, and there is no read on this list.** A verb that
listed the running applications or the open windows would be a background
reader, and context is offered at invocation and never watched — so what is
open reaches an agent as *context*, for that turn, rather than as something it
can ask for whenever it likes. An adapter must not add one either.

Focus is a change because bringing a window forward while somebody is typing
sends the next keystrokes somewhere they did not choose. "It only changes
something small" is exactly the reasoning rule 2 of *adding a verb* refuses.

**`close_application` asks; nothing kills anything.** It does what pressing the
close button does: the application is asked, it may put up its own *save your
changes?*, and the person answers that. A person approving *ask Blender to
close* has approved closing an application and has not approved discarding
unsaved work — everything else on this list is reversible, and that is not. The
word **ask** is therefore in the approval sentence rather than only in
documentation, and a translation that promised otherwise would be promising
something alo OS does not do.

### The identifier is approved; the name is only shown

An application has two names: the identifier this machine knows it by
(`org.blender.Blender`) and whatever its desktop entry calls it (*Blender*).
**Only the identifier is ever granted or approved.** The second is written by
whoever packaged the application, and two applications can claim the same one —
*approve: open Mail* reads identically whichever *Mail* is behind it, and an
approval sentence the approved thing can choose is not an approval. No two
applications share an identifier, so a shell shows the name **beside** the
identifier and never in place of it.

A name that cannot be shown in one line is dropped and the application stays:
nothing is ever acted on by name, so nothing is lost, and refusing the
application would let whoever packaged it decide what this machine can reach.

### An application is checked for after the grants have answered, never before

Reach is decided by matching the identifier exactly, and that leaves one
question the capability model cannot ask: **is there such an application here?**
Whatever executes an application verb asks it, and asks it **second**:

1. do the grants permit this application? If not, that is the refusal, and the
   list of what is installed is not consulted;
2. is it installed? A no is a refusal in its own words, recorded like any other,
   and nothing has happened.

The order is part of the contract, for the reason the file verbs' order is. A
refusal that answered *that is not installed* about an application nobody
granted would tell an agent what somebody has on their machine — which
applications a person uses is a fingerprint of who they are, what they do and
who they work for. Asked in this order, an ungranted application refuses
identically whether it is installed or not.

### An option is a name a model sends and a word a person reads

This is the rule for **every** verb with a choice in it, not only for
`arrange_application`.

An option is declared as two things at once. Its **name** — `left_half` — is an
identity: it is what a model sends, what the record keeps, what a script writes,
and it is matched exactly and never translated. Its **word** is what a person
reads, and it goes into the approval sentence, so it is a string somebody
translates, declared from `alo_strings::Word` exactly as a verb's own sentence
is. Neither can stand in for the other: a sentence built from the name reads
*put Blender on the `left_half`*, and an option identified by its word would let
a translation change what a verb can be called and make the record say something
different on a German machine than on a Greek one.

Three things about an option are refused where a verb is declared, and they are
the argument rules one level down: a name that is not a lower-case identifier,
one name offered twice, and an option with nothing to say.

**The words are written to complete the sentence, not to label a button.** The
preposition belongs in the option — *on the left half of the screen* — because a
language that inflects the place needs the whole phrase in front of it, and the
gap can then move to wherever that language puts it. Whoever adds a choice owes
the translator a note on both halves saying so.

**A refusal names the options by name.** `{argument} has to be one of: …` lists
what has to be *sent*, because a call that never validated is about what
arrived; and validation never consults a vocabulary, so what an agent may do
does not depend on a string table having loaded.

**A sentence is as translated as the words put into it.** A verb sentence
somebody translated with an option nobody has translated inside it answers
`alo_strings::Said::is_translated` with `false`, so a half-translated approval
line cannot pass for a finished one.

The rule holds of every sentence alo OS composes, not only of an approval, and
it holds **at any depth**: a refusal with a place named inside it is only as
translated as that place, and one with another crate's refusal inside it is only
as translated as that refusal. What decides the difference is what the gap
holds. A gap holding **data** — a path, a hostname, a window's identifier, a
colour somebody typed, a key that prints `Q` — carries no language and can never
make a line untranslated; a gap holding a **word** carries where that word came
from. So an adapter that puts one of its own strings into another one puts it in
through `alo_strings::Filling::and_said`, or through `Filling::and_composed`
where the value is assembled out of several — never as text, which reports every
half-English line as finished.

## The measurement verbs

`docs/features.md` promises three measurements of the machine at v0.5 — *search
your own files, without asking anything*, *what is running, and what it is
using*, and *what is filling the disk* — and each names a window. The agent's
*"where is that file?"* and *"why is it slow?"* are these three verbs, declared
in the crates that answer them (`alo-finding` and `alo-measuring`, each in its
`src/verbs.rs` with a `pub fn declare_into`), and **every one of them is a
read**: it runs inside the turn, nobody is asked to approve it, and the record's
entry has no approval to name.

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `search_files` | read | `folder` (path), `named` (name, at most 255 characters) | search the index of {folder} for files whose name contains {named} |
| `what_is_running` | read | `proc` (path) | list what is running and what it is using, read from {proc} |
| `what_is_filling` | read | `folder` (path) | count what is filling {folder} |

**Each requires a grant over the one folder it reads**, and nothing else. A
read inside the turn is about approval and never about reach: an index of a
folder nobody granted would be a way to read the names in it, and a process
list is a fingerprint of who somebody is and what they do.

**`what_is_running` takes the kernel's directory as an argument, and the grant
is over it.** Every number the answer holds is read from a file under `/proc`,
so the grant that permits the verb names exactly what the verb reads, and
revoking it stops the verb the same instant it stops `list_folder`. It carries
no interval: a rate is two readings with time between them, and how long a turn
waits is not the model's to set — whatever carries the verb out chooses the
interval and passes it in.

**`search_files` asks the index, never the disk.** `find_in_folder` walks a
folder and answers with what it met; this verb asks the index the file manager
asks, answers with what matched beside what the index does not hold, and
touches the folder not at all. It asks by name and by nothing else — every
argument is required, and a verb per axis would be four names for one action.
The index it is handed has to be the granted folder's, and one that is not
answers with a refusal in the index's own words rather than a search of
somewhere nobody granted.

**Neither answer depends on who asked.** `Index::answer`, `Reading::now` and
`Holding::of` take no caller, no grant and no name, and a test in each crate
holds the shape; the verb is a road to the same function, and a person in the
window takes none of it.

**These three are declared and carried out, and not yet offered by a turn.**
`alo-turn`'s machine offers the file verbs it has an executor for; adding an
executor and adding to the offered list is one edit there, and it has not been
made. Until it is, an agent on a shipped machine cannot reach these — which is
the honest state of a verb that exists and is not on the list a machine offers.

## The printing verb

`docs/features.md` promises printing at v0.5, and ★ *Printers, solved*. What an
agent may ask for is one verb, declared in `alo-printing`'s `src/verbs.rs` with a
`pub fn declare_into`.

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `print_document` | change | `document` (path) | print {document} on this machine's printer |

**It is a change**, because paper comes out of a machine in the room, and **it
requires a grant over the document**, because a file nobody granted is not one
an agent may send anywhere.

**A printer across the network makes it an egress.** A document sent to a
printer on the office network has left this machine, so carrying the verb out
takes the `alo_egress::Departing` the indicator hands out for an agent *sending
something* to that printer's address, under the agent the call was authorised
for — and refuses without one. A printer on a cable, or at this machine's own
address, is not a departure. An organisation whose policy lets nothing leave
therefore lets no agent print across the network; a person printing their own
document is not an agent's egress and is not stopped by it.

**It takes no argument naming a printer.** v0.5 is one printer that works, and
the verb prints on the one the printing service keeps as this machine's. A
printer argument would be machinery — a queue name — in a sentence a person
approves.

**Finding a printer is not a verb, and setting one up is not this one.** A
printer is a place documents can go, and adding one is a person choosing that
place: no call to `print_document` adds one. An agent may *propose* setting up,
removing or choosing a printer — the three printer verbs below, each a change a
person approves, carried out by the privileged broker.

**Declared and carried out, and not yet offered by a turn**, for the reason the
measurement verbs are not: `alo-turn` offers the verbs it has an executor for,
and adding this one is an edit there.

## The printer verbs

★ *Printers, solved*: setting a printer up, removing one, and choosing the one
this machine prints on. Each is a change to the whole machine, so each is carried
out by the privileged broker and by nothing else. Declared in
`alo-changing-printers`' `src/verbs.rs` with a `pub fn declare_into`;
[ADR 0055](../decisions/0055-a-printer-is-changed-by-its-sentence-through-the-broker.md)
is the decision.

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `add_printer` | change | `printer` (name, at most 127) | set up the printer {printer}, so this machine can print on it |
| `remove_printer` | change | `printer` (name, at most 127) | remove the printer {printer} from this machine |
| `set_default_printer` | change | `printer` (name, at most 127) | print on {printer} from now on |

**A printer is named by the name it gave itself**, never by an address, a queue
or a driver. When the approved change is carried out, the name is matched
exactly against the printers the printing service reports — the printers it can
find for `add_printer`, the printers set up for the other two — and **a name no
printer has, or two printers share, changes nothing**. What crosses into the
broker is the matching printer's identity (below), never the name.

**None requires a grant**, with its reason in ADR 0055 §2 and in each
declaration: a printer is neither a path nor an application, and the approval of
the sentence naming it is what makes the change, once.

**A person does the same in Settings, through the same broker verbs.** What a
person picks in the printers pane becomes the identical `printers.add`,
`printers.remove` or `printers.set-default`, crossing the same door under the
approval number `BY_HAND` (below).

**Declared and carried out, and not yet offered by a turn**, for the reason the
printing verb is not: handing a turn's redeemed approval to
`alo_changing_printers::carry_out_approved` is an edit in `alo-turn` and
`alo-agentd`.

## The network verbs

★ *System verbs through the privileged broker*: joining a Wi-Fi network,
forgetting one, and turning Wi-Fi on or off. Each is a change to the whole
machine, so each is carried out by the privileged broker and by nothing else.
Declared in `alo-changing-network`'s `src/verbs.rs` with a `pub fn
declare_into`;
[ADR 0049](../decisions/0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md)
is the decision.

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `join_network` | change | `network` (name, at most 32), `this_conversation` (choice: `keeps_its_connection`, `loses_its_connection`) | join the Wi-Fi network {network}, and {this_conversation} |
| `forget_network` | change | `network` (name, at most 32), `this_conversation` (choice) | forget the Wi-Fi network {network}, so this machine no longer joins it on its own, and {this_conversation} |
| `switch_wireless` | change | `wireless` (choice: `on`, `off`), `this_conversation` (choice) | turn Wi-Fi {wireless}, and {this_conversation} |

**A network is named by the name it announces**, and never by a device, a file
or an address. When the approved change is carried out, the name is matched
exactly against what the network manager reports — the networks in range for
`join_network`, the saved networks for `forget_network` — and **a name no
network has, or two networks share, changes nothing**. An open network using the
name of a protected one is a second network of that name. A network asking for
an organisation's sign-in is not joined in v0.5. What crosses into the broker is
the matching network's identity (below), never the name.

**No argument can hold a password.** Joining a protected network asks the person
for its password in their own session, through the secret agent
`alo_networks::secret_agent` registers with the network manager, which answers
the network manager's connection and nobody else's. The password crosses no
verb, no door and no record.

**Each says what it does to this conversation.** `this_conversation` must be the
truth: the change *loses* the conversation's connection exactly when the
conversation is answered over the network and the change takes the machine off
the Wi-Fi network it sends through now (`alo_changing_network::would`). A call
saying anything else is refused before it is proposed
(`alo_changing_network::proposable`), and an approval that is no longer true
when it is carried out is refused without asking the broker.

**None requires a grant**, with its reason in ADR 0049 §2 and in each
declaration: a network is neither a path nor an application, and the approval of
the sentence naming it is what makes the change, once.

**Not verbs, deliberately:** setting the proxy (a person sets it in Settings,
through the broker's `network.set-proxy`; ADR 0049 §3), configuring a VPN (not in
v0.5), and listing the networks nearby.

**A person does the same in Settings, through the same broker verbs.** What a
person picks in the network pane becomes the identical `network.join`,
`network.forget` or `network.radio`, crossing the same door under the approval
number `BY_HAND` (below).

**Declared and carried out, and not yet offered by a turn**, for the reason the
printing verb is not: handing a turn's call to `proposable` and its redeemed
approval to `alo_changing_network::carry_out_approved` is an edit in `alo-turn`
and `alo-agentd`.

## The converting verb

`docs/features.md` promises at v0.5 that *the documents people are actually sent
open: `.docx`, `.xlsx`, `.pptx`*. [ADR 0039](../decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md)
decides how. What an agent may ask for is one verb, declared in
`alo-converting`'s `src/verbs.rs` with a `pub fn declare_into`.

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `convert_document` | change | `file` (path), `into` (path) | convert {file} into a PDF copy in {into}, leaving the original as it is |

**It is a change, and it requires grants over both arguments**: it reads the
document and writes a new file. `into` is the folder the copy goes in; the copy
is named after the document, ending in `.pdf`, and the model never chooses that
name. **The grants are asked once more about the copy's own path** before it is
created, so a grant over the folder alone (`Reach::File`) is not a grant over
what goes inside it.

**The copy is never the original.** The document is opened read-only; the copy
is created with `O_EXCL`, and a name already in the folder is refused — nothing
is replaced and nothing is quietly renamed. A copy the conversion did not finish
is removed.

**It answers with what the copy could not carry, by name**: a font substituted,
a field fixed at its value, macros, linked content not fetched, comments,
tracked changes — or that nothing was lost, which is said only when both the
document and the copy were checked. A document or a copy that could not be
checked is a refusal, and no copy is kept.

**What converts is a closed set, and it grows only by measurement.** Today:
a Word document, an Excel workbook and a PowerPoint presentation, and an
OpenDocument text document, spreadsheet and presentation — each into a PDF. The
kind is read from the file's own bytes and never from its name, so a document
called something else converts as what it is, and a program named as a document
converts as nothing.

This set is additive and will grow: ADR 0039 makes each further kind a
registration and **a test against a real file**, so a format appears here on the
day somebody measures one and not before. A kind that is recognised but does not
convert is not a silent failure — the machine says *nothing here opens it* and
what would, rather than offering a conversion it cannot complete.

**Nothing leaves the machine.** The verb hands two open descriptors to
`alo-convertd` over a Unix socket; the service has no network and no view of any
home folder. There is no remote form, fallback or setting.

**Declared and carried out, and not yet offered by a turn**, for the reason the
printing verb is not.

## The screen verb

`docs/features.md` promises *context on invocation* and never harvesting. A
picture of the screen is the most harvest-shaped thing this machine has: one
call hands over everything a person was looking at — their mail beside their
bank, a colleague's name, a photograph on a second display. So there is **one**
verb, it is a change, and what it produces goes to the turn that asked and
nowhere else.

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `picture_of_the_screen` | change | none | take a picture of your screen |

**It is a change, so it waits for one approval**, and the sentence a person
approves is the whole of it: there are no arguments to fill in, because there is
no version of this that is narrower than *your screen*.

**It requires no grant, and the reason is ADR 0040's.** Facilities — the screen,
the camera, notifications — are granted to **applications and never to agents**:
*a durable grant to the camera would be a background reader by another name*.
`alo-capability` refuses such a grant outright. So an agent holds nothing
standing over the screen, and **the approval of the sentence is the whole of the
authority**.

It is the second verb in this contract that requires no grant, for a different
reason from the first: `install_application` needs none because there is nothing
on the machine yet to grant over; this needs none because what a grant would be
over is something an agent may never hold.

**One approval is one picture.** An approval is never a session (ADR 0001), and
this verb is where that rule is load-bearing rather than tidy: an agent that
could look again on an approval a person gave this morning is an agent watching
the screen, which `docs/features.md` forbids in the same breath as it promises
context on invocation. `alo_capturing::ForTheAgent::approved` takes the
authorisation **by value and spends it**, and refuses one that came from
anything but an approval, so a second picture needs a second approval. Declared
in `alo-capturing`'s `src/verbs.rs` with a `pub fn declare_into`.

**What the picture may reach.** The turn that asked, and nothing else: not the
index, not the record, not a file. The record says *a picture of the screen was
taken* and never what was in it.

**While the turn holds it, the in-use indicator shows the screen in use by the
agent**, in terracotta with the agent's mark and word (ADR 0010) — the same line
an application gets, in the colour reserved for the agent.

## The installing verb

`docs/features.md` promises *install applications* at v0.5. What an agent may
ask for is one verb, declared in `alo-software`'s `src/verbs.rs` with a
`pub fn declare_into`.

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `install_application` | change | `application` (application), `source` (name, at most 64) | install {application} from {source} |

**It is a change, and it requires no grant** — the one verb in this contract
that does not, with its reason in ADR 0042 and in the declaration: there is
nothing on the machine for a grant to be over, the application arrives granted
nothing, and the approval of the sentence naming the application and the place
installs once.

**The place is checked when it runs, not when it is proposed.** A `source` not
set up on the machine, outside the organisation's bound, or set up without
signature checking is refused at the moment of installation, in words, whatever
was approved; and the installation goes on the egress indicator as
*alo OS is installing an application from* the place's host.

**Updating, removing and listing applications are not verbs.** An agent only
proposes an installation; a person updates and removes applications themselves.

**Declared and carried out, and not yet offered by a turn**, for the reason the
printing verb is not.

## The verb classes

| Class | What it covers | Where it runs |
|---|---|---|
| **Files** | List, read, find, rename, move, archive — within granted paths | `alo-agentd`, as the person |
| **Applications** | Open, focus, arrange, close — over granted applications | `alo-agentd`, as the person |
| **Measurements** | Search the index, what is running, what is filling — over the granted folder each reads | `alo-agentd`, as the person; declared, not yet offered by a turn |
| **Printing** | Print a granted document on this machine's printer | `alo-agentd`, as the person; declared, not yet offered by a turn |
| **Converting** | Convert a granted document into a PDF copy in a granted folder, saying what the copy could not carry | `alo-agentd`, as the person, through `alo-convertd`; declared, not yet offered by a turn |
| **Software** | Propose installing an application from a place this machine installs from | `alo-agentd`, as the person; declared, not yet offered by a turn |
| **Context** | The focused window, the selection, the open document | Offered at invocation only |
| **Adapters** | An installed application's own verbs — `text_editor.open_document`, named under their adapter | `alo-agentd`, as the person; declared and carried out by `alo-adapters`, not yet offered by a turn. See `app-adapters.md` |
| **Accessibility fallback** | For a granted application with no adapter: read what its windows show (`accessible.read_window`, a read), and press one control named by its kind and the name it shows (`accessible.activate_control`, a change). Never a password field's contents, never a position | `alo-agentd`, as the person; declared and carried out by `alo-adapters`, not yet offered by a turn. See `app-adapters.md` |
| **Network** | Propose joining or forgetting a Wi-Fi network by the name it announces, or turning Wi-Fi on or off, saying what each does to the conversation | Carried out by the **privileged broker** (`alo-brokerd`), asked by `alo-changing-network`; declared, not yet offered by a turn |
| **Printers** | Propose setting up, removing or choosing a printer, by the name it gave itself | Carried out by the **privileged broker** (`alo-brokerd`), asked by `alo-changing-printers`; declared, not yet offered by a turn |
| **System** | Printers, network, updates, storage | The **privileged broker**, never the agent directly |

## A turn, and the order the steps happen in

The rules above are about one verb. A **turn** is what they happen inside:
`alo-turn` holds the order, and everything an agent does to this machine goes
through it. An adapter and a client can rely on four things about it.

**There is no door that takes a call.** A turn is asked for a verb by name with
a value per argument, and it makes the call itself against the closed list the
machine offers. Nothing accepts a call somebody else validated, so law 2 is
carried by there being nowhere else to come in.

**The list an agent may ask from is the list the machine can carry out.** A
machine offers the verbs it has an executor for and no others, so *the machine
could not* is always about the machine and never about a capability that was
advertised and cannot happen. Adding an executor and adding to the offered list
are one edit.

**Nothing is handed back that has not been written down.** A turn cannot be made
without somewhere to keep its record, and every door writes its entry before it
answers. A turn that could not write one **stops**: nothing further happens
under it, which is the honest answer to a machine that has stopped keeping
evidence. What that cannot close is the window a change leaves open — a file has
moved before there is anything to write about it — so a client meeting it has a
machine to stop rather than a call to retry.

**A question put to a person is not a thing that happened.** What the record
keeps is the answer: it ran, the person declined it, or the grants refused it.
A change nobody answered goes away with the turn and leaves no entry, because an
entry about somebody not answering would be a record of the person rather than
of the agent.

## The privileged broker

System verbs do not execute in `alo-agentd`. They cross into a separate broker
that holds the few operations needing privilege, with:

- its own fixed verb list, enumerated like this one;
- **no free-form parameters** on any of them;
- no path by which the agent can reach anything the list does not name.

The broker is small enough to be audited in an afternoon, and that is a
constraint on its design rather than a hope about its future.

`crates/alo-broker` is the list and the door, added 2026-09-16.
`crates/alo-brokerd` is the process that runs it, and carries out the four
network verbs (added 2026-09-16) and the two storage verbs (added 2026-09-17).
The recovered printer carrier adds its three verbs beside them through
`Carriers::with_printers(Printers::against(service))`. The published
`Carriers::of(network, proxy, storage)` remains compatible and refuses printer
verbs until that service is supplied; the process supplies `PrintingService`
at its own socket through PrintingService::for_the_broker. Its fixed root
peer-credential header is verified against the actual socket UID by CUPS;
it carries no password and gives a non-root caller no authority. The two update
verbs are answered `not-carried` until ADR 0053, proposed, is accepted and built.

**The list.** Eleven verbs, each with exactly one argument:

| Verb | Argument |
|---|---|
| `printers.add`, `printers.remove`, `printers.set-default` | an identity |
| `network.join`, `network.forget`, `network.set-proxy` | an identity |
| `network.radio` | `on` or `off` |
| `updates.apply`, `updates.roll-back` | an identity |
| `storage.mount`, `storage.eject` | an identity |

An **identity** is the SHA-256 of the identity the rented service reported for
the thing — the printer the print service found, the network the network manager
reported, the drive or filesystem the disk service reported, the build the base
staged — written as
sixty-four lowercase hexadecimal characters. The broker never interprets one; the
verb compares it with what the service reports at that moment and acts on the
match or on nothing. There is no argument of any other shape: no text, no path,
no command, no device name, no password, and no verb formats, repartitions or
erases anything.

**The door.** One Unix socket, one line in and one line out, one caller at a
time. The caller must be the user `alo-agentd` runs as, read from the kernel
(`SO_PEERCRED`) before its line is read. A request is five words separated by
single spaces:

```text
<verb> <argument> <approval> <issued> <proof>
```

`approval` is the turn's number for the approval that was spent, `issued` the
moment in whole seconds since the epoch, both decimal with no leading zero, and
`proof` an HMAC-SHA-256, in sixty-four lowercase hexadecimal characters, over the
exact verb, argument, approval and moment under the broker's approving key. A
token is good once, and for sixty seconds.

The answer is `carried`, `not-kept`, or `refused` and one of
`not-the-agent-service`, `not-a-request`, `not-one-of-its-verbs`,
`not-approved`, `approval-spent`, `approval-lapsed` or `not-carried`. Every
answer is written to the record, as a `brokered` entry, before it is given
(`record-file.md`); `not-kept` means it could not be, and a broker that has said
it once carries nothing out again.

**An approval made by hand.** A person's own change in Settings has no turn and
no proposal number behind it; its token is issued under the approval number
`18446744073709551615` (`alo_broker::BY_HAND`, the largest a `u64` holds), which
no turn's approval reaches. In the broker's record, that number means *a person
made this change themselves*.

**Where the door and the key are.** On a machine the door is
`/run/alo-broker/door.sock`, mode `0660`, and the approving key is
`/run/alo-broker/approving.key`: thirty-two bytes, mode `0440`, owned by root, in
the broker's group — which is the person's own group, never the agent's. The
directory is `0750` in that group. The broker makes a fresh key each time it
starts, so a token issued under the last one is refused; whatever issues tokens
reads the key each time it issues one, and believes it only as a plain file of
exactly thirty-two bytes, owned by root, that nobody but root can write and
nobody outside its group can read.

**The process.** `alo-brokerd` runs as root in the person's group, holding no
capability. Before its door opens it reads who `alo-agentd` runs as from the
machine description and refuses root; refuses to run in root's group or the
agent's; opens its own record, `/var/lib/alo-broker/record.jsonl`, in the
record file's format; makes `/run/alo-broker/wanted`, `0770` in its group; and
hands its key over. A refusal at any step opens no door.

**The printer verbs carried out.** `printers.add` asks the printing service for
the printers it can find now and sets up the one whose reported address digests
to the identity; `printers.remove` and `printers.set-default` ask for the
printers set up and act on the one whose queue digests to it. No match, or more
than one, is `not-carried`, and nothing is changed. The carrier uses
`alo-printing`'s `Found`, `Printer`, `printers_set_up`, `set_up`, `remove` and
`make_default`; `as_reported` supplies the bytes hashed into `Identity`.
`CannotChange` reports a missing printer, denied permission or an unanswered
service. No caller supplies an address, queue or driver to the broker.

**The network verbs carried out.** `network.join` asks the network manager for
the networks in range now and joins the one whose name and protection digest to
the identity, answering once it is joined or has failed; `network.forget` asks
for the saved networks and forgets the one whose identifier digests to it;
`network.radio` turns Wi-Fi on or off. No match, or more than one, is
`not-carried`, and nothing is changed. `network.set-proxy` reads the proxy a
person handed over at `/run/alo-broker/wanted/proxy.json` — a plain file owned
by the person, opened without following a link — sets it only if its bytes
digest to the identity and it is a setting `alo-proxy` would itself have made,
and never over a proxy the organisation set. The machine's proxy file is
`machine-proxy-file.md`.

**The storage verbs carried out.** The disk service is udisks2, spoken to over
the system bus. A drive's identity is the SHA-256 of
`alo-drives drive 1`, a zero byte, and the identifier the disk service keeps for
the drive (its `Id`). A filesystem's identity is the SHA-256 of
`alo-drives filesystem 1`, a zero byte, that drive identifier, a zero byte, and
the filesystem's UUID (`alo_drives::Drive::as_reported`,
`alo_drives::Drive::filesystem_as_reported`). A device name is never an identity.
`storage.mount` asks for every filesystem now and mounts the one whose identity
matches. It is mounted *as* the person the door is for, by the login name
`/etc/passwd` gives that user, with no other option, where the disk service puts
that person's drives. `storage.eject` asks for every drive now and ejects the one
whose identity matches. Every mounted filesystem on it is unmounted first, never
by force, and then the drive is switched off, or its medium ejected, when the
disk service says it can be. Each is `not-carried`, and nothing is changed, when:
no filesystem or drive matches, or more than one does; the drive is part of the
machine rather than one a person plugs in (it is not removable and not attached
over USB or an SD slot, or any of its block devices holds the system); the
filesystem is already mounted; the account file does not name exactly one login
for the person; or a filesystem on a drive being ejected will not unmount.
Mounting makes no grant. What an agent may read on a drive is granted in a
picker, like any folder. **A drive's health is not a broker verb.** It is a read
the disk service answers to anybody on the system bus (`alo_drives::Drives::now`).

**The update verbs.** `updates.apply` and `updates.roll-back` are answered
`not-carried`, and nothing is run. The base's program changes the machine only
for a process holding `CAP_SYS_ADMIN`, and the broker holds no capability. ADR
0053 proposes how they are carried out.

## Records

Every execution — read or change, permitted or refused — is recorded with what
ran, under whose authority, from which approval, and against which grant. A
refusal is recorded too: "the agent tried and was stopped" is exactly the
sentence a security review needs, and it is worthless if only successes are
kept.

The record lives in `alo-record`, and an entry is one of six things: a verb ran,
a properly formed call was stopped, something never became a call at all, a
question was answered on this machine (ADR 0008), something left this machine
(law 1), or the egress policy held something back. Three of the six are ways of
not happening, and a stopped call says *where* it was stopped — nobody was
asked, the person said no, or the grants said no at the last moment — because
those are three different facts about a machine.

**A departure is one entry, and it is the only kind that counts as egress.** A
question answered somewhere else *is* the egress it caused, so where that answer
came from is read off the departure rather than off a second entry beside it;
otherwise "what left this machine today" would count one departure twice. An
egress the policy refused is a refusal and never a departure, because nothing
left. An adapter cannot write either kind of entry from a destination it names
itself: what left is recorded from the departure the indicator showed, and what
was held back from the refusal the policy made.

Two things an entry never carries, and no adapter should expect to add them:

- **the question a person asked.** Where an answer came from is recorded; what
  was asked is not, and there is no field for it;
- **the arguments of a call that never validated.** They are whatever arrived,
  and an entry carrying them would look like every other entry while saying
  something nobody did. The verb name and the refusal are kept, as one readable
  line each.

"Explain what it did" is a query put to the record in its own terms — by agent,
by span, by grant, by approval, refusals only, egress only — and never a search
for text. An adapter that wants a new question answered adds it there rather
than formatting a line for somebody to match against.

## Adding a verb

1. It is in `docs/features.md` with a tier, in the current release.
2. Its `effect` is honest. A verb that changes anything is `change`, including
   one that only changes something "small".
3. Its arguments are typed and validated at the boundary, and none of them
   reaches an interpreter.
4. Its sentence fills from those arguments, and **names every one of them**.
   An argument the sentence leaves out is an argument the person did not agree
   to, so a verb whose sentence omits one cannot be declared at all.
   Its purpose, each argument's purpose and its sentence are declared strings,
   and every one of them is in the adapter's own vocabulary — a word declared
   nowhere reaches a person as a key.
5. It names the grant it requires, and names it over an argument a grant can
   cover — a path or an application. A verb that requires no grant needs a
   written reason in its ADR, and the reason is carried in the declaration
   rather than only in prose.
6. It has a test for the refusal path, not only the happy one.
7. **It names how a person does the same thing without the agent**, in
   `docs/by-hand.md`, quoting the promise in `docs/features.md` that gives them
   the surface — or it says what is owed and which release owns the answer.
   ADR 0009: the agent is unavailable for six reasons and only one of them is a
   choice, so a verb that is the only way to do something is a capability that
   disappears when somebody's card is declined.

Rules 1 to 5 are enforced where a verb is declared, in `alo-capability`: a
declaration that breaks one of them is refused, and the registry has no way to
hold a verb that was not checked. The one thing no check can reach is a verb's
*implementation* passing an argument to an interpreter, which stays rule 1 and
stays on whoever writes one.

**Rule 7 is enforced too**, by `crates/alo-by-hand`: it is handed the same
`alo_capability::Verbs` a daemon enforces, and a verb with no entry in
`docs/by-hand.md` fails the gate in the change that adds it — which is the one
moment anybody has the knowledge to answer it. A quotation the definition does
not make, and a debt owed at a release nobody ships, are refused with it.

### Where verbs are declared, which that check reads

**A crate declares verbs in `src/verbs.rs`, through a `pub fn declare_into` that
puts them on somebody else's `Verbs`.** `alo-files`, `alo-applications`,
`alo-finding`, `alo-measuring`, `alo-printing`, `alo-changing-network`,
`alo-changing-printers`, `alo-software` and `alo-adapters` all do exactly that, and it is a rule rather
than a habit because `alo-by-hand` walks
this workspace's own member list for it: **a crate that declares verbs and was
not handed to that check would make every verb in it invisible to rule 7**, and
the check would go on passing in the same colour. An adapter outside this
workspace keeps the same shape for the same reason — whoever assembles its list
is who has to answer rule 7 for it.

**And the list that is handed in is one list, in one file.** A crate that
declares verbs is named in `crates/alo-declared/src/shipped.rs` — one registry entry
pairing its name with its `declare_into` function. That entry derives
`WHO_DECLARES_THEM`, the combined declarations and the per-crate declarations.
Add the dependency to that crate's `Cargo.toml` too, because a test cannot call a
crate it does not depend on. Nothing else needs
touching: every check over the verbs alo OS ships is handed that one list, and
`crates/alo-declared` holds it to this workspace's own member list, naming the
crate and this file when they disagree. It was two hand-written copies until
2026-09-17, in `alo-by-hand`'s test and `alo-software`'s, and a crate arriving
broke lanes that had not touched either.
