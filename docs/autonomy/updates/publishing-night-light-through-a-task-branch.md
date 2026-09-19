# Publishing *Night light and display colour* through a task branch

**Date:** 2026-09-18
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 4
(*Night light and display colour*). A follow-up to
`docs/autonomy/updates/night-light-and-display-colour.md`, which is that task's
own report and is not edited here.
**Contributor:** second Claude Code worker on task 4, in `C:\dev\alo-os`.
**Crate:** `crates/alo-displays` — unchanged by this report, byte for byte.
**Status:** ready for integration, through a pull request and not through a push
to `main`. Its pull request follows task 3's, because task 4 depends on task 3.

## What the gates actually said

Task 4's code was written, gated and committed. `.kernel-loop/loop.log` records
the run in full:

```text
1789777273 this task's tree passed: formatting; clippy, warnings denied; the
           workspace's tests; the supervisor's formatting; the supervisor's
           clippy; the supervisor's own tests; rustdoc, warnings denied; the
           BPF target's formatting; the BPF target's clippy
1789777280 the evidence stood up: … (six tests, each on its own)
1789777281 committed 1b497ed locally
1789777285 the gates refused task 4
```

All nine gates passed. All six pieces of acceptance evidence stood up. The
refusal came from the step after them:

```text
$ git push origin main
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote: - Changes must be made through a pull request.
remote: - Required status check "alo/nine-gates" is expected.
 ! [remote rejected] main -> main (protected branch hook declined)
```

`origin/main` had not moved, so this was not a lost race — `publishing.rs`
says so itself and stops rather than retrying. `main` is protected exactly as
`docs/autonomy/SHARED_MAIN.md` § *Main protection* says to configure it. The
push was refused because it was a push to `main` at all.

**Nothing in `crates/alo-displays` is changed by this report.** Not one line of
the earlier worker's code, tests or report is altered, and the tree gated below
is byte-for-byte the tree that was refused — measured, not assumed.

## This is the second time, and that changes what is owed

One iteration earlier the same refusal stopped task 3, and its second worker
made the same repair this report makes:
`docs/autonomy/updates/publishing-a-persons-screens-through-a-task-branch.md`.
The loop then re-gated task 3, committed it again as `41e122c`, pushed, **and
was refused identically** — `1789773329` in the log — after which it went
straight on to task 4 and reached the same wall at `1789777285`.

So the repair below is necessary and is not sufficient, and saying only the
first half would be the more comfortable report to write. A task branch makes
this work *landable* by a person. It does not stop the loop walking into the
same refusal on task 5, because the loop's publisher still pushes `main`, and
that is now the thing blocking this plan rather than an item somebody might get
to. It has cost two finished, fully gated tasks their publication.

## What was done

### The gated commit is preserved by name

`1b497ed` was sitting on local `main`, where nothing could reach it and where
the next `git merge --ff-only origin/main` would have had to step over it. It is
now also a branch, named as SHARED_MAIN.md § *Task lifecycle* names one:

```text
task/dev-pc/night-light-and-display-colour → 1b497ed
```

That branch's parent is `41e122c`, task 3's commit, which is itself preserved on
`parked/task-3-1789773329`. That is deliberate and is the point of the next
section.

### Local `main` is a checkout that can take a handoff again

`git reset --mixed 41e122c`. A mixed reset does not touch the working tree:
every file the earlier worker wrote is still on disk exactly as they left it,
and the seventeen of them are once more changes against `main` — which is what
a handoff's file list describes, and what the loop's `stage()` and `commit()`
need to find. Nothing was discarded: the commit is on the branch above, and one
`git merge --ff-only task/dev-pc/night-light-and-display-colour` puts it back.

**The reset stops at `41e122c` rather than at `origin/main`, on purpose.** The
plan says task 4 **depends on 3**, and it is true in the code: this change
modifies `lib.rs`, `wearing.rs`, `changes.rs`, `keeping.rs`, `notes.rs` and
`words.rs`, all of which task 3 wrote. Resetting to `origin/main` would have put
both tasks' files in one change and one commit, which SHARED_MAIN.md forbids
(*Do not batch unrelated tasks into one branch*) and which would have produced a
commit message describing a third of what it contained. Local `main` at
`41e122c` is `main` as it will be once task 3 has landed, which is the only base
on which task 4 compiles.

That the reset lost nothing is measured rather than promised. Reading `41e122c`
into a scratch index, staging the handoff's seventeen files into it and writing
the tree gives

```text
staged-tree=ed08f46e0895aa5337f8ae32e3ba6478e4257024
commit-tree=ed08f46e0895aa5337f8ae32e3ba6478e4257024
```

— the same tree object as `1b497ed^{tree}`.

### The handoff names the whole change

`.kernel-loop/handoff.toml` is rewritten in full. Its file list is every file
now changed against `main` — eighteen: the earlier worker's seventeen, one of
which, the plan, this change edits again, plus this report. Its
evidence is the same six acceptance tests, each re-run on its own here, against
the committed tree.

## The decision, and what was not decided

**The fix is the route, not the publisher — and the publisher is now named as
what stops the next task.**

The obvious larger repair is `tools/kernel-loop`. `repository::pushed` is
`git push origin main`; the constant `MAIN` is the only branch it knows;
`repository::tests::the_only_branch_this_pushes_is_main` asserts that as a
property; `repository::rebased_onto_origin` assumes the base is `origin/main`;
and `publishing::Steps::push` documents its result as *the published sha*, which
for a task branch is no longer a sha on `main`. Changing that is a genuine piece
of work with its own gates, and it was not taken here, for three reasons.

SHARED_MAIN.md already says so: *Existing `kernel-loop` and `dev-loop`
publication commands still target `main`. Keep those publishers paused until
**separately adapted and verified** for this workflow.* The adaptation is
sanctioned and is explicitly separate work.

Second, it would not make task 4 land even if it were done. Protection requires
a pull request **and** the `alo/nine-gates` status on the head being merged, and
that status is set by whoever holds the integration turn after reviewing the
evidence (SHARED_MAIN.md § *Taking the integration turn*, step 6). A loop that
pushed a branch would still stop there, which is the correct place for it to
stop: a supervisor reporting its own green status on its own candidate and
merging it is the check being removed rather than satisfied.

Third, `tools/kernel-loop` is not this plan's to edit. The displays plan owns
`crates/alo-displays`; reaching into the supervisor's own source is the shape of
change `who_owns.rs` exists to refuse, and a rewrite of the one component whose
job is to be trustworthy about what it publishes is not a thing to do inside a
night-light task under a deadline.

**What that leaves, named plainly rather than proposed politely:** a
`tools/kernel-loop` task — *the loop publishes by pushing `main`, which
protection refuses; adapt it to push its task branch and stop at the pull
request, or retire its push step in favour of the manual integration turn.*
Until one of those happens, every task this loop finishes reaches the same
refusal at the same step, and it has already happened twice. It is a
`tools/kernel-loop` task and this report does not write it into a plan it does
not own.

## How to publish this from here

For whoever holds the integration turn. Nothing below is run by this worker: a
worker does not commit and does not push. **Task 3 lands first**; task 4 does
not compile without it.

1. Task 3, as its own report describes: a pull request from a branch carrying
   `41e122c`, gated, merged, deleted.
2. `git push origin task/dev-pc/night-light-and-display-colour` — after the loop
   has committed this report into that branch, so the branch carries all
   eighteen files rather than seventeen.
3. Open one pull request to `main`, using the handoff's subject and body. Where
   task 3 has not yet merged, open it against task 3's branch and retarget it, or
   wait; do not squash the two together.
4. Claim `refs/heads/coordination/integration-lock` at the candidate head, fetch
   `main`, integrate it, run all nine gates on the combined tree, and report
   `alo/nine-gates` on that exact head.
5. Squash-merge, delete the branch in the same turn, then fast-forward this
   checkout's `main`.

## Verification

**Platform:** Windows Server 2022 host, gates in WSL (Ubuntu). The source gated
is **the committed tree, not the working tree**, as SHARED_MAIN.md § *Gate the
committed tree* requires: `git archive 1b497ed` was extracted to
`/root/alo-committed` and rsynced into `/root/alo-trees/this-machine`, this
machine's one serialized gate tree, with `CARGO_TARGET_DIR` at
`/root/alo-builds/this-machine`, its one build directory. No other cargo process
was running; C: had 31.6 GiB free and the Linux filesystem 919 GiB.

```text
commit 1b497ed0e92013ae17f4e374458ce81ce6accb34
tree   ed08f46e0895aa5337f8ae32e3ba6478e4257024
```

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all` (in WSL, on `/mnt/c/dev/alo-os`) | exit 0, `git status` reported no change — already formatted |
| Lints, whole workspace | `cargo clippy --all-targets -- -D warnings` | exit 0, zero warnings, 39 s |
| Rustdoc, the crates touched | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-displays -p alo-saying --no-deps` | exit 0 |
| Tests, `alo-displays` | `cargo test -p alo-displays` | exit 0 — 104 lib, 6 + 2 + 4 + 4 + 4 + 2 integration and doc |
| Tests, `alo-saying` | `cargo test -p alo-saying` | exit 0 — 63 lib, 4 integration, 1 doc-test; it collects this crate's words |
| Each acceptance test alone | six runs, below | all six exit 0, one passing test each |

The six, each run on its own with `--exact` and each reporting
`1 passed; … filtered out`:

| Clause | Command |
|---|---|
| a schedule a person sets | `cargo test -p alo-displays --lib -- --exact nightly::tests::a_schedule_holds_the_hours_it_was_given_and_no_others` |
| sunset to sunrise computed on the machine | `cargo test -p alo-displays --lib -- --exact nightly::tests::the_sun_turns_the_screens_warm_when_it_actually_sets` |
| never a location service or a network lookup | `cargo test -p alo-displays --test the_sun_is_worked_out_on_this_machine -- --exact nothing_here_can_reach_a_network` |
| a colour temperature within a closed range | `cargo test -p alo-displays --lib -- --exact warmth::tests::a_warmth_no_screen_can_be_drawn_at_is_refused_and_names_the_range` |
| applied per display | `cargo test -p alo-displays --lib -- --exact wearing::tests::every_screen_wears_the_warmth_beside_its_own_background` |
| terracotta still means the agent | `cargo test -p alo-displays --test terracotta_still_means_the_agent_under_night_light -- --exact the_agents_colour_stays_apart_from_every_accent_at_every_warmth` |

Every row above was then **run a second time**, after this report and the plan's
status line were written, against the working tree exactly as it is handed over
— `cargo fmt --all` (clean, no file changed), workspace clippy, rustdoc, both
crate suites and all six acceptance tests, every one exit 0. The two passes
differ only by two markdown files, which no test in `alo-displays` or
`alo-saying` reads, and the second is there so that *the gates passed* describes
the tree in the handoff rather than an earlier one.

The whole-workspace suite was **not** run here, by instruction: the supervisor
runs it, and finished tasks have already been lost to a worker waiting on it.

Nothing on certified hardware: `alo-displays` opens no device, sets no colour
ramp and speaks no protocol, and nothing in this report changes that. The shell
applies the warmth per output in the shell plan's task 9, and that is where a
screen first actually goes orange.

## Limitations

- This report does not make the work land. It makes it *landable*: the branch
  exists locally and has not been pushed, and the pull request has not been
  opened, because publication is the supervisor's and a worker that pushes is a
  worker that published something nobody gated.
- It does not stop the next task meeting the same refusal. See **The decision**:
  that is a `tools/kernel-loop` task, it is named there, and it is now blocking.
- Everything the earlier worker recorded as owed still stands unchanged:
  `docs/contracts/person-settings.md` describes neither `sleeping.toml` nor
  `displays.toml`, and `docs/features.md`'s *Per display, so the dock can sit
  along the bottom of the laptop and down the side of the external screen* is
  still not met because `alo_dock::Dock` holds one edge for the machine.
- Local `main` in this checkout now sits at `41e122c`, which `origin/main` does
  not have. That is not a mistake to be tidied by a fast-forward: it is task 3,
  unpublished, and task 4 is written against it. Both are on branches, and
  neither is to be reset away.

## Proposed shared-document updates

Proposals for the integration owner. This report does not edit `CHANGELOG.md`,
`ROADMAP.md`, `docs/autonomy/QUEUE.md` or `docs/autonomy/STATE.md`.

**`CHANGELOG.md`:** nothing beyond what
`docs/autonomy/updates/night-light-and-display-colour.md` already proposes. This
change adds no behaviour a person outside this repository can see.

**`ROADMAP.md`:** unchanged by this report.

**`docs/autonomy/QUEUE.md`:** the `tools/kernel-loop` item task 3's report asked
for is owed a second time and should be raised from *owed* to *blocking* —
*`alo-kernel-loop publish` pushes `main`, which protection refuses; adapt it to
push its task branch and stop at the pull request, or retire its push step.* Two
fully gated tasks have now been refused at that step. The reasoning is in **The
decision** above.

**`docs/autonomy/STATE.md`:** reference this report's path beside task 4's own,
and record that task 4's publication is a pull request from
`task/dev-pc/night-light-and-display-colour`, based on task 3's commit and
following task 3's pull request.
