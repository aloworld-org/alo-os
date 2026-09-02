# Journal

One entry per loop iteration, newest last. What was built, what the gate said,
and anything the next iteration should know before it starts.

The stop markers `LOOP COMPLETE` and `LOOP HALT` are read from this file by the
supervisor, so they appear only when they are true.

---

## 2026-09-02 — before the first iteration

*(Superseded by the entry below. Kept because the warnings in it still hold.)*

The queue is written and nothing has been taken from it yet.

What exists already, outside the loop: `crates/alo-models` — the catalogue with
its licence gate, the `ModelRuntime` contract, and the Ollama adapter. 22 tests,
nine of them driving the adapter against a real socket. `cargo clippy` reports
zero warnings and zero errors against the workspace deny list.

What the next iteration should know:

- **Item 1 (grants) is the foundation**, and items 2, 3, 4 and 6 all depend on
  its vocabulary. Getting the shape wrong is expensive; read
  `docs/decisions/0001-the-capability-model.md` in full before starting, not the
  summary in the queue.
- **The gate is real and it bites.** An earlier commit in this repository
  claimed clippy was clean when it was not — ten missing-doc warnings and a
  genuine `indexing may panic` error. Run the gate, read what it says, and
  report what it said rather than what was expected.
- **`indexing_slicing` is denied workspace-wide**, tests included. In a test
  module the honest fix is an `#[expect]` with a reason, because a panic on a
  bad index there is the failure being reported. In library code, it is a bug.
- **Do not couple a test to a dependency's formatting.** One test asserted on
  the exact JSON its HTTP client emitted and failed over a space. A test that
  fails over whitespace is a test that eventually gets silenced.


---

## 2026-09-02 — queue refreshed, still before the first iteration

The loop has still not run. Everything in `crates/alo-models` was built directly
in a session, and four decisions arrived after the queue was first written, so
the queue has been rewritten against what the code actually is.

**What changed that a first iteration must know:**

- **ADR 0007 — the CPU is the default.** A GPU is acceleration. The catalogue
  states what a model costs in system memory and how it behaves without a card,
  and `default_for_cpu` picks for an ordinary laptop. Anything later that assumes
  a GPU is wrong.
- **ADR 0008 — a question is answered in one of three places**, and the person is
  always told which. `source.rs` carries that; item 4 (the record) must record
  the source, and item 5 (egress policy) starts from `SourcePolicy` rather than
  from nothing.
- **The region is the customer's to name**, and alo OS ships no default that
  chooses a provider. Built in Europe, not only for Europe. An item that
  hardcodes a region or a provider is wrong however convenient it looks.
- **`provider.rs` sets the pattern for anything holding a credential:** a
  reference into the keyring, never the secret, with a test that renders the
  struct through `Debug` and serde and asserts nothing secret-shaped survives.

**One rule of the house, before it is broken:** the loop and a person must not
work in the same checkout at once. `CLAUDE.md` forbids it, and this session came
close — the loop was reported as running in `C:/dev/alo-os` while ten commits
were being made there. It was not running; the check that said so had matched its
own command line. Whoever starts the loop owns that checkout until it reports
`LOOP COMPLETE` or halts, and nobody else edits it meanwhile.

Ten ready items. Item 1 (grants) is first because items 2, 3, 4 and 6 all speak
its vocabulary.


---

## 2026-09-02 — iteration 1: grants

**Built: item 1.** `crates/alo-capability`, a new crate holding ADR 0001 as
working code — grants now, and items 2 to 5 beside them later. The name is not
`alo-agentd` on purpose: this is the portable logic a daemon will one day serve,
and calling it the daemon would invite the Linux half to be written into it.

Four files, one job each:

| | |
|---|---|
| `reach.rs` | `Reach` — what a grant covers; `Ask` — what a verb wants to touch. Facing each other, with no method turning one into the other |
| `grant.rs` | One grant, and everything that has to be true of it, checked where it is made rather than where it is used |
| `grants.rs` | The list: what is granted to whom until when, revoke, and the refusal in words |
| `path.rs` | Whether one path is inside another, component by component, touching no disk |

**The gate:** `cargo fmt --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, 72 tests passing
(28 new, 44 in `alo-models`). Built and unit tested on Windows; **no hardware
verification, and none claimed** — there is nothing here that touches a machine
yet.

Every promise the queue asked for is a test rather than a sentence: an expired
grant permits nothing (including at the exact moment it expires), a revoked one
stops on the next question, a path outside a grant is refused, and asking about
a path a hundred times leaves the list byte-identical.

**What the next iteration must know:**

- **Nothing in the crate reads the clock.** Every time-dependent question takes
  `now`. Item 3 (approvals) has the same shape of problem and should do the same
  — an approval that expires needs the same testability, and a second way of
  telling the time is a second thing to disagree about.
- **Containment is lexical.** `path.rs` decides without a syscall, which means a
  symbolic link inside a granted folder can point outside it. The rule is
  written in that file's docs: whatever executes a verb resolves the real path
  and asks about *that*. **Item 6 is where this becomes real, and getting it
  wrong there defeats item 1 entirely.**
- **Identities are matched exactly** — agent names, application ids, paths. This
  differs from `alo-models`, where provider names are compared
  case-insensitively, and the difference is deliberate: a name a person types is
  matched kindly, a name that decides reach is not. If item 2 adds verb names,
  they are identities.
- **A grant cannot be made without an end.** There is no "for ever" variant and
  zero is refused. If something later genuinely needs an indefinite grant, that
  is an ADR, not a new enum variant.
- **The errors are English in the source**, as in `alo-models`. Item 9 (strings)
  is where that gets fixed, and `GrantError` plus `Grants::refusal` are on its
  list — they are user-facing text, and `provider.rs` is not the only file
  holding some.
- **Storage is serde only.** `Grants` round-trips and handles keep counting from
  where they were, so a revoke from a stale list cannot land on a grant made
  since. *Where* the list is written belongs to the daemon and does not exist.

Nine ready items left. Item 2 (the verb registry) is next, and it speaks this
crate's vocabulary — read `reach.rs` before starting it, because `Ask` is what a
verb's arguments have to reduce to when the registry asks whether a call is
within its grant.
