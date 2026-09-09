# A key, over a verified connection

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-agentd`, `alo-asking`)
- Contributor: Claude Code
- Task: Authenticated HTTPS acceptance, and ADR 0022's measured findings
- Status: **done.** The key reaches an owned HTTPS server through the production
  daemon path, with certificate and identity verification on, and **production
  trust is unchanged.**

## What is proved

`the_key_reaches_the_chosen_provider_over_a_verified_connection` — a real
`gnome-keyring-daemon` holds a synthetic key; the daemon reads the person's
settings, fetches `provider/Mine`, and the key **arrives at a server this test
owns**. It is read back **out of the decrypted TLS stream**, so what is asserted
is what came through the connection rather than what was pointed at it. The key
itself is matched, not merely the presence of an `authorization:` header, which
would pass for an empty one. A second provider is watched throughout and is never
connected to.

`a_certificate_for_the_wrong_address_is_refused_and_no_key_is_sent` — the half
that makes the first one mean something. The certificate is issued by **the same
authority the daemon trusts** and is entirely valid; it simply carries a
different address from the one being connected to. The handshake does not
complete, the question is refused, and **nothing that arrived contains the key**.
If identity were not being checked, this test would pass silently.

Between them: the chain is verified, the identity is verified, and neither is
assumed.

## Production trust is untouched

`alo-asking` still sets no root certificates, so `ureq`'s default still applies —
`RootCerts::WebPki`, the Mozilla programme compiled into the binary. **No
platform verifier was enabled and no production root configuration was added.**

What was added is a test-only seam, `an_authority_a_test_made`, and four things
keep it out of a machine:

1. **A cargo feature that is off by default.** Without `trust-a-test-authority`
   the module is not compiled and `put` is byte-for-byte what it was.
2. **A scope, not a setting.** `while_trusting_only(pem, doing)` applies the
   authority for one closure and takes it away afterwards. An environment
   variable was the obvious mechanism and is wrong twice: `std::env::set_var` is
   `unsafe` in this edition and `CLAUDE.md` forbids `unsafe` outside
   `alo-bounding-kernel`, and a variable stays set for whatever runs next.
3. **Only a `dev-dependencies` entry turns it on**, in `alo-agentd`. The image
   builds `--package alo-agentd --package alo-boundaryd`, and a `--package`
   release build resolves no dev-dependency.
4. **A check rather than a promise.** `nothing_ships_the_fixture.rs` now refuses
   any manifest naming the feature outside a `dev-dependencies` table. Declaring
   it under `[features]` is allowed, because declaring is not enabling — but a
   line spelling `some-crate/trust-a-test-authority` is refused wherever it
   appears, since that *is* enabling. The slash is the whole difference.

The seam also cannot weaken anything: it replaces the root set and nothing else.
There is no path in it that disables verification, and it trusts the test's
authority **instead of** the real roots rather than as well as them — so a
certificate that chained to a public authority by accident could not pass
unnoticed.

## Why the identity is an address

The Provider door will carry a key to nothing else. A loopback endpoint makes
`Provider::source` say *this machine*, which `Asking::to_a_provider` refuses as
`Miswired::NotAProvider`; and `Provider::checked` refuses plain `http://`
anywhere but this machine. So the endpoint is `https://` at this machine's own
address, and the certificate carries that address as its subject-alternative
name. Verification is therefore of an IP identity — real certificate and identity
checking, and named as what it is rather than as a hostname.

The mismatch test uses `198.51.100.7`, an RFC 5737 documentation address this
machine is not.

## Dependencies added, and why these

`rcgen` and `rustls`, **dev-only**, in `alo-agentd`. `rustls` with `ring` rather
than `aws-lc-rs`, because `ureq` already brings `ring` and a second cryptography
provider — a C one — is not a thing to add for a test.

## ADR 0022 records the measurements

The lifetime findings are now in the ADR, with libsecret's behaviour preserved
as history and clearly separated from the current implementation. The claims are
stated exactly: four simultaneous handles produced four bus connections and
dropping them returned the count to baseline **within the bounded wait**; eight
concurrent retrievals succeeded.

The logout claim is scoped precisely. The fixture **stops the private bus and
keyring daemon it started** — that is disconnection handling, not real user
logout, where `/run/user/<uid>` and the user bus may survive another session or a
lingering user. **Real-session logout acceptance is outstanding and is not
claimed.**

## Still open

**Real-session logout acceptance**, reported separately as the ADR now says: a
machine where a person logs out of one seat while another session or a lingering
user keeps the bus alive. That needs a machine with `logind` and two seats, not a
fixture.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible; no provider is configured on any real
machine yet. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — credential store: authenticated HTTPS proved end to
end; real-session logout outstanding.

**docs/autonomy/STATE.md** — a provider key fetched from a real keyring reaches
an owned HTTPS server through the production daemon path with certificate and
identity verification on, and a certificate for the wrong address is refused with
no key sent; production trust is unchanged and the test-only seam is feature-off
by default, scoped to one closure, dev-dependency-only and guarded by a test.
