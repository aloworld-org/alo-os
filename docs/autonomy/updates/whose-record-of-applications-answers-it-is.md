# Whose record of applications' answers it is

**Date:** 2026-09-17
**Workstream:** v0.5 — applications, and what they expect
**Task:** [task 11](../v0-5-applications-and-what-they-expect-plan.md), the last
one in that plan
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 3 GB of memory**.
**Egress:** none.
**Status:** done. **The plan is finished**, and what was added to it after it was
written is at the bottom of this report.

## The decision, which came first

[ADR 0052](../../decisions/0052-what-a-persons-applications-asked-for-is-the-persons-record.md).
The record of what a person's applications asked for and were told is **the
person's**: one file per login, in that login's own state
(`$XDG_STATE_HOME/alo/portal-answers.jsonl`, or the specification's default under
`$HOME`), readable by that login and nobody else.

Tasks 8 to 10 had made this file durable, shortened and readable, and left *whose
it is* open on purpose — and everything about it pointed both ways at once. The
backend answers on a **session** bus, as that login; the path was one
machine-wide `/var/lib/alo/portal-answers.jsonl`; and `believed_file` accepted a
file owned by **root or by this login**, which is true of either answer and
decides neither. On a machine with two people, that file would have held both
people's applications' requests in one place, each readable by the other. **Nobody
chose that**; it was the shape the first version happened to have.

### It applies ADR 0004 rather than extending it

ADR 0004 enumerates what a managed organisation gets, and *the record* in that
list is one thing: **agent executions and refusals**, exportable to their SIEM. A
portal answer is not an agent execution — ADR 0040 part 2 is emphatic that an
application is not an agent, and this file's own contract already said it is not
the agent's record. So ADR 0004 never handed this to an organisation, and ADR
0052 takes nothing from one. It says which side of ADR 0004's existing line a
record it never named falls on.

The side is the person's, in ADR 0004's own words: an administrator **cannot
watch a person**. A list of every time somebody's calendar asked for their
microphone, their browser for the camera, their editor for a folder — with the
times, and with what they said each time — is a description of a working day. A
machine that filed it where an administrator reads would be watching a person
with extra steps. [ADR 0016](../../decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
drew the same line once already, for settings: the organisation's bound in
`/etc`, the person's choice under `$XDG_CONFIG_HOME/alo/`, *not in `/etc`, where
an administrator would be reading somebody's preferences on their own machine.*

### The option that was rejected, and what it cost

One machine file written by a system service on the sessions' behalf, each answer
carrying whose session it was, each person reading back only their own. It costs
**a new privileged service** on the portal road — something running as root that
every session asks to write for it — in the exact place ADR 0005's sandbox and
ADR 0001's model are trying to keep narrow, for a file nobody has asked to read.
And *each person reads only their own* would then be **our filter, in our code,
over a file that physically holds everybody's**: one bug in that filter is one
person reading another's day. The chosen option's equivalent bug does not exist,
because the other file is not open. On a managed machine, the single file is also
one artefact an administrator can take in one act.

## What changed in the code

| | |
|---|---|
| `where_the_answers_are.rs` (new) | `ThePlace::for_this_login(state_home, home)` — the base directory specification's rule, with the variables arriving as **arguments** so a login with neither is a test rather than something to arrange on a real machine |
| `believed_file.rs` | the ownership rule narrows from *root's or this login's* to **this login's**: in a person's own state directory, a root-owned file is not the ordinary case, it is evidence that something else wrote their record |
| `answers_file.rs` | `THE_ANSWERS` becomes `WHERE_IT_USED_TO_BE` — kept as a name so a machine upgraded from a pre-release image can say what the file it no longer reads was |
| `docs/contracts/portal-answers-file.md` | a *Whose it is* section, the new place, and the folder rule |

**One directory is made, and only under one that is there.** The old file lived in
a folder the image makes, so nothing here made folders — a typo would otherwise
become a second record nobody reads. A person's state directory is nobody's to
make but the programs that use it, so `ThePlace::made` makes `alo` (and, under
`$HOME`, the specification's own `.local/state` before it) **inside a base that
already exists**, and refuses when it does not. The typo is still a refusal.

**Nothing is migrated.** A pre-release machine keeps `/var/lib/alo/portal-answers.jsonl`
and nothing reads it: it may hold two people's answers, there is no honest way to
split one, and deleting somebody's record is a thing a person does rather than a
thing an upgrade does.

## The acceptance, taken

`crates/alo-portals/tests/one_persons_answers_are_not_another_persons.rs`:

1. **Two logins, two files.** Ada's backend keeps an answer in Ada's own state;
   Bo's backend, reading Bo's place, gets `NotThere` — the refusal that says
   *nothing has been answered here yet*, never an empty list standing in for a
   question that could not be asked — and no folder of Bo's was made.
2. **The filesystem holds it, not a filter of ours.** The file is then given to
   another login (`chown` to `nobody`, which needs root and therefore the machine
   the gate runs on), and is **refused before a byte of it is read** — and refused
   for writing too, so an answer is never added to a record this login cannot
   vouch for. Where a machine will not chown, that half says what it skipped.
3. **A login with nowhere refuses**, as task 8's *an answer that is not kept is
   not sent* has it: a state directory that is not there, and a login with neither
   variable, are both refusals rather than a guess at `/`.

All 112 tests in `alo-portals` pass; clippy clean.

## Plan hygiene: what was added to this plan after it was written

Asked for explicitly, and worth the count. `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`
was published on **2026-09-13 with five tasks**. It finished with **eleven**:

| Added | Tasks |
|---|---|
| 2026-09-15, as the work was done | 6 (the Settings portal), 7 (an application named by the process the bus holds), 8 (answers kept after the backend stops), 9 (kept as long as the machine's record), 10 (read back in the person's language) |
| 2026-09-16 | 11 — this one |

So six of the eleven were added after the plan was written, all of them by the
lane doing the work, and the last of them sat *ready* while that lane moved to
another plan. That is the shape the fleet's seven one-task-short plans have, and
it is invisible from a count of ticks: the plan looked finished at task 10
because task 10 was the last one anybody had written down.

**Nothing further was added.** This plan is closed at eleven of eleven.

## What is still open elsewhere, and is not this plan's

- **Nothing draws the list.** This plan decided what an application's grant is,
  where the answers go and who may read them; a person reading their own answers
  needs a surface, which is the desktop lane's.
- **The backend is not started in a session by anything here** — the plan's own
  constraint. Whoever starts it passes `ThePlace::for_this_login` the login's own
  `$XDG_STATE_HOME` and `$HOME`, and refuses to answer if it has nowhere to keep
  a record.
