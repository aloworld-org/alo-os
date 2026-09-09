# Concurrent development with shared kernel tests

2026-09-09. Owner-approved replacement of the temporary single-workstream rule.
Desktop integration owner; Claude's checkout and running supervisor are untouched.

## Independent work, serialized shared fixtures

Both workers can develop and compile in their own checkouts. Keep their respective
Linux targets `/root/alo-os-target` and `/root/target-claude`. The 12 GiB Windows
host-space reserve remains; separate targets are not a disk quota or memory limit.
Do not delete another worker's cache or stop its processes to make room.

No new test lock was needed. Inspected `alo-bounding/src/waiting.rs`, the shared
bounding fixture, both daemon kernel fixtures and boundary-loader fixture: they
already use `Waited::on_this_kernel()` and the same abstract socket name
`alo-os/one-kernel-at-a-time`. It is held for one fixture, including its children,
and released by closing the socket or process exit. Timeout fails; no bypass.
An outer lock on the same name would deadlock tests trying to acquire it again.
The existing cross-process tests cover contention, release and holder death.

Private fixture resources may overlap. Shared service/package/mount/session
maintenance outside those fixtures still needs a coordinated idle handoff. This
rule is cooperative, not an OS sandbox or enforcement over arbitrary commands.
Existing sessions must reread the updated rules; this report does not silently
change Claude's saved instructions or restart its supervisor.

## Desktop supervisor change

`linux_gate.rs` now gives each existing Linux phase its own `checked` invocation.
That reuses the existing disk preflight before every phase instead of only once
before a potentially long combined shell command. Commands, warnings policy,
workspace coverage and pinned BPF checks remain intact. The initial bpffs check
now refuses a missing mount rather than mounting shared state automatically.
It names the maintenance handoff on failure. No WSL/service restart is added.

Updated `WORKER.md`, `DELIVERY.md`, `SHARED_MAIN.md` and current shared progress
entries. Rebuilt the desktop executable so it embeds the new worker instructions.

## Evidence so far

Windows supervisor fmt, all-target clippy with warnings denied, all 17 tests and
release build passed. Two new tests exercise all phase invocations and failure
at every position, proving no later phase is called after a refusal. Existing
disk-threshold, process failure, single-supervisor and publication-race tests pass.
No test weakens a production or publication gate.

The pending painter and smaller profiles are preserved. Fast-forwarded from
`4077d9e` to Claude's `8096910`; its five files did not overlap the dirty work.
Reviewed `updates/a-real-keyring-answers.md` for integration: contributor reports
12 secret-store checks including real DH retrieval, Missing and schema isolation;
the real-bus-with-no-service, locked/denied, daemon/HTTPS and lifetime/concurrency
continuations remain unfinished. No service/package change from that report was
performed by the desktop worker. No tier or release checkbox moved.

Full combined-tree Windows/Linux/kernel and painter graphics gates passed at
`8096910`, including 128 full-frame graphics comparisons. No gate exemption or
new ignored test was added. Claude then published `9109875`; it was integrated
by another non-overlapping fast-forward and the full combined-tree re-gate started.

Reconciled `updates/the-four-refusals-against-a-real-store.md` precisely: real
Locked and Denied fixtures plus error-name classification, not proof of all four
states against a real store. Unavailable still has socket-shaped coverage, and
the no-service-on-a-live-bus path remains unproven. The refusal type's no-send
assertion is not authenticated HTTPS or production daemon no-send evidence.
Those integrations, lifetime/concurrency and logout remain open. No report from
Claude was rewritten, no shared host configuration changed, no release tick moved.
Publication and desktop restart wait for the new combined-tree gates.

Those complete `9109875` gates passed, including graphics. A second incoming
publication, `c4e20c7`, was then fast-forwarded without overlap and a third complete
combined-tree run started (`integrated-c4-*` logs). Reviewed its
`updates/a-bus-with-nothing-on-it.md`: a fixture starts a live bus without starting
a keyring, proves bus liveness and name absence, then asserts Unavailable promptly.
This supersedes the previously open empty-bus case; it does not complete daemon
credential wiring, HTTPS, lifetime/concurrency or logout. No new release claim.
The owner was asked to coordinate a temporary publication hold with Claude (not
a development/test pause), to let this integration finish without repeated races.

## Final gate result

The complete final tree with `c4e20c7` passed Windows workspace fmt/clippy/tests,
Linux workspace fmt/clippy/tests/rustdoc, pinned BPF fmt/clippy and all 128 graphics
frames. Existing kernel-lock cross-process tests passed within those suites.
The rebuilt desktop supervisor's own 17 tests, lint and format checks also passed.
All commands exited successfully; no assertion, gate or warning policy weakened.
The last fetch found no further main advance. Normal publication and clean-tree
desktop restart are now eligible. Claude can continue its own loop under the
updated rules; no second process is launched into its checkout by this work.
