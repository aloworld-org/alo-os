# v0.5 — the models, measured, on a machine that can hold them

**Workstream:** the sentence `docs/features.md` promises at v0.01 and nothing
has yet been able to make true — *★ the catalogue says whether a model can
drive the verbs, not just whether it will run — measured by us, not claimed by
the publisher* — and the three v0.5 questions every report so far has ended
with: does the pinned runtime accept what alo OS sends it, does any file a
person brings clear the verb-driving bar, and what a person is told about a
model that cannot.
**Why it exists:** every catalogue entry is `not-measured` because no machine
this project has could load a 7B model (`docs/autonomy/updates/`, 2026-09-11).
A Mac with unified memory can. [ADR 0028](../decisions/0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
governs; `docs/autonomy/a-loop-on-a-mac.md` says how this lane runs.

**Crates this plan owns:** `alo-models`, `alo-driving`,
`alo-answering`, `alo-telling`, and `alo-asking`'s hosted and served doors —
lane B's original partition, idle since 2026-09-13. **Nothing in
`alo-nearby`, `alo-asking/src/corridor.rs`, `alo-record`, `alo-capability`,
`alo-turn`, `alo-egress`** (lane A's), nothing in `alo-finding` or
`alo-measuring` (lane B's), nothing Linux-only, and nothing in `crates/alo-shell`.

**What this plan may not do** (ADR 0028's terms): move any v0.01 box, line or
wording; tick anything *on the machine*; edit a crate another lane owns.
**Every measurement names the machine it ran on** — a grade with no machine
beside it is a claim. Before writing the next task, `git pull` and read the
plan as published — numbers are a shared space.

**Finished, 2026-09-15.** Nineteen tasks done; task 9 is not pursued, by the
owner's decision recorded in it. The catalogue now says of every model an 8 GB
machine can hold whether it drives the verbs **the way this machine asks**, two
entries clear that bar, and a machine with room is given one as its agent. The
lane stops here rather than choosing what to do next: the plans are assigned so
that five machines working one branch do not collide.

**Ownership clarification, 2026-09-18.** This plan already records its work
finished above. Its historical changes to `alo-choosing` do not reserve that
crate from the settings plan: the owner assigned that plan task 7 to this PC
lane B. The active claim is released to the settings plan. Task 9 remains
not pursued under the existing owner decision; no missing measurement is
marked done and no model run is authorized by this clarification.

## Tasks

### 1. One catalogue entry, graded on a machine that can hold it

**Status:** **Done, 2026-09-13.** **Depends on:** nothing.
**Report:** `docs/autonomy/updates/one-catalogue-entry-graded-on-a-machine-that-can-hold-it.md`.

The first real measurement. `alo-driving` has the ten exercises and the bar —
nine attempts in ten — and has never been run against a loaded model, because
none would load. This runs it once, for one entry, and writes the grade into
the catalogue with the machine named beside it.

- **Acceptance:** `alo-driving`'s `Exercises` are put to one catalogue entry
  through the pinned runtime on this machine, every attempt scored through the
  daemon's own door as the crate already does, and the entry's `drives_verbs`
  becomes `reliably`, `sometimes` or `rarely` — **never left `not-measured`
  after a completed run**, and never set by anything but a completed run; the
  catalogue entry carries the machine and date the grade was measured on, in a
  field a reader can see, and a test refuses a grade with no machine beside
  it; the ten attempts, verbatim, are in the report so the grade can be
  re-derived by a reader who disagrees with it; and a run that skipped an
  exercise is not a measurement, which the crate already refuses and the report
  restates.
- **Constraint:** the runtime is asked on `127.0.0.1` and nothing here adds a
  provider — the model runs on this machine, and the door is
  `Asking::to_a_service_on_this_machine`, which shows nothing on the indicator
  because nothing leaves. The exercises are not edited to suit the model: a bar
  that moves to meet the candidate is not a bar. Which entry is measured first
  is the one the catalogue recommends, so that the first fact is about the
  default.

### 2. Every entry graded, or refused with the reason

**Status:** **Done, 2026-09-13.** **Depends on:** 1.
**Report:** `docs/autonomy/updates/every-catalogue-entry-graded-or-refused-with-the-reason.md`.

Task 1 is one fact; the promise is the catalogue. This grades every entry the
catalogue ships, and for any it cannot, says why in the entry rather than
leaving `not-measured` to mean *probably fine*.

- **Acceptance:** every entry in the shipped catalogue is either graded with a
  machine and date beside it, or carries a one-line reason it could not be —
  *weights not published*, *larger than this machine's memory*, *the runtime
  refused the file* — in words `alo-saying` collects, because a person choosing
  a model reads that line; the catalogue's recommendation is re-derived from
  the grades rather than from memory alone, and if the entry it recommended
  before does not clear the bar the recommendation moves and the report says
  so; and `alo-choosing`'s offer of a model that does not drive the verbs is
  the sentence the crate already has, now shown for a **measured** *rarely*
  rather than an unmeasured one, checked by a test.
- **Constraint:** no entry is removed for failing. `docs/features.md` says a
  model that cannot drive the verbs is offered for what it is and never the
  agent; a catalogue that hid its failures would be a catalogue somebody could
  not trust for its successes. Nothing here changes what the bar is.

### 3. Does the pinned runtime accept what alo OS sends it

**Status:** **Done, 2026-09-13.** **Depends on:** nothing.
**Report:** `docs/autonomy/updates/the-pinned-runtime-and-what-alo-os-sends-it.md`.

Three reports end with the same sentence: *what no test here shows is that the
pinned runtime accepts this exact request* — `/api/create` with a one-line
Modelfile (`a-brought-file-is-one-the-runtime-answers-to.md`), the chat request
`openai.rs` puts, and the model listing `found_on_this_machine` reads. This
machine has Ollama 0.34.0; this task answers all three.

- **Acceptance:** each of the three requests is put to the pinned runtime's
  own version on this machine and what it answered is recorded verbatim — the
  bring-a-file road walked with a real `.gguf`, a question answered by the id
  the door taught, the listing read back; any difference between what the
  fixtures in `alo-models` assume and what the runtime actually said is
  **fixed in the fixture** with a test naming the runtime version, so the
  fixtures stop being a guess about a program nobody had run; and the runtime
  version is pinned in one place that a test reads, so a later Ollama is a
  deliberate change.
- **Constraint:** the runtime is not upgraded to make a request work; the
  request is fixed, or the report says the pinned runtime cannot do the thing
  and the promise's wording is re-read. Nothing here fetches from a publisher:
  the file brought is one already on this disk.

### 4. Does a file a person brings clear the bar

**Status:** **Done, 2026-09-14.** **Depends on:** 1, 3.
**Report:** `docs/autonomy/updates/a-file-a-person-brings-measured-against-the-bar.md`.

*Point alo OS at weights you already have and it runs them* is built. Whether
what a person brings can be **the agent** is task 10 of the old lane B plan,
blocked since 2026-09-11 on *a catalogue entry that clears the verb-driving
bar*. This measures a brought file the same way a catalogue entry is measured,
and says what the person is told either way.

- **Acceptance:** the exercises are put to weights brought by file rather than
  by catalogue id — the same ten, the same bar, the same door — and the grade
  is written into the person's own settings beside the file (never into the
  shipped catalogue, which is not theirs), with the machine and date; a brought
  file that grades *reliably* may be given the agent and one that does not is
  offered for what it is, checked by a test on `alo-choosing`; and the sentence
  a person reads for each outcome is in the vocabulary, with a test that none
  of them nudges towards the catalogue.
- **Constraint:** the grade is the person's machine's and travels nowhere —
  nothing here sends a measurement anywhere, and a test reads the crate's
  shipped source to say so. If no file on this machine clears the bar, that is
  the finding and the old task stays blocked with a truer reason written into
  it; the bar does not move.

### 5. What a person is told, measured against what a person can read

**Status:** **Done, 2026-09-14.** **Depends on:** 2, 4.
**Report:** `docs/autonomy/updates/what-a-person-is-told-in-the-order-they-meet-it.md`.

The measurement work ends in sentences, and the sentences are the product.
This task reads every one of them as a person would — in the order a person
meets them, on a machine where some models clear the bar and some do not —
and fixes what does not read.

- **Acceptance:** with the catalogue graded, a walk through `alo-choosing` and
  `alo-telling` from a fresh settings file produces the exact sequence of
  sentences a person sees for: the recommended model, a model that runs but
  *sometimes* drives, one that *rarely* does, one too large for the machine,
  and a file they brought — recorded verbatim in the report as a table, and
  held by one test that fails if any sentence in the sequence changes without
  the table changing; every sentence in the table is in the vocabulary and
  carries a translator's note; and no sentence in the table claims a
  measurement that was not made on this machine.
- **Constraint:** nothing here re-grades anything. If a sentence is true and
  reads badly, the sentence changes; if a sentence reads well and is not true,
  the sentence changes the other way. Nothing on this list moves an *On the
  machine* box, because a Mac is not the machine.

### 6. A grade that lands one short of a line gets a second round

**Status:** **Done, 2026-09-14.** **Depends on:** 1.
**Report:** `docs/autonomy/updates/a-grade-near-a-line-gets-a-second-round.md`.

Written from task 4's finding: the runtime samples every answer (temperature 0.8,
the default, because alo OS asks the way a turn asks), and the same Qwen 2.5 7B
bytes drove 4 of 10 by catalogue name and 3 of 10 by file. Four is one short of
`sometimes`. A grade that close to a line is a grade from ten samples, and the
catalogue should not say `rarely` or `sometimes` about it on the strength of one.

- **Acceptance:** `qwen2.5-7b-instruct` is put to a second round of the fixed ten
  on the same machine through the same door (`ALO_DRIVING_ROUNDS`), and the
  catalogue's grade is the one twenty attempts earn, with the machine and date
  beside it; the report carries all twenty verbatim; and the catalogue's rule
  for when a second round is owed — a first round landing within one of five or
  nine — is written into `data/catalogue.toml` and held by a test that refuses a
  one-round grade in that band.
- **Constraint:** the bar, the prompt, the scoring, the wait and the sampling are
  unchanged. A second round is a bigger sample, not a different method.

### 7. Whether a turn asks a local model for the shape it must answer in

**Status:** **Done, 2026-09-14.** **Depends on:** 2.
**Report:** `docs/autonomy/updates/a-local-model-held-to-the-envelope.md`.

Every 7B-class model measured so far failed the call's **grammar** rather than
its reasoning: reads went through the right door, and changes named their verb
where the door belongs, or wrote the arguments as a plain object. The pinned
runtime can hold a model's answer to a JSON schema (`/api/chat`'s `format`), and
the protocol already has the schema a call must match. Asking for it changes how
**every** turn asks a local model, not how the measurement scores one — which is
exactly why it is a decision before it is a change.

- **Acceptance:** an ADR in `docs/decisions/` decides whether a turn asks the
  pinned runtime for the protocol's shape, weighing what it costs (a model that
  can only emit the envelope cannot decline in prose; a hosted provider may not
  offer the same), and what it would mean for the catalogue's grades, which were
  earned without it; if the ADR accepts, `alo-models`' chat request carries the
  schema, the measurement is run again against every entry graded on the Mac
  **as a new measurement with the method named beside it**, and both sets of
  grades are reported side by side; if it rejects, the report says what was
  measured to decide it.
- **Constraint:** the exercises and the bar do not move. A grade earned with a
  schema is never written over one earned without, and the catalogue says which
  method a grade was earned by.

### 8. Teuken's chat template, decided before Teuken is graded

**Status:** **Done, 2026-09-14.** **Depends on:** 3.
**Report:** `docs/autonomy/updates/teukens-chat-template-and-the-fetch-that-could-not-fetch.md`.

`docs/quirks.md` has it: the GGUF the catalogue names for `teuken-7b-instruct`
carries no chat template, the runtime warns and answers with the end-of-turn
token in the text, and a grade made that way would measure the missing template.
The constitution lets alo OS configure an engine; choosing a template for a model
whose publisher wrote one elsewhere is a decision with a name on it.

- **Acceptance:** the template openGPT-X publishes for the commercial release is
  found and cited, and either the catalogue entry carries it (applied as a
  Modelfile `TEMPLATE` when the model is fetched, with a test that a fetch of
  that entry sends it and no other entry's does) or the entry says why it does
  not; the reason Teuken carries today stays until a machine with room grades it.
- **Constraint:** nothing is invented: a template not published by the model's
  publisher is not used.

### 9. The four entries the measuring machine could not hold

**Status:** blocked — **not pursued, by the owner's decision of 2026-09-15**:
*testing is done on the small model; once a small model works, the larger ones
are not run.* Mixtral is not attempted on any machine, and neither is raising
the GPU's share for the nine-billion entries. The entries keep the reasons
below. **Depends on:** 2, 8 for Teuken.
**Report:** `docs/autonomy/updates/the-nine-billion-entries-tried-on-this-machine.md`.

**Two thirds of it is answered, on 2026-09-15, and the answer was free.** The
development VM's 4 GiB was the memory: with it stopped, this machine **loads**
both nine-billion entries. Neither can answer with them. EuroLLM sits at 6.49 GB
with 1.9 GB off the graphics processor and answers a nineteen-token question in
168 seconds, but not one of `alo-driving`'s own prompts — which carry every verb
the machine has — inside the 300 seconds `alo-models` waits, in three runs.
Gemma sits at 7.45 GB with 3.3 GB off and does not answer the nineteen-token
question inside 300 seconds either. So both keep
`too-large-for-the-measuring-machine`, whose own words are *does not have the
memory to run the model inside the time `alo-models` waits* — now from an
attempt rather than from subtracting this machine's memory from `min_ram_gb`.

What is left is `mixtral-8x7b-instruct`: 26.4 GB of weights, three times this
whole machine, and no setting reaches it. It was **not** attempted, deliberately
— 26 GB of somebody's connection to watch arithmetic happen — and that is stated
in its entry rather than dressed up as a measurement. Raising the GPU's share
(`sudo sysctl iogpu.wired_limit_mb=6144`, the owner's password) would buy about a
gigabyte of residency and is worth trying for the two nine-billion entries; it
cannot touch Mixtral.

- **Acceptance:** each is graded with the machine beside it, or keeps its reason
  with a machine that actually tried named in it.
- **Constraint:** nothing is loosened to fit — not the context window, the wait
  or the quantisation.

### 10. Every entry this machine can hold, asked in the envelope

**Status:** **Done, 2026-09-14.** **Depends on:** 7.
**Report:** `docs/autonomy/updates/every-entry-this-machine-holds-asked-in-the-envelope.md`.

ADR 0032 took Qwen 2.5 7B from 40% to 87.5% by holding it to the protocol's
envelope. Four entries have an envelope grade; the six small entries graded on
the development PC in 2026-09 were never asked that way, and two of them —
`qwen3-1.7b` and `granite-3.2-2b-instruct` — were chosen because their publishers
train them for tool calls and constrained output.

- **Acceptance:** every catalogue entry this machine can hold carries a
  `drives_verbs_in_the_envelope` grade with its machine, date, runtime and
  counts, each earned by two rounds through the harness and each fetched through
  `Ollama::fetch` rather than by hand; the report carries every answer verbatim
  and says which, if any, clears the bar.
- **Constraint:** the free grades those entries carry are not touched — they were
  earned on another machine asking another way, and ADR 0032 keeps the two apart.

### 11. Candidates published for tool calls, at a size this machine holds

**Status:** **Done, 2026-09-14.** **Depends on:** 10.
**Report:** `docs/autonomy/updates/qwen3-at-the-sizes-this-machine-holds.md`.

If no catalogued entry clears the bar in the envelope, the next question is
whether a model this catalogue does not list does. The candidates are the ones
their publishers train for tool calls at a size 8 GB holds — the Qwen3 family at
4B and 8B first.

- **Acceptance:** each candidate's licence is read against its publisher's own
  repository before anything is fetched (the catalogue's rule 1); each is added
  as an entry only if its licence permits commercial use or states its
  conditions; each added entry is graded both ways with its machine beside it;
  and an entry that clears the bar in the envelope is reported as the first local
  model that could be given the agent once lane A's turn asks that way.
- **Constraint:** nothing is added for its grade; an entry is added for its
  licence and training, then measured, and kept whatever it earns.

### 12. The five that failed, read one by one

**Status:** **Done, 2026-09-14.** **Depends on:** 10.
**Report:** `docs/autonomy/updates/the-five-that-failed-read-one-by-one.md`.

Qwen 2.5 7B drove the verbs 35 times in 40 in the envelope. The bar is 36.
Before any more models are fetched, the five attempts that failed are the
cheapest measurement on this machine and the most likely to say what the gap
*is* — a shape the model cannot hold, one exercise it always misses, or an
exercise asking for something the protocol never needs.

- **Acceptance:** each failed attempt is read against the exercise it answered
  and classified as exactly one of: a valid call to the wrong verb, the right
  verb with an argument the protocol refuses, or output that is not a call at
  all — with the attempt verbatim beside the classification; the five are
  tabulated by exercise, so a single exercise that accounts for most of the
  gap is visible; if any exercise's wording asks for a thing `alo-capability`
  never validates, that is written down as a finding about the exercise and
  the exercise is **not** changed here — the bar does not move to meet the
  candidate (task 1's rule); and the report ends with one sentence on whether
  the gap looks like the model's or the harness's, and the measurement that
  would settle it.
- **Constraint:** nothing is re-graded and no grade changes. This reads what
  was already measured; a second run belongs to task 6's second-round rule
  and is not this.

### 13. The same weights at a higher quantisation

**Status:** **Done, 2026-09-14.** **Depends on:** 12.
**Report:** `docs/autonomy/updates/qwen-2-5-7b-at-five-bits.md`.

The catalogue grades `qwen2.5-7b-instruct` at `Q4_K_M`. A model five in forty
short of the bar at four bits may clear it at five or six, and the image's
pinned weights are chosen by this catalogue — so which quantisation the entry
names is a product decision, and today it rests on nothing measured.

- **Acceptance:** the same weights at `Q5_K_M` and, if 8 GB holds it with the
  VM stopped, `Q6_K` are graded in the envelope with the same ten exercises
  and the same second-round rule, each with its machine, runtime, digest and
  counts beside it; the loaded size and whether the weights stayed on the GPU
  are recorded per quantisation, because a grade earned against swap is a
  different measurement (`docs/quirks.md`, *what 8 GB holds*); the catalogue
  entry carries one grade per quantisation rather than one grade for the
  model, and `quantised_at` is what a reader uses to tell them apart; and if a
  quantisation clears the bar, the report says so as the first local model
  that could be given the agent, and names what the image would have to pin
  for that to be true on a shipped machine.
- **Constraint:** the exercises, the bar and the door are unchanged. A
  quantisation that does not fit this machine is refused with the reason
  task 2 established, never graded against swap and reported as a grade.

### 14. The door that asks in the envelope

**Status:** **Done, 2026-09-14.** **Depends on:** 7.
**Report:** `docs/autonomy/updates/the-door-that-asks-in-the-envelope.md`.

ADR 0032 decided that an agent turn asks a model on this machine for the
envelope and the door, never the call — and its point 5 says the catalogue
keeps reading the free grade *until the agent turn asks that way*. Today only
`alo-driving`'s harness asks that way. This makes the ask a door in
`alo-asking`, so that a turn can take it.

- **Acceptance:** `Asking::to_this_machine` can be given the envelope's
  schema — the protocol version and exactly one of `read`, `propose`, `ask` —
  and puts it to the pinned runtime the way ADR 0032 measured, with a test
  that reads the request off a socket and finds the schema and nothing about
  the call's inside; a question a person puts to a model is never given the
  schema, held by a test; the hosted and served doors are untouched and a
  test says so; and `alo-driving`'s harness uses this door rather than its
  own request, so the measurement and the product ask in one way.
- **Constraint:** `alo-turn` and `alo-agentd` are lane A's. Wiring the real
  turn through this door is written as a task on
  `v0-5-the-local-network-plan.md` for lane A, and this task ends at the door.
  Nothing here changes which grade the recommendation reads: that is task 15,
  after the turn asks this way.

### 15. The recommendation reads the grade for the way turns ask

**Status:** **Done, 2026-09-14.** **Depends on:** 14.

Lane A landed the wiring this waited on: `e99be94`, *an agent's next request
is asked of the pinned runtime in the envelope* — `alo-agentd`'s `doing`,
`questioned` and `questions` now put a turn's request through the door task 14
built. So the condition ADR 0032 §5 names is met: **the agent turn asks that
way**, and the recommendation may read the grade earned that way.

ADR 0032 point 5, second half. Once the agent turn asks in the envelope, the
grade that says whether a model may be given the agent is the enveloped one,
and the free grade is what a person is shown as history.

- **Acceptance, when unblocked:** `Catalogue`'s recommendation reads the
  enveloped grade for an entry that has one and the free grade only where
  none was measured, with a test for each; `can_be_the_agent` follows the
  same rule; `alo-choosing`'s offer names which way the grade was earned; and
  the report says whether any local model is now given the agent, with the
  machine that measured it.
- **Constraint:** no grade is rewritten and both stay in the catalogue side by
  side. If lane A's wiring has not landed, this task stays blocked rather than
  reading the enveloped grade for a turn that still asks freely.

### 16. The instructions' one example, and what a second one costs

**Status:** **Done, 2026-09-14.** **Depends on:** 12.

Task 12 read the five attempts Qwen 2.5 7B failed and found that three were
a correct request through the wrong door — a change asked for as a *read* —
and that the only example the instructions give a model uses the read door.
That is a question about the instructions, and task 12 rightly changed
nothing. This task decides it, in the open, the way the bar itself was
decided: a bar that moves to meet a candidate is not a bar, but an
instruction that shows one door and is then surprised when a model takes it
is not a bar either.

- **Acceptance:** an ADR sets out the options — the instructions as they are;
  one example per door; an example that names the door it does *not* use —
  with what each would measure and what each would hide, and recommends one;
  the recommended instructions are then put to the same ten exercises on the
  same weights, in the envelope, as a **new** grade beside the old and never
  over it, with the machine, runtime, counts and the instructions' own hash
  recorded so a reader can tell which instructions earned which grade; the
  five failed attempts from task 12 are re-read against the new instructions
  and the report says, one by one, which changed and which did not; and if
  the change clears the bar for any model, the report says so as the first
  local model that could be given the agent, and says in one sentence what
  it took.
- **Constraint:** the exercises, the verbs they name, the scoring and the bar
  are untouched; only the instructions a model is shown may change, and only
  as the ADR decides. A grade earned under the old instructions is not
  rewritten — `alo-driving` records which instructions a grade was earned
  under, so the two can never be confused.

### 17. The same ten exercises through the engine's own server, held to the whole call

**Status:** **Done, 2026-09-14.** **Depends on:** 12, 16.

[ADR 0035](../decisions/0035-the-wrapper-or-the-engine.md) is *proposed*, and
this task is what accepts or rejects it. Every friction the measuring lane
found was in the wrapper, and one of them is that it cannot hold a model to
the whole call in the protocol's key order. The engine's own server can, with
a grammar.

**Task 16 changed what this is for, and the ADR says so.** The bar was cleared
under the wrapper — 80 in 80 — by showing the instructions' every door, so
this is no longer *the thing standing between the product and a local agent*.
It is now a question of margin, of the four frictions that live in the image,
and of what the next model needs. Measure it anyway; accept it only if the
engine's server is **better** rather than equal.

- **Acceptance:** `llama.cpp`'s `llama-server`, at one pinned version and
  digest recorded in the report, serves the same weights the wrapper served
  (the same file, the same digest — never re-fetched from a publisher if the
  bytes are already on this disk); the ten exercises are put to it through a
  door in `alo-asking` that speaks the server's OpenAI-shaped API — the
  hosted door already does, on loopback, and nothing new is opened — with a
  **GBNF grammar for the entire call** derived from `alo-protocol`'s own
  shape and checked by a test against every call `alo-capability` accepts;
  every run is under ADR 0034's `OneExamplePerDoor` instructions, named by
  their digest like every other grade, so the comparison has one variable;
  the grade is written beside the wrapper's, never over it, with machine,
  runtime, digest, counts, residency and the grammar's hash; and the report's
  first paragraph answers the three questions ADR 0035 accepts on — did a
  model clear the bar here that cannot through the wrapper, is the margin on
  the one that already does materially wider, and was the whole call held —
  then the ADR's status is changed to accepted or rejected in the same commit,
  with the numbers.
- **Constraint:** the exercises, the bar, the second-round rule and the door's
  scoring are untouched. Nothing here changes the pinned runtime, the image
  or `ollama.rs`: this measures a candidate; the change, if the number earns
  it, is a task of its own after the ADR is accepted. No source patch to
  either program. If the engine's server cannot be pinned or its grammar
  cannot express the call, that is the finding, recorded in `docs/quirks.md`,
  and the ADR is rejected on it.

### 18. The words a turn shows a model are the product's own

**Status:** **Done, 2026-09-14.** **Depends on:** 16.
**Report:** `docs/autonomy/updates/the-words-a-turn-shows-a-model.md`.

Every grade this lane has made is a measurement of words that only the
measurement has. `alo_turn::Turning::asking_for_the_next_request` takes what a
model is shown as a string from whoever calls the daemon's door — its rustdoc
says *what the agent composed for the model — its instructions, the verbs and
what the person said* — and **no crate in this repository composes one**. ADR
0034's second cost names the consequence and leaves the remedy to the turn; read
against the code there was no prompt to fix, and task 16 had just measured that
the words are worth 80 of 80 against 71 of 80 on the same weights.

- **Acceptance:** the words a model is shown — the instructions, every verb the
  registry declares in the verb's own sentence, the request last — are one
  function in a crate of their own that carries the verb registry and SHA-256
  and nothing else, held to that by a test reading its own manifest and its own
  source rather than by a sentence in a header; `alo-driving` composes its
  prompt by calling it, with a test that an exercise is asked in exactly those
  words, and every digest a catalogue grade names is unchanged — the first
  instructions' pinned SHA-256 travels with the text and its test still passes;
  an ADR decides which set a turn shows, on the measurements rather than on
  taste, and records that **no grade moves until a turn composes from the
  crate**, so `grade_for_the_turn` is untouched here; and the wiring is written
  as a task on `v0-5-the-local-network-plan.md` for the lane that owns the turn,
  naming the function.
- **Constraint:** the exercises, the bar, the scoring and every grade are
  unchanged — this moves text without editing a byte of it, which is what the
  pinned digest proves. `alo-turn`, `alo-agentd` and `alo-capability` are not
  edited. The words stay English and that limit, which was a fact about a
  measurement, becomes a fact about the product the day a turn uses them:
  `docs/quirks.md` records it before it is true rather than after.

### 19. The catalogue reads the grade for the words a turn shows

**Status:** **Done, 2026-09-15.** **Depends on:** 18, 20.
**Report:** `docs/autonomy/updates/the-catalogue-reads-the-grade-a-turn-earns.md`.
Lane A landed the wiring this waited on — `Turning::asking_for_the_next_request`
builds what it shows a model with `alo_instructing::shown_to_a_turn` — so the
condition ADR 0037 decision 3 names is met. **Two entries are now given the
agent**: `qwen2.5-7b-instruct` (80 of 80) and `qwen3-8b` (20 of 20), and a
16 GB machine is given the second. No entry lost a grade it appeared to have,
because task 20 had measured every entry this machine can hold the same way
first.

ADR 0037 decision 4. Once a turn is shown the product's own words, the grade
that says whether a model may be given the agent is the one earned under those
words, and a grade earned under any other set is history — the same rule ADR
0032 decision 5 set for the envelope, one variable on.

- **Acceptance, when unblocked:** `Model::grade_for_the_turn` reads the grade
  earned under `Instructions::SHOWN_TO_A_TURN`'s digest, with a test for an
  entry that has one and a test for an entry that does not; an entry never
  measured under it **has no grade for the turn** and `can_be_the_agent` is
  false for it, rather than a grade under other instructions being read for it;
  `alo-choosing`'s offer names which words the grade was earned under; and the
  report says which entries a person may now be offered as the agent and which
  lost a grade they appeared to have.
- **Constraint:** no grade is rewritten and every grade stays in the entry. If
  the wiring has not landed, this stays blocked rather than reading a grade for
  a turn shown somebody else's words.

### 20. Every entry this machine can hold, graded in the words a turn shows

**Status:** **Done, 2026-09-14.** **Depends on:** 18.
**Report:** `docs/autonomy/updates/every-entry-graded-in-the-words-a-turn-shows.md`.
Twelve entries re-measured: 98 of 240 against 23 of 240, none worse, five bands
moved, and `qwen3-8b` at **20 of 20** is the second entry to clear the bar.

Task 10 graded the catalogue in the envelope under the first instructions; task
16 graded one entry under the instructions ADR 0037 now names as the turn's.
Between them the catalogue has one entry — `qwen2.5-7b-instruct`, and its
five-bit file — measured the way a shipped machine will ask. Task 19 turns that
into the grade that decides, and until every entry this machine can hold has one,
doing so would take a grade away from twelve entries rather than move it.

- **Acceptance:** every catalogue entry this machine can hold carries a grade
  under `Instructions::SHOWN_TO_A_TURN`, in the envelope, through
  `alo-asking`'s door, with its machine, date, runtime, digest, counts and
  residency beside it and the second-round rule applied as every other grade
  was; each is fetched through `Ollama::fetch` rather than by hand; the report
  carries every answer verbatim and says, entry by entry, what the words moved;
  and an entry too large for this machine keeps the reason task 2 gave it, with
  the machine that tried named in it.
- **Constraint:** the exercises, the bar, the door and the scoring are
  unchanged, and no existing grade is touched — the new grades sit beside them
  under their own digest (ADR 0034, decision 3). Nothing is fetched that the
  catalogue does not already list, and an entry is not removed for what it
  earns.

### 21. A hosted provider answers, and the person is told who and where

**Status:** **Done, 2026-09-21** — and **most of it was already built**, which
is the finding rather than a footnote. **Depends on:** nothing — ADR 0008 is
accepted and the roads exist. Asked for by the owner on 2026-09-21.

Report:
[What a person reads while the question leaves](updates/what-a-person-reads-while-the-question-leaves.md).

**The premise below is stale, and by eighteen days.** *Nothing speaks to one*
was not true when this task was written: `832ac90`, **2026-09-03**, added
`alo-asking`'s `openai.rs` and `hosted.rs` and an end-to-end test against a real
HTTP service the test starts, and `crates/alo-turn/src/asking.rs:236` has called
`to_a_provider` since. Building the speaking part again would have been a second
answer to *how does a question reach a provider*, which is the thing this
repository refuses.

**What was measured instead**, clause by clause, before anything was written:
a question a person chooses to send to a hosted endpoint is answered through it
and comes back by the same road as a local one (`from_a_question_to_what_left.rs`);
the road out is `alo_proxy::the_way` for `Road::AskingAProvider`, with
`Hosted::taking` documenting why a road decided by `HTTP_PROXY` in a process's
environment is a road nobody chose; a provider that refuses on the wire is
`RefusedThere` and anything else is another of `WentWrong`'s refusals in the
person's words, each carrying **offers a person approves** rather than a retry,
which is *never a silent fallback* held structurally; the credential is held to
one endpoint (`a_key_reaches_one_provider_only.rs`) and kept from an agent
(`nothing_here_keeps_the_key.rs`).

**The one thing genuinely missing was the sentence itself.** Every test on that
road asserts the indicator is **quiet once the answer is home** — the second half
of law 1 — and none read the line **while the question was still out**, which is
where ADR 0008's *who and where* lives. `what_a_person_reads_while_the_question_leaves.rs`
reads it against a real service on a real socket, in the two states that decision
distinguishes:

| The provider | What a person reads |
|---|---|
| says where it runs | `@mail is asking a question of Mistral, in the EU` |
| will not say | `@mail is asking a question of Aurora, which has not said where it runs` |

and asserts of the second what it must **not** say — not `127.0.0.1`, which is
where that service really was, and not any of the words that would place it
nearby.

**ADR 0008's third place, which has never been built.** That decision names
three places inference can happen, and the repository has two of them: this
machine, and a machine on this network. The third —

> **A hosted API.** alo's EU service, the customer's own hosted endpoint, or a
> third-party provider. The question leaves the building. The indicator fires,
> and says **who** and **where**: a provider that will not say where it runs is
> reported as unknown rather than assumed to be nearby.

— is decided, scaffolded and unreachable. `alo-answering` already has
`Elsewhere`, `Offer` and `RefusedThere`; `alo-proxy` has
`Road::AskingAProvider`, *the one road on this list that an agent can cause*;
`alo-egress` has *a provider* and *a provider somewhere* as destinations, with
their words. **Nothing speaks to one.** `alo-driving` and `alo-answering` reach
no provider today, which is why a machine that cannot answer locally simply
cannot answer.

- **Acceptance:** a person's machine can be given a hosted endpoint — alo's own
  EU service, an endpoint the organisation runs, or a third party — and a
  question they choose to send there is answered through it, with the answer
  arriving by the same road as a local one so nothing downstream can tell which
  kind it was except by asking; **the indicator says who answered and where they
  run**, from what the provider itself states, and a provider that will not say
  is shown as *unknown* rather than assumed to be nearby (ADR 0008); the road
  out is `alo_proxy::the_way` for `Road::AskingAProvider`, so a company proxy
  carries it like everything else; a provider that refuses, times out or answers
  with nothing is `alo_answering::RefusedThere` in the person's words, never a
  silent retry somewhere else; the endpoint and its credential are the
  organisation's to set (ADR 0004, ADR 0016) and the credential never reaches an
  agent (ADR 0049's shape); and one end-to-end test against a real HTTP service
  the test starts, with the whole exchange in the report.
- **Constraint:** **never a silent fallback**, which is ADR 0008's own rule and
  the thing this task is most likely to break. A machine that cannot answer on
  its own hardware says so; it does not quietly send the question somewhere
  else. Where a question is answered is the person's choice and they always
  know, which means a default that sends a first question to a provider without
  having been chosen is out of scope of this task and of that decision. Nothing
  here changes what `docs/features.md` promises about local inference: expected
  inference egress over a working day is still **zero** on a machine nobody
  pointed at a provider.

### 22. The graphics card, when there is one, and the machine when there is not

**Status:** **Done, 2026-09-22**, on the third PC — the processor road
measured, every refusal on the way to a card tested, and the card road
`- [x] The code.` with the machine it still needs named in the report.
**Taken by the third PC at the owner's instruction**, moved across from the Mac
because this machine is x86_64 with **no discrete graphics**, which is the
condition this acceptance was written for and the condition the owner's laptop
is in. Measured here before starting, and again by the test that ships: no VGA
or 3D device on the bus, no `/dev/dri`, no `nvidia-smi`. **Depends on:**
nothing. Asked for by the owner on 2026-09-21.

Report:
[Which part of this machine runs the model](updates/which-part-of-this-machine-runs-the-model.md).

**What it left for task 23, said here so nobody reads this as the whole line.**
A card *being used* is shown nowhere: no machine in this fleet has one, and no
emulated device stands in for one (ADR 0056's reason, unchanged). The code for
it is written and runs from a machine that has one — `alo_models::Road::of`
reads the runtime's own residency, `Ollama::answers_in_the_envelope_on` puts
one question on a named road, and `alo_driving::BothRoads` holds the two runs
to each other and **refuses** a pair that is not one of each, which is what
stops a machine with no card reporting agreement with itself.

**Split, 2026-09-22, the way [ADR 0056](../decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
split the chip's half.** This task is the road a machine without a card takes,
and **task 23** below is the road a machine with one takes. Neither ticks
`docs/features.md`'s line alone. A machine with no card cannot show that a card
is used, and saying so is the work rather than a gap in it.

**The distinction the owner named, and it is not a detail.** A machine that has
an NVIDIA card and no NVIDIA driver must read as **no driver**, never as *no
card*. The base is rented and unmodified (ADR 0011) and **does not ship that
driver**, so this is the likely state of every NVIDIA machine alo OS meets
today — not an edge case. A person told *no card* would go looking for hardware
they already own; a person told *no usable driver for the card you have* knows
what is true. The two readings send them to different shops.

**ADR 0007 settled the principle three weeks ago and nothing implements it.**
*The CPU is the default; a GPU is acceleration* — *a GPU changes speed, not
capability*. `alo-choosing` already carries `on_the_gpu_bytes`, and it is
`None` at every site in the repository: nothing ever measures a card, so nothing
can use one. A machine with a GPU today runs exactly as slowly as a machine
without.

The owner's laptop has no discrete graphics, so **this task's acceptance is
written to pass on a machine with none** — the card's half is the part that
waits for hardware, the way the chip's half of encryption does.

- **Acceptance:** the machine says whether it has a graphics card the runtime
  can use and how much memory it has, measured rather than assumed, and
  `on_the_gpu_bytes` carries it; a machine with a usable card runs the pinned
  model on it and a machine without runs the same model on the processor, with
  **the same answers from both** — a GPU changes speed and not capability, so a
  test asks one question of each road and holds the results to each other; which
  road was taken is something a person can ask for and is said in the
  vocabulary, never inferred from how long it took; a card the runtime cannot
  use — the wrong vendor, no driver, too little memory for the pinned entry — is
  **a machine that runs on its processor and says why**, not a failure and not a
  silent slowdown; and the catalogue's `min_vram_gb`, which ADR 0007 says stopped
  being the judge, does not come back as one — an entry this machine can hold on
  its processor stays offerable whatever card is or is not there.
- **Constraint:** the base is rented and the runtime is pinned (ADR 0011,
  ADR 0006) — no driver of ours, no second runtime for the card, and nothing
  here changes which entries the catalogue offers. **The card's half is not
  ticked from a machine without one.** On a machine with no discrete graphics
  the processor road is measured and the card road is `- [x] The code.` with the
  machine that is still needed named, exactly as ADR 0056 does for the chip.
  ADR 0007's rejected framing — the GPU workstation first, `min_vram_gb` as the
  judge, *the GPU changes what is possible* — is not reintroduced here.

### 23. The card road, on a machine that has one

**Status:** blocked — **on hardware only.** Task 22 is done (2026-09-22) and
this task's dependency on it is cleared: what remains is a machine with a
discrete graphics card the pinned runtime can use, and a driver for it.
`docs/hardware.md` lists none, and the machine this plan's lane runs on has no
VGA or 3D device at all. Split from task 22 on 2026-09-22, the way
[ADR 0056](../decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
split the chip's half from the disk's. **Depends on:** 22, which is finished.

**The instrument is written and waiting.** Task 22 left
`crates/alo-driving/tests/the_same_question_on_each_road.rs`, whose
`one_question_on_each_road_of_a_machine_that_has_both` is `#[ignore]`d and runs
with `ALO_DRIVING_MODEL` set on a machine that has a card: it puts the fixed
set on each road through `Ollama::answers_in_the_envelope_on`, reads the road
each run really took from the runtime's own residency rather than from what was
asked for, and holds the two grades to each other. It refuses before it starts
on a machine with no usable card, naming what is missing. What this task adds
is the hardware, the run, and the lines in the report and `docs/hardware.md`.

**One thing to confirm on that machine before trusting a result:**
`"options":{"num_gpu":0}` — how the processor road is asked for — has never
been put to a real pinned runtime, because this machine has neither a runtime
nor a card. It is Ollama's own documented option and the ordinary path sends no
`options` field at all, but a road *asked for* and not *taken* would make the
comparison a comparison of one road with itself. `Road::of` reads what actually
happened, so the check is that the two runs report different roads; if the
release disagrees, `docs/quirks.md` gains the line.

Task 22 measures the road a machine without a card takes, and every refusal on
the way to a card that cannot be used. What it cannot show is a card **being
used**: that the runtime loads onto it, that `on_the_gpu_bytes` carries a number
a real device reported, and that the same question answered on the card and on
the processor comes back **the same** — ADR 0007's *a GPU changes speed, not
capability*, which is the claim this task exists to hold rather than assert.

- **Acceptance:** on a named machine with a card the runtime can use, the pinned
  model runs on it; `on_the_gpu_bytes` carries what that device reported, not a
  catalogue figure; one question is asked on the card road and on the processor
  road and **the two answers are held to each other**; the road taken is said in
  the vocabulary and never inferred from how long it took; and the run is pasted
  into the report with the machine named, as ADR 0056 asks of the chip's three
  promises. `docs/hardware.md` gains the line saying which machine showed it and
  when.
- **Constraint:** the card road is **never ticked from a machine without a
  card**, and no emulated or software device stands in for one — the same reason
  ADR 0056 rejected a simulated chip: a run against something that agrees with
  itself is evidence about the simulator. No driver of ours and no second runtime
  (ADR 0011, ADR 0006). `min_vram_gb` does not return as a judge of what is
  offerable.
