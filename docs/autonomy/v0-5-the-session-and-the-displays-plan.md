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

**Status:** **Done, 2026-09-20.** The audit is
`crates/alo-sleeping/tests/every_sentence_this_workstream_says.rs` and the walk
is `crates/alo-sleeping/tests/the_walk_from_lock_to_resume_to_a_new_desk.rs`.
Both live in `alo-sleeping` because **no crate in this workspace may name all
five**: `alo-sleeping`, `alo-leaving` and `alo-notifying` each hold a test that
reads every manifest in the workspace and refuses any dependant but
`alo-saying` and `alo-shell`, and widening one of those lists — or writing into
`alo-saying`, which this plan's header says it reads and never edits — would
have paid for an audit with the guarantee the audit exists to keep. So the
audit reads the **assembled vocabulary by key area**, which is the stricter
question (what the machine says, not what a crate claims) and is complete
because each of the five already holds its own test that every one of its keys
sorts under its own area; `alo-locking`, `alo-sleeping` and `alo-displays` are
read both ways and the counts must agree.

**The walk found what the audit could not.** Every sentence in `alo-displays`
was clean, and a connector name was reaching a person anyway — through the gap:
a screen that says nothing about itself was **named** to its owner by the
socket's own string, *eDP-1*, which is what the kernel's side of a graphics card
calls a connector and is the only screen most laptops have. So this task added
`crates/alo-displays/src/plugged_into.rs`: seven declared strings — *Built-in
screen*, *HDMI socket 1*, *DisplayPort socket 2*, *DVI socket 1*, *VGA socket
1*, *Socket 3*, *Another screen* — and `Identity::named_in`, the one road from a
screen into a sentence, which puts a make and model in as data and one of these
in as words alo OS said (`Filling::and_said`), so a translated sentence is only
as translated as the piece inside it. Nothing about **remembering** a screen by
its socket changed: that is task 3's decision and it stands. The sixteen-row
table a person really meets is in the report, and one test parses it out of that
file rather than holding a copy. Report:
`docs/autonomy/updates/every-sentence-and-the-walk-from-lock-to-resume-to-a-new-desk.md`.
Not on hardware — no lid has closed, nothing has suspended and no screen has
been plugged into anything; the base underneath the walk is a `Logind` in the
test file that counts what it was asked. **Depends on:** 1, 2, 3, 4, 5, 6.

- **Acceptance:** every sentence these five crates can say — `alo-locking`,
  `alo-sleeping`, `alo-displays`, `alo-leaving` and `alo-notifying` — is in the vocabulary
  with a translator's note, and one walk — lock, suspend with the lid, resume,
  unlock, dock to a second display, undock — produces the exact sequence a person
  meets, recorded in the report as a table and held by one test that fails if a
  sentence changes without the table; no sentence names `logind`, DRM, EDID, a
  connector name or any other part of the machinery (`docs/features.md`: *a
  person never learns the name of anything we rented*).
- **Constraint:** nothing here re-decides what the sentences describe.

### 8. Switching to another person at a locked screen

**Status:** **Done, 2026-09-21.** The answer is **yes, once**, and the argument
is `docs/decisions/0061-a-locked-screen-offers-a-road-to-the-greeter.md`. It
separates what a lock screen may **show** — task 1's four things, unchanged, and
`alo_locking::LockScreen` is still the type of exactly those — from what it may
**offer**, and it refuses the shape most desktops ship: a list of who has an
account here is a disclosure about people who are not at the desk, and ADR 0024
had already decided this product's greeter asks for a name typed rather than
offering one to pick. The road is `crates/alo-locking/src/somebody_else.rs`.
`SomebodyElse` **has no generic parameter and carries no session**, so a value
that could hold a notification the seat is keeping does not exist; the road is
`&self`, so *not ended, not signed out, not unlocked* is the signature rather
than a paragraph; and the one refusal is real — a machine whose accounts stand
at *make an account* is `NotWhileLocked`, because *make an account* offered over
somebody's locked session is an offer to create one on their machine.
`tests/a_road_to_the_greeter_and_no_second_one.rs` reads this crate's shipped
code (its `cfg(test)` modules cut off, which its own header says why) and holds
that one file names what a greeter stands at, that the road names no `Session`,
`held`, `uid`, `Knock`, `signs_in` or `password`, and that no crate an agent's
request is carried out in depends on `alo-locking` at all.
`alo_leaving::switching::asked` now locks — which leaves a locked seat exactly
as it is — and asks that road on both ways in, so *switch user* means one thing
at a desk and at a lock screen; task 5's finding is closed rather than restated.
One string joins the vocabulary, `locking.somebody-else`, and task 7's audit
reads it without being edited. Report:
`docs/autonomy/updates/switching-to-another-person-at-a-locked-screen.md`.
Not on hardware — nothing here draws, opens a device or ends a session; the
shell draws the road later, and `docs/contracts/lock-screen-rendering.md` gains
it in that change rather than this one. **Depends on:** 1, 5.

Task 5 built *switch user* and then found it unreachable where a household
actually needs it: `alo_leaving::switching::asked` locks this session and hands
the screen to the sign-in, and **a screen that is already locked refuses it**,
because a road to the sign-in would be a fifth thing on a lock screen and what a
lock screen may show is task 1's decision. So two people sharing one machine
have the first unlock — and type their own password — before the second can sign
in at all. That is the wrong way round: the person who has to prove who they are
is the one who wants in, not the one who has gone.

**This is a decision before it is any code.** Task 1's acceptance is published
and says *nothing that would reach a person is shown on the lock screen except
the time, the lock image, the battery, and that the machine is locked*, and it
gives reasons that still hold. Either that rule stands and a household is told
plainly to unlock first, or it gains a fifth thing and the reasons for the other
four have to survive it. Whichever way it goes, the argument is written down
before the code is: a lock screen is the surface this product will be judged on
by somebody standing at somebody else's desk, and *we added a button* is not a
record anybody can audit.

The shape of an answer, if the answer is yes: a road to the **greeter** is not
the same as a fifth thing *about this session*. It shows nothing about who is
signed in, nothing about what is waiting and nothing that was on the screen —
`alo_greeting::Standing` is already what a greeter draws from and already refuses
to name a session it was not built for. That is the case to make or to refuse,
and it is the one a reviewer will look for.

- **Acceptance:** the decision is recorded — as an ADR under `docs/decisions/`
  if a lock screen may show a fifth thing, because task 1's rule is one this
  product argues in public, and as a paragraph in the task's report if it may
  not; **whichever it is, `alo-locking` holds it as a type rather than as
  prose**, with the refusal path tested as carefully as the road: a locked seat
  either produces an `alo_greeting::Standing` and nothing else about the session
  it came from, or it answers the same `NotWhileLocked` everything else does, and
  a test reads this crate's own source for any second road; the session that was
  locked is **not ended, not signed out and not unlocked** by any of it, which is
  task 1's clause 8 and stays held by
  `tests/nothing_here_ends_a_session.rs`; nothing a person could read on the way
  names who was signed in, what was waiting, or what was open (task 5's list is
  never consulted); and if the answer is *no*, the sentence a household reads
  instead is declared here with a translator's note and joins task 7's audit.
- **Constraint:** nothing here draws, and the greeter is `alo-greeting`'s —
  read, never edited. No second road to a password: whatever a locked screen
  offers, signing in is still `alo_greeting::Greeting::signs_in`. This task may
  not touch `alo-sessiond`, and it may not make switching a thing an agent can
  ask for.

### 9. The four files this plan keeps, in the contract that describes them

**Status:** **Done, 2026-09-21.** `docs/contracts/person-settings.md` gains a
section apiece for `sleeping.toml`, `displays.toml`, `leaving.toml` and
`notifying.toml`, each in the shape the existing four have, and each held to
its crate by that crate's new `tests/the_contract_describes_this_file.rs`: the
keys the section lists are the keys the crate writes, the file the section
shows is byte for byte the file the crate writes, every example it offers as
reading reads, and every example it calls refused is refused in the sentence it
names with nothing in the file honoured. **The count was worse than the
omission**, and it was worse than the task knew: the folder held **ten** files
and not eight — `keyboards.toml` (`alo-keyboards`) and `gestures.toml`
(`alo-desktops`) are kept the same way and belong to no plan named here. All
ten are in the table, the sentence says ten, and each of the four new tests
reads that number against the rows under it, so the count can no longer drift
from the table. Eight of the ten have a section; the two that do not are
marked as such rather than left to look described. `alo-displays` also turns
out not to check unknown keys *inside* an arrangement, which the contract now
says plainly and task 10 closes. Report:
`docs/autonomy/updates/every-file-in-a-persons-folder-in-the-contract-that-describes-them.md`.
Not on hardware — nothing here opens a device; it is a contract and four tests.
**Depends on:** 2, 3, 5, 6.

`docs/contracts/person-settings.md` is a **published surface**: it is what a
third party writing anything that reads a person's folder builds against, and
`CLAUDE.md` says contracts outlive code. It says *since 2026-09-15 there are
four*, lists them in a table, and gives each a section of its own with its keys,
its `format`, what a missing file means and what a file that will not read is
told — each held to its crate by that crate's
`tests/the_contract_describes_this_file.rs`.

This plan has since added **four more** and described none of them:
`sleeping.toml` (task 2), `displays.toml` (tasks 3 and 4), `leaving.toml`
(task 5) and `notifying.toml` (task 6). Three of those four tasks said in their
own status paragraph that the debt was owed and not theirs to pay, and the count
in that sentence went from two to three to four while nobody paid it. It is this
plan's to pay, because the four crates that keep those files are this plan's.

**What the contract is wrong about matters more than what it is missing.** A
reader today is not told *there may be more*; they are told there are four, with
a date. Somebody writing a backup tool, a migration, or an organisation's
provisioning from that sentence writes something that silently drops half of
what a person chose.

- **Acceptance:** `docs/contracts/person-settings.md` describes all eight files —
  the table gains a row for each of `sleeping.toml`, `displays.toml`,
  `leaving.toml` and `notifying.toml` with its keeper, its `format` number and
  its keys, and each gains a section of its own in the shape the existing four
  have: every key with the values it takes, what a missing file means, what a
  file that will not read is told and in whose words, and a refused example per
  way a file can be wrong; the sentence that counts them is true after the
  change and says how it stays true; **each of the four crates gains
  `tests/the_contract_describes_this_file.rs`**, reading the contract's own text
  the way `alo-appearance`'s does rather than restating it, so a key added to a
  file without a line in the contract fails in the change that adds it; and the
  refusal path is held as carefully as the road — a test per crate that a file
  the contract says is refused really is refused **whole**, with nothing in it
  honoured (ADR 0038).
- **Constraint:** no file's shape changes to make it easier to describe, and no
  key is added, renamed or removed. This is a contract catching up with four
  crates, not four crates being rewritten for a contract. `alo-kept`'s rule is
  read and never edited, and the four existing sections are not rewritten — a
  correction to one of them is a separate change with its own argument.

### 10. An arrangement with a key nobody declared

**Status:** **Done, 2026-09-21.** `crates/alo-displays/src/arrangement.rs`'s two
written shapes gain `deny_unknown_fields`, which is the whole of the code: an
`[[arrangements]]` table and an `[[arrangements.screens]]` row now have exactly
the keys this crate declares, and one alo OS does not know refuses the whole
file as `displays.kept.not-understood` — the sentence a value it cannot take
already gives, because the key is inside a value rather than at the top of the
file, so there is no top-level key to name and `FileNotRead::key` answers
`None`. `alo_displays::keeping::at_sign_in` then answers with a machine that has
arranged nothing and the refusal beside it, as it does for every other way this
file can be wrong. Five tests hold it: the two refusals, each through a real
file on a real disk; the same file with only the declared keys reading, with its
arrangement, its second screen's place and its main screen honoured; a file this
alo OS wrote reading back unchanged, because nothing about the shape moved and
yesterday's file is today's; and one in `arrangement.rs` that asks the shape
directly for both tables at once. `docs/contracts/person-settings.md` loses the
paragraph headed *One check `appearance.toml` has that this file does not yet*
for the ordinary sentence the other sections have, and gains a refused example
per table; `tests/the_contract_describes_this_file.rs` reads both of the new
examples and holds them to the sentence they name, unloosened.
**What a person notices, and it is a change to what they experience:** a
hand-edited `displays.toml` with a stray key inside an arrangement stops
reading, where yesterday it read and the arrangement was honoured. They are
told that their screen arrangements could not be read, that nothing in the file
has been used and that their screens have been laid out side by side; the file
is not written over, so mending it by hand keeps everything in it, and Settings'
*put this back as alo OS ships it* is the one door that replaces it. Report:
`docs/autonomy/updates/an-arrangement-with-a-key-nobody-declared.md`.
Not on hardware — nothing here opens a device; it is a serde clause, a contract
paragraph and the tests around both. **Depends on:** 3, 9.

Task 9 read `alo-displays` against the rule every file in a person's folder is
kept by, and found one place the rule stops short. A key alo OS does not know
**at the top** of `displays.toml` refuses the whole file, with the key named,
the way it does in all seven other files. A key alo OS does not know *inside*
an `[[arrangements]]` table, or inside one of its `screens` rows, is **read
past**: `format = 1`, an arrangement, and `brightness = 50` on a screen row
reads, and the arrangement is honoured.

Every other nested shape in these four crates already refuses one —
`alo_displays::night_light::Written`, `Between`, `Whereabouts`,
`alo_leaving::open::Written` and `alo_notifying::quiet_hours::Written` all carry
`deny_unknown_fields`. `arrangement.rs`'s two do not, and nothing suggests that
was decided rather than missed.

**It matters more here than the size of the fix suggests.** `alo-leaving`'s
`deny_unknown_fields` is the clause that stops a `title` reaching a person's
folder, and the argument for it is not about tidiness: a file read past is a
file a later release, or a person's own hand, can quietly put something into.
An arrangement row is the one place in this plan's files where a screen is
described, and *which screen this is* is exactly the sort of thing somebody
would be tempted to add a field to.

Task 9 did not fix it, deliberately: its own constraint says the four crates are
not rewritten for the contract, and making a hand-edited file that reads today
stop reading tomorrow is a change to what a person experiences, which deserves
its own sentence in `CHANGELOG.md` rather than arriving inside a documentation
task. The contract says plainly what happens today and tells nobody to rely on
it, which is what made it safe to leave.

- **Acceptance:** an unknown key inside an `[[arrangements]]` table, and inside
  an `[[arrangements.screens]]` row, refuses the whole file — the same
  `displays.kept.not-understood` a bad value gives, since the key is inside a
  value and not at the top of the file, and `alo_displays::keeping::at_sign_in`
  then answers with a machine that has arranged nothing; the refusal path is
  tested in `alo-displays` for both tables and for a key that *is* declared
  still reading; `docs/contracts/person-settings.md`'s `displays.toml` section
  loses the paragraph headed *One check `appearance.toml` has that this file
  does not yet* and gains the ordinary sentence the other sections have, with a
  refused example per table; and
  `crates/alo-displays/tests/the_contract_describes_this_file.rs` goes on
  passing without being loosened to do it.
- **Constraint:** nothing else about the shape changes — no key added, renamed
  or removed, and no `format` moved: a file this alo OS wrote yesterday reads
  today. `alo-kept`'s rule is read and never edited. Say in the change
  description that a hand-edited file with a stray key in an arrangement will
  stop reading, and what the person is told when it does.

### 11. The desk a machine wakes up at

**Status:** **Done, 2026-09-22.** `crates/alo-displays/src/resuming.rs` is the
argument and `Attached::resumed_to` is the one door: the whole reported set at
once, answering a `Resumed` that names every screen that has gone with the
`Moved` an unplug already answers with, every screen that is back with its
`CameBack`, and the one `Note::TheDeskChanged` a person reads. Nothing about
layout is decided there — the set the person arranged for exactly these screens
is restored, a set nobody has arranged goes side by side at each screen's own
size, and what was open on a screen that has gone belongs on the main screen of
what remains, all of it task 3's. A machine that wakes to the same screens
reporting themselves the same way returns before touching anything: nothing
moved, nothing said, the arrangement unchanged down to the places in it. A
machine that wakes with nothing plugged in at all is `NotArranged::NoScreens`
and keeps the screens it went to sleep with, because the next resume is
compared against them and a dock that has not woken yet must not cost somebody
their arrangement. `alo-sleeping`'s `Woke::the_desk`
(`crates/alo-sleeping/src/the_desk.rs`) is where it is asked, so a resume is the
place it happens; that makes `alo-displays` a dependency of `alo-sleeping`
rather than a dev-dependency, which breaks no rule — the three crates that
refuse dependants refuse *theirs*, and `alo-displays` holds no such rule.
`displays.the-desk-changed` is declared with a translator's note and joined
task 7's audit with that test unedited, because the audit reads the assembled
vocabulary by area. One test in `alo-sleeping` walks one laptop through four
resumes — at another desk, at a desk nobody has arranged, at the same one, at
none — and back to the first, with a real sign-in, a real lock and a real sleep
under each. Report: `docs/autonomy/updates/the-desk-a-machine-wakes-up-at.md`.
Not on hardware — no machine has suspended and no cable has moved; the base
underneath the walk is a `Logind` in the test file that counts what it was
asked. **Depends on:** 2, 3.

A laptop suspended at home and opened at the office is the commonest thing this
workstream will be judged on, and it is the one path through it that nothing
decides. `alo_displays::Attached` learns about screens from events —
`unplugged` and `plugged_in`, one cable at a time — and **a machine that was
asleep saw none of them.** It wakes holding the set it went to sleep with: two
screens that are no longer plugged into anything, or one screen where there are
now three.

Task 7's walk docks a screen *after* the resume, which is the hotplug road
working exactly as task 3 built it. What is missing is the step before it: the
moment a machine comes back and the desk is not the desk it left.

**This is not a new layout rule and must not become one.** Everything needed is
already decided — a set the person has arranged is restored (task 3's second
clause), a set nobody has arranged is laid out side by side at each screen's
last size, and what was on a screen that has gone belongs on the main screen of
what remains. What is missing is one door that asks all of it again from *what
is reported now* rather than from the events nobody was awake for, and one
sentence that tells a person the desk changed rather than leaving them to find
out by looking for a window.

- **Acceptance:** `alo-displays` gains the one road from a machine that slept to
  the screens in front of it — the whole reported set, never a cable at a time —
  answering an `Attached` that restores the arrangement the person made for this
  set, lays a set nobody has arranged out side by side, and says where what was
  on each screen that is gone belongs, by task 3's rule and in the same shape an
  unplug answers with; a screen that is still there is not moved and nothing is
  said about it. **The refusal path is held as carefully as the road:** a
  machine that wakes with nothing plugged in at all is refused
  (`NotArranged::NoScreens`) with the screens it went to sleep with left exactly
  as they were rather than thrown away, and a machine that wakes to the same set
  reporting itself the same way moves nothing and says nothing. A `Note` a
  person reads when the set changed while the machine was asleep is declared
  here with a translator's note and joins task 7's audit without that test being
  edited. `alo-sleeping`'s `Woke` is where the road is asked, so a resume is the
  place this happens rather than something every caller has to remember. And one
  test suspends at one desk and resumes at another, at the same one, and at
  none.
- **Constraint:** no window identifier enters `alo-displays` — task 3's clause,
  and what keeps this crate out of what a person had open: the moving is the
  shell's and the decision of where is this crate's. Nothing polls and nothing
  watches: the screens are read once, at the resume, from what the caller was
  handed. The arrangement remembered for the desk the machine left is not
  forgotten — the set it belongs to is simply not the set in front of anybody.
  Nothing in `crates/alo-shell`, no modesetting, and `logind` is rented
  (ADR 0011).

### 12. Every sentence at a desk that changed, and what this plan still owes

**Status:** ready. **Depends on:** 7, 11.

Task 7 walks one person from locking their machine to docking it at another
desk and holds the **sequence** they meet to a table in its report, because
each crate's own tests hold its sentences one at a time and none of them can
ask whether the sentences read as one account. That walk sleeps and wakes at
the same desk: its steps 6 and 7 say nothing, which was right when nothing had
been decided about a desk that changed while the machine was asleep.

Task 11 decided it, and added a sentence — `displays.the-desk-changed` — that
**no recorded sequence meets.** The audit has it, the crate's own tests have
it, and the one thing this workstream holds itself to beyond those is the
thing it is missing: what a person reads, in order, on the morning the desk is
not the desk they left. A resume at a new desk can say four things at once —
the desk changed, what was open on the screen that is gone is now here, this
screen has never been used with this machine, and this one is back — and
whether those four read as an account or as four crates talking past each
other is not a question any of the tests written so far can ask.

And this plan is otherwise finished, which is worth writing down rather than
leaving somebody to work out by reading eleven status paragraphs. One promise
it named is **not met**: `docs/features.md`'s *Per display, so the dock can sit
along the bottom of the laptop and down the side of the external screen*.
`alo_dock::Dock` holds one edge for the machine, this plan reads `alo-dock` and
never edits it, and `alo-dock` belongs to
`v0-5-where-a-persons-settings-are-kept-plan.md`. Task 3 said so in its status
paragraph and `alo_displays::Wearing::of` is named as the one function that
changes when that plan decides otherwise. A promise owed to another plan is
still owed; it is not narrowed, and the place it is recorded should be
somewhere a person reads before this plan is called done.

- **Acceptance:** one walk in `alo-sleeping` — signed in at one desk with a
  screen the person arranged, the lid closed, the machine woken at another desk
  where a screen it has never seen is plugged in, and woken again back at the
  first — produces the exact sequence a person meets, recorded in that task's
  report as a table and held by one test that reads the table out of the report
  rather than a copy of it, exactly as task 7's does; every sentence comes out
  of the machine's one assembled vocabulary through the value that really
  produces it, whole, with no gap unfilled and **no connector name in any gap**;
  task 7's own walk and its table are not edited, because a published report is
  never rewritten and that walk is still true; and the report says in plain
  words what this plan leaves owed — the per-display dock edge, to which plan,
  and what changes in `alo-displays` when it is paid.
- **Constraint:** nothing here re-decides what the sentences describe, and no
  sentence is added or reworded to make the table read better — a sequence that
  reads badly is a finding for a later task with its own argument, not
  something this one edits away. Nothing in `crates/alo-shell`, nothing on the
  machine, and `logind` stays rented (ADR 0011).
