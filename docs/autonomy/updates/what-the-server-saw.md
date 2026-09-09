# What the server saw

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-agentd`, `alo-asking`)
- Contributor: Claude Code
- Task: Reconciling `329c6be` against the HTTPS acceptance checklist
- Status: **three real gaps closed and one evidence claim corrected.**

## An evidence claim that did not follow

`329c6be` asserted that neither the key nor the question appeared in the **raw
bytes** a rejected connection carried, and called that *nothing was transmitted*.

**That does not follow.** Application data on a TLS connection is encrypted, so a
client that had completed a handshake and sent the credential would leave no
plaintext either — the assertion would pass on exactly the case it existed to
catch. It was a true statement doing a job it could not do.

What settles it is the **server's own view**. `WhatTheServerSaw` now reports
whether the handshake ever completed and whether any application data was
decrypted, and HTTP cannot cross a TLS connection that never finished being
established. So *no completed handshake, nothing decrypted* is the claim, made
from the side that would know.

The raw capture is **kept as supplementary**, and labelled as such: it says the
credential did not cross in the clear, which is worth knowing and is a different
sentence.

### The control, which is what makes the rejections mean anything

`a_trusted_certificate_lets_the_key_and_question_through` runs the **same server**
and the **same assertions** with a certificate the daemon trusts. The handshake
completes, application data is decrypted, and it carries the synthetic key and
the question.

Without it, *no application data* proves nothing: a server that recorded nothing
under any circumstances would satisfy every rejection test in the file.

## The untrusted-issuer case was missing

`329c6be` had only the wrong-identity test. Both are now present and they fail
for **different** reasons:

| Test | Chain | Identity |
|---|---|---|
| `a_certificate_for_the_wrong_address_…` | sound, from the trusted authority | **wrong** |
| `a_certificate_from_an_authority_we_do_not_trust_…` | **from a stranger** | correct |

A verifier that checked only one would pass the other. Both assert through the
instrumentation above.

## Concurrent requests could inherit a test's trust

They could, and now cannot. `IN_FORCE` was a **global**: the mutex serialised
windows but did nothing about a request on another thread, which would have
verified against an authority its own test never chose. It is **thread-local**,
with a `Drop` guard so a panicking test cannot leak it either.

**Verified by mutation, not by reading.** Reverting the seam to a global makes
`a_request_on_another_thread_does_not_inherit_a_tests_trust` fail with exactly
its own message; restoring it passes.

That test was itself vacuous at first — it pointed the other thread at a server
that never answers, so it would have passed whether trust was shared or not. Both
servers now would answer a client that trusted them, so the two halves are
distinguishable.

## The production build leaves the override off

`cargo tree -p alo-agentd -e features --edges no-dev` mentions
`trust-a-test-authority` **zero** times; with dev-dependencies resolved, once.
That is the image's resolution, since `image/Containerfile` builds
`--package alo-agentd --package alo-boundaryd`.

Made durable: the guard now also refuses a `--features` on the image's **build
command**. Scoped to that command deliberately — a first version read the whole
file and failed on `cargo install bpf-linker --features llvm-…`, which is a tool
this image needs and has nothing to do with what our workspace compiles. A check
reporting the wrong thing is worse than no check. Mutation-tested: adding
`--features trust-a-test-authority` to the build line makes it fail and names the
line.

## The bpffs remount is gone

The supervisor already checked read-only and refused; the **unconditional remount
was mine**, run before every publish. It is not done any more.

Its message also *instructed* that remount, which is what led there. It now says
the work is untouched, that mounting to get a task out is the wrong move, and
that the mount is shared with whoever else is testing on this kernel — so it asks
for a coordinated handoff instead.

One was taken for this publish: both workstreams were confirmed idle (no
`cargo`, no `rustc`, no pins, nothing on the Windows side), `/sys/fs/bpf` was
absent, it was mounted once and verified `rw` with exactly one mount at that
path. **WSL was not restarted, no pins removed, no services changed, no lingering
enabled.**

## Real-session logout: scheduled, not done

Task 10 of the plan, with the cases separated and **two seats not assumed**:

1. **One session ended** — one `ssh` login, logged out, no lingering.
2. **Logged out while lingering** — `loginctl enable-linger`, one login, no
   second seat. The user bus is expected to **survive**, which is a question for
   the owner rather than a bug: a credential reachable when nobody is signed in
   is a policy.
3. **Two sessions, one ended** — two `ssh` logins. Concurrency, still no second
   physical seat.

`enable-linger` is machine-wide and flagged as needing its own handoff. **Nothing
here is claimed as done**, and ADR 0022 says the same where it records the
measurement.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — credential store: HTTPS acceptance reconciled;
real-session logout scheduled as task 10.

**docs/autonomy/STATE.md** — certificate rejection is now evidenced by the
server's own handshake and application-data instrumentation with a trusted-
certificate control, the raw capture kept as supplementary; untrusted-issuer and
wrong-identity are separate tests; the test trust seam is thread-local and proved
so by mutation; the image build is confirmed to enable no features.
