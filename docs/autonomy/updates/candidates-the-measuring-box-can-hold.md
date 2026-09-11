# Candidates the measuring box can actually hold

**Date:** 2026-09-11
**Workstream:** v0.01 lane B — accounts and session entry
**Task:** *Candidates the measuring box can actually hold*
(`docs/autonomy/v0-01-lane-b-plan.md`, task 12)
**Contributor:** Claude Code worker, checkout `C:\dev\alo-os-b`
**Status:** ready for integration.

## What this task was, and what it produced

Task 9 tried to grade a 7B entry on this lane's box and could not: the guest has
5,926 MB, the model loads more slowly than `alo_models` waits for an answer, and
the guest went down under the attempt. Task 12 was written from that outcome —
not a bigger model run harder, but **entries small enough to be measured here
that have a reason to clear the bar**. The five already-measured entries are
general chat models that failed at the *shape*; models trained for tool calls and
constrained output are a different population and nobody had put one to the
fixed ten.

**Two were added, both measured, and both graded `rarely`.** A third was
measured and deliberately not catalogued. The hypothesis was tested and it did
not hold: training for tool calls does not carry a model of this size over the
bar, so task 10's blocker is a *size* question rather than a curation one. That
is a more useful sentence than the one the task hoped for, and it is the
deliverable.

The second half of the task — the question Teuken raised about a `quantisation`
nobody can point at — is answered in the catalogue's own rules and enforced by
the loader.

## The two entries

| | `qwen3-1.7b` | `granite-3.2-2b-instruct` |
|---|---|---|
| Name | Qwen3 1.7B | Granite 3.2 2B Instruct |
| Publisher | Alibaba Cloud | IBM |
| Artefact | `qwen3:1.7b` | `granite3.2:2b` |
| Quantisation | Q4_K_M | Q4_K_M |
| `download_bytes` | 1_359_279_776 | 1_545_296_256 |
| Licence | Apache-2.0, unconditional | Apache-2.0, unconditional |
| Drove | **3** of 20 | **1** of 20 |
| Grade | `rarely` | `rarely` |

**Chosen for their training, then filtered by memory** — which is the order the
task asked for. Qwen3 is published with an agentic/function-calling mode and
Granite lists function calling among its stated core capabilities; both are
under 1.6 GB, so the box that measured everything else could measure them too.

**Every number is checked, not remembered.** The licences are the `license`
field of the publishers' own Hugging Face metadata, read 2026-09-11
(`Qwen/Qwen3-1.7B` → `apache-2.0`; `ibm-granite/granite-3.2-2b-instruct` →
`apache-2.0`), and both publish first-party GGUF repositories under the same
licence. The sizes are the model layer of each artefact's own registry
manifest, confirmed against the copies on this disk — not a rounded figure.

## What the run found, in one sentence

The two models fail in different ways and neither way is reasoning. Qwen3 drops
the door (`{"format":1,"asks":{"open_application":{…}}}`), shouts the
identifiers (`"verb":"READ"`, `"named":"FILE"`), and once leaked a token of
another language into the middle of a file operation. Granite loses the
punctuation — a brace over, a brace short, a stray quote — and twice wrote
`"format":2`, a message from a version of alo OS that does not exist, which is
the first time any measured model has reached `alo-protocol`'s
`FromANewerAloOs` branch. `docs/quirks.md` carries both, with the per-outcome
tallies and what each model actually wrote, beside the earlier runs.

**Nothing moved to help them.** Not the prompt (the registry's, as
`alo-driving` builds it), not the scoring (`alo-protocol`'s reader and
`alo-capability`'s validation), not the runtime's context window, and not
`alo_models::WHILE_A_MODEL_THINKS`. Qwen3 was measured with its thinking
enabled because that is what the artefact does by default and therefore what a
machine would get; turning it off would have been a measurement of a different
model.

## What I decided, and why

The task left several things open. Each was decided rather than handed back.

**Which candidates.** The criterion the task set is *trained for structured
output or tool use*, not *small*. Qwen3 1.7B and IBM Granite are the two
publishers who state that capability plainly, publish under an unconditional
Apache-2.0 licence, and ship something a 6 GB box can load. Both conditions had
to hold: a model with a conditions-bearing licence is catalogued too, but it is
never a default, and the catalogue's first rule makes stating a licence wrongly
worse than omitting the model.

**Granite 3.2 rather than 3.3.** `ollama pull granite3.3:2b` fails with
`Error: EOF` on this box's Ollama 0.33.3, repeatably, while three other models
pulled from the same registry minutes either side of it, and the explicit tag
`granite3.3:2b-instruct-q4_K_M` answers *file does not exist*. The 3.2 release
is the same family, pulls normally and is Apache-2.0. **No client was patched
and no version was moved** — engines are configured, never patched. The
behaviour is in `docs/quirks.md` under *Pinned engines*, with the one visible
difference in the failing manifest recorded as a plausible and unconfirmed
cause.

**Hermes 3 was measured and is not catalogued.** It is the strongest case for
the hypothesis — Nous Research tag it *function calling* and *json mode* — and
it drove 0 of 20. It is left out on rule 1 rather than on its grade: the
publisher's own metadata says `license: llama3` while the stated base model is
Llama **3.2**, and those are two different Meta community licences. A catalogue
that picked the one we think they meant would be doing exactly what rule 1
forbids. The grade is recorded in `docs/quirks.md` so the run is not lost.

**The quantisation question: pair it, or do not claim it.** `quantisation`
becomes `Option<String>` and gains a companion `artefact`, and
`Catalogue::parse` refuses either half alone — *a quantisation nobody can point
at*. `artefact` is what the pinned runtime fetches (or a publisher's repository
where the runtime's library has no entry), which is also what
`ALO_DRIVING_MODEL` is set to, so a grade and the file it was earned against are
written in the same place. Ten entries gained the runtime tag they were always
measured or fetched by, each verified against the registry.
`teuken-7b-instruct` and `eurollm-9b-instruct` state **no** quantisation: every
Q4_K_M of either is a stranger's requantisation, and choosing one in passing is
how a catalogue starts pointing at files nobody curated. The rule is written at
the top of `data/catalogue.toml`, because that is what the next curator reads.

**The size gap those two now carry is written down, not corrected here.**
Their `download_bytes`, `min_vram_gb` and `min_ram_gb` were four-bit figures for
artefacts the entries no longer claim. Correcting them changes the
carry-or-fetch table task 8 holds to the catalogue, and doing that inside a
measurement run would have mixed a curation fix into a grade. Rule 4 records the
gap in as many words and task 13 is written for it.

**Two rounds, not one.** The five existing grades were made over two rounds of
the fixed ten; `measured.rs` is explicit that repeats are a bigger sample rather
than a different method, and matching the earlier runs keeps the grades
comparable.

## What changed

- `crates/alo-models/src/catalogue.rs` — `quantisation` is optional and paired
  with a new `artefact`; `Catalogue::parse` refuses either half alone;
  `Model::quantised_at` is the only road to the pair. Two new unit tests, one of
  them putting all four half-statements in front of the loader. The `MEASURED`
  list is seven.
- `crates/alo-models/data/catalogue.toml` — rule 4 and the reasoning behind it;
  `artefact` on the ten entries that have one; the quantisation claim removed
  from `teuken-7b-instruct` and `eurollm-9b-instruct` with the reason above each;
  two new entries with the grades they earned; the header's counts brought up to
  date.
- `crates/alo-models/tests/candidates_the_box_can_hold.rs` — new. Holds the two
  candidates to having been measured, to fitting the box that measured them, to
  naming an artefact, and to an account in `docs/quirks.md` that carries each id,
  each artefact and each grade; holds the method to having been left alone; holds
  rule 4 to being in the catalogue's own rules; and puts every one of its own
  refusals in front of itself.
- `docs/quirks.md` — a new entry under *Models* with the two runs, the tallies,
  what each model wrote and the third candidate that was not catalogued; the
  carry-or-fetch table and its surrounding sentences brought into agreement with
  a fourteen-entry catalogue; a new entry under *Pinned engines* for the
  `granite3.3:2b` pull failure.
- `docs/autonomy/v0-01-lane-b-plan.md` — task 12 marked **Done, 2026-09-11**
  with its outcome; task 13 written from it.

Nothing was touched in `crates/alo-shell`, no weights went near the image, no
setup flow was written, and `docs/contracts/` did not move —
`person-settings.md`'s `quantisation` is the *brought weights* field, which is a
different type and is unchanged.

### For a person outside this repository

alo OS measures every model in its catalogue before it will let one act on your
files, rather than repeating what the publisher claims. The models measured so
far were general chat models and all of them lost the structure. This change
added two models their makers train specifically for this kind of work — a
small Qwen3 and an IBM Granite — and measured them the same way. They did no
better. That is worth knowing: the reason a small local model cannot yet drive
alo OS is its size, not the way it was trained, and the catalogue now says so
with the numbers behind it. The catalogue also no longer claims a file format
for weights nobody publishes in that format.

## Verification

Run from `C:\dev\alo-os-b`, through the WSL2 Ubuntu guest the gates use
(`CARGO_TARGET_DIR=/root/target-claude`, `RUSTDOCFLAGS=-D warnings`):

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test --workspace` | passed |
| `cargo test -p alo-models --test candidates_the_box_can_hold` | 6 passed |
| `cargo test -p alo-models --lib catalogue` | passed |

The measurement itself, for the record and **not** as a gate — it is the run the
grades came from:

```
ollama serve (0.33.3), OLLAMA_KEEP_ALIVE=10m, OLLAMA_MAX_LOADED_MODELS=1
ollama pull qwen3:1.7b ; ollama pull granite3.2:2b ; ollama pull hermes3:3b
ALO_DRIVING_MODEL=<artefact> ALO_DRIVING_ROUNDS=2 \
  cargo test -p alo-driving --test against_a_model_on_this_machine -- --ignored --nocapture
```

Results: `qwen3:1.7b` 3/20 `rarely`, `granite3.2:2b` 1/20 `rarely`,
`hermes3:3b` 0/20 `rarely`.

**The measuring box was left as it was found.** Ollama was not running when this
task started; the three models' weights were removed afterwards and the runtime
stopped. Re-measuring any of them means re-fetching, which took about a minute
each at the link speed here.

## Limitations, and one thing the supervisor should know

- **No grade cleared the bar**, so task 10 stays blocked and task 8's
  carry-or-fetch sentence is unchanged. The table under it grew two rows, which
  `the_carry_or_fetch_measurement.rs` required.
- **The finding is about models of this size**, not about tool-call training in
  general. A 7B or 8B model trained the same way may well clear the bar; nothing
  here says what it would score, and this box still cannot run one.
- **`hermes3:3b` has a grade and no entry.** If Nous Research correct their
  licence metadata, the entry can be written from the run recorded in
  `docs/quirks.md` without re-measuring.
- **Two entries carry sizes with no artefact behind them** — task 13.
- The box has Ollama 0.33.3 and the image pins 0.34.0
  (`image/Containerfile`). That gap is unchanged by this task and is the same
  one task 9 recorded.
- **This checkout was reset under me while the task was running.** At 15:46 a
  concurrent recovery in `C:\dev\alo-os-b` checked out `recover-11`, pulled
  `main` and reset the tree; `git reflog` has it. Every uncommitted file of this
  task — including an untracked test file — was discarded and had to be written
  again. Nothing was lost permanently, but `docs/autonomy/SHARED_MAIN.md`'s *one
  agent per working tree* was not held to here, and a worker that had been
  mid-`cargo test` rather than mid-edit would have handed over a tree that did
  not match its report.

## Proposed shared-document updates

Not edited here — `docs/autonomy/SHARED_MAIN.md` gives them one writer.

- **CHANGELOG.md:** *The model catalogue gained two entries — Qwen3 1.7B and
  IBM Granite 3.2 2B — chosen because their publishers train them for tool calls
  and structured output, and measured rather than assumed: both drive the verbs
  `rarely`, so neither can be given the agent. A catalogue entry may no longer
  state a quantisation without naming the artefact it means; two entries whose
  publishers ship no such file now state none.*
- **QUEUE.md / STATE.md:** lane B task 12 done 2026-09-11, two entries added and
  measured, no grade clearing the bar; task 10 still blocked, and its blocker is
  now understood to be model size rather than the catalogue's breadth; task 13
  written and ready. The standing item worth carrying is that **seven measured
  entries all grade `rarely`**, including two trained for the job.
