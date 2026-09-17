# ADR 0052 — What a person's applications asked for is the person's record, kept in their own state

**Status:** accepted
**Date:** 2026-09-17
**Context:** task 11 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`;
`crates/alo-portals` (`answers_file`, `believed_file`, `shortening`);
`docs/contracts/portal-answers-file.md`;
[ADR 0001](0001-the-capability-model.md) §7,
[ADR 0004](0004-the-organisations-machine.md),
[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md),
[ADR 0040](0040-what-an-applications-grant-is-over.md)

## The decision in one line

The record of **what a person's applications asked for and were told** is the
**person's**, kept per login in their own state directory
(`$XDG_STATE_HOME/alo/portal-answers.jsonl`), readable by that login and nobody
else — and it is **not** the machine's record, not beside the agent's, and not a
thing an administrator reads.

## What was actually undecided

Tasks 8 to 10 made the answers file durable, shortened it under the machine's own
`[record].keeping` rule, and made it read back in a person's language. None of
them decided **whose file it is**, and everything about it pointed both ways at
once:

- the backend answers on a **session** bus, as the login that session belongs to;
- but `alo_portals::THE_ANSWERS` was one machine-wide path,
  `/var/lib/alo/portal-answers.jsonl`, in a folder the image makes and no login
  owns;
- and `believed_file` accepted a file owned by **root or by this login**, which
  is true of either answer and so decides neither.

On a machine with two people, that file would have held both people's
applications' requests in one place, each readable by the other. Nobody chose
that; it was the shape the first version happened to have.

## What ADR 0004 already says, and what it does not

ADR 0004 enumerates what a managed organisation gets, and **the record** in that
list is one thing: *agent executions and refusals, exportable to their own SIEM*.
It also says what an organisation does not get — the content of a person's files,
the content of agent conversations, silent screen or context access, the ability
to act as the person.

A portal answer is none of those, in either direction. It is not an agent
execution: ADR 0040 part 2 is emphatic that an application is not an agent, and
`docs/contracts/portal-answers-file.md` already says this file is not the agent's
record. So ADR 0004's list does not hand this record to an organisation, and this
decision does not take anything away from one — **it says which side of ADR
0004's existing line a record it never named falls on.**

The side it falls on is the person's, for a reason ADR 0004 gives in its own
words: an administrator *cannot watch a person*. A list of every time somebody's
calendar asked for their microphone, their browser asked for the camera, their
editor asked for a folder — with the times, and with what they said each time —
is a description of a person's working day. It is not what was *executed* on the
machine. A machine that filed it where an administrator reads would be watching a
person with extra steps.

[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md) already drew
this line once, for settings: the organisation's bound is in `/etc`, the person's
choice is under `$XDG_CONFIG_HOME/alo/`, *not in `/etc`, where an administrator
would be reading somebody's preferences on their own machine*. This is the same
sentence about a record rather than a preference.

## The two options, and what each costs

**A — every login keeps its own.** The file is in the person's own state, made
`0600`, owned by that login, and shortened under the machine's `[record].keeping`
rule exactly as now.

- The backend already runs as that login on that session's bus, so it writes it
  with the authority it already has. **No new privilege anywhere.**
- Two people on one machine get two files, and neither can read the other's —
  which is the guarantee, held by the filesystem rather than by a field in a line.
- An administrator on a managed machine has the agent's record and nothing more,
  which is what ADR 0004 promised.
- It costs: the folder has to be made, where `/var/lib/alo` was the image's; a
  person's answers do not survive their account being deleted, which is correct;
  and something that wanted a machine-wide view — nothing does — could not have
  one.

**B — one machine file, written by a system service on the sessions' behalf**,
each answer carrying whose session it was, each person reading back only their
own.

- It costs **a new privileged service** on the portal road: something running as
  root that every session asks to write on its behalf. That is a new privilege
  boundary in the exact place ADR 0005's sandbox and ADR 0001's capability model
  are trying to keep narrow, built for a file nobody has asked to read.
- *Each person reads back only their own* would then be **our filter**, in our
  code, over a file that physically holds everybody's. One bug in that filter is
  one person reading another's day. Option A's equivalent bug does not exist,
  because the other file is not open.
- On a managed machine, the whole file is one artefact an administrator can take
  in one act — and the sentence *the administrator cannot watch a person* would
  then depend on nobody choosing to.
- Its only advantage is a single place to look, for a reader who does not exist.

**A is the decision.** B is rejected for the privilege it adds and for turning a
filesystem guarantee into a code guarantee.

## What this means in the code

- **Where:** `$XDG_STATE_HOME/alo/portal-answers.jsonl` when that variable is set
  and absolute; otherwise `$HOME/.local/state/alo/portal-answers.jsonl`, the base
  directory specification's own default. A login with neither has **nowhere** for
  this file, and that is a refusal rather than a guess — `/` is not somebody's
  state directory. The rule is `alo-choosing`'s for settings, applied to a
  record; the variables arrive as arguments so that a machine with neither is a
  test rather than something somebody has to arrange on a real login.
- **Whose:** the file must belong to **the login reading it** — not to root, and
  not to anybody else — and nobody else may write it. This tightens what
  `believed_file` accepted; a root-owned file in a person's state directory is
  now refused rather than believed, because in the person's own directory it is
  evidence that something else wrote it.
- **The folder is made, and only one of it.** `alo` is created `0700` inside a
  state directory **that already exists**, and under `$HOME` the specification's
  own `.local/state` chain is created when the home directory exists. Nothing
  creates a tree under a path that is not there, so a typo is still a refusal
  rather than a second record nobody reads.
- **Shortening is unchanged**: the machine's `[record].keeping` rule still says
  how long a person's answers are kept. The rule is the machine's; the file is the
  person's; an organisation may say *keep nothing longer than thirty days* without
  ever reading a line of it.
- **Nothing is migrated.** A pre-release machine with
  `/var/lib/alo/portal-answers.jsonl` keeps a file nothing reads any more. It is
  not moved into somebody's home directory — a machine-wide file may hold two
  people's answers, and there is no honest way to split it — and it is not deleted
  by us, because deleting a record is a thing a person does. v0.5 has not shipped;
  the image's own release notes say to remove it.

## Consequences

- `docs/contracts/portal-answers-file.md` changes where it says the file is and
  who may read it. It is a v0.5 contract that has not shipped, so this is a
  change to an unreleased surface rather than a break of a published one; the
  contract says so in its own history.
- A person who signs into a second machine does not take their answers with them,
  and should not: the record is of what happened on **this** machine.
- Anything that wants *what applications asked for* across a fleet is asking for
  something this design deliberately does not have.

## Alternatives rejected

**Keep it machine-wide and rely on the reader filtering by session.** That is
option B, above.

**Keep it machine-wide but make it root-only, and have a person read it through a
privileged service.** The same new privilege as B with none of B's single-file
convenience, and the same *our filter* problem.

**Put it in `$XDG_DATA_HOME`.** The base directory specification separates *data*
a person would expect to keep and carry from *state* a program keeps between
runs — logs, history, recently used things. This is a log of what was asked and
answered on one machine, and `$XDG_STATE_HOME` is where the specification puts
exactly that.
