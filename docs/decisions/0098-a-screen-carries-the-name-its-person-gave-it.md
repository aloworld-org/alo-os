# ADR 0098 — A screen carries the name its person gave it

**Status:** **ACCEPTED, 2026-10-08**, by the owner, stated directly: *"users can
name the monitors by any name like 'cat'"*, and in the same conversation *"I want
all the discussed features of the monitors to be in the next release after this
one"*.
**Date:** 2026-10-08
**Proposed by:** the owner (the decision), the Mac lane (the consequences below)
**Context:** Law 5 (*the person chooses*), `docs/features.md`'s **Screens and
desks**, [ADR 0068](0068-a-published-sentence-changes-by-getting-a-new-key.md)
(what a translated string owes), `crates/alo-displays/src/identity.rs`,
`crates/alo-shell/src/the_session_holds_its_screens.rs`,
`docs/autonomy/screens-and-desks-plan.md`

## The question in one line

**What may a person call a screen, and what does the machine do with that name?**

## The decision

**Any name they like. `cat` is a valid name for a monitor.** Free text, typed on
the screen it belongs to, and not chosen from a list.

Law 5 settles the headline and four consequences settle the rest.

**1. A name is a label, never a key.** The machine already tells screens apart by
`Identity` — what the screen reports about itself, or which socket it is plugged
into. **That does not change.** The person's name sits on top of it and is what
is shown. Nothing in the system ever looks a screen up *by* the person's word.

**2. Two screens may carry the same name.** Somebody with two identical monitors
may reasonably call both of them `cat`, and refusing it would take a choice away
for the machine's convenience — which Law 5 names as a bug whatever reason is
given for it. Nothing becomes ambiguous internally, because of rule 1.

**3. A person's own name is never translated.** It joins the category a screen's
make and model are already in: `displays.new-here`'s translator note says
`{display}` *"is either what the screen says about itself, such as "Dell
U2720Q", which is never translated"*. A name a person typed is theirs in the
language they typed it.

**4. The default name is descriptive, never a number.** *the laptop*, *the big
one on the right*. A number is the one thing nobody can match to the screen in
front of them, which is the step people actually fail at — and the reason
`docs/features.md` carries *screens are named, not numbered, by default* as its
own promise rather than as a detail of this one.

## What this costs, said plainly

**Moving a cable to another port loses the arrangement, and now loses the name
with it.** This compositor identifies a screen by its socket, because
`OutputMetadata` carries no serial number and a make and model without one
identify a *kind* of monitor rather than a monitor —
`Server::the_displays_as_reported` documents at length why offering a panel
instead would collapse two identical screens into one.

So the loss is not new. **What is new is that it now has a face:** a person will
see `cat` disappear and ask where it went, where before they only noticed their
windows had moved. That is an argument for fixing it, not against naming.

**And the fix is a sentence, not a serial number.** `alo-displays` already has
the shape of it — `Note::RememberedByItsSocket` tells a person outright that a
screen with nothing to say about itself is remembered by its port, and
`Note::ToldApartByTheirSockets` says *"swapping their cables swaps the two"*. The
honest answer to a cable move is to recognise the near-miss and ask: *this looks
like your office desk with one screen in a different port — use that
arrangement?* That is work this ADR names and
`docs/autonomy/screens-and-desks-plan.md` orders; it is not claimed as done.

**Inventing a serial would be worse than having none.** It would make two
monitors *stably* one screen rather than visibly one, which is the failure that
survives a restart.

## What this does not change

- **`Identity`, `Socket` and `Panel`** — untouched. Rule 1 is the whole of the
  relationship between a name and an identity.
- **The arrangement, and its per-set memory.** A desk is a set of screens; what
  they are called has no bearing on which set it is.
- **ADR 0068.** A person's name is data, not a string this repository declares,
  so it has no key and needs no translator's note.
- **The shell shows and never measures.** A name is a person's setting, so it is
  read where their settings are read and handed to the shell, exactly as the
  canvas layout and the shortcuts already are.

## What is not enforced, said plainly

| rule | what enforces it |
|---|---|
| 1 — a name is a label, never a key | **owed:** a test that no lookup takes a person's name. Without it, the first convenient `HashMap<String, _>` keyed on the name reintroduces it silently |
| 2 — two screens may share a name | **owed:** a test that two screens named the same are still two screens |
| 3 — never translated | **nothing automatic.** The existing vocabulary tests say nothing about data |
| 4 — the default is descriptive | **owed:** a test that no default name is a bare number |

**All four rest on tests that do not exist yet**, named in the plan with an
owner. Rule 1 is the one worth building first, because it is the one a later
convenience quietly undoes.
