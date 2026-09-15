# v0.5 — applications, and what they expect, before there is a dialog to show

**Workstream:** the `[v0.5]` promises under *Software, and what applications
expect* in `docs/features.md` (ADR 0005) that are decisions rather than
dialogs: *★ one list of what has been granted to what — agents and
applications in the same place, revoked the same way*; each portal request
being *a grant in the sense of ADR 0001*; *secret storage — one keyring behind
the Secret portal*; and *file associations — what opens what, changeable by a
person*. The file chooser that a portal opens is the desktop lane's. What it
means for an application to have been allowed something is this plan's.
**Why it exists:** [ADR 0028](../decisions/0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
begins v0.5's screenless work while v0.01 waits on hardware. Lane B's second
partition, *the machine, measured*, finished on 2026-09-13; this is its third,
chosen because it shares no crate with lane A's local-network plan or the Mac
lane's models plan.

**Crates this plan owns:** a new `crates/alo-portals` for what a portal
request is and how it is answered, `crates/alo-granted` for the one list,
`crates/alo-applications` for what opens what, and `crates/alo-secrets` for
the keyring behind the Secret portal. **Since 2026-09-15 it also owns the
change [ADR 0040](../decisions/0040-what-an-applications-grant-is-over.md)
makes to `crates/alo-capability` and to the grants file in
`crates/alo-remembering`**, and only that change: a closed list of what an
application may reach that is not a path, a grantee that is an agent or an
application, application grants that outlive declining the agent, and the
grants file's new format. Anything else in those two crates is not this
plan's. Nothing in `crates/alo-shell`, nothing in `image/`, nothing in
`alo-nearby`, `alo-asking`, `alo-record`, `alo-turn`, `alo-egress` (lane A's),
nothing in
`alo-models`, `alo-driving`, `alo-choosing`, `alo-answering`, `alo-telling`
(the Mac's), and nothing in `alo-finding` or `alo-measuring` (finished, and
left alone).

**What this plan may not do** (ADR 0028's terms): move any v0.01 box, line or
wording; tick anything *on the machine*; edit a crate another lane owns. Before
writing the next task, `git pull` and read the plan as published — numbers are
a shared space, and two lanes have taken the same one twice.

## Tasks

### 1. A portal request is a grant, and is refused like one

**Status:** **Done, 2026-09-15.** Built on
`docs/decisions/0040-what-an-applications-grant-is-over.md` (accepted
2026-09-15, option C, all four parts). `alo-capability` gained a closed
`Facility` reach granted only to applications, a `Grantee` that is an agent or
an application with an `Applicant` door (`Grants::allowing`) whose refusal
(`NotAllowed`) never calls it an agent, and application grants that survive
`Agent::declining`. The grants file in `alo-remembering` reads formats 1 and 2
and writes the lowest that holds the list (`docs/contracts/grants-file.md`).
`crates/alo-portals` names the fifteen v0.5 portals and judges a request with
`Grants::allows_anything` and `Grants::allowing`. The waiting test
`crates/alo-granted/tests/a_portal_grant_waits_on_its_decision.rs` was retired;
what it held about the ADR's table and the features line is now held by
`crates/alo-portals/tests/a_portal_request_is_a_grant.rs`. Report:
`docs/autonomy/updates/portal-requests-judged-as-grants.md`.
**Depends on:** nothing.

**Decided rather than built, 2026-09-15.** The first worker found that the
acceptance below cannot be met without editing `alo-capability`, and this plan
never edits it. Four facts from `main` block it:

- `Reach` is a folder, a file or an application. Eleven of the fifteen v0.5
  portals ask about none of those: the camera, the microphone, the screen, the
  person's notifications, and so on.
- A camera as its `/dev/videoN` path can come to mean another camera after a
  replug.
- Grants live inside `Agent`, so declining the agent (ADR 0009) would end every
  application's grants.
- A `Grantee` and its refusals speak of an agent, and tell a person to pick a
  folder.

ADR 0040 sets out three options, what each costs, and recommends that
`alo-capability`, through the lane that owns it, learns a closed list of what a
portal may reach, a grantee that is an agent or an application, and application
grants that outlive declining the agent. The task is ready again once three
things happen: the owner answers, the capability change is made by the crate's
owner or moved into this plan in writing, and the grants file moves to a new
format. `crates/alo-granted/tests/a_portal_grant_waits_on_its_decision.rs`
held the ADR in place while it was *proposed* (retired once task 1 was built).
Report:
`docs/autonomy/updates/portal-grants-decided-before-they-are-built.md`.

*Applications install sandboxed and reach the system through the XDG Desktop
Portal interfaces… each portal request is a grant in the sense of ADR 0001.*
This is that sentence as a type. An application asking to open a file, read
the clipboard or show a notification is asking for exactly what an agent asks
for when it calls a verb: a bounded thing, decided against what the person
allowed, recorded, and revocable.

- **Acceptance:** `alo-portals` names the portal interfaces `docs/features.md`
  lists for v0.5 as a closed enum — file chooser and documents, open-with and
  default applications, notifications, print, screenshot, screen capture,
  camera, microphone, clipboard, trash, wallpaper, settings, inhibit, network
  and power-profile monitors — with a sentence each in the vocabulary; a
  request from a named application for one of them is evaluated the way
  `alo-capability` evaluates a verb — against a grant naming the application
  and the reach, refused outside it, never widened — and the evaluation reuses
  that crate's types rather than restating its rules; a request from an
  application nobody has granted anything is refused before any dialog is
  imagined; and what a request is *for* — a file, a folder, a device — is the
  same `Reach` an agent's grant names, so one list can hold both.
- **Constraint:** no D-Bus, no socket, no dialog. This task decides; task 5
  speaks. The enum is closed and the v1 portals (USB, global shortcuts,
  launchers, remote desktop) are absent from it rather than present and
  refused — a portal this machine does not offer is not a portal it lists.

### 2. One list of what has been granted to what

**Status:** **Done, 2026-09-15.** `alo-granted`'s rows are one shape for
both kinds: a `Seen` keeps the grantee's name and nothing about its kind, is
worded by one clause (`granted.one-grant`, now *{who} has been granted
{what}*), and is revoked by the one `Seen::revoke`; `Seen::revoke_on` is the
same action on a declined machine, where only applications' grants are held.
The list's four sentences no longer speak of agents alone.
`crates/alo-granted/tests/applications_on_the_one_list.rs` holds each clause,
including a revocation with a portal request in flight on another thread.
Report: `docs/autonomy/updates/one-list-of-grants-for-agents-and-applications.md`.
**Depends on:** 1.

*★ One list — agents and applications in the same place, revoked the same
way.* The star is on the word *one*. `alo-granted` already derives the list a
person sees from the machine's own kept grants and revokes through the
mechanism the daemon enforces; this puts applications on the same list, as
rows a person cannot tell from an agent's except by the name.

- **Acceptance:** `alo-granted`'s list holds an application's grants beside an
  agent's, in one order, with one shape per row — who, what, until when — and a
  test that reads a mixed list and finds no row it can tell apart by anything
  but the name; **revoking an application's grant is the same action as
  revoking an agent's**, takes effect at the next portal request, and is tested
  by a revocation with a request in flight; an application that has been
  granted nothing does not appear, because an empty row is a row a person
  would reason about; and every sentence the list shows is in the vocabulary
  `alo-saying` collects.
- **Constraint:** the list is derived, never a second store. `alo-granted`'s
  rule stands: what a surface shows is what the daemon enforces, read from the
  same place, so the list and the machine cannot disagree. Nothing here draws
  the list.

### 3. One keyring behind the Secret portal

**Status:** **Done, 2026-09-15.** `alo-capability` gained a twelfth facility,
`secrets`, and `alo-portals` a sixteenth portal, `Portal::Secret`, held to its
own line of `docs/features.md` (ADR 0040, amendment of 2026-09-15; the grants
file names it `secrets`). `alo-secrets`' `TheKeyring::for_the_application`
refuses any other portal, judges the request with `Request::judged`, and hands
back an `ItsOwn` that does one thing — keep, read, forget, or the portal's
one secret — filed under the allowed application's identifier in the person's
default collection, apart from the provider keys. The keyring fixture starts
every bus from a configuration naming no service directory, closing the
2026-09-13 activation race for itself. Revoking ends an application's reach at
its next request and does not delete what it kept.
`crates/alo-secrets/tests/one_keyring_behind_the_secret_portal.rs` holds each
clause. Report: `docs/autonomy/updates/one-keyring-behind-the-secret-portal.md`.
**Depends on:** 1.

*Secret storage — one keyring behind the Secret portal, so applications stop
inventing credential storage.* `alo-secrets` keeps a provider's key today and
owns the bus that keeps it. This makes that keyring the one an application
reaches through the Secret portal, so a password an application stores lands
where a person's other secrets are, under the same lock, and nowhere else.

- **Acceptance:** an application's request for the Secret portal is a grant
  (task 1) naming the application, and a secret it stores is kept in the same
  keyring `alo-secrets` already uses, in a collection the application's name
  owns, checked by a test that stores as one application and cannot read as
  another; the keyring is the machine's one keyring — a test that starts the
  fixture finds no second Secret Service on the bus, which is the race
  `docs/quirks.md` recorded on 2026-09-13 and this task closes for its own
  fixture; and a secret stored through the portal is revocable with the
  application's grant, gone at the next request.
- **Constraint:** Linux, gated in WSL as `alo-secrets` already is. Nothing
  here reads a secret on an application's behalf for anything but that
  application; the daemon's own keys and an application's are separated by
  collection and by grant, never by hoping.

### 4. What opens what, changeable by a person

**Status:** **Done, 2026-09-15.** `alo_applications::WhatOpensWhat` answers
*what opens this* from a file's bytes, read with `alo_opening::decide` and no
name handed over: the person's choice for the kind (`Chosen`, kept in their own
`what-opens-what.toml` by `alo_applications::keeping` under ADR 0038's rule),
or where they chose none, the first installed application declaring it
(`Declared`, from desktop-entry media types; a later declaration never moves
ahead). The `Opener` says which it was (`Because`), including a choice passed
over because it is not installed. A kind nothing opens, and a file that is a
program, empty, damaged or unrecognised, is `NothingOpens` with a sentence —
never a fallback. `alo_portals::open_with::answered` judges the file's grant
before reading it, takes the opener from `WhatOpensWhat` alone, and judges the
opener's grant (ADR 0040's row). `docs/contracts/person-settings.md` gains the
file's section, held by `crates/alo-applications/tests/the_contract_describes_this_file.rs`.
Tests: `crates/alo-applications/tests/what_opens_what.rs` and
`crates/alo-portals/tests/open_with_is_answered_from_what_opens_what.rs`.
Report: `docs/autonomy/updates/what-opens-what-changeable-by-a-person.md`.
**Depends on:** 1.

*File associations — what opens what, changeable by a person.* And the portal
that asks it: *open-with and default applications*. `alo-applications` knows
the list an agent is checked against; this gives it the answer to *which
application opens this kind of file*, as a setting the person owns.

- **Acceptance:** `alo-applications` answers *what opens this* for a file by
  its kind — the kind read from the file's bytes, the way `alo-finding` reads
  it, never from the extension alone — from an association the person set, or
  from the application's own declaration where the person set none, and says
  which of the two it was; a person's association is written into their own
  settings, read back, and wins over any declaration; an open-with portal
  request (task 1) is answered from this and nowhere else; and the answer for
  a kind nothing opens is a sentence in the vocabulary rather than a
  fallback to a text editor.
- **Constraint:** nothing here launches anything. The answer is a name and a
  reason; running it is a verb with a grant, or a person's click. No
  association is set by an application on its own behalf — the rule
  `docs/features.md` implies with *changeable by a person* is that only a
  person changes it.

### 5. The portal backend speaks D-Bus, and only for what was decided

**Status:** **Done, 2026-09-15.** `alo_portals::serving::Backend::serve_on` owns
`org.freedesktop.portal.Desktop` on a session bus and serves, at
`/org/freedesktop/portal/desktop`, `org.freedesktop.portal.Secret` (version 1)
and `org.freedesktop.portal.OpenURI` (version 3), and registers no other portal.
The caller's application is read from its sandbox. The bus names the process
(`GetConnectionCredentials`), and `alo_portals::Sandboxes` reads
`/proc/<pid>/root/.flatpak-info`. A program with no sandbox is refused as
unidentified. `RetrieveSecret` writes the application's own portal secret through
`alo_portals::KeepsSecrets`, which `alo_secrets::TheKeyring` implements with task
3's door. `OpenFile` resolves the handle, is answered by `open_with::answered`,
and opens the file in its opener through D-Bus activation
(`org.freedesktop.Application.Open`). The response is `0` only when the opener
answered. `OpenURI` and `OpenDirectory` are answered `2`, because nothing decides
them yet. Every answer, and every refusal with its application named, is written
to `alo_portals::Recording` before the response is sent.
`docs/contracts/portals.md` lists what is answered and what is not yet, and is
held to `Portal::answered_on_the_bus`. Tests:
`crates/alo-portals/tests/the_portal_backend_answers_on_a_real_bus.rs` and
`crates/alo-secrets/tests/the_secret_portal_answers_on_a_real_bus.rs`. Report:
`docs/autonomy/updates/the-portal-backend-on-the-session-bus.md`.
**Depends on:** 1, 2, 3, 4.

The contract every Linux application already speaks is
`org.freedesktop.portal.*` on the session bus. This is the backend that
answers it — for the portals decided in tasks 1–4 — so that an existing
application, sandboxed and knowing nothing about alo OS, is answered by the
grants a person made.

- **Acceptance:** `alo-portals` serves `org.freedesktop.portal.Secret`,
  `OpenURI` (open-with) and the request/response shape the portal
  specification requires, on a private session bus a test starts, and a real
  D-Bus client from the test — not a mock — makes each request and receives
  what tasks 3 and 4 decided; a request from an application with no grant
  receives the portal's own refusal response, and the record carries it as
  refused with the application named; the file chooser and everything that
  draws is **not** served here — the backend answers what it can decide and
  declines the rest by not registering for it, listed in the report; and
  `docs/contracts/` gains the one page describing which portals this machine
  answers and which it does not yet.
- **Constraint:** `zbus` is already in the workspace for the keyring; no new
  dependency. Linux, gated in WSL. If a portal cannot be answered without a
  dialog, the honest deliverable is its absence from the list and a sentence
  saying so — never a backend that answers *yes* to keep an application from
  asking again.

### 6. The Settings portal answers appearance, from what the person set

**Status:** **Done, 2026-09-15.** `alo_portals::serving::Backend` now also
serves `org.freedesktop.portal.Settings` (version 2) — `ReadAll`, `ReadOne`,
the deprecated `Read`, and `SettingChanged` — for `org.freedesktop.appearance`
alone. `alo_portals::appearance_settings` decides it without the bus:
`color-scheme` from `Appearance::scheme_at` and `accent-color` from
`Appearance::accent_at`, at the time of day `TheMachine::time_of_day` gives, and
only for an application `allowed` by a grant of the `appearance-settings`
facility, judged before the person's settings are read. **`contrast` is left
out**: `alo-appearance` keeps no contrast preference, and no value is invented.
A refused application, a program with no sandbox, and any other namespace or key
all receive `org.freedesktop.portal.Error.NotFound`, and every read and refusal
is recorded with the application named. `crate::watching_appearance` looks every
second and sends `SettingChanged`, with a destination, only to connections whose
application may read the value at that moment, recording each before it is
sent. `docs/contracts/portals.md` moves `settings` to *answered*. Tests:
`crates/alo-portals/tests/the_settings_portal_answers_appearance.rs`. Report:
`docs/autonomy/updates/the-settings-portal-answers-appearance.md`.
**Depends on:** 5.

*Portals: … settings.* An application that follows light and dark, and the
accent colour, asks `org.freedesktop.portal.Settings`. The request is for a
facility ADR 0040 already names, `appearance settings`. The answer is already
decided by `alo-appearance`, which resolves what the release ships against what
the person changed. Nothing is drawn, so the backend task 5 built can answer it
without a dialog.

- **Acceptance:** the backend serves `org.freedesktop.portal.Settings` —
  `ReadAll`, `ReadOne` and the `SettingChanged` signal — for the
  `org.freedesktop.appearance` namespace only (`color-scheme`, `accent-color`,
  `contrast`), read through `alo-appearance`'s public API and never restated.
  An application granted the `appearance settings` facility receives the
  person's values on a private bus a test starts, with a real client. An
  application not granted it receives the D-Bus error the specification gives
  for an unreadable namespace, and nothing else. That refusal is recorded with
  the application named, as task 5's are. Any other namespace is answered as
  unknown, never with a value. `docs/contracts/portals.md` moves `settings`
  from *not answered yet* to *answered*, in the same change.
- **Constraint:** no new dependency, and no edit to `alo-appearance`, which
  another plan owns. If its public API cannot answer a value without an edit,
  that value is left out and the report says so. Linux, gated in the virtual
  machine like task 5. `SettingChanged` is sent only to an application that
  could read the value at the moment it changed.

### 7. An application is named by the process the bus holds, never by a number that can be reused

**Status:** ready. **Depends on:** 5.

Task 5's report left one window open. The backend asks the bus for the process
behind a request (`GetConnectionCredentials`), gets a process **number**, and
reads `/proc/<pid>/root/.flatpak-info`. Between the bus answering and the file
being read, the process can exit and its number be given to another process,
and the request is then judged as whatever application that other process is.
Task 6 widened the window's use: `SettingChanged` is sent to connections judged
this way. Every portal answer on this machine rests on this one lookup, so it
has to name the process that sent the request and no other.

- **Acceptance:** `alo_portals::Sandboxes` identifies a caller through a
  process descriptor (a pidfd), never a bare number. It takes `ProcessFD` from
  `GetConnectionCredentials` when the bus gives one. When the bus does not, it
  opens one for the number the bus gave and checks, after reading
  `.flatpak-info`, that the process behind the descriptor is still alive and is
  still the connection's process, so a number reused while the file was read is
  refused rather than named. A test reproduces the reuse: a caller whose process
  is gone by the time its sandbox is read is `Unanswered::NotIdentified` and is
  recorded that way, for `OpenURI`, `Secret` and `Settings` and for a
  `SettingChanged` that is then not sent. The legitimate path of every existing
  bus test still passes unchanged. `docs/contracts/portals.md`'s *Who is asking*
  says how the process is held, and `docs/quirks.md` records which bus daemons
  give `ProcessFD` (the test VM's `dbus-daemon` 1.14.10 does not).
- **Constraint:** no `unsafe` and no new crate in the tree. `rustix` is already
  a workspace dependency (`alo-agentd`, `alo-accounts`), and its `pidfd_open` is
  safe; if the features it needs would add a crate to the tree, the report says
  so and the task stops at the ADR that decides it. Nothing here changes
  what a grant covers or who is judged, only that the caller judged is the
  caller that asked. Linux, gated in the virtual machine like task 5.
