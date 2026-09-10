# The local account that needs no tenant

**Date:** 2026-09-10
**Workstream:** v0.01 lane B — accounts and session entry
(`docs/autonomy/v0-01-lane-b-plan.md`, task 1; the main plan's task 4)
**Contributor:** the lane B build loop's worker, in this checkout

## What changed

v0.01's exit gate opens with *sign in*, and until this change nothing in the
repository signed anybody in. There is now a crate for the half of phase 4
the gate actually requires: a local account, against the machine's own store
— no identity provider, no tenant, no network.

New crate `crates/alo-accounts`:

- `src/account.rs` — one account: a name held to the shape a Unix login
  already has to have, a uid, and the Argon2id hash of a password. Uid 0 and
  the no-user value `4294967295` are refused at creation.
- `src/hashing.rs` — the one thing done to a password, and the decoy hash an
  unknown name is verified against so it costs what a wrong password costs.
  `Debug` for a hash prints nothing of it.
- `src/store.rs` — the accounts, creation with collision refusals, and
  `signs_in`, which answers a wrong password and an unknown name with one
  value after the same work.
- `src/written.rs` — the store as TOML (`format = 1`), read back believed or
  refused: unknown keys, other formats, raw passwords where a hash belongs,
  and duplicate names or numbers all refuse the whole file.
- `src/keeping.rs` (Unix only) — the store on the disk at
  `/etc/alo/accounts.toml`: not a symlink, owned by root or the reader,
  writable by nobody else, created `0600`, and replaced atomically via a
  synced sibling and a rename. The folder is refused rather than created.
- `src/session.rs` — `SignedIn` (authenticated, not yet a session) and
  `Session`, whose only constructor takes the described person's uid and
  refuses to exist where the two numbers differ.
- `src/refusing.rs` — the refusals, split by audience; the person-facing one
  has no `Display` and speaks only through the vocabulary.
- `src/words.rs` — the two person-facing sentences, externalised for
  translation with notes, and gap-free by test.

Wired in: the root workspace gains the member and two dependencies (below);
`crates/alo-saying` collects the new crate's words into the machine's one
vocabulary (its list, its count test and its sum test updated). Both delivery
plans carry the Done line for this task, per the lane B protocol — the same
change marks the matching task in `v0-01-delivery-plan.md` so the image task
there is not left waiting on work that is done.

**User-readable change description:** alo OS can now create a local account
on the machine itself and check a password against it, entirely offline. A
wrong password is refused in plain words, and the refusal gives nothing away
about which accounts exist — not even by how long it takes. The session a
correct password opens is guaranteed to carry the same identity the agent
service is configured with, so the machine can never disagree with itself
about who is signed in. The sign-in screen itself is separate work.

## Decisions taken (DECIDE RATHER THAN STOP)

- **A new crate, `alo-accounts`.** Accounts are a responsibility no existing
  crate has; putting them in `alo-agentd` would give the privileged daemon a
  second reason to change (law 4), and `alo-shell` is constrained out by the
  task.
- **The store is alo OS's own file, beside the machine description.**
  `/etc/alo/accounts.toml`, TOML like `agentd.toml`, believed under the same
  ownership rules ("Who may write it" in
  `docs/contracts/machine-description.md`). PAM/shadow integration was
  considered and deliberately not chosen for this task: the acceptance is
  about the machine's own store and the authentication the entry surface
  will call, and wiring `pam_systemd`/logind belongs to task 2 ("the
  daemon's environment is the session's"), which is where the real session
  registration lives. Nothing here precludes the greeter registering the
  session with logind; this crate answers *who may sign in*, not *what a
  session's environment is*.
- **Argon2id via the RustCrypto `argon2` crate — a new dependency, argued.**
  This repository rents engines and avoids dependencies, but a key
  derivation function written in-house is the one kind of self-reliance that
  is itself a security bug. Pure Rust, no C library, `default-features =
  false` so its own randomness stays out; every random byte comes through
  one narrow `getrandom` call (second new dependency, already in the lock as
  a transitive). Parameters are `Argon2::default()` (Argon2id v19, the
  crate's current recommended cost).
- **Timing indistinguishability by construction, then measured.** An unknown
  name is verified against a decoy hash created with the store (same
  parameters as every real hash, password random and discarded). The store
  refuses at read any stored hash that is not a complete argon2id PHC string
  — a cheaper or truncated hash would make one account answer at a different
  speed. The measured half compares interleaved medians and allows a factor
  of three, which is noise-proof under CI load and still catches the real
  failure (skipping the KDF) by orders of magnitude.
- **One refusal value, one sentence.** Wrong password and unknown name are
  the same variant, the same words (`accounts.not-signed-in`), and the
  sentence has no gap a typed name could enter by. The translator's note
  asks every language to keep the ambiguity.
- **The uid agreement is a constructor, not a check.** `Session::opened`
  takes the authenticated account and the described person's uid and refuses
  a mismatch, so a `Session` value carrying a number the description does
  not name cannot exist. The described uid is a parameter: `alo-agentd`'s
  reader stays the one runtime reader of `agentd.toml`, and the integration
  test supplies the number from the shipped image description through
  `alo-image`'s checked reader, so the test and the machine cannot drift
  apart on a copied constant.
- **Uid 0 refused.** The person `alo-agentd` runs as is never root
  (ADR 0001 §2); graphical sign-in as root is not a thing this OS offers,
  and the owner keeps the terminal.
- **No password policy beyond "not empty".** Complexity rules are product
  policy for the first-boot surface to decide; an empty password is an open
  door and is refused here.

## Acceptance, and the test that shows each criterion

All in `crates/alo-accounts/tests/a_person_signs_in.rs`:

1. *A local account is created and authenticated against the machine's own
   store* — `an_account_is_created_and_signs_in_against_the_machines_own_store`:
   created, kept on the disk, found again under the ownership checks, signed
   in; the wrong password refused on the same store from the same disk.
2. *A wrong password is refused in words and is not distinguishable by
   timing from an unknown user* —
   `a_wrong_password_and_an_unknown_name_are_one_refusal_in_words_and_in_time`:
   same value, same rendered sentence (which names neither the account nor
   the cause), and interleaved medians within a factor of three.
3. *The session carries the uid `alo-agentd` is told about* —
   `the_session_carries_the_uid_the_daemon_is_told_about`: the uid is read
   from `image/etc/alo/agentd.toml` via `alo_image::Image`, the session
   carries it, and an account at any other number opens no session.

Refusal paths beside the legitimate ones, unit-tested per module: names a
Unix login cannot have; uid 0 and `u32::MAX`; empty passwords; taken names
and numbers; exact-match sign-in (`Ada` is not `ada`); empty stores; text
that is not a store; unknown keys; `format = 2`; raw passwords in the store;
truncated and non-argon2id hashes; duplicate accounts in the file; symlinked
stores; group/world-writable stores; somebody else's file (every branch of
the ownership rule); a missing folder refused rather than created; a stale
staging file not wedging the next write; and the vocabulary tests (gap-free
sentences, notes, one-key-one-string, collection into `alo-saying`).

## Verification

Platform: WSL Ubuntu on the Windows checkout's bridge, target
`$HOME/target-claude` — development evidence, not certified-hardware
acceptance. Executed:

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` — clean.
- `cargo test -p alo-accounts -p alo-saying` — 44 + 3 + 4 unit/integration
  tests in the new crate, all green; `alo-saying`'s collection tests green
  with the new list.
- `cargo test --workspace` — run before handoff; result recorded by the
  supervisor's gate as well.
- Each evidence test run alone with `--exact` (the supervisor re-runs them).

Not executed here: anything on real hardware (nothing in this task touches a
device), and the BPF target's gates (untouched by this change; the
supervisor runs them at publication).

## Limitations that remain

- Nothing yet *calls* `signs_in` on a booted machine: the entry surface is
  the compositor lane's, and the first-boot flow that creates the initial
  account (and writes the store as root) is part of that surface's work.
- The store is separate from `/etc/shadow`; the account's uid ties it to the
  login the image declares, but no `passwd`-visible password is set. Task 2
  (the session's environment) is where sign-in meets logind, and if PAM
  integration is ever wanted instead, that is an ADR-sized decision for that
  task, not silently precluded by this one.
- Passwords pass through as `&str` and are not zeroised after hashing; the
  hash itself never renders. Worth revisiting if a memory-scraping threat
  model ever enters scope.
- The timing property is measured for the store's own parameters; a future
  parameter migration must keep the decoy and the stored hashes on the same
  cost, which `Hashed::read`'s argon2id-only rule currently enforces.

## Proposed shared-document updates (for the integration owner)

- `CHANGELOG.md`: "alo OS can now create a local account and authenticate a
  sign-in against the machine's own store, entirely offline. A wrong
  password is refused in plain words and gives nothing away — not even by
  timing — about which accounts exist, and the session a sign-in opens
  always carries the identity the agent service is configured with
  (`crates/alo-accounts`)."
- `docs/autonomy/QUEUE.md` / `STATE.md`: lane B task 1 done; main plan task
  4 done (Done lines already in both plan files per the lane protocol);
  task 2 ("The daemon's environment is the session's") is now unblocked.

## Status

Ready for integration.
