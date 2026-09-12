# A done mark on the status line is read

**Date:** 2026-09-12. **Workstream:** v0.5 lane B, task 1 — *An address that
is not https is refused, unless it is a service on this machine* — and the
supervisor that drives it, `tools/kernel-loop`. **Contributor:** the lane B
worker, in `C:\dev\alo-os-b`.

## What this is, and what it is not

Task 1's code was finished and published before this worker was launched:
commit `1d59afe`, report
[`an-address-that-is-not-https-is-refused-at-the-write.md`](an-address-that-is-not-https-is-refused-at-the-write.md),
seven pieces of evidence held up by the supervisor on both the task's tree and
the combined tree. **Nothing in `crates/alo-choosing` is touched here.** This
worker was sent at task 1 because the loop, having published it, selected it
again in the same second (`.kernel-loop/loop.log`, `1789183718`): the plan's
done mark was written where the loop could not read it.

The plan's own instructions say every task has a `**Status:**` line and a
finished one is marked `**Done, <date>.**`, so the previous worker wrote
`**Status:** **Done, 2026-09-12.** Report: …`. `tools/kernel-loop/src/plan.rs`
read a task as done only when a line *began* with `**Done,`, and that line
began with `**Status:**`. Left alone, every iteration of the loop would send a
worker at task 1 forever; the worker instructions name this exact failure and
it happened on the very first task of the plan.

## What changed

- `tools/kernel-loop/src/plan.rs` — `marked_done` reads the mark at the start
  of a line, **or directly after the `**Status:**` label**, and nowhere else.
  Not anywhere in a line: a task's prose may quote `**Done, <date>.**` when it
  says how the plan is read, and a reader that took the quotation for the mark
  would finish a task by describing the finishing. The two labels are named
  constants; the blocked check uses the same `**Status:**` constant. The
  module's rustdoc records the failure and why the status line counts.
- `docs/autonomy/v0-5-lane-b-plan.md` — task 1's mark now stands at the start
  of its own line, the canonical form every other plan uses, with a note that
  it first sat after the label and a link here. The reading instructions say
  "at the start of a line of its own" and that the loop also reads it after
  the label, so the next worker does not have to know the parser to be read.

**User-readable change description:** the build loop now recognises a task
marked done on its status line, so a finished task is not handed to another
worker; lane B's plan is corrected to mark its first task the ordinary way.

## Decisions

- **Fix the reader as well as the plan.** Correcting the plan alone would end
  this loop, and the same instruction would lead the next worker to the same
  line. The reader accepting the one place the plan invites the mark is
  narrower than a free-text search and costs no false positive, which the
  second test pins.
- **Hand this over under task 1's name.** The loop selected task 1 and refuses
  a handoff naming a different task. The report says plainly that the task's
  code was already on `main` and that this change is the plan's bookkeeping
  and the loop's reading of it, not the task's second implementation.
- **No change to `crates/alo-choosing`.** Its seven acceptance tests are the
  previous report's evidence and were held up twice by the supervisor. Naming
  them again here would be refused — their file is not in this change — and
  rightly: the state of the repository is not proof of what this change wrote.

## Acceptance and evidence

| Criterion | Test |
|---|---|
| A `**Done,` mark after `**Status:**` finishes the task, and the next task is chosen | `plan::tests::a_done_mark_on_the_status_line_finishes_the_task` |
| Prose quoting the mark, above the tasks or inside one, finishes nothing; a mark at line start or after the label does | `plan::tests::a_line_that_quotes_the_done_mark_does_not_finish_a_task` |

Both live in `tools/kernel-loop/src/plan.rs` (workspace `tools/kernel-loop`,
crate `alo-kernel-loop`, target `bin`).

## Verification

Executed in WSL Ubuntu against `/mnt/c/dev/alo-os-b/tools/kernel-loop`, with
its own `CARGO_TARGET_DIR`, on 2026-09-12:

| Check | Result |
|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test -p alo-kernel-loop` | 93 passed, 0 failed |
| each evidence test alone, `--bins -- --exact <name>` | 1 passed, 92 filtered out, each |

Not run: the product workspace's full suite (the supervisor runs it; nothing
in the product workspace changed). No hardware acceptance applies.

## Limitations and proposed shared-document updates

- The reader still does not accept a mark placed anywhere else, by design.
- Proposed `CHANGELOG.md` line, under the build loop: "A task marked done on
  its status line is read as done; lane B's plan corrected."
- No `ROADMAP.md`, `QUEUE.md` or `STATE.md` change beyond referencing this
  report; task 1 of the lane B plan remains done as of `1d59afe`, and task 2,
  *Test a provider before saving it*, is next.

**Status:** ready for integration.
