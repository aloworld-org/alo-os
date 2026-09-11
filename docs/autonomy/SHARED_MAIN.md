# Working together on main

The owner chose direct publication to main on 2026-09-07. Pull before each task,
integrate concurrent work before publishing, and push every completed, tested
task. No feature branch or pull request is required by this workflow.

## Separate checkouts, shared remote

- The continuous desktop worker owns `C:\dev\alo-os`.
- Claude Code uses a separate clone, for example `C:\dev\alo-os-claude`.
- Both local branches can be `main`; they are separate Git repositories.
- Current division: desktop worker owns the native desktop compositor and
  release-progress integration. Claude owns the filesystem-security workstream;
  secure file opening and handle-based file moves have been published. Confirm
  the next assignment with the owner rather than repeating completed work.
- Use a separate Linux `CARGO_TARGET_DIR` per checkout. Coordinate tests that
  alter the shared WSL kernel, cgroups, BPF pins or system services; separate
  build directories do not isolate those resources.

## Concurrent work and shared-system tests

Owner-approved 2026-09-09: both loops may implement, compile and run isolated tests
concurrently. This replaces the temporary single-workstream disk-space rule.
Keep the desktop target at `/root/alo-os-target`; never clean or reuse the other
worker's target. Claude's supervisor no longer uses a fixed
`/root/target-claude`: since 2026-09-11 `alo-kernel-loop` chooses
`$HOME/alo-builds/<checkout name>-<fingerprint of its path>`, one per checkout,
and says which it is in the first line of every run and in `.kernel-loop/loop.log`
(`tools/kernel-loop/src/where_it_builds.rs`). `/root/target-claude` is left where
it is and nothing removes it; whether it goes is the owner's decision.

The 12 GiB reserve before each build/test phase is unchanged. What changed is
where it is measured: the kernel-loop supervisor measures the filesystem its own
build directory is on, which is the one the build writes to, and its refusal names
that filesystem. The desktop lane's Windows C: preflight is unchanged. Separate
targets prevent artifact collisions, not memory pressure or exhaustion during a
running command.

Tests that attach BPF programs or manipulate kernel-global state must take
`alo_bounding::Waited::on_this_kernel()` for the entire fixture lifetime. Both
checkouts use the abstract socket name `alo-os/one-kernel-at-a-time`. Existing
bounding, boundary-loader and daemon integration fixtures already participate.
The parent holds it across its test child; the child must not acquire it again.
Do not add an outer suite lock using the same name. Contention waits, and the
existing bounded timeout fails rather than running unprotected. Never remove
another fixture's pins or stop its processes to obtain access.

Private buses, temporary files and ordinary compile/lint work may overlap when
they are actually isolated. Package installs, mounts, global service changes,
session changes or other shared maintenance outside these fixtures need an
explicit idle handoff with both workers. No supervisor restarts WSL or silently
repairs the shared environment. A waiting test is not a reason to pause all
development or weaken its assertions.

## Task lifecycle

1. Start with a clean working tree and `git pull --ff-only origin main`.
2. Implement one complete task, its tests and a separate descriptive task report
   under `docs/autonomy/updates/`. Follow the document ownership rules below.
3. Pass the required Windows/Linux/component checks and make a local commit.
4. Fetch `origin/main` again. If it advanced, rebase only unpublished task
   commits onto it. Resolve conflicts deliberately and rerun the required
   checks on the combined tree before publishing. Never rewrite published work.
5. Push normally to `main`. If another push wins the race, repeat integration
   and verification. Never use force-push or discard another worker's changes.

The Rust supervisor performs this sequence for its worker. It retries up to
three publication races; an unchanged remote after rejection means a real push
error, so it stops and preserves the local commit. Rebase conflicts require
deliberate review. Failed integration checks enter up to three repair attempts
on the same combined tree, followed by all independent gates again. A repair
is separately committed only after those gates pass; exhausted recovery or a
missing authority/resource preserves the work without publication. See
`DELIVERY.md` for the recovery protocol and its non-bypassable safety checks.
The worker itself still does not stage, commit or push; the supervisor does.

Keeping main clean means it contains integrated, tested work from both checkouts.
Pulling only at task start is insufficient: another task can finish while this
one is being implemented.

## Descriptive names and document ownership

Use descriptive task titles, filenames, commit subjects and status updates.
Examples: "Secure file moves", "Filesystem race protection", and "Native desktop
compositor". Legacy queue codes remain secondary references only; do not rename
historical ADRs or break existing links. Report filenames use lowercase hyphenated
words, for example `secure-file-moves.md`, not an internal queue code.

Only the integration owner in `C:\dev\alo-os` edits these shared documents:

- `CHANGELOG.md`
- `ROADMAP.md`
- `docs/autonomy/QUEUE.md`
- `docs/autonomy/STATE.md`

Other contributors include proposed changes to these documents in their own task
report. They still update code-local rustdoc and relevant contracts in the same
task; coordinate ownership before editing another shared specification. Tests
and publication gates are unchanged. This is an agreed contributor workflow,
not a GitHub permission rule that technically prevents those edits.

Use one uniquely named report per task. Only that task's owner writes it. After
publication, add a descriptively named follow-up report for corrections rather
than rewriting another contributor's report. No shared report index is required.

At the start of each iteration, the integration worker reads published reports
not yet referenced in STATE.md, checks their evidence, and consolidates their
change descriptions, queue status and remaining acceptance work into the four
shared documents. Reference the report paths in STATE.md for traceability; do
not edit the source reports to mark them processed. A report arriving during
publication is consolidated next iteration. Do not declare the release verified
while published reports remain unreconciled.

Before starting another task, existing contributor sessions must pull these
rules and reread them. Saved instructions do not update an already-running
Claude session automatically. Genuine code conflicts still require review;
never use an automatic union merge or force-push to hide one.
