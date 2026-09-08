# What actually protects a credential

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-bounding`, ADR 0022)
- Contributor: Claude Code
- Task: Resolve ADR 0022's implementation and release-scope questions
- Status: **the four questions are answered and one architecture is
  recommended.** [ADR 0022](../../decisions/0022-where-a-providers-key-is-kept.md)
  stays **PROPOSED**. No release tier changed and nothing was implemented.

## Two corrections to what I published before

**1. The release-scope change is withdrawn.** The first ADR 0022 proposed moving
*the key goes to the keyring* to v0.5. The owner declined it, and it was wrong on
its own terms: [ADR 0005](../../decisions/0005-applications-are-sandboxed-and-ask.md)
says portals are how **third-party sandboxed applications** reach the system, so
the v0.5 *Secret portal* line is an interface for other people's software.
`alo-agentd` is not a sandboxed application — it is the system, running as the
person. **The two are distinct deliverables**, a v0.01 store does not need the
portal, and the portal does not deliver one. Nothing moves.

**2. `where-a-credential-goes.md` said a D-Bus or kernel-keyring store "lands on
neither" of the boundary questions a file store lands on. That was wrong**, and
the measurement is below. It is corrected here rather than by rewriting the
published report.

## What specifically blocks the desktop secret service at v0.01

Not "it does not exist". Three named dependencies, read off the image and the
unit:

1. **The image ships no Secret Service.** `image/Containerfile` is
   `fedora-bootc:42` plus two binaries, two units, two directories and one
   description — *"no compositor in it, no desktop, no wallpaper and nothing to
   sign in at."*
2. **`alo-agentd` is a system unit; a Secret Service is a session one.** The unit
   is `WantedBy=multi-user.target`, `User=alo`, started at boot. It runs **as the
   person** and has no session and no session bus.
   [ADR 0017](../../decisions/0017-the-agents-door-is-ours-and-not-in-the-session.md)
   put the door outside the session **deliberately** — so this is the dependency
   that needs a **decision**, not work.
3. **Unlock is tied to sign-in**, which is the compositor and session
   workstream's and is not done.

## Can an existing implementation satisfy it — without inventing a store?

Yes. `libsecret` and the **Secret Service** are the standard, and *locked*,
*unavailable* and *denied* are native states rather than ones alo would invent.

Two others were evaluated and are recorded so they are not re-proposed:

- **`systemd-creds`** — already in the base, needs no session or bus, and does
  not fit: it provisions credentials to a **unit**, decrypted by systemd at start
  from a root-only host key or the TPM. A person adding a provider at runtime
  cannot write one, and a service running as the person cannot decrypt one.
- **The kernel keyring** (`@u`) — works today, no daemon and no session, and
  **nothing survives a reboot**. A real answer with a real price, and the
  fallback in decision 3 below.

## The trust boundary, measured

The instruction not to assume D-Bus resolves isolation was right, and the
measurement says so plainly. New, against the real loaded programme:

**`a_bound_turn_may_reach_a_unix_socket_nobody_showed_it`** — a turn bound to one
folder and shown no destination is refused a non-loopback address with `EACCES`
and **connects to a Unix socket in the same breath**. That is deliberate:
`deciding.rs` permits it because *a Unix socket is not egress, and refusing it
would be enforcing something no policy claims*.

**So a bound turn can reach the session bus, or a keyring daemon's socket,
directly.** Beside it, two facts already reproduced: a connection opened before
a turn stays usable inside it, and `keyctl` is a syscall this boundary does not
hook at all.

**None of the three mechanisms is isolated from a turn by the kernel boundary.**
Choosing between them on that basis would be choosing on a mistake. What
protects a credential is three things, all about *when* rather than *where*:

1. it is fetched **outside** the turn's boundary, where the endpoint is already
   resolved (ADR 0020);
2. what crosses in is a `Secret` — no accessor, no `Display`, no `Serialize`, no
   `Clone`, a hand-written `Debug` that says nothing;
3. **the store handle is not held open across a turn** — the same discipline
   ADR 0020 applies to the HTTP client and to DNS, and the one an implementation
   can get wrong quietly.

## The recommended architecture

The **Secret Service** through `libsecret`, under the attributes alo already
derives (`provider/<name>`); reached by `alo-agentd` as the person it already
runs as, with no new login and no capability; **opened and closed per retrieval,
immediately before the question and outside `carrying_out_a_departure`**;
unlocked at sign-in. `SecretRef` is unchanged — this adds an implementation
behind it and changes no abstraction. Unavailable, locked, missing and denied
each refuse in their own words and **none falls back**.

Dependencies in blocking order: a keyring in the image; **a session bus the
daemon can reach, which collides with ADR 0017**; sign-in; and a `libsecret`
binding — C behind a Rust crate, the same shape as `aya` or `ureq`, and not a
third language in this repository.

## The acceptance plan, and what the published evidence does not cover

Stated precisely: **`the_key_reaches_the_chosen_one_and_the_other_hears_nothing`
used the local-service door on loopback**, because a plain-HTTP provider cannot
be constructed. It establishes the pairing. **It establishes nothing about
authenticated HTTPS provider operation through the daemon**, and that test is
first in the acceptance plan: an owned TLS server, a synthetic credential in a
real store, driven through `alo-agentd`, with the credential asserted **on the
wire** at the far end. Plus the four store states, the no-fallback cases, and an
assertion that no store handle is held across a turn.

## Preserved

The published redaction (`41c9f1e`) and refusal guarantees are untouched: a
settings refusal repeats nothing the file said, a provider needing a key is
refused with nothing sent, and a key cannot be rendered at all. ADR 0021 stays
PROPOSED. No release tier changed; no alo endpoint invented; the three model
choices are unchanged.

## Gates

`cargo fmt --check`, `clippy --workspace --all-targets -D warnings`, `cargo test
--workspace`, `RUSTDOCFLAGS="-D warnings" cargo doc`, the supervisor's own tests,
and the BPF target's fmt and clippy on the pinned nightly, with kernel tests
taking the shared lock. WSL is development evidence and certifies no hardware.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible.

**ROADMAP.md** — no tick, **and no tier moved**. *Add your own provider* stays at
v0.01.

**docs/autonomy/QUEUE.md** — the credential-store item stays, now blocked on one
decision (the session bus) rather than on an open architecture.

**docs/autonomy/STATE.md** — three facts. A v0.01 provider-key store and the v0.5
Secret portal are **distinct deliverables** and nothing moves. A bound turn can
reach a Unix socket nobody showed it, so **no store mechanism is isolated from a
turn by the boundary** — what protects a credential is that it is fetched outside
the turn and crosses in as a value that cannot be read. And ADR 0022 now
recommends the Secret Service, blocked on whether `alo-agentd` may reach the
person's session bus, which collides with ADR 0017.
