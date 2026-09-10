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

**Still owed:** all of it. Nothing in this repository implements a clipboard —
no crate, no Wayland data-device handling in `alo-shell`, no test. This is a
promise with no line at all, and it is the seventh of the kind the roadmap's
audit kept finding one at a time.

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

### A model runs in one command

**Shown by:** `crates/alo-models/src/runtime.rs`,
`crates/alo-models/src/ollama.rs`, `crates/alo-models/src/weights.rs`

**Still owed:** the runtime adapter has never been run against a real model
service on any machine — the tests put its own protocol to a socket, which is
the shape of the exchange rather than the thing working.

### model by default — sovereignty is the default configuration

**Still owed:** this promise has no line, and it is the finding of this audit
that needs a decision rather than code. `alo-choosing` is deliberately unable to
produce a choice nobody made — a machine nobody has configured has no answer at
all — because ADR 0016 settled that a default is a choice made by whoever set
it. Either the promise means *setup offers the local model first*, which is a
surface nobody has built, or it means a default in the settings, which ADR 0016
refuses. Nothing in this repository may narrow the promise to fit, and nothing
may contradict the ADR: the owner decides which it is.

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

**Still owed:** this is a standing rule with nothing checking it. No test walks
the verbs alo OS ships and asks, for each, which surface does the same thing by
hand — and today the honest answer for most of them is *none, because there is
no surface at all*. Whether that check is possible before the shell exists is
itself the open question.

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

**Still owed:** *it says it once, where it happened* is a property of a surface
over time, and there is no surface — nothing has ever shown the sentence, once
or otherwise.

### And it never nags

**Still owed:** nothing implements or checks this. A promise about what a
machine does *not* do repeatedly needs something that could repeat, and no
surface exists yet — but it also means nothing will notice when one arrives and
starts asking. This is the third promise this audit found with no line at all.

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
the delivery plan, which needs a machine nobody has plugged in. *To sign-in* is
owed twice over: there is nothing to sign in at until ADR 0024 is accepted and
task 13 is built.

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
