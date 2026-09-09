# The daemon fetches, and connects to one provider only

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-agentd`)
- Contributor: Claude Code
- Task: The daemon end-to-end against a real keyring
- Status: **the refusal half is complete. The authenticated-HTTPS half is
  blocked on an owner decision**, stated below with the evidence, and nothing
  was weakened to get around it.

## What is proved

Two tests, both against a **real** `gnome-keyring-daemon` on a bus of the shared
fixture's own, with synthetic credentials only.

### Every store refusal reaches no provider and falls back nowhere

`no_store_refusal_reaches_a_provider_or_falls_back` walks all four states, each
produced for real rather than injected:

| State | How it was actually produced |
|---|---|
| `Unavailable` | no bus at all — `WhoseKeyring::Nobodys` |
| `Missing` | a real keyring, open, with no such key in it |
| `Locked` | a real collection locked through the daemon |
| `Denied` | a running bus whose policy was changed to refuse delivery |

For each, **two** listeners are watched: the provider the person chose, and one
they did not. Neither is ever connected to, and the refusal is not reported as a
fault in alo OS. That second listener is the fallback check — a machine that
tried another provider when the first had no key would pass a test that only
watched the first one.

The distinction the task asked for is kept: an injected transport failure would
prove that a broken socket sends nothing, which nobody doubted. These are real
refusals from a real store.

### The key is fetched, and only the chosen provider is connected to

`the_key_is_fetched_and_only_the_chosen_provider_is_connected_to` stores a
synthetic key in the real keyring, and the daemon reads the person's settings,
asks that keyring for `provider/Mine`, and connects to the address they wrote
down — and to nothing else.

**Nothing is connected to until a key has been obtained**, so a connection
happening at all is the key having come out of a real keyring. That is asserted
positively as well: the refusal that comes back is checked against all four
credential sentences and is none of them, so the ask did not fail for want of a
key.

## The blocker: an owned HTTPS server cannot be trusted

The task asks for an owned HTTPS server with certificate and hostname
verification enabled, and for proof that it receives the correct key. **Those two
cannot both hold today**, and the reason is a deliberate property of the
production path rather than an oversight.

**One.** The Provider door refuses a local address. `Asking::to_a_provider`
answers `Miswired::NotAProvider` for `InferenceSource::ThisMachine`, and a
loopback endpoint makes `Provider::source` say exactly that — a provider on this
machine belongs to the `Served` door. Measured: pointing the daemon at a loopback
server refuses with *something in alo OS is set up wrongly*.

**Two.** `Provider::checked` refuses plain `http://` to anywhere that is not this
machine.

Together: **the only address the Provider door will carry a key to is a real TLS
endpoint.** A preliminary HTTP test is therefore not available through the
production daemon path at all — not merely insufficient for acceptance.

**Three.** The daemon's trust anchors are compiled in. `ureq` is configured
`default-features = false, features = ["rustls", "json"]`, `alo-asking` sets no
`root_certs` and no `tls_config`, so ureq's default applies: `RootCerts::WebPki`,
which is `webpki_roots::TLS_SERVER_ROOTS` — the Mozilla root program. Not the
machine's store, not `SSL_CERT_FILE`, not an environment variable. Read out of
the vendored source, not inferred from the lockfile.

So a certificate this repository can issue is **not trustable by the daemon**,
and no test may make it so: doing that means either turning verification off or
adding a root, both of which are the authentication this task exists to prove.

## The decision this needs

Three ways forward. **None is implemented**, and the third is what the code does
today.

**A — trust the machine's certificate store.** Enable `ureq`'s
`platform-verifier` feature. A test then installs its CA into an isolated store
and the full assertion becomes possible. It is also what a normal operating
system does, and an enterprise deploying alo OS behind an internal CA will
eventually need it. It is a **change to the trust model**: what the daemon
accepts becomes something the machine's administrator controls, which is a
sovereignty question and not a testing one.

**B — an operator-configurable extra root.** A config key naming additional trust
anchors. Narrower than A, but it is a new public surface and a contract, and it
puts a *credential-bearing* connection's trust in a file.

**C — leave it, and accept the narrower claim.** Key delivery over TLS to an
owned server is not asserted at the daemon layer. What stands instead: the key is
fetched from a real keyring and only the chosen provider is connected to (here),
the bytes carrying a key are read off an owned server one layer down in
`alo-asking`, and the four refusals send nothing anywhere.

**Recommended: A**, on the merits rather than for the test — an OS that cannot be
told about an organisation's own CA will not survive contact with one. But it
changes what the daemon trusts, so it is the owner's to decide, and **ADR 0022 is
not amended by this report.**

## What is preserved

Explicit bus selection from the daemon's own uid; the encrypted `Dh` session;
redaction; refusal without fallback; agent and daemon identity separation. **No
verification was disabled, no root was added, and no test asserts less than it
says.** The fixture keeps its private bus, isolated storage, synthetic
credentials and cleanup limited to its own processes.

## Still to do

Connection lifetime, concurrent retrievals and logout behaviour — the next task,
and independent of the decision above.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick, and
authenticated-HTTPS acceptance is **not** complete.

**docs/autonomy/QUEUE.md** — credential store: refusals proved end-to-end at the
daemon; authenticated HTTPS blocked on the trust-anchor decision above.

**docs/autonomy/STATE.md** — the daemon fetches a provider's key from a real
keyring and connects only to the chosen provider; all four store refusals reach
no provider and fall back nowhere; proving key *delivery* to an owned HTTPS
server requires a trust-model decision, since the daemon trusts compiled-in
Mozilla roots and the Provider door will carry a key to no other kind of address.
