# User-owned model choice, independent of privacy guarantees

Date: 2026-09-08. Workstream: owner direction and integration documentation.
Status: documentation clarification ready for integration; no runtime change.

## Owner direction

The owner reaffirmed an open choice of alo-provided models/services, users' own
local models/runtimes and third-party APIs. This is not an alo-only ecosystem.
`docs/features.md` now distinguishes model ownership from processing location
and from any enforceable privacy guarantee. Existing paired-machine and no-agent
configurations are preserved. No existing release tier or completion box moves.

Third-party ownership alone must not disqualify a genuinely local runtime from
an established local-only guarantee; alo ownership alone cannot establish one.
Compatible protocol support, typed capabilities and accepted organisation policy
remain relevant. No claim is made that every model or API already works.

## Decisions deliberately not taken

ADR 0021 remains PROPOSED. The owner's statement about model freedom is not
approval of Option A, B, C, a new kernel hook or programme type, or a changed
`ThisMachineOnly` allow/refuse rule. Neither a one-time warning nor a truthful
answer label is evidence that a service cannot forward a question. The egress
promise, ADR 0020, grants and current enforcement remain unchanged.

Claude's next handoff, provided in the conversation rather than another prompt
file, is to revise the proposal around this distinction, identify meaningful
missing behavioral tests, and present the precise remaining policy decision.
Any new local-only guarantee needs an implementation and measured acceptance,
including relevant failure and bypass cases, before it is offered as verified.

## Verification and coordination

Documentation-only: inspected the feature tiers, existing model-choice text,
proposed ADR 0021 and the diff; `git diff --check` passed. No Rust or hardware
test result is claimed for this clarification. No acceptance test was invented
to certify prose. Only this report and `docs/features.md` changed.

Used the clean integration helper checkout while the desktop worker continued
in `C:\dev\alo-os`; no edits to that worker's tree, Claude's tree, or the four
shared progress documents. Publish normally to main. The desktop supervisor
integrates incoming main at its publication boundary and reconciles this report
on its next iteration. No worker or network configuration was changed.

## Proposed progress-document integration

Record this owner clarification in STATE. Keep existing model/provider choices,
release tiers and outstanding implementation work in QUEUE and ROADMAP. Keep
the ADR 0021 policy decision pending. No user-visible behavior change or release
acceptance is implied, so no functional completion announcement is warranted.
