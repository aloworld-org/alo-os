# Task branches and serialized integration

Owner-approved 2026-09-18. This replaces direct-to-main publication and the
older prohibition on feature branches. It also replaces the requirement to run
all nine gates before every push: **progress pushes to task branches are allowed;
merging into `main` still requires all nine gates and task acceptance.** A branch
push is a checkpoint, not a completed task or a release.

## Ownership and execution

- One short-lived branch per task: `task/<machine>/<descriptive-subject>`.
  One owner and one working tree per task. Branches do not change crate or plan
  ownership in `a-new-machine-becomes-a-lane.md`.
- Never run Git or edit a checkout while its worker or gate owns it. Finish the
  current run before switching its branch. Preserve existing work; no stash,
  destructive reset, automatic conflict resolution or force-push.
- Both this development PC and the third PC may integrate and squash-merge
  their own completed tasks, then delete their verified merged task branches.
  The Mac is stopped. Neither PC needs the other to perform its merge.
- Only one PC holds the integration turn at a time. The holder is the temporary
  coordinator for its candidate; the other PC keeps developing and pushing task
  branches. This is a manual queue, not a deployed GitHub merge-queue bot.
  Do not enable auto-merge or let direct-to-main supervisors race this queue.
- Existing `kernel-loop` and `dev-loop` publication commands still target `main`.
  Keep those publishers paused until separately adapted and verified for this
  workflow. Documentation does not change their executable behavior.

## Taking the integration turn

Use the shared, temporary remote ref `refs/heads/coordination/integration-lock`
as an atomic claim, not a product-work branch. Before the final gate, create
that ref through GitHub's create-reference API at the candidate's current head
SHA (`POST /repos/aloworld-org/alo-os/git/refs`). Creation succeeds for only one
PC. If it already exists, wait; never update, overwrite or force-push it. Record
which PC owns it, its claim SHA and the PR in that PR's integration evidence.

After claiming, fetch current main, integrate it into the task branch and run
all required gates on the final candidate. The claim SHA remains unchanged even
if integration changes the candidate. Only the holder may set its candidate's
`alo/nine-gates` status and merge. Keep the turn until the merge is verified;
then delete the coordination ref after confirming it still names the held claim
SHA. Delete the merged task branch separately. On a failed gate, preserve logs
and work, release the claim, and repair before taking another turn.

A crashed or disconnected holder does not lose its turn by timeout. Confirm its
gate/merge has stopped and arrange an explicit handoff before removing a stale
claim. Read failures or unavailable permissions mean no claim, not permission
to merge. This convention coordinates trusted PCs; main protection still checks
PR status and freshness. Never merge without both the claim and valid evidence.

## Task lifecycle

1. In an idle, clean checkout, fetch `origin`, update local `main` with
   `git merge --ff-only origin/main`, and create the task branch from it. For
   existing unfinished work, inspect and preserve it before creating its branch;
   do not replace it with a fresh checkout or implement it again.
2. Implement the assigned task and its tests, contract changes and separate
   task report. Commit with the checkout's configured owner identity, without
   a co-author trailer. Push normally to the task branch whenever useful.
3. Open one draft pull request to `main`. State what is unfinished, checks
   actually run and acceptance still owed. Never describe an ungated checkpoint
   as passed. Finish focused acceptance before requesting integration.
4. Either authorized PC selects its ready PR and claims the integration turn. Freeze that branch's head for the gate
   run (use a separate task branch for further work), fetch latest `main`, and
   merge it into the task branch if needed. Preserve published branch history;
   do not rebase it and force-push. Resolve conflicts deliberately.
5. Record the exact base SHA, candidate head SHA and Git tree SHA. Run **all nine
   gates from `tools/kernel-loop/src/gates.rs`**, plus required task acceptance,
   on that combined tree. Keep logs and durations. Existing requirements for
   Windows/Linux/component and real-hardware evidence remain in force; do not
   imply hardware certification from developer checks.
6. Only the coordinator reports `alo/nine-gates` success, on that exact PR head,
   after reviewing all results and confirming the gated source tree matches it.
   Include base/tree SHAs and evidence in the PR. Mark it ready. A pending or
   failed gate never permits a merge; a prior head's success is not transferable.
7. Recheck that `main` and the PR head have not moved. Squash-merge through the
   PR, using one descriptive conventional commit and the configured owner's
   identity, with no co-author trailer. If either moved, stop, integrate and
   validate the new combined tree; never bypass required checks or protection.
8. Verify the merged tree matches the gated tree, record the resulting main
   commit, and delete only that merged task branch, remotely and locally once
   its work is confirmed reachable. Preserve unmerged/parked recovery branches.
   Update an idle checkout's `main` by fast-forward before starting another task.

Do not batch unrelated tasks into one branch or keep release-long development
branches. Dependencies should land first; dependent branches integrate them
before their own final gate. The coordinator chooses the next ready PR promptly.

## Main protection

Configure GitHub to require a pull request, the up-to-date `alo/nine-gates`
status and linear history for `main`, including administrators. Disallow force
pushes and deletion. Enable squash merging and deletion of merged branches;
disable merge-commit and rebase merging. No extra reviewer is required for this
small-team workflow; that does not replace the coordinator's evidence review.
The status reports the existing local nine-gate run; it is not a newly installed
CI runner. Credentials able to write commit statuses remain trusted, so every
machine must follow the integration-turn rule. Do not bypass protection if API
access is unavailable: preserve the branch and report the concrete blocker.

## One build cache and one gate run per machine

The 2026-09-17 disk/contention correction supersedes older instructions to give
each checkout its own target or run builds/tests concurrently on this PC.
Reuse `/root/alo-builds/this-machine` and the serialized Linux source copy at
`/root/alo-trees/this-machine`; synchronize the chosen source before gating.
Never create another target directory. Check running Cargo processes and the
machine-wide gate lock before any build/test, and wait if another run owns them.
Check host C: free space as well as the Linux filesystem; preserve the existing
reserve checks and act before C: falls below 10 GiB. Other machines reuse their
existing configured cache and serialize their own shared environment.

Kernel tests retain `alo_bounding::Waited::on_this_kernel()` for their complete
fixture lifetime. Do not wrap the whole suite in that same lock, bypass it, remove
another test's BPF pins or stop its services. Shared mounts, packages, services
and WSL maintenance require an idle handoff. Branches do not isolate a kernel.

## Descriptive names and document ownership

Use descriptive task titles, filenames, commit subjects and status updates.
Examples: "Secure file moves", "Filesystem race protection", and "Native desktop
compositor". Legacy queue codes remain secondary references only; do not rename
historical ADRs or break existing links. Report filenames use lowercase hyphenated
words, for example `secure-file-moves.md`, not an internal queue code.

Only the designated integration owner edits these shared documents:

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
