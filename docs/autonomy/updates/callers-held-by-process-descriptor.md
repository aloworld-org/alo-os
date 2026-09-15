# An application is named by the process the bus holds, never by a number that can be reused

**Date:** 2026-09-15
**Workstream:** v0.5 applications, and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 7)
**Contributor:** Claude (Mac lane worker, Lima VM)
**Status:** ready for integration

## What changed

Every portal answer on this machine depends on one lookup: which application
sent the request. The backend asked the bus for the sender's process **number**,
then read that process's sandbox file. If the sender ended in between, and its
number went to another process, the request was judged as whatever application
that other process was. The same lookup decides who is sent `SettingChanged`.

The backend now holds the sender by a **process descriptor** (a pidfd). A
process descriptor names one process and cannot come to mean another. The
sandbox file counts only if that process is still alive after the file is read.
A caller that ended in that window is not identified. It gets the refusal a
program with no sandbox gets, and is recorded that way.

### Change description (for CHANGELOG.md)

> **A portal request is judged as the application that sent it, and no other.**
> alo OS identifies an application asking a portal by holding the process that
> sent the request, not by its process number, which the system can reuse. If
> that process ends while its sandbox is being checked, the request is refused
> as unidentified and recorded that way, rather than being judged as whichever
> program received the number next. The same holds for appearance changes sent
> to applications.

### Source

`crates/alo-portals`:

- `src/held_process.rs` (Linux, new). `HeldProcess`:
  - `given(descriptor, said)` holds the `ProcessFD` a bus daemon sends. It is
    refused if it is not a living process's descriptor, or if the `ProcessID`
    beside it names another process.
  - `opened_for(number)` opens a pidfd with `rustix::process::pidfd_open`.
  - `is_still_alive()` reads the descriptor's `Pid:` line in
    `/proc/self/fdinfo/<fd>`, which is `-1` once the process has ended and been
    reaped.
- `src/sandboxed.rs`. `Sandboxes::application_of` now takes `&HeldProcess`, not
  a `u32`, and returns `Sandboxed`: `Named(String)`, `Nobody` or `Gone`. Reading
  by number is now private. The public API has no way to identify a caller from
  a bare number.
- `src/caller.rs`. `caller_of` asks `GetConnectionCredentials`, holds the
  process (from `ProcessFD`, or else by opening a descriptor for `ProcessID`),
  and reads the sandbox. Without `ProcessFD`, it then asks the bus again, and
  the connection must still have the same number. `application_of` keeps its
  signature for `asked.rs` and `settings_portal.rs`, so every request path
  changed without being edited.
- `src/watching_appearance.rs`. A connection whose process is `Gone` is sent
  nothing and is recorded as `Unanswered::NotIdentified`, naming nobody.
- `src/lib.rs`. Registers `held_process` and exports `HeldProcess` and
  `Sandboxed`.
- `Cargo.toml`. Adds `rustix` (a workspace dependency, `process` and `fs`
  features already on) for Linux, and as a Linux dev-dependency to make the
  test's named pipe. `Cargo.lock` gains one edge (`alo-portals → rustix
  1.1.4`) and no package.

Tests: `crates/alo-portals/tests/a_caller_is_named_by_the_process_the_bus_holds.rs`
(new), and unit tests in `src/held_process.rs`.

Docs: `docs/contracts/portals.md` *Who is asking* says how the process is held
and what is refused. `docs/quirks.md` gains *`dbus-daemon` 1.14.10 names a
caller's process only by its number*.

## Decisions

1. **A pidfd from rustix, with no `unsafe` and no new crate.** `pidfd_open` is
   safe in rustix 1.1.4 under the `process` feature, which the workspace already
   enables. To get the number behind a descriptor I read `/proc/self/fdinfo`.
   `PIDFD_GET_INFO` would need an ioctl rustix does not wrap safely, and
   `pidfd_send_signal` with signal 0 cannot be expressed with rustix's `Signal`.
   The fdinfo `Pid:` line is the kernel's own statement, including `-1` for a
   process that has been reaped.

2. **Alive afterwards means the read was of that process.** A process number
   cannot be given to another process until its holder has been reaped. If the
   held process is still alive after the file is read, the number named it for
   the whole read. A zombie (ended, not reaped) still holds its number, and its
   `/proc/<pid>/root` cannot be read, so it is `Nobody`.

3. **Without `ProcessFD`, ask the bus again.** A descriptor opened for a number
   can already belong to a reused number if the caller ended before it was
   opened. A process that has ended has closed its socket, so the bus drops the
   connection. Asking `GetConnectionCredentials` again after the read, and
   requiring the same number, catches that case whenever the daemon has noticed.
   The window that remains (the number reused before the descriptor was opened,
   and the daemon not yet aware of the closed socket) is recorded in
   `docs/quirks.md`. Only a daemon that sends `ProcessFD` closes it completely.
   With `ProcessFD`, the second question is skipped: the descriptor came from
   the socket.

4. **Any failure to hold a named process is `Gone`, not `Nobody`.** If the bus
   named a number or sent a descriptor and it cannot be held, the process that
   sent the request is not there to be judged. Both refuse the request the same
   way. They differ only in whether the watcher records it (decision 5).
   A kernel without pidfds (before 5.3) would refuse every caller. That fails
   closed. The certified kernels are far newer.

5. **The watcher records a gone connection, and only that.** Task 6 decided that
   a connection sent nothing is not recorded, so the record is not buried under
   every program on the bus. A connection whose process ended while it was being
   identified is the exception. It is what a number-reuse attempt looks like,
   and it is the refusal a person reading the record needs to find. It is
   recorded as `NotIdentified` for `Portal::Settings`, naming nobody. A program
   with no sandbox is still not recorded.

6. **The reproduction uses a real second process and a named pipe, with no hook
   in production code.** The caller is the test binary run again with
   `--exact on_the_bus::a_caller_run_as_another_process`. That test does nothing
   unless its two environment variables are set. The caller's `.flatpak-info`
   in the `/proc`-shaped directory is a FIFO. Opening it blocks the backend
   until the test opens the write end. At that moment the test kills and reaps
   the caller, then writes a sandbox naming `org.gnome.Fractal`, which holds
   every grant the request needs. A backend that trusted the number would answer
   Fractal. Reusing the number itself cannot be forced in a test. The number
   being free when the file is read is the condition the fix has to handle, and
   the test creates exactly that.

7. **The number beside a `ProcessFD` must agree with it.** A daemon that gives
   two different processes is believed about neither.

## Acceptance, clause by clause

| Clause | Test |
|---|---|
| A caller gone by the time its sandbox is read is `NotIdentified` and recorded that way, for `OpenURI` | `alo-portals` `a_caller_is_named_by_the_process_the_bus_holds` `on_the_bus::an_open_file_request_whose_caller_is_gone_is_not_identified` |
| … for `Secret` | `on_the_bus::a_secret_request_whose_caller_is_gone_is_not_identified` |
| … for `Settings` | `on_the_bus::a_settings_read_whose_caller_is_gone_is_not_identified` |
| … and for a `SettingChanged` that is then not sent | `on_the_bus::a_change_for_a_connection_whose_process_is_gone_is_not_sent` |
| A caller in another process that is still there is named (the refusals above come from the process being gone, and nothing else) | `on_the_bus::a_caller_in_another_process_that_is_still_there_is_named` |
| `Sandboxes` identifies through a pidfd: `ProcessFD` when given (must agree with `ProcessID`), otherwise one opened for the number, checked alive after reading | `alo-portals` lib `held_process::tests::a_descriptor_handed_over_is_held_only_when_it_agrees`, `…::a_process_that_ended_after_it_was_held_is_not_alive`, `…::a_living_process_is_held_and_alive`, `…::a_number_with_no_process_is_not_held` |
| The legitimate path of every existing bus test still passes unchanged | `the_portal_backend_answers_on_a_real_bus` (5), `the_settings_portal_answers_appearance` (5), and `alo-secrets`' `the_secret_portal_answers_on_a_real_bus`, none of them edited |
| `docs/contracts/portals.md`'s *Who is asking* says how the process is held; `docs/quirks.md` records which daemons give `ProcessFD` | `a_caller_is_named_by_the_process_the_bus_holds` `the_contract_and_the_quirks_say_how_a_caller_is_held` |
| Constraint: no `unsafe`, no new crate in the tree | `Cargo.lock` gains one dependency edge and no package; no `unsafe` in the diff |

**The reproduction was checked against the old behaviour.** With the
alive-after-read check and the second bus question both disabled, all four
gone-caller tests failed. The record showed `application:
Some("org.gnome.Fractal")` with `KeyringUnavailable` (Secret),
`AppearanceRead` (Settings), `Refused(NotAllowed …)` for Fractal (OpenFile),
and `AppearanceSent` to Fractal (the signal). The checks were then restored and
all seven tests pass.

## Verification

Run on the Mac lane's Lima VM (`alo`, Ubuntu 24.04 aarch64, kernel
`7.0.0-31-generic`, `dbus-daemon` 1.14.10, as root), with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-portals`: clean.
- `cargo test -p alo-portals`: all pass (29 unit; integration 7 + 9 + 5 + 5 + 5;
  1 doc).
- `cargo test -p alo-secrets`: all pass. The ignored tests are ignored as before.
- `dbus-send … GetConnectionCredentials string::1.0` against a fresh
  `dbus-daemon --session`: the answer has `ProcessID`, `UnixUserID`,
  `UnixGroupIDs` and `LinuxSecurityLabel`, and no `ProcessFD` (the quirk).

On the Mac: `cargo clippy -p alo-portals --lib -- -D warnings` is clean, so the
cross-platform half of `sandboxed.rs` still builds away from Linux.

**Not run:** the full workspace suite, which the supervisor runs.
**Not measured:** which other bus daemons (newer `dbus-daemon`, `dbus-broker`)
send `ProcessFD`. The code uses it when it is there. The `given` path is covered
by unit tests with real pidfds, not by a daemon that sends one.
**Not physical acceptance:** no real Flatpak application. The processes, the
pidfds, the bus and the reaping are real; the sandbox is a file in a directory
laid out like `/proc`, as in tasks 5 and 6.

## Limitations

- **With a daemon that sends no `ProcessFD`, one narrow window remains**
  (decision 3): the caller ends, and its number is reused, before the descriptor
  is opened, and the daemon still reports the dead connection when asked again.
  Shipping a bus daemon that sends `ProcessFD` closes it. That choice belongs to
  the image, not this plan.
- **`.flatpak-info` is still read by path, through `/proc/<number>/root`.** This
  is correct because of decision 2, not because the file is opened through the
  descriptor. Linux has no stable way to open a process's root from a pidfd.
- A caller that has become a zombie is `Nobody`, not `Gone`, and the watcher
  does not record it. Its number cannot have been reused, so nothing was misread.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** applications plan task 7 is done. Task 8, *what an
  application was answered is kept after the backend stops*, is written in the
  plan and ready.
- **ROADMAP.md:** nothing moves. This is v0.5 screenless work under ADR 0028.
