# A port held in both families does not stop the service

Date: 2026-09-18. Workstream: local network. Contributor: Codex development worker.

Task: **A port another program holds in both families at start does not stop the service**.

Status: implementation and tests written; validation pending. This report is the
worker's handoff to the supervisor, not a claim that the publication gates passed.
Local validation is deferred to the supervisor. The worker's execution instructions
prohibit cargo, rustc, tests, builds, formatters and synchronization into the shared
Linux gate tree. None were run. No build directory was created, and nothing was
committed or pushed. The repair worker rewrote `.kernel-loop/handoff.toml` from
the evidence and full file list below on 2026-09-18. It requests serialized
validation; it does not certify a gate result.

## Formatting refusal repair

The supervisor refused formatting twice before this repair. The repair changes
only the two reported expressions in
`crates/alo-agentd/src/a_port_held_over_ipv6_at_start.rs`: the IPv4 answer's hex
fold uses the reported multiline method chain, and the dual-stack squatter's
socket constructor uses the reported assignment wrapping. These are manual edits
matching the gate output; no formatter was run. All implementation, tests and
assertions from the earlier worker are preserved. The plan records implementation
completion and publication validation pending, correcting its stale statement that
the supervisor had not run. No passing formatting, lint, test or publication gate
is claimed for the repaired tree.

## Change and decision

A program temporarily holding the advertised port in both families no longer
necessarily stops the service before its person's door opens. `Listeners::bound`
permits an empty listener set when a retryable bind was refused `AddrInUse` and
the kernel's port-release subscription is open. The existing serving loop keeps
discovery and the person's door running and retries the listeners on that event.
There is no new interval, polling, permission, socket subscription or wire field.

`listen_on` returns whether any attempted IPv4 bind encountered `AddrInUse`.
`OverIpv6::held_elsewhere` supplies the corresponding IPv6 state. These distinguish
a recoverable startup conflict from the existing refused-network set, which also
contains other errors. The IPv4 fallback on a machine whose interfaces cannot be
read is not retried, so its refusal cannot authorize an empty startup. A retryable
IPv6 conflict can qualify independently because its listener needs no interface.

Listening nowhere remains `NotBound::NoWire` if there is no recoverable conflict,
or if there is no port-release subscription. A kernel without IPv6 and with every
IPv4 bind refused for another reason therefore still refuses startup. A working
listener needs no subscription and keeps the existing network-change fallback.
This avoids starting an empty wire that cannot recover from the holder's release
alone. The subscription is still opened before any bind, avoiding a release lost
between the bind refusal and subscription.

Presence continues to name the machine and its protocol's fixed port. It does not
certify reachability or identify whoever currently holds that port. Discovery
continues to confer no authority (ADR 0003); pairing and proof are unchanged.
The IPv6 refusal no longer claims that IPv4 successfully bound. The additive wire
contract and `NoWire`/`Wire::bound` rustdoc explain these semantics. ADRs 0041 and
0044 are unchanged; no departure or grant is widened.

## Tests written, not executed

The existing startup-contention namespace fixture now has a second outer test;
its IPv6-only case and all its assertions remain. The dual-stack case explicitly
sets `IPV6_V6ONLY` false, adds IPv4 to the same link-local cable, and starts the
squatter before telling reception to serve. Reception still runs through
`setpriv` with an empty capability bounding set and asserts both effective and
bounding sets are zero.

The new case requires:

- A working person's door and discovery over both families while the squatter
  holds the port; both protocol probes must fail, even though the squatter can
  accept TCP connections. Kernel multicast membership is checked in both families.
- Socket closes while the squatter still holds the port must not make the service
  count the cable as listened on or repeat its refusal log lines.
- Once the squatter lets go, both families must answer the service's own protocol
  refusal, while `ip monitor link address` remains subscribed and prints nothing.
- Discovery bytes must match across families and before/after recovery.
- Exactly one IPv6 refusal/bind, and one refusal/bind each for loopback and cable0;
  no unexpected IPv4 listener log lines and no false IPv4-success claim.

Startup-policy unit tests cover no recoverable conflict, loss of the subscription,
an IPv4 fallback that does not follow interfaces, an IPv4 conflict without IPv6,
and a working listener with no subscription. The IPv6-state test also rejects a
log message claiming IPv4 success without evidence.

Supervisor evidence candidates (workspace, crate, target, full test name):

```text
. alo-agentd lib a_port_held_over_ipv6_at_start::tests::a_port_held_in_both_families_at_start_does_not_stop_the_service
. alo-agentd lib listeners::tests::listening_nowhere_without_a_port_conflict_refuses_to_start
. alo-agentd lib listeners::tests::listening_nowhere_needs_a_retryable_conflict_and_a_subscription
. alo-agentd lib listeners::tests::a_working_listener_starts_without_a_subscription
. alo-agentd lib listening_over_ipv6::tests::a_port_held_over_ipv6_is_said_once_and_bound_once_when_let_go_of
```

The first test covers all kernel-fixture acceptance clauses; the next three cover
the explicit startup-refusal decision. Each test lives in a changed file.
Suggested individual command, substituting each full name above:

```text
cargo test -p alo-agentd --lib <full-name> -- --exact --nocapture
```

Required component checks, all **unrun**, for the supervisor's serialized Linux
gate environment:

```text
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test -p alo-agentd
```

Executed inspection on the repaired Windows checkout: `git diff --check` returned
exit 0 with only Git's CRLF-to-LF notice for the plan. This checks whitespace
only; it is not formatting, build or test validation.

The existing IPv6-only outer fixture should also be run individually as a
regression. No new kernel measurement or mutation result is claimed, and no
unmeasured behavior has been added to `docs/quirks.md`. The known dual-stack bind
conflict is already recorded in Task 37's premise. Physical hardware acceptance
is not claimed.

## Integration metadata

Suggested subject: `fix(agentd): survive a port held in both families at startup`

Suggested body:

> Keep discovery and the person's door serving when every listener is refused
> by a temporary port conflict and the kernel can notify a release. Preserve
> startup refusal without a recoverable conflict or subscription, and remove
> the IPv6 log's unsupported claim of IPv4 success. Extend the startup namespace
> fixture to cover a dual-stack holder and recovery without a network change.

Files changed:

- `crates/alo-agentd/src/a_port_held_over_ipv6_at_start.rs`
- `crates/alo-agentd/src/listeners.rs`
- `crates/alo-agentd/src/listening_over_ipv6.rs`
- `crates/alo-agentd/src/refusing.rs`
- `crates/alo-agentd/src/wire.rs`
- `docs/contracts/local-network-wire.md`
- `docs/autonomy/v0-5-the-local-network-plan.md`
- `docs/autonomy/updates/a-port-held-in-both-families-at-start.md`

Proposed changelog: A temporary conflict on the local-network port in both address
families leaves the person's service and discovery running, and the network port
recovers when the other program releases it.

The staged plan records implementation completion with validation explicitly
pending. That state may only be published after the supervisor passes all nine
gates and the task evidence; no prior validation success is claimed. No subsequent
task was appended: the worker is explicitly limited to its assigned task and may
not extend the plan. CHANGELOG, ROADMAP, QUEUE and STATE were not edited.
