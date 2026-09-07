# Kernel enforcement — plan and supervisor

- Date: 2026-09-07
- Workstream: kernel enforcement (`alo-bounding-kernel`, `alo-bounding`, `alo-boundaryd`)
- Contributor: Claude Code, kernel-enforcement workstream
- Status: ready for integration

## What changed and why

Two things, and neither is enforcement: an **audit-based plan** for what remains
of this workstream, and a **supervisor** that gates and publishes its tasks.

`docs/autonomy/kernel-enforcement-plan.md` is the plan. It is written from the
code and the tests rather than from the queue, and every line in its *implemented
and verified* table names the test that says so.

`tools/kernel-loop/` is the supervisor, a standalone Rust crate with its own
workspace beside `tools/dev-loop`. It is a second program rather than a change
to the compositor worker's one: two workstreams sharing a supervisor would share
its lock, and that supervisor has to keep running whatever happens here.

### The finding that matters most

**Almost all of this work belongs to the next release, and the plan says so
rather than letting a finished task list read as a finished release.**

`docs/features.md` is the only list of what was promised, and it marks the
kernel-boundary promises `[v0.5]`:

- `[v0.5] ★ The grant is a boundary the kernel imposes, not a rule the daemon follows (ADR 0013)`
- `[v0.5] ★ And the kernel is taught what a turn is (ADR 0015)`
- `[v0.5] So the record stops being anybody's account of itself.`

Both ADRs carry **"Status: accepted — direction, ordered behind `alo-agentd` and
the turn"**. They are accepted decisions about *how*, written early so the turn
was not built assuming ambient authority — not v0.01 delivery commitments.

**One kernel-security requirement sits inside a v0.01 line**, in the *On the
machine* half of *Egress indicator, and no telemetry*: "the enforcement at the
network boundary, without which all of this describes only the code that asked".
So network egress enforcement and attribution is the in-scope item, and the four
filesystem hooks already shipped are v0.5 work delivered early.

### The second finding: one item needs a decision, not code

ADR 0015 promises the record becomes *what the kernel watched happen*. Its own
discipline — **the LSM decides and forgets**, with a test that fails if a third
map appears — forbids the mechanism that would produce it. Both cannot be
delivered as written. That is a decision about how much a security module may
remember, it is the most dangerous question in this repository, and this
workstream will not build a reporting path without an ADR. Recorded in the plan;
not scheduled.

## The audit, in short

**Implemented and verified**, each against the real loaded BPF LSM: the four
hooks (`file_open`, `inode_rename`, `inode_unlink`, `inode_link`), authority
ending with the turn, data preserved on refusal, non-turn processes unaffected,
the LSM writing nothing down, and the loader's attach, refusal-over-leftovers and
take-away paths.

**Not built**: network egress enforcement (v0.01, in scope); `inode_create`,
`mkdir`, `rmdir`, `symlink`, `setattr`, `setxattr` (v0.5); descriptors inherited
into a turn (v0.5, not addressed anywhere); Landlock, seccomp and namespaces —
ADR 0013's other three primitives, none of which exists (v0.5); snapshot and undo
(v0.5/v1).

**Hardware-dependent**: everything. Every measurement is from WSL2, which
`docs/hardware.md` says cannot certify a machine.

## The supervisor, and what it deliberately does not do

**It does not write code.** A supervisor that claimed to would be a placeholder
with a loop around it. It verifies and publishes: pull, gate, commit exactly the
named files, integrate what arrived on `main`, **re-gate the combined tree**,
push with bounded retries, and log.

Its guards, each verified by running it:

| Guard | Behaviour |
|---|---|
| single instance | a lock made with `create_new`, so two loops starting together cannot both be told they are alone |
| clean checkout on `main` | refuses, naming any changed file no task accounted for |
| the plan knows this task | refuses, listing the tasks the plan names — and **only numbered task headings count**, so an audit section heading cannot be published as a task |
| a complete handoff | refuses a missing subject, and a report absent from the files being staged |
| bounded retries | three integrate-and-retry attempts on a lost push race, then it stops and says the commit is sitting locally |
| graceful stop | `stop` writes a flag; the loop finishes what it is doing and begins nothing else |

**What is not in `repository.rs` is the point**: no `reset`, no `checkout --`, no
`clean`, no `push --force`, no `rebase --abort`. A conflicted rebase is left
exactly where git stopped. Nothing here can discard work.

Commands, from the checkout root:

```
tools/kernel-loop/target/release/alo-kernel-loop run
tools/kernel-loop/target/release/alo-kernel-loop status
tools/kernel-loop/target/release/alo-kernel-loop stop
```

## Verification

Platform: Ubuntu on WSL2, kernel 6.18.33.2, stable Rust 1.98.0, own
`CARGO_TARGET_DIR`.

The supervisor's own gates: `cargo fmt --all --check` clean, `cargo clippy
--all-targets -- -D warnings` zero warnings, release build clean. It forbids
`unsafe`, and denies `unwrap`, `expect`, `panic`, `todo`, `unimplemented` and
`indexing_slicing`, as the workspace crates do.

**The loop was exercised before being relied on**, each case run against the real
checkout:

- `status` with no run — reports not running.
- `run` with nothing handed over — "nothing is handed over to publish", exit 0.
- a handoff naming a task the plan does not have — refused, listing the plan's
  five tasks.
- a handoff naming an audit *section* heading — refused. This found a real bug:
  the first version accepted any `###` heading, so *Implemented and verified*
  would have been publishable as a task. Fixed to require a numbered heading.
- a handoff with no subject — refused.
- a handoff whose report is not among its files — refused.
- a lock already held — refused, naming the process.
- `stop` then `status` — the flag is set and reported.

No task was published during verification, because every handoff used was
invalid on purpose.

The workspace gates were not re-run by this change: it adds a standalone tool
outside the workspace, a plan, a report and two `.gitignore` lines, and touches
no crate the workspace builds.

**Windows**: not run, and not applicable to this change — the supervisor drives
the Linux gates, and `alo-bounding` compiles to nothing on Windows. Reports from
this workstream will keep saying so rather than implying a check that did not
happen.

**WSL is development evidence and never certified-hardware acceptance.** No *On
the machine* box is affected by this change.

## Remaining gaps and hardware obligations

- Everything in the plan's *Incomplete* table, with the release that owns each.
- The ADR-0015 reporting tension, which needs a decision before any code.
- A failure path that cannot be induced: a genuine mid-attach kernel refusal on
  a healthy machine. Named in the plan rather than claimed as tested.
- All physical acceptance.

## Proposed shared-document updates

**CHANGELOG.md** — nothing. This is contributor tooling and a plan; no
user-visible behaviour changed.

**ROADMAP.md** — no box changes. Worth knowing when the v0.01 egress line is next
read: its remaining *On the machine* clause names network-boundary enforcement,
and this workstream has it as the in-scope item with a written plan.

**docs/autonomy/QUEUE.md** — no new item; the plan is this workstream's own list
and lives at `docs/autonomy/kernel-enforcement-plan.md`.

**docs/autonomy/STATE.md** — reference this report and the plan. The fact worth
carrying: the kernel-boundary promises are v0.5 in `docs/features.md`, the four
filesystem hooks already shipped are that work delivered early, and the one
v0.01 kernel-security requirement outstanding is enforcement at the network
boundary.
