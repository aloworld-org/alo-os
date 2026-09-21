# The snapshot nobody removed

**Date:** 2026-09-21
**Workstream:** `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, task 13.
**Contributor:** this development PC's worker lane.
**Status:** ready for integration. Focused acceptance run and recorded below;
the workspace suite is the supervisor's.

## What was wrong

[ADR 0045](../../decisions/0045-what-undoing-rewinds-to.md)'s first two accepted
terms both end in a snapshot being **removed**: what falls outside a bounded
window goes, and under a named amount of free space the oldest go first.
`crates/alo-keeping-up/src/how_far_back.rs` decided precisely which ones, with
no clock and no file, and it was right. **Nothing removed them.** A search of
every crate for a snapshot deletion found none.

So a machine decided that a snapshot had expired and then kept it for ever —
the disk filling quietly that the owner's question at acceptance was about,
answered in prose and not in code. The window was built; the forgetting was not.

## What changed

**A new crate, `crates/alo-letting-go`, and a privileged unit a timer starts.**
One firing reads the folder, asks each person's own window what is outside it,
removes exactly that with the base's own program, and writes down which turns
can no longer be put back.

| File | What it is |
|---|---|
| `src/the_folder.rs` | The layout, as types: `THE_FOLDER`, `Theirs` (`theirs.json`), `TheTurn` (`kept.json`), `NotWhatItSays`, and days counted **down** the way the window counts them |
| `src/found.rs` | That folder read off a disk — `Everyone`, `Whose`, `Found`, newest first — and everything it would not read and therefore left exactly where it was |
| `src/changes.rs` | `Changes` and `Settings`: how far back this person asked, over what the release ships |
| `src/keeping.rs` | `undo.toml`, kept by the rule of [ADR 0038](../../decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md) through `alo-kept` |
| `src/unkept.rs`, `src/words.rs` | What a person reads when that file will not read — eight strings, collected by `alo-saying` |
| `src/deciding.rs` | `WhatGoes::decided`: what the window no longer reaches, oldest first, and then what it does, also oldest first |
| `src/the_disk.rs` | `THE_FLOOR`, and the two questions `statfs` answers: is this a filesystem that keeps, and how much room is left |
| `src/removing.rs` | `btrfs subvolume delete --commit-after`, fixed arguments, no shell |
| `src/sweeping.rs` | One firing, whole |
| `src/writing_it_down.rs` | `THE_RECORD`, and the one way this crate reaches a record file |
| `src/bin/alo-letting-go.rs` | The unit's program: the machine's own paths, and nothing else |
| `alo-letting-go.service`, `alo-letting-go.timer` | Root, three capabilities, a fixed command line, no `[Install]` — and a timer as the only thing that starts it |

**Beside it, additively:**

- `crates/alo-record`: `Happened::LetGo { why, turns }` and `Entry::let_go`,
  with `src/let_go.rs` holding `WhyLetGo` and `Forgone`. `format` stays `1`.
- `crates/alo-recounting`: two outcomes and two strings, so a person reading
  their record is told which it was.
- `crates/alo-updating`: `putting_back.rs`'s exhaustive table answers the new
  kind — *not an agent's*.
- `docs/contracts/kept-undo-folder.md`, new: the folder two lanes meet in.
- `docs/contracts/record-file.md` and `docs/contracts/person-settings.md`: the
  `let-go` kind and the `undo.toml` row and section.

### What a person sees

They can say how far back their machine keeps what the assistant changed to
their files, in their own settings, and it is the only lever there is: no *keep
this one for ever* and no *never expire*. What falls outside goes, on its own,
once a day. If the disk is running out of room the oldest go first, and the
machine never fills a disk to keep something it could put back. Either way the
record says which pieces of work can no longer be put back — **named in the
words they approved at the time**, not counted — so they find out from their own
history rather than from an offer that is not there. No assistant can ask for
any of it.

## What was decided, and why

Nine things were open. Each was chosen the way a senior engineer would and is
written down here rather than left for the next reader to infer.

1. **A new crate rather than a line in an existing one.** `alo-keeping-up` may
   not gain it: it has no clock, no socket and no file, its dependency list is
   four crates long, and its own test reads that list. `alo-brokerd` may not
   either: this unit is started by a timer and never by the broker, and putting
   it there would put a timer's program inside the crate whose whole promise is
   that it holds no capability. The name is the vocabulary ADR 0045 already
   uses — `WhatWasKept::LetGo` is what a person meets when this has run.
2. **The folder's layout is a contract, not a shared constant.**
   `docs/contracts/kept-undo-folder.md`. Two processes meet in it, one as the
   person and one as root, and the one that writes it does not exist yet. A
   constant they could both `use` would have been enough for a compiler and not
   enough for the lane that has to write `alo-turn`'s half.
3. **The moment is in a file, and both directory names are opaque.** A name is
   read by everything and checked by nothing; the first reader that parses a
   moment out of one differently is a machine removing the wrong snapshot.
4. **The sentence the person approved is copied beside the snapshot.** ADR
   0045's second term needs the record to name the turn *in the person's words*.
   The remover runs as root, on a timer, long after the turn, and the person's
   own record is not its to read — so the process that had those words writes
   them down. It is the same *a copy, not a pointer* `Happened::Undone` already
   keeps, for the same reason: the record is shortened, and a position in it is
   not a name that lasts.
5. **Where the person's settings are is written down by their own session.**
   The window is the person's, in `undo.toml` in `$XDG_CONFIG_HOME/alo` — and
   `$XDG_CONFIG_HOME` is a variable of a *session*, which a root unit on a timer
   cannot know. Guessing `$HOME/.config` would quietly give the shipped window
   to exactly the people who had moved theirs. So `theirs.json` names the folder,
   the unit reads the person's own file at the path the person's own session
   named, and **no window is copied anywhere** — a second copy of a value is a
   value that can disagree with itself.
6. **The floor is ten gibibytes, and it is not a setting.** An amount rather
   than a share of the disk, because what a machine needs room for is an amount:
   `bootc` stages a whole system image before a person restarts into it, and a
   floor under that would be a machine that kept an undo and could not take the
   update it had told the person was ready — two promises in `docs/features.md`,
   with the wrong one winning. Not a setting, because ADR 0045's seventh term is
   explicit that the only way to keep a snapshot longer is to widen the window,
   and a second lever over the same snapshots is how a machine ends up with
   nobody able to answer *when will this be gone*.
7. **`--commit-after`, and one at a time.** `btrfs subvolume delete` returns
   before the space comes back. Under the floor the machine removes one and asks
   the filesystem again, so a deletion that had not yet freed anything would
   read as *that did not help* and take the next one, and the next. `-C` waits
   for the commit, which makes the answer afterwards mean what the loop reads it
   to mean.
8. **Its own record file, `/var/lib/alo-letting-go/record.jsonl`.** Not the
   person's — that one is `alo-agentd`'s and has exactly one writer, because two
   processes appending to one file interleave. It is the shape `alo-brokerd`
   already has and for the same reason. **The cost is named rather than hidden:**
   a surface putting *what this machine did* in front of a person reads more
   than one record file, and has had to since the broker gained one. What this
   crate owes such a surface is an entry it can word, and `alo-recounting` can
   word this one.
9. **Nothing that did not read is ever removed.** A person's directory whose
   `theirs.json` will not read is stepped over whole; one kept turn whose
   `kept.json` will not read is stepped over on its own; both leave the snapshots
   where they are. A turn the machine cannot describe cannot be named, so
   removing it would take a person's undo away and leave nothing able to tell
   them it had gone. A snapshot left behind costs disk; one removed silently
   costs the only account there is.

## What was measured, on a real `btrfs` filesystem

`crates/alo-letting-go/tests/on_a_real_btrfs_machine.rs`, `#[ignore]`d and run
by name, as root in this lane's WSL Ubuntu 24.04 (kernel
`6.18.33.2-microsoft-standard-WSL2`, `btrfs-progs v6.6.3`). A `btrfs`
filesystem is made on a loop device, a person's home is made a subvolume,
read-only snapshots are taken either side of each turn with the base's own
program, and `btrfs subvolume list` is asked before and after. It **fails**
rather than skips when anything it needs is missing.

```
cargo test -p alo-letting-go --test on_a_real_btrfs_machine -- --ignored --nocapture --test-threads=1
```

### One: the window, on a machine with room to spare (16 GiB)

Two kept turns — one a day old, one thirty days old — against the shipped
window of seven days or fifty changing turns, read off the person's own
`undo.toml` on that disk.

```
--- btrfs subvolume list, before ---
ID 256 gen 10 top level 5 path home/ada
ID 257 gen 7 top level 5 path undo/ada/recent/before
ID 258 gen 8 top level 5 path undo/ada/recent/after
ID 259 gen 9 top level 5 path undo/ada/old/before
ID 260 gen 10 top level 5 path undo/ada/old/after

--- btrfs subvolume list, after ---
ID 256 gen 10 top level 5 path home/ada
ID 257 gen 7 top level 5 path undo/ada/recent/before
ID 258 gen 8 top level 5 path undo/ada/recent/after
```

The turn outside the window is gone, the turn inside it is untouched, the
person's own home is untouched, and the record holds one `let-go` entry with
`why` = `outside-the-window` and one turn named `archive Old letters`.

### And the refusal, measured beside it

Before that run, the same snapshot was attacked two other ways. An ordinary
removal fails — a read-only snapshot does not yield to `rm -rf`. And the base's
own program, with the capability dropped:

```
--- without CAP_SYS_ADMIN ---
Delete subvolume 259 (commit): '…/mnt/undo/ada/old/before'
ERROR: Could not destroy subvolume/snapshot: Operation not permitted
WARNING: deletion failed with EPERM, you don't have permissions or send may be in progress
```

Five subvolumes before, five after: the machine is exactly as it was. **That is
the whole reason this is a unit and not a line in `alo-turn`.**

### Two: the disk, on a machine under the floor (1 GiB)

Three kept turns, **all of them inside the shipped window** — one, two and three
days old. Nothing here would go for the window's sake.

```
--- btrfs subvolume list, before ---
ID 256 gen 12 top level 5 path home/ada
ID 257 gen 7 top level 5 path undo/ada/c-newest/before
ID 258 gen 8 top level 5 path undo/ada/c-newest/after
ID 259 gen 9 top level 5 path undo/ada/b-middle/before
ID 260 gen 10 top level 5 path undo/ada/b-middle/after
ID 261 gen 11 top level 5 path undo/ada/a-oldest/before
ID 262 gen 12 top level 5 path undo/ada/a-oldest/after

--- btrfs subvolume list, after ---
ID 256 gen 12 top level 5 path home/ada
```

`outside_the_window` = 0 and `for_room` = 3, the person's home untouched, and
one `let-go` entry with `why` = `the-disk-needed-the-room` naming the three
turns **in the order they were let go**: `archive March`, `move April.pdf`,
`rename May.pdf`. The oldest first.

**What this measurement cannot show, said plainly.** A loop device on this
machine cannot climb back over a ten-gibibyte floor by removing a few
snapshots, so *the machine stops the moment there is room* is not measured
here. It is held by
`sweeping::tests::under_the_floor_the_oldest_go_first_and_it_stops_when_there_is_room`,
against a disk that gives room back after each removal: two of three go and the
third is never asked for.

## Acceptance, criterion by criterion

Every line below was run on its own before this was written. The workspace is
`.`; the crate is `alo-letting-go` unless another is named.

| Criterion | Test |
|---|---|
| A privileged unit, started by a timer and by nothing else | `the_expiry_is_a_unit_on_a_timer::a_timer_starts_it_and_nothing_else_can` |
| It reads the window from the person's settings and removes every snapshot outside it, **measured on a real `btrfs` machine** with `btrfs subvolume list` asked afterwards | `on_a_real_btrfs_machine::on_a_real_btrfs_machine_the_expired_snapshots_go_and_the_rest_stay` |
| Under the named free-space floor the **oldest go first**, and the record says which turns can no longer be undone, in the person's words and not as a number | `on_a_real_btrfs_machine::on_a_real_btrfs_machine_under_the_floor_the_oldest_go_first` |
| …and it stops the moment there is room again | `sweeping::tests::under_the_floor_the_oldest_go_first_and_it_stops_when_there_is_room` |
| The window is the person's one setting, and widening it keeps snapshots a narrower one would have taken | `sweeping::tests::a_wider_window_keeps_what_the_shipped_one_would_have_taken` |
| The unit is **not reachable from any verb**: no name on `alo_broker::SystemVerb`'s list begins `undo.`, and `SystemVerb` gained nothing | `nothing_here_is_a_verb::no_verb_on_the_brokers_list_begins_undo`, `nothing_here_is_a_verb::the_brokers_list_is_exactly_as_long_as_it_was` |
| `alo-turn` gains **no** capability | `nothing_here_is_a_verb::alo_turn_gains_no_capability` |
| …and this crate cannot be asked for anything at all | `nothing_here_is_a_verb::nothing_here_can_be_asked_for` |
| `alo-measuring` still counts what undo is holding by name, and the number goes **down** after the unit runs | `what_undo_is_holding_goes_down::what_undo_is_holding_is_a_line_of_its_own_and_goes_down_after_the_unit_runs` |
| A machine on `ext4` does nothing, rather than failing every timer (term 6) | `sweeping::tests::a_machine_that_keeps_nothing_does_nothing` |
| The deciding stays where it is: `how_far_back.rs` is asked, never re-decided | `deciding::tests::what_the_window_no_longer_reaches_goes_and_the_rest_stays` |

### The refusal paths, tested beside the legitimate ones

| What is refused | Test |
|---|---|
| A removal the machine refused is said and **never recorded as having happened** | `sweeping::tests::a_refused_removal_is_said_and_never_recorded` |
| A record that will not take the entry is said loudly | `sweeping::tests::a_record_that_will_not_take_the_entry_is_said` |
| A kept turn whose file will not read is stepped over and left exactly where it is | `found::tests::a_turn_that_will_not_read_is_stepped_over_and_left_alone` |
| A person who never said where their settings are is stepped over | `found::tests::a_person_who_never_said_where_their_settings_are_is_stepped_over` |
| One broken directory does not stop anybody else's disk being tidied | `found::tests::one_broken_directory_does_not_stop_the_others` |
| A settings file that did not read uses the shipped window and says so | `sweeping::tests::a_settings_file_that_did_not_read_uses_the_shipped_window_and_says_so` |
| A person trying to switch expiry off by hand is refused whole | `keeping::tests::a_wrong_file_is_refused_whole_and_the_shipped_window_is_used` |
| A window of nothing in the file is refused | `keeping::tests::a_window_of_nothing_in_the_file_is_refused` |
| A file that did not read is not written over | `keeping::tests::a_file_that_did_not_read_is_kept_until_it_is_put_back` |
| A settings folder that is not absolute, a turn with nothing said about it, another `format` | `the_folder::tests::*` (three tests) |
| A `format` a later alo OS wrote is refused **first**, before any other key | `the_contract_describes_this_folder::another_format_is_refused_before_anything_else_is_judged` |
| `btrfs subvolume delete` without `CAP_SYS_ADMIN` | measured, above |
| An entry of turns that is empty is not written at all | `let_go::tests::a_turn_with_no_sentence_is_refused` (`alo-record`) |

### Contracts held to the code

| | |
|---|---|
| `docs/contracts/person-settings.md` | `the_contract_describes_this_file` (six tests), and the four other crates' copies still pass with *eleven* |
| `docs/contracts/kept-undo-folder.md` | `the_contract_describes_this_folder` (six tests) |
| `docs/contracts/record-file.md` | `alo-record`'s own suite; `format` stays `1` |

## Verification

Run from this checkout, through WSL Ubuntu 24.04, building in
`/root/alo-builds/this-machine` from the serialized source copy at
`/root/alo-trees/this-machine`, as `docs/autonomy/SHARED_MAIN.md` requires. All
green.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean |
| `cargo test -p alo-letting-go -p alo-record -p alo-recounting -p alo-updating -p alo-saying -p alo-collected -p alo-keeping-up` | all pass |
| `cargo test -p alo-sleeping -p alo-displays -p alo-leaving -p alo-notifying --test the_contract_describes_this_file` | all pass |
| `cargo test -p alo-citing -p alo-conforming` | all pass |
| `cargo test` in `tools/kernel-loop` | 152 pass |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-letting-go -p alo-record -p alo-recounting --no-deps` | clean |
| `cargo test -p alo-letting-go --test on_a_real_btrfs_machine -- --ignored` | 2 pass, output above |

**The workspace suite was deliberately not run here**, per this lane's standing
instruction: the supervisor runs it after the worker, and an hour of a worker's
window spent pre-empting it has killed finished tasks before.

**What is not claimed.** WSL is development evidence and never certified-hardware
acceptance. The `btrfs` measurement is this crate's code against a real `btrfs`
filesystem on a loop device — not a booted alo OS, not the certified machine,
and not a machine on which `alo-turn` has ever taken a bracket, because nothing
takes one yet.

## Limitations, and what is owed to whom

1. **Nothing takes a bracket yet.** `alo-turn`'s half of ADR 0045 is lane A's
   and is not built, so on a real machine today this unit walks a folder that is
   not there and says so. That is the honest state and it is what
   `Everyone::under` answers *nobody* for. This change is what makes the day
   `alo-turn` lands a day on which nothing fills quietly.
2. **Owed by the image lane**, as ADR 0053's own consequence puts it for the
   update units: installing `/usr/libexec/alo-letting-go`, the unit and the
   timer, and `systemctl enable alo-letting-go.timer`. The unit lives beside the
   crate until then, and its lines are held by a test either way. The image lane
   should also measure which of the three capabilities a booted machine really
   needs and narrow the list; what it carries now is `CAP_SYS_ADMIN` — measured —
   plus two derived from reading and removing files a person owns.
3. **The record window reads more than one file.** Point 8 above. That surface
   is the shell plan's, and has owed this since the broker gained a record.
4. **Point 5 of ADR 0045 — forgetting, as one act — is the last of the seven
   that nothing builds**, and is now task 14 of the plan. Its first line is a
   decision, not code: the act is the person's and the removal needs
   `CAP_SYS_ADMIN`, and neither road this repository has is open to it as it
   stands.
5. **No `docs/quirks.md` entry was added**, deliberately: nothing measured here
   disagreed with what task 11 of the installer plan already wrote down, and
   repeating a quirk in different words is how two readers end up with two
   answers.

## Proposed updates to the shared documents

For the integration owner; this report does not edit them.

**`CHANGELOG.md`**, under v0.5:

> **The machine lets go of what it can no longer put back.** alo OS keeps what
> your files were before the assistant changed them, so that you can put them
> back — and now it stops keeping them when you said it should. You choose how
> far back that reaches, in your own settings; what falls outside goes, once a
> day, on its own. If the disk is running short the oldest go first, because the
> machine will not fill a disk to hold on to something it could put back. Either
> way your machine's own record tells you which pieces of work can no longer be
> undone, named in the words you approved at the time rather than as a number.
> No assistant can ask for any of this, ever: something that could forget what
> you can undo could erase the evidence of what it did.

**`docs/autonomy/QUEUE.md`:** task 13 of the machine-keeps-itself plan is done;
task 14, *Forgetting what is kept, as one act a person asks for*, is ready and
depends on it. It begins with a decision about how a person's own act reaches a
privileged unit, and may finish as an ADR.

**`docs/autonomy/STATE.md`:** reference this report. Note that ADR 0045's terms
1 and 2 are now obeyed by a machine rather than decided by one, measured on a
real `btrfs` filesystem; that the amendment of 2026-09-21 carries a *Built*
line; and that `alo-turn`'s bracket and the image lane's installation remain
owed.

**`ROADMAP.md`:** nothing to tick. ★ *undo what the agent did* still waits on
the bracket, which is another lane's.
