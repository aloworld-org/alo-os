# Resume one desktop loop with a host-disk reserve

Date: 2026-09-08. Owner-authorized restart after build-cache cleanup.
Workstream: development supervisor, not the shipped OS.

The owner reserved Windows.old and company-managed system cleanup for the
company administrator, and requested one development workstream at a time.
The desktop checkout was left with an interrupted client-maximize task; that
work must be verified and published before the normal clean-tree loop restarts.
No dirty-tree gate is bypassed and no unfinished edits are discarded.

`tools/dev-loop/src/storage.rs` reads actual available bytes on Windows C:
through a read-only PowerShell DriveInfo query. Before selecting another task
and before each top-level publication-gate command, less than 12 GiB or an
unavailable/malformed reading refuses work. The supervisor records the failure
through its existing HALTED path and preserves source and unpublished commits.
The Windows reserve also matters to WSL because its backing disk is on C:.

This is a conservative operating reserve, not an OS minimum or a hard quota.
There is no live cancellation of a command that consumes space after preflight;
workers must check before their own focused builds/tests. The Linux gate is
one top-level command containing multiple checks. Existing stop, locking,
review, integration, re-gating and no-force-push rules are unchanged.

`WORKER.md` and `DELIVERY.md` record one workstream until explicit handoff and
the exclusion of company/system/personal/credential cleanup. No automatic file
cleanup, service changes, WSL shutdown or storage migration is implemented.
Claude's prompt is supplied in chat, not saved as a repository file. This rule
coordinates the other checkout by instruction; it is not a cross-tool build lock.

Verification: supervisor all-target clippy with warnings denied, all 15 tests
and release rebuild passed on Windows. Four new tests cover the exact reserve,
above/below it, zero, malformed/absent/overflow readings and CRLF output. The
existing process-boundary test also exercises the real query before its checked
command. Low space is tested with values, not by filling the disk. No claim that
query subprocess failure was induced on the host. Final diff/fmt checks precede
publication. The rebuilt executable embeds the current worker instructions.
