# The pinned model runtime is on the image

- **Date:** 2026-09-11
- **Workstream:** lane B (`docs/autonomy/v0-01-lane-b-plan.md`, task 7)
- **Contributor:** build-loop worker, this checkout
- **Status:** ready for integration

## What changed

ADR 0025 (accepted 2026-09-11) made *the local model is what the machine
arrives ready to run* the promise, and named the expensive half: an artefact on
the disk of every machine we ship. This task is the first half of that work —
the pinned runtime (ADR 0006), without the weights.

**`image/Containerfile`** now carries the runtime the way it carries everything
else it did not write: as a pinned upstream artefact. `THE_RUNTIME=0.34.0` and
`THE_RUNTIME_SHA256=cf9588…c0804` sit with the other pins at the top of the
file; a build stage of its own fetches upstream's release artefact for this
architecture (`ollama-linux-amd64.tar.zst`, v0.34.0, the current release), and
the digest is checked with `sha256sum --check` **before** anything is unpacked
— a release published again under the same number is a failed build, never a
different runtime on a certified machine. The artefact lands exactly as
upstream lays it out: `/usr/bin/ollama` (mode set to 0755 beside the other
binaries) and `/usr/lib/ollama/`. **No weights, no unit and nothing that starts
it**: a runtime alone answers nothing (ADR 0019), and serving it belongs with
the weights work.

**`crates/alo-image`** holds it the way it holds everything else — a check that
names it and twins that break one line each. A new module,
`src/runtime.rs` (`TheRuntime`), reads the recipe leniently and is judged in
`src/checking.rs` by three new disagreements in `src/wrong.rs`:

- `TheRuntimeIsNotOnTheImage` — the binary or the libraries are not copied out
  of a build stage onto the machine. `--from=` is part of the question: an
  artefact copied out of the build context would be a runtime committed into
  this repository, the source-tree shape ADR 0006 refuses.
- `TheRuntimesVersionIsNotPinned` — the version is missing, `latest`, partial
  (`0.34`), or otherwise not one exact `major.minor.patch` release. Refused in
  words that say why: a floating version is a different runtime on two builds
  of one image.
- `TheRuntimeArrivesUnverified` — no whole sha256, or a digest no build step
  checks. The version says what was asked for; only the checked digest says
  what arrived.

`Image::at` now also reads the `Containerfile` beneath the image directory, so
a directory without one is not an image this crate can check. The shipped
image is held to all of it by the existing
`the_image_this_repository_ships_agrees_with_itself` and by a new integration
test in `tests/what_the_image_owes_the_daemons.rs`.

**`crates/alo-models`** (`src/ollama.rs`): `found_at` — the private half of
`found_on_this_machine` — now answers `None` for a runtime holding no weights.
Since this task, the runtime is an artefact on every machine this repository
builds, so *something answered at the known address* stopped implying anybody
put a model on the disk. A runtime alone must not read as a model: discovery
keeps giving ADR 0019's found-nothing answer, and a machine in that state keeps
saying nothing on it answers questions (`agentd.nothing-answers-questions`)
rather than routing a person's question to an empty runtime.

**`crates/alo-saying`** (`src/rented.rs`): a test pinning that the runtime's
name stays on `EVERYTHING_WE_RENT` with ADR 0006 as the reason, and that
everything this machine can say still names nothing rented — the moment the
artefact ships on every machine is exactly the moment *X is not running* wants
to be written into a sentence, and this is the check that catches it.

### User-readable change description

Every alo OS machine now leaves the build with the local model runtime already
installed — pinned to one exact, verified version — so running a model on your
own machine needs nothing found and nothing fetched except the model itself.
Nothing starts answering questions because of this: until a model is on the
disk and you choose it, your machine says so, plainly.

## Decisions taken (and why), where the task left them open

- **Fetched from upstream's release artefact, verified by sha256 — not copied
  from upstream's container image.** Both would pin; the release tarball is the
  layout upstream's own installer uses (`bin/`, `lib/ollama/`), while the
  container image's internal layout is an implementation detail nobody
  documents. The digest, checked before unpacking, is what makes the pin real
  either way.
- **Version 0.34.0** — upstream's current stable release at the time of
  writing, with the sha256 taken from the release's own published
  `sha256sum.txt` and pinned here so upstream's file is never consulted again.
- **No unit that serves the runtime.** The acceptance names none, ADR 0025's
  *until the person chooses, nothing answers* is easiest to keep with nothing
  listening, and an empty runtime serving on every shipped machine would be a
  socket with nothing behind it. The weights task decides what starts the
  runtime, beside the weights that make starting it mean something.
- **`crates/alo-image` names `/usr/bin/ollama`, and that is not a breach of
  ADR 0006's one-file rule.** The rule (re-affirmed by ADR 0019) is about how
  the runtime is *spoken to* — endpoint, naming convention, response fields,
  address — and all of that stays in `crates/alo-models/src/ollama.rs`. Where
  the artefact's files land is the image's own fact; the Containerfile has to
  spell it to put them there, and the checker could not check what it may not
  name. This is argued in `runtime.rs`'s module documentation.
- **Discovery semantics moved in `alo-models`, not in `alo-agentd`.**
  `found_on_this_machine` is the door everything uses; filtering the empty
  runtime there keeps the rule in the one file that knows the runtime exists,
  and no caller changes. The cost, stated honestly: a person who chose a local
  model, whose runtime is running but whose weights are gone, is now told the
  `NotRunning` sentence rather than `NotInstalled`. On every machine this
  repository ships that corner disappears when the weights task lands, and the
  alternative — an empty runtime reading as a model — is the thing the plan
  forbids by name.

## Acceptance criteria, and the test behind each

1. *The image carries the pinned artefact, with a check that names it and a
   twin that breaks one line* —
   `alo-image::checking::tests::an_image_that_dropped_the_runtime_is_caught`
   (plus `an_image_that_dropped_the_runtimes_libraries_is_caught`, and the real
   image under `the_model_runtime_is_aboard_pinned_and_verified` in
   `tests/what_the_image_owes_the_daemons.rs`).
2. *The check holds the pin; an unpinned or floating version is refused in
   words that say why* —
   `alo-image::checking::tests::a_runtime_left_floating_is_caught` (and
   `a_runtime_arriving_unchecked_is_caught` for a digest nothing checks;
   the words themselves also in
   `wrong::tests::a_floating_version_is_refused_in_words_that_say_why`).
3. *Discovery with the runtime present but no weights still answers
   found-nothing; a runtime alone must not read as a model* —
   `alo-models::ollama::tests::a_runtime_with_no_weights_is_nothing_found`.
4. *No rented name reaches a person through any string this adds* —
   `alo-saying::rented::tests::the_runtime_the_image_carries_is_a_name_a_person_never_meets`
   (this change adds no vocabulary at all; the test pins the runtime's entry on
   the rented list and re-walks the machine's whole vocabulary).

## Verification

Run on Windows 11, from the checkout root:

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean, zero warnings.
- `cargo test --workspace` — all green (see handoff evidence; each acceptance
  test also run alone with `--exact`).

Not executed, and said plainly: `docker build` of the image itself (the gates
do not build the image; the alo-image suite is exactly the check that exists
because a build proves nothing), and any boot. *An image that builds is not an
image that boots*, and no *On the machine* claim moves here.

## Remaining limitations

- The runtime is aboard and inert: no weights, no unit, nothing listening.
  ADR 0025's ledger entry stays unevidenced until a machine boots with a model
  on it — this task does not tick it, and the report says so on purpose.
- The carry-or-fetch measurement ADR 0025 owes is not taken here; it is
  written as lane B's task 8, ahead of any weights work.
- `docs/contracts/` unchanged — no public surface moved. `docs/quirks.md`
  unchanged — nothing disagreed with a specification.

## Proposed shared-document updates (for the integration owner)

- `CHANGELOG.md`: "The pinned model runtime (v0.34.0) is on the image, fetched
  and digest-verified at build; `crates/alo-image` refuses an image that drops
  it, floats its version, or stops checking its digest. A runtime with no
  weights reads as nothing found, so nothing answers until a model is on the
  disk and chosen."
- `docs/autonomy/v0-01-evidence.md` (owner's call): the *arrives ready to run*
  promise can now cite the runtime half as built, weights still owed.
