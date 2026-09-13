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

**Crates this plan owns:** `alo-models`, `alo-driving`, `alo-choosing`,
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

## Tasks

### 1. One catalogue entry, graded on a machine that can hold it

**Status:** ready. **Depends on:** nothing.

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

**Status:** ready. **Depends on:** 1.

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

**Status:** ready. **Depends on:** nothing.

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

**Status:** ready. **Depends on:** 1, 3.

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

**Status:** ready. **Depends on:** 2, 4.

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
