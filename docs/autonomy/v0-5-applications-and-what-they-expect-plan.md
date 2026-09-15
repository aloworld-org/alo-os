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

**Status:** **Done, 2026-09-15.** `alo_portals::HeldProcess` holds a process by
a pidfd. `HeldProcess::given` takes the `ProcessFD` a bus sends, and a
`ProcessID` beside it must agree. `HeldProcess::opened_for` opens one with
`rustix::process::pidfd_open` when the bus sends only a number.
`HeldProcess::is_still_alive` reads the descriptor's `Pid:` in
`/proc/self/fdinfo`. `Sandboxes::application_of` now takes a held process and
returns `Sandboxed`: `Named`, `Nobody` or `Gone`. What `.flatpak-info` says
counts only if the process is still alive after the file is read. When the bus
sent no descriptor, `crate::caller` also asks the bus again whether the
connection still has the same number. A `Gone` caller is
`Unanswered::NotIdentified` for every portal. For `SettingChanged`, a connection
whose process is gone is sent nothing and is recorded as not identified (the one
connection the watcher records without sending). `docs/contracts/portals.md`'s
*Who is asking* and `docs/quirks.md` (*`dbus-daemon` 1.14.10 names a caller's
process only by its number*) say so. The test is
`crates/alo-portals/tests/a_caller_is_named_by_the_process_the_bus_holds.rs`:
the caller is another process, its `.flatpak-info` is a named pipe, and the
caller is killed and reaped while the backend waits on the pipe. With the checks
removed, the same test names the gone caller `org.gnome.Fractal` and answers it.
Report: `docs/autonomy/updates/callers-held-by-process-descriptor.md`.
**Depends on:** 5.

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

### 8. What an application was answered is kept after the backend stops

**Status:** **Done, 2026-09-15.** Decided the second way: `alo-portals` keeps its
own file, `/var/lib/alo/portal-answers.jsonl` (`alo_portals::THE_ANSWERS`), and
`alo-record` is not touched. `alo_portals::AnswersFile` is a `Recording` that
appends one synced JSON line per answer (`KeptAnswer`: time, application or
nobody, portal, `KeptOutcome`) after a `{"format":1}` line. It opens and reads
the file under `alo-remembering`'s rules: not a link, a regular file, root's or
this login's, not writable by others, made `0600`, and the folder never made.
A file in a newer format or with no format line is refused and left as it was.
A torn last line is ended before the next answer, and reported by number when
read back. `Recording::keep` now returns `Result<(), NotRecorded>`, and every
door treats a failure as a refusal: response `2`, or `Error.Failed` on Settings,
with no secret written, no file opened and no `SettingChanged` sent. A secret is
buffered and written, and a file handed to its opener, only after its answer is
kept. A failed delivery then follows the answer as `not-written` or
`not-opened`. `docs/contracts/portal-answers-file.md` is the format, and
`docs/contracts/portals.md` says what an unrecorded answer gets. Tests:
`crates/alo-portals/tests/what_an_application_was_answered_is_kept.rs`, and
`on_the_bus::a_caller_gone_is_read_back_from_the_disk_after_the_backend_stops`
in `a_caller_is_named_by_the_process_the_bus_holds.rs`. Report:
`docs/autonomy/updates/portal-answers-kept-after-the-backend-stops.md`.
**Depends on:** 5, 7.

Every answer and refusal the portal backend gives is written to a
`Recording` before the application hears it. The only `Recording` is
`alo_portals::Kept`, which is in memory. Task 5's report left this open: when
the backend stops, what applications were refused is gone, and *every execution
and every refusal leaves a record* is a promise about what a person can read
later, not about a process's memory. The record file (`alo-record`) has no kind
of entry for an application. ADR 0040 part 2 rules out writing one under the
agent column, and `alo-record` is lane A's.

- **Acceptance:** the backend's answers are kept durably and read back in the
  order given, each with its time, the application named (or nobody, for a
  caller not identified), the portal and the outcome. A backend started again
  over the same record finds what the last one wrote. Every refusal kind task
  5–7's tests produce — not granted, not identified, a caller gone — is written
  and read back, and a test shows that an answer the record could not keep is
  never sent to the application. Where it is kept is decided first. Either
  `alo-record` gains an additive application kind, which needs that lane's
  owner, so this task's deliverable is then the ADR with the options and a
  recommendation. Or `alo-portals` keeps its own file under the rules
  `alo-remembering` follows for the person's other kept files, with its format
  in `docs/contracts/`. The report says which, and why.
- **Constraint:** no edit to `alo-record` without its owner's written
  agreement. No new crate in the tree. Nothing here starts the backend in a
  session: that is image work and not this plan's. A record that cannot be
  written refuses the request rather than answering it unrecorded.

### 9. What applications were answered is kept as long as the machine's record, and no longer

**Status:** **Done, 2026-09-15.** `alo_portals::AnswersFile::shortened(keeping,
now)` shortens the answers file under `alo_keeping::Keeping`, asking it
`oldest_kept` and `keeps` and restating neither; it returns `Shortened`
(removed, kept, since). Only an answer that reads and is older than the rule
goes. A line that did not read stays byte for byte in its place, a torn last
line is ended and stays, and a file that would lose nothing is not rewritten.
The first line (`crate::answers_head`) gains `since` — the later of the rule's
edge and any earlier `since` — and `under`, with `format` still `1`, and
`ReadBack` reads both back. The shortened file is written new beside the file
(`portal-answers.jsonl.shortening`, `0600`, never through a link), synced, and
renamed over it. It then becomes the file answers are appended to. The file's
lock is held from the first read to the rename, so no answer is kept or sent
meanwhile. A path that no longer holds the open file is `NotRecorded::Replaced`,
and every refusal leaves the file as it was. `docs/contracts/portal-answers-file.md`
gains the two fields and *Shortening it*. `alo-keeping` and `alo-record` are
not edited. Test:
`crates/alo-portals/tests/what_applications_were_answered_is_kept_as_long_as_the_record.rs`.
Report: `docs/autonomy/updates/portal-answers-kept-as-long-as-the-record.md`.
**Depends on:** 8.

Task 8 made the portal answers file durable, and it only grows. The agent's
record does not only grow: `[record].keeping` in the machine description
(`docs/contracts/machine-description.md`) is the organisation's retention rule
(ADR 0004), `"forever"` or `{ for-days = n }`, and `alo-keeping` shortens the
record under it and says so in its first line (`since`, `under`). An
application's requests are as much a record of what happened on a staff
member's machine as an agent's verbs. If one file is kept for ninety days and
the other forever, the organisation's rule holds for half of what the machine
recorded.

- **Acceptance:** the answers file is shortened under the same
  `[record].keeping` rule the agent's record is, read through `alo-keeping`'s
  public `Keeping` rather than restated. A shortening removes only answers
  older than the rule allows, and never a line that did not read. It writes
  `since` and `under` into the format line, as the record file does, so a
  shortened file never reads as one where nothing happened before its first
  answer. It replaces the file whole or not at all, and the backend answers
  nothing while it is replaced. `docs/contracts/portal-answers-file.md` gains the
  two fields additively, and `format` stays `1`. Tests: an answer past the rule
  is gone after a shortening and one inside it is not; `"forever"` removes
  nothing; a torn line survives; a file shortened twice still says it was; and a
  shortening that cannot write leaves the file as it was.
- **Constraint:** no edit to `alo-keeping` or `alo-record` without their
  owner's written agreement. If `Keeping` cannot be used without one, the
  deliverable is the proposal in the report. Nothing here starts the backend in
  a session or decides when a shortening runs on a machine: that is the session
  and image work this plan does not own.

### 10. What applications were answered reads back in the person's language

**Status:** ready. **Depends on:** 8, 9.

The answers file holds identities, never sentences
(`docs/contracts/portal-answers-file.md`), so a person asking *what did my
applications ask for, and what were they told* still has nobody to answer them
in words. `alo_portals::Outcome::said` words an answer while the backend holds
it, but `KeptAnswer` and `ReadBack`, which are what survives the backend
stopping, say nothing. Neither does a file shortened under task 9, whose `since`
is the difference between *no application asked anything in March* and *this
file does not reach March*. `alo-keeping`'s `Head::said` is that sentence for
the agent's record.

- **Acceptance:** `KeptAnswer` is worded through `alo-strings` in the language
  the person reads: the application or *an application that could not be
  named*, the portal's sentence from task 1, and what it was answered with,
  reusing the words `Outcome::said` already uses rather than a second set. A
  test words an answer of every `KeptOutcome` kind and finds no key shown in
  place of a sentence. `ReadBack` says whether the file is whole or shortened,
  and from when, with the moment kept as a moment rather than written into the
  sentence. Lines that did not read are said as a count beside everything that
  did, never dropped. Every new string is declared in `alo_portals::words` and
  collected by `alo-saying`, tested as the existing words are.
- **Constraint:** nothing here draws a list, starts the backend in a session,
  or decides who may read the file. No edit to `alo-keeping`, `alo-record` or
  `alo-granted`. No new dependency.
