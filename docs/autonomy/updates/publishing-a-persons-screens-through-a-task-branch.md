# Publishing *A person's screens* through a task branch

**Date:** 2026-09-18
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 3
(*A person's screens: which, where, how large, and remembered*). A follow-up to
`docs/autonomy/updates/a-persons-screens-which-where-how-large-and-remembered.md`,
which is that task's own report and is not edited here.
**Contributor:** second Claude Code worker on task 3, in `C:\dev\alo-os`.
**Status:** ready for integration, through a pull request and not through a push
to `main`.

## Why there is a second report on a finished task

Task 3's code was written, gated and committed. It was then refused at the last
step of publication, and the refusal was not about the work:

```text
$ git push origin main
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote: - Changes must be made through a pull request.
remote: - Required status check "alo/nine-gates" is expected.
 ! [remote rejected] main -> main (protected branch hook declined)
```

`origin/main` had not moved, so this was not a lost race. `main` is protected
exactly as `docs/autonomy/SHARED_MAIN.md` § *Main protection* says to configure
it: a pull request is required, the `alo/nine-gates` status is required, and
administrators are included. The push was refused because it was a push to
`main` at all.

So nothing in `crates/alo-displays` is changed by this report. **Not one line of
the earlier worker's code, tests or report is altered**, and the gated tree is
byte-for-byte the tree that was refused — checked, not assumed, below. What is
changed is the road the work takes to `main`.

## What was done

### The gated commit is preserved by name

The refused commit, `589d14be`, was sitting on local `main` where nothing could
reach it and where the next `git merge --ff-only origin/main` would have had to
step over it. It is now also a branch, named the way SHARED_MAIN.md § *Task
lifecycle* names one:

```text
task/dev-pc/a-persons-screens-which-where-how-large-and-remembered → 589d14be
```

### Local `main` is a checkout that can take a handoff again

`git reset --mixed origin/main`. The working tree is not touched by a mixed
reset: every file the earlier worker wrote is still on disk exactly as they left
it, and the 26 of them are once more changes against `main` — which is what a
handoff's file list describes, and what the loop's `stage()` and `commit()` need
to find. Nothing was discarded: the commit is on the branch above, and one
`git merge --ff-only task/dev-pc/a-persons-screens-which-where-how-large-and-remembered`
puts it back on `main` unchanged if a person prefers it there.

That the reset lost nothing is measured rather than promised. Staging the
handoff's file list into a scratch index and writing the tree gives

```text
staged-tree=003f1da4168a91214025cea562964af6e5634915
commit-tree=003f1da4168a91214025cea562964af6e5634915
```

— the same tree object as `589d14be^{tree}`.

### The handoff names the whole change

`.kernel-loop/handoff.toml` is rewritten in full. Its file list is every file
now changed against `main`: the earlier worker's 26, plus this report. Its
evidence is the same eighteen acceptance tests, each re-run on its own here.

## The decision, and what was not decided

**The fix is the route, not the publisher.** The obvious larger repair is to
teach `tools/kernel-loop/src/publishing.rs` to push a task branch and open a pull
request instead of pushing `main`. That was considered and not taken, for two
reasons that both point the same way.

SHARED_MAIN.md already answers it: *Existing `kernel-loop` and `dev-loop`
publication commands still target `main`. **Keep those publishers paused** until
separately adapted and verified for this workflow.* The publisher is not broken
in a way this task may quietly patch; it is a component the workflow document has
already stood down, and adapting it is its own task with its own gates — it
touches `publishing.rs`, `repository.rs`, the GitHub reference API used for the
integration lock, and every test in `tools/kernel-loop` that asserts the order of
steps. Second, `tools/kernel-loop` is not this plan's to edit. The displays plan
owns `crates/alo-displays`; reaching into the supervisor's own source to change
how it publishes is the shape of change `who_owns.rs` exists to refuse.

The smallest change that makes the push succeed is therefore to publish the way
the repository now requires: a task branch, a pull request, the `alo/nine-gates`
status set by the integration-turn holder, and a squash merge. That is steps 3
through 8 of SHARED_MAIN.md § *Task lifecycle*, and it needs no code.

**Work this leaves named for somebody:** adapting `alo-kernel-loop publish` to
the task-branch workflow, or retiring its push step in favour of the manual
integration turn. Until one of those happens, every task this loop finishes will
reach the same refusal at the same step. That is a `tools/kernel-loop` task and
this report does not write it into a plan it does not own.

## How to publish this from here

For whoever holds the integration turn. Nothing below is run by this worker: a
worker does not commit and does not push.

1. `git push origin task/dev-pc/a-persons-screens-which-where-how-large-and-remembered`
   — after the loop has committed this report into that branch, so the branch
   carries all 27 files rather than 26.
2. Open one pull request to `main`, using the handoff's subject and body.
3. Claim `refs/heads/coordination/integration-lock` at the candidate head, fetch
   `main`, integrate it, run all nine gates on the combined tree, and report
   `alo/nine-gates` on that exact head.
4. Squash-merge, then delete the branch in the same turn, then fast-forward this
   checkout's `main`.

## Verification

**Platform:** Windows Server 2022 host, gates in WSL Ubuntu. The source gated is
**the committed tree, not the working tree**, as SHARED_MAIN.md § *Gate the
committed tree* requires: `git archive 589d14be` was extracted to
`/root/alo-committed` and rsynced into `/root/alo-trees/this-machine`, and a
`find` of both trees was diffed to confirm they held the same files.
`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, the one build directory this
machine shares. No other cargo process was running and C: had 31.6 GiB free.

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all` (in WSL, on `/mnt/c/dev/alo-os`) | exit 0, and `git status` reported no change — the tree was already formatted |
| Lints, the crates touched | `cargo clippy --all-targets -p alo-displays -p alo-saying -- -D warnings` | exit 0 |
| Lints, the whole workspace | `cargo clippy --all-targets -- -D warnings` | exit 0, 2m 26s |
| Tests, `alo-displays` | `cargo test -p alo-displays` | exit 0 — 60 lib, 6 + 2 + 4 integration, 1 doc-test |
| Tests, `alo-saying` | `cargo test -p alo-saying` | exit 0 — 63 lib, 4 integration, 1 doc-test |
| Each acceptance test alone | `cargo test -p alo-displays --lib -- --exact <name>` and `--test <target> -- --exact <name>`, eighteen runs | all 18 passed, one test each |

The whole-workspace suite was **not** run here, by instruction: the supervisor
runs it, and two finished tasks have already been lost to a worker waiting on it.
Nothing on certified hardware: `alo-displays` opens no device and sets no mode,
and nothing in this report changes that.

## Limitations

- This report does not make the work land. It makes it *landable*: the branch
  exists locally and has not been pushed, and the pull request has not been
  opened, because publication is the supervisor's and a worker that pushes is a
  worker that published something nobody gated.
- The finding the earlier worker recorded stands unchanged: `docs/features.md`'s
  v0.5 line *Per display, so the dock can sit along the bottom of the laptop and
  down the side of the external screen* is **not met**, because `alo_dock::Dock`
  holds one edge for the machine. It waits on `alo-dock`, and `Wearing::of` is
  the one function that changes when that crate decides otherwise.
- The sentence that report leaves for the access plan's owner also stands:
  `crates/alo-access/tests/every_surface_the_shell_draws_is_read_aloud.rs` reads
  `crates/alo-shell/src/lib.rs` from disk while it runs, and `include_str!`
  removes the race between lanes sharing one gate tree.

## Proposed shared-document updates

Proposals for the integration owner. This report does not edit `CHANGELOG.md`,
`ROADMAP.md`, `docs/autonomy/QUEUE.md` or `docs/autonomy/STATE.md`.

**`CHANGELOG.md`:** nothing beyond what
`docs/autonomy/updates/a-persons-screens-which-where-how-large-and-remembered.md`
already proposes. This change adds no behaviour a person outside this repository
can see.

**`ROADMAP.md`:** unchanged by this report.

**`docs/autonomy/QUEUE.md`:** a `tools/kernel-loop` item is owed —
*`alo-kernel-loop publish` pushes `main`, which protection now refuses; adapt it
to the task-branch workflow or retire its push step.* Naming it is the queue
owner's; the reasoning is in **The decision** above.

**`docs/autonomy/STATE.md`:** reference this report's path beside task 3's own,
and record that task 3's publication is a pull request from
`task/dev-pc/a-persons-screens-which-where-how-large-and-remembered`.
