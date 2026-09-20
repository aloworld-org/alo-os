# Finishing the offer after `main` moved

**Date:** 2026-09-19
**Workstream:** `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, task 7
**Contributor:** this development PC's worker lane, second worker
**Status:** ready for integration

A follow-up to `docs/autonomy/updates/an-offer-a-person-can-act-on.md`, which is
this task's report and stands as its author wrote it. Nothing in it was edited
and nothing it claims was changed: every measurement in it was re-run here on a
different base and came back the same. This report is only about why the gates
refused that work, what the refusal actually was, and what re-gating it on the
new `main` showed.

## What the gates refused, and what it was not

Not the code. `git rebase origin/main` stopped on the first of three commits
with add/add conflicts in four files and a content conflict in the plan:

```
CONFLICT (add/add): crates/alo-looking/src/looking.rs
CONFLICT (add/add): crates/alo-looking/src/place.rs
CONFLICT (add/add): crates/alo-looking/src/testing.rs
CONFLICT (add/add): docs/autonomy/updates/finding-out-there-is-an-update.md
CONFLICT (content): docs/autonomy/v0-5-the-machine-keeps-itself-plan.md
```

Every one of those files belongs to **task 6**, not task 7. Local `main` had
been at `325a9ed` and carried three commits: `6f61c2a` and `9f7caf3`, which are
task 6, and `643e54b`, which is task 7. While task 7 was being written, task 6
landed on `origin/main` on its own as the squash-merge `087fef4` (PR #45), and
`origin/main` moved on twice more (`b41b4b5`, `9fcc99b`).

So the rebase was replaying task 6 onto a `main` that already had it. A file
created by both sides is an add/add conflict even when the two sides are
identical, which is why five files that nobody had touched twice came back as
five conflicts. The one commit this task is about applied with none.

**This is worth naming because it will happen again.** A lane that builds task
*n+1* on top of an unmerged task *n* holds *n* twice the moment *n* is
squash-merged: once as its own commits and once inside the squash. The squash
has a different SHA and no ancestry link, so nothing tells git the two are the
same work.

## How it was resolved

By checking rather than by choosing a side. Before anything was changed, the
pre-rebase head was kept on a branch (`backup/pre-rebase-task7`, `643e54b`) so
the original three commits remain reachable whatever happens next.

Then the question was asked directly — does `main` already contain all of task
6?

```
git diff --stat 9f7caf3 origin/main -- crates/alo-looking crates/alo-saying \
    docs/autonomy/updates/finding-out-there-is-an-update.md \
    docs/autonomy/v0-5-the-machine-keeps-itself-plan.md
(nothing)

git diff --stat 9f7caf3 origin/main -- Cargo.toml
 Cargo.toml | 4 ++++
```

Nothing, and four added lines in `Cargo.toml` that came from the two *other*
merges — a member entry each. Task 6's content on `main` is byte for byte what
the branch held. So the two commits were skipped, which is the resolution for a
patch that is already upstream rather than a discarding of anything: git
recognised the second one itself and dropped it with *patch contents already
upstream*. `643e54b` then replayed clean as `7d72c0e`.

The result is one commit on top of `origin/main`, holding task 7 and nothing
else, and no conflict marker survives anywhere in `docs/` or `crates/`. The
working tree carried nothing beyond that commit when the gates were run, except
the two things this second pass adds and the handoff names: the `Cargo.lock`
line below, and this report.

| | |
|---|---|
| base (`origin/main`) | `9fcc99bd0c432b789de860a216ba4fc520404e4c` |
| candidate head | `7d72c0ed6ef2fc277f613e566ecc5242291fd504` |
| candidate tree | `dbf5a201860d73912b72f44cca0b9a388f3849f4` |
| kept for recovery | `backup/pre-rebase-task7` → `643e54b` |

## Re-gated on the new base

The first worker's numbers were measured against `325a9ed`. These were measured
against `9fcc99b`, which is three merges further on, from the Linux copy the
gates read (`/root/alo-trees/this-machine`, Ubuntu 24.04 under WSL2 on Windows
Server 2022), synchronized from the checkout first.

| What | Result |
|---|---|
| `cargo fmt --all --check` | clean, 7 s |
| `cargo clippy -p alo-keeping-up -p alo-looking -p alo-updating -p alo-saying --all-targets -- -D warnings` | clean, 18 s |
| `cargo doc -p alo-keeping-up -p alo-looking -p alo-updating -p alo-saying --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean, 8 s |
| `cargo test -p alo-keeping-up` | passed, 9 s |
| `cargo test -p alo-looking` | passed, 4 s |
| `cargo test -p alo-updating` | passed, 13 s |
| `cargo test -p alo-saying` | passed |
| `cargo test -p alo-citing` | passed — the citation check, because this change touches `docs/` |
| `cargo test` in `tools/kernel-loop` | 151 passed — the plan checks, because this change edits a plan |
| each of the twelve evidence tests, on its own, `--exact --include-ignored` | one test passed, twelve times |
| `cargo build -p alo-updating --locked` | clean, once `Cargo.lock` was brought back into agreement with the manifest |

**The whole workspace suite was not run here**; the supervisor runs it after
this. `alo-saying` and `alo-citing` are in the list because they are what this
change can reach that is not its own: `alo-saying` collects the two new
sentences, and the citation check reads the ADR links the new documentation
adds.

**The two measurements against the real world were re-run rather than quoted**,
because a report's number is worth what its last run is worth:

- `alo-looking`, `against_the_real_registry`,
  `what_this_machine_is_offered_today_and_what_it_is_told_about_it`, 2.1 s. The
  place answers seven names; the newest release is `0.0.4` at
  `sha256:48bd5f31…`; `vouched_for` reads `NobodyHasVouchedForIt`, and a person
  reads *A new version of this machine's system is available, but this machine
  cannot confirm that it came from alo OS.*
- `alo-updating`, `an_offer_a_person_can_act_on`,
  `what_the_real_base_says_about_the_real_places_builds`, 3.7 s. The base's own
  `skopeo` under a policy requiring `image/signing/alo-os.pub` refused both
  `0.0.4` and the pinned `0.0.3` with *Source image rejected: A signature was
  required, but no signature exists*, before any layer was fetched.

Both are the same answers the first worker recorded, which is the point: the
finding handed to the installer lane is not an artefact of the tree it was
measured in. It is still true today, and no alo OS release published so far can
be staged under `--enforce-container-sigpolicy`.

## One thing that was actually missing

`Cargo.lock`. Task 7 gave `alo-updating` a dependency on `alo-image` in its
`Cargo.toml` and the lock file was never committed with it, so the tree carried
a manifest and a lock that disagreed. Nothing on the branch noticed, because
every build there resolved the dependency and rewrote the lock in the working
tree without being asked; a build with `--locked`, and any gate run against
`git archive` of the commit rather than against a working tree, would have. It
is one line — `alo-image` under `alo-updating`'s dependencies — and it is in
this change.

This is the only code-shaped defect the second pass found. It is worth naming
because of how it hides: a lock file that a local build silently repairs looks
correct to whoever is building, and only looks wrong to whoever checks out the
commit.

## Two findings for whoever comes next

**`cargo fmt --all` cannot be run on the Windows side of this machine.** It
fails with *The filename or extension is too long. (os error 206)* — 95
workspace members is more than a Windows command line holds — and prints
rustfmt's usage, which reads like a syntax error in the invocation rather than
a limit of the host. A worker that stops there will report a tree as
unformattable when it is fine. Two roads work: run it in WSL, where it is the
gate's own command and takes 7 s, or run `cargo fmt -p <crate>` in batches,
which is what `--all` expands to anyway. `cargo clippy` on the Windows side is
no use either — the host toolchain is `x86_64-pc-windows-gnu` with no `gcc`, so
`ring` cannot build. **The gates belong in WSL on this machine**, which is
where `tools/kernel-loop/src/gates.rs` already puts them; running them on the
Windows side is not a stricter check, it is a broken one.

**A stale blocker, reported rather than edited.**
`docs/autonomy/v0-5-the-shell-plan.md` task 13 (*The recovery and rollback
screen*) reads *blocked — on `v0-5-the-machine-keeps-itself-plan.md` task 3*,
and that task has been done since 2026-09-15. It is takeable and every lane
surveying for free work is stepping over it. It is left for the shell plan's
owner rather than corrected here: it names task 3, not the task this change
finishes, and editing another lane's plan from this branch would widen the diff
into a file another branch is likely holding.

## Proposed changes to the shared documents

The integration owner's to make; nothing here edits them. The proposals in
`an-offer-a-person-can-act-on.md` stand unchanged — the `CHANGELOG.md` entry,
the `ROADMAP.md` note that v0.5 cannot be called complete while no published
release can be staged, and the `QUEUE.md`/`STATE.md` lines for task 7 done and
task 8 blocked. Add to `STATE.md` a reference to this report beside that one, so
the pair reads as what it is: one task, two workers, and a rebase that was about
task 6.
