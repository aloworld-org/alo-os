# A model on the disk, sized for the machine it lands on

**Date:** 2026-09-22
**Workstream:** lane B — the image's weights half (`C:\dev\alo-os-3`, AGAI01)
**Task:** 10 in `docs/autonomy/v0-01-lane-b-plan.md`, *A model on the disk,
sized for the machine it lands on*
**Status:** ready for integration. Not committed and not pushed; the supervisor
gates and publishes.

## First, the sentence the task said this report must open with

The grade that put `qwen3-8b` on this image was earned **in the envelope** —
the runtime holding the answer to the protocol's shape — and **a shipped
machine's agent turn does not yet ask that way.** Wiring `alo-asking`'s local
door and `alo-turn` to it is lane A's task 13 on the local-network plan, and it
has not landed. So what this change puts on the disk is weights whose grade is
real and whose road into the product is one task from finished; the catalogue's
recommendation to a person still reads the free grade (ADR 0032 §5), and that
is untouched here.

## What changed, in words a person outside this repository can read

Until today every alo OS image arrived carrying **Phi-3 Mini**, a model that
the catalogue's own measurement grades `rarely` at driving the agent's verbs —
which is to say it could not drive them. It was chosen when nothing in the
catalogue could, and it was the largest measured model that fitted the laptop
alo OS certifies first.

Something can now. The image carries **Qwen3 8B** at four bits, the entry
`alo_models::Catalogue::agent_for_cpu` recommends for a 16 GB machine with no
graphics card, which drove 20 of 20 in the words a turn actually shows a model.

Two more things changed, and both are about *not lying about which weights
these are*.

**The recipe no longer says which model the image carries.** It names one, and
`crates/alo-image` holds that name to what the catalogue recommends — so
swapping the model by editing a build argument is a failing test rather than a
shipped image. And where the catalogue recommends **nothing** for a machine's
class, the image must carry **nothing**: a machine that arrived with five
gigabytes of a model that cannot drive anything would be a machine whose first
promise is untrue, and what it says instead is the answer it already gives —
why there is no model here, what weights of your own would do, and the two
other places you could ask.

**And the weights now come from the runtime's own library rather than from the
publisher's repository**, because those are not the same file. Qwen publishes a
Q4_K_M GGUF of 5,027,783,488 bytes; the runtime library's is 5,225,374,496.
Every grade in this catalogue was earned against the library's. Fetching the
publisher's would have put weights on every machine that nobody here measured,
under a grade earned on weights nobody ships.

## The decisions I made, and why

**1. Which model: nobody's, because it is read rather than chosen.**
`Catalogue::agent_for_cpu(16.0)` answers `qwen3-8b` today. Writing that method's
answer into a checker as a constant would have been a second opinion about
which model this product ships, kept in a file nobody looks at — the drift
`crates/alo-image` exists to catch everywhere else. So
`crates/alo-image/src/arrives_with.rs` asks the method, and the recipe's
argument is held to what it says. The name in `image/Containerfile` is a
declaration, not a decision.

**2. The weights come from `registry.ollama.ai/v2/library/qwen3/blobs/sha256:…`,
addressed by content.** The first version of this recipe fetched Hugging Face
and objected in a comment that *a registry tag is not a digest*. That objection
is right about tags and wrong about registries: a blob URL under
`/blobs/sha256:…` names exactly one file, which is a stronger pin than a
repository revision. What actually decided it is the grade — the measurement
was made against the artefact `artefact` names, served out of that library. I
verified the blob is a plain GGUF (magic `GGUF`, version 3) and that its length
is the 5,225,374,496 bytes the catalogue's `download_bytes` already states, by
a range request rather than by downloading five gigabytes.

The same comparison on the model that was aboard is the uncomfortable half:
the publisher's `Phi-3-mini-4k-instruct-q4.gguf` is 2,393,231,072 bytes and the
library's is 2,393,231,808. **736 bytes apart**, so the machine has been
shipping weights that were not quite the ones graded since 2026-09-11, and
nothing here would have noticed. That is in `docs/quirks.md`.

**3. The template is pinned and fetched too, and this is new surface I added
deliberately.** A grade is earned against a model **as it was served**, and what
decides where one turn ends and the next begins is the runtime's template. Qwen3's
is 1,723 bytes of Go template with tool-call and thinking branches in it; the
alternative was to transcribe it into a shell `printf` in the Containerfile,
unbuilt and unverifiable in this lane, which is the kind of silent difference
this whole file exists to refuse. So `THE_MODELS_TEMPLATE` and
`THE_MODELS_TEMPLATE_SHA256` are a second pin beside the weights, checked the
same way and in the same step, and `TheWeights::carries_its_template_pinned` is
four conditions rather than three — the fourth being *and the import is
actually handed it*, because a template fetched, pinned, checked and then
ignored passes everything else.

The sampling parameters are written out as `PARAMETER` lines from the same
library's 120-byte params blob, because six short lines are checkable by eye
where a template is not. **No context length is set**, which is a change from
the phi-3 recipe's `num_ctx 4096`: the grade was earned against this model
served exactly as its library publishes it, and a parameter added in our file
would be serving it some other way.

**4. `the_weights_a_class_arrives_with` takes the memory as an argument.** The
*no entry clears the bar* half had to be shown happening, and the honest way to
show it was not a catalogue invented for the test. The real catalogue has five
measured entries a machine with **eight** gigabytes can run and none of them
clears the bar — so the recipe that really ships is put to that class and
refused for carrying weights, with `alo_models::NoAgentHere`'s own reason
rather than a sentence of the checker's. Strip the weights out and the same
class has nothing to say about them, which is the other half.

**5. The image's size is arithmetic and is labelled as arithmetic.** I did not
build the image; see *What is not claimed*. `docs/quirks.md` states the
predicted 8.79 GiB **as** a prediction, with the measured 6.15 GiB it is built
on, the blob difference it adds, and the sentence that a build is owed. Task 17
of the plan is that build, written from this outcome.

## What was added and changed

- `image/Containerfile` — `THE_MODEL`, `THE_MODELS_ARTEFACT`,
  `THE_MODELS_WEIGHTS` and `THE_MODELS_SHA256` moved to `qwen3-8b`; two new
  arguments for the template and its digest; the weights stage fetches both and
  checks both before anything reads either; the Modelfile is assembled from the
  fetched template and the library's own parameters. The comment block above
  the arguments now says that the model is read off the catalogue rather than
  chosen here, and why the road changed from the publisher's GGUF.
- `crates/alo-image/src/arrives_with.rs` — **new.** `ArrivesWith`, which is the
  catalogue's recommendation for a class or the refusal a person is shown for
  it, and `THE_CERTIFIED_LAPTOP_GB`, moved here from `checking.rs` so the check
  and its tests read one constant.
- `crates/alo-image/src/weights.rs` — the template pin: two arguments read, the
  digest-check reader generalised to find a fetch **by the argument that names
  its source** rather than by being the first fetch in the stage, and
  `carries_its_template_pinned`.
- `crates/alo-image/src/checking.rs` — the weights check restructured around
  the recommendation, and parameterised by the machine's memory so the
  no-recommendation path is testable against the real recipe.
- `crates/alo-image/src/wrong.rs` — three variants:
  `TheWeightsAreNotWhatTheCatalogueRecommends`, `TheWeightsCannotDriveAnything`,
  `TheTemplateIsNotPinned`.
- `crates/alo-image/src/lib.rs` — the module, its exports, and the table entry.
- `docs/quirks.md` — a new entry, *The image sized for the machine it lands on*,
  beside the carry-or-fetch measurement task 8 wrote: the size, the arithmetic
  it is predicted from, and the two-GGUFs finding. The 2026-09-11 entry about
  the weights a machine arrives with gained a dated note saying what superseded
  it and what in it still holds.
- `docs/autonomy/v0-01-evidence.md` — the entry for *the local model is what the
  machine arrives ready to run* rewritten: which model, who decides it, the
  no-weights case, and the bar now cleared with the road to it unfinished.
- `docs/autonomy/v0-01-lane-b-plan.md` — task 10 marked done; **task 17 written
  in the same change**, which is the build.
- `docs/autonomy/v0-01-delivery-plan.md` — task 31, the matching task, was
  already marked done on 2026-09-11 and now carries a dated note saying lane B's
  task 10 superseded it and in what way.

## Verification

Run on this machine (`AGAI01`), Windows host, gates executed on the Linux side
this checkout gates on — `/root/alo-trees/this-machine`, built into
`/root/alo-builds/this-machine`, which is the one build cache this machine uses.
No other Cargo process was running.

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all` then `cargo fmt --all --check` | clean |
| Lints | `cargo clippy --all-targets -p alo-image -p alo-models -p alo-reconciling -p alo-citing -- -D warnings` | clean, zero warnings |
| Rustdoc | `cargo doc -p alo-image --no-deps` with `RUSTDOCFLAGS=-D warnings` | clean |
| This crate's tests | `cargo test -p alo-image` | 280 + 47 passed |
| The crates that read what I changed | `cargo test -p alo-models -p alo-reconciling -p alo-citing` | passed |
| The image's neighbours | `cargo test -p alo-installing -p alo-updating -p alo-software -p alo-saying` | passed |
| The plan checks | `cargo test` in `tools/kernel-loop` | 152 passed |

Acceptance, one line per criterion, each run on its own:

| Acceptance | Workspace | Crate | Target | Test |
|---|---|---|---|---|
| The image carries the entry `Catalogue::agent_for_cpu` recommends for the certified class | `.` | `alo-image` | `--lib` | `checking::tests::the_image_carries_the_model_the_catalogue_recommends` |
| …and anything else is refused, with the twin that passes every other check | `.` | `alo-image` | `--lib` | `checking::tests::weights_that_are_not_the_catalogues_recommendation_are_caught` |
| The recommendation is the catalogue's own answer, not a name held here | `.` | `alo-image` | `--lib` | `arrives_with::tests::the_model_a_machine_arrives_with_is_the_one_the_catalogue_recommends` |
| A class with nothing that clears the bar arrives with nothing, and says which of the three reasons | `.` | `alo-image` | `--lib` | `arrives_with::tests::a_class_with_nothing_that_clears_the_bar_arrives_with_nothing_and_says_why` |
| A machine too small for the entry that clears the bar arrives with nothing rather than something smaller | `.` | `alo-image` | `--lib` | `arrives_with::tests::a_machine_too_small_for_the_one_that_clears_the_bar_arrives_with_nothing` |
| The shipped recipe is refused for a class that has nothing, in `NoAgentHere`'s own words | `.` | `alo-image` | `--lib` | `checking::tests::a_class_with_nothing_that_clears_the_bar_arrives_with_no_weights` |
| …and an image carrying none is right for that class | `.` | `alo-image` | `--lib` | `checking::tests::a_class_with_nothing_that_clears_the_bar_is_content_with_an_image_carrying_none` |
| The weights are pinned and digest-checked the way the runtime is, and the template with them | `.` | `alo-image` | `--lib` | `weights::tests::a_recipe_that_carries_the_weights_pinned_reads_as_one` |
| A template not pinned, not checked, or not used is read as none | `.` | `alo-image` | `--lib` | `weights::tests::a_template_that_is_not_pinned_checked_and_used_reads_as_none` |
| A template checked after something read it is not a check | `.` | `alo-image` | `--lib` | `weights::tests::a_template_checked_after_it_was_read_is_not_a_check` |
| …and the recipe that ships is refused when that check is removed from it | `.` | `alo-image` | `--lib` | `checking::tests::a_template_that_is_not_pinned_is_caught` |
| The whole shipped image still says one thing | `.` | `alo-image` | `--lib` | `checking::tests::the_image_this_repository_ships_agrees_with_itself` |
| The weights the recipe declares are aboard, pinned and measured, read from outside the crate | `.` | `alo-image` | `--test what_the_image_owes_the_daemons` | `the_weights_a_machine_arrives_with_are_aboard_pinned_and_measured` |
| The store the image carries is the weights once | `.` | `alo-image` | `--test what_the_image_owes_the_daemons` | `the_store_the_image_carries_is_the_weights_once` |
| The carry-or-fetch measurement still agrees with the catalogue after this change | `.` | `alo-models` | `--test the_carry_or_fetch_measurement` | `the_smallest_entry_that_clears_the_bar_is_read_off_the_catalogue` |
| The evidence ledger still reconciles against the definition | `.` | `alo-reconciling` | `--test every_v0_01_promise_is_reconciled` | `every_v0_01_promise_is_reconciled_against_evidence_that_runs` |

## What is not claimed

- **No build.** `podman build` of this recipe has **not** been run. The fetch of
  two new blobs, the two digest checks, the Modelfile assembled from a fetched
  Go template and the import of a GGUF from the runtime's own registry are a
  recipe, not a measurement. Task 17 of the lane plan is that build, and it is
  written with the three specific things that can fail where no test here can
  see them.
- **The 8.79 GiB is a prediction**, stated as one: the 6,607,474,716 bytes
  measured on 2026-09-12 plus the difference between two pinned blobs whose
  sizes their registries state, plus 1,651 bytes of template and parameters.
- **No machine has booted this image**, and nothing here may be read as having
  answered that.
- **No grade was earned or moved.** Nothing was put to `alo-driving` in this
  change; every grade read is one somebody else measured and wrote down.
- **No agent turn.** As the first paragraph says, the grade was earned in the
  envelope and a shipped machine does not ask that way yet.
- **Nothing was downloaded beyond metadata.** The blob's first 64 bytes, its
  `Content-Length`, the registry manifest, and the template and params blobs
  (1,723 and 120 bytes, whose sha256 I computed and which match the digests
  their names carry). Five gigabytes were not fetched, and the weights' digest
  is the one the registry's own manifest publishes for that layer.

## Remaining limitations

- A tag re-pointed at another upload is not a risk for these two fetches, since
  both are content-addressed; a registry that stopped serving them is, and it
  fails the build loudly, which is the right way round.
- `carries_its_template_pinned` reads that some line after the check names the
  fetched template and that the import is handed a Modelfile. It cannot read
  that those are the same shell command, and its rustdoc says so rather than
  implying more.
- The four catalogue entries larger than 8 GB remain unmeasured. This task needed
  **an** entry that clears the bar and had one; whether a bigger one would be a
  better agent on a 16 GB laptop is unmeasured and unclaimed.

## Proposed shared-document updates

I did not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`. For the integration owner:

- **CHANGELOG.md:** *The model every alo OS machine arrives with is now the one
  the catalogue recommends for the machine it lands on — Qwen3 8B at four bits,
  which drove 20 of 20 of the agent's verbs where the model shipped before it
  drove almost none. Which model is no longer written in the build recipe: it is
  read from the catalogue's own recommendation, and an image naming anything else
  fails a test. A machine whose class has nothing measured well enough arrives
  with no model at all and says so, rather than carrying gigabytes of one that
  cannot work. The weights and the template are now taken from the runtime's own
  library, pinned by content, because the publisher's file of the same model at
  the same quantisation is a different file from the one we measured. The image
  has not been built with them and no machine has booted it.*
- **ROADMAP.md:** no line ticks. The image line's machine half stays empty.
- **QUEUE.md / STATE.md:** lane B task 10 done; task 17 (*The recipe built with
  the model it now carries*) written into
  `docs/autonomy/v0-01-lane-b-plan.md` and ready. `v0-01-delivery-plan.md`
  task 31 was already done and now records what superseded it.
