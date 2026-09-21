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
scale, remembered per set of screens, and night light), `crates/alo-notifying`
(notifications and do-not-disturb, task 6), and `crates/alo-leaving` (logging
out, switching user, and what was open — task 5, which had no crate named for it
until the work was done; it is this plan's). **It reads and never
edits** `alo-accounts`, `alo-greeting`, `alo-sessiond` (a lock is not a sign-out,
and unlocking is authenticated the way signing in is), `alo-appearance` (the lock
image and a per-display background are its values), `alo-dock` (per-display
placement is its decision), `alo-portals` (the *inhibit* portal is an application
asking to keep the machine awake), `alo-approving` and `alo-overlay`, and
`alo-saying`/`alo-strings`. **Nothing in `crates/alo-shell`**: every surface this
plan needs is drawn by the shell plan's later tasks, from the decisions made here.

**Where this plan crosses into lane A's crates:** task 2 adds
`Turning::slept_through` to `alo-turn`, and the slept-through entry to `alo-record`
and its contract `docs/contracts/record-file.md`. Both are additive, and both were
published after `v0-5-the-local-network-plan.md`, the plan working in those crates,
had finished. Task 2 also adds that entry's sentence to `alo-recounting`, which no
plan owns. This plan claims none of the three; a later task that needs them again
checks who is working in them first.

**What this plan may not do:** tick anything *on the machine* — a lid that has
never closed on certified hardware is `- [x] The code.` and nothing more; write a
display driver, a power-management daemon or a session manager of our own (ADR
0011: the base's `logind`, the kernel's DRM and the compositor library are
rented, configured and never patched); or decide anything about signing in that
ADR 0024 already decided. Before writing the next task, `git pull` and read the
plan as published.

## Tasks

### 1. What a locked session is, and what the lock screen may show

**Status:** **Done, 2026-09-16.** `crates/alo-locking`: a `Seat` is a session
open or locked and holds the session in both, with no road to ending one
(`tests/nothing_here_ends_a_session.rs` reads the source and manifest).
`LockScreen` has room for the time, the lock image (`Appearance::lock_on`, read),
the battery and an egress `Lamp` that counts and names nothing, and no generic
parameter a notification could fill; a notification on a locked seat is
`Arrived::Held` and handed back only at an unlock. The agent's key and all three
approval calls answer `NotWhileLocked` without reaching the compositor or the
turn. `Seat::unlocks` is `alo_greeting::Greeting::signs_in` for the locked
session's own number, knocking at the locked session rather than the opener
(`tests/unlocking_is_signing_in.rs`). Report:
`docs/autonomy/updates/what-a-locked-session-is-and-what-the-lock-screen-may-show.md`.
Not on hardware — nothing here touches the machine; the shell draws it later.
**Depends on:** nothing.

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

**Status:** **Done, 2026-09-17.** `crates/alo-sleeping`: `asked` decides whether
the machine sleeps before anything reaches `logind`, and `Going::carried_out`
locks `alo_locking::Seat` and only then calls `Logind::sleep`, which takes a
`LockedFirst` that only a locked seat can produce. A sleep started somewhere
else waits on `UntilLocked`, a delay inhibitor, and a sleep that fails leaves the
seat locked. `Lid` sleeps unless another display is attached and the person
chose otherwise, kept in `sleeping.toml` through `alo-kept` (ADR 0038). `Holding`
is the closed list of `Keeper`s, each named: the person's setting, an
application's `alo_portals::Allowed` for the inhibit portal (the grants are asked
again at every decision), and a `Turning` for its own length. Each hold is an
`idle` block inhibitor from `logind`. A keeper holds off only idle sleep and never
a sleep the person asked for. No crate that answers an agent depends on this one
(`tests/an_agent_cannot_keep_this_machine_awake.rs`). At a wake, `Woke::a_turn`
carries a turn on or stops it with a sentence, and writes the additive
`slept-through` record entry through `alo_turn::Turning::slept_through`. Report:
`docs/autonomy/updates/suspend-resume-the-lid-and-what-may-keep-a-machine-awake.md`.
The real `logind` hold is measured in WSL. No sleep and no lid have been measured
on certified hardware, and none is claimed. **Depends on:** 1.

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

**Status:** **Done, 2026-09-18.** `crates/alo-displays`: an `Identity` is what a
screen says about itself — make, model, serial where it has one — and, for a
screen that says nothing **and for two screens that say the same thing**, the
socket it is plugged into, which `whoever_is_attached` decides over the whole set
because whether a description is unique is not a fact about one screen. An
`Arrangement` is every screen of one set with its `Placed` — position, `Scale`,
and exactly one main screen — remembered under the `Screens` set it holds, one
per set, in `displays.toml` through `alo-kept` (ADR 0038).
`tests/three_sets_of_screens_remembered_apart.rs` plugs three sets in turn
through that file and finds each restored on its own. `Scale` is a whole per
cent from 100 to 300, fractional where `Support::Fractional` says the compositor
can draw it and rounded up to the nearest whole multiple with a sentence where it
cannot; a screen nobody has sized is worked out from its own pixels and glass
against 96 per inch, rounded to the nearest quarter — never 100% unless the
screen reports no size at all. `Attached::unplugged` answers where the windows of
a screen that went belong (the main screen of what remains) and `plugged_in`
answers that they go back, carrying a chain of two hops
(`tests/a_screen_that_goes_and_comes_back.rs`); no window identifier appears
anywhere in the crate. `Wearing` reads `alo_appearance::Appearance::background_on`
under the name the shell knows each screen by, and `alo_dock::Dock::edge` — the
background is per screen and the dock's edge is not, because `alo_dock::Dock`
holds one edge for the machine and this plan never edits it, so
`docs/features.md`'s *Per display, so the dock can sit along the bottom of the
laptop and down the side of the external screen* is **not met** and waits on
`alo-dock`; `Wearing::of` is the one function here that changes when it decides
otherwise. A remembered arrangement that would draw two screens over each other
is set aside for one worked out, and the person is told. Report:
`docs/autonomy/updates/a-persons-screens-which-where-how-large-and-remembered.md`.
It reaches `main` through a pull request from
`task/dev-pc/a-persons-screens-which-where-how-large-and-remembered` and not
through a push to a protected branch;
`docs/autonomy/updates/publishing-a-persons-screens-through-a-task-branch.md`
says why and what it leaves owed to `tools/kernel-loop`.
Not on hardware — nothing here opens a device or sets a mode; the shell draws it
later. **Depends on:** nothing.

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

**Status:** **Done, 2026-09-18.** `crates/alo-displays`: a `NightLight` is
*when* and *how warm*, kept separately because a person changes them
separately. `Nightly` has three arms and deliberately no fourth — never, a
`Between` the person set, or `FromSunsetAt(Whereabouts)` — so *sunset for your
location* cannot exist on a machine that was never told a location, and the
setting offers a schedule instead. `sun.rs` is the published sunrise equation,
worked on this machine from two numbers somebody typed;
`tests/the_sun_is_worked_out_on_this_machine.rs` reads the crate's own source
and manifest and refuses seventeen roads off the machine, and a fourth test in
it checks the arithmetic answers (London on the longest day of 2026: sets 21:21,
rises 04:43, which is what an almanac prints). A timezone is used for one thing
only — putting a sunset worked out for the whole earth onto the person's own
clock (`Moment`, handed a `SystemTime` and an offset, never reading either).
Inside a polar circle `Sun::NeverSets` and `Sun::NeverRises` are each said, as
`Note::TheSunDoesNotSet` and `Note::TheSunDoesNotRise`. `Warmth` is a closed
range, 2000 K to `Warmth::NEUTRAL` at 6500 K where it changes **nothing**
exactly, and `Warming` is the curve normalised at that point. It is applied per
screen through `Wearing`, beside that screen's background, rather than as one
tinted sheet over a desk; there is no *except this one*, and the report says
why. `tests/terracotta_still_means_the_agent_under_night_light.rs` walks every
warmth at 50 K and holds the agent's colour ≥ 5.0 ΔE\*ab from all ten accent
values — measured minimum **9.48, rose on a dark ground at 2000 K** — and holds
that night light never makes the colour sufficient on its own, because ADR 0010
measured that it never was. `displays.toml` gains one key, `night-light`.
Report: `docs/autonomy/updates/night-light-and-display-colour.md`.
It reaches `main` through a pull request from
`task/dev-pc/night-light-and-display-colour`, which is based on task 3's commit
rather than on `origin/main` because this task depends on task 3 — so task 3's
pull request lands first, and this one is opened on top of it;
`docs/autonomy/updates/publishing-night-light-through-a-task-branch.md` says
why, and records that the same refusal has now stopped two finished tasks in a
row, which makes adapting `tools/kernel-loop`'s publisher the thing blocking
this plan rather than a queue item anybody can take at leisure.
**Owed, and not this task's to pay:** `docs/contracts/person-settings.md` still
describes four kept files and names neither `sleeping.toml` (task 2) nor
`displays.toml` (task 3), so this change adds a key to a file the contract does
not yet describe; the report says what that section and its
`tests/the_contract_describes_this_file.rs` would have to hold. Not on hardware
— nothing here opens a device or sets a ramp; the shell applies it later (the
shell plan's task 9). **Depends on:** 3.

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

**Status:** **Done, 2026-09-18.** `crates/alo-leaving`, new and named here
because the plan's header named no crate for this task. `logging_out::asked`
walks a session's applications through a `TheApplications` the compositor
implements — one at a time, **last opened first**, because a stack unwinds and
the application a person was just in is the one they expect to be asked about
first — and asks every one of them even after one has refused, so what comes back
is one list rather than a queue of dialogues. Whatever stayed is a
`WouldNotClose` carrying its own name and what had already closed, each said as
its own sentence (one per application, so no language has to form a plural alo OS
chose for it). **Nothing here kills anything:** `MayEnd` has no public
constructor and exactly two roads — every application closed, or
`WouldNotClose::even_so`, which consumes the named list, so the session can only
end over an application that said no once the person has been shown which
(`tests/nothing_here_reaches_into_an_application.rs` reads the crate's shipped
code for a process, a signal or a kill, and holds its dependency list closed).
`switching::asked` locks `alo_locking::Seat` and **only then** makes the
`alo_greeting::Standing` a greeter draws from; nothing closes, nothing ends, and
nothing is written down, because a switch is not a sign-out. What was open is an
`Open` with three fields — the application's **identifier**, the screen's name,
and a `Split` (the whole screen, or one of `alo_dividing::Place`'s halves,
quarters and parts) — and no fourth: `tests/what_was_open_is_applications_and_places.rs`
reads this crate's shipped source for a title, a document, an address, a subject
and a window identifier, and holds that a file carrying any of them is refused
whole rather than read past. It is kept in `leaving.toml` through `alo-kept`
(ADR 0038), written **once, at a log-out, and only for a person who asked for
it** — a machine nobody configured is left with no file at all, and turning the
setting off takes the list with it. `restoring::at_sign_in` reopens nothing
unless the choice is on, whatever a hand-edited list says.
`tests/the_agent_never_reads_what_was_open.rs` reads every manifest in the
workspace and every source file of `alo-agentd`, `alo-turn`, `alo-capability`,
`alo-protocol` and `alo-broker`: none of them reaches this crate or names its
file. Report:
`docs/autonomy/updates/log-out-switch-user-and-reopen-what-was-open.md`.
**Two findings in it rather than silences:** *switch user* is refused at a
**locked** screen, because a road to the sign-in would be a fifth thing on a lock
screen and what a lock screen may show is task 1's decision — so a household
sharing one machine has the first person unlock before the second signs in, and
the change that would fix it is a change to `alo-locking`; and
`docs/contracts/person-settings.md` still describes four kept files, naming
neither `sleeping.toml` (task 2), `displays.toml` (task 3) nor `leaving.toml`,
which is now three files owed to that contract rather than two. Not on hardware
— nothing here opens a device, ends a session or draws a dialogue; the shell
draws it later. **Depends on:** 1.

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

**Status:** **Done, 2026-09-19.** `crates/alo-notifying`, new and owned by this
plan. A `Notification` is four things — who sent it, a title, a body, and at
most three things to do, each a name its sender knows it by and a label a person
reads — and **there is no public constructor**: the only roads are
`arriving::from_an_application`, which takes the `alo_portals::Allowed` that
judging a request against the person's grants produced and refuses an `Allowed`
for any of the other fifteen portals; `arriving::from_the_agent`, which refuses
a grantee that is an application; and `arriving::from_alo_os`, held to the same
rules as everybody else. `Quiet::now` asks **the screen first** — `alo_in_use`,
read — so a screen being shared or recorded holds notifications with every
setting a person could touch turned off, and a machine that could not ask its
media server holds them too and says so; only then does it ask the person's own
switch and the hours they set aside (`QuietHours`, a stretch that wraps through
midnight, kept in `notifying.toml` through `alo-kept` (ADR 0038) alongside
`do-not-disturb`, and refused in this crate's words when it begins and ends at
once). `deciding::arrives` hands the notification to `alo_locking::Seat::arrives`
**before anything else**, so task 1's rule is task 1's own code; `Became::Held`
carries a `Why` and no notification, and none of the five sentences it can say
has a gap anything could be put into. What a person missed waits in `Missed`
until they dismiss it — **in the session's own memory, with no serde, no path
and no file anywhere in it**, which is a narrowing of *until they dismiss them*
the report states plainly: signing out empties the list as dismissing does.
An agent's own notification carries its mark and says *the agent* in words
before it is terracotta (ADR 0010), and no application can wear either. **An
application cannot notify with an action that answers an approval**: an `Action`
is a name and a label, a `Picked` is addressed to the sender and refuses an
action the notification never offered, `alo-approving` is a dev-dependency of
the tests and not a dependency of the crate, and
`tests/a_notification_cannot_answer_an_approval.rs` puts a real proposal on a
real approval surface and finds it still waiting after the person has picked
*approve*, *yes* and *allow*. Report:
`docs/autonomy/updates/notifications-and-do-not-disturb.md`.
**Owed, and not this task's to pay:** `docs/contracts/person-settings.md` still
describes four kept files and now names none of `sleeping.toml` (task 2),
`displays.toml` (task 3), `leaving.toml` (task 5) or `notifying.toml`, which is
four files owed to that contract rather than three. Not on hardware — nothing
here draws, opens a device or reaches a bus; the shell draws it later.
**Depends on:** 1.

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

- **Acceptance:** every sentence these five crates can say — `alo-locking`,
  `alo-sleeping`, `alo-displays`, `alo-leaving` and `alo-notifying` — is in the vocabulary
  with a translator's note, and one walk — lock, suspend with the lid, resume,
  unlock, dock to a second display, undock — produces the exact sequence a person
  meets, recorded in the report as a table and held by one test that fails if a
  sentence changes without the table; no sentence names `logind`, DRM, EDID, a
  connector name or any other part of the machinery (`docs/features.md`: *a
  person never learns the name of anything we rented*).
- **Constraint:** nothing here re-decides what the sentences describe.
