# A turn without a boundary does not run

- **Date:** 2026-09-12
- **Workstream:** kernel enforcement (`alo-bounding`, `alo-agentd`, and the
  record it reaches through `alo-turn`, `alo-record` and `alo-recounting`)
- **Contributor:** Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- **Task:** **A turn whose boundary cannot be applied does not run** —
  `docs/autonomy/kernel-enforcement-plan.md`, task 15
- **Status:** ready for integration. The promise is asked of the machine
  before every turn, every way a running service loses its boundary was
  reproduced before it was closed, the refusal is recorded, and nothing is
  ticked *on the machine*.

## What changed, in one paragraph a person can read

alo OS promises that an agent's turn does not run at all if the machine cannot
put a boundary around it. Until today that was checked once, when the service
started. A machine could lose its boundary afterwards — a pin removed by hand,
or the loader run again — and the service would go on running turns, believing
each one bounded and writing it down as such, while the kernel enforced
nothing. Now the service asks the machine before every turn whether the
boundary is still there, refuses the turn before its first verb if it is not,
tells the person in one sentence that the machine and not they are at fault,
writes the refusal into the record as the machine's own, and puts the technical
reason on the service log for whoever looks after the machine. A machine whose
boundary is in place is unaffected. There is no setting that turns the refusal
into a warning, and there will not be one.

## What changed, for whoever reads the code

### The check, and the three questions it asks

`crates/alo-bounding/src/in_place.rs` is new: `Boundary::in_place`, called
first thing in `Turns::doing` (before a control group is made or an entry
written) and again by `Boundary::opened` at start. It asks the machine, never a
note the service kept:

| Question | Asked how | Refusal |
|---|---|---|
| Is the map of turns still pinned? | a `stat` of the pin | `NotBounded::NoBoundaryHere`, naming the loader |
| Is the programme still held on each of its twelve hooks? | a `stat` of each pin, by hook name, in attach order | `NotBounded::HookIsNotHeld { hook, path }` — **new** |
| Is the map at the pin the map this service holds? | `BPF_OBJ_GET` on the pin and `BPF_OBJ_GET_INFO_BY_FD` on both, compared by the kernel's map id | `NotBounded::NotTheSameBoundary { held, pinned }` — **new** |

Every refusal points at `docs/quirks.md` by section title. `Pinned` gained
`every_hook_named()`, which `every_hook()` is now cut from, so the list that
attaches, the list that refuses over leftovers, the list that takes a boundary
away and the list that is asked before every turn are one list. `Boundary` now
keeps the `Pinned` it was opened from.

**The third question is the one the task did not name and the one that
mattered most.** A loader run again since the service started — an operator
restarting `alo-boundaryd` by hand — takes every pin away and makes new ones.
The programme now on the hooks reads a map the service never opened, and what
the service writes into the map it holds is read by nothing. Every turn
afterwards is a thread in a control group the kernel holds no entry for, which
is a thread the kernel allows everything, on a machine whose pin listing shows
nothing wrong.

### What the daemon may see, and why it is not given more

The pins are `0600 root:root` in a `0750` directory the agent's group may
enter, so the service can `stat` a pin and cannot open one. That mode is not
loosened: a descriptor on a link is enough to detach it — `BPF_LINK_DETACH`
checks nothing about how the descriptor was opened — so a mode that let the
daemon read a pin would let the person's own service take the machine's
boundary off. What the daemon may ask about the programme is therefore whether
the pins that hold it are there, and about the map, which it may write, which
map it is.

**A probe was considered and refused.** The most literal check is to have the
turn's own thread open something outside its bound and see it refused. That
would be the service doing inside a turn, on purpose, the one thing the
boundary exists to refuse, and `alo-agentd`'s own test header already says why
the service must not be able to do that. The pins and the map's number are the
kernel's own state, and they are what is asked.

### The record: `not-bounded`, additive

`alo-turn`'s `carrying.rs` used to argue that a turn which could not be bounded
left no entry because nothing had happened. That was reversed: the machine
refused to run the agent's turn, which is a refusal, and a record that kept
every refusal but the machine's own showed a boundary that had gone as a record
that simply stopped. `CLAUDE.md`'s gate is *every execution and every refusal
leaves a record*.

- `alo_record::Happened::NotBounded { agent, why, machine }` and
  `Entry::not_bounded` — `why` is the sentence the person was shown, `machine`
  is what the machine said about its own boundary. No call, no verb, no
  approval, no grant: a file verb had a call and a question had none. Counts as
  stopped; stopped at no point in a call's journey. `docs/contracts/record-file.md`
  names the tag and says it is additive; `format` stays `1`.
- `alo-turn`: `carrying_out` returns `(Entry, Result<Answer, NotDone>)` with no
  `Err` at all — every road out carries an entry, which the module doc always
  claimed. The question path (`asking.rs`) writes the same entry, and a thread
  lost putting a question is now remembered on the turn as it was for a verb
  (`Turning::a_thread_was_lost`), which was an omission.
- `alo-recounting`: `Outcome::NotBounded` and the word
  `recounting.outcome.not-bounded`, so *what did the agent do* reads the
  machine's refusal back as the machine's.
- `alo-agentd`: `doing.rs` says the machine's account on the service log when a
  verb or a question is refused for want of a boundary, and answers the agent
  in the person's words as before.

### Decisions taken here, and why

- **Per-turn, in `Turns::doing`, not only at start.** The acceptance says
  *before its first verb*, and the state that matters changes after start.
  Putting it in the mechanism's one door means every caller gets it, including
  tests, and it sits beside the ordering property `inside.rs` already argues.
- **`stat`, not a functional probe** — above.
- **The map's identity is compared by the kernel's id, both read now.** Not
  an id remembered at open and compared against the pin: two of the kernel's
  own answers, so nothing the service remembers about itself is part of the
  check.
- **The machine's English goes into the record.** `alo_bounding::NotBounded`
  is already an administrator's type in English by an earlier decision; a
  review wants *which pin, at which moment*, and the person's sentence
  deliberately carries no fact about a kernel.
- **No repair by the service.** A service that re-opened a boundary while
  running would be deciding for itself which boundary it was under. The quirks
  entry says to restart `alo-agentd` after the loader is run again.

## Reproduced before it was closed

`crates/alo-bounding/tests/a_turn_without_a_boundary_does_not_run.rs` was
written in its final, refusing form and run against the crate with
`in_place` unwired from `Turns::doing`. Four of its eight tests failed, each
with the turn **running**:

| State | Before the check | Now |
|---|---|---|
| loader run again (map at the pin is not the one held) | ran; **key opened**, invoice opened | refused, `NotTheSameBoundary`, both map numbers named |
| a hook's pin removed (each of twelve, in reverse order) | ran; key refused by the hooks still held | refused, `HookIsNotHeld`, the hook named |
| the map's pin removed | ran | refused, `NoBoundaryHere` |
| the whole boundary taken away | ran | refused, `NoBoundaryHere` |
| boundary in place (control) | ran; key refused, invoice opened | unchanged |

Every refusal asserts: the work never started, no control group is left under
the service's subtree, the kernel holds no entry, the sentence names what is
missing and contains `docs/quirks.md`, and no thread is left inside.

**One measurement worth keeping.** The kernel releases a removed pin from a
work queue, so a turn run in the same breath as `rm` of `file_open`'s pin still
saw `EACCES` from the old programme — the *before* column's second row. What is
stable is the pin's absence, and that is what the service asks about.
`docs/quirks.md` records it.

## Verification

Platform: Ubuntu under WSL2, kernel `6.18.33.2-microsoft-standard-WSL2`, as
root, with `bpf` in `/sys/kernel/security/lsm` and a BPF filesystem at
`/sys/fs/bpf`; `CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-claude-bd192ccccbc3745b`.
Windows for `cargo fmt` and a second `clippy`.

Executed:

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean on Linux and on Windows.
- `cargo test -p alo-bounding` — see the counts below; the new file is 8
  tests, all against the real loaded BPF LSM.
- `cargo test -p alo-agentd` — including the new one-test file, record on a
  real disk.
- `cargo test -p alo-turn`, `cargo test -p alo-record`,
  `cargo test -p alo-recounting` — on Linux.
- `cargo doc --no-deps` for the five crates with `RUSTDOCFLAGS=-D warnings`.

Results, measured on 2026-09-12 by the task 16 worker on the same platform
and target directory, because this report was left with its results section
empty when the session that wrote it ended before writing a handoff (the
supervisor's log: *nobody handed over its work*). Every count is from
`cargo test -p <crate>` run on the tree as it stood, with this task's changes
in it and nothing else uncommitted but task 16's own files:

| Crate | Result |
|---|---|
| `alo-record` | 66 unit tests passed; 3 integration files, 7 passed; 0 failed |
| `alo-recounting` | 55 unit tests passed; 5 integration files, 22 passed; 0 failed |
| `alo-turn` | 70 unit tests passed; 5 integration files, 15 passed; 0 failed |
| `alo-bounding` | 36 unit tests passed; 17 integration files, 93 passed, 12 ignored; 0 failed — `a_turn_without_a_boundary_does_not_run.rs` 8 passed in 12.97 s against the loaded BPF LSM |
| `alo-agentd` | 245 unit tests passed; 6 integration files, 23 passed; 0 failed — `a_turn_is_refused_when_the_boundary_is_gone.rs` 1 passed, record on a real disk |

`cargo fmt --all -- --check` clean and `cargo clippy --all-targets -- -D
warnings` clean over the whole workspace on the same tree, on Linux.

Known and unrelated: on **Windows**, six tests in
`alo-recounting`'s `recounting::tests` fail on the unmodified tree too
(`NotRead { why: "unsupported" }` from the record reader), which was confirmed
by stashing this change and running them; they pass on Linux. Nothing here
touched them.

Not executed: the full workspace suite, by instruction; nothing on a certified
machine — every measurement is WSL2, and no *On the machine* box moves.

## Remaining limitations

- The check sees a pin's presence, not the link's state behind it. A link
  detached by root through `BPF_LINK_DETACH` with its pin left in place would
  pass the second question; only root can do that, and root is trusted by
  every other line of ADR 0018. Named in `in_place.rs`.
- A `file_ioctl` gap and an `mmap_file` gap remain as before; nothing here
  touches them.

## Proposed changes to the shared documents

- **CHANGELOG.md** — *A turn whose boundary has gone is refused before its
  first verb: the service asks the kernel before every turn whether its
  boundary is still there — the map pinned, every hook held, the map the one
  it opened — and refuses, records the refusal as the machine's own, and tells
  the person the machine is at fault. A loader run again under a running
  service, which left every turn unbounded, is refused by name. No override
  exists.*
- **ROADMAP.md** — no box moves; the promise is v0.5 and stays marked as
  implemented-early with hardware acceptance pending.
- **docs/autonomy/QUEUE.md / STATE.md** — task 15 of the kernel-enforcement
  plan is done; the plan file carries the evidence rows.
