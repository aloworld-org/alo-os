# The Settings portal answers appearance, from what the person set

**Date:** 2026-09-15
**Workstream:** v0.5 applications, and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 6: *the
Settings portal answers appearance, from what the person set*)
**Contributor:** Claude (Mac lane worker, Lima VM)
**Status:** ready for integration

## What changed

An application that follows light and dark, and the accent colour, can now ask
alo OS over the standard Settings portal. It gets the person's own settings, and
it gets them only if it was granted the appearance settings. When the person
turns dark, or their schedule does it at six, a granted application is told. An
application that was not granted is told nothing: not the values, and not the
change.

### Change description (for CHANGELOG.md)

> **Applications follow your light and dark.** alo OS now answers
> `org.freedesktop.portal.Settings` for the appearance namespace. An application
> granted your appearance settings reads whether the machine is light or dark and
> which accent you chose, resolved at the moment it asks, and is told when either
> changes, including when your schedule turns the machine dark. An application
> without that grant is answered as if the setting did not exist, and is never
> sent a change. No other setting on the machine is answered. Every read and
> every refusal is recorded with the application named.

### Source

`crates/alo-portals`:

- `src/appearance_settings.rs` (every platform). What is answered, and to whom,
  with no bus involved. `THE_NAMESPACE`, `Setting` (`color-scheme` and
  `accent-color`), `Value` and how the specification writes each value.
  `reaches` and `asked_of` for `ReadAll`'s namespace patterns. `allowed` judges
  a caller with `Request::judged` for `Portal::Settings`. `read` resolves the
  values through `Appearance::scheme_at` and `Appearance::accent_at`. `changed`
  compares two readings.
- `src/settings_portal.rs` (Linux). `org.freedesktop.portal.Settings` version 2:
  `ReadAll`, `ReadOne`, the deprecated `Read`, the `SettingChanged` signal, and
  the `version` property. Errors use the portal's own names.
- `src/watching_appearance.rs` (Linux). Looks every `WATCHED_EVERY` (one second)
  and sends `SettingChanged` to each connection whose application may read the
  value at that moment.
- `src/caller.rs` (Linux). Identifies the caller, moved out of `asked.rs`
  because the Settings portal and the watcher identify callers too and neither
  is a request with a handle (law 4).
- `src/serving.rs`. Registers the Settings portal. Starts the watcher, and
  `Served`'s `Drop` stops it and waits for it to finish.
- `src/the_machine.rs`. `TheMachine` gains `appearance()` and `time_of_day()`.
  It re-exports `Appearance` and `TimeOfDay`, so an implementor needs no
  dependency on `alo-appearance` to name them.
- `src/answered.rs`. `Outcome::AppearanceRead`, `Outcome::AppearanceSent`,
  `Unanswered::NoSuchSetting`, `Unanswered::AppearanceUnread`.
- `src/words.rs`. Four new sentences (38 in all), collected by `alo-saying`.
- `src/portal.rs`. `Portal::Settings.answered_on_the_bus()` is
  `org.freedesktop.portal.Settings`.
- `Cargo.toml`. `alo-appearance`, a workspace crate. `Cargo.lock` gains one edge
  and no package.

`crates/alo-secrets/tests/the_secret_portal_answers_on_a_real_bus.rs` and
`crates/alo-portals/tests/the_portal_backend_answers_on_a_real_bus.rs` implement
the two new `TheMachine` methods and are otherwise unchanged.

`docs/contracts/portals.md` moves `settings` from *not answered yet* to
*answered*. Its row says what each method answers, that `contrast` is not
answered, the errors, and who is sent `SettingChanged`.

## Decisions

1. **`contrast` is left out.** The plan says to leave out a value that
   `alo-appearance`'s public API cannot answer. The specification's `contrast`
   is a person's preference for higher contrast. `alo-appearance` keeps no such
   preference: its `contrast` module measures two colours against EN 301 549,
   and `Changes` has no contrast setting. Answering `0`, *no preference*, would
   report something the person never set. `Setting` has no variant for it, so
   `ReadOne(…, "contrast")` is `NotFound` and `ReadAll` does not list it. ADR
   0040 lists contrast as part of the facility. When `alo-appearance` keeps the
   preference, adding a third variant is additive.

2. **`color-scheme` is never `0`.** The machine always shows light or dark. Which
   one is the person's setting, or the release's default resolved through their
   schedule, and either way it is what the application should match.

3. **`accent-color` is the value for the scheme currently showing.**
   `Appearance::accent_at` decides it, so a rose accent in the evening is the
   dark-ground rose. The portal carries one colour. Sending the light-ground
   value at night would put back the legibility problem that crate exists to
   prevent.

4. **One error for refused and for unknown.** The specification's error for a
   setting that cannot be read is `org.freedesktop.portal.Error.NotFound`. An
   application without the grant gets exactly that, with the same message text
   as a namespace that does not exist. It cannot tell *not granted* apart from
   *not there*. The record can tell them apart. A program with no sandbox is
   refused the same way, as task 5 decided. Grants or settings that could not be
   read are `org.freedesktop.portal.Error.Failed`, because nothing was refused.

5. **Unknown before judged, judged before read.** A namespace or key this
   machine does not answer gets `NotFound` from every caller, granted or not,
   and no grant is looked up. For the appearance namespace, the caller is judged
   first and the person's settings are read only after it is allowed. A unit
   test checks that no refusal reads the appearance.
   `ReadAll` of patterns that reach nothing answered is an empty dictionary, as
   the specification gives. `ReadAll` of patterns that reach the appearance
   namespace, from a caller that is refused, is `NotFound`, never a dictionary
   with the namespace missing.

6. **The deprecated `Read` is served.** The acceptance names `ReadAll` and
   `ReadOne`. `Read` is a method of the same interface and version, older
   applications (GTK 3, libhandy) still call it, and it answers exactly what
   `ReadOne` answers, wrapped in one more variant. Leaving it out would give
   those applications `UnknownMethod`, and nothing would be protected by that.

7. **`SettingChanged` is found by looking, not by being told.** A value changes
   two ways: the person changes a setting, or the clock crosses their schedule
   and no file changes. Nothing notifies the backend of either, and the service
   that will write `appearance.toml` belongs to another lane. So the watcher
   reads `TheMachine::appearance` and `time_of_day` once a second and compares
   the result with the previous reading. The first reading is taken before the
   thread starts, so a change made just as serving begins still counts as a
   change. A reading that fails does not count as a change.

8. **Sent to each connection that may read, never broadcast.** When a value
   moves, the watcher lists the bus's connections (`ListNames`), identifies each
   by its sandbox, and judges it at that moment with `allowed`, the same
   judgement a `ReadOne` gets. Each signal has a destination. A connection that
   has not read anything yet but is listening is still sent the change if its
   application is granted. The backend's own connection is skipped.

9. **Signals are recorded before they are sent, and withheld signals are not
   recorded.** Every `SettingChanged` is recorded as `AppearanceSent` with the
   application named, and then sent. This is the order task 5 uses for
   `Response`: *recorded, then sent*. A connection that was not sent anything
   made no request, so nothing is recorded for it. Recording every program on
   the bus at every change would bury the refusals that matter.

10. **The time of day comes from `TheMachine`.** `alo-appearance` never reads a
    clock, and the local time of day (time zone included) belongs to the machine.
    `TheMachine::time_of_day() -> Option<TimeOfDay>` and a `None` refuses the
    request as `AppearanceUnread`, like settings that did not read. Nothing
    implements `TheMachine` on a real machine yet (task 5's limitation). When the
    service that runs the backend exists, it provides the clock.

## Acceptance, clause by clause

| Clause | Test |
|---|---|
| The backend serves `Settings` — `ReadAll`, `ReadOne` and `SettingChanged` — for `org.freedesktop.appearance` only, read through `alo-appearance`'s public API; a granted application receives the person's values on a private bus with a real client | `alo-portals` `the_settings_portal_answers_appearance` `on_the_bus::a_granted_application_reads_the_persons_appearance` |
| An application not granted receives the specification's error for an unreadable namespace and nothing else, recorded with the application named | `alo-portals` `the_settings_portal_answers_appearance` `on_the_bus::an_application_not_granted_receives_not_found_and_is_recorded_by_name` |
| Any other namespace is answered as unknown, never with a value | `alo-portals` `the_settings_portal_answers_appearance` `on_the_bus::any_other_namespace_is_answered_as_unknown_and_never_with_a_value` |
| `SettingChanged` is sent only to an application that could read the value at the moment it changed | `alo-portals` `the_settings_portal_answers_appearance` `on_the_bus::a_change_is_sent_only_to_an_application_that_could_read_it` |
| `docs/contracts/portals.md` moves `settings` to *answered* in the same change | `alo-portals` `the_settings_portal_answers_appearance` `the_contract_lists_settings_as_answered_with_what_it_answers`, and task 5's `the_portal_backend_answers_on_a_real_bus::the_contract_names_every_portal_answered_and_every_one_not` |
| Values are what `alo-appearance` resolves, never restated; judging never reads the settings; unread settings are refused, never defaulted | `alo-portals` lib `appearance_settings::tests::the_values_are_the_persons_resolved_at_the_time_given`, `…::only_an_application_granted_appearance_settings_is_allowed`, `…::unread_settings_are_refused_rather_than_defaulted` |
| Constraint: no new dependency; no edit to `alo-appearance` | `alo-appearance` is a workspace crate, `Cargo.lock` gains one edge and no package; `git diff` touches nothing under `crates/alo-appearance` |

Refusal paths on the bus: an application granted nothing; an application granted
a different facility (the camera); a grant revoked between two reads; a grant
that expired; a program with no sandbox. Each is checked through `ReadOne` for
both keys, through `ReadAll` of the namespace and of everything, and through
`Read`: 25 refusals, each recorded. Unknown settings: three other namespaces,
`contrast`, a made-up key, and three `ReadAll` patterns that reach nothing,
asked by a granted application and by a stranger. `SettingChanged`: a clock
crossing while the caller is a stranger, the evening arriving after the grant was
revoked, and a change while the caller has no sandbox. None of them arrives, and
in the first two cases the next signal that does arrive is the next change the
application could read.

**The change test was checked against a broken backend.** With the watcher
changed to send to connections that were not allowed, the test failed with
`the change made while it was a stranger reached it`. The watcher was then put
back.

## Verification

Run on the Mac lane's Lima VM (`alo`, Ubuntu 24.04 aarch64, `dbus-daemon`
1.14.10, as root), with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`:

- `cargo fmt --all`: clean.
- `cargo clippy -p alo-portals -p alo-secrets --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-portals`: clean.
- `cargo test -p alo-portals`: all pass (25 unit, 9 + 5 + 5 + 5 integration, 1
  doc). The new file's five tests take 13 s, most of it waiting for the
  one-second watcher.
- `cargo test -p alo-secrets`: all pass. Its test was edited for the trait. The
  ignored tests are ignored as before.
- `cargo test -p alo-saying`: all pass. It was not edited, and it collects the
  four new sentences.

On the Mac: `cargo clippy -p alo-portals --lib -- -D warnings` is clean, so the
cross-platform `appearance_settings` builds away from Linux. `--all-targets` on
macOS fails in `alo-sessiond` (`rustix::net::sockopt::socket_peercred`), and it
fails the same way with this change stashed, so it was already failing.

**Not run:** the full workspace suite, which the supervisor runs.
**Not physical acceptance:** no real Flatpak application or signed-in desktop
session. The bus, the client and the signals are real D-Bus traffic. The sandbox
is read from a directory laid out like `/proc`, as in task 5.

## Limitations

- **`contrast` is not answered** until `alo-appearance` keeps a contrast
  preference (decision 1). Proposed to that crate's owner: a person's
  higher-contrast preference in `Changes`, which this portal would answer as `1`.
- **Nothing implements `TheMachine` on a real machine yet.** The local time of
  day, and reading `appearance.toml` through `alo_appearance::keeping`, arrive
  with the session service that runs the backend. That is not this plan's.
- **A change is noticed within a second**, not the moment it happens. A change
  that is made and undone within one look is never sent, which is correct:
  nothing an application could read changed.
- **An application is identified by process number** (task 5's limitation). The
  watcher now relies on that lookup too, so task 7 closes the reuse window with a
  process descriptor.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** applications plan task 6 is done. Task 7 (*an
  application is named by the process the bus holds, never by a number that can
  be reused*) is written in the plan and ready.
- **ROADMAP.md:** nothing moves. This is v0.5 screenless work under ADR 0028.
