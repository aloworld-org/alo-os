# A green gate check that ran three gates of nine

**The invisible kind.** Nothing failed. The check that should have said so was
green on every pull request and every queue build, and the fault was found by
somebody reading a log for a different reason.

**Concluded:** *a green `gate` check on a pull request means the nine gates
`tools/kernel-loop/src/gates.rs` defines were run on GitHub's runner, and only
the tests that runner is declared unable to run had failed.* That is what
`.github/workflows/gate.yml` printed on success — *"All of them passed"* — and
what the status `alo/gates-on-a-runner` was read as.

**What was true:** three gates ran. The supervisor ran its gates in order and
returned at the first refusal:

```rust
// tools/kernel-loop/src/gates.rs, before 2026-11-07
        refused.push_str(
            "\n\nRun twice and refused both times, so this is the work rather than the machine.",
        );
        return Err(refused);
```

The third gate is the workspace's tests, and on the runner it always refused, on
the one test `.github/the-tests-a-hosted-runner-cannot-run.txt` declares. The
workflow then asked *which gate failed*, and accepted the answer:

```sh
failed=$(sed -n 's/.*alo-kernel-loop: the gate `\(.*\)` did not pass.*/\1/p' \
  /tmp/the-gates.log 2>/dev/null | head -1)
if [ "$failed" != "the workspace's tests" ]; then
```

So the supervisor's formatting, clippy and tests, rustdoc with warnings denied,
and the BPF target's formatting and clippy ran nowhere. A run on 2026-11-07
(job 128549334431) shows it: *"running 9 gates"*, then *"the gate `the
workspace's tests` did not pass"*, then the verdict, and nothing between them
for the six. A broken rustdoc link reached `main` in #1107 and was found by hand
in #1114.

**The mechanism.** The workflow checked the refusal it was given and never
asked what it was not given. *Which gate failed* has an answer; *which gates
ran* was not a question anybody had written down, and a gate that does not run
prints nothing — so its absence read exactly like its passing. Stopping at the
first failure was right where it was written, for a loop deciding whether to
publish, and became wrong the day a caller started *accepting* a failure and
carrying on: the acceptance was added downstream of a stop that nothing
downstream could see past. This had happened once before, to the BPF target's
formatting gate in September, and `crates/alo-bounding-kernel/rust-toolchain.toml`
records it — fixed for that gate, not for the shape.

**The cure.** A caller that accepts a failure must run everything after it and
**account for every item it was given, by name**. `alo-kernel-loop gates` now
runs each gate whatever the one before it said (`tools/kernel-loop/src/every_gate.rs`,
with a test that a refusal does not stop the next gate), and prints
`GATE defined | <name>` for each before any runs and `GATE passed` or
`GATE refused` as each finishes. The workflow fails a run in which any defined
gate is not answered exactly once, and fails one whose exit code and answers
disagree. That check runs on every pull request and every queue build. The
question to ask of any list a check accepts failures from: *what else was on it,
and where does it say they ran?*
