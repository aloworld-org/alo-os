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
| `purpose` | One sentence, in the words a person would use. |
| `effect` | `read` or `change`. Decides whether it runs in the turn or waits for approval. |
| `args` | Typed, each with a purpose. Validated at the boundary before anything runs. |
| `requires` | Which grant must be held for this call to be possible at all. |
| `sentence` | How the approval sentence is generated **from the validated arguments**. |

Two rules that are easy to state and easy to violate:

1. **No argument is ever passed to an interpreter.** Not a path fragment, not a
   filter expression, not a "script" field. A verb that needs to run something
   builds it internally from typed arguments.
2. **The approval sentence is generated, not written by the model.** It is
   derived from the validated arguments. A `change` verb whose sentence cannot
   be generated from its arguments is refused, because an approval a person
   cannot understand is not an approval.

## What an argument can be

A closed list, like the verbs themselves. An argument is one of:

| Kind | What it takes |
|---|---|
| **path** | A full path, with no `..` in it. What a grant is usually over. |
| **application** | An installed application, by its identifier. |
| **name** | One name — a file's, a folder's, or what is being searched for — of at most the length the verb declares. One name, never a path. |
| **count** | A whole number inside a range the verb declares, both ends included. |
| **choice** | One of a list of options the verb wrote down, matched exactly. |

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

## The verb classes

| Class | What it covers | Where it runs |
|---|---|---|
| **Files** | List, read, find, rename, move, archive — within granted paths | `alo-agentd`, as the person |
| **Applications** | Open, focus, arrange, close | `alo-agentd`, as the person |
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

The record lives in `alo-record`, and an entry is one of four things: a verb
ran, a properly formed call was stopped, something never became a call at all,
or a question was answered somewhere (ADR 0008). Three of the four are ways of
not happening, and a stopped call says *where* it was stopped — nobody was
asked, the person said no, or the grants said no at the last moment — because
those are three different facts about a machine.

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
4. Its sentence generates from those arguments, and **names every one of them**.
   An argument the sentence leaves out is an argument the person did not agree
   to, so a verb whose sentence omits one cannot be declared at all.
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
