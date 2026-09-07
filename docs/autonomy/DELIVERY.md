# Continuous delivery toward v0.01

Authorized by the owner on 2026-09-07: pursue the full v0.01 release continuously,
and commit and push each completed step to `aloworld-org/alo-os`, branch `main`.
Intermediate steps are engineering checkpoints, not a reduced release scope.
`docs/features.md` and the v0.01 exit gate in `ROADMAP.md` remain the definition
of the release. Later-release features remain later-release features.

## Execution order

This order supplements QUEUE.md and supersedes its obsolete portable-only
restriction. An entry under its historical Linux section is eligible here.
Linux is available in Ubuntu WSL2 and a Wayland socket is available through
WSLg, checked 2026-09-07. Check the graphics development packages before building.
Code readiness, an integration demonstration and physical verification are
separate facts. None of the tasks below is finished merely by writing this list.

1. **Linux and graphical development gate.** Establish the existing Linux/BPF
   test baseline and a reproducible Smithay build environment. Record actual
   missing prerequisites and install ordinary development packages in Ubuntu.
   Gate: existing Linux tests, clippy, rustdoc and pinned BPF target pass.
2. **Native compositor and input.** Implement ADR 0002's Rust/Smithay compositor,
   one display, keyboard/pointer and real client windows. A nested WSLg session
   is the development fixture; the booted desktop also needs the direct display
   backend. Refuse failed graphics initialization clearly; verify client exit,
   disconnect and input routing. No production capability is ticked for a fixture.
3. **Window management, launcher, dock and shortcuts.** Connect the existing
   appearance/dock/shortcut/application models to rendered, usable surfaces;
   open/focus/close/move/resize/minimise/maximise/tile, window switching and
   configurable shortcuts. Preserve label access when dock names disappear.
   Read the existing contracts and ADR 0010. Complete v0.01 clipboard formats.
4. **Accounts and session entry.** Local account and the v0.01 identity sign-in,
   correct per-login daemon environment and session lifetime. Implement the
   native entry surfaces and test invalid authentication and session teardown.
5. **Model/provider selection and grants.** Person-owned configuration, keyring
   integration, native folder selection, persistent/revocable/expiring grants,
   and the no-agent setup path. Connect existing choosing/capability models.
   Revisit 21i, 21l and 21o when their concrete dependencies exist; ADRs 0016/0019
   preserve the boundary between organisation policy and personal choices.
6. **Agent interaction and application integration.** Native invocation overlay,
   context captured only on invocation, proposed changes and single approvals,
   records, visible egress and real Wayland/D-Bus/portal application adapters.
   Test refusal, expiry, revocation, process death and no background context reads.
   Complete 19b, the Linux filesystem race fixes (6b), and remaining security
   wiring; audit older blocked entries against the code before duplicating work.
7. **Desktop image integration.** Package shell, session, services, necessary
   upstream runtimes, vocabulary, design tokens and wallpaper into the existing
   image. Boot in a VM and exercise a real local-model turn through approval,
   execution and its record. Test failures and boot/update recovery mechanisms
   required by v0.01. Do not silently pull in v0.5's recovery screen or other UI.
8. **Release coverage and physical acceptance.** Reconcile every v0.01 feature
   and roadmap line with executable checks and named evidence. Finish remaining
   model measurements where hardware permits. Record the physical laptop and
   GPU workstation checks required by docs/hardware.md; ask for access where
   unavailable. No automatic release-complete verdict without that evidence.

Each entry may require several complete commits. The worker must put the exact
next component and its acceptance checks in QUEUE.md before implementing it,
then update the same entry with evidence and what remains. The release target
never becomes whatever was easiest to test on the current development host.

## Runner

`tools/dev-loop` is a Rust developer tool, separate from the shipped workspace
and absent from the system image's runtime. It launches one `codex exec` worker
at a time, following the installed CLI and official non-interactive interface:
<https://developers.openai.com/codex/noninteractive/>.

Build and check it from `C:\dev\alo-os`:

```powershell
cargo fmt --manifest-path tools/dev-loop/Cargo.toml --check
cargo clippy --manifest-path tools/dev-loop/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tools/dev-loop/Cargo.toml
cargo build --manifest-path tools/dev-loop/Cargo.toml --release
```

Run in the foreground, or start this same executable with a hidden window:

```powershell
$workerExe = (Get-Command codex).Source
& .\tools\dev-loop\target\release\alo-dev-loop.exe run $workerExe
```

The CLI uses the existing signed-in account and configured model. Work consumes
that account's usage; an authentication/usage failure halts rather than retries
forever. The worker uses the full-access execution environment authorized for
this development session; the prompt limits its work to this repository and
Ubuntu build prerequisites. This is not an OS agent verb or part of alo OS.

The supervisor owns an OS file lock for its lifetime. Each step begins with a
clean `main` updated by `git pull --ff-only origin main`; a dirty tree, failed tests,
unexpected commits, supervisor edits and missing progress documents stop it.
It independently runs Windows fmt/clippy/tests and Linux fmt/clippy/tests/docs
plus BPF fmt/clippy before staging, committing with the owner's Git identity,
and normal-pushing. Worker-specific integration evidence remains required.
If another worker advances main, the supervisor rebases its unpublished task
commit onto the new main and repeats the gates before pushing. It retries at
most three publication races. A rebase conflict, failed integration gate or push
rejection without a remote change preserves the local work and halts; no reset,
forced push or automatic conflict resolution occurs. See `SHARED_MAIN.md` for
the direct-to-main collaboration workflow. Gates may take substantial time on
a cold checkout.

The worker has a six-hour deadline. On expiry the runner attempts to end that
worker's process tree and halts; inspect WSL descendants before restarting.
Gate commands themselves run to completion; a hung gate requires intervention.
The runner does not start a second worker while a gate is pending.

Status, stop and local logs:

```powershell
& .\tools\dev-loop\target\release\alo-dev-loop.exe status
& .\tools\dev-loop\target\release\alo-dev-loop.exe stop
Get-Content .git\alo-loop\history.log -Tail 10
```

`stop` finishes the current step and prevents the next one. Review the working
tree and processes before a restart; remove `.git/alo-loop/STOP` explicitly if
present. Each timestamped directory holds worker events, errors, the result and
independent gate output. Logs stay local in `.git/alo-loop`, never in a commit.
The runner operates while this PC is awake; it is not a cloud job or a scheduled
task and does not restart itself after a reboot. Status is last recorded state;
check its PID is still alive after interruption or reboot.

Blocked means blocked, not complete. The historical LOOP COMPLETE marker only
closed the old backend queue. The new runner uses the current worker result,
never scans historical journal prose, and stops on lack of completed work.
