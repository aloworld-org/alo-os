# Keeping Ubuntu active through desktop verification

Date: 2026-09-09. Owner: desktop integration worker.
Status: both combined trees verified; ready for normal publication and restart.

## Recovery

The desktop supervisor preserved its label-rendering commit when the re-gate
after integration failed at `mountpoint -q /sys/fs/bpf` (exit 32). Both supervisors
were stopped and Linux had no build/kernel test active when the owner requested
repair. Mounted `bpf` at `/sys/fs/bpf` once and verified with `findmnt`; no remount
over an active fixture, pin removal, service restart or Windows change. Held one
owned temporary WSL process during recovery, to be released after the supervisor
has its own lease. No other checkout was edited.

Rebased the preserved label commit onto 329c6be. Resolved two lockfile hunks by
retaining distinct rcgen/read-fonts and yasna/yazi entries with their original
versions, checksums and dependencies. Did not regenerate the lockfile or update
dependencies. Label source/painting/report review identified no source correction.

## Preventing recurrence

`awake.rs` holds a piped stdin to an owned Linux `cat` while the supervisor runs.
This covers worker time, Windows gates and git publication, when no other Linux
command may be active. Startup requires a readiness marker within 30 seconds.
Closing the pipe delivers EOF; normal drop waits for exit with bounded launcher
cleanup. A dead launcher or closed pipe refuses work at the next lease check;
it never restarts WSL. The helper does not mount anything, change services,
read user files or hold the shared kernel-test lock.

This avoids the ordinary idle-shutdown gap of a timed keep-alive. It does not
prevent Windows sleep, reboot or externally requested WSL termination. bpffs
readiness remains mandatory and read-only. Another absence still requires an
idle maintenance handoff, not automatic repair. The 12 GiB host-space floor and
separate targets remain unchanged.

## Verification

Supervisor `cargo fmt`, all-target `cargo clippy -- -D warnings` and `cargo test`
passed: 22 tests, zero failures. Five additions cover missing launcher, invalid
handshake, actual WSL lifetime until pipe EOF, loss of the owned launcher and a
bounded handshake timeout. Existing publication/race/failure, disk-reserve and
kernel-gate ordering tests remain unchanged. No exemptions added.

Full Windows/Linux workspace, rustdoc, pinned BPF and label/control/nested
graphics gates run on the integrated tree. Logs are local under
`.git/alo-loop/label-recovery-329/`; no gate success is claimed until completion.
Rebuild the release supervisor after updating its embedded worker instructions,
then normally publish verified work and start from a clean, synchronized main.

The complete 329c6be-based run passed: Windows fmt/clippy/tests, Linux
fmt/clippy/tests/rustdoc, pinned BPF fmt/clippy, all shell examples, 72 label
frames, 896 control frames and all 28 nested offscreen stages. The mount survived
the Windows phase. Supervisor checks passed again (22 tests), and the release
binary was rebuilt with current worker instructions. Main advanced to 5e3a66b
during verification, so integrate and fully re-gate that tree before publication.

## Contributor evidence reconciled

`connections-come-and-go.md` measures four handles/four bus connections, return
to baseline within a bounded wait after drop, and eight concurrent retrievals.
`a-key-over-a-verified-connection.md` supplies exact key delivery over HTTPS with
fixture trust through the daemon and a wrong-IP-identity refusal. Production roots
remain unchanged. Its ADR correction narrows fixture shutdown to disconnection,
not real user logout. Session acceptance remains in its delivery phase.

The published initial HTTPS seam uses global state; untrusted-issuer rejection
and cross-thread trust isolation are Claude's announced follow-up. These are not
inferred from the two original tests. No contributor report was rewritten and no
feature/release/hardware checkbox was moved by this recovery.

Subsequent 5e3a66b and `what-the-server-saw.md` were integrated after the first
complete recovery run. Reviewed its thread-local seam, distinct certificate
refusals with server-side observations/control, production-feature guard and
read-only bpffs failure wording. Full gates are being rerun on that combined
tree. Windows and Linux checks use separate targets and may run concurrently;
each individual phase retains the host-space preflight and Linux fixtures retain
their shared kernel lock. No mutation-test result is claimed by this worker.

The full 5e3a66b-based re-gate passed too: Windows workspace fmt/clippy/tests;
Linux workspace fmt/clippy/tests/rustdoc; pinned BPF fmt/clippy; all shell
examples and the same 72-label/896-control-frame and 28-stage graphical checks.
Every command exited zero. Logs: `.git/alo-loop/label-recovery-5e3-windows/`
and `.git/alo-loop/label-recovery-5e3-linux/`. Final remote check still matched
5e3a66b. No source changed during this re-gate; only this evidence/reconciliation
was updated. Supervisor release binary includes the verified lease and current
worker instructions. Publish normally, then restart only on clean synchronized main.

One existing fixture diagnostic remains visible in the Linux log: the keyring
bus-routing child reports BrokenPipe after its parent stops reading at the
`alo:tried` marker. Reviewed `which_bus_is_reached.rs`: it drops the reader before
the child harness finishes and does not wait/check that child's final exit. The
parent's routing assertions and the entire suite passed. This is a Claude-owned
fixture cleanup follow-up, not evidence of a mount failure; no test was modified
or diagnostic suppressed during this recovery. Expected Mesa/negative-client
diagnostics likewise skipped no graphical assertions.
