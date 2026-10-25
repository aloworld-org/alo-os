# ADR 0001 — The capability model: what an agent may reach, and under whose authority

**Status:** accepted — the foundation the rest of this repository rests on.
**§1 replaced by [ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md), 2026-09-22:** running code becomes the person's
grant at three levels. The broker's verb list stays typed and closed, and the
rest of this decision stands.
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
