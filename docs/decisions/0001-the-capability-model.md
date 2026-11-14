# ADR 0001 — The capability model: what an agent may reach, and under whose authority

**Status:** accepted — the foundation the rest of this repository rests on.
**§1 replaced by [ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md), 2026-09-22:** running code becomes the person's
grant at three levels. The broker's verb list stays typed and closed, and the
rest of this decision stands.
**§4 amended by [ADR 0146](0146-a-person-may-switch-watching-on.md), 2026-11-06:**
nothing watches unless the person switches it on, knowing what it costs. With
every watcher off, §4 reads exactly as written.
**Amended 2026-11-13 (below):** an argument can carry **text** — many lines, no
maximum length, shown in full, never handed to an interpreter.
**§4 amended 2026-11-14 (below):** the installed applications' names and
identifiers are offered with each request the person sends — at invocation, for
that turn, never as a verb and never in the background.
**Date:** 2026-09-02
**Context:** `alo-agentd`, the application adapters, `docs/contracts/`;
carries forward the propose-then-approve and intent-layer decisions made in
`alo-workplace` (its ADRs 0023, 0047, 0057, 0058)

## The decision in one line

An agent reaches the machine **only** through enumerated verbs with typed
arguments, over paths and applications a person has granted, executing reads
inside the turn and changes only after one approval of the sentence describing
them — and **no verb runs an arbitrary command**, which is the single rule that
makes every other rule here true rather than decorative.

## Why this is ADR 0001

`alo-agentd` is the largest attack surface this product will ever ship, on a
system sold on sovereignty, driven by a language model that reads text an
attacker may have written. The design assumption is therefore not that prompt
injection is unlikely, but that **the model is already saying whatever an
attacker wants it to say.** Everything below exists to bound what that can
cause.

A capability model retrofitted onto a running daemon is how this product would
fail publicly. So it is written before the service exists, and it is numbered
first.

## The model

### 1. No verb runs an arbitrary command

There is no `exec`, no `run_command`, no shell, no script evaluation, no
"advanced mode", and no verb whose argument is handed to an interpreter.

This is the load-bearing rule. Every other control in this document assumes the
model can only choose *from* a list, never *write* what runs. Add one command
verb and the grants, the approvals and the records all become theatre — an
agent that can write a script can do anything the person can do, in one
approval, with a record that says nothing useful.

It has a cost and the cost is accepted: driving an application through its
scripting API is easy, and driving it through typed verbs is work. See §6.

### 2. No ambient authority

`alo-agentd` runs as the signed-in person. Never as root, never with
capabilities the person does not have.

The genuinely privileged operations — printer configuration, network settings,
system updates, storage — sit behind a **separate broker** with its own fixed
verb list and no free-form parameters. The agent reaches the broker only
through those verbs. The broker is small enough to audit in an afternoon, and
that is a design constraint on it, not an aspiration.

### 3. The filesystem is granted, not browsed

An agent sees no path a person has not granted it. A grant comes from a
deliberate act: a folder chosen in a picker, or the document offered at
invocation.

Grants are **enumerated** (a list, not a rule), **visible** where the person can
find them without hunting, **revocable** in one action, and **expiring** by
default. There is no grant to `/`, and there is no grant that outlives the
reason it was made.

### 4. Context is offered, never watched

The focused window, the current selection and the open document reach an agent
**only at the moment of invocation** — the hotkey — and only for that turn.

There is no background reader, no screen watcher, no clipboard monitor. This is
testable rather than promised: with no invocation, `alo-agentd` makes no context
calls at all, and that is a test we run in CI rather than a sentence in a
privacy policy.

### 5. Reads answer; changes wait for one approval

Unchanged from `alo-workplace`'s ADR 0047 and ADR 0057, because a person should
not have to learn two rules.

A **read** executes inside the turn, under the run's budget. Asking what is in a
folder returns the answer, not a request to be allowed to look.

A **change** comes back as a proposal carrying a sentence describing exactly
what it will do. What the person approves is that sentence. One approval, one
action — **an approval is never a session**, and no approval grants anything
beyond the action named in it.

### 6. Applications are reached through adapters, and adapters expose verbs

An installed application becomes an agent the same way a product does: a set of
typed verbs. `@blender`, `@resolve`, `@gimp` sit alongside the workspace's own
agents, under the same approval rules.

There are four mechanisms, and they are not equal:

| Mechanism | When | Quality |
|---|---|---|
| The application's own automation API | It has one — Blender, Resolve, LibreOffice, GIMP, Inkscape | Best: semantic, reliable, verifiable |
| The accessibility tree (AT-SPI) | No API exists | Good: a real widget tree, activated properly |
| D-Bus interfaces | The application exposes one | Good, where it exists |
| Screenshots and synthetic input | Last resort | Poor: fragile, and **unauditable** |

The rule that makes this safe is the one that is easiest to get wrong: **the
adapter exposes typed verbs and generates any script internally, from validated
arguments.** The model chooses `resize_image(width, height)`; the model never
authors code that executes. Driving Blender by handing it a model-written Python
script would satisfy §1's letter and destroy its purpose.

Screenshot-and-click is permitted only where nothing better exists, is marked as
such in the record, and can be disabled by policy — precisely because afterwards
nobody can say what it actually did.

### 7. Everything executed is a record with an origin

Every execution is recorded with what ran, under whose authority, from which
approval, and against which grant. "Explain what it did" is a query, not a log
to grep — and a compromised turn leaves evidence shaped exactly like every other
turn, which is what makes the evidence worth having.

### 8. Nothing leaves silently

Egress policy lives in a Rust service, not a settings checkbox. The indicator
fires at the moment of egress, not in a daily summary.

With a local model, expected inference egress for a working day is **zero**,
measured at the network boundary. We publish that measurement. Where inference
goes to a server the customer chose, the indicator says so as it happens.

## What a compromised model can and cannot cause

Stated plainly, because this is the question a security reviewer will ask and it
deserves an answer that does not hide behind the controls above.

**It can:** read anything already granted, and propose anything the verb list
allows. A person who approves without reading the sentence can be led into a
single harmful action — the same exposure a person has with any tool.

**It cannot:** reach a path, application or device that was never granted; run
code of its own composition; escalate to root; act more than once per approval;
persist authority beyond a grant's expiry; read the screen or clipboard without
invocation; or send anything to a network without the egress indicator firing.

**The residual risk is the approval sentence itself.** If the sentence is vague,
the approval is uninformed, and the model chooses the words. So the sentence is
generated from the *validated arguments*, not from model prose, and a change
whose sentence cannot be generated from its arguments is refused rather than
approved. That is a constraint on every adapter and it belongs in the contract,
not in review comments.

## Alternatives rejected

**A permission prompt per action, with a "remember this" checkbox.** Rejected:
"remember" converts one approval into an unbounded session, which §5 exists to
prevent. Grants are the durable thing, and they are made deliberately rather
than accumulated by clicking through dialogs.

**A sandboxed scripting verb — let the model write code, but confine it.**
Rejected: it moves the security boundary from a verb list we designed to a
sandbox we would have to make perfect, and every sandbox escape becomes a total
compromise. The verb list is auditable by reading it. A sandbox is not.

**Screenshot-and-click as the universal mechanism, since it works everywhere.**
Rejected as the default and kept as a last resort. It cannot be verified after
the fact, which breaks §7, and on a machine sold on auditability that is not a
trade we can make quietly.

---

## Amended 2026-10-25 — a grant may last until the person removes it

§3 says grants are **expiring by default**, and one sentence later that *there
is no grant that outlives the reason it was made*. The two pull apart, and the
code read the second: `Grant` stores `expires: SystemTime`, with no way to say
*never*, so `alo-picking` could only offer lengths and a person had to come
back and grant the same folder again.

**The owner settled it on 2026-10-25**, in these words:

> I think it should be like an agent here, Claude co work or Claude code — I
> can give an agent full access or remove it but there is no time limit. We can
> put the time limits but also we should add where the user can put so as he
> does not need to come back every one hour or month to change.

So **until I remove it** is one of the lengths a person may choose, beside an
hour, a day and a month. *Expiring by default* stands and is what it says: a
default, which law 5 makes theirs to change. *No grant outlives the reason it
was made* is narrowed to what it was protecting — a grant nobody can see and
nobody remembers — and that is now carried by visibility instead of by a clock:
it is in the one list, it says **until you remove it** where another says a
time, and one press revokes it.

Nothing else in §3 moves. A grant is still enumerated, still visible where a
person can find it, still revocable in one action, and there is still no grant
to `/`.

### What this costs, written here because it is the part to argue with

A grant that never expires is the one a person forgets. The clock used to be
the backstop; now the list is the only one. That raises what the grants section
of Settings owes a person — it must be findable, it must read plainly, and an
endless grant must never be indistinguishable from a timed one at a glance.

**And it is not a default.** The shortest length is still the row the ring lands
on, so the least a person can give stays the cheapest to choose, and *until I
remove it* is a deliberate reach rather than the path of least resistance.

---

## Amended 2026-11-06 — the fourth road is the screen road, under ADR 0147's rules

*Accepted by the owner on 2026-11-06 with
[ADR 0147](0147-the-screen-road-and-who-reads-the-screen.md) ("yes").*

§6's fourth row is read, from that day on, as:

| Mechanism | When | Quality |
|---|---|---|
| The screen: one window's picture read, and keys and clicks delivered to that window | **Only for a control no better road reaches** | Least reliable: it reads pictures, and afterwards nobody can say for certain what was pressed — so every act is marked in the record |

Everything §6 already said of it stands — *only where nothing better exists,
marked as such in the record, and can be disabled by policy* — and ADR 0147 §1
adds how it is kept honest: one window, read at the moment of the act and never
kept, found by what it says, read again after, password fields never read or
sent. **alo uses it for an application only after the person switches it on for
that application; alo never acts by position; a person's own typed commands use
it at once** (ADR 0147 §2–§3). *Screenshot-and-click as the universal
mechanism* stays rejected, below: it is the last road, never the first.

---

## Amended 2026-11-13 — an argument can carry text, with no limit, shown in full

*Decided by the owner on 2026-11-13, for stage 5 of
`docs/autonomy/demo-two-alo-inside-applications-plan.md`.* The owner's words:

> There should be no input limits

> We should make something user-friendly and no limits

**The reading the owner was given, and which is built:** a new kind of
argument, **text** — many lines, **no maximum length**.

- **Invisible and confusable characters are not refused.** The approval marks
  each one where it is, in plain words — *invisible character U+200B*, *a
  character that changes the direction of the text* — and says how many there
  are, so nothing hidden passes unseen. That is law 5 read the way it reads
  itself: a protection that restricts nothing stays on; a wall goes.
- **The person can see all of it in the approval.** It scrolls, it says how
  long it is (lines and characters), and it is never cut and never summarised
  in place of the text. The sentence itself stays one line and names the text
  as *the text below*; the text is shown whole beneath it.
- **Nothing on this machine hands the text to an interpreter or a shell.** This
  is §1 and law 2, not an input limit: the text is data that a verb delivers —
  typed into a field, saved, posted — and nothing runs it. What alo types into
  a terminal is ADR 0111's choice and the person's alone, as before.
- **The record keeps the text**, whole.

What still refuses a text argument is what is not text at all: nothing (an
empty value), a number where text was declared, and bytes that are not valid
UTF-8, which never reach a verb because the wire refuses them first.

**What this replaces.** [ADR 0143](0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
fact 1 proposed this kind **bounded in length** and **refused if it holds
characters a person cannot see**. Both halves of that proposal are replaced by
this decision: there is no bound, and such characters are marked rather than
refused. Everything else fact 1 proposed — shown in full, never handed to an
interpreter — stands. The other kinds (path, application, name, count, choice)
keep every rule they had, including their refusal of control characters: a
value that appears *inside* the one-line sentence still has to read as one
line, and text does not appear inside it.

### What this costs, written here because it is the part to argue with

§1 rested partly on *there is no shape a composition could arrive in*. Text is
that shape. What keeps §1 true is now the rule that no verb passes an argument
to an interpreter — a rule about verb implementations, held by review, by
`alo_capability::Verb::checked`'s tripwire on argument names, and by a
workspace test that no source file naming the text value also starts a process
(`crates/alo-capability/tests/text_never_reaches_a_process.rs`). That test is
a tripwire too and says so: a verb that hands text to an editor which itself
runs code is the risk ADR 0143 fact 3 names, and the answer there is that alo
never types into a terminal, inside or outside an editor, unless ADR 0111's
choice says so.

A very long text is also a long thing to read. The approval cannot make a
person read it; it can only make sure that everything is there to be read, that
the length is said before they start, and that anything hidden is marked.

## Amended 2026-11-14 — the names of the installed applications are offered with the request

*Decided by the owner on 2026-11-14.* Asked whether a model should be told the
installed applications' names, after a walk where *open vscodium* was refused
on a machine that had `com.vscodium.codium` and alo then installed something
else from Flathub, the owner answered:

> Yes, send app names

**What is built.** When the person sends alo a request, their shell sends with
it each installed application's **name and identifier** —
`VSCodium (com.vscodium.codium)` — and nothing else: no file, no window, no
document, no icon, nothing about use. It is the seventh line on the asking
socket (`docs/contracts/asking-socket.md`), and the model is told it before the
folders and the request (`alo_instructing::TheApplicationsInstalled`).

**Why this is §4 kept, not §4 bent.** `alo-applications`' header says a verb
that listed the running applications would be *a background reader*, and that
stands: **this is not a verb**, and an agent cannot ask for it. It is read on
the person's side **at the moment of invocation** — when they press Enter —
from what their shell already holds to draw the Dock, and it travels with that
one request, for that turn, exactly as the window offered and the person's
folders do. With no invocation nothing is read and nothing is sent, so the
test §4 names still holds as written.

**What it costs, said plainly.** With an online provider the list leaves the
machine inside the question, under the egress indicator, like every other part
of the question. Which applications a person has says something about them
(`alo_applications::Reaching`'s header calls it a fingerprint), and the owner
chose to send it for the turn to work. **It allows nothing**: every
application verb is still decided by `alo-agentd` against the grants and the
installed list it reads itself, and a refusal still says nothing about what is
installed.
