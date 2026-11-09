# ADR 0149 — Full access is everything the person can do

**Status:** **accepted, 2026-11-08, by the owner.** It amends
[ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
(the three levels, and which one a new machine starts on) and the `CLAUDE.md`
standing rule *The person grants; the agent never assumes*.
**Date:** 2026-11-08

## The owner's words

> Alo should be able to do all the human can do without limits, when the users
> gives alo full access alo should be able to do anything on the pc without any
> limitations

Asked three questions the same day, the owner answered:

- *Should full access reach every file and folder the person can reach, with no
  separate folder grants?* — **"Yes, everything I can reach."**
- *Should it include administrator actions?* — **"Ask my password each time."**
- *What should a new machine start with?* — **"Full access from the start"**;
  then, told that law 5 says a risky thing is *switched on by the person*, and
  offered a first-start screen with full access already selected:
  **"First-start asks, full access pre-selected."**

## What it changes for the person

**At full access, alo can do anything the person's own account can do**, without
asking: read, create, change and delete any file the account can, run any
command, start, drive and close any application. Before this decision, ADR
0064's *full trust* ran code without asking but alo could still reach only the
folders granted one at a time, and on a fresh machine it could not even create a
folder, because no verb did and it did not fall back to running a command.

**A new machine asks once, during first start**: one screen, *How much can alo
do?*, with **Full access** already selected and its cost in one plain sentence —
ADR 0064's: *text in a web page, an email or a downloaded project can steer
alo, and at this level whatever it steers alo to do, happens.* The person
presses Continue, or chooses *Ask me each time* or *The sealed box*. Law 5 is
kept as written: the person switched it on.

## The decision

1. **Full access** (ADR 0064's level 3, renamed for the person) reaches
   **everything the person's account can reach**. There are no folder grants
   at this level; the account's own permissions are the boundary.
2. **Administrator actions ask for the person's password each time** — anything
   the account itself could only do with its password. alo never stores the
   password and never asks for it in a way the person cannot see is alo asking.
3. **Where no verb exists, alo does it by running a command** at full access
   (without asking) and at *ask me each time* (after the OK). *"There is no
   verb for that"* is never an answer a person sees at either level.
4. **First start asks**, with Full access pre-selected, as above. Choosing
   later, in Settings → Running code, is unchanged.
5. **The other two levels stay** exactly as ADR 0064 describes them.

## What stays on at every level, including full access

Unchanged from ADR 0064, because none of it restricts anything: **the record**
of every command and every change; **the snapshot before each run**, so Put back
works; **the egress indicator** for everything that leaves the machine. And
everything that guards the person from others: no telemetry, never a silent
fallback to a hosted model, watching only if switched on (ADR 0146). **An
organisation** may still limit the levels on machines it owns (ADR 0016).

## The constitution

The standing rule *The person grants; the agent never assumes* said *There is
no grant to `/`.* It now says that the whole of what the account can reach is
granted only by the person choosing full access, knowing what it costs.

## Not decided here

- The first-start screen's design. It needs a frame in the design file before it
  is built; until then the build follows the existing first-start sheets and
  records the departure.
- How the password is asked for (a system dialog the person recognises, or alo's
  own). Recommended: the system's own, so it cannot be imitated.

## How this is checked

Nothing checks it yet. The change that builds it adds, as tests: full access
reaches a file outside any grant; ask-each-time does not run without the OK;
the sealed box still cannot reach outside its box; an administrator action
without the password is refused; every run is in the record with a snapshot.

*Added 2026-11-08, with decision 3 built (a command where no verb fits).* These
now run on every build, as tests: at full access a command reaches a folder no
grant names and is written down with no grants and no box
(`crates/alo-agentd/src/running_a_command_tests.rs`); at *ask me each time*
nothing runs before the OK, one OK runs it exactly once, a repeated or stale
number runs nothing and a no is written down (the same file, and
`crates/alo-turn/src/a_command_where_no_verb_exists_tests.rs`); at the sealed
box, with nothing chosen and with a file that will not read, nothing runs and
the refusal is written down; and the snapshot is asked for before each run.
Still unchecked, because still unbuilt: full access reaching files through the
verbs without grants, the first-start screen, and the password for
administrator actions. **And not yet true:** what a command sends does not
reach the egress indicator (`docs/contracts/daemon-protocol.md`, under `run`).
