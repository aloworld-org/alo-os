# v0.5 — the session and the displays: lock, sleep, come back, and more than one screen

**Workstream:** five `ROADMAP.md` v0.5 lines that are one subject — *Lock screen,
suspend and resume*; *Multi-monitor, scaling, hotplug*; *Session management: log
out, switch user, lock, and reopen what was open* (under *Software*); *Night light
and display colour* (under *Devices*); and the per-display half of *Making it
yours* (a background and a dock per display). They belong together because they
are all the same question: **what a person's session is while they are not
looking at it, and on which screens it is when they are.**
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan.

**Crates this plan owns, all new:** `crates/alo-locking` (what a locked session
is, what the lock screen may show, what survives it), `crates/alo-sleeping`
(suspend, resume, the lid, and what may keep a machine awake), and
`crates/alo-displays` (a person's arrangement of screens: which, where, at what
scale, remembered per set of screens, and night light), and `crates/alo-notifying`
(notifications and do-not-disturb, task 6). **It reads and never
edits** `alo-accounts`, `alo-greeting`, `alo-sessiond` (a lock is not a sign-out,
and unlocking is authenticated the way signing in is), `alo-appearance` (the lock
image and a per-display background are its values), `alo-dock` (per-display
placement is its decision), `alo-portals` (the *inhibit* portal is an application
asking to keep the machine awake), `alo-approving` and `alo-overlay`, and
`alo-saying`/`alo-strings`. **Nothing in `crates/alo-shell`**: every surface this
plan needs is drawn by the shell plan's later tasks, from the decisions made here.

**What this plan may not do:** tick anything *on the machine* — a lid that has
never closed on certified hardware is `- [x] The code.` and nothing more; write a
display driver, a power-management daemon or a session manager of our own (ADR
0011: the base's `logind`, the kernel's DRM and the compositor library are
rented, configured and never patched); or decide anything about signing in that
ADR 0024 already decided. Before writing the next task, `git pull` and read the
plan as published.

## Tasks

### 1. What a locked session is, and what the lock screen may show

**Status:** ready. **Depends on:** nothing.

A lock screen is the one surface a stranger at the desk is guaranteed to see. On
most systems it leaks: the names of recent documents in notifications, the
agent's last question, an approval waiting to be tapped.

- **Acceptance:** `alo-locking` holds what locking decides, as types with a test
  per clause: a locked session keeps running — applications, turns already
  approved, downloads — and **nothing that would reach a person is shown on the
  lock screen except the time, the lock image, the battery, and that the machine
  is locked**; a notification arriving while locked is held and never drawn there,
  with no preview variant a setting could turn on; **the agent overlay cannot be
  summoned while locked**, and a proposal waiting for approval cannot be answered
  from the lock screen, because approval is the signed-in person's act (ADR 0001);
  the egress indicator still fires while locked, because *nothing leaves silently*
  does not pause for a closed screen, and it is drawn without naming what left;
  unlocking is authenticated by the same composition signing in uses
  (`alo-greeting`) and never by a second, weaker road; and a locked session is not
  a signed-out one — `alo-locking` never ends a session and a test holds that it
  cannot.
- **Constraint:** nothing here draws. The lock image is `alo-appearance`'s value,
  read. No *show notification contents on the lock screen* setting exists; a
  person who wants to read their messages unlocks.

### 2. Suspend, resume, the lid, and what may keep a machine awake

**Status:** ready. **Depends on:** 1.

*Power management, battery, sleep on lid close*, and the inhibit portal's
*no sleep mid-presentation*. The mechanism is `logind`'s; what is ours is the
order of things and who may hold the machine awake.

- **Acceptance:** `alo-sleeping` decides, before any call reaches `logind`, that
  **a session is locked before the machine sleeps** — a resume never lands on an
  unlocked desktop, and a test walks suspend then resume and finds task 1's lock
  in place first; closing the lid sleeps the machine unless a display is attached
  and the person chose otherwise, a choice kept by this crate at a path it is
  handed (ADR 0038); what may keep the machine awake is a closed list — a person's
  own setting, an application holding the *inhibit* portal under a grant, a turn
  that is running — and each is **named** when it blocks sleep, so *why won't this
  laptop sleep* has an answer; an agent cannot inhibit sleep by asking, only a turn
  already approved holds it for its own length; and what happens to a turn that
  was running when the machine slept is decided and recorded — it is resumed or
  refused with a sentence, never silently lost.
- **Constraint:** no daemon of our own. Inhibitors are `logind`'s mechanism,
  configured. Nothing here decides battery thresholds or power profiles; those are
  the devices plan's.

### 3. A person's screens: which, where, how large, and remembered

**Status:** ready. **Depends on:** nothing.

*Multi-monitor, display scaling, hotplug.* The failure everybody knows is the
laptop that forgets, every morning, that the external screen is on the left.

- **Acceptance:** `alo-displays` holds an arrangement — each display by a stable
  identity read from what the display reports (make, model, serial where it has
  one), its position, its scale, and which one is primary — and **remembers an
  arrangement per set of displays**, so docking at the office restores the office
  layout and docking at home restores home's, held by a test that plugs three
  sets in turn; scale is fractional where the compositor supports it, and a
  display the machine has never seen gets a scale derived from its reported size
  and resolution rather than 100 %; **unplugging a display moves its windows to one
  that remains**, and plugging it back restores them where the arrangement had
  them — the moving is the shell's, the decision of where is this crate's; a
  per-display background and a per-display dock edge are read from
  `alo-appearance` and `alo-dock` and applied to the display they name; and the
  arrangement is kept in the person's folder by this crate at a path it is handed
  (ADR 0038).
- **Constraint:** the compositor library and the kernel's DRM are rented. No
  modesetting code is written here. A display that reports no stable identity is
  remembered by its connector, and the report says that is weaker.

### 4. Night light and display colour

**Status:** ready. **Depends on:** 3.

- **Acceptance:** `alo-displays` decides night light — on a schedule a person
  sets, or from sunset to sunrise computed **on the machine** from a location the
  person typed or a timezone, **never from a location service or a network
  lookup**, held by a test that the calculation opens no socket; its strength is a
  colour temperature within a closed range, applied per display; and **terracotta
  still means the agent** under night light — a test holds that the warmed palette
  keeps the agent's colour distinguishable from every accent, with its mark and
  word beside it as always (ADR 0010).
- **Constraint:** no location service. *Sunset for your location* is from what a
  person entered, and absent that the setting offers a schedule instead.

### 5. Log out, switch user, lock — and reopen what was open

**Status:** ready. **Depends on:** 1.

*Session management.* Reopening what was open is a promise about applications
and windows, not about their contents, and it has to be made carefully in a
system whose agent could otherwise learn what a person was doing yesterday.

- **Acceptance:** logging out ends applications in an order that lets each save,
  names any that refused to close, and never kills one silently; *switch user*
  locks this session (task 1) and hands the screen to the sign-in; **what was open
  is a list of applications and which display and split each was on — never a
  document's contents, a window's title or a URL**, kept in the person's folder,
  and restored at the next sign-in only if the person chose it; **the agent never
  reads this list** — a test reads the shipped source of `alo-agentd`'s crates and
  refuses a reader of it, because *what were you doing yesterday* is not context
  an agent is offered (ADR 0001's context is offered at invocation, never
  harvested).
- **Constraint:** no application's own session files are read or written; an
  application that restores its own documents does so itself, under its own
  grants.

### 6. Notifications, and do-not-disturb

**Status:** ready — `v0-5-capture-and-the-room-plan.md` task 1, whose `alo-in-use`
says when the screen is shared or recorded, was published on 2026-09-16. **Depends on:** 1.

*Notifications, with do-not-disturb* (`docs/features.md` v0.5). A notification is the
one thing on a screen that arrives uninvited, and the thing most likely to put a private
sentence in front of somebody else.

- **Acceptance:** a new `crates/alo-notifying`, owned by this plan, holds a notification
  as who sent it, a title, a body and its actions, arriving through the notification portal
  under the sender's grant (`alo-portals`); **do-not-disturb holds every notification and
  shows none**, turned on by a person or by a schedule, and **automatically while the
  screen is shared or recorded** (the capture plan's in-use state, read) — held by a test;
  while locked, task 1's rule applies and nothing is shown; the notifications a person
  missed are kept in a list on the machine until they dismiss them, never synced; an
  agent's own notifications are marked as the agent's with its mark and word (ADR 0010);
  and **an application cannot notify with an action that answers an approval**, because
  approval is only ever the approval surface (ADR 0001), held by a test.
- **Constraint:** nothing draws. No notification is ever read by an agent as context.

### 7. Every sentence, and the walk from lock to resume to a new desk

**Status:** ready. **Depends on:** 1, 2, 3, 4, 5, 6.

- **Acceptance:** every sentence these four crates can say is in the vocabulary
  with a translator's note, and one walk — lock, suspend with the lid, resume,
  unlock, dock to a second display, undock — produces the exact sequence a person
  meets, recorded in the report as a table and held by one test that fails if a
  sentence changes without the table; no sentence names `logind`, DRM, EDID, a
  connector name or any other part of the machinery (`docs/features.md`: *a
  person never learns the name of anything we rented*).
- **Constraint:** nothing here re-decides what the sentences describe.
