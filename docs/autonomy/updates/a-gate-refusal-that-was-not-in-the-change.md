# A gate refusal that was not in the change

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 26 — second and last worker
**Contributor:** Claude (`C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this is

Task 26 — *The one privileged thing that turns a correct password into a
session* — was implemented in full by the worker before me and refused by the
supervisor's `the workspace's tests` gate, twice, with one compile error:

```text
error[E0599]: no method named `numbers` found for struct `Accounts`
   --> crates/alo-sessiond/src/machine.rs:177:41
177 |             Ok(accounts) => Ok(accounts.numbers(person)),
```

**The method was in the tree, and it still is.** `alo_accounts::Accounts::numbers`
is at `crates/alo-accounts/src/store.rs:152`, written at 13:31 and untouched
since; the gate ran at 14:01. Not one line of the component was wrong, and this
report exists because *refused twice* was read as *the work rather than the
machine* and it was not.

I wrote no product code for this task. Their work is the work, it is described in
`docs/autonomy/updates/the-one-privileged-thing-that-opens-a-session.md`, and
that report stands as written. What follows is what actually refused it, how that
was established rather than guessed, and the one paragraph of repository
documentation it earned.

## What refused it

Every check a worker is asked to run passed on the unchanged tree, in the gates'
own environment (WSL Ubuntu, `CARGO_TARGET_DIR=$HOME/target-claude`, the same
toolchain):

| Command | Result |
|---|---|
| `cargo check -p alo-sessiond --all-targets` | clean |
| `cargo check --workspace --all-targets` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo build -p alo-sessiond` | clean |
| `cargo test -p alo-accounts`, `-p alo-sessiond` | green |
| **`cargo build --workspace`** | **the E0599 above** |
| **`cargo test --workspace --no-run`** | **the E0599 above** |

So it was reproducible, and it was not a flake and not the source: the same
files compile for `-p` and for `--check`, and fail for the whole-workspace
**build**. `cargo build --workspace -v` names the culprit — `alo-sessiond` is
handed `--extern alo_accounts=…/libalo_accounts-b3abda5bbb78fe01.rmeta`, and no
`rustc` invocation for `alo-accounts` appears in that build at all, because
Cargo held that unit to be fresh. It was a **cached `alo-accounts` from before
`numbers` existed.**

```text
cargo clean -p alo-accounts -p alo-sessiond
cargo build --workspace          # Finished in 18.25s, exit 0
cargo test  --workspace --no-run # exit 0
```

That is the whole repair, and it is a repair of an artefact rather than of a
change.

**Why the fingerprint stayed fresh across the source edit, I did not establish,
and I am not going to claim it.** Two things are worth writing down about it and
both are observations rather than a mechanism. A whole-workspace build resolves
features differently from `cargo build -p <crate>`, so the two produce *different
units* of the same crate — the `-p` unit was rebuilt correctly and the workspace
unit was the stale one, which is exactly why a worker's own per-crate gates
cannot see this and the supervisor's whole-workspace gate can. And the checkout
is on `/mnt/c`, where every source mtime comes across drvfs from Windows, which
is the sort of ground on which Cargo's freshness test is known to be delicate. A
sharper diagnosis than *it was stale* would need the fingerprint directory as it
was at 14:01, and cleaning it is what made the gate pass.

## The decision I had to make, and why

The brief says to prefer the smallest change that makes the gate true and, if the
fix looks like deleting a test, to think again. Here the smallest change that
makes the gate true is **no source change at all** — and a handoff whose only
content is "I cleaned a build directory" is one the next person cannot learn
anything from, because the target directory is not in the repository and the
sentence would be gone with it.

So the change I made is the finding, put where it will be read:

- **`tools/kernel-loop/src/gates.rs`** — a paragraph in the module that *prints
  the refusal*, saying that a second run does nothing about a stale artefact, so
  *refused twice* is not evidence that a compile error is in the change, and
  naming the check that tells the two apart: `cargo clean -p` the crate that owns
  the missing item, then the whole-workspace **build**. It is documentation only.
  **No gate is weakened, retried, skipped or made conditional**, and I did not
  touch the gate list: a gate is right to refuse a tree it cannot build, whoever
  broke it.
- **`docs/autonomy/v0-01-delivery-plan.md`** — task 26's entry, already marked
  **Done, 2026-09-11** by the first worker with task 27 written after it, now
  also records that it was refused once for something that was not in it. A
  reader of the plan who finds the parked branch or the refused handoff should
  not have to re-derive this.

I considered `docs/quirks.md` and decided against it. That file says in its own
first lines what it is for — hardware and firmware, applications driven through
their own automation, and pinned upstream engines — and the build tool the loop
runs is none of the three. Putting a Cargo freshness note there would widen a
document with a stated charter in order to avoid choosing a home for one entry.

I also decided against making the supervisor immune to it. The only reliable
guard is a clean build directory per iteration, which costs the better part of an
hour of rebuilding on every task and would be a real cost paid for a rare fault;
and a gate that cleaned and retried on failure would be a gate that hides the
next genuine build break behind an hour. That is a judgement about a trade-off, so
it is written here rather than acted on quietly.

## What I checked of the work itself

I am the last worker on this task, so I read the component rather than only its
error. `src/opening.rs` is the whole decision and it decides in the order its
module documentation claims — caller's group first, so a caller that may not ask
learns nothing about the number it asked for; the accounts file second, before
`logind` is spoken to at all; the already-signed-in check third. `src/door.rs`
compares a group and nothing else, root's group is refused at start-up, and both
halves of that are tested. `src/unix.rs` is the only file naming `rustix`, asks
three questions, and takes a `&UnixStream` rather than a listener so
`SO_PEERCRED` cannot be asked of something that would answer about nothing.
`src/listening.rs` gives both directions a timeout, removes a socket it bound but
could not hand over, and creates no directory. Every test named in the handoff's
evidence exists, in the file the handoff names, and none of them is a test of
something absent. I changed none of it, and I am not reporting on it as though
it were mine — the point of saying this is that the refusal was investigated
rather than assumed away.

## What is still owed

Everything the first worker listed under *What is still owed, plainly* is still
owed, unchanged and not restated here. Nothing in this report reduces it: no
machine has opened a session with `alo-sessiond`, the image has not been rebuilt,
and the privileged half of ADR 0024's measurement remains a by-hand call on the
pinned base.

Owed by this report and new: **why Cargo held a stale unit fresh is unexplained.**
If a gate refuses a worker again for an item that exists, that is the same fault
and the paragraph in `gates.rs` is the two-minute check, not the answer.

## Verification

Run from `C:\dev\alo-os-claude`; the Linux column is WSL Ubuntu with
`CARGO_TARGET_DIR=$HOME/target-claude`, which is the environment the supervisor's
gates run in.

| Command | Where | Result |
|---|---|---|
| `cargo fmt --all` | Windows | clean, changed nothing |
| `cargo clippy --all-targets -- -D warnings` | Windows | exit 0, whole workspace |
| `cargo fmt --all --check` | Linux | clean |
| `cargo clippy --all-targets -- -D warnings` | Linux | exit 0, whole workspace |
| `cargo doc --workspace --no-deps`, `RUSTDOCFLAGS=-D warnings` | Linux | exit 0 |
| `cargo test -p alo-sessiond` | Linux | 45 + 2 + 1 + 4 passed, 0 failed |
| `cargo test -p alo-accounts` | Linux | 48 + 3 + 4 passed, 0 failed |
| `cargo test -p alo-image` | Linux | 118 + 18 passed, 0 failed |
| `cargo test -p alo-saying` | Linux | 63 + 4 + 1 passed, 0 failed |
| `cargo test -p alo-collected` | Linux | 8 + 11 passed, 0 failed |
| `cargo test -p alo-citing` | Linux | 21 + 10 passed, 0 failed |
| `cargo test -p alo-bounding --test the_unwatched_mutations_are_written_down` | Linux | 4 passed — the one test that *parses* `docs/quirks.md`, which this change adds an entry to |
| `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` in `tools/kernel-loop` | Windows and Linux | clean, exit 0 |
| `cargo test` in `tools/kernel-loop` | Windows | 51 passed, 0 failed |
| each of the eleven evidence tests, alone, `-- --exact --include-ignored` | Linux | 1 passed each, eleven times |
| `cargo test --workspace --no-run` | Linux | **exit 0** — the gate's own compile, which is what failed |

The whole-workspace **suite** was not run here, per the standing instruction; the
whole-workspace **compile** was, because that is precisely the thing that
refused this task and a handoff that had not re-run it would be a guess.
Everything in the Linux column ran in WSL Ubuntu with
`CARGO_TARGET_DIR=$HOME/target-claude`, which is the directory the gates use and
the directory the stale artefact was in.

## Proposed updates to the documents this task does not own

**`CHANGELOG.md`** — no separate entry. This report adds no product behaviour;
task 26's own entry, proposed in
`docs/autonomy/updates/the-one-privileged-thing-that-opens-a-session.md`, is the
one a reader outside this repository wants and it is unchanged.

**`docs/autonomy/STATE.md`** — worth one line, because it is about the loop
rather than the product: task 26 was refused once by `the workspace's tests` for
a cached artefact rather than for its change, cleared by `cargo clean -p
alo-accounts`, and the lesson is recorded in `tools/kernel-loop/src/gates.rs`
and in this report. A refusal that was not the worker's is the kind of thing the
journal exists to remember.

**`docs/autonomy/QUEUE.md`** — nothing. Task 26 is done and task 27 is written
into the delivery plan by the first worker.
