# Resuming native window control verification

2026-09-09. Owner-authorized recovery of the stopped desktop workstream.

## Preserved work and review

The clean-tree supervisor refused restart at `4077d9e` because the previous
window-control view task and the later build-profile change were still present.
Nothing was reset, stashed, discarded or committed merely to clear that guard.
C: measured 82.3 GiB free on recovery, above the unchanged 12 GiB preflight. The
cause of that later space recovery was not established here; no cleanup,
compaction or Windows configuration change was performed during this recovery.

Reviewed all three new shell modules, both graphical-fixture files, the public
contract, dependency/lockfile changes and the two development/test debug profiles.
The view's private fixed controls cannot acquire execution authority; coordinates
are bounded before offset arithmetic, painting is clipped, and disabled controls
retain hit ownership. Independent graphical expectations use literal masks and
palette values. No production behavior fix was identified in this review.

The supplied enabled/restoring flags are presentation data only. Live mapping
snapshots, derived availability, native labels and pointer press/release routing
remain the next component, not acceptance silently claimed by this painter.
The root profiles retain assertions, overflow checks and normal optimization;
release and the excluded BPF workspace remain unchanged.

## Recovery gates

Local logs are under `.git/alo-loop/recovery-2026-09-09/` (not committed).
Each build/test phase checks C: against the 12 GiB reserve first.

- Windows `cargo fmt --all --check`: passed.
- Windows `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- Windows `cargo test --workspace --locked --quiet`: passed, including its
  documentation tests. Linux-only crates' zero-test Windows runs do not count
  as evidence for those crates.
- Linux workspace fmt/clippy/tests/rustdoc, pinned BPF fmt/clippy and the
  `window_controls_check` graphics fixture: passed on the combined tree at
  `8096910`. The fixture checked all 128 full 5,760-pixel frames successfully.

Claude's kernel supervisor began a workspace test run in its own checkout during
Windows verification. It and its processes were not stopped or edited. Shared
Linux/kernel verification initially waited for handoff; the owner subsequently
approved concurrent development using existing per-test machine locks. The desktop
supervisor stays stopped until the pending work is fully gated, integrated and
published. No weakened gate, force-push or dirty-tree restart is authorized.

At the initial handoff, C: still had 81.3 GiB free. Under the subsequent concurrency
approval, all product gates and the graphical fixture ran successfully. Existing
ignored kernel test children remain intentionally invoked by their parents; no
new ignore or gate exemption was added. Expected refused-protocol and Mesa
fallback diagnostics did not invalidate the passing assertions.

Claude published `9109875` during verification. Fast-forwarded it without overlap,
reviewed its report and started the complete combined-tree re-gate using the warm
targets. Logs use `integrated-910-*` beside the first run's `linux-phase-*` and
`combined-windows-*`. Publication/restart waits for this new run, not a green result
from its predecessor. No commit or push yet.

The complete `9109875` run also passed, including the graphical fixture. A second
incoming fix, `c4e20c7`, was integrated next and complete verification restarted
with `integrated-c4-*` logs. The new report `a-bus-with-nothing-on-it.md` is
reconciled in the shared progress files; no contributor report was rewritten.

## Final verification

All gates passed on the final combined tree with `c4e20c7`: Windows workspace
fmt/clippy/tests, Linux workspace fmt/clippy/tests/rustdoc, pinned BPF fmt/clippy,
example build and all 128 full 5,760-pixel graphical frames. The final local logs
are `integrated-c4-*`; each command exited zero, including the real Secret Service
fixtures. No source correction, skipped assertion or new ignored test was needed
for the painter. The additive API still completes only its layout and painting.

The updated supervisor separately passed fmt, all-target clippy with warnings
denied, 17 tests and a release build. Last remote check matched the integrated
base. The preserved work can now be committed and pushed normally, followed by
the clean-tree supervisor restart; no dirty-tree guard is weakened.
