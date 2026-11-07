# ADR 0146 — A person may switch watching on

**Status:** **accepted, 2026-11-06**, by the owner's direction the same day:
*"Change this law first before we go forward"*, given after this exact
sentence was put to them: *never watched unless the person switches it on,
knowing what it costs.* It amends [ADR 0001](0001-the-capability-model.md) §4
and the standing rule *Context is offered, never watched* in `CLAUDE.md`.
**Date:** 2026-11-06
**Context:** the owner's alo Cursor concept (November 2026), and their answer
of 2026-11-06: *"I want to also make the cursor ai native but the user can
choose in the settings."*

## What it changes for the person

**Nothing, until they switch something on.** A machine nobody has touched
watches nothing, exactly as before: what is on the screen reaches alo only
when the person calls it, and only for that turn.

What is new is that **the person may choose to have something watch**, and the
first thing that will ask is the AI cursor: a pointer that understands what it
is over as the person moves, and suggests. Before 2026-11-06 the rule forbade
that even for somebody who wanted it, which is the one kind of protection law 5
says this product does not have: a wall where a choice should be.

## The decision

The rule becomes: **context is offered, never watched, unless the person
switches watching on, knowing what it costs.**

A watcher is any part of alo OS that reads what is on the screen, under the
pointer, in the clipboard or in a document while nobody has called alo. Every
watcher, now and later:

1. **Is off until the person switches it on**, by name, in Settings, beside a
   sentence saying in plain words what it reads and where that goes.
2. **Says so while it watches**, on screen, for as long as it is on. A watcher
   nobody can see is the background reader this rule still forbids.
3. **Stops the moment it is switched off**, and keeps nothing of what it saw.
4. **Is switched on only by the person.** An agent can never switch one on,
   propose switching one on as part of other work, or keep one on after the
   person said no.
5. **Feeds nobody else.** What a watcher sees is not context for alo's turns:
   alo still receives only what is offered at the moment it is called.
6. **Sends nothing silently.** Anything a watcher sends off the machine is
   agent-caused egress under law 1: it fires the indicator and leaves a record.
7. **Can be forbidden by an organisation** on a machine it manages
   ([ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)), and
   the person is told it is forbidden and by whom, never shown a switch that
   does nothing.

## What stays exactly as it was

- With no invocation and every watcher off, `alo-agentd` makes no context
  calls at all. That guarantee is a public promise and a CI test, and this
  decision does not touch it.
- A grant to an agent still never reaches the screen, the camera or the
  clipboard by watching. The refusals that say *a background reader by
  another name* in `alo-capability`, `alo-capturing` and `alo-clipboard` stay
  true: what this decision adds is a switch the person flips, not a grant an
  agent holds.

## What this costs, written here because it is the part to argue with

A watcher that is on reads everything it is pointed at, all day, including
what the person did not mean to show it: a password field, a private message.
The switch, the sentence beside it and the sign on the screen are the only
things between the person and forgetting that. That raises what Settings and
the shell owe: the sentence must be true, and the sign must never be hidden.

## Not decided here

- **Whether the AI cursor may use a cloud provider**, or only the model on this
  machine. That is the owner's to answer, and rule 6 applies either way.
- **What the AI cursor is**, in `docs/features.md`. It needs its own line there
  before it is built, as every promise does.

## How this is checked

**Nothing checks it yet, because no watcher exists.** The change that builds
the first watcher also adds, as CI tests: a watcher that is off reads nothing;
a watcher switched off stops at once; an agent cannot switch one on; and
nothing a watcher sends escapes the indicator.
