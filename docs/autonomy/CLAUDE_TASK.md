# Claude Code task prompt

Copy the following prompt into Claude Code. This handoff reserves item 6b; it
does not start a Claude session or claim that the task is already underway.

```text
Work on alo OS: https://github.com/aloworld-org/alo-os

Use your own checkout at C:\dev\alo-os-claude, outside OneDrive. Clone it if
absent; otherwise inspect its remote, branch and working tree first. Never edit
C:\dev\alo-os or C:\dev\alo-os-loop-update: another worker owns those.

Your assignment is queue item 6b: "Opening from a handle, and renaming without
replacing" in crates/alo-files. The other worker owns graphics/compositor
development. Complete this assignment end to end; do not take unrelated tasks.

First read CLAUDE.md, any applicable AGENTS.md, docs/features.md, ROADMAP.md,
docs/autonomy/DELIVERY.md, docs/autonomy/SHARED_MAIN.md, queue item 6b, relevant
ADRs, contracts and docs/quirks.md.

Implement:
- Linux directory-handle-relative filesystem operations that close the
  documented path-check/open race, including intermediate path components.
- Atomic no-clobber rename/move using the appropriate Linux primitive.
- Reuse the repository's pinned rustix approach; preserve the unsafe-code ban.
- Preserve capability checks, approval semantics, records, public contracts and
  portable behavior. Fail safely when guarantees cannot be provided.
- Linux integration tests for symlink substitution, attempts to escape grants,
  destination collisions and preservation of existing data, plus happy paths
  and error reporting. Use deterministic race tests where feasible.

Run formatting, clippy with warnings denied, relevant tests, workspace regression
tests and documentation checks. Linux-specific code must compile and run on
Linux; Windows success alone is insufficient. Use a separate Linux build
directory. Do not alter shared WSL services, kernel/BPF state or stop another
worker's processes without coordination. Report physical hardware acceptance as
outstanding unless actually verified.

Update CHANGELOG.md, ROADMAP.md, docs/autonomy/QUEUE.md, docs/autonomy/STATE.md
and affected contracts/quirks in the same task. Never weaken gates or mark
unfinished work complete.

Publish directly to main:
1. Before each task, start clean on main and git pull --ff-only origin main.
2. Implement and test the complete task.
3. Commit only your intended changes with a conventional message and the
   repository owner's configured identity. No author overrides or
   Co-Authored-By trailers.
4. Fetch origin/main again. If it advanced, rebase only your unpublished commits
   onto it, preserve both contributors' changes, and rerun the required checks
   on the combined tree.
5. Push normally to main. If another push wins the race, integrate and retest
   before retrying. Never force-push, reset away work or overwrite another
   contributor's changes.
6. If a conflict needs an architectural decision, a check fails or publication
   remains blocked, preserve your work and report the exact blocker.

At handoff, report the pushed SHA, implementation summary, actual test results
and remaining limitations. Do not claim the whole OS is finished because this
task passes.
```
