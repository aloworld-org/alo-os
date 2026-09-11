# The weights a machine arrives with

**Date:** 2026-09-11
**Workstream:** the image (Claude's checkout, `C:\dev\alo-os-claude`)
**Task:** 31 in `docs/autonomy/v0-01-delivery-plan.md`, *The weights a machine
arrives with*
**Status:** ready for integration. Not committed and not pushed; the supervisor
gates and publishes.

## What changed, in words a person outside this repository can read

alo OS's image now says **which model a machine arrives with**, and carries it.
Until today the recipe put a model runtime on every machine and nothing for it to
load, which is a machine that can run a local model as soon as somebody downloads
one — not a machine that arrived able to.

The model is **Phi-3 Mini Instruct**, quantised `Q4_K_M`, fetched from
Microsoft's own GGUF repository at one pinned revision, held to a sha256 that the
build checks **before any other step reads the file**, and imported into the
runtime's model store during the build so the machine does not spend a person's
first minutes doing it. The weights **ride on the image** rather than being
fetched when somebody sets the machine up.

What this does not do is make the agent work. No model anybody has measured
drives the verbs dependably — this one included — so what a machine arrives able
to do is load and answer with a model of its own, offline, on arrival. And
nothing starts the runtime yet. Both are written down rather than implied.

## The decisions I made, and why

**1. Carried, not fetched.** ADR 0025 left this open and recommended carrying;
the recommendation is taken. What decides it is one sentence: *a machine that
fetches at setup has not arrived ready when it is offline at setup*, and the
promise is about what is in the box. The cost is real and is ours: 2.23 GiB of
image, and an update channel that moves those bytes whenever the pin moves. The
other answer costs the same bytes, paid by a person on their first morning, in a
place where nobody here can help them. `docs/quirks.md` carries the measurement
ADR 0025 asked for — including the part of it that came out negative (below).

**2. Which model: `phi-3-mini-instruct`.** Three catalogue facts decide it, and
none of them is a preference:

- **Measured.** Somebody put it to `alo-driving` on a real runtime. Seven
  entries have been; the other eleven have not, and the one model every machine
  arrives with is the last place to take a publisher's word (ADR 0007,
  `docs/features.md`'s *measured by us, not claimed by the publisher*).
- **It fits the machine that matters.** `docs/hardware.md` certifies two
  machines and says which decides whether this product has a market: an ordinary
  business laptop, 16 GB, no card. This entry needs 6 GB and is graded
  `comfortable` without a card.
- **Its licence was ours to hand on.** This is the one that is easy to miss:
  **carrying weights in an image is redistributing them.** A licence with
  conditions — Gemma's terms, the Llama community licences — would attach those
  conditions to everybody who receives a copy of alo OS. Phi-3 is MIT. The
  catalogue may offer a conditioned model; the machine may not arrive with one,
  and that is now a refusal rather than a habit.

It is the largest entry satisfying all three.

**3. The digest is checked in a step of its own, before anything reads the
file.** Stricter than the runtime's check, and the difference is the failure this
one can have: weights are *imported* by the runtime rather than unpacked by
`tar`, so a recipe that fetched, imported and then checked would have built a
model store out of whatever the network handed the build machine and gone red
with those bytes already in a layer. `TheWeights::is_verified` is the digest
**and** its position; `crate::recipe::in_stage` is what makes position an
answerable question.

**4. The recipe names the catalogue's `artefact`, not an invented name.** The
model is created under `phi3:3.8b-mini-4k-instruct-q4_K_M`, which is the string
`crates/alo-models/data/catalogue.toml` already publishes for that entry — so the
entry a person reads and the name their machine answers to are one string, and
`ALO_DRIVING_MODEL` reaches the model that is actually there. The weights come
from the publisher's GGUF rather than the runtime's registry because a registry
tag is not a digest.

**5. Where the weights land: `/usr/share/alo/models/`.** Ours rather than the
runtime's default, which is a home directory belonging to a login this image does
not make. What points the runtime at it is the unit that starts the runtime, and
this image has none — said in as many words beside the `COPY` line so nobody
reads a store on a disk as a model a machine is serving.

**6. No unit, deliberately.** Task 31's acceptance does not ask for one, and a
unit is not a `COPY` line's worth of decision: which login the runtime runs as,
which group may reach its socket, and what it may reach off the machine (law 1)
are a decision with security in it. Writing one here to make the image look
finished would be taking that decision in the file nobody reviews. It is written
as task 32 in the plan instead.

**7. A shared reader for the recipe.** `crate::runtime` and `crate::weights` were
each going to spell *a copy out of a build stage* and *a whole sha256*;
`crates/alo-image/src/recipe.rs` owns that spelling once. `runtime.rs` was moved
onto it in the same change, which is CLAUDE.md's fourth law rather than a
refactor for its own sake.

## The measurement ADR 0025 asked for, including its negative half

ADR 0025 asked for *the smallest catalogued model that clears the verb-driving
bar against what the update channel can carry*. **There is no such model.** Every
entry anybody has put to `alo-driving` — seven of them, five general chat models
and two trained for tool calls — is graded `rarely`. So no size of carried model
makes the agent work today, and the carry decision cannot be justified by the bar
it was supposed to be measured against. It is justified by the other half: a
machine that can load and answer with a local model, offline, on arrival. That is
in `docs/quirks.md` and in the evidence entry, in those words.

## What was added

- `image/Containerfile` — five build arguments (the catalogue id, the
  quantisation, the artefact, the pinned source and the digest), a `weights`
  build stage that fetches, checks and imports, and one `COPY` that lands the
  runtime's store on the machine. The runtime's own comment about *no weights*
  is now false and was rewritten rather than left.
- `crates/alo-image/src/weights.rs` — `TheWeights`, the reader, with
  `is_pinned` (the source names a revision, not a branch) and `is_verified` (a
  whole digest, checked before anything reads the file).
- `crates/alo-image/src/recipe.rs` — how a Containerfile line is read, once.
- `crates/alo-image/src/checking.rs` —
  `the_weights_are_aboard_pinned_and_measured`, seven refusals, and the
  catalogue lookup behind four of them.
- `crates/alo-image/src/wrong.rs` — eight variants, each naming the decision it
  breaks.
- `crates/alo-image/Cargo.toml` — `alo-models`, for the catalogue. No cycle:
  nothing in `alo-models` reaches `alo-image`.
- `docs/autonomy/v0-01-evidence.md` — the entry rewritten: what is now shown,
  what is still owed, and the three things standing in front of *arrives ready
  to run*.
- `docs/quirks.md` — the carry-or-fetch measurement ADR 0025 asked for.
- `crates/alo-reconciling/tests/every_v0_01_promise_is_reconciled.rs` — two
  assertions that were true about the old entry and are not about this one. That
  crate's audit asserts its own counts rather than describing them, so a promise
  moving from *no evidence at all* to *partly owed* fails a test until somebody
  says which moved. Both say so, in the test's own words, with the date.

## Why a test in another crate changed

`alo-reconciling` held two facts about this promise: that it named no evidence,
and that exactly three v0.01 promises name none. Both were accurate yesterday and
neither is now — the entry names evidence and the count is two. That is the
check working rather than the check being in the way: it refuses to let the
ledger's own summary drift, and the price is that whoever moves a promise writes
down which. Nothing was weakened; the assertion about this entry is now the
opposite one, that it names *something*, so an entry emptied later still fails.

## Verification

Windows 11, this checkout, 2026-09-11. Every command run from the checkout root.

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets --workspace -- -D warnings` — clean, no output.
- `cargo doc -p alo-image --no-deps` — clean, so the rustdoc links resolve.
- `cargo test -p alo-image` — 138 unit + 19 integration, all passing.
- `cargo test -p alo-reconciling` — 20 unit + 15 integration, all passing.
- `cargo test -p alo-citing` — 21 + 10, all passing: it reads these documents
  for pointers that land.
- Every test named in the handoff's evidence block run again on its own with
  `--exact`, each selecting exactly one test.

**Not run, and named rather than skipped quietly:** the workspace suite (the
supervisor's) and `docker build -f image/Containerfile`. **No container build of
this recipe has been made in this lane**, so the fetch, the digest check and the
import are a recipe rather than a measurement; what is tested here is the
declaration and its refusals, which is what task 31 asks for and all a gate that
downloads nothing can do. The digest itself was read twice from Hugging Face's
own metadata for that file — the LFS object id and the `X-Linked-ETag` of the
same URL, which agree — rather than by downloading 2.23 GiB on a build machine.

## What is still owed

- **A machine.** Nothing here says the image boots, and nothing may be read that
  way.
- **The unit that starts the runtime** — task 32, written into the plan.
- **A model that clears the bar.** Until one exists, a machine arrives able to
  run a local model and not able to be given an agent turn.

## Proposed shared-document updates

I did not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`. For the integration owner:

- **CHANGELOG.md:** *The image now says which model a machine arrives with, and
  carries it: Phi-3 Mini Instruct, Q4\_K\_M, pinned by digest and checked before
  anything reads it. Which model is decided by the catalogue — measured by us,
  small enough for the laptop we certify first, and under a licence that was ours
  to pass on — and an image naming anything else fails a test. It does not yet
  start the model runtime, and no machine has booted it.*
- **ROADMAP.md:** no line ticks. The image line's machine half stays empty.
- **QUEUE.md / STATE.md:** task 31 done, task 32 (the unit that starts the model
  runtime) written into `docs/autonomy/v0-01-delivery-plan.md` and ready.
