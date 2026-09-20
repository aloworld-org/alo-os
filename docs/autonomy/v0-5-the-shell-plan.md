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

**Status:** blocked — on crates that do not exist yet: one that measures the
battery and the network's state and says each in the vocabulary, one that
owns the volume, and a regional way of writing a time (the finding task 4 made
for dates). Since 2026-09-15 each has a plan: the battery is `alo-power` and the
volume `alo-sound` (`v0-5-devices-and-media-plan.md`), a time written regionally
is `alo-formats` (`v0-5-access-and-language-plan.md`), and the network's state
is read from the network monitor portal `alo-portals` names. It unblocks when
those have landed. **Depends on:** 5.

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

**Status:** ready — **its blocker cleared on 2026-09-20.** It waited on
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

**Status:** blocked — on `v0-5-the-session-and-the-displays-plan.md` task 6 and
`v0-5-capture-and-the-room-plan.md` tasks 1 to 5. **Depends on:** 5.

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

**Status:** ready. **Depends on:** 1, 2, 3, 4, 5.

**Unblocked 2026-09-20.** It read *blocked on
`v0-5-access-and-language-plan.md` tasks 1, 2 and 3*; that plan is closed, all
six tasks done — what each accessibility setting changes, the screen reader and
the tree it reads, and keyboard-only operation of everything. Every one of the
three is finished and the line was never moved.

- **Acceptance:** the shell exposes every surface's role, name and state to AT-SPI
  as `alo-access` decides them, and a test reads the exposed tree over the bus for
  each surface the shell draws; the magnifier and high contrast apply as decided;
  focus is always visible, never trapped, and every action has the keyboard road
  `alo-access` lists, with a test that walks them; and the approval surface is
  exposed as the sentence, then two answers, nothing preselected.
- **Constraint:** the tree is the rented AT-SPI's protocol; no reader of our own.

### 13. The recovery and rollback screen

**Status:** **built, and blocked on one decision that is not there.** The whole
screen is written and gates clean on its own — `crates/alo-shell/src/recovery_screen.rs`,
`recovery_keys.rs`, `recovery_seat.rs`, `recovery_raster.rs`, `recovery_paint.rs`,
`recovery_reached.rs`, `nested_recovery.rs` and `tests/recovery_source.rs`, 24
tests green, `fmt=0 clippy=0 doc=0` — and it **cannot be published**, because
`alo-access` has nothing to say about it: its own
`tests/every_surface_the_shell_draws_is_read_aloud.rs` reads `alo-shell`'s
exports and fails the workspace on `["RecoveryFrame", "RecoveryScreen"]`, since
no `alo_access::Surface` names them. That is the right failure — a screen a
person cannot be told about is a screen they cannot use — and the decision it
asks for is **the role, the name and the state of every control on this
screen**, which is an accessibility decision. This plan reads `alo-access` and
does not edit it, so the decision is not made here: it needs
`Surface::Recovery` with its controls, the words for them, and its place in
`reaching`'s focus order. Once that lands the branch merges unchanged.
The work is on `task/dev-pc-lane-a/the-recovery-and-rollback-screen`, in a draft
pull request that is **not** to be merged until that decision exists.

**A second finding, independent of the first:** *what is running* and *what it
replaced* have **no sentence in any crate**. `Deployments` has no `said`,
`Since` has no words at all, and `alo-keeping-up`'s own rule is that a person is
told an update is ready and **never which build it is** — so drawing them would
be the drawing crate deciding, which this plan refuses. That half of the
acceptance waits on a word in `alo-keeping-up`, exactly as task 5's clock and
battery waited for task 7. Everything that crate *does* word is drawn: the
offer, the two moments, and each reason going back cannot be offered.

Evidence, decisions and both findings in
`docs/autonomy/updates/the-recovery-and-rollback-screen.md`. **Depends on:** 1.

### 14. Every new surface, walked

**Status:** blocked — on tasks 10, 11, 12 and 13. **Tasks 8 and 9 are done**
(2026-09-18 and 2026-09-20), so the lock screen and a second display are there
for the walk to use; task 13 is written but cannot be published until
`alo-access` can say what its controls are. Narrowed here by the lane that
finished each, because a blocker that outlives its cause makes takeable work
look untakeable.
**Depends on:** 8, 9, 10, 11, 12, 13.

- **Acceptance:** one walk through the nested compositor — sign in, dock a second
  display, divide the screen, take a screenshot with a blur, receive a notification,
  lock, unlock by keyboard with the screen reader on — produces a raster at each step
  and the exact sequence of spoken and shown text, recorded in the report and held by
  one test; every sentence drawn is the vocabulary's, and none is written in this
  crate.
- **Constraint:** measured under a nested compositor; the certified machine has seen
  none of it and the report says so.

