# The record says a command ran

**Date:** 2026-10-16
**Lane:** Mac
**Branch:** `task/mac/the-record-says-a-command-ran`

## What changed, for a person outside this repository

Your machine's record can now say that the agent ran a command, and that it was
refused one. It says what ran, what the command was allowed to reach, how it
ended, and whether the kernel was holding the boundary while it ran.

## Why this, and why it is separate from the box

ADR 0064 keeps three protections on at **every** one of its three levels: the
record, a snapshot before each run, and the egress indicator. The sealed box
landed as `alo-running`, and its own documentation said the record was the
caller's — correct about ownership, and it left a promise with no way to keep
it, because the record had no entry for a command.

It has two now. They take plain values, so this crate does not depend on
`alo-running` and lands on its own rather than behind it.

## Two new kinds of entry, and both are additive

`docs/contracts/record-file.md` already decided this case: *a new kind of
`happened` is additive, and does not raise `format`*. An older reader reports the
line it cannot parse, with its line number, beside everything it could. So
`format` stays `1` and the contract gains both tags and what they carry.

**`ran-a-command`** carries `agent`, `program` (always absolute), `arguments` as
given, `allowed` — the folders the kernel let it reach — `returned`, and
`the-kernel-was-holding-it`.

**`a-command-was-not-run`** carries `agent`, `program` and `why`, the sentence
the person was shown. **No arguments**, for the reason `turned-away` already
gives: the arguments of a request that never ran are whatever the model was
persuaded to send, and keeping them under an entry shaped like every other entry
is how a record gets used to say something nobody did.

## The decision a reviewer should check: this is not a verb execution

`Happened::ran()` answers **false** about a command, and `ran-a-command` has no
`from_approval` and no `against`.

ADR 0064 is explicit that running code is **not** a verb on the broker's list.
So a reader counting verb executions must not find commands among them, and one
wanting both asks for both. `ran()` gained a sentence saying so, because the
next reader to add a variant will have to make the same call and the reasoning
should be there rather than inferred from a `matches!`.

The refusal goes the other way: `was_stopped()` **does** count
`a-command-was-not-run`, and `why_stopped()` carries its sentence, so a review
asking what this machine refused finds it with everything else it refused.

## `returned` is absent rather than zero

A command killed by a signal has no exit status. `0` would say it succeeded and
`-1` would say something no platform said, and both are sentences a person could
act on that nothing said. So the field is an option and absent survives the
disk — asserted, because an absence that silently became a zero on the way to
JSON and back is exactly the kind of thing that reads correctly forever.

## The field nothing writes yet, and why it is there

`the-kernel-was-holding-it` is false only for a run at full trust whose boundary
could not be applied — which `docs/features.md` permits if the person chooses,
*told that the kernel is not watching it, and the record says so*.

Nothing passes false today. The field and its test are here anyway, because a
record that could only ever say one of the two could not keep that promise at
the moment it mattered, and **a field only ever written one way is a field
nobody would notice had stopped working.**

## The compiler walked four crates, and that is the design working

Adding a kind of entry is additive on disk and **not** additive in code: the
readers match exhaustively on purpose. `alo-updating` says why in its own words
— *a kind of entry added to the record and not given an answer here is a build
that fails rather than a change an agent made that quietly answers the wrong
thing.*

So the compiler walked every reader and each one needed a real answer:

| crate | what it had to answer |
|---|---|
| `alo-record` | ten accessors: whose authority, which approval, which grants, what was stopped and why |
| `alo-recounting` | the clause a person reads back, which is two new words |
| `alo-updating` | what one entry means as far as undoing cares |
| `alo-keeping-up` | whether there is any road back for it |

**The two words are the only person-facing strings added**, and the vocabulary
snapshot is regenerated in the same change — `alo-saying` is the machine's one
vocabulary and adding a word without regenerating breaks the build, which only
a whole-workspace run catches.

## The classification a reviewer should check hardest

`alo-updating` maps a command to a **new** `WhatWasDone::ACommandRan` rather
than to the existing `ChangedSomethingElse`, and the difference is not
cosmetic. That variant means *a verb that changed something which is not a file
of the person's* — a printer, a network. **A command can change anything inside
its box, the person's own files included.** A person told *it changed something
else about the machine* about a command that rewrote their project would have
been misled by a classification, which is the one thing that enum exists to
prevent.

What there is no road back from is then the honest part:
`could_be_put_back` answers `NotOneOfTheChangesPutBack`, because what a program
did cannot be worked out backwards the way a rename can. **No new refusal
reason was invented for it**, because that one's own translator note already
covers this case in writing: *the honest answer for anything the machine has not
been taught to return, including something added after this was written.* The
road ADR 0064 actually promises is the snapshot, which is a different question
asked somewhere else — and is blocked, below.

## Measurements: where each one runs again

| what | where it runs again |
|---|---|
| both entries round-trip through JSON byte for byte | `alo-record`'s unit tests, every build |
| a command is not counted as a verb execution | same |
| a refused command *is* counted among refusals | same |
| a signal-ended command keeps no exit status, through the disk | same |
| a run the kernel was not watching records as such | same |
| the tags appear in the written JSON | same |
| no arguments are written for a refused command | same |

All seven run in the merge gate on every build. None of this needs root, a
kernel or a disk.

The vocabulary snapshot is the eighth and it runs there too: `alo-saying`'s
`the_snapshot_is_what_this_machine_says` fails on a word added and not
regenerated.

## What is still owed

- **Nothing calls these yet.** `alo-running` returns `WhatHappened`, which
  carries exactly what the first constructor needs, and the wiring is the next
  change — `ran` should *require* somewhere to write, the way
  `alo_turn::Kept` makes a turn's record structural rather than remembered.
  Until then the record can say a command ran and nothing says it.
- **The snapshot before each run is blocked**, and not by this. Nothing creates
  `/var/lib/alo/undo` and nothing takes a bracket; the published contract says
  the folder is `0700 root:root` while `alo-letting-go` says taking a bracket
  needs no capability. Both cannot hold. Reported separately.
