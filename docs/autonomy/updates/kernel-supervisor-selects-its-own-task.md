# Kernel supervisor selects its own task

- Date: 2026-09-07
- Workstream: kernel enforcement (`tools/kernel-loop`)
- Contributor: Claude Code, kernel-enforcement workstream
- Status: ready for integration

## What changed and why

The supervisor published whatever handoff it was given. It now **reads the plan
and chooses**: each iteration takes the next task that is not done and whose
dependencies are, says which one it is, waits for that task's work, gates it,
publishes it, and goes round for the next. A handoff naming a *different* task
is refused, because a supervisor that took whatever it was handed would be one
whose plan is decoration.

A task is a numbered `###` heading in
`docs/autonomy/kernel-enforcement-plan.md`; it is **done** when its section
contains a `**Done,` line; it **depends on** the numbers on its
`**Depends on:**` line. Nothing here writes to the plan — *done* is a judgement
about evidence and this program cannot make one, so a person marks it, in the
change that earned it.

### What it still does not do, said plainly

**It does not write the code.** That step cannot be automated by this program,
and a supervisor that claimed to would have a placeholder at the centre of the
thing that publishes. What the loop removes is everything around it: choosing
what is next, the pull, the gates, the rebase, the re-gate on the combined tree,
the bounded retry, and the log.

So a run is *select → wait → gate → publish → select…*, ending when the plan has
no executable task left, nobody produces the work for the one selected, or
somebody stops it. The waiting is bounded at an hour; a loop that waited forever
would sit on a lock for a reason nobody could see.

**An empty plan is not completion.** When the list runs out the loop says so in
those words — *a list being empty rather than a workstream being finished* —
because the plan's own last section says the difference and an exhausted queue
is exactly how a release gets called done.

## Acceptance criteria and actual verification

Acceptance: the loop chooses the plan's next task and refuses work for any
other.

Executed on Windows, where the loop runs:

- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
  release build — all clean.
- Given a handoff for *Documenting the filesystem mutations that remain
  unwatched* while the plan's next task was *Socket attribution and default-deny
  for a bound turn*, it selected the latter, named both in its refusal, and
  published nothing.
- The dependency reading was exercised by the plan itself: tasks 1 and 2 are
  marked done, so task 3 — which depends on both — became the next executable
  one, and the loop chose it rather than task 4, which is unblocked but later.

Not re-run: the workspace gates. This change touches only a standalone tool
outside the workspace and a report.

**Windows:** this is the platform for this change. **WSL is development evidence
and never certified-hardware acceptance.**

## Decisions and approvals

No approval needed; contributor tooling, no shipped crate touched.

The decision recorded: **the loop does not mark tasks done.** It reads the plan
and never writes it. A supervisor that could tick its own list is a supervisor
whose list means nothing.

## Remaining gaps and hardware obligations

- The implementation step is a person's, and is not automated.
- A failure path still not induced: a genuine mid-attach kernel refusal.
- All physical acceptance.

## Proposed shared-document updates

**CHANGELOG.md**, **ROADMAP.md**, **QUEUE.md** — nothing; contributor tooling.

**docs/autonomy/STATE.md** — reference this report beside the plan.
