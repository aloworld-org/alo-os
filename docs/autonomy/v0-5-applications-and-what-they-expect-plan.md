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
the keyring behind the Secret portal. **It reads `crates/alo-capability` and
never edits it** — a portal request is a grant in that crate's sense, and this
plan uses that sense rather than restating it. Nothing in `crates/alo-shell`,
nothing in `image/`, nothing in `alo-nearby`, `alo-asking`, `alo-record`,
`alo-capability`, `alo-turn`, `alo-egress` (lane A's), nothing in
`alo-models`, `alo-driving`, `alo-choosing`, `alo-answering`, `alo-telling`
(the Mac's), and nothing in `alo-finding` or `alo-measuring` (finished, and
left alone).

**What this plan may not do** (ADR 0028's terms): move any v0.01 box, line or
wording; tick anything *on the machine*; edit a crate another lane owns. Before
writing the next task, `git pull` and read the plan as published — numbers are
a shared space, and two lanes have taken the same one twice.

## Tasks

### 1. A portal request is a grant, and is refused like one

**Status:** blocked — waits on
`docs/decisions/0040-what-an-applications-grant-is-over.md`, proposed
2026-09-15. **Depends on:** nothing.

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
holds the ADR in place: it fails if `crates/alo-portals` appears while the
decision is still *proposed*. Report:
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

**Status:** ready. **Depends on:** 1.

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

**Status:** ready. **Depends on:** 1.

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

**Status:** ready. **Depends on:** 1.

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

**Status:** ready. **Depends on:** 1, 2, 3, 4.

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
