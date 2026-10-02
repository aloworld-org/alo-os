# Every promise of *it boots and the agent acts*, against the evidence

*Named `v0.01 — every promise, against the evidence` until 2026-10-02. **A release code is not a subject** — `v0.01` says *when*, never *what*, which is what `CLAUDE.md`'s *names are for strangers* forbids and what the owner asked be cleared out of this repository. **Both evidence documents had the same subject**, so the release was the only thing telling them apart; this one is named for the release's own descriptive title rather than its number. The release this belongs to is in `ROADMAP.md`, which is where release codes live.*

`docs/features.md` is the definition of what alo OS is. This is the other half
of it: **for every `[v0.01]` line, the test or the report that shows it, and
what is still owed.** It is not a release verdict — `ROADMAP.md`'s exit gate is
— and nothing here ticks anything. It answers one question, promise by promise:
*what in this repository would show somebody that this is true, and what would
not?*

## Why it is a file a test reads

The last audit of these promises was done by reading, seven times, and
`ROADMAP.md` records what that cost: **six v0.01 promises with no item and no
line, found one at a time, and twice the reading believed it had found the
last.** A promise is not missed through carelessness. It is missed because a
long document is read in a different order every time and the eye stops where
the last reading stopped.

So `crates/alo-reconciling` reads this file and `docs/features.md` together and
fails the gate when they disagree: a promise no entry is about, an entry about a
promise the definition no longer makes, a named test that is not there, a file
offered as a test that tests nothing, and a promise with no evidence and nothing
owed. **A promise added to `docs/features.md` and not reconciled fails in the
change that adds it** — which is the only moment anybody has the knowledge to
reconcile it.

## What counts as evidence here

Two things and no third: **a test in this repository**, named by its path, or **a
task report** under `docs/autonomy/updates/`. An ADR is where an argument was
settled, not evidence that anything was built; `ROADMAP.md` is the release's
account of itself and cannot be its own evidence; a file in another repository
cannot be run from here. All three are refused by name.

Nothing here judges whether a test *proves* its promise — no test can judge
that, and pretending otherwise would be this audit lying in a new way. The
reader is the check on that, and this file is written to be read.

## How to read an entry

Most entries have both halves, and that is the honest shape of v0.01: the code
is finished and the machine has never been asked. **Both halves are written
down**, because a promise recorded as shown while a third of it is owed is how a
release convinces itself it is finished — a failure `ROADMAP.md` already made
once, in a line ticked outright whose own footnote said the hardware was still
owed.

*Still owed* is deliberately not "not yet". It names the missing thing and why
it is not done, and a sentence too short to do that is refused.

## Every v0.01 promise, one at a time

### Every goal is a canvas

**Shown by:** `crates/alo-shell/tests/one_plane_under_one_viewport.rs`,
`crates/alo-shell/tests/client_lifecycle.rs`,
`crates/alo-shell/tests/the_canvas_walked.rs`,
`crates/alo-shell/tests/the_nested_fixtures.rs`,
`docs/autonomy/updates/the-canvas-walked.md`

The plane and the camera, the viewport that does not move under them, a press
reaching the right surface at three zooms and two pans, dragging, resizing,
panning on three roads, zoom and *Show all*, and ten steps of a walk read back off
a real parent's frames.

**Still owed:** **everything above one Place.** The plan that is closed is one
endless surface, and this promise says *a Place*. There is no canvas `Place` type
in this repository — the word is spent four times on other things — so *where the
person put them* is currently answerable only within a single surface. See task 1
of the canvas plan.

### Tidy this canvas

**Shown by:** nothing yet.

**Still owed:** all of it. Alignment and distribution of a selection, and the
same asked of alo as a proposal shown before anything moves. **The propose-then-
approve half already exists** — `crates/alo-put-aside/src/proposing.rs` shows a
placement before anything moves — and what does not exist is a proposal that
carries a **set** rather than one frame, which is the hard part of this promise
rather than the alignment arithmetic. Reached v0.01 on 2026-09-30 because the
frames promise had already moved with *several can be taken at once* in it,
leaving one operation at two tiers. Where
the work is: task 3 of `docs/autonomy/the-canvas-and-its-places.md` gives it the
selection to act on; nothing yet names the arrangement proposal itself.

### The World, and moving between Places

**Shown by:** nothing yet.

**Still owed:** all of it. **There is no World** — no type, no navigation, and no
promise in `docs/features.md` until the day this entry was written, though
`docs/decisions/0065` has defined `World → Place → Object` since it was written.
Blocked on Place identity existing at all. The increment is task 2 of `docs/autonomy/the-canvas-and-its-places.md`.

### Frames, dragged and resized like a design canvas

**Shown by:** `crates/alo-shell/tests/client_lifecycle.rs`,
`crates/alo-shell/tests/the_frame_in_numbers.rs`,
`docs/autonomy/updates/the-canvas-walked.md`

The name moves a frame at three zooms and two pans and a press in its content does
not; all eight edges and corners resize with the application told its size during
the drag; the four double-headed arrows say which. ADR 0071 settled the shape.

**Still owed:** *several can be taken at once*, *guides and snapping line them
up*, *fit the Place to the screen* as distinct from *Show all*, *fill the screen
with what is selected*, and *double-click to work inside*. **And each of those is
promised with a keyboard form**, which is a larger gap than it reads: there is no
keyboard road to move or resize a frame at all, and the three keyboard forms this
repository has are zoom in, zoom out and *Show all*.

### A frame arrives the shape its work is

**Shown by:** nothing yet.

**Still owed:** all of it. An application declares no opening shape anywhere, and
*remembered per Place once the person changes it* needs both a Place to remember
per and a store to remember in. The arranging crate is the store and has no Place. The increment is task 5 of `docs/autonomy/the-canvas-and-its-places.md`.

### A frame can be dragged out of one Place and into another

**Shown by:** nothing yet.

**Still owed:** all of it, and it is two roads rather than one — through the World
by pointer, and by keyboard with *Move to Place*, ruled by the owner on
2026-09-30. **And task 4 is the thing this promise is not:**
restoring a minimised window returns it to the Place it was already on and
relocates nothing, and the two are separate tasks because they read as one
sentence and are two acts. The increment is task 3 of `docs/autonomy/the-canvas-and-its-places.md`.

### The habits people arrive with still work

**Shown by:** `crates/alo-shell/tests/every_road_a_keyboard_takes.rs`,
`crates/alo-shell/tests/client_lifecycle.rs`

The three canvas actions that exist — zoom in, zoom out and *Show all* — and the
window ones a person arrives with: next window, previous window, close.

**Still owed:** *switch desktops becomes move between Places*, which needs Places.
And *cycle frames* is currently *cycle windows* — the same ring, not yet a ring
per Place.

### Every application lives in a Place

**Shown by:** nothing yet.

**Still owed:** all of it. A dock window carries a patch and a how-it-sits and
**nothing that says which surface it is on**, so an application does not live
in a Place today because there is no Place for it to live in. This is the promise
that makes *one world rather than a modern half and an old half* true, and it
rests entirely on task 1. The increment is task 1 of `docs/autonomy/the-canvas-and-its-places.md`.

### The colours come from a source this repository can read

**Shown by:** `crates/alo-appearance/tests/a_palette_with_one_source.rs`,
`docs/autonomy/updates/a-palette-with-one-source.md`

**Still owed:** the promise says the source *generates* both the shell's
constants and the custom properties the web client needs, and nothing in this
repository generates the second — `alo-workplace`'s stylesheet is a fourth copy
of the palette that nothing checks, in a repository this test cannot read.

### Compositor: Wayland via Smithay

**Shown by:** `crates/alo-shell/tests/client_lifecycle.rs`,
`crates/alo-shell/tests/output_metadata/mod.rs`,
`crates/alo-shell/tests/input/mod.rs`, `crates/alo-shell/tests/pointer/mod.rs`

**Still owed:** every one of those runs against a nested session, which is a
development fixture and proves a client talks to us rather than that a machine
boots to us. No certified machine has ever displayed this compositor.

### Sign-in with an alo identity, and a local account that needs no tenant

**Shown by:** `crates/alo-accounts/tests/a_person_signs_in.rs`,
`docs/autonomy/updates/the-local-account-that-needs-no-tenant.md`

**Still owed:** two things. The alo identity half — a tenant, an account that
is not this machine's — is not built at all. And nothing on the image shows a
person a place to type a password: that is task 13 of the delivery plan, blocked
on `docs/decisions/0024-what-a-person-signs-in-at.md` being accepted.

### The agent overlay: one key, from anywhere

**Shown by:** `crates/alo-overlay/tests/one_key_summons_the_agent.rs`,
`crates/alo-overlay/tests/what_the_overlay_shows_at_rest.rs`,
`crates/alo-context/tests/from_an_invocation_to_a_change.rs`,
`docs/autonomy/updates/agent-overlay-summoning-seam.md`

**Still owed:** nothing draws it. The chord resolves, the request is made once
and the state at rest is derived from the daemon's own answers, and no pixel of
any of that has ever been on a screen.

### Launcher and window management: open, focus, close, tile

**Shown by:** `crates/alo-applications/tests/from_a_call_to_a_window.rs`,
`crates/alo-shell/tests/one_layout_decider.rs`,
`crates/alo-shell/tests/window_close/mod.rs`,
`crates/alo-shell/tests/window_activation/mod.rs`

**Still owed:** the launcher. `alo_shortcuts::Action::Launcher` is a chord with
nothing behind it — no surface lists the installed applications and nothing
starts one from a person's own choice rather than from a verb.

**What changed on 2026-09-26:** tile named the shell's window-tiling Wayland
tests until the v0.5 shell plan's task 16 took the half-output tile out and
replaced it with a division between two windows. The test that stood there is
gone with the mechanism it tested, and what is named in its place holds the
thing that replaced it: that there is exactly one layout decider in this
compositor, and that it is the dividing crate.

### Copy, cut and paste

**Shown by:**
`crates/alo-clipboard/tests/copy_cut_and_paste_across_applications.rs`,
`crates/alo-clipboard/tests/the_clipboard_is_not_a_turns_context.rs`,
`docs/autonomy/updates/the-clipboard-before-there-is-anything-to-draw.md`

**Still owed:** the compositor. `crates/alo-clipboard` is the selection — an
owner, the forms it offers, and a transfer somebody asked for, with every
refusal decided — and **nothing wires it to `wl_data_device`**, so no two
applications on a real machine have ever moved anything between them through it.
That wiring is `crates/alo-shell`'s and is the desktop lane's, and until it
exists images and files are offered forms measured against a fixture rather than
against a drawing program and a file manager. The three v0.5 lines around this
one — the clipboard portal for sandboxed applications, screenshots to the
clipboard — stay where the definition puts them, and *clipboard history* stays
at v1, which is why nothing here remembers anything.

The paragraphs below are the audit's own, left as it wrote them: a finding
rewritten by whoever closed it is a finding nobody can check.

**Was owed, 2026-09-11, before task 20:** all of it. Nothing in this repository
implemented a clipboard — no crate, no Wayland data-device handling in
`alo-shell`, no test. This was a promise with no line at all, and it was the
seventh of the kind the roadmap's audit kept finding one at a time.

**Read against the repository, 2026-09-11: this one needs no screen, no decision
and no machine, and the increment is task 20 of
`docs/autonomy/the-executable-plan.md`.** The audit that found it sorted it with
*the GPU works on first boot* and *boots on one certified machine* into work
waiting on hardware, in one pass while it was finding six things at once, and
that sorting was never examined. It is wrong. A clipboard is a **protocol before
it is a surface**: one client owns the selection and says which types it can
give; another asks for one of those types and is handed a pipe; the compositor
brokers and holds nothing of its own. Every one of those is a value, and every
refusal in it — a type that was never offered, an offer left over from an owner
who has since given the selection up, a paste with nothing behind it — is
decidable with no pixels, exactly as `crates/alo-overlay`, `crates/alo-approving`,
`crates/alo-indicator` and `crates/alo-recounting` decided their surfaces without
drawing one.

Nothing gates it either. **No agent verb touches the clipboard**: the ten in
`docs/contracts/agent-verbs.md` and `docs/by-hand.md` are six about files and
four about applications, so ADR 0001's grant model is not in this promise's way —
copy and paste is a person moving their own text between their own windows. ADR
0005's portal is the *sandboxed application's* route to the same thing and
`docs/features.md` schedules it at v0.5, so what v0.01 promises is the native
Wayland selection between clients of our own compositor. `smithay` 0.7 already
carries the protocol the compositor half would wire in, and `crates/alo-shell`
enables the `wayland_frontend` feature that holds it.

What is **not** closed by the increment, and is named so nobody reads task 20 as
the promise: the compositor wiring is `crates/alo-shell`'s and that crate is the
desktop lane's; images and files as offered types will be measured against real
clients rather than a fixture; and the three v0.5 lines around this one — the
clipboard portal, screenshots to the clipboard — stay where the definition puts
them.

### Keyboard shortcuts, and a person can change them

**Shown by:** `crates/alo-shortcuts/src/changes.rs`,
`crates/alo-shortcuts/tests/what_this_crate_says.rs`,
`crates/alo-shell/tests/shortcut_dispatch/mod.rs`,
`crates/alo-shell/tests/shortcut_dispatch/layout.rs`

**Still owed:** a person changes them through a settings surface that does not
exist. Today a change is a value in a crate, and the file it would be written to
is not read by anything a person can reach.

### Window management: move, resize, snap, tile, minimise, maximise, close

**Shown by:** `crates/alo-shell/tests/window_move/mod.rs`,
`crates/alo-shell/tests/window_resize/mod.rs`,
`crates/alo-shell/src/window_dividing_tests.rs`,
`crates/alo-shell/tests/window_minimize/mod.rs`,
`crates/alo-shell/tests/window_maximize/mod.rs`,
`crates/alo-shell/tests/window_close/mod.rs`

**Still owed:** snap is a shortcut action dispatched to a layout and has no
pointer gesture behind it — dragging a window to an edge does nothing. **And it
now needs two windows:** a division divides between windows, so a chord with one
window open is refused by name, where the half would have put the only window on
half a display with nothing beside it. All of it is still measured in a nested
session rather than on a machine.

**What changed on 2026-09-26:** snap and tile were one mechanism — the shell's
window-tiling module, which computed half an output — and the v0.5 shell plan's
task 16 removed it, because a division becoming session state would have meant a
window's place decided twice, once by a tree of shares and once by a half. A
chord now divides the display between the focused window and the next one. The
Wayland tests that stood here went with the mechanism, and what is named in
their place holds what a chord does now, including what it refuses.

### The alo Dock

**Shown by:** `crates/alo-dock/src/layout.rs`,
`crates/alo-dock/tests/what_this_crate_says.rs`,
`docs/autonomy/updates/the-dock-fixed-to-the-bottom-edge.md`

**Still owed:** **nothing draws a dock.** The labels giving way to icons are
arithmetic that no compositor asks for yet, and everything the Dock is for —
showing what you can open, bringing what is already open into focus, what a click
does, favourites and an overflow area — has no code at all.

*This entry read **The dock, and the person decides where it goes**, and cited
`along.rs` for the two orientations. [ADR
0076](../decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
withdrew that promise and fixed the Dock to the bottom edge; `along.rs` is
deleted. **The owner reversed that decision the next day and the edge choice is
a promise again**, now its own entry above — so the Settings surface is owed
after all, and `along.rs` is the deletion that has to be answered rather than a
file nobody needs. What the Dock's settings section also asks — whether it gives
way when a window needs the room — is its own promise at v0.5.*

### Switching between windows, and between applications

**Shown by:** `crates/alo-shell/tests/window_switch/mod.rs`,
`crates/alo-shell/tests/shortcut_dispatch/mod.rs`

**Still owed:** there is no switcher surface — nothing shows a person what they
are switching to — and, as with everything in the compositor, no machine has
displayed it.

### list, read, find, rename, move, archive

**Shown by:** `crates/alo-files/tests/doing.rs`,
`crates/alo-files/tests/on_this_machine.rs`,
`crates/alo-files/tests/what_one_execution_reaches.rs`,
`crates/alo-files/tests/a_file_with_another_name_is_not_read.rs`

**Still owed:** the six verbs have never run on a certified machine, which is
the half of law 3 only hardware gives.

### open, focus, arrange, close

**Shown by:** `crates/alo-applications/tests/from_a_call_to_a_window.rs`,
`crates/alo-applications/tests/what_this_crate_says.rs`

**Still owed:** the four verbs reach a window through a port that a real
compositor does not implement yet, so no application on any machine has been
opened, focused, arranged or closed by an agent.

### focused window, selection, open document

**Shown by:** `crates/alo-context/tests/from_an_invocation_to_a_change.rs`,
`crates/alo-context/tests/what_this_crate_says.rs`

**Still owed:** the context offered at invocation comes from a fixture rather
than from a compositor: nothing reads a real focused window, a real selection or
a real open document, because no surface has ever invoked a turn.

### Grants: pick a folder, see what is granted, revoke it, and it expires

**Shown by:** `crates/alo-picking/tests/a_person_picks_a_folder.rs`,
`crates/alo-remembering/tests/the_grants_a_machine_keeps.rs`,
`crates/alo-capability/tests/what_this_crate_says.rs`,
`crates/alo-granted/tests/the_grants_a_person_can_see.rs`,
`crates/alo-changing/tests/a_persons_change_reaches_the_daemon.rs`,
`docs/autonomy/updates/native-folder-selection.md`,
`docs/autonomy/updates/a-grant-made-now-reaches-the-daemon-now.md`,
`docs/autonomy/updates/the-grants-a-person-can-see.md`,
`docs/autonomy/updates/a-persons-change-reaches-the-file-the-daemon-re-reads.md`

**Still owed:** the list has never been drawn. `crates/alo-granted` is *see
what is granted* and *revoke it by hand* as a value — derived from the
machine's own kept grants, with revocation through the same
`alo_capability::Grants::revoke` the daemon enforces, an expired grant never
shown as live, and *nothing granted* a sentence — and `crates/alo-changing`
is the person's half of a change made through either: written whole to the
file the daemon re-reads, the knock after the write and never before, and a
machine with no daemon keeping the change for the next sign-in. Putting any
of it on a screen is the compositor's, in `crates/alo-shell`, which no
machine has displayed.

### Every execution recorded with its origin, approval and grant

**Shown by:** `crates/alo-keeping/tests/on_this_machine.rs`,
`crates/alo-recounting/tests/afterwards_ask_what_it_did.rs`,
`docs/autonomy/updates/afterwards-ask-what-it-did.md`

**Still owed:** the record is written and read back on a development machine
only; no record has ever been written by a daemon running on a certified one.

### It runs on the machine you already own

**Shown by:** `crates/alo-models/src/catalogue.rs`,
`crates/alo-driving/tests/against_a_model_on_this_machine.rs`,
`crates/alo-driving/tests/from_a_prompt_to_what_a_machine_offers.rs`

**Still owed:** the catalogue's memory figures are a table until a model has run
on the certified laptop, and seven of the twelve entries have never been
measured anywhere because the measuring box has less memory than they want.

### It works well on a CPU and it works well on a GPU

**Shown by:** `crates/alo-models/src/catalogue.rs`,
`crates/alo-driving/tests/against_a_model_on_this_machine.rs`

**Still owed:** the GPU half entirely. No machine with a card has run any of
this, so *works well* is measured on one side of a promise that says it is
measured on both.

### The catalogue says whether a model can drive the verbs

**Shown by:** `crates/alo-driving/tests/from_a_prompt_to_what_a_machine_offers.rs`,
`crates/alo-driving/src/exercises.rs`, `crates/alo-driving/src/measured.rs`,
`crates/alo-models/src/driving.rs`

**Still owed:** seven of the twelve entries say `not-measured`, which is an
honest answer and not a grade — the catalogue can carry the judgement for
entries nobody has put a question to.

### And it is measured by us, not claimed by the publisher

**Shown by:** `crates/alo-driving/tests/against_a_model_on_this_machine.rs`

**Still owed:** the five measured grades were taken on a development box rather
than on a certified machine, and the measurement has never been repeated on the
hardware the catalogue's figures describe.

### A machine is only offered agent work it can actually do

**Shown by:** `crates/alo-models/src/refusing.rs`,
`crates/alo-driving/tests/from_a_prompt_to_what_a_machine_offers.rs`

**Still owed:** the refusal and its three alternatives have no setup screen to
appear on, so no person has ever been shown the choice this promise is about.

### The GPU works on first boot

**Still owed:** all of it. Nothing in `image/` installs or verifies a driver
stack, no test asks a machine what card it has, and no machine with a card has
booted this image. It is a v0.01 promise with no line anywhere in the
repository, and it is the second such finding of this audit.

**Read against the repository, 2026-09-11: it waits on a machine, and on an
image that carries a stack to accelerate.** `docs/hardware.md` already defines
the promise in four clauses — the display comes up at native resolution with no
configuration, the card is available to the model runtime with no driver
installation and no CUDA or ROCm archaeology, a model runs in one command, and an
upgrade cannot break that stack because the runtime is versioned with the drivers
it needs. Three of the four are answers a machine gives and nothing else does;
the fourth is about what an image contains, and `image/Containerfile` adds two
binaries, two units, two directories and one description to a pinned base and
**carries no model runtime and no weights at all**
(`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md` found
the same gap from the other side). So there is nothing on this image for a card
to accelerate, and a check that the image names a driver stack would be a check
with nothing to hold: which stack is pinned for the certified workstation is an
engine decision under ADR 0011's *configured, never patched*, and the certified
workstation itself is `docs/hardware.md`'s *24 GB VRAM or more* with no model
named in the table. The machine is task 12 of
`docs/autonomy/the-executable-plan.md`, which is scheduled and needs hardware
nobody has plugged in. **Nothing here is reachable by this lane**, and saying so
with the reading behind it is what this entry is for.

### A model runs in one command

**Shown by:** `crates/alo-models/src/runtime.rs`,
`crates/alo-models/src/ollama.rs`, `crates/alo-models/src/weights.rs`

**Still owed:** the runtime adapter has never been run against a real model
service on any machine — the tests put its own protocol to a socket, which is
the shape of the exchange rather than the thing working.

### The local model is what the machine arrives ready to run

**Shown by:** `crates/alo-image/src/weights.rs`,
`crates/alo-image/src/checking.rs`,
`crates/alo-image/tests/what_the_image_owes_the_daemons.rs`,
`crates/alo-choosing/tests/the_three_choices.rs`,
`docs/autonomy/updates/the-weights-a-machine-arrives-with.md`,
`docs/autonomy/updates/the-one-thing-that-serves-the-model.md`,
`docs/autonomy/updates/the-recipe-built-rather-than-read.md`

The promise this entry is about was reworded on 2026-09-11, when
ADR 0025 was accepted as Option D under the owner's standing delegation. It used
to read *model by default — sovereignty is the default configuration*, and what
it gave up is the claim that a value sits in a settings file before a person has
touched one: unbuildable here on purpose, since ADR 0016 keeps that file for the
person and ADR 0024 ships no accounts to write it into. What it took on is
heavier — **a model on the disk of every machine we ship, sized for that
machine** (ADR 0007) — and that is what the image now declares.

The recipe says **which weights a certified machine arrives with**:
qwen3-8b, Q4\_K\_M, the pinned runtime library's own blob addressed by content,
held to a sha256 that is checked before any other step reads the file, with that
artefact's own template pinned and checked beside it. The open
question ADR 0025 left is **decided — the weights ride on the image** — because a
machine that fetches at setup has not arrived ready when it is offline at setup;
docs/quirks.md carries what that costs and what the other answer would have cost.
**Which model is not a preference and is not written in the recipe**: since
2026-09-22 it is the answer alo_models::Catalogue::agent_for_cpu gives for the
16 GB laptop docs/hardware.md certifies first — run here, may be used without
reading a licence first, and measured driving the verbs the way a turn asks —
and crates/alo-image refuses a recipe that names anything else. **And where that
method recommends nothing, the image carries nothing** and the machine says so
through the answer alo-telling already gives, rather than shipping gigabytes of
a model that cannot drive anything. Ten refusals hold each of those, each a
disagreement naming the decision it
breaks. The four setup choices, local first and nothing pre-selected, are the
crate added for them and the settings nothing writes on a person's behalf.

**Something now serves them, 2026-09-11.** image/usr/lib/systemd/system/alo-modeld.service
is the one thing on the machine that serves the model it arrived with, and until
it existed a machine built from this recipe booted with gigabytes of model on its
disk and no process serving it — which is the answer a machine with no model at
all gives. It runs as alo-model (60991), a login and a group of its own that is
neither the person's nor the agent's; it holds no capability and both lines say
so; its store is the directory the weights landed in; it answers at the one
loopback address alo-models knocks at, read off that crate rather than
spelled twice; and it reaches nothing off this machine — an IP deny of everywhere
under an allow list naming this machine alone, which is a kernel-side filter on
its own control group rather than a comment. alo-image holds each of
those, with a twin that breaks one line of a copy of the image and is caught.

**Still owed:** *arrives ready to run*, which is the sentence in the promise.
Three things stand in front of it, and one of them is smaller than it was.
**No machine has booted this image.** The recipe itself was built on 2026-09-11
— 27 minutes, 8.38 GiB, every login number created exactly as asked and
`bootc container lint` passing 13 of its 14 checks — so the import step is now a
measurement rather than a recipe, and the runtime started out of the image it
produced lists the model the machine arrived with. **A build is not a boot**, and
two things the first build measured were owed work rather than evidence, and
were done on 2026-09-12 (task 34): the store carried the weights **twice**,
because `ollama create` leaves the source GGUF beside the blob its manifest
names — the weights stage now removes it after the import and holds every blob
left to the manifest, and the rebuilt image is measured in
`docs/autonomy/updates/the-weights-carried-once-and-a-runtime-that-does-not-call-home.md`;
and the pinned runtime made **two requests to `ollama.com` within eight
milliseconds of starting** — the unit now sets the runtime's own switch beside
the filter, and with it the runtime asks neither. The egress claim above is
**a setting read and, once, watched working — not at a boot**: the unit was
started by the image's own systemd under a container, sixteen packets to the
publisher's port were attempted by its login and none reached the host side of
the bridge while an unfiltered process in the same container was answered.
`docs/quirks.md` carries all of it with versions and dates. A packet counter
beside a *booted* image is still owed, with the boot. **The bar is cleared and
the road to it is not finished, 2026-09-22.** `qwen3-8b` drove 20 of 20 in the
words a turn shows a model, which is why the image carries it — but that grade
was earned **in the envelope**, and a shipped machine's agent turn does not yet
ask that way. So what a machine arrives able to do is load and answer with a
local model, and the last step between that and an agent turn is lane A's
wiring of `alo-asking`'s local door rather than another model. The image's
predicted size with those weights aboard, and the arithmetic it is predicted
from, are in `docs/quirks.md`; no build of the recipe carrying them has been
made. And **who on the machine may ask the model anything is not decided**: a
loopback TCP port has no owner and no mode, so no line in any unit gates it, and
`docs/decisions/0027-who-may-ask-the-model-anything.md` is where that is argued
and priced. That last one does not block this promise — it is about v0.5's
sandbox — and it is written here so the next reader inherits the reasoning
rather than the port. This entry stays owed until a machine boots with the
weights on it, and
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md` is
where the wording it is held to was settled.

### Add your own provider in Settings

**Shown by:** `crates/alo-choosing/tests/the_three_choices.rs`,
`crates/alo-choosing/tests/a_persons_choice_reaches_the_machine.rs`,
`crates/alo-models/src/secret.rs`,
`crates/alo-secrets/tests/a_real_keyring_answers.rs`,
`crates/alo-asking/tests/a_key_reaches_one_provider_only.rs`,
`docs/autonomy/updates/a-persons-choice-written-where-the-machine-reads-it.md`

**Still owed:** the Settings surface — the place in the shell where a name, an
address and a key are typed, which is the compositor lane's and has no pixels
anywhere yet.

**No longer owed, 2026-09-11: adding one by hand.** This entry used to read *a
provider is added by writing the person's own settings file by hand*, and that
was the true state of the repository: `alo-choosing` read the file and nothing
wrote it, so every choice ADR 0016 gives the person was one no surface could
carry out. `alo_choosing::Choosing` is the written change — a provider added, a
model chosen, weights brought, a language picked — landing in the person's own
file whole or not at all, and read back through the door `alo-agentd` reads it
through. What a surface has left to do is show it. The key is still the
keyring's and is still not in the file: there is nowhere in the shape to put one
and the writer cannot invent one, which
`crates/alo-choosing/src/writing.rs`'s round trip refuses by name.

### Setup's fourth choice

**Shown by:** `crates/alo-record/tests/a_machine_with_no_agent.rs`,
`crates/alo-capability/tests/what_this_crate_says.rs`

**Still owed:** there is no setup, so there is no fourth choice to make. What is
built is the machine that results from having made it — the agent's reach gone
at once, nothing further recorded — rather than the moment of choosing.

### A person never learns the name of anything we rented

**Shown by:** `crates/alo-saying/src/rented.rs`,
`crates/alo-saying/src/translated.rs`,
`crates/alo-saying/tests/what_this_machine_can_say.rs`,
`docs/autonomy/updates/a-translators-line-held-to-the-same-rule.md`

**Still owed:** a surface that composed a sentence of its own would pass
through nothing — the checks read the English declarations, the vocabulary
they build, and, since 2026-09-11, every translated line at the moment a
machine loads the file: a line naming a rented component is left out with the
same list and the same argument, the rest of the file is kept, and the refusal
names the file, the key and the language. What keeps a surface from composing
is construction rather than a check — every surface crate's sentence types are
sealed against being made from text — and nothing verifies that mechanically
across the workspace.

### And it is enforced rather than remembered

**Shown by:** `crates/alo-saying/src/rented.rs`,
`crates/alo-saying/src/translated.rs`,
`crates/alo-saying/tests/what_this_machine_can_say.rs`

### Anything an agent verb can do, a person can do by hand

**Shown by:** `crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs`,
`docs/autonomy/updates/every-verbs-by-hand-answer.md`

**Still owed:** the check holds every verb to *naming* a plain way that the
definition promises; it cannot hold one to a plain way that is **built**, and
today none of them is. Six of the ten verbs ship at v0.01 and their surface —
the file manager, the search, the text editor, the terminal — is v0.5, so a
v0.01 machine whose agent is unavailable can do none of the six by hand;
`docs/by-hand.md` says so in its own words and `ROADMAP.md`'s v0.5 exit gate is
where it comes due. One verb, `archive_folder`, has no promised plain way at all
and is recorded as owed: nothing in `docs/features.md` promises **making** an
archive by hand, and the proposed line is in the report rather than added here,
because the scope gate is the owner's.

### And it holds however the agent became unavailable

**Shown by:** `crates/alo-answering/tests/from_a_question_that_failed_to_what_left.rs`,
`crates/alo-answering/src/failed.rs`, `crates/alo-asking/src/ran_out.rs`

**Still owed:** the six ways the agent becomes unavailable are refusals a crate
produces; nothing shows one to anybody, so *the machine loses convenience and
never capability* has never been true of a screen.

### Running out of credit is its own answer, not an error

**Shown by:** `crates/alo-asking/src/ran_out.rs`,
`crates/alo-answering/src/wrong.rs`,
`crates/alo-asking/tests/a_key_reaches_one_provider_only.rs`

**Still owed:** *it says it once* is now a mechanism rather than a hope —
`crates/alo-telling` is the memory, and a repeat of one unavailability produces
no value a surface could word. What is owed is the surface itself: no screen has
ever shown the sentence, once or otherwise.

### And it never nags

**Shown by:**
`crates/alo-telling/tests/a_machine_that_cannot_reach_a_model_says_so_once.rs`,
`crates/alo-telling/src/telling.rs`,
`docs/autonomy/updates/a-machine-that-cannot-reach-a-model-says-so-once.md`

**Still owed:** the memory is a session's and nothing holds one yet, because
nothing in this repository runs a session with an agent in it. So the promise is
kept by the only thing that could break it — a shell adopting `alo-telling`
cannot say the same thing twice unasked — and not yet by a machine anybody has
watched all afternoon. The other half is a surface, which is task 3's overlay
and does not exist.

### Or use an API instead

**Shown by:** `crates/alo-asking/tests/from_a_question_to_what_left.rs`,
`crates/alo-asking/tests/a_key_reaches_one_provider_only.rs`,
`crates/alo-models/src/source.rs`

**Still owed:** every provider exchange in the tests is against a socket this
repository stands up. No question has been put to a real provider's API from a
machine, with a real key, and the paired-machine place named in the same
sentence is a v0.5 line elsewhere in the definition.

### is a bound around it and never a choice inside it

**Shown by:** `crates/alo-choosing/tests/on_this_machine.rs`,
`crates/alo-choosing/tests/the_three_choices.rs`,
`crates/alo-agentd/tests/what_a_machine_says_about_itself.rs`,
`docs/autonomy/updates/an-administrator-set-that-rule.md`

**Still owed:** two files and two owners are real and no person has seen either.
The refusal naming the rule has no surface to appear on, and nothing writes the
machine description on a machine an administrator manages.

### Where the answer came from is said where the answer appears

**Shown by:** `crates/alo-asking/src/answer.rs`,
`crates/alo-asking/tests/a_day_that_only_looks_like_it_never_left.rs`,
`docs/autonomy/updates/an-operating-system-that-does-not-claim-what-it-cannot-know.md`

**Still owed:** *where the answer appears* is a screen, and there is none. The
provenance line is carried beside every answer in the code and has never been
put in front of a person.

### Never a silent fallback, in either direction

**Shown by:** `crates/alo-answering/tests/from_a_question_that_failed_to_what_left.rs`,
`crates/alo-answering/src/wrong.rs`, `crates/alo-answering/src/failed.rs`

### A model on this machine and a provider you added are both ordinary

**Shown by:** `crates/alo-choosing/tests/the_three_choices.rs`,
`crates/alo-models/src/source.rs`, `crates/alo-answering/src/failed.rs`

**Still owed:** the promise is about how the two are *presented* as much as how
they behave, and nothing presents anything: no setup screen, no settings panel,
no place where one could be shown as the good option and the other as the
degraded one.

### Model lifecycle: pull, list, serve, unload, remove

**Shown by:** `crates/alo-models/src/runtime.rs`,
`crates/alo-models/src/costing.rs`, `crates/alo-models/src/brought.rs`

**Still owed:** the five operations have never been run against a real model
service, so *disk accounted honestly* is arithmetic over figures nothing has
checked against a disk.

### every network egress an agent causes, visible at the moment it happens

**Shown by:** `crates/alo-egress/tests/what_this_crate_says.rs`,
`crates/alo-indicator/tests/the_egress_indicator_on_a_screen.rs`,
`crates/alo-asking/tests/from_a_question_to_what_left.rs`,
`docs/autonomy/updates/the-egress-indicator-on-a-screen.md`

**Still owed:** the indicator is decided, worded and kept in step with the
machine, and it has never been drawn. *Visible at the moment it happens* is a
claim about something a person can see, and nobody has seen it.

### No telemetry

**Shown by:** `crates/alo-egress/src/errand.rs`,
`crates/alo-egress/src/itself.rs`,
`crates/alo-asking/tests/a_day_that_never_left.rs`

**Still owed:** the policy is an enumerated list of why alo OS itself reaches
the network, and the promise's proof is a measurement at a network boundary on a
machine that has run a working day. No machine has.

### Boots on one certified machine, firmware to sign-in

**Still owed:** all of it, and it is scheduled rather than missing — task 12 of
`docs/autonomy/the-executable-plan.md`, which needs a machine nobody has plugged
in. *To sign-in* is owed twice over: there is nothing to sign in at until
`docs/decisions/0024-what-a-person-signs-in-at.md` is accepted and task 13 of
`docs/autonomy/the-executable-plan.md` is built.

**Read against the repository, 2026-09-11: not reachable, and it is the one of
the four that is honestly waiting rather than unexamined.** Both halves were
checked again. The firmware half is a machine: `crates/alo-image` holds what the
image owes the daemons and `image/Containerfile`'s own first paragraph says an
image that builds is not an image that boots. The sign-in half is a binary that
does not exist — `crates/alo-shell` has no `src/main.rs` and no `[[bin]]`, which
is the finding task 10 stopped on — and building one means choosing between the
options ADR 0024 sets out. So this promise waits on a machine **and** on a
decision, and no increment in between is available to this lane.

### Image built as an OCI container image

**Shown by:** `crates/alo-image/tests/what_the_image_owes_the_daemons.rs`,
`docs/autonomy/updates/what-a-person-signs-in-at.md`

**Still owed:** the image is built and booted in a VM, which is not a machine,
and it carries no shell to boot to — `crates/alo-shell` has no binary, which is
the finding task 10 stopped on.

### Compact, and minimised — two things, and the person picks

**Still owed:** all of it, and it is scheduled rather than missing — task 1 of
`docs/autonomy/putting-a-window-aside.md`, with tasks 2 to 8 behind it. **This
entry has no *Shown by* half, and that is what it is for.** The promise moved into
this release on 2026-09-30, when the owner asked for the minimised-windows panel
to be built now. Nothing has been written for it yet, so an entry claiming
evidence would be the failure this ledger exists to catch, one day old.

What is decided rather than built is in
`docs/design/the-windows-put-aside.md`: the panel is a viewport control and never
a child of the canvas, which `crates/alo-canvas` already enforces by refusing a
viewport surface the camera; each preview names one window rather than one
application; and seven of the owner's ten behaviour rules are decidable in a
crate with no display. Which vocabulary it speaks for the plane is settled in
`docs/design/one-plane-two-vocabularies.md`.

**Compacting is the other half of the same promise and is equally unbuilt.** A
later entry must not tick this one when only the panel exists: the line names two
things and says *the person picks*, so half of it answered is a promise owed.

### Full screen

**Shown by:** `crates/alo-shell/tests/a_window_that_fills_the_screen/mod.rs`,
`crates/alo-shell/tests/support/wm_capabilities.rs`

**Still owed:** **somebody seeing it on a display.** The mechanism landed on
2026-10-01 in `7c236e0` and the entry that stood here — *all of it*, `Mode` has
no case that fills the screen, no `fullscreen_request`, a client answered with
silence — was true until that morning and is kept above in this sentence rather
than deleted, because what a ledger entry said is how a reader judges what it
says now.

What is built and runs: `window_mode::Mode::FillingTheScreen`;
`fullscreen_request` and `unfullscreen_request` on the `XdgShellHandler`;
`Fullscreen` advertised in `wm_capabilities`; and the shell answering
`a_window_is_filling_the_screen()`, which `desktop_raster` asks before drawing
the Dock and the panel, so both give way whatever the person chose about the
Dock hiding. Four tests in
`crates/alo-shell/tests/a_window_that_fills_the_screen/mod.rs`, three of them
driving a real client through the protocol rather than the shell's own entry,
because the request is the thing that was missing: a client asking is answered;
`Fullscreen` is sent without `Maximized`; leaving returns to normal rather than
to maximised; and the shell knows, through the trusted entry.

**Why that is not a tick.** Every one of those runs under a nested compositor.
A nested session shows that a client talks to us and cannot show that a person
sees a window take a real display — the rule this plan holds itself to. The
owed half is the same owed half as tasks 38 and 39: a machine with a real
display.

### Where the Dock goes

**Still owed:** **all of it, and the code was taken out rather than never
written.** `alo-dock` has no edge at all: `crates/alo-dock/src/changes.rs` says
so in its own words — *there is no `edge` field and no `displays` field* — and
its tests pin the absence, one asserting that a `dock.toml` naming an edge loads
with the edge **ignored** rather than refused, so that a file written by an
earlier release still reads. ADR 0076's remedy removed the field on 2026-09-29;
the owner reversed the decision on 2026-09-30 and the field has not come back.

**So the honest state is a promise restored in the record and removed from the
code**, which is the reverse of the usual gap and worth naming as such: the
tests that would change when this is built are the two in `changes.rs` that
currently assert an edge is ignored. A reader checking whether this is built
will find tests passing, and they pass *because* it is not.

The default is not the question. Bottom remains the default and is what the
layout does today; what is owed is that a person may choose left, right or top,
and that the band works in both orientations rather than being a horizontal bar
turned sideways.

**Where the work is:** task 11 of `docs/autonomy/the-smallest-canvas-worth-showing.md`,
written on 2026-10-01 by the owner's direction and **Open, blocked on design**.
It carries the four edges, orientation-aware layout, labels, overflow, the
reveal paths and shared-edge collision, and it names the five states each of the
left, right and top designs needs. *This entry cited only the ADR until that
task existed, because a plan carried no task for the promise — which the owner
then directed into this plan.*

### Reaching the Dock over a full-screen window

**Still owed:** the whole interaction, because the situation it happens in
cannot be created — the entry above is why. The rules for it exist and are
tested: `alo-dock`'s revealing module holds *which regions keep a surface open*,
with the pointer, the keyboard, a drag and an open menu each holding it by
itself, and twelve tests including one that reads its own source to hold that
nothing in it consults a clock. **They are deliberately not cited above.**
Nothing calls them, on any machine, so they are reusable groundwork and not
evidence that a person can reach the Dock over a full-screen window — which is
the distinction this ledger exists to keep. Where the work is: task 6 of
`docs/autonomy/putting-a-window-aside.md`.

## What this audit found

**The audit in figures: 53 promises, 2 shown whole, 41 shown in part, 10 with no
evidence at all.**

*This line is the ledger's own count of itself and it is checked.*
`crates/alo-reconciling/tests/every_v0_01_promise_is_reconciled.rs` reads these
four numbers and compares each against what the reconciliation computes, so a
promise entering or leaving a release fails the audit until this sentence is
brought with it. **Whoever changes which promises are in v0.01 changes this
line in the same commit.**

*Written in digits on 2026-09-30 because the figure had nowhere to live. The
current count used to exist only as a literal in that test file, while the
paragraph below — the ledger's own account of itself — still said six, and the
correction under it said five. The test's failure message claimed to be quoting
this document and was quoting nothing: three appeared nowhere here. Two lanes
had to remember a number that lived in neither of the places a reader would
look, and getting it wrong failed the workspace's tests for all three.*

***And the first run of the new check found the ledger already wrong by one.***
*It was written as 41 promises and 36 shown in part, from the paragraph below;
the audit counts **42 and 37**. A promise had entered v0.01 and been reconciled
as shown in part with nothing recording it. The old literal could not have
caught it — it asserted only the number with no evidence, which had not moved,
so three of the four figures were unchecked and one of them had already
drifted.*

The paragraph below is the audit as first written, kept as history. Where its
numbers differ from the line above, the line above is the current one and the
notes that follow record what moved.

Forty-one promises. **Two are shown with nothing owed on them**, thirty-three
are shown in part with the rest named above, and **six have no evidence at
all**. The six are the finding, and they are not one kind of thing:

***And seven more arrived the same day, when the owner put the full canvas
experience into v0.01.*** *Three are shown in part — the closed canvas plan built
one Place and its tests are named above — and **four name nothing**: the World, a
frame arriving the shape its work is, a frame dragged between Places, and every
application living in a Place. Each names the task that is its increment in*
`docs/autonomy/the-canvas-and-its-places.md`*. That is 44 promises to 51 and five
with no evidence to nine, and this line moved with them because the figures above
are checked.*

- **All seven that name nothing**: *copy, cut and paste*, *the
  GPU works on first boot*, *it never nags*, and the four the canvas brought on
  2026-09-30 — *the World*, *a frame arrives the shape its work is*, *a frame
  dragged out of one Place and into another*, and *every application lives in a
  Place*. Each is a v0.01 promise with no crate, no test and no report — the same
  kind the roadmap's audit found six of, one at a time, over seven readings.
  **The difference between the first three and the last four is that the four
  name the task that is their increment** in
  `docs/autonomy/the-canvas-and-its-places.md`, and the three point nowhere.
- **One is a standing rule nothing checks**: *anything an agent verb can do, a
  person can do by hand*. It is the check on every verb anybody proposes, and no
  test walks the verbs asking it.
- **One cannot get a line without a decision**: *the agents point at the local
  model by default*. ADR 0016 refuses a default that nobody chose, and nothing
  here may narrow the promise to fit or contradict the ADR.
- **One is scheduled and needs hardware**: *boots on one certified machine*,
  which is task 12 of the delivery plan.

They are written down before anything else is done about them, which is this
task's acceptance. What follows belongs to whoever owns the scope: four of them
are work nobody has scheduled, one is a question only the owner can answer, and
one is waiting for a machine.

### One of the six closed, 2026-09-11

*Anything an agent verb can do, a person can do by hand* now has a line, so the
count above stands at **five with no evidence at all** and the standing rule is
no longer one of them. `docs/by-hand.md` answers for each of the ten verbs alo OS
declares and `crates/alo-by-hand` holds the document to them: a verb added with
nothing said about it fails the gate in the change that adds it. The entry above
carries what the check cannot reach, and it found one thing worth reading twice —
**six of the ten verbs ship at v0.01 and their plain way arrives at v0.5.** Task
14 of `docs/autonomy/the-executable-plan.md` and
`docs/autonomy/updates/every-verbs-by-hand-answer.md` are the work; the paragraphs
above are left as the audit wrote them, because a finding rewritten by whoever
closed it is a finding nobody can check.

### A second of the six closed, 2026-09-11

*And it never nags* now has a line, so the count stands at **four with no
evidence at all**, and it is the last of the six this lane could close without a
screen, a decision or a machine. `crates/alo-telling` is the memory nothing had:
the same unavailability told once is told once, and telling it again takes the
source changing, the reason changing, or the person asking again themselves.
Task 15 of `docs/autonomy/the-executable-plan.md` and
`docs/autonomy/updates/a-machine-that-cannot-reach-a-model-says-so-once.md` are
the work.

What is left of the six is what nobody here can close alone: *copy, cut and
paste* and *the GPU works on first boot* are unscheduled work, *the agents point
at the local model by default* waits on a decision ADR 0016 will not let this
lane make, and *boots on one certified machine* waits on a machine. The
paragraphs above are left as the audit wrote them, for the reason the first
closure gave: a finding rewritten by whoever closed it is a finding nobody can
check.

### The third is sent to a decision rather than closed, 2026-09-11

**The count stands at four**, deliberately. *The agents point at the local model
by default* now has
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md` behind
it — the options, a recommendation, and what each would cost `alo-choosing`,
`alo-image` and the setup flow — and **a proposed decision is not evidence that
anything was built**, which is what this crate refuses an ADR for in the first
place. The entry above says what is owed under it, including the fact the wording
had hidden from every reading until now: the image carries no model runtime and
no weights, so the promise is unbuilt in `image/` rather than blocked on a
settings key. Task 16 of `docs/autonomy/the-executable-plan.md` and
`docs/autonomy/updates/a-default-nobody-chose.md` are the work. The entry closes
when a machine arrives with a model on it, and not before.

### The four were read one at a time, and one of them was mis-sorted, 2026-09-11

**The count stays at four.** Nothing was closed here and nothing was ticked; what
changed is that each of the four now carries the reading behind its verdict,
under the promise it is about, so the next person inherits an argument rather
than a sorting. Task 19 of `docs/autonomy/the-executable-plan.md` is the work and
`docs/autonomy/updates/the-four-promises-with-no-evidence.md` is the report.

**One of the four was in the wrong pile.** *Copy, cut and paste* was sorted into
*needs a machine* by the audit that found it, in the same pass that found five
other things, and the sorting was carried unexamined ever since. A clipboard is a
protocol before it is a surface — an owner, the types it offers, and a transfer
somebody asks for — and every refusal in it is decidable with no screen, no
machine and no decision. It is now task 20 of
`docs/autonomy/the-executable-plan.md`, with its own acceptance. The other three
are genuinely waiting: *the GPU works on first boot* on a machine with a card and
on an image that carries something for it to accelerate, *the agents point at the
local model by default* on the owner accepting a proposed decision, and *boots on
one certified machine* on both a machine and a decision.

**And the ledger is now held to saying where the work is.** A promise with no
evidence at all must name the decision it waits on or the task that is the
increment — with the plan beside the number, because three plans in this
repository number their tasks from one — and the pointer is followed to a file on
the disk. `crates/alo-reconciling/src/waiting.rs` is the rule and
`a_promise_with_no_evidence_and_nowhere_to_go_is_refused` is it refusing. Being
owed is not the finding; being owed and pointing nowhere is, because that is the
entry whose reasoning is derived again from scratch every time somebody opens
this file — which is the seven-times-over reading this ledger exists to end,
arriving from the other end.

### The mis-sorted one is closed, and three are left, 2026-09-11

*Copy, cut and paste* now has a line, so the count stands at **three with no
evidence at all** — and it is the one the previous reading found in the wrong
pile rather than one anybody had scheduled. `crates/alo-clipboard` is the
selection this repository never had: an owner says which forms it can give,
somebody asks for one of them, and the broker holds the offer and the way back
to the owner and holds nothing else. Task 20 of
`docs/autonomy/the-executable-plan.md` and
`docs/autonomy/updates/the-clipboard-before-there-is-anything-to-draw.md` are the
work.

**What the increment does not close is written into the entry above**, so nobody
reads a crate as the promise: no two applications on a machine have moved
anything through it, because the `wl_data_device` wiring is `crates/alo-shell`'s
and is the desktop lane's.

The three that are left are the three the previous reading said were genuinely
waiting, and nothing about them has changed: *the GPU works on first boot* waits
on a machine with a card **and** on an image carrying something to accelerate,
*the agents point at the local model by default* waits on the owner accepting
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md`, and
*boots on one certified machine* waits on both a machine and
`docs/decisions/0024-what-a-person-signs-in-at.md`. **None of the three can be
closed by this lane**, which is a fact about scope rather than about effort: two
need hardware nobody here can plug in, and the third needs a decision only the
owner can make.
