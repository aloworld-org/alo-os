# Every promise of *a person can work on it all day*, against the evidence

*Named `v0.5 — every promise, against the evidence` until 2026-10-02. **A release code is not a subject** — `v0.5` says *when*, never *what*, which is what `CLAUDE.md`'s *names are for strangers* forbids and what the owner asked be cleared out of this repository. **Both evidence documents had the same subject**, so the release was the only thing telling them apart; this one is named for the release's own descriptive title rather than its number. The release this belongs to is in `ROADMAP.md`, which is where release codes live.*

`docs/features.md` is the definition of what alo OS is. This is the other half
of it for the second release: **for every `[v0.5]` line, the test or the report
that shows it, and what is still owed.** It is not a release verdict —
`ROADMAP.md`'s exit gate is — and nothing here ticks anything. It answers one
question, promise by promise: *what in this repository would show somebody that
this is true, and what would not?*

`docs/autonomy/evidence-it-boots-and-the-agent-acts.md` is the same file for the first release, and its
own preamble is worth reading beside this one: the rules about what counts as
evidence are the same, and they were written after the reading they replace.

**A promise that leaves this tier takes its entry with it.** Printing did on
2026-09-29 — `docs/features.md` had said `[v0.5]` since `ROADMAP.md` moved printing
to v1 on 2026-09-26, and the definition is the binding one, so the line is corrected
to **`[v1]`** and its evidence is no longer this gate's to hold. Nothing was
withdrawn: the seven tests and two reports that showed it, and the sentence saying
what is still owed — *a printer; everything is here, and no paper has come out of
anything* — are carried in
[ADR 0078](../decisions/0078-what-printing-owes-is-a-printer.md), which also records
why only *Files and printers shared between paired machines* went to `[v2]`: it is
the one printing promise with nothing built behind it. This note is here rather than
left as an entry because `alo-reconciling` reads every `###` heading as a promise of
this tier, and a heading saying *moved* would be an entry claiming to be one.

## Why this file exists, in one paragraph

The v0.5 gate was read by hand seven times over three months. The eighth reading
took a day and found four boxes left behind by a release cut, promises with no box
at all, a box denying work a plan said was done, and plan tasks reading `ready`
above their own evidence — and it also got **six of its own refusals wrong**, four
on 2026-09-26 and two more found on 2026-09-27, every one of them a negative claim
from a search narrower than the claim.

Every one of those is mechanically detectable. **A person reading 92 promises is a
rule; a crate that fails the gate is a check.** So `crates/alo-reconciling` reads
this file with `docs/features.md` and `ROADMAP.md`, and the checks in
`crates/alo-reconciling/src/the_gate.rs` each name the finding that specified them.

Writing this file made the same mistake five more times, and that is worth knowing
before reading it. Five entries were first drafted as **Shown by: nothing** —
the lock-screen image, the dock's size, a file manager, USB storage, a text editor
and a terminal — and every one of them was wrong, written from the crate whose name
matched rather than from the crate that held the answer. Each of those entries now
says so under itself. **The pattern is always the same: a negative claim made from
a search narrower than the claim**, and it is why this file exists as something a
crate reads rather than as a reading.

## What counts as evidence here

Two things and no third: **a test in this repository**, named by its path, or **a
task report** under `docs/autonomy/updates/`. An ADR is where an argument was
settled, not evidence that anything was built; `ROADMAP.md` is the release's
account of itself and cannot be its own evidence; a file in another repository
cannot be run from here. All three are refused by name.

Nothing here judges whether a test *proves* its promise — no test can judge that.
The reader is the check on that, and this file is written to be read.

## How to read an entry

**Shown by** is evidence that exists now. **Still owed** names what is missing
and why, and a sentence too short to do that is refused — *not yet* is not an
answer.

Two things are true of almost every entry and are written rather than implied:

- **No promise here is shown on a certified machine**, because there is not one
  yet. The first physical install is the owner's, and until it happens every
  *on the machine* half of every promise in this release is owed. An entry that
  does not say so is an entry somebody forgot to finish.
- **A finished plan is not evidence.** Most of these promises have a plan behind
  them whose tasks are done, and a plan is a record of what somebody meant to
  build. The test or the report is the evidence; the plan is where to find it.

## Every v0.5 promise, one at a time

Grouped as `docs/features.md` groups them, in the order it makes them. The
groups are bold lines rather than headings so that every `###` under this one
heading is an entry and nothing else is — `crates/alo-reconciling/src/ledger.rs`
reads it that way, and a heading somebody moved would otherwise take a section
of entries out of the audit without failing it.

**The shell — what a person signs into (ADR 0002)**

### Lock screen, suspend and resume

**Shown by:** `crates/alo-locking/tests/nothing_here_ends_a_session.rs`,
`crates/alo-locking/tests/unlocking_is_signing_in.rs`,
`crates/alo-locking/tests/a_road_to_the_greeter_and_no_second_one.rs`,
`crates/alo-sleeping/tests/the_walk_from_lock_to_resume_to_a_new_desk.rs`,
`crates/alo-leaving/tests/switching_user_locks_this_session_and_hands_over_the_screen.rs`,
`docs/autonomy/updates/what-a-locked-session-is-and-what-the-lock-screen-may-show.md`,
`docs/autonomy/updates/suspend-resume-the-lid-and-what-may-keep-a-machine-awake.md`,
`docs/autonomy/updates/every-sentence-and-the-walk-from-lock-to-resume-to-a-new-desk.md`

**Still owed:** **no lid has ever closed.** Every suspend, resume and lid event
in this repository is a stand-in for what alo OS would ask of the base —
`TheMachinesLogind::sleep` calls logind's `Suspend` over the system bus in code
that no certified machine has run. A lid that has never closed is code, and the
walk that carries a person from locking to resuming at another desk has only ever
walked in a test.

### Multi-monitor, display scaling, hotplug

**Shown by:** `crates/alo-displays/tests/a_screen_that_goes_and_comes_back.rs`,
`docs/autonomy/updates/several-displays-each-with-its-own-background-and-dock.md`

**Still owed:** **no screen has been plugged into anything.** The arrangement is
decided in `alo-displays` and applied by the compositor, which is `alo-shell`'s;
neither the applying nor the scaling a person would see has happened on hardware,
and it needs a certified machine and a second screen at the same time.

### Recovery and rollback screen — reachable when the workspace is not

**Shown by:** `crates/alo-updating/tests/back_to_yesterdays_machine.rs`,
`crates/alo-updating/tests/going_back_to_yesterdays_machine.rs`,
`docs/autonomy/updates/back-to-yesterdays-machine.md`,
`docs/autonomy/updates/the-recovery-and-rollback-screen.md`

*The first of those is the rollback measured rather than argued: two bootable
images on the base alo OS ships, the second differing by one file, the first
installed to a disk and booted under QEMU through three starts. It shows the
going-back. It shows no screen, which is what the sentence below says.*

**Still owed:** the screen itself. What is built is going back to the build the
machine ran before, the decisions such a screen draws, and what a person is told
on the way — the broker's `RollBack` carries it out. What is not built is a
screen drawn *when the workspace is not reachable*, which is the shell's and
which needs a machine that has actually failed to start. Nothing in this
repository has ever failed to start.

### Settings, as one place

**Shown by:** `docs/autonomy/updates/one-place-for-settings-drawn-with-every-section-through-its-own-crate.md`,
`crates/alo-choosing/tests/no_agents_door_reaches_these_settings.rs`,
`crates/alo-choosing/tests/every_sentence_about_a_persons_settings_carries_a_note.rs`,
`crates/alo-portals/tests/the_settings_portal_answers_appearance.rs`,
`crates/alo-shell/tests/settings_source.rs`

**Still owed:** a machine to open it on, and the one finding the report names —
nothing opens Settings yet, because `alo-shortcuts` declares no action for it.
All nine areas are decided, each in its own crate, and the one place **is**
drawn: `alo-shell/src/settings_*.rs` and `nested_settings.rs`, rasterised on a
nested compositor and measured by tests on real files.

*This promise's box in `ROADMAP.md` said **the one place does not exist** on
2026-09-26. That was wrong, and it was wrong because the claim was made without
opening `the-shell-plan.md` — seventeen tasks, sixteen done, which draws
almost every surface that reading called owed.*

### Accessibility: the AT-SPI tree the agent uses is the one a screen reader uses; EN 301 549 conformance is the same work, not extra work

**Shown by:** `crates/alo-adapters/tests/the_accessibility_fallback.rs`,
`crates/alo-adapters/tests/the_accessibility_fallback_on_a_real_application.rs`,
`docs/autonomy/updates/the-accessibility-fallback.md`,
`docs/autonomy/updates/what-each-accessibility-setting-changes.md`

**Still owed:** **the conformance claim, which nothing here assesses.** One tree
for both readers is built and tested against a real application. EN 301 549 is a
standard with clauses, and a conformance claim on the shell is a document
somebody signs against a shell running on a machine — neither the assessment nor
the machine exists. The claim must not be read out of the fallback's tests, which
is what *the tree is shared* would be mistaken for.

### **Set the surface's material** — its colour and how it is finished; per display on a multi-monitor desk

**Shown by:** `docs/autonomy/updates/several-displays-each-with-its-own-background-and-dock.md`,
`crates/alo-appearance/tests/appearance_kept_in_its_own_file.rs`,
`crates/alo-shell/src/screen_background_tests.rs`

**Still owed:** the material, and a screen. What exists is the **colour** half:
`screen_background.rs` draws one screen's chosen colour across the whole of it,
per display, and the tests above hold that. *How it is finished* — the surface
ADR 0075 says a person chooses — has no code and no decision beyond the record's
own sentence, and what somebody sees while choosing one is owed to a machine with
a display along with the Settings panel that offers it.

**And the removal is owed before the promise can be read at all.** ADR 0075 is
accepted and its code half is not done: `Picture`, `Of::File`, `Of::Shipped` and
`Fitting` are still in `alo-appearance`, and
`screen_background_tests.rs` still holds *a picture background is fitted to this
screen*. So this crate currently passes tests for the feature the record removes.

*This entry read **from a file, a folder that rotates, or a solid colour** until
2026-09-29, when the promise was reworded to match ADR 0075.*

### The lock screen shows the same surface and no client's pixels

**Shown by:** `crates/alo-appearance/src/lock.rs`,
`crates/alo-appearance/tests/appearance_kept_in_its_own_file.rs`,
`crates/alo-appearance/tests/the_contract_describes_this_file.rs`

**Still owed:** the rule this promise now makes, and a lock screen somebody walks
past. *No client's pixels* is shown:
`crates/alo-shell/examples/support/the_walk_check.rs` says in its own words that
the lock texture imports no client, and `lock_image_decode.rs` composites
transparency against opaque black so none can show through. **The same surface**
is not shown by anything: `Lock::Its` still takes any background at all, which
was the old *independently*, and nothing asserts the lock screen shows what the
plane shows.

**The picture path is still here and ADR 0075 removes it**:
`lock_image_decode.rs`, `lock_image_fit.rs`, `lock_texture.rs` and
`lock_background_path.rs`. Until they go, the evidence above is evidence about a
lock screen that can still show a photograph.

*This entry read **Set the lock-screen image, independently of the desktop**
until 2026-09-29. Its old form was right that `Lock::Its` is the independence;
the promise changed under it.*

*This entry read **Shown by: nothing** in its first draft, which was wrong, and
wrong the way this whole audit exists to prevent: the claim was written without
opening `alo-appearance`, which has had `lock.rs` and two tests for it all along.*

### **Light and dark**, following the time of day if a person wants

**Shown by:** `crates/alo-appearance/tests/what_this_crate_says.rs`,
`crates/alo-appearance/tests/appearance_kept_in_its_own_file.rs`

**Still owed:** the following. Light and dark are working code and kept in the
person's own file. *Following the time of day* is a schedule, and the one
schedule this repository computes from the sun is `alo-displays`' night light —
nothing joins it to light and dark, and no test asks for the join. A screen to
see either on is owed on top.

### **Accent colour** — four designed hues, each with a value for a light ground and one for a dark

**Shown by:** `crates/alo-appearance/tests/a_palette_with_one_source.rs`

**Still owed:** nothing. The withdrawal was reconsidered on 2026-09-30 and
lifted. `Accent::ALL` is `[Self; 4]` — Indigo, Violet, Moss, Rose — and the doc
comment says four.

This entry read *the withdrawal can be reconsidered* for four days while the
roadmap box went on asking for a fifth hue and `docs/features.md` went on
promising five. **The ledger was right and both documents around it were
wrong** — which is the opposite of the failure this ledger was built to catch,
and worth recording as such: an instrument can be correct and still not be read.
Both are corrected in the same change, and the code half is ticked.

The tick was withdrawn because #185 was read as implementing half of ADR 0067:
the decision said `Token::Terracotta` stops being a palette token and *becomes
`Accent::Terracotta`*, and the token was removed while the accent was never
added. **The decision was the wrong half.** An accent has to reach 4.5:1 on both
grounds; terracotta on cream measures 2.87:1, which this repository's own
contrast test had recorded before any of it. ADR 0067 is amended to say
terracotta is released from being reserved and is **not** added to the set, and
four hues that all read is a better set than five with one nobody measured.

So there is no fifth hue owed and no empty slot waiting for one. What the code
does is what the decision now says.

### **Deep teal is not one of them.** It means the agent and nothing else, so it is reserved

**Shown by:** `crates/alo-conforming/tests/every_clause_that_is_met_names_a_test_that_exists.rs` (clause 11.1.4.1),
`crates/alo-displays/tests/alo_is_legible_under_night_light_and_still_not_a_signal.rs`

**Still owed:** nothing. Reserving the hue is held by a clause and by the accent set not
containing it. The measurement ADR 0067 recorded as *to be measured* — the
readability of terracotta on a cream ground, had it become an accent — **is
measured: 2.87:1**, and that is what decided it could not be one. Deep teal, which
holds the reservation now, measures 5.78:1 on the same ground.
`crates/alo-appearance/src/contrast.rs` holds both figures, and a test refuses the
two to disagree.

### **The agent is never signalled by colour alone** — deep teal always arrives with a mark and a word

**Shown by:** `crates/alo-conforming/tests/every_clause_that_is_met_names_a_test_that_exists.rs` (clause 11.1.4.1),
`crates/alo-displays/tests/alo_is_legible_under_night_light_and_still_not_a_signal.rs`

**Still owed:** the mark and the word wherever the agent's colour appears on a
screen. The rule is a clause and is held for the one surface measured — legible
under night light and still not a signal on its own — and every other surface the
agent's colour reaches is drawn by the shell on a machine that does not exist.

### Text size and scaling, which is an accessibility setting as much as a taste one

**Shown by:** `crates/alo-appearance/tests/appearance_kept_in_its_own_file.rs`,
`docs/autonomy/updates/what-each-accessibility-setting-changes.md`

**Still owed:** text that is actually larger. Scaling is working code and kept in
the person's own file; what nothing here shows is a rendered surface at two
scales, which is the compositor's and is owed to a machine.

### **A fresh machine already looks composed** — the plane ships with a surface of its own, so nobody meets a grey rectangle

**Shown by:** `crates/alo-appearance/src/shipped.rs`

**Still owed:** a screen. `a_fresh_machine_is_not_grey` holds that a machine
nobody has changed shows `Token::Porcelain`, which `token.rs` documents as *the
workspace canvas* — and it asserts against the palette rather than against a
literal, so a release that moves that token moves the machine with it. What
nothing here shows is that surface drawn on a display somebody is looking at,
which is the compositor's and is owed to a machine.

*Until 2026-09-29 this entry read **Shown by: nothing that survives ADR 0075**,
and explained that its only evidence — the `the_default_wallpaper_is_installed`
test and the artwork
report — was for the photograph the record deletes. That was the honest reading
while the removal was pending. The removal landed, the test went with it, and
`alo-reconciling` caught the citation still standing: **evidence that moved is
evidence nobody can run, and it reads exactly like evidence that is still
there.***

*And this paragraph then made the same mistake one level down: it quoted the
dead test **as a path**, so a note about a pointer that no longer lands was
itself a pointer that no longer landed. Written without a path or an extension
from 2026-09-30, the way `alo-dock` records its own dead names. Found by
`alo-citing`'s check that every path this repository's documents name lands —
which found it in the sentence explaining the fault it checks for.*

*This entry read **Wallpapers shipped with the image, so a fresh machine is not
grey** until 2026-09-29, and its **Still owed** said the promise was plural and
one wallpaper was not enough. The promise is now that there is no wallpaper.*

**The ordinary things a desktop must do**

### The dock's size, and whether it hides when a window needs the room

**Shown by:** `crates/alo-dock/src/layout.rs`,
`crates/alo-dock/src/hiding.rs`, `crates/alo-shell/src/dock_room.rs`,
`crates/alo-dock/tests/dock_kept_in_its_own_file.rs`,
`docs/autonomy/updates/appearance-dock-and-shortcuts-keep-their-own-files.md`

**Still owed:** **a screen to see it on**, and nothing else in this repository.

*That sentence was untrue when it was written, and this entry carried it from
2026-09-27 to 2026-09-30.* A screen was not the only thing owed: **nothing
computed `TheRoom` from real windows**, so the person could choose the
behaviour, the dock knew what to do when told, and nothing ever told it.
`TheRoom::` appeared nowhere outside `alo-dock`. The caller is
`crates/alo-shell/src/dock_room.rs`, told the windows by
`Server::window_areas` at the one place holding both a server and a frame, and
`docs/design/when-the-dock-gives-way.md` settles what *a window needs the room*
has to mean — four readings were available and three of them oscillate.

**The shape of the error is worth more than the fix.** This entry did not say
*the caller is missing*; it said *nothing else is owed*, which is a claim about
everything rather than about one thing, and nothing could contradict it without
enumerating the whole. A completeness claim is the hardest kind to check and
the easiest kind to write.

The size is there: a dock is laid out on a screen at a text scale, the names give
way to icons where there is not room and say so, and **the dock never takes more
than its share** at any text size.

**The hiding is there too, as of ADR 0076's change, and it was two-thirds there
before it.** `crates/alo-dock/src/hiding.rs` has held the choice and the answer —
four combinations, one of which is hidden — since 2026-09-27, and `Changes` wrote
it to `dock.toml`. What was missing was the middle: nothing resolved a person's
choice against what the release ships, so a written answer could not be read back,
and the two strings for the rows a person picks between had no code that could say
either of them. `Dock::hiding`, `Dock::showing` and `Hiding::said` are that middle,
and `crates/alo-shell/src/settings_lines.rs` now draws the two rows.

*This entry read **still owed: the hiding** and cited `layout.rs` writing* whether
it hides when a window needs the room is v0.5 *at the place it would go. That
sentence was stale from 2026-09-27; it survived because the thing it described was
half-built, and half-built read as not built until the edge came out and left the
hiding as the only setting there was.*

*This promise had no box in `ROADMAP.md`'s v0.5 gate until 2026-09-26, and this
entry read **Shown by: nothing** in its first draft — written off `dock.rs`'s
opening line without opening `layout.rs`, which holds the sizing.*

### **Divide the screen** — drag a window to an edge to take half, a corner to take a quarter

**Shown by:** `crates/alo-dividing/tests/halves_and_quarters_that_hold.rs`,
`crates/alo-dividing/tests/a_division_never_overlaps_or_leaves_a_gap.rs`,
`crates/alo-dividing/tests/the_walk_through_a_working_morning.rs`,
`docs/autonomy/updates/a-split-halves-and-quarters-that-hold.md`

**Still owed:** the drag. What is held is the arithmetic of a division and the
sentences around it — never an overlap, never a gap, and a walk through a working
morning. The edge and the corner a person drags a window to are the compositor's
pointer road, and no pointer has ever been dragged on a machine.

### Remember a split, so returning to a pair of windows restores the arrangement rather than the last position of each

**Shown by:** `crates/alo-dividing/tests/a_split_remembered_per_display.rs`,
`docs/autonomy/updates/a-split-remembered-per-display.md`

**Still owed:** a person returning to it. The remembering is held per display and
tested; what is owed is a session on a machine that ends and begins again with the
windows in it.

### Splitting works on an external display independently of the laptop's own

**Shown by:** `crates/alo-dividing/tests/a_split_remembered_per_display.rs`

**Still owed:** an external display. *Independently* is decided and tested
against two named displays in a test; nothing in this repository has had a second
screen attached to it, so the independence is arithmetic rather than a
measurement.

### Drag and drop between applications

**Shown by:** `crates/alo-handing/tests/a_drop_carries_what_a_paste_carries.rs`,
`crates/alo-handing/tests/a_drop_on_the_agent_is_not_a_grant.rs`,
`docs/autonomy/updates/drag-and-drop-and-context-menus.md`

**Still owed:** two real applications, and a drag between them. What is held is
what a drop carries and that a drop on the agent is not a grant — the second is
the refusal that matters and it is tested. The drag itself is the compositor's and
the applications are not written.

### Right-click context menus, wherever a person expects one

**Shown by:** `crates/alo-menus/tests/a_menu_is_a_closed_list_of_actions.rs`,
`crates/alo-menus/tests/no_entry_reaches_the_agent_unless_it_says_so.rs`,
`docs/autonomy/updates/drag-and-drop-and-context-menus.md`

**Still owed:** *wherever a person expects one*, which is a claim about every
surface and cannot be shown by a crate. The menu's behaviour is decided in the
report; the surfaces are the shell's and the four applications', and the
applications do not exist.

### Touchpad gestures: scroll, zoom, swipe between workspaces

**Shown by:** `crates/alo-desktops/tests/gestures_from_a_touchpad.rs`,
`crates/alo-desktops/tests/swipes_use_the_desktop_switch.rs`,
`crates/alo-desktops/tests/gesture_preferences_are_kept.rs`,
`docs/autonomy/updates/touchpad-gesture-intents.md`

**Still owed:** a touchpad. The gestures are read from events and turned into
intents, and a swipe uses the same desktop switch a keyboard does — all of it from
synthesised events. No finger has touched anything.

*This promise's `ROADMAP.md` box said **no touch gesture is read** on 2026-09-26.
That was wrong: it was written after reading one file, and `alo-desktops` had
three tests for it.*

### **Keyboard layouts, switched easily** — and dead keys and a compose key that work

**Shown by:** `crates/alo-keyboards/tests/a_keyboard_for_every_language.rs`,
`crates/alo-keyboards/tests/dead_keys_and_compose_through_the_rented_tables.rs`,
`crates/alo-keyboards/tests/switching_is_one_shortcut.rs`,
`crates/alo-keyboards/tests/keyboards_are_kept_in_the_persons_folder.rs`,
`docs/autonomy/updates/keyboards-layouts-dead-keys-compose-input-methods.md`

**Still owed:** a keyboard somebody types on. Dead keys and compose go through
the rented tables rather than a table of ours, which is the part that would
otherwise be quietly wrong; switching is one shortcut. Every key in every test is
a synthesised one.

### Input methods for non-Latin scripts

**Shown by:** `crates/alo-keyboards/tests/an_input_method_is_added_without_knowing_its_name.rs`,
`docs/autonomy/updates/keyboards-layouts-dead-keys-compose-input-methods.md`

**Still owed:** the method running. What exists is adding one without knowing its
name — the part a person meets — and the engine that composes the characters is
rented and has never been started on a machine.

*This promise has **no box at all** in `ROADMAP.md`'s v0.5 gate, and it has a
report and a test. It is one of three such promises found on 2026-09-27 and the
only one of the three with evidence behind it.*

### Virtual desktops

**Shown by:** `crates/alo-desktops/tests/desktops_a_person_arranges.rs`,
`crates/alo-desktops/tests/every_promise_is_on_every_desktop.rs`,
`docs/autonomy/updates/virtual-desktops.md`

**Still owed:** a screen to switch on. The arranging is held, and *every promise
is on every desktop* is the test that stops a desktop being a place where a rule
does not apply.

### Screenshots: whole screen, one window, a selected region — to a file or the clipboard

**Shown by:** `crates/alo-capturing/tests/a_person_takes_a_picture_of_their_screen.rs`,
`crates/alo-capturing/tests/nothing_leaves_when_a_picture_is_taken.rs`,
`crates/alo-capturing/tests/what_cannot_be_captured.rs`,
`crates/alo-capturing/tests/the_agent_reaches_the_screen_by_one_road.rs`,
`docs/autonomy/updates/notifications-the-capture-tools-and-the-in-use-indicator.md`

**Still owed:** a screen with something on it. The scopes are decided, *nothing
leaves* is tested rather than asserted, and what cannot be captured is named. The
pixels come from the compositor on a machine.

### Annotate a screenshot without opening anything else

**Shown by:** `crates/alo-capturing/tests/a_blur_cannot_be_taken_off.rs`,
`docs/autonomy/updates/annotation-without-opening-anything-else.md`

**Still owed:** the surface a person draws on. *A blur cannot be taken off* is
the one property worth more than the feature — an annotation the recipient can
undo is a privacy hole — and it is tested. The drawing is the shell's.

### Screen sharing for calls

**Shown by:** `crates/alo-capturing/tests/a_share_is_picked_every_time.rs`,
`crates/alo-capturing/tests/an_application_asking_is_judged_first.rs`,
`crates/alo-capturing/tests/the_walk_through_a_call.rs`,
`docs/autonomy/updates/sharing-the-screen-in-a-call.md`

**Still owed:** a call. *A share is picked every time* — no remembered
permission, no session — is the refusal this promise turns on and it is tested.
There is no conferencing application on this machine and no certified machine to
run one on.

### **A visible indicator whenever the screen, camera or microphone is in use** — by any application, including ours

**Shown by:** `crates/alo-in-use/tests/two_lines_and_neither_is_the_other.rs`,
`crates/alo-in-use/tests/a_stream_through_the_media_server_is_listed.rs`,
`crates/alo-capturing/tests/a_picture_of_the_screen_is_on_the_indicator.rs`,
`crates/alo-capturing/tests/while_it_records_the_indicator_says_so.rs`

**Still owed:** **the camera and the microphone**, which are carried to v2 by the
owner's decisions of 2026-09-26 and 2026-09-27 — so *by any application* is held
for the screen and is owed for two of the three devices the promise names. The
media-server test skips where no session bus and no WirePlumber exist, and prints
the skip rather than passing quietly.

### Status area: clock, battery, network, volume, brightness

**Shown by:** nothing.

**Still owed:** **a location, before an implementation.** It waits on
`docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md`,
which took *at the far end of the dock, wherever the dock is* off this promise —
a clock is not something a person opens or brings into focus, so it is not the
Dock's — and handed the question *where does the status area go* to the shell's
own plan, the smallest canvas worth showing. **The promise is not withdrawn.** A
machine still needs those things.

**What went, and what this entry said before.** It read *shown by* two reports,
and *still owed: a fifth* — four of the five were drawn, inside the dock's status
area, by a raster in the shell that is deleted in the change carrying the
record, because drawing them somewhere else would be a lane picking the location
the record reserves. **So four things that were drawn are not drawn now.** The
file that decides what a clock *says* is kept, since that was never the part
about where it goes; the report below names it.

**The fifth is still unnamed**, and that question outlived the drawing: the
report was *the status area's four* and the promise names five. This entry does
not guess which is absent, and it does not let five be read out of a report that
says four.

**The egress indicator did not go with them.** It is its own entry below, it has
a corner of its own, and it sits where it sat.

### The egress indicator sits at the far end of the dock, above it, so "nothing has left this machine" sits where a person already glances

**Shown by:** `crates/alo-indicator/tests/the_egress_indicator_on_a_screen.rs`,
`docs/autonomy/updates/the-egress-indicator-drawn-in-the-status-area.md`,
`docs/autonomy/updates/the-egress-indicator-on-a-screen.md`

**Still owed:** the screen it sits on. Drawn and measured; the sentence a person
reads is owed to a machine with a display.

*It was drawn* in the status area *until ADR 0076, which took the status area off
the Dock. The indicator did not move — same corner, same pixels, and its test
keeps the numbers it had before the record so that it can say so. What it is no
longer beside is a clock, because nothing draws one.*

### A file manager, with trash, and archives that open

**Shown by:** `crates/alo-software/tests/what_a_fresh_machine_has.rs`,
`crates/alo-files/src/archiving.rs`,
`docs/autonomy/updates/what-a-fresh-machine-has.md`

**Still owed:** **the installing.** The applications are decided and pinned —
`org.kde.dolphin` 26.04.3 for the file manager, which is also the trash, and
`org.kde.ark` from the same release for the archives a file manager hands on, each
named by the identifier its source knows it by and the version it was decided at
on 2026-09-15. Nothing has booted the image. Since #229 the installing exists:
`alo-shipping` is built by the recipe, installed to `/usr/libexec`, carries
`alo-shipping.service` — *the applications a fresh machine has*, after
`network-online.target` — and is in the recipe's own `systemctl enable` list
beside the daemons, with `crates/alo-image` holding the recipe to it
(ADR 0073: at first boot, not at image build). The seven identifiers appear
nowhere under `image/` and that is correct rather than missing, because the list
is `shipped.toml`'s and the installer reads it. **What has never happened is a
boot.**

*This entry read **Shown by: nothing** and *there is no file manager in this
repository* in its first draft. It is a pinned upstream application, which is what
this project does with applications, and `alo-software` is where that is held. The
same mistake is in `ROADMAP.md`'s *ordinary desktop* box, which said the decision
was owed twelve days after it was taken, and is corrected there.*

### USB drives and external storage that appear when plugged in

**Shown by:** `crates/alo-drives/tests/a_drive_is_only_ever_mounted_or_ejected.rs`,
`crates/alo-drives/tests/the_disk_service_on_a_bus.rs`,
`docs/autonomy/updates/every-sentence-and-the-walk-from-a-new-printer-to-a-recovered-disk.md`

**Still owed:** **the appearing.** What a drive is, its filesystem and its health
come from the rented disk service, read on a real bus in a test; the broker mounts
and ejects one and **only ever** those two, against what the service reports at
that moment, for the person's own login rather than text a request carried. What
nothing does is notice a drive arriving and put it somewhere a person sees — that
is the desktop's — and no USB device has been plugged into anything this repository
runs on.

*This entry read **Shown by: nothing** in its first draft, written off
`alo-changing-drives` without opening `alo-drives`.*

### File associations — what opens what, changeable by a person

**Shown by:** `crates/alo-applications/tests/what_opens_what.rs`,
`crates/alo-applications/tests/from_a_call_to_a_window.rs`,
`crates/alo-portals/tests/open_with_is_answered_from_what_opens_what.rs`,
`docs/autonomy/updates/every-sentence-and-the-walk-to-a-working-application.md`

**Still owed:** the *changeable by a person* half's surface. What opens what is
decided, and the open-with portal answers from the same place rather than a second
list — which is the property that stops the two disagreeing. Where a person
changes it is Settings, drawn but not reachable, and there is nothing installed to
choose between.

### A text editor and an image viewer, so a fresh machine is not helpless

**Shown by:** `crates/alo-software/tests/what_a_fresh_machine_has.rs`,
`crates/alo-adapters/tests/the_reference_adapter_on_a_real_bus.rs`,
`docs/autonomy/updates/what-a-fresh-machine-has.md`

**Still owed:** **the installing.** Both are decided and pinned —
`org.gnome.TextEditor` 50.1 and `org.gnome.Loupe` 50.0, the viewer chosen because
it decodes each image in a separate sandboxed process — and the text editor is
also the reference adapter, driven end to end through its own automation
interface. Nothing has booted the image. Since #229 the installing exists:
`alo-shipping` is built by the recipe, installed to `/usr/libexec`, carries
`alo-shipping.service` — *the applications a fresh machine has*, after
`network-online.target` — and is in the recipe's own `systemctl enable` list
beside the daemons, with `crates/alo-image` holding the recipe to it
(ADR 0073: at first boot, not at image build). The seven identifiers appear
nowhere under `image/` and that is correct rather than missing, because the list
is `shipped.toml`'s and the installer reads it. **What has never happened is a
boot.**

### A terminal

**Shown by:** `crates/alo-software/tests/what_a_fresh_machine_has.rs`,
`docs/autonomy/updates/what-a-fresh-machine-has.md`

**Still owed:** **the installing.** `app.devsuite.Ptyxis` 50.1 is pinned, and the
part that matters is held rather than described: `Shipped::decided` refuses the
list unless the terminal is one **no agent can be granted** (ADR 0043). Law 2
forbids the agent running arbitrary commands and says nothing about the person,
and that refusal is what makes the distinction real instead of stated. Nothing has booted the image. Since #229 the installing exists:
`alo-shipping` is built by the recipe, installed to `/usr/libexec`, carries
`alo-shipping.service` — *the applications a fresh machine has*, after
`network-online.target` — and is in the recipe's own `systemctl enable` list
beside the daemons, with `crates/alo-image` holding the recipe to it
(ADR 0073: at first boot, not at image build). The seven identifiers appear
nowhere under `image/` and that is correct rather than missing, because the list
is `shipped.toml`'s and the installer reads it. **What has never happened is a
boot.**

### **Search your own files, without asking anything** — by name, kind, date and contents

**Shown by:** `crates/alo-finding/tests/search_your_own_files_from_an_index_on_the_machine.rs`,
`crates/alo-finding/tests/one_search_over_every_indexed_folder.rs`,
`crates/alo-finding/tests/a_search_answers_in_time_and_says_what_it_did_not_read.rs`,
`crates/alo-finding/tests/nothing_here_opens_a_socket_or_asks_anybody.rs`,
`crates/alo-finding/tests/a_search_over_every_folder_timed.rs`,
`crates/alo-finding/tests/an_index_made_whole_for_a_folder_larger_than_one_walk.rs`,
`docs/autonomy/updates/search-your-own-files-is-an-index-on-the-machine.md`,
`docs/autonomy/updates/one-search-over-every-indexed-folder.md`,
`docs/autonomy/updates/search-answers-in-time-and-says-what-it-did-not-read.md`

**Still owed:** the surface that shows an answer, which is the desktop lane's. The
index is a file under the person's own directory, *without asking anything* is
tested by reading the shipped source for a socket, an answer says what it did
**not** search beside what it found, and it is timed at ten thousand files.

*This promise's `ROADMAP.md` box called the eleventh task of the machine-measured
plan **open** — a folder larger than the walker's bound indexed only to the bound.
That task was finished on 2026-09-14 and the index is made whole. Found on
2026-09-27 by the check that reads a box against the plan it names.*

### **What is running, and what it is using** — processes, memory, disk and network in a window

**Shown by:** `crates/alo-measuring/tests/what_is_running_is_read_from_the_kernel.rs`,
`crates/alo-measuring/tests/nothing_here_acts_or_asks_who_is_asking.rs`,
`crates/alo-measuring/tests/the_measurements_are_asked_the_way_everything_else_is.rs`

**Still owed:** the window. Every measurement is read from the kernel rather than
guessed, and the crate cannot act — which is what makes it safe to answer inside a
turn. Drawing it is the shell's.

### **What is filling the disk** — shown as sizes you can open up and click through

**Shown by:** `crates/alo-measuring/tests/what_is_filling_is_a_tree_of_sizes.rs`

**Still owed:** the clicking through, which is the shell's.

**The line for what undo is holding was built on 2026-09-30** — task 15 of
`docs/autonomy/the-machine-measured-plan.md`, the plan that owns the crate.
`crates/alo-measuring/src/undo.rs` and
`crates/alo-measuring/tests/what_undo_is_holding_has_its_own_line.rs`: a line
beside the tree rather than a node in it, because a snapshot's bytes are shared
with the live files and a node would double-count them; three states and never
a zero; and `Holding::of` takes the answer rather than defaulting it, so ADR
0045's fourth term is held by the compiler rather than by anybody remembering
it. A test counts one folder with two different undo answers and asserts the
two trees are identical, which is what says the tree's sizes did not move.

*This entry said `alo-measuring` mentions no snapshot and no undo, and that was
true when it was written and for three days after: the gap was found twice
independently, by this ledger and by the keeps-itself plan's task 15, and
written down both times before anybody built it.* **What every machine here now
answers is *not on this machine*** — a person's home is not a subvolume, so
there is nothing for a snapshot to be of — and that is a sentence rather than a
zero, because a machine that cannot keep an undo is not a machine holding none.

### **Install applications**, sandboxed, from Flathub or a repository the organisation runs; update and remove them

**Shown by:** `crates/alo-software/tests/installing_updating_and_removing.rs`,
`crates/alo-software/tests/from_nothing_to_a_working_application.rs`,
`docs/autonomy/updates/installing-updating-and-removing-an-application.md`,
`docs/autonomy/updates/every-sentence-and-the-walk-to-a-working-application.md`

*Those tests hold each clause of the acceptance and put each refusal beside the
act it refuses, against a stand-in for the rented tool that records **every act it
was asked to do** — so a refusal is shown to have reached nothing rather than only
to have returned an error. They say themselves what they cannot show: the real
tool on a real machine.*

**Still owed:** an application actually installed. The three verbs and what a
person is told are decided and walked; nothing has been fetched from Flathub, and
`image/Containerfile` installs no application runtime, so *sandboxed* has never
been the case rather than the plan.

### **One list of what has been granted to what** — agents and applications in the same place

**Shown by:** `crates/alo-granted/tests/the_grants_a_person_can_see.rs`,
`crates/alo-granted/tests/applications_on_the_one_list.rs`,
`docs/autonomy/updates/one-list-of-grants-for-agents-and-applications.md`

**Still owed:** the list on a screen. *One* place is the promise and it is held by
a test that puts applications on the same list as agents rather than beside it.
The drawing is Settings', which is drawn and not reachable.

### Portals: file chooser and documents, open-with and default applications, notifications, print, screenshot, screen capture

**Shown by:** `crates/alo-portals/tests/the_portal_backend_answers_on_a_real_bus.rs`,
`crates/alo-portals/tests/a_portal_request_is_a_grant.rs`,
`crates/alo-portals/tests/open_with_is_answered_from_what_opens_what.rs`,
`crates/alo-portals/tests/the_settings_portal_answers_appearance.rs`,
`crates/alo-portals/tests/the_network_monitor_portal_answers_the_network.rs`,
`crates/alo-portals/tests/a_caller_is_named_by_the_process_the_bus_holds.rs`,
`crates/alo-portals/tests/what_an_application_was_answered_is_kept.rs`,
`docs/autonomy/updates/one-keyring-behind-the-secret-portal.md`

**Still owed:** **nothing starts the backend.** The backend is written and it
answers on a real bus in a test; no unit, no service file and nothing in
`image/Containerfile` starts it on a machine, so no application has ever reached
it. *A portal request is a grant* is the property that matters and it is tested.

*This promise's box said **no portal backend exists** on 2026-09-26. That was
wrong — it was written after grepping one spelling of one interface name. What is
true is the sentence above, and it is a different claim.*

### **Secret storage** — one keyring behind the Secret portal

**Shown by:** `crates/alo-secrets/tests/one_keyring_behind_the_secret_portal.rs`,
`crates/alo-secrets/tests/the_secret_portal_answers_on_a_real_bus.rs`,
`crates/alo-secrets/tests/a_real_keyring_answers.rs`,
`crates/alo-secrets/tests/nothing_ships_the_fixture.rs`,
`crates/alo-secrets/tests/which_bus_is_reached.rs`,
`docs/autonomy/updates/one-keyring-behind-the-secret-portal.md`,
`docs/autonomy/updates/a-real-keyring-answers.md`

**Still owed:** an application storing a secret in it. A real keyring answers on
a real bus, *nothing ships the fixture* is the test that stops a test double
reaching a machine, and the quirk about libsecret not being told which bus is
written down. No application exists to keep a password for.

### **Session management**: log out, switch user, lock, and reopen what was open

**Shown by:** `crates/alo-leaving/tests/logging_out_asks_every_application_and_names_what_stayed.rs`,
`crates/alo-leaving/tests/what_was_open_is_applications_and_places.rs`,
`crates/alo-leaving/tests/switching_user_locks_this_session_and_hands_over_the_screen.rs`,
`crates/alo-leaving/tests/the_agent_never_reads_what_was_open.rs`,
`crates/alo-leaving/tests/nothing_here_reaches_into_an_application.rs`,
`docs/autonomy/updates/a-session-that-really-ended.md`

**Still owed:** applications to reopen. All four verbs are built, and the two
refusals — the agent never reads what was open, and nothing here reaches into an
application — are tested. What is owed is a session on a machine with something
in it.

### Unsandboxed installation as a deliberate, clearly-marked act

**Shown by:** nothing.

**Still owed:** **all of it, and the entry used to imply otherwise.** It waits on
`docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md`,
which brought this promise forward from v1 and is the only record saying an
unsandboxed install is a thing alo OS does at all. It named the
report about installing, updating and removing an application — which is a
different promise, the sandboxed road. Searched for `unsandboxed`, *without a
sandbox*, *outside the sandbox*, *no sandbox* and `--system` across every crate
and `docs/features.md`: **nothing in this repository distinguishes an install that
is sandboxed from one that is not.** `alo-software/src/asked.rs`'s `--system` is
machine-wide versus per-login, which is a different axis. So there is no
deliberate act to mark, nothing to mark it with, and no refusal for a managed
machine to make — three things, where the entry said one.

### Audio in and out, with device switching that works mid-call

**Shown by:** `crates/alo-sound/tests/a_headset_plugged_in_mid_call_takes_the_call.rs`,
`crates/alo-sound/tests/a_mute_is_silence_not_a_low_volume.rs`,
`crates/alo-sound/tests/the_walk_through_a_working_days_devices.rs`,
`docs/autonomy/updates/sound-out-and-in-and-switching-mid-call.md`

**Still owed:** a headset. Mid-call switching is the hard half and it is decided
and walked; *a mute is silence, not a low volume* is the refusal that stops a
mute being a mistake somebody can be heard through. No sound has come out of
anything.

### Bluetooth: pairing, audio, keyboards, mice

**Shown by:** `crates/alo-bluetooth/tests/nothing_pairs_that_a_person_did_not_choose.rs`,
`crates/alo-bluetooth/tests/a_bluetooth_pairing_grants_nothing.rs`,
`crates/alo-bluetooth/tests/a_machine_with_no_bluetooth_says_so.rs`

**Still owed:** a radio. Pairing is built and its two refusals are tested —
nothing pairs that a person did not choose, and a pairing grants nothing — and *a
machine with no Bluetooth says so* is the ADR 0063 shape rather than a failure.
Audio, keyboards and mice over it are three device classes none of which has been
seen.

*This promise has **no box** in `ROADMAP.md`'s v0.5 gate. `Devices` in the **v1**
section names Bluetooth, which is a second disagreement: `docs/features.md` makes
this promise at `[v0.5]`.*

### Camera and microphone

**Shown by:** `crates/alo-cameras/tests/a_camera_is_named_by_what_it_is_not_by_a_number.rs`,
`crates/alo-cameras/tests/off_means_there_is_nothing_left_to_open.rs`,
`crates/alo-cameras/tests/an_application_with_a_grant_gets_nothing.rs`,
`docs/autonomy/updates/a-camera-is-a-thing-not-a-number.md`,
`docs/autonomy/updates/the-camera-is-not-a-candidate.md`

**Still owed:** **carried to v2 by the owner's decision of 2026-09-27**, with
task 4 of `docs/autonomy/devices-and-media-plan.md` scheduled rather than
open. The reports are what was learnt before it was carried, and they are here so
the next person starts from them rather than from nothing.

### Media playback, and the codecs people actually have files in

**Shown by:** `crates/alo-playing/tests/what_this_machine_plays_is_what_the_decision_says.rs`,
`crates/alo-image/tests/the_image_can_play_what_the_decision_says.rs`,
`docs/autonomy/updates/what-this-machine-plays.md`,
`docs/autonomy/updates/what-the-image-carries-to-play-with.md`,
`docs/autonomy/updates/the-image-carries-a-media-server.md`

*The pair holds ADR 0051 to the code and to the recipe — the crate's list is the
decided list, and the image carries what the decision says. Neither plays a file,
which is the half carried to v2.*

**Still owed:** **carried to v2 by the owner's decision of 2026-09-26** — task 1
of the devices and media plan, scheduled. What the image carries and what this
machine plays are written down; playing a file is not done and is not this
release's.

### Power management, battery, sleep on lid close

**Shown by:** `crates/alo-power/tests/this_machines_battery_and_its_profiles.rs`,
`crates/alo-power/tests/a_battery_running_down_is_told_about_once.rs`,
`docs/autonomy/updates/a-battery-and-what-a-machine-will-not-guess.md`,
`docs/autonomy/updates/suspend-resume-the-lid-and-what-may-keep-a-machine-awake.md`

**Still owed:** a battery and a lid. *Told about once* is the property — a
warning repeated is a warning ignored — and *what a machine will not guess* is the
refusal to invent a time remaining. Every reading in every test is a stand-in.

### Night light and display colour

**Shown by:** `crates/alo-displays/tests/alo_is_legible_under_night_light_and_still_not_a_signal.rs`,
`docs/autonomy/updates/night-light-and-display-colour.md`,
`docs/autonomy/updates/publishing-night-light-through-a-task-branch.md`

**Still owed:** a screen whose colour actually changes, which is the compositor's.
*When* and *how warm* are kept apart because a person changes them for different
reasons; the schedule is worked out on the machine from a latitude and longitude
rather than asked of anybody; and the two answers a polar circle forces — the sun
does not set today, the sun does not rise today — are said rather than papered
over.

*This promise had **no box** in the v0.5 gate until 2026-09-26, and it was the
first found whose code was already finished. It had two reports and nothing to
tick it in.*

### Regional formats and timezones per language, and a keyboard layout offered with it

**Shown by:** `crates/alo-formats/tests/no_date_is_written_anywhere_else.rs`,
`crates/alo-keyboards/tests/a_keyboard_for_every_language.rs`,
`docs/autonomy/updates/regional-formats-and-timezones-per-language.md`

**Still owed:** the *regional* half, and the timezone. **This entry said no test
in this repository held any of it, and that was wrong** — the same mistake this
file warns about five times, made in it. `no_date_is_written_anywhere_else.rs`
reads the shipped source of every crate and fails on a date built by hand, which
holds the one property the promise rests on: every date a person sees goes
through `alo-formats` and none is assembled by whoever happened to be writing
that line. `a_keyboard_for_every_language.rs` holds the *keyboard layout offered
with it* clause. What no test holds is that the format a person gets is the one
their region uses, or anything about a timezone — a third of the promise, not all
of it.

### The agent answers in the language you asked in

**Shown by:** `docs/autonomy/updates/the-agent-answers-in-the-language-it-was-asked-in.md`,
`docs/autonomy/updates/a-translators-line-held-to-the-same-rule.md`,
`crates/alo-portals/tests/what_applications_were_answered_reads_back_in_the_persons_language.rs`,
`crates/alo-saying/tests/a_published_sentence_keeps_its_key.rs`

**Still owed:** a language other than English with a translation in it. Every
sentence this machine can say is keyed and collected into one vocabulary, a
published sentence cannot quietly change its meaning under its key, and a portal
answer reads back in the person's own language. What is owed is the 24 EU
languages themselves, of which this repository ships one.

### Screen reader, magnifier, high contrast, larger text

**Shown by:** `docs/autonomy/updates/the-screen-reader-and-the-tree-it-reads.md`,
`docs/autonomy/updates/the-tree-a-screen-reader-reads.md`,
`docs/autonomy/updates/what-each-accessibility-setting-changes.md`,
`crates/alo-shell/tests/the_tree_a_reader_finds.rs`

**Still owed:** **the magnifier and high contrast**, neither of which has code:
what each setting changes is written down, and the tree a reader finds is tested,
but nothing magnifies anything and no high-contrast palette exists. Larger text is
the *text size and scaling* promise above and is working code. A screen reader
running is owed to a machine.

### Sticky keys, slow keys, and keyboard-only operation of everything

**Shown by:** `crates/alo-shell/tests/every_road_a_keyboard_takes.rs`,
`docs/autonomy/updates/what-each-accessibility-setting-changes.md`

**Still owed:** **sticky keys and slow keys**, which have no code. Keyboard-only
operation is held for the shell's own roads by a test that walks every one of
them; *everything* includes the four applications that do not exist, so the claim
cannot be made yet even for what is built.

**`alo-agentd` — the agent's reach into the machine (ADR 0001)**

### **The grant is a boundary the kernel imposes, not a rule the daemon follows** (ADR 0013)

**Shown by:** `crates/alo-bounding/tests/the_kernel_refuses.rs`,
`crates/alo-bounding/tests/the_kernel_refuses_a_delete_or_a_link.rs`,
`crates/alo-bounding/tests/the_kernel_refuses_a_departure.rs`,
`crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs`,
`crates/alo-bounding/tests/a_hard_link_is_inside_every_boundary.rs`,
`crates/alo-bounding/tests/what_an_o_path_handle_is.rs`,
`crates/alo-agentd/tests/a_turn_is_bounded_by_the_kernel.rs`

**Still owed:** a turn a person started. The boundary is imposed by this kernel
and the refusals are read back from it rather than asserted — the delete, the
link, the departure and the hard link each have a test that the kernel says no.
What is owed is the same on the certified machine's kernel, and an agent turn
that a person actually caused.

### **And the kernel is taught what a turn is** (ADR 0015)

**Shown by:** `crates/alo-bounding/tests/a_turn_is_this_thread.rs`,
`crates/alo-bounding/tests/two_processes_take_turns_on_this_kernel.rs`,
`crates/alo-bounding/tests/what_a_turn_inherits.rs`

**Still owed:** the certified machine's kernel. A turn is a thread the kernel
knows about, two processes can hold turns at once without confusing them, and what
a turn inherits is decided rather than left to the loader.

### So the record stops being anybody's account of themselves

**Shown by:** `crates/alo-bounding/tests/the_unwatched_mutations_are_written_down.rs`,
`crates/alo-bounding/tests/what_a_turn_inherits_is_written_down.rs`,
`crates/alo-bounding/tests/what_a_bound_turn_can_still_change.rs`,
`docs/autonomy/updates/kernel-enforcement-for-file-renames.md`

**Still owed:** the record being written from the observation rather than beside
it. What a turn touched is observed and what is **not** observed is written down —
which is the honest half and the one that keeps this from being a claim. Joining
it to `alo-record` so the record is the observation is the rest of the work, and
`docs/autonomy/kernel-enforcement-plan.md` is where its tasks are.

### And it forgets everything that was not an agent

**Shown by:** `crates/alo-bounding/tests/the_boundary_decides_and_forgets.rs`

**Still owed:** the same on a machine with other people's processes on it. A
syscall outside a turn is checked and dropped, which is what stops this being a
surveillance tool; the test holds it for this kernel.

### So the record stops being a claim and becomes an observation

**Shown by:** `crates/alo-bounding/tests/the_unwatched_mutations_are_written_down.rs`,
`docs/autonomy/updates/a-machine-that-missed-what-the-kernel-said.md`

**Still owed:** the joining, as above, and the case the report names — a machine
that missed what the kernel said, and what it must do about it rather than carry
on quietly.

### A turn whose boundary cannot be applied **does not run** — a refusal, not a warning

**Shown by:** `crates/alo-bounding/tests/a_turn_without_a_boundary_does_not_run.rs`,
`crates/alo-agentd/tests/a_turn_is_refused_when_the_boundary_is_gone.rs`,
`docs/autonomy/updates/a-turn-without-a-boundary-does-not-run.md`

**Still owed:** a certified kernel to be refused by. This is the refusal the whole
of ADR 0013 rests on and it is tested twice — in the boundary and at the daemon's
door.

### **System verbs** through the privileged broker: printers, network, updates, storage

**Shown by:** `crates/alo-broker/tests/only_what_a_person_approved_is_handed_on.rs`,
`crates/alo-broker/tests/every_answer_is_written_down_before_it_is_given.rs`,
`crates/alo-broker/tests/the_door_hears_only_the_agent_service.rs`,
`crates/alo-broker/tests/small_enough_to_audit_in_an_afternoon.rs`,
`crates/alo-brokerd/tests/only_the_printer_approved_is_changed.rs`,
`crates/alo-brokerd/tests/only_the_network_approved_is_changed.rs`,
`crates/alo-brokerd/tests/only_the_update_approved_is_carried_out.rs`,
`crates/alo-brokerd/tests/only_the_drive_approved_is_mounted_or_ejected.rs`,
`crates/alo-brokerd/tests/the_unit_is_the_process.rs`

**Still owed:** a person approving something on a machine. All four verb families
are built and each has the same refusal tested — only what was approved is carried
out — and *small enough to audit in an afternoon* is held as a test rather than
hoped for. What is owed is the approval, which needs a screen.

### The accessibility fallback: any application with no adapter is still readable and operable

**Shown by:** `crates/alo-adapters/tests/the_accessibility_fallback.rs`,
`crates/alo-adapters/tests/the_accessibility_fallback_on_a_real_application.rs`,
`docs/autonomy/updates/the-accessibility-fallback.md`

**Still owed:** more than one real application. The fallback is tested against one,
which is worth far more than against none; *any application* is a claim about
software nobody has written an adapter for, and one sample is not that.

**The AI stack — model choice and deployment configurations**

### An address that is not https is refused rather than warned about

**Shown by:** `crates/alo-choosing/tests/an_address_that_is_not_https.rs`,
`docs/autonomy/updates/an-address-that-is-not-https-is-refused-at-the-write.md`

**Still owed:** nothing this entry named. **It said nothing in this repository ran
it; that was wrong.** The test is the `docs/features.md` sentence taken one clause
at a time, each against a settings file on a real disk and against the vocabulary
the whole machine loads rather than the crate's own list — including the exception
for a service on this machine, which is the clause a refusal written from the
headline would have got wrong.

### Test a provider before saving it, so a mistyped key is found now

**Shown by:** `crates/alo-asking/tests/a_provider_is_tested_before_it_is_saved.rs`,
`crates/alo-asking/tests/nothing_here_keeps_the_key.rs`,
`docs/autonomy/updates/a-provider-is-tested-before-it-is-saved.md`

**Still owed:** a provider to reach. The order — test, then save — is decided; no
request has left this machine for a provider, and the key in every test is a
fixture.

### The same model, whichever place you run it

**Shown by:** `crates/alo-models/tests/the_file_the_two_european_entries_mean.rs`,
`crates/alo-models/tests/whose_requantisation_this_catalogue_vouches_for.rs`,
`crates/alo-models/tests/the_pin_an_entry_states.rs`

**Still owed:** running it in two places and comparing. What is held is that the
catalogue's two European entries mean one file, that a requantisation is vouched
for by name rather than assumed, and that an entry states its pin. The comparison
itself needs two machines and a working day on each.

### And our own service gets no exemption

**Shown by:** `docs/autonomy/updates/alos-own-service-as-a-provider-like-any-other.md`,
`crates/alo-models/tests/the_proxy_on_the_road_to_a_provider.rs`

**Still owed:** our own service existing. It is one more provider with no
privilege in `alo-egress` and a policy refusing hosted inference refuses ours; the
account and the billing live outside this repository, so what the machine knows is
an address, a key in the keyring, a region and whether the last request was
accepted.

### A provider that will not say where it runs is reported as **unknown**, never assumed to be nearby

**Shown by:** `docs/autonomy/updates/a-provider-that-will-not-say-where-it-runs-is-unknown.md`

**Still owed:** a test. *Unknown* rather than assumed-European is the whole point
of the promise and it is decided; no test holds it.

### Run a model we never catalogued

**Shown by:** `crates/alo-models/tests/a_model_we_never_catalogued.rs`,
`crates/alo-models/tests/a_brought_file_is_one_the_runtime_answers_to.rs`,
`crates/alo-models/tests/the_pinned_runtime_accepts_what_alo_os_sends.rs`,
`docs/autonomy/updates/a-brought-file-is-one-the-runtime-answers-to.md`,
`docs/autonomy/updates/the-pinned-runtime-and-what-alo-os-sends-it.md`

**Still owed:** somebody's own weights on their own machine. A brought file is one
the runtime answers to rather than one we recognise, and the pinned runtime accepts
what alo OS sends it — tested against the runtime itself.

### The machine warns and then gets out of the way

**Shown by:** `crates/alo-models/tests/candidates_the_box_can_hold.rs`,
`crates/alo-models/tests/sizes_an_entry_can_point_at.rs`,
`crates/alo-models/tests/the_carry_or_fetch_measurement.rs`,
`docs/autonomy/updates/a-model-on-the-disk-sized-for-the-machine-it-lands-on.md`,
`docs/autonomy/updates/candidates-the-measuring-box-can-hold.md`

**Still owed:** a machine whose memory is really that size. *Warns and then gets
out of the way* is the shape — a model too large is offered with the warning rather
than hidden — and the sizes are measured on the development machine.

### What you bring is yours, including its licence

**Shown by:** `crates/alo-models/tests/whose_requantisation_this_catalogue_vouches_for.rs`,
`crates/alo-models/tests/the_grade_the_weights_wait_on.rs`

**Still owed:** the licence stated for everything shipped, which is a claim about
the catalogue's own entries and is held only where an entry states a pin. A reader
cannot get the licence of every catalogued model out of this repository today.

### The dataset, the adapter and the resulting weights never leave the machine

**Shown by:** `crates/alo-adapting/tests/nothing_here_can_send_anything.rs`,
`crates/alo-adapting/tests/the_flow_names_nothing_rented.rs`,
`docs/autonomy/updates/an-adapter-is-the-learning-and-the-base-is-never-touched.md`,
`docs/autonomy/updates/the-weights-carried-once-and-a-runtime-that-does-not-call-home.md`

*The first is the strongest kind of evidence a promise like this can have: it
holds that the crate **has no road to the network at all**, and fails the day
somebody gives it one. Its header says why that is the property worth holding —
an adapted model carries the documents it was trained on, so a road from here to
a socket is a road from somebody's correspondence to somebody else's computer,
whatever the code calls it.*

**Still owed:** a fine-tune. Nothing in this repository trains anything; the
decision that an adapter is the learning and the base is never touched is
recorded, and the runtime not calling home is measured for inference rather than
for training.

### Model runtime versioned *with* the drivers it needs, so an upgrade cannot break a working stack

**Shown by:** `crates/alo-image/tests/the_image_can_play_what_the_decision_says.rs`,
`docs/autonomy/updates/the-pinned-model-runtime-is-on-the-image.md`

**Still owed:** an upgrade that does not break a working machine, which can only
be shown by upgrading one. The runtime is pinned on the image with what it needs.

**Everyday pain — what people actually complain about**

### **"Where is that file?"** Ask in words

**Shown by:** the *Search your own files* entry above — the same crate and the
same tests. `crates/alo-finding/tests/search_files_is_asked_the_way_everything_else_is.rs`
is the agent's road, a read under a grant over the folder whose index is searched.

**Still owed:** asking in words. The verb is reachable and the index answers; the
turn that turns *the contract Anna sent before the summer* into that query is the
agent's, and no agent has been asked anything on a machine.

### "Why is it slow?" and "what is filling my disk?"

**Shown by:** `crates/alo-measuring/tests/what_is_running_is_read_from_the_kernel.rs`,
`crates/alo-measuring/tests/what_is_filling_is_a_tree_of_sizes.rs`,
`crates/alo-measuring/tests/nothing_here_acts_or_asks_who_is_asking.rs`

**Still owed:** the window, which is the shell's, and **the line for what undo is
holding**: ADR 0045's fourth term says `alo-measuring` counts what undo is holding,
by name, and nothing in the crate mentions a snapshot or an undo — so a person
whose disk is full of yesterday's turns is shown a tree that does not account for
them. It is **task 15 of `docs/autonomy/the-machine-measured-plan.md`**. This
is the same pair as the two entries above, because the promise is the agent's way
of asking what they answer.

### "I can't open this file."

**Shown by:** `crates/alo-opening/tests/what_this_machine_can_do_with_a_file.rs`,
`crates/alo-opening/tests/a_file_this_machine_cannot_open_is_explained.rs`,
`crates/alo-opening/tests/a_document_from_pages.rs`,
`crates/alo-opening/tests/a_photo_from_a_telephone.rs`,
`crates/alo-opening/tests/a_drawing_from_a_cad_program.rs`,
`crates/alo-opening/tests/deciding_never_leaves_the_machine.rs`,
`crates/alo-converting/tests/converting_a_real_document.rs`,
`crates/alo-converting/tests/a_machine_that_cannot_run_the_engine_says_so.rs`,
`docs/autonomy/updates/i-cannot-open-this-file-said-properly.md`,
`docs/autonomy/updates/what-this-machine-can-do-with-a-file.md`

**Still owed:** the `.heic` and the `.dwg` actually converting. The three formats
the promise names are recognised and the answer for each is decided; converting
beyond the Office and OpenDocument set waits on its own decision, which
`crates/alo-opening/tests/recognising_three_more_formats_waits_on_its_decision.rs`
holds open rather than letting it be forgotten. *Deciding never leaves the
machine* is tested.

### Undo what the agent did

**Shown by:** `crates/alo-keeping-up/tests/undo_what_the_agent_did.rs`,
`crates/alo-keeping-up/tests/undoing_is_decided_before_it_is_built.rs`,
`crates/alo-changing-undo/tests/the_machine_really_obeys_the_changed_window.rs`,
`crates/alo-changing-undo/tests/a_turn_cannot_arrive_at_this_pane.rs`,
`crates/alo-image/tests/one_filesystem_and_it_can_hold_an_undo.rs`,
`docs/autonomy/updates/undo-what-the-agent-did.md`,
`docs/autonomy/updates/undo-what-the-agent-did-waits-on-a-snapshot-road.md`,
`docs/autonomy/updates/a-disk-that-can-hold-an-undo.md`

**Still owed:** **a snapshot on a real disk.** Build points 1 to 5 of ADR 0045 are
done and every change verb answers *not yet on this machine* until the bracket
exists on it; the image installs onto a filesystem that can hold an undo, and no
machine has been installed. The four terms still owed by other lanes are named in
the ADR, and one of them — `alo-measuring` counting what undo is holding — is
recorded against *what is filling the disk* above.

### Updates that never interrupt

**Shown by:** `crates/alo-keeping-up/tests/what_an_update_may_never_do.rs`,
`docs/autonomy/updates/what-an-update-is-and-may-never-do.md`,
`docs/autonomy/updates/an-update-applied-and-the-same-machine-afterwards.md`,
`docs/autonomy/updates/a-staging-decided-from-an-approval.md`,
`docs/autonomy/updates/updates-through-the-broker.md`

**Still owed:** an update applied to a machine somebody was using. `THE_RULE`
refuses an update as a cause of any disturbance — restarting the machine, closing
an application, interrupting the person — and allows only the person as a cause,
with no third. The refusal is the promise and it is tested; the machine is owed.

**The local network — machines that find each other (ADR 0003)**

### Machines find each other with zero configuration — no addresses typed, no accounts

**Shown by:** `crates/alo-nearby/tests/the_local_network_says_no_more_than_a_machine_exists.rs`,
`crates/alo-nearby/tests/asked_and_answered_over_ipv6.rs`,
`crates/alo-agentd/tests/a_machine_on_two_networks.rs`

**Still owed:** two machines on one network. Discovery says no more than *a machine
exists* — never who or what — which is the refusal ADR 0003 turns on, and it is
answered over IPv6 in a test. A machine on two networks is handled. Nothing has
discovered anything across a real cable.

### Pairing: mutual, deliberate, enumerated, revocable in one action, and expiring

**Shown by:** `crates/alo-nearby/tests/a_pairing_is_made_by_two_people_and_by_nothing_else.rs`,
`crates/alo-nearby/tests/the_machine_that_asks_is_the_machine_that_paired.rs`,
`crates/alo-nearby/tests/a_proposal_reaches_the_other_machine_and_the_answer_comes_back.rs`,
`docs/autonomy/updates/a-pairing-is-revoked-the-way-a-grant-is.md`,
`docs/autonomy/updates/pairing-a-device-and-what-it-is-not.md`

**Still owed:** two people at two machines. Every adjective in the promise is
held: made by two people and by nothing else, the asking machine is the paired
one, revoked the way a grant is. What is owed is the pairing outliving a restart
on hardware.

### The whole of it works with no internet at all

**Shown by:** `crates/alo-nearby/tests/asked_and_answered_over_ipv6.rs`,
`crates/alo-nearby/tests/a_workspace_on_the_network_is_found_not_configured.rs`

**Still owed:** an office. *The whole of it* is a claim about every feature on a
disconnected network, and what is tested is discovery and pairing without one.
Nothing has been unplugged from the internet and then used for a day.

### A self-hosted workspace on the network is **discovered, not configured** — no DNS step

**Shown by:** `crates/alo-nearby/tests/a_workspace_on_the_network_is_found_not_configured.rs`,
`crates/alo-nearby/tests/an_alo_machine_answers_for_the_workspace_it_hosts.rs`

**Still owed:** a workspace. `alo-workplace` is another repository and nothing of
it runs here, so what is tested is that an alo machine answers for one — the
answering, not the workspace.

**Sovereignty, as testable claims**

### **A working day with the runtime alo OS ships produces zero inference egress**, measured at the network boundary

**Shown by:** `crates/alo-asking/tests/a_day_that_never_left.rs`,
`crates/alo-asking/tests/a_day_that_only_looks_like_it_never_left.rs`,
`crates/alo-asking/tests/from_a_question_to_what_left.rs`,
`crates/alo-bounding/tests/what_a_turn_can_reach_on_the_network.rs`,
`crates/alo-bounding/tests/a_private_ipv4_departure_is_held_to_its_network.rs`,
`crates/alo-bounding/tests/a_link_local_departure_names_its_interface.rs`,
`docs/autonomy/updates/the-weights-carried-once-and-a-runtime-that-does-not-call-home.md`,
`docs/autonomy/updates/network-egress-enforcement.md`

**Still owed:** **the working day, and the publication.** The measurement is the
promise — the boundary decides what a turn may reach and the kernel refuses the
rest, tested — and the claim is *zero over a working day, measured and published*.
No day has been worked on a machine and nothing has been published. This is the
promise most likely to be believed on the strength of its tests, and it must not
be.

### Full-disk encryption, enrolled at install

**Shown by:** `crates/alo-encrypting/tests/no_road_enrols_without_a_recovery_key_the_person_kept.rs`,
`crates/alo-encrypting/tests/the_key_is_never_kept_on_the_disk_it_recovers.rs`,
`crates/alo-encrypting/tests/the_sequence_against_a_virtual_disk.rs`,
`docs/autonomy/updates/full-disk-encryption-decided-before-it-is-built.md`

**Still owed:** an install. The sequence runs against a virtual disk, and the two
refusals that matter are tested — no road enrols without a recovery key the person
kept, and the key is never kept on the disk it recovers. Enrolment at install needs
the first physical install.

*This promise has **no box** in `ROADMAP.md`'s v0.5 gate; a line in the **v1**
section names full-disk encryption, which disagrees with the definition's `[v0.5]`.*

**The system and the image**

### Atomic updates with rollback — the previous deployment stays bootable

**Shown by:** `crates/alo-image/tests/how_the_image_is_published.rs`,
`crates/alo-installing/tests/a_btrfs_disk_keeps_what_an_update_passes_over.rs`,
`docs/autonomy/updates/an-update-applied-and-the-same-machine-afterwards.md`,
`docs/autonomy/updates/back-to-yesterdays-machine.md`

**Still owed:** booting the previous deployment. Largely inherited from bootc
rather than written here, which is the honest description; what is tested is that
a btrfs disk keeps what an update passes over and that the image is published the
way the decision says.

### Installed from the machine it replaces

**Shown by:** `crates/alo-installing/tests/installed_in_a_virtual_machine.rs`,
`crates/alo-installing/tests/the_boot_environment_installs.rs`,
`crates/alo-installing/tests/what_the_environment_carries.rs`,
`docs/autonomy/updates/the-installer-walked-on-a-real-windows.md`,
`docs/autonomy/updates/the-windows-installer-program.md`,
`docs/autonomy/updates/the-boot-environment-says-why-an-install-stopped.md`

**Still owed:** **the first physical install**, which is the owner's and is
scheduled. It installs beside a real Windows in a virtual machine and the
switching is walked both ways; the boot environment says why an install stopped
rather than sitting on *still installing*, which was found by a run that did
exactly that.

### The documents people are actually sent open: `.docx`, `.xlsx`, `.pptx`

**Shown by:** `crates/alo-converting/tests/converting_a_real_document.rs`,
`crates/alo-converting/tests/a_file_that_cannot_be_opened_is_recorded.rs`,
`crates/alo-converting/tests/a_pages_document_is_offered_as_a_conversion.rs`,
`docs/autonomy/updates/what-this-machine-can-do-with-a-file.md`

**Still owed:** somebody opening one. All three formats convert, against real
fixture documents, and what a conversion cost is reported rather than hidden. There
is no viewer and no machine.

### A web browser for the open web — a pinned upstream one

**Shown by:** `crates/alo-software/tests/the_web_browser.rs`,
`crates/alo-software/tests/what_a_fresh_machine_has.rs`,
`docs/autonomy/updates/the-web-browser.md`,
`docs/autonomy/updates/what-a-fresh-machine-has.md`

**Still owed:** **the installing.** The policy is built — `http` and `https`
only, a name-and-password in an address refused with its own sentence, and which
application opens one decided as the person's choice, then the one `Shipped` names,
then nothing, and never a person's own application. The pin is
`org.mozilla.firefox` 156.0 from Flathub, MPL-2.0, chosen for engine diversity and
a published enterprise policy that turns telemetry off. `image/Containerfile`
installs no browser.

*`ROADMAP.md`'s box for this said **no browser is pinned** until 2026-09-27.
Firefox had been pinned since 2026-09-15; the claim came from searching `image/`,
where it is true that nothing installs it, and reporting that as the pin being
absent.*

### Installer

**Shown by:** the *Installed from the machine it replaces* entry above, and
`crates/alo-installing/tests/the_boot_environment_installs.rs`.

**Still owed:** the download, the click and the reboot on somebody's own Windows
machine, and the held work behind it: task 20 of
`docs/autonomy/the-installer-plan.md` — a download that stops arriving ending
the install in words — is blocked on task 4 and its work is off-repository.

**The interface (ADR 0065)**

### **Which system the machine starts by default** — changed from alo OS's settings or from Windows

**Shown by:** `crates/alo-brokerd/tests/only_the_next_start_approved_is_set.rs`,
`crates/alo-brokerd/tests/only_the_default_approved_is_set.rs`,
`docs/autonomy/updates/the-default-a-machine-starts-at-changed-by-the-person-who-owns-it.md`,
`docs/autonomy/updates/restarting-into-windows-and-the-menu-a-machine-starts-at.md`,
`docs/autonomy/updates/a-default-nobody-chose.md`

**Still owed:** a machine with two systems on it. Both roads are built and each
carries the same refusal — only what was approved is set — and *a default nobody
chose* is the case that would otherwise be decided silently.

**The person chooses (ADR 0064)**

### Fast Startup: the installer asks

**Shown by:** `crates/alo-installer/tests/the_installer_walked_on_a_real_windows.rs`
— its test *a windows with fast startup on is asked about and turned off* —,
`crates/alo-installer/tests/the_installer_checks_consents_and_stages.rs`,
`docs/autonomy/updates/the-windows-installer-program.md`

**Still owed:** nothing that this entry claimed was owed. **It said a test and the
asking on a real Windows were both missing, and both exist** — and that is the
worse half of the mistake, because the real-Windows walk is the expensive kind of
evidence and it was already there. The walk builds a second Windows with
hibernation and Fast Startup **on** for the express purpose of reaching the
question, which the base every other walk uses cannot reach at all, then asserts
it is asked about and turned off. What remains owed is what remains owed of every
promise in this release: a certified machine.

*A comment at `the_installer_checks_consents_and_stages.rs:170` names a test
`fast_startup_is_asked_about` that exists nowhere in this repository — a stale
name for the one above. `alo-installer` is the owner's; it is noted here rather
than changed.*

### Multi-user on one machine, with per-person grants and no shared agent memor

**Shown by:** nothing.

**Still owed:** **task 18 of `docs/autonomy/accounts-and-session-entry-plan.md`**, written 2026-10-03 after `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md` moved this promise into the release. **Its three clauses are in three different states.** Switching between people is built — `crates/alo-leaving/src/switching.rs` locks the seat and only then hands the screen to the greeter. Accounts are built, with the uid tying one to the Unix login the image declares. **Grants are kept for the machine rather than for a person**: `crates/alo-remembering/src/keeping.rs` holds `THE_GRANTS = "/var/lib/alo/grants.toml"`, one literal path with no person in it. The file is owner-protected, and *one file whose owner is checked* is not *one file per person* — the second account to sign in meets the first one's grants. **The third clause, no shared agent memory, is not measured**: it is a property of what `alo-agentd` keeps across a sign-in, and that is its own reading rather than an assumption to make here.

### ★ **Ask for it** — "make the surface warmer", "use dark after six

**Shown by:** nothing.

**Still owed:** **task 8 of `docs/autonomy/where-a-persons-settings-are-kept-plan.md`**, written 2026-10-03 after `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md` moved this promise into the release. **Half of it is built and the crate says which half.** `crates/alo-appearance/src/time.rs` opens by naming this promise — *use dark after six means six o'clock where the person is, every day* — and `scheme.rs`'s `Following` answers a scheme from a moment passed in, tested at the hour and the half hour, while `changes.rs` holds what a person changed as the difference from the release's defaults. **So a schedule is reachable from a settings panel already. What nothing does is the asking**: there is no road from *make the surface warmer*, said in words, to a proposal naming what would change, to a person approving it. That road is the promise. The propose-then-approve half is `alo-asking`, which the Mac owns and this task consumes rather than edits.

### Portals: USB devices, global shortcuts an application registers, dynamic launchers, remote desktop

**Shown by:** nothing.

**Still owed:** **It waits on `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`**, which moved it from v1 into this release on 2026-10-03 and says what the next step is: this promise has no task in any plan, because none was expected before v1, and one has to be written before anybody can start it. Its evidence has not been assessed either — nothing here is offered as showing it, and nothing here claims none exists. Where to look first: the portal backend answers four interfaces today — Secret, OpenURI, Settings and NetworkMonitor — and none of the four named here is among them. **USB is the one a developer notices within an hour**, and it is a different thing from this release's *USB drives and external storage that appear when plugged in*, which is storage rather than a portal.

### **The shell in the user's language — all 24 official EU languages to begin with**, and a

**Shown by:** nothing.

**Still owed:** **It waits on `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`**, which moved it from v1 into this release on 2026-10-03 and says what the next step is: this promise has no task in any plan, because none was expected before v1, and one has to be written before anybody can start it. Its evidence has not been assessed either — nothing here is offered as showing it, and nothing here claims none exists. Where to look first: there is no language crate, and locale appears in a dozen files none of which is the shell speaking Maltese. This is the largest of the seven by a wide margin, and the promise is explicit that *English plus the big five* does not satisfy it.

### ★ **Guided fine-tune**: LoRA/QLoRA over a granted folder or a tenant's records, as a

**Shown by:** nothing.

**Still owed:** **It waits on `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`**, which moved it from v1 into this release on 2026-10-03 and says what the next step is: this promise has no task in any plan, because none was expected before v1, and one has to be written before anybody can start it. Its evidence has not been assessed either — nothing here is offered as showing it, and nothing here claims none exists. Where to look first: adapters are named across the model crates, which suggests the catalogue knows what one is rather than that a person can make one. The promise is a flow rather than a toolchain, so what is owed is a road somebody can walk.

### ★ **"Make this machine like my old one."** Con

**Shown by:** nothing.

**Still owed:** **It waits on `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`**, which moved it from v1 into this release on 2026-10-03 and says what the next step is: this promise has no task in any plan, because none was expected before v1, and one has to be written before anybody can start it. Its evidence has not been assessed either — nothing here is offered as showing it, and nothing here claims none exists. Where to look first: nothing here carries the phrase or the idea, and the nearest plan measures a machine rather than reproducing one. Configuration as a document, pointed at a person rather than an administrator, has no model yet.

### ★ Cross-machine agent work — an agent may **ask** a paired machine, an

**Shown by:** nothing.

**Still owed:** **task 38 of `docs/autonomy/the-local-network-plan.md`**, written 2026-10-03 after `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md` moved this promise into the release and after the crates were read. **Its two clauses are in two different states, and the second has no road at all.** The asking half is built and built well: `crates/alo-nearby/src/pairing.rs` enforces ADR 0003's *enumerated, visible, revocable in one action, expiring* as properties rather than habits, `deliberating.rs` gives `Pairing` no public constructor so the only thing returning one refuses until both people have confirmed on their own machine, and `confirming.rs` makes a confirmation provable rather than merely said. **But what a pairing permits is a list of two things** — `permitting.rs`'s `MayAskIts` has exactly the variants `Models` and `Workspace`, both asks for something the far machine already offers, and **neither is an agent doing work there.** **Nothing joins a pairing to a grant:** grants are `alo-remembering`'s, `alo-agentd` depends on that crate already, and `alo-nearby` depends on `alo-strings` and nothing else in this workspace — which is how ADR 0003's *discovery reveals presence and nothing else* is true rather than merely stated, and why the join belongs in the daemon that already holds both halves. The task names, rather than assumes, the question it has to settle first: whether this is a third `MayAskIts` variant or a separate mechanism.

### ★ **A Place remembers time** — drag the

**Shown by:** nothing.

**Still owed:** **It waits on `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`**, which moved it into this release on 2026-10-03 and says what the next step is: this promise has no task of its own in any plan, and one has to be written before anybody can start it. Its evidence has not been assessed. **It was `[v1.1]`, the only canvas promise above this release**, and the lane that found it would not move it alone: it is not among the five things the owner named as completion, and it rests on the snapshots undo takes rather than on Places at all.

### **Every screen is a view onto the canvas** — two displa

**Shown by:** nothing.

**Still owed:** **It waits on `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`**, which moved it into this release on 2026-10-03 and says what the next step is: this promise has no task of its own in any plan, and one has to be written before anybody can start it. Its evidence has not been assessed. Two displays are two viewports at their own zoom, which is a different thing from this release's *multi-monitor, scaling, hotplug* and will want reading beside it.

### **A panel out of view costs nothing** — it is a st

**Shown by:** nothing.

**Still owed:** **It waits on `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`**, which moved it into this release on 2026-10-03 and says what the next step is: this promise has no task of its own in any plan, and one has to be written before anybody can start it. Its evidence has not been assessed. It is a cost property rather than a feature — a Place holding forty frames stays usable — so what shows it is a measurement, and nobody has taken one.

### **Every canvas also answers as a list** — its panels

**Shown by:** `crates/alo-shell/tests/every_frame_answers_as_a_list/mod.rs`

**Still owed:** **the machine half, as every promise in this release is.** The code half is done and was done before this promise reached this tier: task 7 of `docs/autonomy/the-smallest-canvas-worth-showing.md` is **Done, 2026-09-29, all three thirds**, with six tests reached through `client_lifecycle.rs` — frames enumerable in a stable order with their names, reachable and focusable by keyboard alone, and what a reader is told not depending on where a frame sits. **This one moved as a correction rather than a widening**: it was `[v1]` and built at v0.5, which is a promise built below its own tier.

### ★ **Give it to alo** — anything sele

**Shown by:** nothing.

**Still owed:** **It waits on `docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`**, which moved it into this release on 2026-10-03 and says what the next step is: this promise has no task of its own in any plan, and one has to be written before anybody can start it. Its evidence has not been assessed. **This move overrides a standing decision**: its tier was kept deliberately when the Stop carve-out put *alo working in a window you put aside* at `[v0.01]` and left its neighbours alone. The owner chose on 2026-10-03 to move it anyway, and it is recorded here so the earlier reasoning is not read as still current.
