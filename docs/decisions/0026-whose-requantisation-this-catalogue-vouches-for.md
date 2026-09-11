# ADR 0026 — Whose requantisation this catalogue vouches for

**Status:** **ACCEPTED, 2026-09-11 — Option C**, with the conditions below taken
as written. Accepted under the same standing delegation from the owner that
[ADR 0024](0024-what-a-person-signs-in-at.md) and
[ADR 0025](0025-the-default-is-what-a-machine-arrives-able-to-do.md) were
accepted under — *"make decisions where needed"*, given on 2026-09-10 and
recorded in [ADR 0021](0021-what-a-service-on-this-machine-vouches-for.md). The
owner did not review these options one by one and this decision does not pretend
otherwise; it is recorded here so that whoever reads it later knows whose
judgement it was and can overturn it with one line. Everything below is left as
it was argued.
**Date:** 2026-09-11
**Proposed by:** the v0.01 delivery workstream, lane B, as task 14 of
`docs/autonomy/v0-01-lane-b-plan.md`
**Context:** [ADR 0006](0006-the-pinned-model-runtime.md) (the pinned runtime,
and *weights are never redistributed by us*),
[ADR 0007](0007-the-cpu-is-the-default.md) (the CPU is the default, and the
verb-driving grade is **measured by us and never claimed by a publisher**),
[ADR 0019](0019-a-runtime-is-found-not-configured.md),
[ADR 0025](0025-the-default-is-what-a-machine-arrives-able-to-do.md) (the local
model is what the machine arrives ready to run); `crates/alo-models`,
`crates/alo-models/data/catalogue.toml` rules 1, 4 and 5, `crates/alo-driving`,
`docs/features.md` (*A model runs in one command, from a curated catalogue of
open-weight models with their licences stated*), `docs/quirks.md`

## The question in one line

**When a model's publisher ships no quantised artefact, may a catalogue entry
name a stranger's requantisation of it — and if it may, what is this catalogue
claiming when it does?**

## What is true today, verified rather than remembered

Read off the tree this was written in.

- **Rule 4 made a quantisation point at a file.** `quantisation` and `artefact`
  are one claim, `Catalogue::parse` refuses half of it, and `artefact` is what
  the pinned runtime fetches and what `ALO_DRIVING_MODEL` is set to. So the
  grade and the file it was earned against are written in the same place.
- **Rule 5 made the size belong to that file**, as arithmetic: bytes per
  parameter near 0.6 for a four-bit artefact, 2.0 for a `bfloat16` release, and
  an entry on the wrong side of 1.5 for what it claims fails to load.
- **Two entries can point at nothing.** `eurollm-9b-instruct` and
  `teuken-7b-instruct` name publishers who ship safetensors and no GGUF at all.
  After task 13 they state their publishers' own releases honestly —
  18_304_683_360 and 14_905_484_192 bytes, `on_cpu = "slow"`,
  `min_ram_gb` 24.0 and 21.0 — and both are `not-measured`.
- **Those two are the European entries**, and the catalogue leads with them in
  as many words *because nobody else lists them*. The catalogue's honesty
  therefore costs exactly the models it went out of its way to carry: they are
  the two it can say least about, and the two an ordinary laptop cannot run.
- **A grade is not blocked only by this.** Task 9 measured that this lane's box
  — a 5,926 MB WSL guest — cannot hold a 7B model at four bits at any useful
  speed. So naming an artefact makes these two entries *complete*; it does not
  by itself make them *measurable* here.
- **Four third parties publish Q4_K_M of both models** — `mradermacher`,
  `bartowski`, `QuantFactory` and `lmstudio-community`, read 2026-09-11 — and
  none of them is the publisher.
- **Nothing structural stops one being named today.** `artefact` is a string,
  and `Catalogue::parse` asks only that it be present beside a quantisation. A
  curator could write a stranger's repository into it this afternoon and no
  check in this repository would notice.

That last point is why this is a decision rather than a preference. The
catalogue is not currently refusing third-party artefacts; it is *silent* about
them, and silence is the state in which the next curator does whichever thing
seems reasonable on the day.

## What may not be done, whichever option is taken

- **ADR 0007 may not be weakened.** A grade is a measurement we ran. Nothing
  here may turn a requantiser's claim, a download count or a popularity signal
  into a measurement.
- **Rule 1's harm may not be reintroduced.** A catalogue that states something
  wrongly is worse than one that omits the model. Whatever is named must be
  something a reader can go and check.
- **`docs/features.md` may not be narrowed.** *A curated catalogue of
  open-weight models with their licences stated* is the promise; the v0.5 line
  *the catalogue recommends; it does not gate* means none of this decides what
  anybody may run on hardware they own. This is about what alo OS **offers and
  vouches for**, never about what a person may bring.
- **We do not redistribute weights** (ADR 0006, and the `upstream` field is that
  doctrine as data). Any option that ends with alo OS hosting a model file runs
  into this and has to say so out loud.

## The options

### Option A — first-party artefacts only

Only a file the model's own publisher publishes may be named. Where none
exists, the entry states the publisher's release and can never be graded. This
is what the repository does today, by rule 5 and by silence.

- **What it buys:** the simplest sentence anybody can hold in their head, and a
  catalogue whose authority never extends past a publisher's own upload.
- **What it costs, and it is the cost this task exists to notice:**
  - **The two European entries stay ungradeable forever**, and they are
    ungradeable for a reason that has nothing to do with the models: their
    publishers are research organisations with no distribution team. The
    catalogue's stated bias toward European models becomes a list of entries it
    can say the least about.
  - **The rule silently favours publishers with release engineering.** Meta,
    Google, Alibaba, Microsoft and IBM all ship GGUF or have it shipped for them
    into the runtime's own library; openGPT-X and the EuroLLM project do not.
    An eligibility rule that tracks a publisher's operations budget is not a
    rule about model quality.
  - **It is not even a rule yet.** Nothing enforces it, so *first-party only* is
    at present a sentence in a comment that the next curator may not read.
- **Not rejected outright:** it is the honest fallback, and it is what an entry
  falls back to when Option C's conditions cannot be met.

### Option B — name any requantisation, stated and nothing more

Write `artefact = "hf.co/<somebody>/<repo>:Q4_K_M"` and carry on.

- **What it buys:** every entry becomes completable this afternoon, and the two
  European models become gradeable on a machine with room.
- **What it costs:**
  - **alo OS's authority behind a file alo OS never inspected.** A
    requantisation is a *derivative*: the converter version, the imatrix
    calibration set, the tokeniser it was built against and whether the upload
    is the model it says it is are all the uploader's business and none of them
    is stated. A file can also be replaced under the same tag after we measured
    it.
  - **A grade would migrate.** `drives_verbs` would read as a property of *the
    publisher's model* while having been earned against a stranger's file, which
    is ADR 0007's *measured by us, never claimed by a publisher* broken from the
    other end — the measurement is ours and the *thing measured* is not what the
    entry appears to name.
  - **Rule 1's harm, one field over.** A licence line stating the publisher's
    terms beside a third party's upload says nothing about the terms that
    third party attached to their upload.
- **Rejected.** It is the version where naming is free, and the whole difficulty
  is that naming is not free.

### Option C — a third party's artefact is nameable, and naming it costs something *(recommended)*

An entry may name an artefact its publisher did not publish **only when the
entry says whose it is, which exact file it is, and what a reader needs to know
about it.** Three statements, all checked by the loader:

1. **Whose.** The requantiser is named in the entry as a person or organisation
   — not left to be read out of a URL, and never equal to the publisher.
2. **Which file, pinned.** The artefact's own `sha256`, as its repository
   publishes it, so *the file we measured* and *the file a machine fetches* are
   provably the same file and a tag re-pointed later is a mismatch rather than a
   silent substitution.
3. **What a reader needs to know**, in a note: how it was made where the
   uploader says so, and any terms the upload carries beyond the model's own.
   This is rule 1's *say which conditions* applied to provenance.

And two rules about what a grade then means:

4. **A grade belongs to the artefact, never to the model in general.** An entry
   that names no artefact may not carry a grade at all — a measurement with no
   file behind it is a number about nothing — and wherever a grade is reported
   next to a requantised entry, the requantiser is reported with it.
5. **When the conditions cannot be met, the entry is carried and not omitted**:
   no quantisation, the publisher's own release, `not-measured`, exactly as the
   two European entries stand now. Omitting the model would hide the state of
   the world; carrying it says *nobody has quantised this for us*, which is
   true and useful.

- **What it buys:** the two European entries become completable by a deliberate
  act with a name on it. The catalogue stops being silent about third-party
  artefacts — the state in which the next curator guesses. And the cost of
  naming one is made visible rather than moral: three statements a curator must
  be able to produce, none of which can be produced about a file they have not
  actually looked at.
- **What it costs, and none of it is free:**
  - **We are still vouching for somebody else's file**, and saying whose does
    not make it correct. What it does is make the claim checkable and
    attributable: a reader can see the file, the digest and the name, and can
    disagree with the choice.
  - **Two more fields and a refusal path** in `crates/alo-models`, plus a pin
    that the fetch must eventually verify — an obligation this decision creates
    and does not discharge (see *What the code waits on*).
  - **A pin ages.** A requantiser who re-uploads a better file makes our entry
    stale rather than wrong, and the curator has to notice. That is the correct
    direction for this error to fall.

### Option D — alo OS quantises the weights itself

Run the conversion ourselves against the publisher's release, and the artefact
is one we made.

- **What it buys:** the only option where the file behind a grade is one we
  chose end to end, and it treats every publisher alike.
- **What it costs:**
  - **We would have to host it**, which is ADR 0006's *weights are never
    redistributed by us* reversed — a doctrine `Model::upstream` encodes as
    data, and reversing it means hosting derivatives of the Gemma terms, the
    Llama community licence and everything else the catalogue carries, with the
    redistribution obligations of each.
  - **It adds a build step and an engine** — a converter pinned and run — to a
    repository whose standing rule is that engines are configured, never
    written, and whose image is a pinned artefact list.
  - **The on-machine variant** — the person's own machine downloads 15–18 GB
    and converts locally, so nothing is redistributed — costs hours on the
    laptop ADR 0007 makes the default and contradicts ADR 0025's *the machine
    arrives ready to run*.
- **Rejected for v0.01**, and named because it is the road that reopens if
  third-party artefacts turn out not to be trustworthy: the constraint is
  hosting, and a machine that converts locally is the version that keeps ADR
  0006 intact at a cost in time rather than in doctrine.

## The recommendation

**Option C.** A third party's artefact may be named, and naming one is a
deliberate, attributable act with three statements attached and a pin behind it.
Where those cannot be produced, Option A is what the entry falls back to, which
is where the two European entries stand today.

The distinction that makes this keepable beside ADR 0007: **we never borrow
somebody else's measurement, and under Option C we never do.** What we borrow is
a *file*, stated as theirs, pinned so it cannot change under the measurement we
then run ourselves.

## Consequences

- **`crates/alo-models/data/catalogue.toml` gains rule 6**, because rules 4 and
  5 already send a curator to the file's own rules and this is the question they
  left open. It says what must be stated, and that a catalogue entry may not be
  completed by naming a file nobody looked at.
- **`crates/alo-models` gains the refusals**: a requantisation block with no
  artefact beside it, a requantiser with no pin, a pin that is not a `sha256`, a
  requantiser equal to the publisher, a provenance note that says nothing — and
  **a grade on an entry that names no artefact**, which is rule 4's other half
  and was reachable until this change.
- **`drives_verbs` is unchanged in meaning and in every entry.** No grade moves
  in the change that adds this decision; nothing here measures driving.
- **The two European entries are unchanged by this ADR**, and completing them is
  the next task: naming an artefact is a curation act with sizes to restate and
  a licence question to answer per uploader, and doing it inside the decision
  would be the decision quietly implementing itself.
- **A grade against a requantised entry is reported with the requantiser**
  wherever it is shown. No panel in this repository draws a catalogue entry yet,
  so this lands as a property of the data — `Model::graded_against()` answers
  which file a grade belongs to, and the entry's own `requantised` block answers
  whose it is — rather than as a string invented before there is anywhere to show
  it
  (`crates/alo-models/src/words.rs` says why this crate does not invent those).
- **`docs/features.md` is untouched.** Nothing here narrows *a curated catalogue
  of open-weight models with their licences stated*; it says what *curated*
  means for a file the publisher did not upload.

## What this does not decide

- **Which requantiser to choose for either European entry.** That is the next
  curator's, under rule 6, with the reasoning written into the entry.
- **Whether either model can be graded at all.** Task 9 measured the machine,
  not the models: a 7B model at four bits does not run usefully on this lane's
  box, and that stays true however the file is named.
- **What the fetch does with a pin.** This decision requires the pin to be
  stated and checkable. Verifying it at fetch time belongs with the weights work
  (task 10), where a digest is already how `image/Containerfile` treats every
  pinned artefact.
- **Anything about weights a person brings themselves.** `crates/alo-models`'
  `Brought` is a list a person owns, it has no licence field on purpose, and
  *what you bring is yours* (`docs/features.md`, v0.5) is untouched by every
  word here.

## What the code waits on

Nothing. The rule and its refusals land in the change that adds this decision,
because a rule this repository has already broken twice in the same file —
rule 4 was written from a quantisation nobody could point at, rule 5 from a size
that belonged to no artefact — is a rule that has to be arithmetic rather than
prose. What waits is the **curation**: the two European entries, and the fetch
that checks a pin.
