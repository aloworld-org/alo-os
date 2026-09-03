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
`docs/quirks.md`, and closing them belongs to the code that opens the file.

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
destination already holds anything — a file, a folder, or a link — is refused
and says the name is taken. This is a rule about the sentence, not about
filesystems: what was approved is what happens, and nothing else is.

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

## The verb classes

| Class | What it covers | Where it runs |
|---|---|---|
| **Files** | List, read, find, rename, move, archive — within granted paths | `alo-agentd`, as the person |
| **Applications** | Open, focus, arrange, close — over granted applications | `alo-agentd`, as the person |
| **Context** | The focused window, the selection, the open document | Offered at invocation only |
| **Adapters** | An installed application's own verbs | See `app-adapters.md` |
| **System** | Printers, network, updates, storage | The **privileged broker**, never the agent directly |

## The privileged broker

System verbs do not execute in `alo-agentd`. They cross into a separate broker
that holds the few operations needing privilege, with:

- its own fixed verb list, enumerated like this one;
- **no free-form parameters** on any of them;
- no path by which the agent can reach anything the list does not name.

The broker is small enough to be audited in an afternoon, and that is a
constraint on its design rather than a hope about its future.

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

Rules 1 to 5 are enforced where a verb is declared, in `alo-capability`: a
declaration that breaks one of them is refused, and the registry has no way to
hold a verb that was not checked. The one thing no check can reach is a verb's
*implementation* passing an argument to an interpreter, which stays rule 1 and
stays on whoever writes one.
