# Forgetting what is kept, as one act a person asks for

**Date:** 2026-09-22
**Workstream:** v0.5 — the machine keeps itself
([`../v0-5-the-machine-keeps-itself-plan.md`](../v0-5-the-machine-keeps-itself-plan.md),
task 14)
**Contributor:** this development PC's lane
**Status:** ready for integration.

## What this is

[ADR 0045](../../decisions/0045-what-undoing-rewinds-to.md) point 5 says *what
an undo may keep is visible and forgettable, and **forgetting it is one act***.
Half of it existed, and the wrong half:
`alo_keeping_up::WhatWasKept::forgetting` is the sentence a person approves — in
their own language, saying both of the things point 5 asks it to say — and
**nothing carried it out**. A machine could show a person a sentence it could
not act on, which is worse than not offering it.

It is the last of ADR 0045's seven terms that nothing built. It is built now.

## The road, and why it is this one

The removal needs `CAP_SYS_ADMIN` and a person's session holds none, so the act
has to cross into something privileged. The owner narrowed the choice on
2026-09-22: **the road is a person's act in Settings, and never a broker verb.**
What remained was three candidates, and the plan asked what each costs the
*afternoon's audit* that ADR 0001 §2 is about.

| Road | What it costs the audit | Taken |
|---|---|---|
| **Widen the timer unit to notice a file** | Nothing — the same unit, the same capabilities, the same fixed command line. But a person who asks for their disk back gets it at the next daily firing, up to a day later, which is a machine that looks broken. And one unit would have two jobs (law 4). | no |
| **A `polkit`-authorised action** | A whole authorisation mechanism this repository does not otherwise have, ending in a person's session being able to start a privileged unit — a road worth attacking, added for one act that needs none of it. | no |
| **A second unit, started when a person's session leaves a file** | One more unit to read: a fixed `ExecStart`, no arguments, no environment, the expiry unit's three capabilities and no fourth, and a drop folder whose mode is the one thing an auditor has to reason about. | **yes** |

So: **`alo-forgetting.service`**, which **`alo-forgetting.path`** starts when
`/run/alo/asked-to-forget` stops being empty, and which nothing else starts. It
is the shape [ADR 0049](../../decisions/0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md)
§3 and [ADR 0053](../../decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
already use — *what the door cannot carry is handed over as a file* — with the
door taken out of it.

### The decision that makes the drop folder safe

A folder anybody may write to, feeding the one privileged remover on the
machine, would be alarming if what was left in it carried any authority.
**It carries none.** The file is:

```json
{"format":1,"approved":1760000000}
```

No person, no folder, no path, no turn. There is nothing in it to aim.
**Whose act it is, is the user the filesystem records as having written it**,
which the kernel sets and a writer cannot spell for itself; the machine matches
that against the owner of the settings folder each person's own session wrote
down in `theirs.json`, and forgets for whoever matches and for nobody else.

So *an act that arrives for somebody else's folder* is not a refusal that had to
be got right. **It is not expressible.** The worst a stranger with a session can
do by leaving a name in that folder is start a unit that finds nothing of theirs
to forget.

The mode is `1733 root:root` and every digit is argued in the `tmpfiles`
fragment: `7` for root, the only reader; `3` and `3` — write and search, no
read, because anybody with a session may forget their own undo and who has asked
is nobody else's business; and the sticky bit, so one person cannot stop another
asking by removing their file. It is on `/run`, which is cleared at every start,
so an approval can never outlive the session it was given in.

### Every asking is taken, and taken first

`asking::taken` removes everything it looked at **before anything is decided**
and whatever is decided. Two reasons, both about a destructive act:

- **One approval is one execution** (ADR 0001 §5). An approval that survived
  being acted on is one that can be acted on again, at a moment nobody chose.
- **A path unit re-fires while its folder is not empty.** An asking left behind
  because the disk would not answer is a unit started again, and again.

What that costs is written down rather than hidden: an asking taken on a machine
that could not carry it out is **spent**, the journal says so, and the person
asks again. That is the right way round for an act that removes a person's own
history; the wrong way round is an approval lying about on a disk waiting for a
moment that suits it. An asking older than **an hour** is taken and refused for
the same reason — an approval is never a session, and one that old would forget
turns made after it was given.

## What changed

| File | What |
|---|---|
| `crates/alo-letting-go/src/asking.rs` | The road: `THE_ASKING`, `Asked` and its refusals, `taken` (the machine's half), `ask` (the person's half), `NotAsked` |
| `crates/alo-letting-go/src/whose.rs` | `WhoOwns` — who owns a name on a disk, **never following a link**, which is what settles whose act an asking is |
| `crates/alo-letting-go/src/forgetting.rs` | `forget`: take, ask the disk, match the owner, remove everything that person was keeping, write it down |
| `crates/alo-letting-go/src/bin/alo-forgetting.rs` | The unit's program — no arguments, no environment |
| `crates/alo-letting-go/alo-forgetting.service`, `.path`, `.conf` | The unit, the one thing that starts it, and the one directory it needs |
| `crates/alo-letting-go/src/words.rs` | One string: an asking that never left the person's session. **Not** the sentence they approve |
| `crates/alo-letting-go/src/writing_it_down.rs` | `THE_FORGETTING_RECORD` — its own record file |
| `crates/alo-record/src/let_go.rs` | `WhyLetGo::ThePersonAskedToForget`, additive; `format` stays `1` |
| `crates/alo-recounting/src/words.rs`, `told.rs`, `testing.rs` | The clause a person reads for it, and the fixture the test of it is built from |
| `docs/contracts/asked-to-forget-folder.md` | New. The road, written down, because two processes meet on it |
| `docs/contracts/record-file.md`, `kept-undo-folder.md` | Their additive sections |
| `docs/decisions/0045-what-undoing-rewinds-to.md` | Point 5 narrowed and built; all seven terms are built |

### Two record files, and why

`alo-keeping` says it outright: *two appending to one file would interleave*, so
a record file has one writer. The expiry unit fires off a timer and this one
fires on a person's act, and systemd will happily run them in the same second.
So this writes `/var/lib/alo-forgetting/record.jsonl` and the expiry keeps
`/var/lib/alo-letting-go/record.jsonl`. A record whose lines can be cut in half
is worse than a record in two places: a reader can open two files and cannot
mend one line. The cost was already paid — a surface putting *what this machine
did* in front of a person has read more than one record file since the broker
gained one.

### The third reason, told apart from the other two

`alo_record::WhyLetGo` gained `ThePersonAskedToForget`, and
`WhyLetGo::is_the_persons_own` is how a reader asks which kind it is. The two
existing reasons are the machine tidying up and telling somebody; this one is
the answer to something they asked for. A reader that found all three under one
word would have lost the one difference a person acts on — which is why
`alo-recounting`'s clause for it is addressed to them in the second person.

## Measured on a real `btrfs` filesystem

`cargo test -p alo-letting-go --test on_a_real_btrfs_machine -- --ignored
--nocapture --test-threads=1`, as root in this machine's WSL Ubuntu, on a
`btrfs` filesystem made on a loop device. All four tests pass — task 13's two
unchanged, and this task's two. Output as printed:

**One act forgets everything that was kept.** Three kept turns, every one of
them *inside* the shipped window, on a filesystem with room to spare — so
neither the window nor the disk would have taken any of them.

```text
--- btrfs subvolume list, before ---
ID 256 gen 12 top level 5 path home/ada
ID 257 gen 7 top level 5 path undo/ada/c/before
ID 258 gen 8 top level 5 path undo/ada/c/after
ID 259 gen 9 top level 5 path undo/ada/b/before
ID 260 gen 10 top level 5 path undo/ada/b/after
ID 261 gen 11 top level 5 path undo/ada/a/before
ID 262 gen 12 top level 5 path undo/ada/a/after

--- without CAP_SYS_ADMIN ---
Delete subvolume 261 (commit): '…/mnt/undo/ada/a/before'
ERROR: Could not destroy subvolume/snapshot: Operation not permitted
WARNING: deletion failed with EPERM, you don't have permissions or send may be in progress

--- btrfs subvolume list, after ---
ID 256 gen 12 top level 5 path home/ada
```

Seven subvolumes down to one — the person's own home, untouched, with the file
in it still there. The record holds one entry, `the-person-asked-to-forget`,
naming *archive March*, *move April.pdf* and *rename May.pdf* in the words the
person approved when those turns ran. The act run a second time against the same
folder does nothing at all: the approval was taken before anything was removed.

**Nobody else's undo is forgotten.** Two people, two home subvolumes, two
settings folders — and Bo's `chown`ed to another user, so that *whose* is the
filesystem's answer and not a fixture's. Ada asks.

```text
--- btrfs subvolume list, before ---
ID 256 gen 8 top level 5 path home/ada
ID 257 gen 10 top level 5 path home/bo
ID 258 gen 7 top level 5 path undo/ada/one/before
ID 259 gen 8 top level 5 path undo/ada/one/after
ID 260 gen 9 top level 5 path undo/bo/one/before
ID 261 gen 10 top level 5 path undo/bo/one/after

--- btrfs subvolume list, after ---
ID 256 gen 8 top level 5 path home/ada
ID 257 gen 10 top level 5 path home/bo
ID 260 gen 9 top level 5 path undo/bo/one/before
ID 261 gen 10 top level 5 path undo/bo/one/after
```

And the refusal is measured beside the act, in the same run:
`capsh --drop=cap_sys_admin` answers *Operation not permitted* and the machine
is exactly as it was, which is the whole reason this is a unit and not a line in
a person's own session.

## A turn cannot take this road, and the honest version of that test

The plan asked by name for a test that this road is **unreachable from an
agent's turn** — not merely absent from the verb list. The first version of it
asserted that nothing a turn links reaches this crate, and **it was false**:
`alo-agentd` ships `alo-saying`, which is the machine's one vocabulary and
therefore ships every crate that declares a word, this one included. A test
claiming otherwise would have been simply wrong, and the day somebody noticed,
the promise it stood for would have gone with it.

So `tests/a_turn_cannot_arrive_at_this_road.rs` says the thing that is both true
and the point. Walking the workspace's manifests from `alo-agentd` and
`alo-turn`, over shipped dependencies only:

- **the only crate in that set that ships this one is `alo-saying`**, and a
  second crate reaching for a type, a constant or a function of this one fails
  the test in the change that did it;
- **the only names anything there spells of this crate are `declare_into` and
  `letting_go_words`** — a vocabulary, never an act;
- and **nothing in that set names the road at all**: not `/run/alo/asked-to-forget`,
  not `alo-forgetting`, and not either of the two functions that are the act.

`tests/nothing_here_is_a_verb.rs` is untouched and still passes:
`alo_broker::SystemVerb` gained nothing, no name on its list begins `undo.`, and
`alo-turn` gains no capability. Task 13's tests are all untouched and all pass —
including `a_timer_starts_it_and_nothing_else_can`, which
`nothing_added_here_starts_the_expiry_unit` now guards from the one way this
change could have quietly made it untrue.

## Decisions a worker made, and why

1. **In `alo-letting-go` rather than a new crate.** That crate's own
   documentation said this would be *its own change — additive, through the same
   remover*, and it is: the same `Removing`, the same folder, the same reading
   of `theirs.json`. A second crate would have been a second copy of all three.
2. **Not in the person's settings folder.** An earlier shape put the asking
   beside `undo.toml`. `docs/contracts/person-settings.md` governs that folder
   and every file in it is *what the person changed*, kept by one rule; a
   one-shot request is not a setting, and putting one there would have broken a
   contract to save a directory.
3. **The asking names nobody.** Considered and rejected: a `whose` field naming
   the person's directory, checked against ownership. It would have worked, and
   it would have made the act *aimable* — a selector in a file anybody can write,
   in front of the one privileged remover. A file with nothing in it to aim needs
   no check to be got right.
4. **Taken before acting, not after.** After would have been safe for the act
   itself, since forgetting everything twice forgets nothing the second time.
   Before is what makes *one approval, one execution* observable, and it is what
   stops a path unit looping on an asking a fault left behind.
5. **An hour's lifetime.** `/run` already bounds an approval to one uptime;
   this bounds it to one sitting. Spelt here rather than borrowed from
   `alo_broker::spent`, because this crate deliberately names no door.
6. **Two record files**, above.

## What this does not do

**Nothing draws any of this in Settings.** The person's half is a function their
session calls (`alo_letting_go::ask`), with the sentence they approve
(`Asked::the_sentence`) and the refusal they read (`NotAsked::said`) beside it,
and there is no pane that calls it. Point 5 has two halves — *visible* and
*forgettable* — and this change is the second. **Task 15 is the first**, written
into the plan by this change, and it is buildable now: every piece it needs
exists and none of it is in front of anybody.

**The units are not installed.** They live beside the crate, as task 13's do,
until the image lane carries these programs — `alo-forgetting.service`,
`alo-forgetting.path`, `/usr/libexec/alo-forgetting`, and the `tmpfiles` line
that makes `/run/alo/asked-to-forget`. Every line of all three is held by
`tests/forgetting_is_the_persons_own_act.rs`, so what the image owes is exact.

**No real alo OS machine has forgotten anything through the path unit.** What is
measured is this crate's code against a real `btrfs` filesystem, including the
capability it needs and the refusal without it. That a `systemd` path unit
really starts this service when a file appears is the image lane's
virtual-machine acceptance, like the update units' before it.

## Verification

Run from this checkout, in this machine's WSL Ubuntu, `CARGO_TARGET_DIR=/root/alo-builds/this-machine`.

| What | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test -p alo-letting-go -p alo-record -p alo-recounting` | all pass |
| `cargo test -p alo-letting-go --test on_a_real_btrfs_machine -- --ignored` | 4 passed, as root on a real `btrfs` loop device (output above) |
| `cargo doc -p alo-letting-go -p alo-record -p alo-recounting --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean |
| `cargo test -p alo-citing` (the citation check — `docs/decisions/` changed) | all pass |
| `cargo test` in `tools/kernel-loop` (a plan changed) | 152 pass |

**Not run here:** the whole-workspace suite and the BPF gates, which the
supervisor runs on the combined tree. Nothing on real hardware and nothing on a
certified machine: this is a developer's loop device, and no claim about a
booted alo OS is made from it.

## Proposed shared-document updates

Not made here — `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` and
`docs/autonomy/STATE.md` have one writer (`../SHARED_MAIN.md`).

**Changelog, in a person's words:** *A person can now tell their machine to
forget everything it was keeping so that the assistant's changes could be undone
— one act, and it frees the space. What it forgot is written down by name, in
the words they approved when each change was made, so the record still says what
can no longer be put back. No agent can ask for it, and no agent can reach the
road that carries it.*

**Queue/roadmap:** ADR 0045's seven accepted terms are now all built. The v0.5
line ★ *undo what the agent did* still waits on the bracket
(`crates/alo-turn`, lane A) for the putting-back half; what this plan owns of it
is finished apart from the Settings pane, which is task 15.
