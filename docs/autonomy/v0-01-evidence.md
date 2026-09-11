# v0.01 — every promise, against the evidence

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
`crates/alo-shell/tests/window_tiling/mod.rs`,
`crates/alo-shell/tests/window_close/mod.rs`,
`crates/alo-shell/tests/window_activation/mod.rs`

**Still owed:** the launcher. `alo_shortcuts::Action::Launcher` is a chord with
nothing behind it — no surface lists the installed applications and nothing
starts one from a person's own choice rather than from a verb.

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
`docs/autonomy/v0-01-delivery-plan.md`.** The audit that found it sorted it with
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
`crates/alo-shell/tests/window_tiling/mod.rs`,
`crates/alo-shell/tests/window_minimize/mod.rs`,
`crates/alo-shell/tests/window_maximize/mod.rs`,
`crates/alo-shell/tests/window_close/mod.rs`

**Still owed:** snap is a shortcut action dispatched to a layout and has no
pointer gesture behind it — dragging a window to an edge does nothing. And all
of it is measured in a nested session rather than on a machine.

### The dock, and the person decides where it goes

**Shown by:** `crates/alo-dock/src/layout.rs`, `crates/alo-dock/src/along.rs`,
`crates/alo-dock/tests/what_this_crate_says.rs`

**Still owed:** nothing draws a dock. The reflowing status area and the labels
giving way to icons are arithmetic that no compositor asks for yet, and the
Settings surface that would let a person choose the edge is not built.

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
`docs/autonomy/updates/native-folder-selection.md`,
`docs/autonomy/updates/a-grant-made-now-reaches-the-daemon-now.md`

**Still owed:** *see what is granted* has no surface. A grant can be made, kept
across a sign-out, reached by the daemon and expired, and there is nowhere a
person can look at the list or revoke one by hand.

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
`docs/autonomy/v0-01-delivery-plan.md`, which is scheduled and needs hardware
nobody has plugged in. **Nothing here is reachable by this lane**, and saying so
with the reading behind it is what this entry is for.

### A model runs in one command

**Shown by:** `crates/alo-models/src/runtime.rs`,
`crates/alo-models/src/ollama.rs`, `crates/alo-models/src/weights.rs`

**Still owed:** the runtime adapter has never been run against a real model
service on any machine — the tests put its own protocol to a socket, which is
the shape of the exchange rather than the thing working.

### The local model is what the machine arrives ready to run

**Still owed:** the expensive half, and the promise now says so instead of
disguising it. This is the entry that used to read *model by default —
sovereignty is the default configuration*;
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md` set out
the four things *by default* could mean, and on 2026-09-11 it was **accepted as
Option D** under the owner's standing delegation, with the definition reworded in
the same change. What the promise gave up is the claim that a value sits in a
settings file before a person has touched one — unbuildable here on purpose,
since ADR 0016 keeps that file for the person and ADR 0024 ships no accounts to
write it into. What it took on is heavier: **a model on the disk of every machine
we ship, sized for that machine** (ADR 0007).

So what is owed is exactly what was owed before, stated without the disguise.
`image/Containerfile` adds two binaries, two units, two directories and one
description to a pinned base and carries **no model runtime and no weights**, so
no machine this repository builds arrives ready to run anything; and there is no
setup flow, so none of the four configurations (ADR 0009's *not at all* among
them, first-listed local among them, nothing pre-selected) is offered to anybody.
The open question the ADR left — whether the weights ride on the certified image
or are fetched at setup — is a decision inside the work, not a blocker in front
of it.

**Reachable now.** What stood in front of this entry was a person's answer, and
the answer is on the record; what stands in front of it now is work — the pinned
runtime on the image, weights sized by `Catalogue::agent_for_cpu`'s honesty
rather than a publisher's claim, and a setup flow for the four choices. The
`nothing is chosen on their behalf` half is already held by
`crates/alo-choosing/tests/the_three_choices.rs` (`Settings::untouched` is a
person who has not chosen, and no constructor invents a choice); the
`arrives ready to run` half has no test until the image carries a model, and this
entry stays owed until one boots with it.

### Add your own provider in Settings

**Shown by:** `crates/alo-choosing/tests/the_three_choices.rs`,
`crates/alo-models/src/secret.rs`,
`crates/alo-secrets/tests/a_real_keyring_answers.rs`,
`crates/alo-asking/tests/a_key_reaches_one_provider_only.rs`

**Still owed:** the Settings surface. A provider is added by writing the
person's own settings file by hand, and the promise is about a place in the
shell where a name, an address and a key are typed.

### Setup's fourth choice

**Shown by:** `crates/alo-record/tests/a_machine_with_no_agent.rs`,
`crates/alo-capability/tests/what_this_crate_says.rs`

**Still owed:** there is no setup, so there is no fourth choice to make. What is
built is the machine that results from having made it — the agent's reach gone
at once, nothing further recorded — rather than the moment of choosing.

### A person never learns the name of anything we rented

**Shown by:** `crates/alo-saying/src/rented.rs`,
`crates/alo-saying/tests/what_this_machine_can_say.rs`

**Still owed:** only what a crate declares is held to this. A surface that
composed a sentence of its own, or a translator's line in another language,
passes through nothing — the check reads the English declarations and the
vocabulary they build.

### And it is enforced rather than remembered

**Shown by:** `crates/alo-saying/src/rented.rs`,
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
`docs/autonomy/v0-01-delivery-plan.md`, which needs a machine nobody has plugged
in. *To sign-in* is owed twice over: there is nothing to sign in at until
`docs/decisions/0024-what-a-person-signs-in-at.md` is accepted and task 13 of
`docs/autonomy/v0-01-delivery-plan.md` is built.

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

## What this audit found

Forty-one promises. **Two are shown with nothing owed on them**, thirty-three
are shown in part with the rest named above, and **six have no evidence at
all**. The six are the finding, and they are not one kind of thing:

- **Three have no line anywhere in this repository, and nobody has scheduled
  them**: *copy, cut and paste*, *the GPU works on first boot*, and *it never
  nags*. Each is a v0.01 promise with no crate, no test and no report — the same
  kind the roadmap's audit found six of, one at a time, over seven readings.
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
14 of `docs/autonomy/v0-01-delivery-plan.md` and
`docs/autonomy/updates/every-verbs-by-hand-answer.md` are the work; the paragraphs
above are left as the audit wrote them, because a finding rewritten by whoever
closed it is a finding nobody can check.

### A second of the six closed, 2026-09-11

*And it never nags* now has a line, so the count stands at **four with no
evidence at all**, and it is the last of the six this lane could close without a
screen, a decision or a machine. `crates/alo-telling` is the memory nothing had:
the same unavailability told once is told once, and telling it again takes the
source changing, the reason changing, or the person asking again themselves.
Task 15 of `docs/autonomy/v0-01-delivery-plan.md` and
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
settings key. Task 16 of `docs/autonomy/v0-01-delivery-plan.md` and
`docs/autonomy/updates/a-default-nobody-chose.md` are the work. The entry closes
when a machine arrives with a model on it, and not before.

### The four were read one at a time, and one of them was mis-sorted, 2026-09-11

**The count stays at four.** Nothing was closed here and nothing was ticked; what
changed is that each of the four now carries the reading behind its verdict,
under the promise it is about, so the next person inherits an argument rather
than a sorting. Task 19 of `docs/autonomy/v0-01-delivery-plan.md` is the work and
`docs/autonomy/updates/the-four-promises-with-no-evidence.md` is the report.

**One of the four was in the wrong pile.** *Copy, cut and paste* was sorted into
*needs a machine* by the audit that found it, in the same pass that found five
other things, and the sorting was carried unexamined ever since. A clipboard is a
protocol before it is a surface — an owner, the types it offers, and a transfer
somebody asks for — and every refusal in it is decidable with no screen, no
machine and no decision. It is now task 20 of
`docs/autonomy/v0-01-delivery-plan.md`, with its own acceptance. The other three
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
`docs/autonomy/v0-01-delivery-plan.md` and
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
