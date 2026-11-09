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

*Added 2026-11-08, with decision 1 built (the verbs reach what the account
can).* These now run on every build, as tests: at full access a read and a
change reach a file in a folder no grant names, the change after one OK and
exactly once, and both are written down as allowed by full access rather than
a grant (`crates/alo-agentd/src/the_grants_as_the_level_is_now_tests.rs`); an
application nobody granted is opened after one OK
(`crates/alo-turn/src/acting_on_an_application.rs`); switching full access off
refuses a change proposed under it at its OK, and refuses the next read, and
those refusals are written down as not at full access (both files); at *ask me
each time*, the sealed box, with nothing chosen and with a level file that will
not read, a read and a change outside the grants are refused and written down,
and a granted change waits for its OK (the first file); full access is one
agent's, never an application's, never kept on the list or in the grants file,
does not reach the camera, the microphone, the screen or a terminal the person
has not allowed, does not widen a machine whose person declined the agent, and
adds to grants rather than replacing them
(`crates/alo-capability/src/full_access_tests.rs`). **Not checked by any test
here:** the kernel boundary (ADR 0013) with full access on a real kernel. It is
built from the paths a call names, not from the grants
(`alo_files::Reaching::of`), so nothing in it changed;
`crates/alo-agentd/tests/a_turn_is_bounded_by_the_kernel.rs` still runs only
under a grant. And the account's own permissions as the boundary are the
operating system's: the service runs as the person, and no test here makes a
file the account cannot read, because the build machine runs its tests as root.
*(This paragraph said, when it was written, that a change made through a verb
at full access still waited for the person's OK. That is now built; see
below.)*

*Added 2026-11-08, with the other half of decision 1 built (a change at full
access is done without asking).* These now run on every build, as tests: at
full access a change through a verb is carried out at once, exactly once, with
no approval, after the snapshot was asked for and while the file was still
where it was; nothing is left waiting for the person; it is written down as
ran at full access with no approval; and the person is told *Done without
asking you first, because you chose full access in Settings → Running code*,
with *it can't be put back* when no snapshot was kept
(`crates/alo-turn/src/a_change_at_full_access_tests.rs`,
`crates/alo-agentd/src/a_change_asked_for_tests.rs`). At *ask me each time*,
the sealed box, with nothing chosen and with a level file that will not read, a
granted change still waits, takes no snapshot and runs nothing before the OK,
one OK runs it once and the same number again runs nothing (the second file).
Switching full access off between two changes: the first is done, the second
waits, and one outside every grant is refused (the second file). A change the
disk refuses at full access is answered as a refusal, never as done, and
written down at full access with what the machine said kept beside the attempt
(both files). Only a list made at full access for this agent authorises a
change with no approval (`crates/alo-capability/src/without_the_ok_tests.rs`).
The agent's loop treats a change done at full access as a step and goes on, and
hands back what the person is told however the turn ends
(`crates/alo-agent/tests/a_change_done_at_full_access.rs`); and the terminal's
own switch has nothing to answer at full access
(`crates/alo-ask/src/acting_without_the_ok_tests.rs`). **Not checked by any
test here**, as above: the account's own permissions refusing a change, because
the build machine runs its tests as root — a disk refusal (a name already
taken) stands in for it, and the two take the same road. **Still unbuilt:** the
first-start screen and the password for administrator actions; and what a
command sends still does not reach the egress indicator.
