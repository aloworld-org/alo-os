# Contract — how a person asks their machine to forget what it is keeping

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the folder a person's own session leaves one act in, and the file it
leaves. [ADR 0045](../decisions/0045-what-undoing-rewinds-to.md) point 5 says
*what an undo may keep is visible and forgettable, and **forgetting it is one
act***; the owner narrowed the road on 2026-09-22 to **a person's act in
Settings, and never a broker verb**. This is that road, written down, because
two processes meet on it and neither may guess at the other.

`docs/contracts/kept-undo-folder.md` is the folder this act removes from.

**Nothing an agent can reach is here.** There is no verb over any of this and
there is not going to be one (ADR 0045's seventh term): an agent that can forget
an undo can erase the evidence of what it did.
`crates/alo-letting-go/tests/a_turn_cannot_arrive_at_this_road.rs` holds the
wider sentence — that the only road into that crate from where a turn runs is
its vocabulary, and that nothing a turn runs inside so much as names this
folder.

## Where it is

```
/run/alo/asked-to-forget
```

`1733 root:root`, made by `systemd-tmpfiles`.

**On `/run`, and that is part of the contract.** It is cleared when the machine
starts, so an approval to forget can never outlive the session it was given in.

**Every digit of the mode is a decision.**

- `7` for root, which is the only thing that reads it: the privileged unit walks
  it, takes everything in it, and removes what it took.
- `3` for the group and `3` for everybody — write and search, and **no read**.
  Anybody with a session may leave an asking, because everybody with a session
  is a person who may forget their own undo. Nobody may list what is in it,
  because who has asked to forget their own history is nobody else's business.
- The **sticky bit**, so one person cannot remove or rename another's asking.
  Without it, a machine with two people on it has a way for one of them to stop
  the other asking.

**A folder anybody may write to is the right shape here**, and the reason is the
next section: nothing left in it carries any authority.

## The file

```json
{"format":1,"approved":1760000000}
```

- `format` is `1`, and a file saying anything else is refused **whole and
  first**, before its other keys are judged, so a file a later alo OS wrote is
  not reported as a missing field.
- `approved` is the moment the person approved the sentence, in **whole seconds
  since the epoch**.

**Its name is never read.** It exists so that two askings cannot collide and so
that nobody can stop somebody else asking by taking a name first. What is read
is the content and who owns it.

### It names nobody and nothing

There is **no person, no folder, no path and no turn** in it, and there will not
be: there is nothing in an asking to aim. **Whose act it is, is the user the
filesystem records as having written the file**, which the kernel sets and a
writer cannot spell for itself. The machine matches that against the owner of
the settings folder each person's own session wrote down in `theirs.json`
(`docs/contracts/kept-undo-folder.md`), and forgets for whoever matches and for
nobody else.

So a stranger with a session on the machine can cause exactly one thing by
leaving a name here: their own undo being forgotten, if they have any. Somebody
else's is not a refusal that had to be got right — it is not expressible.

### What a person approved

`alo_keeping_up::WhatWasKept::forgetting`, in their own language, and there is no
second wording of it anywhere. It says both of the things point 5 asks: that
nothing an agent has changed can be put back afterwards, and that the space it
holds is freed.

## What the machine does with it

`crates/alo-letting-go`, as `alo-forgetting.service`, which
`alo-forgetting.path` starts when this folder stops being empty and which
nothing else starts.

1. **Everything in the folder is taken first**, before anything is decided and
   whatever is decided — read, then removed. Two reasons, both about a
   destructive act: an approval that survives being acted on can be acted on
   again at a moment nobody chose (ADR 0001 §5, *one approval causes exactly one
   execution*), and an asking left behind starts the unit again for as long as
   the fault lasts.

   **What that costs is stated rather than hidden:** an asking taken on a
   machine that could not carry it out is spent, the journal says so, and the
   person asks again.
2. **An asking older than an hour is taken and not carried out.** An approval is
   never a session. The path unit starts the machine's half within moments, so
   an asking that reaches this age is one a stopped or masked unit sat on, and
   acting on it then would forget turns made after it was given. An asking from
   a moment that has not happened is refused the same way, beyond the width of
   one second boundary between two reads of one clock.
3. **Anything that is not a file of its own is refused** — a link most of all,
   which is the oldest way there is of making a privileged reader answer about
   somebody else's name. It is taken anyway.
4. **On a machine that keeps nothing** the answer is *not yet on this machine*
   (ADR 0045's sixth term) and nothing is read, walked or removed.
5. **Everything that person's machine was keeping goes**, whatever their window
   says and whatever the disk says. That is what they approved: not the oldest
   and not the expired, all of it, in one act — a machine that made them reclaim
   their disk a turn at a time would be keeping it by attrition.
6. **What went is written down** to `/var/lib/alo-forgetting/record.jsonl` as a
   `let-go` entry with `why` of `the-person-asked-to-forget`
   (`docs/contracts/record-file.md`), **after** the snapshots are gone and only
   for the turns whose snapshots actually went.

## What is not here

- **No window and no setting.** How far back an undo reaches is the person's
  one setting, in `undo.toml` in their own folder
  (`docs/contracts/person-settings.md`), and this act does not touch it:
  forgetting is not a window of nought, which `alo_keeping_up::HowFarBack`
  refuses by name. The next changing turn is kept again.
- **No second lever.** There is one act and it forgets everything. Not one per
  turn and not one per folder (ADR 0045 point 5).
- **No record.** What was forgotten is written outside the folder it was
  removed from.
