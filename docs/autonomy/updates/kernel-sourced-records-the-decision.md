# Kernel-sourced enforcement records — the decision

- **Date:** 2026-09-12
- **Workstream:** kernel enforcement (`alo-bounding`, `alo-bounding-kernel`,
  `alo-boundaryd`, and the record they reach through `alo-record`)
- **Contributor:** Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- **Task:** **Kernel-sourced enforcement records — the decision** —
  `docs/autonomy/kernel-enforcement-plan.md`, task 16
- **Status:** ready for integration. The decision is written and proposed, the
  plan points at it, a test holds the programme to two maps until the owner
  answers, and no code beyond that test was written. **This change also
  carries task 15's finished work**, which was in the tree with its report
  and no handoff; see *What else is in this change* below.

## What changed, in one paragraph a person can read

alo OS promises two things about the record of what an agent did that could
not both be built as written: that the record becomes what the kernel watched
the agent touch, and that the kernel programme which could watch everything
has nowhere to write anything down. Nobody had decided which bends, so nobody
could build either. This change writes the decision out for the owner —
[ADR 0029](../../decisions/0029-what-the-kernel-writes-down-about-a-turn.md) —
with four ways of doing it, what each costs a person reading *what did the
agent do*, what each costs the record file and the loader, and one
recommendation: the kernel keeps a small table of *which files and addresses*
each turn touched, as numbers rather than names, and the daemon checks its own
account against that table at the end of every turn and writes one extra line
saying whether the kernel agrees. Nothing is built yet. A test makes sure the
programme stays exactly as it is until the decision's status changes.

## What changed, for whoever reads the code

### The decision

`docs/decisions/0029-what-the-kernel-writes-down-about-a-turn.md`, status
**proposed**, in the shape ADR 0024 and ADR 0025 used. Its sections:

- *What is true today, verified rather than remembered* — read off the tree:
  the record is the daemon's account written outside the boundary; the
  kernel's one contribution is `EACCES`, written down as one sentence; the
  programme has two maps and nowhere to write; the loader takes no input; the
  record file is additive and synced per entry; and the three v0.5 promises
  quoted exactly.
- *The contradiction, stated exactly* — the two promises agree about what is
  watched and disagree only about where the discipline lives: *cannot write*
  versus *does not write*. And the kernel cannot write the record at all,
  because ADR 0001 §7's four answers are things it does not know.
- *What may not be done, whichever option is taken* — nothing outside a turn
  is written anywhere; what was asked is never kept; the record changes
  additively; no capability for the daemon, no input for the loader; no quiet
  narrowing; a lost observation is written down as lost.
- *The options* — A (the kernel emits a stream, the daemon appends), B (the
  daemon's account, with the kernel's refusal counts in the turn's own map
  entry), C (a table of identity counts the daemon folds against its account;
  the account shown as a claim), D (the rewording, named as a narrowing).
  Each names what it costs a person reading, the record file and the loader.
- *The recommendation* — **C**, built in four steps with the refusal path
  first, and why it is not a narrowing.
- Consequences if accepted, consequences if rejected for B, what it does not
  decide, and what the code waits on.

### Decisions taken here rather than handed back

- **The plan asked for three shapes; the ADR has four.** The three the task
  named are A, B and C. D — leave the record as it is and reword the promise —
  is listed because it is the honest name for what has happened since ADR
  0015 was accepted, and an ADR that omitted the do-nothing option would be
  presenting the owner with a false choice. It is marked as a narrowing and
  as the owner's alone.
- **A table, not a ring buffer, for the recommended shape.** A ring buffer
  is the obvious mechanism and the ADR says why it is wrong here: reading a
  ring drains it, one daemon runs per signed-in person, and ADR 0018 forbids
  telling the loader who to make a ring for. A hash map keyed by cgroup id is
  read rather than drained, so every daemon takes only its own rows, the same
  way the map of turns is shared today.
- **Identity, never a name.** The recommended table holds `(cgroup, hook,
  device, inode)` and never a path. That keeps `bpf_d_path` — whose
  availability on our twelve hooks nobody has measured — out of the design,
  and keeps the strongest reading of ADR 0015's warning: the mechanism that
  could watch everything is never handed a way to write down names.
- **The price is named in one sentence** rather than spread across the
  options: after any observing option, *decides and forgets* is held by a test
  rather than by the absence of a place to write. The owner is asked to
  accept that one downgrade, or take B to keep the structure.
- **Where the test lives.** In `crates/alo-bounding/tests`, beside
  `what_a_turn_inherits_is_written_down.rs` and
  `the_unwatched_mutations_are_written_down.rs`, which already read this
  repository's own documents and need no kernel. `alo-citing` already checks
  that every citation of ADR 0029 resolves; what this test adds is the
  task's own acceptance and the refusal path an ADR has.
- **The next task in the plan** is task 17, *A file's inode flags are inside
  the grant* — the `file_ioctl` hook for `FS_IOC_SETFLAGS` and
  `FS_IOC_FSSETXATTR` only, reversing the reproduction task 14 left standing.
  Chosen because it is the last named filesystem gap a hook closes, it is
  small, it is already reproduced, and it does not touch the two-map
  constraint ADR 0029 leaves in force. The `mmap_file` gap was not chosen:
  it cannot be reproduced without `unsafe` outside the kernel package's one
  file, and a task whose reproduction is forbidden is a task that starts by
  arguing for an exemption.

### The test

`crates/alo-bounding/tests/the_records_source_is_decided_before_it_is_built.rs`,
six tests, no kernel needed:

| Test | Holds |
|---|---|
| `the_decision_exists_once_under_its_number` | exactly one file under `docs/decisions/` carries `0029-`, and it is the one the plan cites |
| `the_decision_stands_as_proposed_or_accepted` | the status line says *proposed* or *accepted*; *withdrawn*, *superseded* or *rejected* means the plan's task 16 has to move |
| `the_plan_points_at_the_decision` | task 16 of the plan names the ADR by filename |
| `the_decision_sets_out_the_options_their_costs_and_a_recommendation` | the four option headings and the recommendation are there, and every option's text names what it costs a person, the record file and the loader |
| `the_code_waits_on_the_decision` | **while the status is *proposed*, `kernel.rs` declares exactly two maps** — the refusal path of an ADR: code that ran ahead of its decision |
| `the_checks_would_notice_the_decision_going_missing_or_the_code_running_ahead` | each check above is handed the thing it exists to catch — two files with one number, none, a renamed one, a withdrawn status, a status line naming nothing, a plan that stopped pointing, a missing option, an option missing a cost, a third map beside *proposed* — and refuses it; and a third map beside *accepted* is **not** refused, so the test cannot forbid building the decision once it is taken |

### The plan

`docs/autonomy/kernel-enforcement-plan.md`: task 16 marked done with what was
decided and why; task 17 written; the section 3 row and the section 5 item for
*kernel-sourced enforcement records* now point at ADR 0029 and say what it
recommends.

## What else is in this change

**Task 15, *A turn whose boundary cannot be applied does not run*, is
published in the same commit.** Its worker finished the code, the tests and
the report (`docs/autonomy/updates/a-turn-without-a-boundary-does-not-run.md`)
and marked the task done in the plan, but its session ended before it wrote
a handoff; the supervisor waited an hour, recorded *nobody handed over its
work*, stepped over it and launched this task with all of task 15's work
still in the working tree. The loop stages exactly the files a handoff names
and refuses a tree with anything else changed in it, so this task could not
be handed over without either carrying task 15's files or moving another
worker's finished work out of the tree. Carrying it was the right call, on
three grounds, and the commit body says so too:

- **It gates.** Before anything else was written, the whole tree was put
  through `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D
  warnings` over the workspace, and `cargo test -p` for the five crates task
  15 touched, on the real loaded BPF LSM. Every test passed; the counts are in
  its report's results section, which was empty (`RESULTS_PLACEHOLDER`) and
  which this worker filled in with what was measured, saying so in the
  section. Nothing else in that report was changed.
- **It is not unrelated.** Task 15 added the `not-bounded` record kind — the
  first entry whose subject is the boundary rather than a verb — and ADR
  0029 is about what else the kernel's side of the record should say. The
  ADR cites it.
- **Its evidence is in this handoff's evidence block**, one line per
  acceptance criterion of task 15 beside this task's, so the supervisor runs
  each of its tests on its own before publishing, exactly as it would have
  for its own handoff.

## Verification

Platform: Ubuntu under WSL2, kernel `6.18.33.2-microsoft-standard-WSL2`, as
root, with `bpf` in `/sys/kernel/security/lsm` and a BPF filesystem at
`/sys/fs/bpf`; `CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-claude-bd192ccccbc3745b`,
the loop's own build directory for this checkout.

Executed, on the tree as handed over:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean over the workspace.
- `cargo test -p alo-bounding` — including the new file, 6 tests, and task
  15's `a_turn_without_a_boundary_does_not_run.rs`, 8 tests against the
  loaded BPF LSM.
- `cargo test -p alo-citing` — every citation of ADR 0029 in the ADR, the
  plan, the test and this report resolves, and the status line reads.
- `cargo test -p alo-record`, `cargo test -p alo-recounting`,
  `cargo test -p alo-turn`, `cargo test -p alo-agentd` — task 15's crates.

Results, measured 2026-09-13 on the tree as it stood, because this report's own
results section was left empty when the session that wrote it ended before it
could write a handoff — the same way task 15's was, one task earlier:

| Crate | Result |
|---|---|
| `alo-bounding` | 135 passed, 0 failed |
| `alo-record` | 73 passed, 0 failed |
| `alo-recounting` | 77 passed, 0 failed |
| `alo-turn` | 85 passed, 0 failed |
| `alo-agentd` | 268 passed, 0 failed |
| `alo-citing` | 31 passed, 0 failed |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean over the workspace |

**On Linux, and the platform is not a detail here.** The same crates run on
Windows report six failures in `alo-recounting` and build no tests at all for
`alo-agentd`: the record's paths and the daemon's session are Linux's, the
gates run under WSL, and Linux is the platform alo OS is. A Windows count would
have read as a broken change and was not one.

Not executed: the full workspace suite, by instruction; nothing on a
certified machine — every measurement is WSL2, and no *On the machine* box
moves.

## Remaining limitations

- The decision is proposed, not taken. Nothing about the record's source
  changes until the owner or the delegate changes the status line, and the
  test will refuse a third map until then.
- The recommended option owes two measurements the implementation must take
  before claiming it: the size of the table a working day needs, and the
  cost of the sweep at daemon start. Neither is claimed here.

## Proposed changes to the shared documents

- **CHANGELOG.md** — *The question of whose sentence the record is — the
  daemon's, the kernel's or both — is written out as ADR 0029 for the owner,
  with four options, their costs and a recommendation; the kernel programme
  is held to its two maps by a test until it is answered.* And for task 15,
  its report's proposed line.
- **ROADMAP.md** — no box moves.
- **docs/autonomy/QUEUE.md / STATE.md** — tasks 15 and 16 of the
  kernel-enforcement plan are done; ADR 0029 joins ADR 0021 under *decisions
  awaiting an answer*; task 17 is next.
