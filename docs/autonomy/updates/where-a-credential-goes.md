# Where a credential goes, and the decision the store needs

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-asking`, `alo-agentd`)
- Contributor: Claude Code
- Task: Secure provider credentials and connect authenticated requests end to end
- Status: **the routing and refusal halves are proved; the store is blocked on a
  decision** — [ADR 0022](../../decisions/0022-where-a-providers-key-is-kept.md),
  **PROPOSED**

The redaction half was published separately as `41c9f1e`
(`settings-refusals-carry-no-credential.md`), as asked.

## The store: blocked, and why that is not a guess

**The accepted documents answer the question two different ways.**
`docs/features.md` promises at **v0.01** that *the key goes to the keyring, never
into a settings file*, and at **v0.5** that *Secret storage — one keyring behind
the Secret portal* is what stops applications inventing credential storage.

Both can be true only if they are two different things, and that reading is
nowhere written down. Three things follow that cannot be guessed at: where the
bytes are, what protects them at rest before full-disk encryption (v0.5), and
**what may read them**.

The third is the one worth an ADR. `alo-agentd` runs as the person and a turn is
one of its threads, so a file store is opened either **inside** a turn — refused
by the kernel, because ADR 0013 means a bound turn opens only what its grant
names — or **before** it, which is the inherited-descriptor gap this workstream
has already reproduced and the one gap that moves contents past a grant. A D-Bus
or kernel-keyring store lands on neither: neither is a `file_open`, and a Unix
socket is not egress.

So the mechanism choice is a security argument rather than a preference, and
ADR 0022 sets out A (a file alo owns), B (the desktop secret service), C (the
kernel keyring), with a recommendation and the exact decision. **Only that work
is stopped.** Everything below was finished.

## What was proved, and where

Existing coverage was reused. `openai.rs` already proves a service given a key is
sent `authorization: bearer …` and one given none is sent no authorisation at
all; `secret.rs` already proves a key cannot be rendered at all. Neither is
repeated.

| Property asked for | Evidence | Result |
|---|---|---|
| The selected provider receives the intended credential | `openai.rs` (existing), and `the_key_reaches_the_chosen_one_and_the_other_hears_nothing` from the far end of a socket | **proved** |
| A different provider cannot receive it | same test — **two services configured and listening; the one not chosen is never connected to** | **proved** |
| Missing or inaccessible credentials prevent transmission | `a_provider_that_needs_a_key_is_refused_and_nothing_is_sent` — **through the daemon**, with an owned listener that is never connected to | **proved** |
| Credentials do not appear in diagnostics or records | `secret.rs` (structural: no accessor, no `Display`, no `Serialize`, hand-written `Debug`), and `41c9f1e` for settings refusals | **proved, and not re-asserted** |

**The daemon test is the one that matters most.** A settings file chooses a
provider that needs a key, pointed at a listener this test owns on this machine's
own interface. The daemon refuses with *the provider you chose needs a key, and
this alo OS has nowhere to keep one yet*, the listener **accepts nothing**, and
the record contains no departure — because nothing departed.

### One thing the tests had to work around, and it is a guarantee

A plain-HTTP test **provider cannot be constructed**: `Provider::checked` answers
`InsecureEndpoint` for `http://` to anywhere that is not this machine. So the
two-endpoint routing test uses the local-service door, which is the *same*
pairing — both doors take a provider and its key together, at construction, and
hand the pair to the same `openai::put`.

That workaround is asserted rather than described:
`a_plain_http_provider_somewhere_else_cannot_be_made_at_all` fails the day the
refusal stops, and whoever changed it reads why the file is shaped as it is.

**Owned fixtures, a synthetic secret (`sk-live-DO-NOT-LOG-9f3c1a`), no real
credential and no external provider contacted.**

## What is preserved

- **Local models / Your own API provider / Alo** — unchanged, and no fourth
  choice.
- **`SecretRef`** — untouched. The abstraction was already right; what is
  missing is one implementation of it.
- **No silent fallback** — a provider that cannot be given its key is refused,
  not asked without it and not answered by another provider or model.
- **ADR 0021 remains PROPOSED**; `ThisMachineOnly` semantics are untouched; no
  alo endpoint was invented.

## Gates

`cargo fmt --check`, `clippy --workspace --all-targets -D warnings`, `cargo test
--workspace`, `RUSTDOCFLAGS="-D warnings" cargo doc`, the supervisor's own tests,
and the BPF target's fmt and clippy on the pinned nightly, with kernel tests
taking the shared lock. Published through the hardened workflow.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible.

**ROADMAP.md** — no tick. *Add your own provider* cannot be called done while a
keyed provider is refused.

**docs/autonomy/QUEUE.md** — one item: **a credential store**, blocked on
ADR 0022.

**docs/autonomy/STATE.md** — three facts. A credential reaches only the provider
it was paired with, proved from the far end of two sockets. A provider needing a
key is refused by the daemon with nothing sent, proved with an owned listener
that is never connected to. And the store itself is blocked on ADR 0022, because
`docs/features.md` promises the keyring at v0.01 and the keyring at v0.5, and a
file store lands on the open inherited-descriptor question.
