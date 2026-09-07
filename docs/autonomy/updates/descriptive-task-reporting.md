# Descriptive task reporting

- Date: 2026-09-07
- Workstream: development coordination
- Contributor: interactive integration maintainer
- Status: ready for integration

## Change and acceptance criteria

The owner requested professional task names instead of queue codes. New task
titles, report filenames, commit subjects and status messages describe the work.
Historical queue references and ADR identifiers remain intact for traceability.

Parallel contributors publish their own reports under `docs/autonomy/updates/`.
Only the integration owner updates CHANGELOG.md, ROADMAP.md, QUEUE.md and STATE.md.
The saved worker instructions now require report reconciliation at iteration
start; Claude's handoff explicitly stops parallel edits to those four files.
Existing sessions must pull and reread the instructions before the next task.

Modified guidance: CLAUDE.md, docs/autonomy/SHARED_MAIN.md, WORKER.md,
CLAUDE_TASK.md, DELIVERY.md and updates/README.md. The supervisor's publication
checks and four-document requirement remain unchanged for the integration worker.

## Verification

Verified on the cursor-rendering change rebased over the published secure
file-moves change. Both gate command groups exited 0:

- Windows and Ubuntu WSL2: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `cargo test --workspace --locked --quiet` passed, including default doctests.
- Linux: `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked`
  passed. In `crates/alo-bounding-kernel`, `cargo fmt --all --check` and
  `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  passed using the pinned toolchain. Existing opt-in measurements remain opt-in.
- Runner: formatting and clippy with warnings denied passed, all 11 tests
  passed, and the release executable was rebuilt with the updated embedded
  WORKER.md prompt. Commands used `--manifest-path tools/dev-loop/Cargo.toml`;
  tests/build used `--locked`, and clippy included `--all-targets`.
- `git diff --check` passed. No source-code changes were needed to resolve the
  shared journal conflict; both contributors' entries were retained.

The OpenAI Docs skill informed retaining the existing saved-prompt execution
mechanism without changing permissions or model configuration; see the official
[non-interactive mode guidance](https://learn.chatgpt.com/docs/non-interactive-mode).

## Proposed shared progress update

Development tasks now use descriptive names and separate reports. One integration
owner maintains shared progress documents, reducing conflicting documentation
edits while contributors continue tested pushes to main. No release gate changes.

## Limitations

This is contributor guidance, not remote branch-permission enforcement. It cannot
update another already-running session's instructions. Genuine shared-code or
specification conflicts still require review. No hardware acceptance is claimed.
