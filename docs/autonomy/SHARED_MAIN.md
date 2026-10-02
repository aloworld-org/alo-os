# Task branches and integration

> ## CI is authoritative since 2026-10-02, and much of what follows describes a retired protocol
>
> `main` now requires **`alo/gates-on-a-runner`**, which the hosted-runner
> workflow posts on both `pull_request` and `merge_group`. **`alo/nine-gates` is
> required by nothing, and no lane types a status.** Read back from the API at
> the moment of the change: `required contexts: ['alo/gates-on-a-runner']`,
> `strict: false`, `enforce_admins: true`.
>
> **Retired, and not to be rebuilt:** hand-posting a required status; exclusive
> holds on the queue; turn-taking between lanes; verdict inheritance across a
> tree; the four-sha guard; and any requirement that `main` be unmoved. A lane's
> own full gate is now **diagnostic rather than admission**.
>
> **A landing is now:** commit, push, open a pull request, enqueue. CI checks the
> head, the queue builds its own commit on current `main`, CI checks that, and it
> merges. Verified end to end on `#363` — no status typed by anyone at any point.
>
> **Why the sections below are wrong rather than merely dated.** We built a
> second integration protocol *around* GitHub's merge queue, and it defeated most
> of the benefit the queue exists to provide. The queue already builds an
> integration candidate and checks it; requiring `main` to stay unmoved so a
> verdict could be inherited was re-implementing by hand the thing we were
> standing on. On 2026-10-01 eleven faults were found across three lanes and
> **not one was in the product** — every one was in the apparatus that existed to
> make a hand-typed status trustworthy, and the fleet delivered 14 merges against
> a five-day average of 40.
>
> **What survives, because it is true whoever posts the result:** a gate must not
> run against a checkout something else is editing; a validator must not reset a
> developer's working checkout to obtain its input, and should test an isolated
> checkout of a named commit; an exit code must be read outside a pipe; a report
> names the fields that would have to agree; and a build needs disk room. Those
> are about whether a measurement is of the thing it claims.
>
> Per-machine work queues are in [the-queues.md](the-queues.md). `ROADMAP.md`
> remains the only order.

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

**Amended 2026-09-28 by the owner, with one exception and the measurement that
earned it.** The boundary suite — `alo-bounding`, `alo-agentd`, `alo-boundaryd` —
runs on `main` after a merge rather than under every merge on every lane. The
numbers, taken on the development PC with `cargo nextest`, 8,883 tests across 551
binaries and zero failures:

| | |
|---|---|
| whole suite, everything, clean | 1,326s |
| the three boundary crates, serialised | **1,241s** |
| the merge gate as it now runs, measured | **166s** (144s of it tests) |

8,883 tests in the first row, 8,149 in the third, zero failures in either.

Those three attach BPF programmes to the kernel. There is one kernel, an attach
waits for an RCU-tasks grace period, and they must run one at a time because the
`Mutex` they were written around cannot cross per-process isolation. **No hardware
shortens it**, and it was the same 1,241 seconds whether a change touched the
kernel or corrected a sentence in a document — a 22-minute floor under every
merge, on a repository three lanes land work on.

So the merge gate is everything else — **166 seconds, measured, not derived** —
and `cargo nextest run --profile boundary` runs on `main` after each merge on the
machine that can run it.

*A caution worth inheriting, because it was got wrong here first:* 1,326 − 1,241
is **not** the answer, and the 85 seconds it gives is not a time anything takes.
The whole-suite run had the boundary crates and everything else in flight at once,
so deleting them returns far less than their wall clock. The only trustworthy
number is the one from running the gate you actually intend to run. **The cost is real and is not hidden:** a boundary
regression can reach `main` and be found minutes later rather than never merged.
That is the presubmit-and-postsubmit trade, taken deliberately, because the rule
as written made the queue a queue.

The rest of this section is unchanged: everything that is not the boundary suite
still runs on the exact combined tree, and a lane that shrinks it further is
outside the rule rather than reading it generously.

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

Found on 2026-09-19. `hands-on-the-desktop-plan.md` task 2 read *blocked —
on `the-session-and-the-displays-plan.md` task 3*, and that task was
finished. The work had been takeable for some time and was being stepped over
by every lane looking for something free — while task 7 of the same plan, which
depends on it, was being offered to a machine that could not have finished it.

So the machine that finishes a task is the one that clears the lines naming it.
It is the only machine that knows, and it knows at exactly the moment the
knowledge is cheap.

**When surveying the plans for free work, key on the `**Done,` marker and not
on the status word.** A task carried `**Status:** ready.` *and*, separately, a
`**Done, <date>.**` marker in its body — so reading the status line alone
reported finished tasks as free. That mistake nearly handed a lane thirteen
completed tasks on the same day this rule was written.

**The marker is still the authority, and the two now agree.** On 2026-09-27
every plan under this directory was read for the disagreement, and **74 tasks
across six plans** said `ready` above their own `**Done, <date>.**` — 32 in
`the-executable-plan.md`, a release that shipped, and 14 each in
`kernel-enforcement-plan.md` and `accounts-and-session-entry-plan.md`. Every one of those
status words now reads `done`, copied from the marker directly beneath it and
with no other word of any plan touched. So this rule is advice about which of
two statements to trust, and no longer a workaround for 74 of them being
wrong — and the rule above, that the machine finishing a task clears the lines
naming it, now includes its own status line.

Do not batch unrelated tasks into one branch or keep release-long development
branches. Dependencies should land first; dependent branches integrate them
before their own final gate. **Nobody chooses the next pull request:** the merge
queue takes them in the order they are enqueued and builds each against current
`main`, which is what retired the coordinator this rule used to name.

That is batching along the other axis, and both are forbidden for the same
reason. *Several tasks in one branch* and *several branches held for one merge*
are the same bet — that a larger thing integrates as easily as a smaller one —
and it loses in the same way, with every conflict arriving at once.

### One task at a time, and as many branches as the machine can keep green

**Each agent actively implements one task at a time. A machine holds as many
unfinished task branches as it can keep gated and green, each in its own
worktree.** When a task is waiting on CI, review or a decision somebody else
owns, start another approved, independent task rather than idling.

```
active implementation   one task per agent
unfinished branches     as many as the machine can keep gated and green
                        ~2-3 where a gate is minutes; 1 where it is an hour
isolation               each task its own branch and its own worktree
local Rust builds       one Cargo operation per machine — the real ceiling
landing                 enqueue the moment CI passes; never batch
completion              acceptance criteria met and the PR merged into main
```

**Why there is no fixed number any more.** *One branch until its PR merges* reads
like discipline and is a stall: it makes an agent idle for every minute of review,
CI or a decision it does not own. On 2026-10-02 this lane sat with nothing to do
through a queue that had been empty for two and a half hours. The fixed count of
**two** that replaced it was right for the day it was written, when landing needed
a hand-typed status and a turn; once the merge queue became authoritative the
count stopped describing anything real, and the owner replaced it the same day.

**The ceiling is the compiler, not the policy.** Gate times measured on
2026-10-02: 330 s on the development PC, 909 s on the panel machine, 57–97 minutes
on the Mac. A machine runs **one Cargo operation at a time**, so ten open branches
cannot be gated any faster than one at a time — they queue on the same compiler.
A number larger than what the machine can keep green is paperwork. Worktrees give
separate checkouts in separate directories, so a second task does not disturb the
first, but they share the machine's memory, CPU and disk, and **they do not
multiply the agent**.

**Add machines, not branches.** Where more parallelism is wanted, the thing to add
is a machine with its own compiler, not another branch on a machine already
building.

### Landing is continuous, and batching is forbidden

**A branch is enqueued the moment CI passes.** Finished work is never held back to
be merged together later, whether at the end of a day or at any other agreed
moment. This is a prohibition, not a preference.

**It does not avoid serialising; it concentrates it.** The merge queue builds each
candidate in turn against current `main` either way. Ten held branches still build
ten times — all at once, in the window with the least time left to fix whatever
breaks.

**Conflicts compound with the age of a branch**, and both of the day's examples are
in this repository. A branch carrying *other* tasks' status edits went `DIRTY`
three times, once for each of those tasks landing beneath it — which is also why
status edits belong with the task they describe rather than riding on a feature
branch. And two branches opened the same afternoon collided where one deleted a
struct field the other's test helpers set, so whichever landed second would not
compile; it was caught before either was sent only because both were fresh enough
for the old field name to be worth grepping. Two branches is one pair to check.
Ten is forty-five.

**A gate is a statement about a tree, and trees go stale.** A verdict taken in the
morning describes a tree the queue will not build, which is why `GATES-ABOUT`
prints the head and the tree it measured and why a rebase requires a new gate
rather than inheriting the old one.

**And landing fast is how a lane finds its own mistakes.** The worst fault of
2026-10-02 — a handle floor converted against an owner's explicit ruling, with the
one test able to detect it asserting the fault as the promise — was already in
`main` and was found by measuring landed code. Every hour that work is held is an
hour of building on top of something not yet examined.

A daily rhythm, where one is wanted, belongs at **review** rather than at merge:
read what landed, correct the plans, and fix what the day's code revealed.

**Opening a second branch does not transfer the first.** Ownership of an open
pull request lasts until it merges or is deliberately closed. Failed checks and
review requests on it come **before** new work on the second — an agent with two
branches has one job queue, not two independent ones.

**Record dependencies explicitly**, and do not let one blocked task block the
machine. A task waiting on a decision waits; the agent does not. Where the
question can be settled by authorised investigation, investigate and settle it
rather than waiting to be told.

### The order a task is worked in

```
1  start from freshly fetched main, with a clear outcome and one owner
2  implement, verify, open the pull request — code, tests and the documentation
   it needs together
3  while it waits on CI, review or a decision, start the next independent task
   in another worktree
4  keep watching the open PR: failed checks and review requests first
5  merge through the protected queue, which validates the integration candidate
   against current main and the changes queued ahead of it
6  update the task's plan entry and remove the branch and worktree once the work
   is accounted for on main
```

**Step 5 is why a branch is not rebased merely because `main` advanced.** The
queue builds and checks the integration candidate; `strict` is off on purpose,
and *Main protection* below records that. Three lanes spent a day re-implementing
that by hand — frozen main, exclusive turns, inherited verdicts — and defeated
most of the benefit the queue exists to provide.

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

### A gate is inherited when the tree is byte-identical, and never otherwise

**A gate is a function of the tree and of the machine that ran it.** If a run on
this machine, with this gate script, already answered nine of nine for exactly
this tree object, running it again asks a question whose answer is recorded.

So a queue commit inherits a pass when **four mechanical facts** hold, and is
gated from scratch when any one is missing:

1. the same **tree object hash** — `git rev-parse <commit>^{tree}`
2. the same **gate script digest**, which the gate already prints for this reason
3. the **same machine**
4. a recorded **nine of nine**

**Byte-identical means the tree's own hash and nothing weaker.** Never *no file
I think the gate reads*. The Mac lane rebased on 2026-09-30 believing their
green carried over; it did not, because the tree had gained six hundred lines
they had not touched, and with the weaker wording they would have talked
themselves into it. **A tree comparison cannot be argued with; a judgement about
which files matter can, and will be, at four in the morning.**

**Why it is worth having.** With three lanes on one queue the gate sat *inside*
the queue path, so depth cost whole runs rather than position — the third entry
waited for two full gates before its own sixty-minute window opened. Two entries
ejected that way on the evening this was written. Taking the gate off that path
is the only change that makes every lane faster rather than one. *The Panel
lane's argument; the tree-hash condition is theirs; the fourth fact — that the
machine is part of the function — is this lane's.*

**What it does not do, measured rather than assumed.** It does not help across a
rebase, because rebasing onto a moved main always changes the tree. The Mac lane
priced that the same evening: their branch's only difference from a nine-of-nine
tree was **one markdown file in `docs/autonomy/` that nothing compiles**, and the
rule still requires a full gate — fifty-five minutes on that machine to prove a
document did not break a compositor. With three lanes landing documents all
night that is the common case, not a rare one.

**And the obvious lever does not work here, which is worth knowing before
somebody builds it.** The Mac lane proposed that the gate declare its own inputs,
computed rather than judged, so the comparison stays mechanical while the tree
stops being the only mechanical thing available. That is the right shape. But
**the inputs of this workspace's gate are not statically computable**:
`crates/alo-reconciling`'s tests read files off the disk at run time by a path
the *ledger's own content* supplies — `read_to_string(here.join(named))`, where
`named` is data. A scan for `include_str!`, or even for literal paths, finds none
of them, and a lane that built the lever that way would inherit a pass over a
documentation change that genuinely breaks the workspace's tests. The version
that could work observes what a run actually opened rather than predicting it,
and that is a real piece of work rather than a midnight one.

### Tree identity serialises every lane, so gate last

**The condition is a still main, and that is the price of the mechanism.** Every
landing changes `main`, so every landing invalidates **every other lane's gated
tree**. With three lanes it is **N sequential gate-and-land cycles with no
overlap** — the repository absorbs about one change per gate duration however
many machines are pointed at it.

**And it is worse than unhelpful capacity: a lane that does not hold the turn
cannot usefully gate at all.** A gate started before you hold the turn is
answering a question about a tree the queue will never build. So the two machines
not holding the turn are idle **by construction** rather than merely unspent.
*Short of turns, not capacity* was the laptop lane's phrase and it understates
it: the capacity is not slow, it is unusable.

**Superseded on 2026-10-02, and left here with its reason because the reason
outlived it.** This section described gating *to earn a landing*, and the order
it gave began `wait for the turn`. `main` no longer requires a hand-typed status
— it requires `alo/gates-on-a-runner`, which CI posts on the head and on the
queue commit — so **a local gate is diagnostic, not admission**, and there is no
turn to wait for. The order is now:

```
1. commit
2. push the branch
3. open the pull request
4. enqueue; CI gates the head and the integration candidate
```

**Run whatever local checks are useful before pushing** — formatting, clippy, the
crates the change touches. Do **not** run the full suite to earn a landing: this
lane paid 89 minutes for a verdict CI produced in 15, and twice killed a
fifty-minute run because `main` had moved under it.

**The order below is what the old one said**, kept because *what* it was wrong
about is instructive and because a machine may still choose to gate locally:

```
1. commit
2. wait for the turn — the queue empty, no hold standing against you
3. rebase onto current main
4. push the branch
5. gate
6. attest, enqueue alone, inherit
```

Step 2 is the stall the owner replaced: *one branch until its PR merges* and
*wait for your turn* are the same mistake in two places, and both make an agent
idle while something it does not own is pending. See **One task at a time, and as
many branches as the machine can keep green**, and **Landing is continuous, and
batching is forbidden** — *hold them and merge them together later* is the same
mistake a third time, moved from the start of the work to the end of it.

**The push moved ahead of the gate, and the reason is the rule below about
refusing an unpublished head.** A gate that refuses unless `HEAD` equals
`origin/BRANCH` cannot run before the push, so the order follows from it rather
than from preference. **A push is not a landing** — the scarce thing is the queue
turn, and a branch on the remote costs nobody anything. What it buys is that the
tree gated is provably the tree pushed, instead of two shas somebody compares
afterwards and remembers comparing.

**Gate at four, never before. The gate is the last thing before the push, not the
first thing after the commit.** This was the habit from before any of this, and
on 2026-10-01 the Mac lane killed **two fifty-minute runs inside forty minutes**
for it: one invalidated when the governance rules landed, one when the panel's
mint test did. Both were producing verdicts for trees that could not land. The
laptop lane's own attestation died the same way the same hour. **A rule neither
lane wrote down and both paid for twice.**

**The hold that makes it work has to be asked for.** Engineering for tree
identity means excluding every other lane for a gate's duration, and the
mechanism does not carry that requirement inside it — both lanes supplied it by
habit, and habit is not a shape. **A hold nobody was asked for is not a hold, it
is a stall**, and the last time this was left to habit it cost five hours of
mutual waiting built on a false premise. Ask, name a deadline, and say *tell me
no rather than hold silently*.

**What to measure, since this is the real constraint rather than the gate's
duration:** three lanes have three machines and one turn. Four times on the night
this was written somebody reached for **capacity** — a lane killing a passing run
to gate for another, a lane offering its machine, a lane offering to send its
scripts — and not one of those created a turn. **A shared constraint nobody has
measured gets answered with whatever is easy to give.**

### While something is queued, the machine belongs to it

**A queue entry has one window and it is sixty minutes.** Inside it the queue
commit must be gated and attested, or the entry ejects and everything behind it
waits again. A gate takes most of that window, so there is room for exactly one
and none at all for a second.

**So while a lane has an entry in the queue, that lane's machine gates the queue
commit and nothing else, until it lands or ejects.** Not the next task. Not the
top of its own stack. Not a quick check. The work that cannot land can always be
built afterwards; **the window cannot be reopened.**

**This is a missing rule rather than two slips, which is why it is written
down.** On 2026-09-30 two lanes independently kept a machine busy through the
only window the work that *could* land had — one building the next feature, one
gating the top of its own stack while the bottom's window opened and closed.
Both entries ejected. Six hours passed with three lanes producing and **nothing
landing at all.**

**And stacking makes this sharper rather than softer.** A stack is a chain of
landings and **only its bottom has a window.** Gating the top while the bottom
is queued is the same fault wearing the clothes of good discipline: the stack
was the right structure and the wrong end of it was being measured.

*Named by the Mac lane, from the hour it cost them.*

### Say which shared document you are about to edit, before you edit it

**To every lane, before the edit — not before the landing.** The rest of this
protocol announces a *landing*, because landing is the scarce thing and a branch
is nobody else's until it lands. A document is not like that. Two lanes can edit
one at the same time, on the same afternoon, both correctly, and neither finds
out until one of them rebases.

**That is not hypothetical.** On 2026-09-30 the Mac lane and the dev PC both
moved canvas promises into v0.01, in two branches, within an hour, each acting on
the owner's direction and neither knowing. `docs/features.md`, the evidence
ledger and the reconciler's own test were edited twice over. One commit was
dropped; nothing was lost, and only because the collision was noticed by reading
the other lane's open pull request rather than by anything in this file.

**Which documents this is about, stated so it can be checked rather than felt.**
A document needs announcing when **it holds a number that something else
asserts** — another document, or a test. `docs/features.md` holds the promise
counts the reconciler reads. `docs/autonomy/evidence-it-boots-and-the-agent-acts.md` states its own
figures and a test reads all four of them. Those behave like locks whether or not
anybody declared one: two lanes each correctly adding one promise produce a count
that is wrong by one, and the test that catches it names neither of them. A
design note that nothing counts is not a lock and needs no announcement.

**It was findable in advance, which is why it is a rule and not a resolution to
be careful.** The Panel lane had already refused to edit another lane's ledger
while its counts were in flight, and said out loud why — *a second lane editing a
ledger's prose while its counts are in flight is how two versions of a number
appear.* Both other lanes heard it. Neither generalised it from that ledger to
the document they were about to edit themselves. A hazard named about one
document is a hazard about every document of that shape, and what failed was not
attention. *Proposed by the Mac lane, from the collision it was half of.*

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

**A branch whose content is on `main` is finished, whether or not it merged.**
The repository is set to `delete_branch_on_merge`, so a branch that merges is
removed without anybody acting — and **that is exactly why the ones left behind
are invisible.** What accumulates is the cases the setting cannot see:

```
merged                deleted automatically — the rule works and nobody notices
superseded            a PR closed because its content landed under another one
a lane's own locals   no repository setting touches these at all
```

On 2026-10-02 the Mac lane had five dead branches on that second and third
footing: `#344` and `#348` were **closed, not merged**, because they were
superseded whole by `#351`, and three more existed only locally. *A merged branch
is deleted* read literally covers none of them, and the lane had edited this very
file twice that night — once **eighteen lines below this rule** — without reading
it. **A document read only as a place to write is not a document anybody is
following.**

**So the test is the content, not the merge.** Before deleting, check that what
the branch holds is on `main` — `git cat-file -e origin/main:<a file it added>`,
or the diff against `origin/main` being empty of its work. Neither *the PR is
closed* nor *the branch merged into main* is the right question: the first is
true of abandoned work, and the second is false of superseded work that did land.

**An unmerged branch whose content is not on `main` is never deleted.** Parked
tasks and recovery work live there, and that work exists nowhere else. Two such
branches were kept that day for exactly this reason: both were hundreds of
commits stale and almost certainly dead, and *almost certainly* is not a
measurement. **A branch holding real work deleted on a guess is unrecoverable; a
stale branch kept costs a line in a list.**

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

### Refuse the ambiguous state rather than choosing which hole to have

**Two lanes' gate scripts were measured against each other on 2026-10-01 and
each had the other's safety.** Neither was simply wrong; they had chosen opposite
defaults and neither had written the choice down.

```
reads the live checkout     nothing can be destroyed
                            the verdict may be about a tree no commit names
forces with reset --hard    the verdict is always about a committed tree
                            unpushed work is discarded, and then it passes
```

The second is the worse of the two and worth naming precisely: a `reset --hard`
over an unpushed rebase gates the **pre-rebase** tree and answers **nine of
nine**. Every line of that log is true. **It is the only fault in this family
that would survive review** — a reviewer reading the output cannot catch it, and
only comparing the gated tree to the pushed tree can.

**The answer is neither default: refuse the state, and gate nothing.** But it
takes **two** comparisons, because the two scripts fail in opposite directions
and neither comparison alone covers both:

```
the working tree is clean       so the verdict is about a commit
HEAD equals origin/BRANCH      so the verdict is about a commit anybody can fetch
```

**The second is the one that is easy to miss, and it is the one that produces the
correct-looking success.** The forcing lane's hole was live with a **clean** tree:
the rebase was *committed and unpushed*, so `git status --porcelain` was empty and
`reset --hard origin/BRANCH` would still have discarded a real commit. *A clean
tree cannot lose anything to a reset* is false — it can lose every commit that
has not been pushed. A dirty-tree check alone passes that state happily.

**Each lane had built exactly the half that made its own hole invisible to it.**
The forcing gate satisfied the first by force and violated the second. The
live-checkout gate satisfied the second by never moving and violated the first.

So the ambiguity to make unconstructible is not *is this committed* but **is the
thing about to be gated the thing the author means** — and a gate refuses unless
that state is already published, so no verdict can be about a tree its author
cannot point at.

```
GATES-REFUSED: the working tree is not clean, so a verdict would not be about any commit
GATES-REFUSED: head <sha> is not published as origin/<branch> (<sha>), so a verdict
               would be about a tree nobody else can fetch
GATES-ABOUT:   head <sha> tree <sha> branch <name>
```

**Use `rev-parse --verify --quiet` for that comparison.** Plain `rev-parse` prints
its argument back when it cannot resolve a ref, so the failure case yields a
*string* rather than an error — which is how this repository's one retracted
attestation happened on 2026-09-30, and it reappeared in the first version of
this very guard. The proof run showed a branch name where a sha belonged.

**And print what the verdict is about**, because that line is what makes either
hole visible: under a live-checkout gate it would differ from any commit, and
under a forcing gate it would differ from what was pushed. The tree hash is also
what the inheritance rule compares, so a run that does not print it leaves
nothing to compare and the comparison becomes a thing somebody remembers.

**This is the same shape as `Place` having no `Default`.** A default is a value
somebody chose and a caller can fail to override; an absence cannot be overridden
wrongly. Here: a state nobody can be in beats a state handled two ways. Both
lanes reached for *which hole do I prefer* and the answer was **make the
ambiguity unconstructible**. *The two holes were found by the laptop lane and
this one measuring each other's scripts; the refusal is this lane's.*


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
