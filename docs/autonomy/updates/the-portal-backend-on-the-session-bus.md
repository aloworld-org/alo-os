# The portal backend on the session bus

**Date:** 2026-09-15
**Workstream:** v0.5 applications, and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 5: *the
portal backend speaks D-Bus, and only for what was decided*)
**Contributor:** Claude (Mac lane worker, Lima VM)
**Status:** ready for integration

## What changed

An application written for any Linux desktop can now ask alo OS for its own
secret and for a file to be opened, over the standard portal interfaces, and it
is answered from the grants the person made. The backend answers only the two
portals whose answers were decided without a dialog. Every other portal is not
registered on the bus, so an application is told that nothing answers it, and
never told *yes*.

### Change description (for CHANGELOG.md)

> **Applications are answered over the standard portals.** alo OS now answers
> `org.freedesktop.portal.Secret` and `org.freedesktop.portal.OpenURI` on the
> session bus. A sandboxed application is named by its sandbox, never by what it
> says about itself. A secret it asks for comes from the person's one keyring and
> only with a grant. A file it asks to have opened is opened in the application
> the person chose for that kind of file, and only when both the file and that
> application were granted. Every answer is recorded, including every refusal,
> with the application named. The file chooser and every other portal that
> needs a dialog is not answered yet, and `docs/contracts/portals.md` lists
> which portals are answered and which are not.

### Source

`crates/alo-portals`:

- `src/serving.rs`, Linux only. `Backend::answering_from(machine, keyring,
  sandboxes, record)` and `Backend::serve_on(address) -> Served`. The backend
  owns `org.freedesktop.portal.Desktop`, and a second backend on the same bus is
  refused with `NotServed::NameTaken`.
- `src/secret_portal.rs`, Linux only. `RetrieveSecret(h, a{sv}) -> o`, version 1.
- `src/open_uri_portal.rs`, Linux only. `OpenFile`, `OpenURI` and
  `OpenDirectory`, version 3.
- `src/opening.rs`, Linux only. Asks the opener to open the file with
  `org.freedesktop.Application.Open`, which is D-Bus activation. No command is
  run.
- `src/asked.rs`, Linux only. Identifies the caller and makes its handle.
  Records the answer, then sends `Response`.
- `src/sandboxed.rs`. `Sandboxes` reads `/proc/<pid>/root/.flatpak-info`.
- `src/handle.rs`. The request handle path from the specification, and the
  `handle_token` check.
- `src/answered.rs`. `Answered`, `Outcome` and `Unanswered`, which are what the
  record keeps, with response codes and sentences.
- `src/recording.rs`. The `Recording` trait, and `Kept`, which keeps the record
  in memory.
- `src/the_machine.rs`. The `TheMachine` trait, which gives grants and
  `Applications` and is read at every request.
- `src/keeping_secrets.rs`. The `KeepsSecrets` trait and `NotKept`.
- `src/portal.rs`. `Portal::answered_on_the_bus`.
- `src/words.rs`. Eleven new sentences (34 in all), collected by `alo-saying`.
- `Cargo.toml`. `zbus` for Linux, at the version and runtime `alo-secrets`
  already uses. `alo-keyring-fixture` as a Linux dev-dependency.

`crates/alo-secrets`:

- `src/behind_the_portal.rs`. `impl alo_portals::KeepsSecrets for TheKeyring`,
  using task 3's `for_the_application` and `the_portals_secret` unchanged.

`docs/contracts/portals.md` is new: which portals are answered, on which
interface and version, how an answer arrives, and why each of the other
fourteen is not answered yet.

## Decisions

1. **The backend is the portal frontend.** The acceptance names
   `org.freedesktop.portal.Secret` and `OpenURI`. `OpenURI` exists only on the
   frontend, because upstream `xdg-desktop-portal` implements it itself on top of
   the `AppChooser` dialog backend. So this backend owns
   `org.freedesktop.portal.Desktop` rather than sitting behind
   `xdg-desktop-portal` as an `org.freedesktop.impl.portal.*` backend.
   Consequence: a machine runs this backend in place of `xdg-desktop-portal`, not
   beside it. The name check (`NotServed::NameTaken`) makes sure only one of them
   answers.

2. **Who is asking comes from the sandbox.** The bus daemon says which process
   sent the request (`GetConnectionCredentials`). `Sandboxes` reads that
   process's `.flatpak-info` through `/proc/<pid>/root`, which is how
   `xdg-desktop-portal` does it. A process with no sandbox is refused as
   `Unanswered::NotIdentified`. There is nothing to judge it against, and an
   unconfined program already reaches what the person can reach. **Known
   limit:** a process running as the person can make its own mount namespace and
   forge the file. It gains nothing it lacked before (ADR 0022's limitation). A
   sandboxed application cannot forge it. A pidfd-based lookup would close the
   pid-reuse window, and is left for a follow-up.

3. **Traits where a crate edge would otherwise form a cycle.** `alo-secrets`
   already depends on `alo-portals`, so the backend reaches the keyring through
   `KeepsSecrets`, and `TheKeyring` implements it. The keyring still judges the
   request itself. The secret is written straight into the application's pipe
   and never returned as a value. Grants and *what opens what* are read through
   `TheMachine` at **every request**, so a revocation takes effect at the next
   request. Grants that cannot be read refuse the request (`GrantsUnread`). An
   unreadable list of choices is never replaced by an empty one
   (`ApplicationsUnread`).

4. **`OpenFile` opens through D-Bus activation, and `0` means it opened.** Task
   4 decided *which* application, and its constraint was that nothing there
   launches. A backend that answered `0` without opening would be the *yes to
   stop an application asking* the task forbids. So the backend asks the opener
   with `org.freedesktop.Application.Open`. That is a message to an application
   on the bus, and no command is run and no `Exec` line is read. The response is
   `0` only when the opener replied. An opener that is not D-Bus activatable is
   answered `2` and recorded as `NotOpened`, naming it. Scope was cut here
   deliberately: launching from `Exec` lines belongs to the launcher.

5. **`OpenURI` and `OpenDirectory` are answered `2`, not left unregistered.**
   They are methods of the same interface as `OpenFile`, so the only way to not
   register them would be to not register open-with. Nothing decides what opens
   a web link or a folder, so both are refused and recorded as `NotDecidedHere`.

6. **`O_PATH` handles are handled.** GTK and libportal pass `OpenFile` a handle
   opened with `O_PATH`. The backend `fstat`s it and refuses anything that is
   not a regular file, including folders and pipes. It resolves the path with
   `/proc/self/fd/N` and reopens it the same way to read. The file's bytes are
   still read only after `open_with::answered` has judged the grant over the
   file.

7. **The response is sent before the method returns.** Every answer is decided
   while the call is handled. The specification tells clients to listen on the
   handle predicted from `handle_token` before calling, and GTK, libportal and
   libsecret all do. `docs/contracts/portals.md` says so. An invalid
   `handle_token` is refused on the call with `InvalidArgs` and recorded, because
   no handle path can be made from it.

8. **The record is the portals' own, not `alo-record`.** See *Limitations*.
   `alo_record::Entry` has no kind of entry for an application. Its constructors
   put a name in the `agent` column, and `Only::ByAnAgent` is how a person who
   declined the agent checks that nothing an agent did is in there. Writing
   portal requests under that column would give a false answer, which ADR 0040
   part 2 rules out, and `alo-record` is lane A's crate. So every answer is an
   `alo_portals::Answered` written to a `Recording`. `Kept` holds it in memory,
   the same way `alo_record::Record` does.

9. **The response is never `1`.** Nobody is asked anything, so no person
   cancelled. Every refusal is `2`, the specification's *ended in some other
   way*.

## Acceptance, clause by clause

| Clause | Test |
|---|---|
| Serves `Secret` on a private bus a test starts; a real client makes the request and receives what task 3 decided | `alo-secrets` `the_secret_portal_answers_on_a_real_bus::a_granted_application_is_handed_its_own_secret_through_the_portal` |
| Serves `OpenURI` (open-with) the same way and the client receives what task 4 decided | `alo-portals` `the_portal_backend_answers_on_a_real_bus::on_the_bus::a_granted_file_is_opened_in_what_opens_it_and_answered_as_done` |
| An application with no grant receives the portal's refusal response, and the record carries it as refused with the application named (Secret) | `alo-secrets` `the_secret_portal_answers_on_a_real_bus::an_application_granted_nothing_is_refused_on_the_bus_and_recorded_by_name` |
| The same for open-with, with the other refusals beside it | `alo-portals` `the_portal_backend_answers_on_a_real_bus::on_the_bus::every_request_no_grant_covers_is_refused_on_the_bus_and_recorded_by_name` and `…::an_opener_that_does_not_answer_is_recorded_and_refused` |
| The file chooser and everything that draws is not served; the backend declines by not registering | `alo-portals` `the_portal_backend_answers_on_a_real_bus::on_the_bus::only_the_portals_decided_here_are_registered_on_the_bus` |
| `docs/contracts/` gains the page listing which portals are answered and which are not yet | `alo-portals` `the_portal_backend_answers_on_a_real_bus::the_contract_names_every_portal_answered_and_every_one_not` |
| Constraint: no new dependency | `zbus 5.19` with `async-io` and `blocking-api`, already compiled through `alo-secrets`' `secret-service`. The `Cargo.lock` diff adds edges only, no packages. |

Refusal paths covered by the bus tests: an application granted nothing; an
application granted something else (camera, not secrets); a grant revoked between
two requests; a program with no sandbox; an invalid `handle_token`; a file not
granted; an opener not granted; a file nothing opens (a program); a handle that
is not a file (a folder); an opener that does not answer; a web link; and a
folder. Unit tests cover the `.flatpak-info` parser and its size limit, handle
paths and invalid tokens, `file://` escaping, activation object paths, response
codes, the record's order, and the keyring's mapping from `Withheld` to `NotKept`.

### Not registered, and why (also in `docs/contracts/portals.md`)

File chooser and documents, notifications, print, screenshot, screen capture,
camera, microphone, clipboard, trash, wallpaper, settings, inhibit, network
monitor and power-profile monitor. Each either needs a dialog, or needs part of
the machine that does not exist yet: a surface on the desktop, a source for the
devices, or a session.

## Verification

Run on the Mac lane's Lima VM (`alo`, Ubuntu 24.04 aarch64, as root), with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets -- -D warnings` on the whole workspace: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-portals -p alo-secrets`:
  clean.
- `cargo test -p alo-portals`: all pass (19 unit, 9 + 5 + 5 integration, 1 doc).
- `cargo test -p alo-secrets`: all pass, including the 2 new integration tests
  and 1 new unit test. The 3 `a_session_that_really_ended` tests and 1
  `which_bus_is_reached` test stay ignored, as before.
- `cargo test -p alo-saying`: all pass. The crate was not edited. It was run
  because it collects the eleven new sentences.

**Not run:** the full workspace suite, which the supervisor runs.
**Not physical acceptance:** a real Flatpak application, a real
`DBusActivatable` opener and a signed-in desktop session were not measured. The
bus, the keyring, the client and the opener in the tests are real processes and
real D-Bus traffic, but the sandbox's `.flatpak-info` is read from a directory
laid out like `/proc`, because the test cannot start a Flatpak sandbox.

## Limitations

- **The record is in memory.** `Kept` holds answers for as long as the backend
  runs. Nothing writes them to the record file yet, because there is no service
  that runs this backend on a machine yet. Starting it in the person's session,
  and replacing `xdg-desktop-portal`, is image work that this plan does not own.
- **The record file has no kind of entry for an application.** Proposed for the
  owner of `alo-record`: an additive `happened` kind for a portal request, whose
  column says *application* and not *agent*, so that `Only::ByAnAgent` stays
  true. Once it exists, a `Recording` that writes into it replaces `Kept`.
- **Only D-Bus-activatable openers open.** Other applications are refused with
  `NotOpened`.
- **The opener receives a host `file://` path.** A sandboxed opener needs the
  document portal to reach that path, and the document portal is served with the
  file chooser, which is the desktop lane's.
- **Identity comes from `/proc/<pid>`**, so the window between a process exiting
  and its pid being reused is not closed. A pidfd-based lookup is the follow-up.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** applications plan task 5 is done. Task 6 (*the
  Settings portal answers appearance, from what the person set*) is written in
  the plan and ready.
- **ROADMAP.md:** nothing moves. This is v0.5 screenless work under ADR 0028.
