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
