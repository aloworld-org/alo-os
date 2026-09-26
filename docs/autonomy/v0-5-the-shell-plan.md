# v0.5 — the shell: the half a person looks at

**Workstream:** the fourteen `ROADMAP.md` lines that cannot start without a
screen, beginning with the one v0.01 line still open — **the sign-in screen**
— and then the surfaces that make the agent's promises visible rather than
merely decided.
**Why it exists:** on 2026-09-14 the owner asked what was blocking the
remaining v0.5 work and lifted the rule that reserved `crates/alo-shell` for
the desktop lane, which has been away since before 2026-09-12. Every crate
beneath these surfaces is built, gated and waiting: `alo-greeting` is
*everything the greeter does except drawing*, `alo-approving` is *the sentence
a person approves*, `alo-egress` is *what leaves and the line shown while it
does*, `alo-recounting` is *what a person is told afterwards*. Each of them
says, in its own header, that the screen is somebody else's. This plan is that
somebody.

**Crates this plan owns:** `crates/alo-shell` and `tools/graphics-check`.
Nothing else. Every decision these surfaces render is already made in a crate
this plan **reads and never edits** — `alo-greeting`, `alo-accounts`,
`alo-approving`, `alo-egress`, `alo-indicator`, `alo-recounting`,
`alo-appearance`, `alo-overlay`, `alo-strings`, `alo-saying`. If a surface
needs a decision that is not there, **that is a finding in the report and the
task stays open**; it is never a decision made in a drawing crate.

**If the desktop lane returns**, it pulls `main` like every other lane and
continues from where this got to; nothing here is on a branch and nothing
rewrites what it built. The 133 files it left in `alo-shell` are its work and
this plan adds to them rather than replacing them.

**What this plan may not do:** tick anything *on the machine* — a surface that
has never been seen on a certified machine is `- [x] The code.` and nothing
more; put a sentence on a screen that is not in the vocabulary `alo-saying`
collects (`CLAUDE.md`: hardcoded English is a bug); or name a rented engine
anywhere a person reads. Before writing the next task, `git pull` and read the
plan as published.

**These gates need the graphics libraries** `docs/autonomy/GRAPHICS.md` lists,
in the distribution the gates run in. They are already there — the workspace's
clippy compiles this crate today — and a worker that finds them missing
installs them by that document rather than working around them.

## Tasks

### 1. The sign-in screen

**Status:** **Done, 2026-09-14.** `crates/alo-shell/src/sign_in_*.rs` and
`nested_sign_in.rs`; evidence and decisions in
`docs/autonomy/updates/the-sign-in-screen-drawn-on-the-nested-compositor.md`.
The code only — a certified machine has not seen it, and the fields carry no
label because no crate declares one (a finding in that report).
**Depends on:** nothing.

`ROADMAP.md` v0.01: *boots on one certified machine, firmware to sign-in* —
the last v0.01 line with nothing drawn behind it, and the reason the demo
stops at a console. `alo-greeting`'s own header names what is missing: *a
surface had to authenticate, then knock, then render what came back… a surface
that authenticated and forgot to knock is a screen that takes a correct
password and does nothing at all.* That composition is built and tested. This
draws it.

- **Acceptance:** a surface in `alo-shell` draws a name and a password field
  on the compositor this crate already is, takes keystrokes through the seat it
  already has, and calls `alo_greeting::Greeting` — **never `alo-accounts` or
  `alo-sessiond` directly**, because the order is the composition's and a
  surface that re-implements it is the bug that crate exists to prevent; what
  it shows for each outcome is `Greeting`'s own words through `alo-strings`,
  with a test that every sentence the screen can show is in the vocabulary and
  none is written in this crate; a wrong password says what `alo-greeting`
  says and never how many attempts remain or whether the name exists, and a
  test names that refusal; the password is never drawn, never logged, never in
  a panic message, and is held only as long as the composition needs it — a
  test reads the crate's shipped source for a `Debug` that could print one;
  and when the knock opens a session the surface hands over to it and stops
  drawing, so two things never own the screen at once.
- **Constraint:** nothing here decides who may sign in, what a password is
  worth, or what a session is. No account is created from this screen —
  `alo-setting-up` owns first-run, and *make an account* is its sentence.
  Measured under a nested compositor the way this crate's other tests are;
  a certified machine has never seen it and the report says so.

### 2. The egress indicator, on a screen

**Status:** **Done, 2026-09-14.** `crates/alo-shell/src/egress_status*.rs` and
`nested_egress_status.rs`; evidence and decisions in
`docs/autonomy/updates/the-egress-indicator-drawn-in-the-status-area.md`. The
code only — a certified machine has not seen it, there is no direct-display
submission yet, and the sign-in screen carries no status area (a finding in
that report).
**Depends on:** 1.

`docs/features.md`, ★: *the egress indicator lives in the status area, so
"nothing has left this machine" sits where a person already glances rather
than somewhere they must learn to look.* Law 1's whole promise —
*nothing leaves silently* — has been decided since `alo-egress` and shown to
nobody. `alo-indicator` already says what a screen must show; its
on-the-machine half has been empty since the repository began.

- **Acceptance:** a surface draws `alo_egress::Indicator`'s lines while they
  are live and nothing when it is quiet, in a status area at the far end of
  the dock wherever the dock is; a question answered on this machine draws
  **nothing**, and one answered anywhere else — a provider, or the machine
  down the corridor — draws its sentence, which is the difference the promise
  rests on and is one test with both halves; the line is the agent's name and
  the destination as `alo-egress` words them, never re-worded here; terracotta
  is the colour and it **never arrives alone** — a mark and a word beside it,
  because a signal carried by hue fails for anybody who cannot distinguish it
  and EN 301 549 does not allow colour as the only means (`docs/features.md`,
  ★); and what alo OS does on its own errand is on the same indicator as what
  an agent caused, because *no telemetry* is a promise only somebody who can
  see the machine's unasked traffic is in a position to check.
- **Constraint:** the indicator shows; it never decides. Whether something may
  leave is `alo-egress`'s and is asked before a socket opens, not by this
  surface. No dismissing, no hiding, no setting that turns it off.

### 3. The sentence a person approves

**Status:** **Done, 2026-09-14.** `crates/alo-shell/src/approval_*.rs` and
`nested_approval.rs`; evidence and decisions in
`docs/autonomy/updates/the-sentence-a-person-approves-drawn-on-the-nested-compositor.md`.
The code only — a certified machine has not seen it, answers are given by
keyboard only, and a refusal has no visible acknowledging control because no
crate declares a word for one (findings in that report).
**Depends on:** 1.

ADR 0001: an agent proposes and a person approves, and `alo-approving` is
*one approval, and the sentence a person approves: the change an agent
proposed, put in front of somebody exactly as the turn worded it, answered
once, and carried out.* Exactly as the turn worded it — which means the
surface may not improve on it.

- **Acceptance:** a surface draws one proposal at a time — the sentence
  `alo-approving` carries, unedited, unsummarised, unwrapped into a different
  claim — with two answers and no third, and a test that the text drawn is
  byte-for-byte the text the turn wrote; **nothing is preselected and nothing
  proceeds on silence**, with a test that a surface left alone answers
  nothing; the answer goes back through `alo-approving` once and a second
  answer to the same proposal is refused there rather than here; what a person
  declined says nothing about why, because *"no" is the whole answer and a
  system that recorded a reason would be a system that asked for one*; and a
  proposal that arrives while another is open is queued rather than replacing
  it, so nobody answers a question they did not read.
- **Constraint:** no *approve all*, no *remember this*, no timer. Each of the
  three is the mechanism by which approval becomes a formality, and ADR 0001
  is what they would hollow out. If a person wants an agent to do a class of
  thing without asking, that is a **grant**, made deliberately in
  `alo-picking`, and this surface is not a road to one.

### 4. What the machine did, in front of the person

**Status:** **Done, 2026-09-14.** `crates/alo-shell/src/record_*.rs` and
`nested_record.rs`; evidence and decisions in
`docs/autonomy/updates/what-the-machine-did-drawn-in-the-record-window.md`.
The code only — a certified machine has not seen it, no key or dock item opens
it yet because `alo-shortcuts` declares no action for it, no date is drawn
because no crate decides how a moment is written, and the window has no title
because no crate declares one (findings in that report).
**Depends on:** 3.

`docs/features.md`: *afterwards, ask what it did.* `alo-recounting` composes
the account and has every sentence a person reads; ADR 0009 says the plain
way to reach it must exist beside the agent's. This is the window.

- **Acceptance:** a surface draws `alo_recounting`'s account — each entry's
  clause at the head of its line and the machine's own sentence after it, in
  the order the record has them, newest first — and every clause is the
  vocabulary's rather than this crate's, held by a test; what was **refused**
  is drawn as plainly as what ran, because a record that showed only successes
  cannot answer what a security review asks; an entry that names no agent —
  alo OS's own errand — is drawn without inventing one; the window is reachable
  without asking an agent anything, which is ADR 0009, and a test names that
  road; and asking the agent *what did you do?* reaches the same account rather
  than a second telling of it.
- **Constraint:** the surface reads the record and writes nothing to it. No
  filtering that could hide a refusal by default, no search that changes what
  *today* means, and no summary — `alo-recounting`'s own rule is that a
  surface able to word what happened would be a machine with two accounts of
  one moment.

### 5. The ordinary desktop: the dock, the status area, and what is running

**Status:** **Done, 2026-09-14.** `crates/alo-shell/src/desktop_*.rs`,
`dock_raster.rs`, `running_*.rs`, `filling_*.rs` and `nested_desktop.rs`;
evidence and decisions in
`docs/autonomy/updates/the-ordinary-desktop-drawn-dock-status-area-and-measuring-windows.md`.
The code only — a certified machine has not seen it. **The clock, battery,
network and volume are not in the status area**: no crate measures a battery,
a network's state or a volume, and none decides how a time is written for a
region, so drawing them would be deciding in a drawing crate. That half of the
acceptance is task 7, blocked on those crates, rather than a promise quietly
dropped. Also findings in that report: nothing is in the dock and nothing opens
either window, because no crate decides either; and the edge is one for every
display, because `alo-dock` has no per-display exception yet.
**Depends on:** 2.

`ROADMAP.md` v0.5: *the ordinary desktop* — the furniture a person expects,
and the frame the other surfaces sit in. `alo-dock` decides the dock's edge
and its per-display placement; `alo-appearance` carries the accent set;
`alo-measuring` answers what is running and what is filling the disk, and has
no window.

- **Acceptance:** a dock is drawn on the edge `alo-dock` names, per display,
  with a status area at its far end holding the clock, battery, network,
  volume and the egress indicator of task 2; the accent is `alo-appearance`'s
  and **terracotta is never offered**, because it means the agent and nothing
  else (`docs/features.md`, ★) — a test refuses a palette that offers it; a
  window drawn from `alo-measuring` shows what is running and what is using
  the machine, and a second shows what is filling the disk as sizes that open
  up, each number the one the kernel gave with nothing derived here; and
  everything drawn follows light and dark as `alo-appearance` decides it,
  never as this crate decides it.
- **Constraint:** the dock is furniture and holds no authority: nothing is
  granted, approved or revoked from it. What the status area shows about the
  machine is measured by the crates that measure, and this surface adds no
  number of its own.

### 6. One place for settings

**Status:** **Done, 2026-09-16.** `crates/alo-shell/src/settings_*.rs` and
`nested_settings.rs`; evidence and decisions in
`docs/autonomy/updates/one-place-for-settings-drawn-with-every-section-through-its-own-crate.md`.
The code only — a certified machine has not seen it. Findings in that report:
no crate declares a heading for a section or a label for *put back as shipped*
(Delete does it, and the section's own refusal names it); a provider's models
are not kept in `settings.toml`, so only the provider model a person has chosen
is offered again; appearance offers the accent alone, because nothing else it
keeps has words or a road a window can take; nothing opens Settings yet,
because `alo-shortcuts` declares no action for it; and a session with no folder
offers no change rather than one it would forget, waiting on the keeping
plan's task 7 for its sentence.
Before it was built, the status read: ready — the keeping that
[ADR 0038](../decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
describes has landed: `docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`'s
tasks 2 and 3 were published on 2026-09-15 (appearance, the dock and shortcuts
each keep their own file; a pairing is revoked from the one list the way a grant
is), and its task 6 walked one person's folder from sign-in to the next change.
What blocked this task until then, kept for the record: three of
the seven sections —
appearance, the dock and shortcuts — have no file any crate reads or writes, so
*reading and writing the same file its crate already owns* has nothing to read
or write, and choosing that file here would be the drawing crate deciding. And
a pairing has no person's road to revoke it the way `alo-changing` revokes a
grant, so *revoked the same way* would be two code paths in a compositor.
Evidence and the proposed keeping lane in
`docs/autonomy/updates/one-place-for-settings-waits-on-where-settings-are-kept.md`;
`crates/alo-shell/tests/settings_source.rs` holds, meanwhile, that the shell
keeps no settings file of its own. **Depends on:** 5.

`ROADMAP.md` v0.5: *Settings, as one place — network, display, sound,
printers, storage, keyboard, accounts, privacy, updates. Not a scattering of
dialogues a person has to know the name of.* Every setting this OS has is
already decided in a crate and written into the person's own file; none of
them has ever been shown.

- **Acceptance:** one surface holds every setting the machine has today — what
  a person chose at setup (`alo-setting-up`), the model and provider
  (`alo-choosing`), appearance (`alo-appearance`), the dock (`alo-dock`),
  shortcuts (`alo-shortcuts`), the grants a person made (`alo-granted`) and
  the pairings they made (`alo-nearby`) — each section reading and writing the
  same file its crate already owns, with a test per section that the surface
  changes nothing the crate would not; **the grants and pairings sections show
  what has been granted to what in one list and revoke the same way**, which
  is `docs/features.md`'s ★ promise and `alo-granted`'s own shape; and a
  setting the machine does not have yet is **absent**, not greyed out, because
  a dialogue full of disabled controls is a promise nobody made.
- **Constraint:** this surface decides nothing. Every value it writes goes
  through the crate that owns it, so that what a person sets and what the
  machine enforces cannot disagree. Nothing here is a second place to grant:
  making a grant is `alo-picking`, a person standing in a folder and choosing
  it, and revoking is the one action `alo-granted` already enforces.

### 7. The status area's clock, battery, network and volume

**Status:** **Done, 2026-09-21.** Report:
[The status area's four](updates/the-status-areas-four.md).
`crates/alo-shell/src/status_items.rs` holds the four as the owning crates said
them — `alo_formats` wrote the clock's text, `alo_power::Reading` is its own
charge and its own charging state, `alo_networks::Reaching` answers *connected*
through `HowFar::reaches_anything`, and `alo_sound::Volume` is its own number —
and `status_items_raster.rs` places them inside `dock_raster`'s status area,
where `desktop_paint` paints them onto the band.

**Nothing is measured in the shell.** The readings arrive on `DesktopFrame`,
which is the arrangement `crate::lock_battery` is already under: a compositor
that opened `/sys` to read a battery would be a compositor measuring.

**Both things the drawing must not undo are held by tests.** A machine with no
battery has **three** cells rather than four with one blank, and the three are
wider for it. And a machine told nothing about metering is drawn as **holding
off**, because that is what `Metered::should_hold_off` counts *nothing said* as
— a status area showing *not metered* there would be adding a claim about
somebody's data allowance.

**What this task does not do, and who does:** putting a *running machine's*
readings into `DesktopFrame` is **task 15**, written 2026-09-21 because nothing
owned it — the shell is handed the four and does not fetch them, and until 15
lands the only caller that fills that field is `examples/desktop_check.rs`, with
fixed readings labelled as fixed.

Was *ready — unblocked 2026-09-20*, all four on `main`. The word
here is *ready* because *unblocked* is not one the supervisor reads, and a
status it cannot parse is a task it selects for ever. **Re-read
2026-09-20**, when a lane sent to write the four found three already there; this
paragraph had said they did not exist, and it was written on 2026-09-15 before
they landed. The fourth was written that day.

| Blocker | | |
|---|---|---|
| the battery | `alo-power` | **clear** — landed 2026-09-17, task 5 of `v0-5-devices-and-media-plan.md`. `TheBattery::on_this_machine` is `Option`, so a machine with no battery reads as **absent** rather than as a battery at zero, which is this task's own word. |
| the volume | `alo-sound` | **clear** — landed 2026-09-17, task 2 of the same plan. `Volume` and `Heard` come through `TheAudioServer`, the one road to the media server. |
| a time written regionally | `alo-formats` | **clear** — landed 2026-09-17, task 6 of `v0-5-access-and-language-plan.md`. `Regionally::time` writes a time as the person's language and region write it, from CLDR. |
| the network's state | the network monitor portal `alo-portals` names | **clear** — landed 2026-09-20 as task 12 of `v0-5-applications-and-what-they-expect-plan.md`, in two halves: `alo_networks::WhatIsReached` reads how far the machine reaches, and `org.freedesktop.portal.NetworkMonitor` answers from it. What a status area reads is `alo_networks::Reaching` — `HowFar::reaches_anything` is the one place *connected* is decided, so this task shows that answer rather than making a second one. |

**Nothing here is on a screen.** Each of the four is a crate that measures or
owns; drawing them is this task, and it is now free to start. **Depends on:** 5.

Two things the drawing must not undo. `alo_power::TheBattery::on_this_machine`
is an `Option`, and a desktop with no battery is that `None` — **absent**, as
this task's acceptance says, never a battery drawn at zero. And
`alo_networks::Metered::should_hold_off` counts *nothing said* as metered: a
status area that showed *not metered* where the machine does not know would be
adding a claim of its own, which the constraint below forbids.

The half of task 5's acceptance that could not be drawn without deciding it
here. `docs/features.md`, v0.5: *Status area: clock, battery, network, volume,
brightness — at the far end of the dock, wherever the dock is.* The status area
exists (`crate::dock_raster`, a segment at the far end of the band) and the
egress indicator grows from it; nothing else is in it.

- **Acceptance:** the status area holds the clock, the battery, the network and
  the volume, each drawn from the crate that measures or owns it and worded in
  the vocabulary, with a test per item that the number or state drawn is that
  crate's and none is made here; each reflows with the dock — a row on a dock
  across the screen, a column on one down it — and none covers the egress
  indicator's lines; and an item the machine does not have (no battery on a
  desktop) is **absent**, not drawn empty.
- **Constraint:** the same as task 5's. The status area shows; it never
  decides, and it adds no number of its own. Brightness waits for the display
  work that owns it.

---

**Tasks 8 to 14 were added on 2026-09-15**, when the rest of v0.5 was planned.
Eight plans decide what a person sees in crates of their own and never draw;
this plan is still the one owner of `crates/alo-shell`, so every surface those
plans need is drawn here, from their decisions. Each task below waits, by its
status, on the named tasks of another plan — a task that runs ahead of its
decision would be the drawing crate deciding, which is what this plan exists to
refuse.

### 8. The lock screen, drawn

**Status:** **Done, 2026-09-18: the code only.** Owner-assigned lane A implemented
`LockSurface`, exclusive nested lock rendering and the approved wallpaper
installation mapping. Publication requires all nine gates on the combined tree;
results accompany the commit. See
`docs/autonomy/updates/the-lock-screen-draws-only-what-lock-policy-allows.md`.
Direct-display, installed-image and certified-machine acceptance remain open.
**Depends on:** 1.

- **Acceptance:** the lock screen draws exactly what `alo-locking` allows — the
  time, the lock image, the battery, that the machine is locked, and the egress
  indicator's line without its destination — and a test holds, from a raster, that
  nothing else is drawn; unlocking reuses task 1's sign-in composition and never a
  second one; the agent overlay's key does nothing while locked, with a test.
- **Constraint:** nothing decided here. No notification preview, no media controls
  that show a title, no *emergency* shortcut that reaches the desktop.

### 9. Several displays, and a background and a dock on each

**Status:** **Done, 2026-09-20: the code only.** `crates/alo-shell/src/screens.rs`,
`screen_background.rs`, `screens_raster.rs` and `tests/screens_source.rs`, with
`alo_shell::desk` as the door a session asks the whole desk through; evidence and
decisions in
`docs/autonomy/updates/several-displays-each-with-its-own-background-and-dock.md`.
**Two displays have never been plugged into this machine**, so every clause below
is held on two outputs a test describes rather than two panels a person can
touch — the layout logic and its pixels, not the hardware. Findings in that
report: `alo-dock` still holds one edge for the whole machine, so each screen's
dock is drawn from that screen's own `Wearing` and every screen's edge is
nevertheless the same one today; the image reader a background shares with the
lock screen words every failure as that screen's, and is said as the desktop's
refusal here rather than carried into a desktop frame; a rotating background
turns on one clock for the whole desk, because that is the only clock
`alo-appearance` decides; and there is still no direct-display submission of a
desk — the DRM path drives one output at a time. **Depends on:** 5.

**Unblocked 2026-09-20**, by work that landed earlier. It read *blocked on
`v0-5-the-session-and-the-displays-plan.md` tasks 3 and 4*, and both are
finished — task 3 gives a display the stable identity an arrangement is kept
under, which is the whole of what this waited for. Nobody moved the line when
they landed, so the task read as untakeable to every machine that surveyed the
plans.

- **Acceptance:** the compositor lays out outputs as `alo-displays` arranges them,
  at the scale it names, restores an arrangement when a known set of displays is
  plugged in, and moves windows off an unplugged display to where that crate says;
  each display draws its own background and its own dock on the edge `alo-dock`
  names for it; night light is applied per display as decided; and a test with two
  nested outputs holds each clause.
- **Constraint:** the arrangement is `alo-displays`' and is never adjusted here.

### 10. Dividing the screen, virtual desktops and gestures, drawn

**Status:** **Done, 2026-09-21.** **For the part it ends at since its split** —
the drawing. The state is task 16 and is **not** done; this line carries the
mark the supervisor reads so the loop does not take the task up for ever, and
the qualifier beside it so no person reads the mark as the whole task.
Report: [The division, drawn](updates/the-division-drawn.md).
The drawing is done and **the state is not**, and the two were split because
they are different work: `crates/alo-shell/src/division_raster.rs` draws the
shares `alo-dividing` decided, the rule where two meet, and the outline a drop
would take — taken from `alo_dividing::Proposal::area` rather than worked out
again, because an outline that disagreed with what `commit` then does would be
the machine lying at the one moment somebody could still change their mind.

**The constraint is half kept, and the half that is not is written down.**
*Which side a chord means* had two answers — `alo_dividing::keyboard::side_for`
and a second mapping in `crate::window_command` — and now has one, held by a
test that walks every `Action` and fails if the shell ever names a different
side from the crate that decides. **`window_tiling` is still the layout
decider**, and today it is the only one, because the `Server` holds no division
at all. The moment it does, that half must go.

**Where the rest lives: task 16**, which owns the `Server`'s division and
desktop state — drawn per display *as decided and restored as remembered*,
desktops, swipes, and the indicator on every desktop. It is **not** blocked on
this task and does not belong to this plan's drawing: it is session lifecycle
state, and task 13 of `v0-01-delivery-plan.md` builds the session that has it.

Was *ready — its blocker cleared on 2026-09-20.* It waited on
`v0-5-hands-on-the-desktop-plan.md` tasks 1, 2, 3 and 5; task 2 was the last of
them and landed that day, and 1, 3 and 5 were done on 2026-09-17 and 2026-09-18.
`alo-dividing` now proposes, commits and **remembers** a division, so there is a
decided split for this task to draw. Cleared by the lane that finished task 2,
in the same change — a blocker that outlives its cause makes takeable work look
untakeable to every machine that reads these plans. **Depends on:** 5.

- **Acceptance:** dragging a window shows the half or quarter `alo-dividing`
  proposes before it is committed and commits it on release; a boundary between
  shares resizes both; divisions and desktops are drawn per display as decided and
  restored as remembered; swipes switch desktops as `alo-desktops` decides; and the
  egress indicator, the approval surface and the overlay are on every desktop, held
  by a raster test on two desktops.
- **Constraint:** v0.01's `window_tiling` yields to the division; there are never
  two tiling decisions in one compositor.

### 11. Notifications, the capture tools and the in-use indicator, drawn

**Status:** **Done, 2026-09-25.** All three surfaces are drawn and reach a real
display. **Two things are named rather than ticked**: one arm of the in-use
mark's colour rule cannot be carried by colour at all in high contrast, and the
two indicators are on the desktop frame and not yet on Settings, a question or
the record. Both are below. **It was ready, and its blockers cleared on 2026-09-19 —
that line outlived them.**
It waited on `v0-5-the-session-and-the-displays-plan.md` task 6 and
`v0-5-capture-and-the-room-plan.md` tasks 1 to 5. All six are done: capture 1
to 5 landed between 2026-09-15 and 2026-09-17, and session task 6 —
`crates/alo-notifying` — on 2026-09-19. Nothing was re-read afterwards, so a
takeable task read as untakeable for two days and task 14 behind it with it.
Re-read 2026-09-21. **Depends on:** 5.

#### What already exists for this task's nouns, read on 2026-09-24 before starting

- **Nothing in the shell draws any of the three.** None of `alo-notifying`,
  `alo-capturing` or `alo-in-use` is a dependency of `crates/alo-shell`, and
  there is no raster for any of them. The gap this task names is the whole of
  it.
- **Every decision is already made, in the crate that owns it.**
  `alo_notifying::deciding::arrives` hands back `Became::Shown(Shown)` or
  `Became::Held(Why)` **having already asked** whether the seat is locked — the
  notification waits behind the lock screen and comes back at the unlock — and
  whether quiet hours hold it. `Shown` is that crate's own words for *what a
  shell draws*. So *never while locked* is a test that this crate draws only
  what `Became::shown()` gives it, and never a second judgement here.
- **`alo_in_use::Line` is shaped like the egress indicator's.** It hands over
  the mark, the position, the colour, the word and the sentence, decided — which
  is what `alo_indicator` hands the egress indicator that
  `egress_status_raster.rs` already draws. The in-use indicator is that file's
  twin rather than a new invention, and `egress_status_place.rs` is where room
  is made for it beside — never as — the egress indicator.
- **Blur is already destructive inside `alo-capturing`.** `Marks::flatten`
  composes the taken picture with the hidden regions before anything is saved,
  so *destructive in what is saved* is not this crate's to implement. What is
  drawn here is the marks a person is making, and what is held is that the
  shell saves nothing.
- **They can reach a real display now.** Task 39 of `v0-01-delivery-plan.md`
  widened the direct seam from one scene to every layer on 2026-09-24. Before
  that, anything drawn here could only ever have been seen in a nested
  compositor.

#### What was built, and the two things that are named rather than ticked

**Notifications.** A card for each one the crate handed over, at the end of the
dock **opposite** the status area — the two indicators own that corner and are
permanent, and a notification that covered *what is leaving this machine* would
be trading a promise for a convenience. *Never while locked, shared or
recorded* is carried by the **type**: the drawing takes `alo_notifying::Shown`,
and only that crate's `arrives` makes one, having already asked about the lock,
the quiet hours, a screen being read, and a machine that cannot tell whether its
screen is being read. There is no path here that could draw a held one.

**The in-use indicator.** One row per line `alo-in-use` wrote, in the status
area, ordered by that crate's `Position` and never by who is using something —
a test holds that an agent picking up the camera does not move it. It takes the
status corner and the egress indicator stacks beyond it, because ADR 0010 makes
this indicator's position one of the three things given besides a colour and
that only holds while its origin does not move. The three marks are compared as
**pixels**: two shapes that happened to rasterise the same would fail.

**The capture tools.** The region as an outline with its middle untouched and
nothing outside it dimmed, and the marks a person is making. **A blur is drawn
as the flat block it will be saved as**, because `capture_flatten` destroys what
is under it for good; drawing it soft would show a person one thing and save
another at the moment they are deciding whether a colleague may see it.

Two indicators sit in that status area and a person reads them as one surface,
so the row moved into `status_row.rs`: its height, the padding round its words,
the side its mark sits on and the way words are cut at the screen's edge are
decided once rather than twice in two files that would drift.

**Named, not ticked.**

- **In high contrast, colour cannot say *the agent* at all.** `Contrast::High`
  collapses every accent to one, on purpose, so the agent's terracotta and an
  application's navy are identical there. ADR 0010 is why that is safe — the
  mark and the word carry it — and the test asserts the equality rather than
  demanding high contrast stop being high contrast.
- **Both indicators are on the desktop frame only.** Settings, a waiting
  question and the record window are submitted by their own paths, which carry
  the egress indicator and not yet the in-use one. A person who opens Settings
  while their camera is on should not lose the line that says so; that is the
  remaining wiring and it is named here rather than left to be found.

- **Acceptance:** notifications are drawn as `alo-notifying` gives them, never while
  locked, shared or recorded; the region selection and the annotation marks of
  `alo-capturing` are drawn with blur destructive in what is saved; the in-use
  indicator of `alo-in-use` is drawn in the status area beside — and never as — the
  egress indicator, with mark, word and position, and stopping a recording or a
  share is one action on it; and a raster test holds that the in-use indicator is
  drawn whenever the camera, the microphone or the screen is in use, including by
  alo OS itself.
- **Constraint:** no dismiss, no hide, no *don't show again* on the in-use
  indicator.

### 12. The accessibility tree, the magnifier and keyboard-only operation

**Status:** **Done, 2026-09-20 — the code.** Two changes, one branch each.
The tree first: `crates/alo-shell`'s `access_roles.rs`, `access_nodes.rs` and
`access_bus.rs` publish every surface `alo-access` names on the accessibility
bus, and `tests/the_tree_a_reader_finds.rs` reads them back **over a real one**
— a session bus of the test's own, at-spi2's own bus launcher and registry on
it, the tree embedded through `org.a11y.atspi.Socket.Embed`, and the reading
done by `alo-adapters`, the agent's own reader, which knows nothing of this
crate. The approval surface reads back as the sentence and its two answers, and
nothing on any surface reads as the default, as focused or as on.

Then the rest of the acceptance. **High contrast is drawn:**
`access_contrast.rs` is the one door between `alo_access::HighContrast` and the
colours this crate paints, every look carries which palette it is drawn in, and
every palette in the crate — the panel's, the sign-in screen's, the
indicator's row, the record window's, the desktop's, the lock screen's and the
agent's own mark — is built through it, with a test per surface holding that
the layout does not move and that no colour outside that palette is painted.
**The magnifier** is `access_magnifier.rs`: the prepared frame magnified around
the pointer, nearest and never smoothed, the view kept inside the screen, at the
magnification the person set — `PreparedScanout::magnified` is the step a
presenter takes. **Keyboard-only operation** is walked by
`tests/every_road_a_keyboard_takes.rs` against `alo-access`' own answers, and
the walk found two surfaces that disagreed with them: **Escape now answers no on
an approval** and **clears what was typed at sign-in**, which is what
`alo_access::leaving` decided and which neither crate's own tests could have
caught. **Focus is visible before any key is pressed** on every surface that has
one, held by a test; on the approval and the recovery screen nothing is drawn as
chosen, which is ADR 0001's rule rather than an omission.

**Two things it does not claim.** **No screen reader has read the tree** —
Orca has never been run against it; the reader in the measurement is the
agent's, asking the same questions on the same bus, and a person hearing it is
not the same as a test reading it. And **two of `alo-access`' Tab stops are not
walked**: the desktop's *ask the agent* and the dock's launcher are surfaces
this crate does not draw at all, so there is no key of this crate's to walk to
them. The walk names them and fails when one is drawn without being walked.
Evidence, decisions and findings in
`docs/autonomy/updates/the-tree-a-screen-reader-reads.md`. **Depends on:** 1, 2,
3, 4, 5.

- **Acceptance:** the shell exposes every surface's role, name and state to AT-SPI
  as `alo-access` decides them, and a test reads the exposed tree over the bus for
  each surface the shell draws; the magnifier and high contrast apply as decided;
  focus is always visible, never trapped, and every action has the keyboard road
  `alo-access` lists, with a test that walks them; and the approval surface is
  exposed as the sentence, then two answers, nothing preselected.
- **Constraint:** the tree is the rented AT-SPI's protocol; no reader of our own.

### 13. The recovery and rollback screen

**Status:** **Done, 2026-09-20: the code only.** `crates/alo-shell/src/recovery_screen.rs`,
`recovery_keys.rs`, `recovery_seat.rs`, `recovery_raster.rs`, `recovery_paint.rs`,
`recovery_reached.rs`, `nested_recovery.rs` and `tests/recovery_source.rs`, 24
tests of its own.

**The decision it waited on was made in the same change.** `alo-access` had
nothing to say about this screen — its own
`tests/every_surface_the_shell_draws_is_read_aloud.rs` reads `alo-shell`'s
exports and failed the workspace on `["RecoveryFrame", "RecoveryScreen"]`, which
is that guard working: a screen a person cannot be told about is a screen they
cannot use. `alo_access::Surface::Recovery` now names the controls — a Window, a
Label for what is running, a Label for what it replaced, a List of the choices
and a Button for each of the two moments — with the words for them and its place
in `reaching`'s focus order, **before** sign-in, because this is the surface that
exists when the workspace does not. That crate names the controls and none of the
sentences, which is its own pattern and not a new one. **No real desktop has ever
failed to start on this machine**: the desktop in the test is made to refuse,
which takes the same road a real failure would and is not one.

**A finding, independent of that one:** *what is running* and *what it
replaced* have **no sentence in any crate**. `Deployments` has no `said`,
`Since` has no words at all, and `alo-keeping-up`'s own rule is that a person is
told an update is ready and **never which build it is** — so drawing them would
be the drawing crate deciding, which this plan refuses. That half of the
acceptance waits on a word in `alo-keeping-up`, exactly as task 5's clock and
battery waited for task 7, and is written down as a finding in
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` for the plan that owns it.
The two lines are named in the accessibility tree and empty until then, so a
reader announces a line with nothing in it rather than a line nobody knows is
there. Everything that crate *does* word is drawn: the
offer, the two moments, and each reason going back cannot be offered.

Evidence, decisions and both findings in
`docs/autonomy/updates/the-recovery-and-rollback-screen.md`. **Depends on:** 1.

### 14. Every new surface, walked

**Status:** ready. Unblocked 2026-09-26: task 11 landed on 2026-09-25 and task
16 on 2026-09-26, so the state a walk needs now exists. A session holds desktops
and divisions, a swipe switches desktops and windows belong to them, which is
what *a walk cannot step through a second desktop that nothing holds* was
waiting for. Two things a walk will meet are named in 16's status and not
ticked: nothing here has plugged a real display, and the agent overlay is drawn
nowhere, so a walk finds two of the three promised surfaces and not three.
**Tasks 8, 9, 12 and 13 are done**
(2026-09-18, 2026-09-20, 2026-09-20 and 2026-09-20), so the lock screen, a second
display, the accessibility tree with the magnifier and high contrast, and the
recovery screen are all there for the walk to use. Narrowed here by the lane that
finished each, because a blocker that outlives its cause makes takeable work look
untakeable.
**Depends on:** 8, 9, 11, 12, 13, 16.

**This asks nothing of the owner**, recorded 2026-09-25 in [what closes this
release, and in what order](updates/what-closes-v0-0-5-and-in-what-order.md),
because it was briefly described as waiting on a display. It waits on tasks 11
and 16 of this plan and unblocks itself when they land. Its constraint asks
only that the report say the certified machine has seen none of it. A blocker
described as hardware when it is really an unfinished task sends the owner
shopping and leaves the task where it was.

- **Acceptance:** one walk through the nested compositor — sign in, dock a second
  display, divide the screen, take a screenshot with a blur, receive a notification,
  lock, unlock by keyboard with the screen reader on — produces a raster at each step
  and the exact sequence of spoken and shown text, recorded in the report and held by
  one test; every sentence drawn is the vocabulary's, and none is written in this
  crate.
- **What a raster here is, written into the acceptance so a tick cannot be
  misread:** each step's raster is **what this compositor drew, not what the
  parent displayed**. The frame is painted twice by the same painter — same
  `GlesRenderer`, same `crate::scene_drawing::paint`, same roots, popups, cursor
  and layers — once into the parent's window, which is the submission, and once
  into an offscreen buffer that can be read, because reading the window's own
  buffer back loses the EGL context on this backend (`docs/quirks.md`). Seven
  green steps are therefore evidence that each surface laid itself out and put
  pixels down. They are **not** evidence that a parent, a compositor above it, or
  any panel showed them.
- **Constraint:** measured under a nested compositor; the certified machine has seen
  none of it and the report says so. The parent may be `weston --backend=headless`
  with no display and no GPU (see *A nested parent needs no WSLg and no display* in
  [COMPOSITOR.md](COMPOSITOR.md)) — which is what makes this task runnable on a lane
  without WSLg, and which composites to nothing, so no screenshot of a real display
  comes out of it.


### 15. A running machine's own clock, battery, network and volume

**Status:** **Done, 2026-09-26.** The four are taken from this machine, in a
package of its own — `crates/alo-desktop`, which the binary moved to so that no
call into `/sys`, the media server or the network manager is made from
`crates/alo-shell`. **What cannot be ticked here is that the numbers match the
hardware**, which needs a certified machine; what was measured on the gate is
below. **Depends on:** 7.

#### What was measured, and what is owed

**Measured on the gate machine, by running the binary.** It read that machine's
**battery**, and reported — with the reason for each — that the machine has no
network manager (`org.freedesktop.NetworkManager was not provided by any
.service files`) and nothing that handles sound (`pw-dump failed: can't
connect`). Three readings taken and two honest absences, none of them invented.

**The volume asked for is the chosen output's**, not the loudest device plugged
in: a machine with headphones and speakers has two volumes and a person hears
one of them, and `alo-sound` already answers which.

**The clock advances**, held by the test this acceptance names — move the time,
find the text moved with it. The readings are taken again once a second rather
than once a frame: a frame is drawn sixty times a second and a battery does not
move sixty times a second. A refusal does not clear what was there, because the
reading it would replace is a second old and truth arriving late beats an
absence arriving early.

**Owed: that the numbers match the hardware.** Nothing on a gate can show that
the battery drawn is the charge in the machine — that is a certified machine's
to show, and it is named here rather than assumed, as this plan's other tasks
name theirs.

Written 2026-09-21 by task 7, which found it unowned. Task 7 drew the status
area's four and was right not to measure them: the readings arrive on
`alo_shell::DesktopFrame`, the arrangement `crate::lock_battery` is already
under, because a compositor that opened `/sys` would be a compositor measuring.
**What nothing does is fill that field on a running machine.**

Nothing outside `crates/alo-shell` constructs a `DesktopFrame` at all, and the
one task that puts a binary in this crate — task 13 of
`v0-01-delivery-plan.md`, *A sign-in surface, and what starts it* — stops at the
sign-in screen and the session opening. So the four readings sit between two
tasks and belong to neither, and `examples/desktop_check.rs` hands over **fixed**
ones, labelled as fixed in its source because that probe asks whether a frame
draws rather than what this machine's battery is.

**Why it is worth its own line rather than a note.** A status area on a real
laptop showing a battery that never moves does not look unfinished, it looks
broken — and it is the one surface a person checks to find out whether their
machine is telling them the truth.

- **Acceptance:** the four readings a status area shows on a running machine are
  that machine's, taken from the entry points the owning crates already have —
  `alo_power::TheBattery::on_this_machine`, the audio server `alo-sound` reaches,
  the network monitor `alo-networks` reads, and `alo_formats::Regionally` for the
  time — and handed to `DesktopFrame` rather than read inside the shell, which
  stays the crate that shows; the absent cases are real rather than defaults, so
  a machine with no battery hands over `None` and a crate that has not been told
  hands over *nothing said*; the clock advances as the machine's clock does, held
  by a test that moves the time and finds the drawn text moved with it; and the
  reading a person sees is the one the crate gave, held per item the way task 7
  holds the drawing.
- **And one reading has no absent case to give**, found by task 39 of
  `v0-01-delivery-plan.md` while standing the desktop up. `StatusItems` says
  [`None`] for a machine with no battery and *nothing said* for a network
  nobody asked, and has **no such value for the volume** — so a desktop that has
  asked nothing still shows one, and `alo-desktop` shows silence, which is a
  claim rather than an absence. Giving the volume an absent case is part of this
  task's *the absent cases are real rather than defaults*, and until it exists
  that acceptance cannot be met for the fourth reading.
- **Constraint:** the shell still measures nothing — this task builds whatever
  stands the desktop up and reads, and adds no call into `/sys`, the media server
  or the network manager from `crates/alo-shell`. **What it cannot tick from a
  machine in a nested compositor is that the numbers match what the hardware is
  really doing**; that half needs a certified machine and is named beside the
  tick rather than assumed, exactly as this plan's other tasks name theirs.

### 16. The division and the desktops a session holds, and the one layout decider

**Status:** **Done, 2026-09-26.** A session holds a division per display and
the desktops on it, kept through windows opening and closing and a display
arriving and leaving; a chord divides through `alo-dividing` and the half is
gone, held by a test that reads the crate; a swipe switches desktops through
`alo-desktops`, and windows belong to desktops. **Two things are named and not
ticked:** a real display plugged in and unplugged has not been done, because
this lane has no machine with a display — the constraint below says so — and the
third of the three promised surfaces, the agent overlay, is on every desktop in
`alo-desktops` but is drawn nowhere in this compositor, so only the egress
indicator and the approval surface are held on two desktops in pixels. Report:
[`updates/the-division-and-the-desktops-a-session-holds.md`](updates/the-division-and-the-desktops-a-session-holds.md).
**Depends on:** task 13 of
[`v0-01-delivery-plan.md`](v0-01-delivery-plan.md) — *A sign-in surface, and what
starts it*.

Written 2026-09-21 by task 10, which drew the division and found that the state
under it does not exist. `crates/alo-shell`'s `Server` holds an overlay, a
press, a presentation, its surfaces, its socket and a switch order, and **no
division and no desktop at all**. So *divisions drawn per display as decided and
restored as remembered*, *swipes switch desktops*, and *the indicator on every
desktop* are not drawing — they are state with a lifecycle, and the lifecycle is
windows opening and closing and displays being plugged in.

**Why it depends on the session rather than on task 10.** That is the same
lifecycle a running session needs, and building it twice — once for a nested
compositor and once for the real one — would be two answers to *what is on this
display now*. So it is built once, on the session task 13 stands up.

**The removal this task owes.** `crate::window_tiling` computes a half of an
output. Today it is the only layout decider and that is sound; the moment a
`Division` is `Server` state, a window's place would be decided twice — once by
a tree of shares and once by a half — and the shell plan's constraint forbids
exactly that. This task takes the half out as it puts the division in.

#### What already exists for this task's nouns, read on 2026-09-26 before starting

- **Both deciding crates already hold the lifecycle.**
  `alo_desktops::Desktops` has `plug_in`, `unplug` and `on(display)`;
  `alo_dividing::Divisions` has `remember`, `on`, `forget` and `restored` — the
  last taking the window numbers a division comes back under. So the `Server`'s
  work is to **hold** an instance of each and route what happens to them, which
  is what *this holds and shows their answers* already asks for. Neither a
  state machine nor a restore needs writing here.
- **The `Server` holds neither today.** It has an overlay, a press, a
  presentation, its surfaces, its socket and a switch order, exactly as task 10
  found.
- **The removal depends on the state, so it comes second.**
  `window_tiling`'s half is reached from `window_command.rs` (a chord),
  `window_mode.rs` (`Mode::Tiled` and the geometry) and `lib.rs`'s exports.
  `set_window_tiled(side)` can only become *the share the division gives for
  that side* once a division is `Server` state, so the state lands first and
  the half goes out after it — not the other way round.
- **Which side a chord means is already `alo-dividing`'s**, since task 10 on
  2026-09-22. What is left in the shell is the mechanism that turns a side into
  half an output, and that is what goes.

- **Acceptance:** the `Server` holds a division per display and the desktops a
  person has, kept through windows opening and closing and a display being
  plugged in and unplugged; a division is restored from
  `alo_dividing::Divisions` as the person left it, per display, with the window
  numbers it comes back under being the new ones; swipes switch desktops as
  `alo-desktops` decides and the shell shows that answer rather than making a
  second one; the egress indicator, the approval surface and the window-control
  overlay are on **every** desktop, held by a raster test on two; and
  `crate::window_tiling`'s half is **gone**, with a test that there is exactly
  **one** layout decider in this compositor — read from the crates rather than
  from a list kept beside them, so a second one added anywhere is a second one
  this check sees.
- **Constraint:** nothing here decides a layout, a side or a desktop —
  `alo-dividing` and `alo-desktops` decide and this holds and shows their
  answers. **What it cannot tick from a nested compositor** is that a real
  display plugged in and unplugged keeps its division; that needs a machine with
  a display to plug, and is named beside the tick rather than assumed.

### 17. Two real clients divided, through the probe's own pixels

**Status:** ready. **Depends on:** 16 (landed 2026-09-26), and on *the nested
fixtures run in the gate* (#143), because a probe nothing runs is where this
coverage went missing in the first place.

Written 2026-09-26 by task 16's owner, about a hole task 16 made. It is here as
a task rather than in a commit message because **a finding that lives in a commit
message is invisible work**: nobody reads one looking for something to do.

**What is uncovered.** `crates/alo-shell/examples/support/offscreen_check.rs`
walked stages 23 to 28 with a real client through real GLES pixels: a window put
on half an output, its configure acknowledged, its buffer attached at the tiled
size, and the committed origin read back off the frame. Task 16 removed
`crate::window_tiling` and those six stages went with it. **Nothing replaced
them.** What a chord does now is held by `crate::window_dividing`'s unit tests,
by `tests/shortcut_dispatch/layout.rs` with one client, and by
`tests/one_layout_decider.rs` reading the source — and by no pixels at all. The
probe's own stage count was left at thirty for a day afterwards, which is how
somebody noticed.

**Why it was removed rather than rewritten.** A division divides *between* two
windows: `alo_dividing::Division::divide_with_next` takes the focused window and
the next one, and with one window open it refuses by name. The probe drives
**one** client through one scripted dance of stage numbers, and the checker side
matches on those numbers. There is no second window to divide with, so the six
stages had no honest translation — a single client asking to be divided is the
one case the new design deliberately refuses.

**What this task is.** Give the probe a second client, and divide between them:
the focused window on a side, both windows configured, both buffers attached at
the sizes the division gave, and both committed origins read back off the frame
and compared against `Division::shares`. The two sides a half could never lay
out — top and bottom — are shares of a tree exactly as left and right are, and
are worth a stage each for that reason.

- **Acceptance:** the offscreen probe divides a display between two real clients
  and reads both windows' committed origins and sizes back off the submitted
  frame, compared against what `alo_dividing::Division::shares` says rather than
  against numbers written in the probe; a chord with one client still refuses and
  that refusal is a stage too; and the probe's stage count matches the stages it
  walks, checked by walking them rather than by a number somebody maintains.
- **Constraint:** the probe drives clients and reads pixels; it decides no
  layout. Every rectangle compared comes from the division. **What it cannot
  tick** is a physical display: the parent is `weston --backend=headless` or
  WSLg, both of which composite to nothing, so this is what was drawn and never
  what a panel showed.
