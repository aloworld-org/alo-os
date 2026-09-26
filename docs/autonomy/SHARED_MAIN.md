# Task branches and serialized integration

Owner-approved 2026-09-18. This replaces direct-to-main publication and the
older prohibition on feature branches. It also replaces the requirement to run
all nine gates before every push: **progress pushes to task branches are allowed;
merging into `main` still requires all nine gates and task acceptance.** A branch
push is a checkpoint, not a completed task or a release.

## A tree already gated is not gated again

**Compare the tree, never the feeling.** `git rev-parse HEAD^{tree}` on what you
gated, and on `main` after the merge. **Identical: do not re-run the gates** —
the second run reads the same bytes and can only repeat the first. **Different:
gate it**, because that is the case the post-merge run exists for, where two
green branches combine into a tree neither of them was.

This is the only permitted reason to skip a gate, and the reason it is safe is
that it is checkable. A tree hash is a fact. *Nothing much changed*, *the tests
passed a minute ago* and *it is only documentation* are not, and each of them
has already cost this repository something this week: a test no gate ran, so a
broken assert passed nine gates twice; a stale output file read as a result; and
a `cargo fmt` that never reached the checkout being reviewed. Every one would
have been hidden by a skipped run somebody felt was safe.

**Gate what the diff can reach.** A change under `docs/` does not need every
suite in the workspace; a change to one crate needs that crate and the crates
that depend on it, which cargo can work out. Reaching for the whole workspace
every time is what makes a gate expensive enough that somebody starts wanting to
skip one — and then they skip the wrong one. Say in the result file which gates
you ran and why those were the ones the diff could reach.

**What never shrinks:** the run on the tree you merge, when that tree is new.
That is the one measurement this whole workflow is built on.

## Ownership and execution

- One short-lived branch per task: `task/<machine>/<descriptive-subject>`.
  One owner and one working tree per task. Branches do not change crate or plan
  ownership in `a-new-machine-becomes-a-lane.md`.
- Never run Git or edit a checkout while its worker or gate owns it. Finish the
  current run before switching its branch. Preserve existing work; no stash,
  destructive reset, automatic conflict resolution or force-push.
- Both this development PC and the third PC may integrate and squash-merge
  their own completed tasks, then delete their verified merged task branches.
  Neither PC needs the other to perform its merge. **The Mac runs a lane of
  its own** — it was stopped for part of 2026-09-18 and has not been since, and
  a sentence here saying otherwise was quoted as a reason elsewhere before
  anybody checked it.
- Only one PC holds the integration turn at a time. The holder is the temporary
  coordinator for its candidate; the other PC keeps developing and pushing task
  branches. This is a manual queue, not a deployed GitHub merge-queue bot.
  Do not enable auto-merge or let direct-to-main supervisors race this queue.
- Existing `kernel-loop` and `dev-loop` publication commands still target `main`.
  Keep those publishers paused until separately adapted and verified for this
  workflow. Documentation does not change their executable behavior.

## Taking the integration turn

**There is no lock. Changed 2026-09-19, after it cost two days.**

A machine merges its own finished work when its pull request carries a passing
`alo/nine-gates` on that exact head. It does not claim anything first and does
not wait for another machine's permission.

### `main` no longer requires a branch to be up to date. Changed 2026-09-19

It did until then — GitHub's **strict** status checks — and the reason it does
not any more is that the requirement was starving the slowest machine in the
fleet. The third PC gated one task three times and was refused three times with
`mergeable_state: behind`, each refusal costing a fresh thirty-to-forty-minute
run, because `main` moved while it gated. The interval between landings had
closed to about the length of one gate run on that machine, so it could lose
indefinitely.

**What strict bought, stated fairly:** it prevented two branches each gated
green against *different* `main`s from merging into a combination neither was
tested as. That is a real fault and this is not a claim that it cannot happen.

**Why losing it is acceptable:** `landing.rs` rebases onto the newest `main` and
re-gates the combined tree immediately before pushing, so the window between
*what was gated* and *what is merged* is seconds rather than the forty minutes
strict was charging to re-check it.

**And the honest part:** strict never caught the failures that actually cost
this repository days. PR #32 merged an unformatted tree with strict on. So did
the renamed ADR whose links all still read the same, and the crate missing from
`alo-software`'s terminal check. Each blocked five machines. Strict guarded one
narrow case while the expensive ones walked past it, which is why what replaces
it is wider rather than narrower.

### A merge is finished when `main` still gates

**The machine that merged gates the new `main` head, and reverts its own merge
if it fails.** This is part of landing, not something done afterwards if there
is time.

It covers more than strict did, because it catches `main` breaking for *any*
reason rather than only for staleness — including every one of the three
failures above. And it turns the expensive shape, *`main` is broken and the next
machine to gate finds out*, into the cheap one: broken for one gate run, then
reverted by the machine that broke it, which is also the machine that still has
the context to fix it.

A revert is not a judgement about the work. It is putting `main` back so four
other machines can keep moving, and the branch is re-gated against the new head
and merged again.

**A failure that does not reproduce is flakiness, not breakage — do not
revert.** Re-run the failing test alone, three times. If it passes every time,
`main` is fine and what you have found is a test that depends on its
environment or on what else is running beside it. Record it, and fix it as its
own task; reverting somebody's work over it would remove good code and leave
the actual fault in place.

This happened the first time the rule was used, on 2026-09-19. `main` gated
`test=101` after three merges, and the failure was `AddrInUse` in
`alo-agentd`'s `a_network_that_will_not_bind_is_a_line_and_the_others_are_still_bound`
— a test that asks for a free port, lets go of it, and then binds it again,
racing the other five hundred tests in the same binary. It passed three times
in a row when run alone. Reverting on that first result would have taken out
the decoder decision and the errand for nothing.

**A merge queue would be the proper answer and is not available here.** The
queue waits for `alo/nine-gates` on a temporary branch of its own, and with no
CI in this repository nothing would ever post it, so every merge would hang.
Recorded so that nobody proposes it a second time.

**Why the lock was removed.** A shared claim ref
(`refs/heads/coordination/integration-lock`) was held by one machine that then
stopped, twice: `main` was frozen for eight hours on 2026-09-18 and fifteen more
on 2026-09-19, with finished work sitting on branches nobody was permitted to
merge. The protocol had a way to take the turn and no way to recover it, and
recovering it by hand needed the claim SHA, a GitHub token, and somebody awake.
A coordination device whose failure mode is *the whole fleet stops* is worse
than the collisions it prevents, when those collisions were already prevented by
the branch protection underneath it.

If a coordination ref exists from before this change, any machine may delete it.

## Merging several ready branches at once

A machine may combine **more than one** ready pull request into a single
candidate, gate that tree once, and merge it — rather than gating each branch
separately.

This is faster, and it is also more honest. Gating four branches one at a time
tests four trees, **none of which is the tree that ends up on `main`**; gating
the combination tests the thing that will actually exist. Two of the breakages
on 2026-09-17 reached `main` through exactly that gap.

The rules for it: every branch in the combination is brought up to date with
`main` first — by the machine, since nothing enforces it any more; the combined
candidate is pushed and gated as one commit; and if it fails, the combination is
split and the branch at fault is named rather than all of them being refused
together.

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
9. **Gate the new `main` head, and revert your own merge if it fails.** See
   *A merge is finished when `main` still gates*.
10. **Clear the blockers that named the task you just finished**, in the same
    change that marks it done.

### A stale blocker is invisible work

A task whose blocker has been cleared still reads *blocked* until somebody
edits the line, and every machine that surveys the plans skips it. Nothing
tells them otherwise: a plan is read, not computed.

Found on 2026-09-19. `v0-5-hands-on-the-desktop-plan.md` task 2 read *blocked —
on `v0-5-the-session-and-the-displays-plan.md` task 3*, and that task was
finished. The work had been takeable for some time and was being stepped over
by every lane looking for something free — while task 7 of the same plan, which
depends on it, was being offered to a machine that could not have finished it.

So the machine that finishes a task is the one that clears the lines naming it.
It is the only machine that knows, and it knows at exactly the moment the
knowledge is cheap.

**When surveying the plans for free work, key on the `**Done,` marker and not
on the status word.** A task carries `**Status:** ready.` *and*, separately, a
`**Done, <date>.**` marker in its body — so reading the status line alone
reports finished tasks as free. That mistake nearly handed a lane thirteen
completed tasks on the same day this rule was written.

Do not batch unrelated tasks into one branch or keep release-long development
branches. Dependencies should land first; dependent branches integrate them
before their own final gate. The coordinator chooses the next ready PR promptly.

## Main protection

Configure GitHub to require a pull request, the `alo/nine-gates` status and
linear history for `main`, including administrators. Disallow force pushes and
deletion. **Do not require a branch to be up to date** — that setting was
turned off on 2026-09-19 for the reason given under *Taking the integration
turn*, and what replaced it is step 9 of the task lifecycle. Enable squash merging and deletion of merged branches;
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

## A branch belongs to the machine that made it

The machine that creates a task branch gates it, merges it and deletes it.
Nobody waits for another machine's permission to merge their own finished work,
and nobody merges somebody else's.

**A merged branch is deleted in the same turn it is merged.** A finished branch
left in the list is indistinguishable from work still in flight, and a branch
list that cannot be read is not a record of anything. GitHub deletes it on merge
where the repository is set to; where it is not, delete it explicitly and
confirm it is gone.

**An unmerged branch is never deleted.** Parked tasks and recovery work live
there, and that work exists nowhere else.

**A branch nobody has moved for a day is reported, not removed.** Its owner says
whether it is alive or abandoned; silence is not consent to delete.

## Gate the committed tree, not the working tree

Run the gates against **what the commit contains**, not what the working
directory happens to hold. Use `git archive`, a fresh clone, or a checkout of
the candidate SHA — not a copy of a working tree that a worker has been writing
into.

The two differ in ways that are invisible until they are expensive. On
2026-09-18 a gate failed on `alo-updating`'s base-image check because the
recipe in the working tree carried CRLF line endings: a worker had written the
file that way, `.gitattributes` normalised it to LF *in the commit*, and the
gate — which copied the working tree — tested bytes that would never reach
`main`. The branch was refused for a fault that did not exist in the work being
merged, and the same trap catches any test that compares exact file content.

The rule is the same one the integration turn already follows for `main`: gate
the thing that will actually land.

## Gate what the change can reach

The nine gates take about twenty minutes, and nearly all of it is the whole
workspace's tests. Running all of them for a change that cannot possibly reach
them is the largest avoidable cost in this repository: on 2026-09-19 a
documentation-only commit spent twenty-three minutes running 120 test binaries,
none of which read a word it changed.

### One gate is not scoped, and this is the reason

**If the diff contains a single `.rs` file, `cargo fmt --all --check` runs —
whatever else it touches, and wherever that file lives.**

It is not in the table below because it is not a choice. It takes seconds
against the whole workspace, and it is the gate that has stopped every machine
here most often: PR #32 merged an unformatted file under `crates/`, and the next
five candidates on five machines all failed on a file none of them had touched.

The table read the other way round until 2026-09-19, and that is how #32 got
through: it required *formatting* of the rows that cannot break `cargo fmt` —
Markdown under `docs/` — and not of the rows that can. A gate demanded where it
is inapplicable and skipped where it applies is worse than no table, because it
reads as care. Found by the third PC's lane.

### The rest is scoped to what the diff can reach

| What the diff touches | What must pass |
|---|---|
| `docs/decisions/**` | the citation check, which is what a renamed decision breaks |
| `docs/autonomy/*plan.md` | the plan checks that read every plan |
| `docs/**` otherwise | the citation check, the plan checks |
| `image/**` | `alo-image`, `alo-installing`, `alo-updating` |
| `tools/kernel-loop/**` | the supervisor's own three gates |
| `crates/<name>/**` | that crate, and every crate that depends on it |
| `Cargo.toml`, `Cargo.lock`, `.gitattributes`, anything workspace-wide | all nine |

**Where the mapping is uncertain, run all nine.** A documentation change did
break `main` for five machines on 2026-09-17 — an ADR was renamed and every link
to it still read the same — which is why the decisions row is not simply a
formatting question. The saving comes from the common case, not from trusting
the rare one.

Say in the pull request which gates ran and why those. A candidate that merges
several branches runs the union of what each reaches.
