# Approval for bounded provider requests

- Date: 2026-09-07
- Workstream: kernel and provider-request integration
- Contributor: interactive maintainer recording the repository owner's decision
- Status: ready for integration; approval recorded, implementation not verified

## Owner decision

The owner explicitly approved the scoped request-boundary and connection-lifecycle
change after reviewing the finding that provider questions do not currently enter
the file-operation boundary. This records authority to implement the change, not
evidence that the network release requirement is complete.

## Approved implementation scope

- Run provider requests inside a network-aware turn boundary without inventing
  filesystem grants or widening existing grants.
- Extend the lifecycle and public interfaces as needed for that request scope;
  document the decision in an ADR before implementing it and preserve existing
  contracts through additive/versioned changes as required by repository rules.
- Separate address resolution from connection establishment. Connect only to
  registered destinations while retaining original-hostname TLS verification.
- Define DNS access explicitly, without a blanket network exception.
- Prevent connections from surviving or being reused beyond their authorized
  request lifetime. Cover success, error, cancellation and teardown.
- Preserve provider/region policy, credentials handling, indicator and record
  behavior, local-model operation and the no-silent-fallback guarantee.
- Test the complete request path under real enforcement and document coverage
  of retries, redirects, UDP, existing/inherited sockets and loopback proxies.

The development loop must distinguish accepted completion evidence from a
blocked or partial worker result; passing existing tests alone is insufficient.

## Limits of this approval

No new agent capabilities, broader grants, unrestricted loopback-proxy access,
host-wide networking changes, physical installation or boot-setting changes are
authorized. Development remains in WSL/VM environments; hardware acceptance is
pending. Coordinate shared kernel resources and do not remove another worker's
pins. Kernel-sourced audit recording and changes to "decides and forgets" remain
separate architectural decisions requiring approval.

If the implementation cannot meet these limits, report the specific additional
decision needed rather than treating this approval as unlimited discretion.

## Handoff and verification

Claude should pull this report, record the scoped ADR, update its own kernel plan
to remove the resolved approval blocker, and continue end-to-end network
enforcement integration. This report does not launch Claude or change its checkout.

This change records an explicit user decision only. No runtime code, permissions,
tests or shared progress documents were changed. Verification: inspect the report
against the approved scope and run `git diff --check`; no runtime test result is
claimed. The integration owner should reference this report in STATE.md and retain
the network requirement as unfinished until end-to-end acceptance passes.
