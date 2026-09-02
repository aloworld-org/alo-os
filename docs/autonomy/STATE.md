# Journal

One entry per loop iteration, newest last. What was built, what the gate said,
and anything the next iteration should know before it starts.

The stop markers `LOOP COMPLETE` and `LOOP HALT` are read from this file by the
supervisor, so they appear only when they are true.

---

## 2026-09-02 — before the first iteration

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
