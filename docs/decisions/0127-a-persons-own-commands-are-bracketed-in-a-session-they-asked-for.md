# ADR 0127 — A person's own commands are bracketed in a session they asked for

**Status:** **accepted, 2026-10-19, by the owner.** Proposed the same day by the
Mac lane, which laid out four options and recommended this one; the owner
answered *I accept*. Nothing is built in the change that records it.
**Context:** [ADR 0045](0045-what-undoing-rewinds-to.md) (what undoing rewinds
to, and its six accepted terms); law 5 (*the person chooses*, and *never
offering is not taking away*); `docs/features.md` ★ *scrollback you can move
along*; tasks 7, 8 and 10 of `docs/autonomy/the-terminal-in-full-plan.md`;
`crates/alo-turn/src/the_bracket_a_turn_takes.rs`,
`crates/alo-agentd/src/the_bracket_this_turn_takes.rs`.

## What the owner decided

Shown four options for whether a command the **person** types is bracketed —
every command, none, a session they ask for, or only ones they mark — they
accepted the third.

**Read as: a person turns snapshots on for a session, and for that session every
command they run is a place they can return to.** An ordinary session takes no
snapshots and costs nothing.

*If that reading is wrong, this ADR is the thing to correct, and one sentence
does it.*

## The question this answers

`docs/features.md` promises, with a star:

> ★ **Scrollback you can move along, not only read.** Every command in the
> session is a place you can return to: scroll up, choose one, and your files
> are as they were before it ran.

The scrolling landed on 2026-10-15. **Putting the files back did not**, and
could not, because nothing takes a snapshot of a person's own work.

## What was measured, 2026-10-19

| asked | answer |
|---|---|
| what a bracket is | two read-only snapshots, from `btrfs subvolume snapshot -r` of a subvolume the person owns |
| who takes one | `alo-agentd`, at the moment an agent's change is approved — `TheTurnsBracket` is named in **7** files, all of them in `alo-agentd` |
| does anything in `alo-terminal` or `alo-shell` take one | **no** |
| what ADR 0045 term 5 says | *Only a turn that changes files is bracketed* |
| can alo tell whether `make` changed anything | **no** — ADR 0045 is explicit, and that is why this is a decision rather than a caller |

## Why *every command* was not chosen, although it is what the promise says

Because the four rules the owner set on 2026-09-16 do not survive it, and they
were set in answer to the owner's own question:

> *if the system takes snapshots every turn, will the machines not get fuller
> with them?*

ADR 0045's window is *the last seven days or the last fifty changing turns,
whichever ends first*. **An agent turn is a handful a day; a shell command is
hundreds.** With every command bracketed, fifty is spent before lunch — so by
the afternoon the morning's real mistake has already expired, and the feature
fails exactly when it is wanted. And since alo cannot tell a changing command
from a reading one, `ls` and `cd` would each take a pair of snapshots.

**So *every command* is honest to the sentence and unusable as a machine.**

## Why *only commands they mark* was not chosen

It protects what a person predicted. The commands that ruin a day are the ones
nobody thought twice about, and **a command cannot be marked after it has run.**
It is also the cheapest to build, which is the usual reason a weaker answer gets
taken.

## Why a session, and why that is not a new idea here

The feature **directly above this one** in `docs/features.md` is already

> **Opt in, per session or per project**

so this is the shape the owner has already chosen for this family. It also fits
when a person actually wants it: the moments worth protecting — a codemod across
four hundred files, somebody else's migration script, a `make install` into a
real prefix — are moments known **in advance**.

## What this narrows, said plainly because it is a published promise

**The promise said *every command in the session*. It now means every command in
a session the person asked for.** That is a narrowing of a starred line in
`docs/features.md`, and it is recorded here rather than left for a reader to
notice.

Law 5's *never offering is not taking away* does **not** excuse it: the sentence
was offered, in the file that is the record of what was offered. What makes this
not a law 5 violation is that the owner decided it, knowing the cost, which is
the law's own test — *can the person choose this, knowing what it costs?* — asked
of the owner about the product rather than of a person about a setting.

**`docs/features.md` is edited in the change that builds this**, so the file and
the machine say the same thing. A promise narrowed in an ADR and left standing in
the feature list is the fault law 5's own paragraph was added for.

## What this does not decide

- **How a person asks.** A setting, a word when the terminal opens, a line in
  their settings file — unspecified here. It is a surface, and the design file
  is where a surface comes from.
- **Whether a session's snapshots expire differently.** ADR 0045's window and its
  four rules apply unchanged until something says otherwise; a session that asked
  for snapshots is still subject to *disk pressure wins over undo* and *a machine
  too full to snapshot still runs the command*.
- **Anything about the comparer**, which is the next section.

## What this unblocks, and what it does not

| task | what it needed | after this |
|---|---|---|
| 10 — *choose one, and your files are as they were* | the bracket only — restoring reads a snapshot and compares nothing | **buildable** |
| 8 — bisect the machine | the bracket **and** the comparer — stepping back needs to know what each step changed | still waiting on the comparer |
| 7 — hand somebody a session they can replay | the bracket **and** the comparer — the promise is *the commands **and what each one changed*** | still waiting on the comparer |

**The comparer is not this lane's and is not this decision.** ADR 0045 gives the
privileged comparing path to the third PC in `the-broker-and-the-disk-plan.md`,
and `alo-agentd` already records the gap in its own words: there is deliberately
no state for *both sides are kept and nothing has compared them*, which is
exactly the state of a machine the moment a bracket is taken.

## The test that decides whether this works

1. In a session where the person asked for it, they run a command that writes a
   file, scroll up, choose the line before it, and **the file is as it was** — on
   a real `btrfs` machine.
2. In a session where they did **not** ask, the same command takes **no
   snapshot**, and `alo-measuring`'s answer about what is filling the disk is
   unchanged by running it. This is the half worth writing first: it is the one
   that proves an ordinary day costs nothing.
3. A machine with no room takes no snapshot, **runs the command anyway**, and
   says the command cannot be undone — ADR 0045 term 3, which this does not get
   to weaken.
