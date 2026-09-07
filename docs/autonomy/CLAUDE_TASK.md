# Claude Code task prompt

Copy this workflow update into the existing Claude Code session. Secure file
opening and handle-based file moves are already published; this is not a new
assignment to repeat them.

```text
Work on alo OS: https://github.com/aloworld-org/alo-os

Use your own checkout at C:\dev\alo-os-claude, outside OneDrive. Clone it if
absent; otherwise inspect its remote, branch and working tree first. Never edit
C:\dev\alo-os or C:\dev\alo-os-loop-update: another worker owns those.

Your workstream is filesystem security in crates/alo-files. The other worker
owns the native desktop compositor and shared release-progress documents.
Confirm your next assignment with the owner; do not repeat published work.
Use descriptive task titles, filenames and status messages, not queue codes.

First read CLAUDE.md, any applicable AGENTS.md, docs/features.md, ROADMAP.md,
docs/autonomy/DELIVERY.md, docs/autonomy/SHARED_MAIN.md, your assigned task, relevant
ADRs, contracts and docs/quirks.md.

Preserve the unsafe-code prohibition, capability checks, approval semantics,
records, public contracts and portable behavior. Test both success and refusal
paths. Changes to accepted security decisions require approval before proceeding.

Run formatting, clippy with warnings denied, relevant tests, workspace regression
tests and documentation checks. Linux-specific code must compile and run on
Linux; Windows success alone is insufficient. Use a separate Linux build
directory. Do not alter shared WSL services, kernel/BPF state or stop another
worker's processes without coordination. Report physical hardware acceptance as
outstanding unless actually verified.

Do not edit CHANGELOG.md, ROADMAP.md, docs/autonomy/QUEUE.md or
docs/autonomy/STATE.md. Instead, add your own uniquely named task report under
docs/autonomy/updates/, following its README. Use a descriptive filename such as
secure-file-moves.md. Include the user-readable change description, exact test
results, decisions, remaining limitations and proposed shared-document updates.
The integration worker consolidates the report. Update code-local documentation
and affected contracts in the same task; coordinate shared specification edits.
Never weaken gates or mark unfinished work complete.

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
