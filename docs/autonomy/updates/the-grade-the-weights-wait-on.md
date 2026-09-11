# The grade the weights wait on

**Date:** 2026-09-11
**Workstream:** v0.01 lane B — accounts and session entry
**Task:** *The grade the weights wait on* (`docs/autonomy/v0-01-lane-b-plan.md`,
task 9)
**Contributor:** Claude Code worker, checkout `C:\dev\alo-os-b`
**Status:** ready for integration.

## What this task was, and what it produced

Task 8 measured the carry-or-fetch question ADR 0025 owed and found the first
number did not exist: no catalogued entry clears the verb-driving bar, so the
weights-aboard task waits on the catalogue. Task 9 was the run that would
change that — `mistral-7b-instruct`, `teuken-7b-instruct` and
`qwen2.5-7b-instruct`, the three unmeasured entries an ordinary business laptop
could hold, put to `alo-driving` against the pinned runtime.

**The run was made and no grade was earned.** That is this task's finding, and
it is stated rather than softened: the machine this lane runs on cannot carry a
7B-class measurement, the three entries stay `not-measured`, and the task's own
constraint says exactly what to do about that — *a machine without the memory
for a run does not guess; it leaves `not-measured` standing, which is the true
sentence about it.*

Two further findings came out of looking for the weights, and both outlive this
box. One of them was a licence stated wrongly in a catalogue we ship, which is
the harm `data/catalogue.toml`'s own first rule names, and it is fixed here.

## The premise was about the wrong machine

The plan opened this task with *the memory question is answered*: the
development machine has 15.5 GB, measured 2026-09-11, so the three entries are
runnable here. The 15.5 GB is the Windows host's. The pinned runtime, and every
grade in the catalogue, live inside the WSL2 Ubuntu guest — that is where
Ollama is installed and where the five existing grades were made — and
`C:\Users\SBW\.wslconfig` caps that guest at **6 GB of memory and four
processors**. The cap is deliberate and its own comment gives the reason: WSL2
would otherwise take half of physical memory and not hand it back, on a host
that already pages about 20 GB.

So the box that measures models has 5,926 MB, which is the six gigabytes
`docs/quirks.md` has said all along. Nothing about that had changed; what had
changed was which machine somebody measured.

## What the box did, in numbers

Ollama 0.33.3, four CPU cores, `mistral:7b-instruct-v0.3-q4_K_M` — 4.4 GB, the
weights the entry's `upstream` names at the quantisation it states — fetched
2026-09-11. Two attempts:

| | |
|---|---|
| Cold load | 284.7 s, then 447 s on the second attempt |
| `alo_models::WHILE_A_MODEL_THINKS` | 300 s |
| Resident while loaded | 5.0 GB in a 5,926 MB guest; 4.3 GB RSS with 1.25 GB paged out, 464 MB free |
| CPU actually used | 1.5 of 4 cores — it waits on paging, not on arithmetic |
| Warm-up question (12 tokens in, 21 out) | 101.7 s: 0.41 tokens/s reading, **0.25 tokens/s writing** |
| The `list` exercise's prompt | **715 tokens** (2,607 characters, the ten verbs as the registry declares them) |
| Its first 512-token chunk | 138.32 s, 3.70 tokens/s |

The first run failed at `alo-driving`'s own warm-up: loading alone outlasts the
five minutes `alo-models` waits, so `RuntimeError::TookTooLong` arrived before
an exercise was put. The second run pre-loaded the model outside the harness —
legitimate, and the same state a real turn finds a machine in — and got past
the warm-up; the `list` exercise then passed five minutes with three tokens
written, `TookTooLong` again, and the harness stopped. That is the harness
working as designed: *a runtime that fails is not a model that failed*, and a
grade that blamed a model for this machine would be worse than no grade.

And between the two attempts **the guest itself went down** — `uptime` back to
zero, the runtime gone with it — while the host had about 700 MB of physical
memory free. A measurement that can take a shared checkout's build down with it
is not one this box can be asked for. Nothing else was building at the time; I
checked before the second attempt and there were no `cargo` or `rustc`
processes in the guest.

At 0.25 tokens per second, prefix caching does not rescue it either: ten
exercises whose answers are sixty tokens each is four minutes of writing per
exercise before a single token of prompt is read.

## What I decided, and what I refused to decide

Three ways round the box were available and all three were rejected. They are
written here because the next worker will reach them too.

- **Raise the guest's memory.** It needs `wsl --shutdown`, which
  `docs/autonomy/SHARED_MAIN.md` puts behind an idle handoff from both loops —
  and a 15.5 GB host that already pages cannot lend 12 GB to a VM. This is an
  owner's decision, not a worker's.
- **Install the runtime on the Windows side**, where the 15.5 GB is. A package
  install on a shared machine, a second 4.4 GB copy of the weights against a
  C: drive with 12.4 GB free and a 12 GiB floor the workstream keeps, and a
  host with 700 MB of memory to spare.
- **Loosen something.** A smaller context window, a shorter prompt, fewer
  exercises, a smaller quantisation than the entry states, or a longer wait.
  Every one of them measures the loosening. The task's constraint forbids it
  and so does ADR 0007; nothing in the prompt, the scoring, the runtime's
  context or `WHILE_A_MODEL_THINKS` was touched.

What I did decide, as the task's *decide rather than stop* asks:

- **The finding is the deliverable**, written where the ledger can carry it —
  `docs/quirks.md`, beside the earlier runs — rather than a task handed back.
- **It is held by a test rather than left as prose.** A paragraph saying *none
  of the three has been measured* rots the day one of them is, and nothing
  about that run would rewrite it. `crates/alo-models/tests/the_grade_the_weights_wait_on.rs`
  parses the entry, compares it against the catalogue, and fails the day a
  grade arrives — the same shape task 8 gave its own entry.
- **The next task is candidates this box can hold**, not the same run tried
  harder. The five measured entries are general chat models that failed at the
  *shape*; models trained for tool calls are a different population and nobody
  has put one to the fixed set. That is task 12 in the plan, with the
  hardware route named in its constraint so it is not forgotten.
- **`ALO_DRIVING_ROUNDS=1`** for the second attempt rather than the two rounds
  the earlier grades used, because the run was already at the harness's limits
  and `measured.rs` is explicit that repeats are a bigger sample, not a
  different method. It made no difference: nothing was scored.

## The two findings about Teuken

Looking for the weights `teuken-7b-instruct` names turned up two things, and
the first is a bug in something we ship.

**openGPT-X publishes the model twice, under two licences.** Hugging Face's own
metadata, read 2026-09-11: `Teuken-7B-instruct-research-v0.4` is
`license: other`, and `Teuken-7B-instruct-commercial-v0.4` is
`license: apache-2.0`. The catalogue entry named the **research** release while
stating `Apache-2.0` with `commercial_use = "permitted"` — so an organisation
reading our catalogue would have been told it may rely commercially on a
release its publisher licensed for research. `data/catalogue.toml`'s first rule
calls a licence stated wrongly worse than a model omitted. `upstream` now names
the commercial release, which is the one the licence line was always true of;
the size is unchanged, because the two releases differ in their licence rather
than in their weights. A test now refuses **any** entry that permits commercial
use while naming a release whose name says research — a curator's question
asked of all twelve entries, not a one-off correction.

**Neither release ships a first-party GGUF.** There is no `teuken` in the
pinned runtime's library at all, and every Q4_K_M of it is a third party's
requantisation. So the entry's stated `Q4_K_M` names an artefact its publisher
does not publish, and measuring one would be measuring a stranger's file while
reporting the grade as this entry's. That is **not** worked around: this
repository has no rule for what an entry means when its publisher ships nothing
at the quantisation stated, and inventing one in passing is how a catalogue
starts pointing at files nobody curated. It is written into task 12, and until
it is answered `teuken-7b-instruct` cannot be measured on any machine, however
much memory it has.

## What changed

- `crates/alo-models/data/catalogue.toml` — Teuken's `upstream` corrected to
  the commercial release, with the reason in a comment above the entry; the
  header's paragraph about the unmeasured seven now records that the fact was
  tested on 2026-09-11 rather than assumed. **No grade changed**: all twelve
  entries carry exactly the grades they carried this morning.
- `crates/alo-models/src/catalogue.rs` — the `MEASURED` list is still five, and
  the doc comment above it now says that is not for want of trying, so the next
  reader does not assume nobody attempted it.
- `crates/alo-models/tests/the_grade_the_weights_wait_on.rs` — new. Holds the
  finding to the catalogue, holds the method to having been left alone, refuses
  a commercial licence over a research release, and puts every one of its own
  refusals in front of itself.
- `docs/quirks.md` — two entries under *Models*: what the box could not do with
  its numbers, and the two Teuken findings.
- `docs/autonomy/v0-01-lane-b-plan.md` — task 9 marked **Done, 2026-09-11**
  with the outcome; task 12 written from it.

Nothing was touched in `crates/alo-shell`, no weights went near the image, no
setup flow was written, and `docs/contracts/` did not move.

### For a person outside this repository

alo OS measures every model in its catalogue before offering it the agent,
rather than repeating what the publisher claims. Three seven-billion-parameter
models were due that measurement. The machine doing the measuring could not run
them — it has six gigabytes and they need more — so they still say *nobody has
measured this*, which is the truth about them, and the numbers showing why are
written down. While looking for one of those models we found that our catalogue
pointed at a research-licensed release while telling businesses they could use
it commercially; that is corrected, and a test now asks the question of every
entry.

## Verification

Run from `C:\dev\alo-os-b`, through the WSL2 Ubuntu guest the gates use
(`CARGO_TARGET_DIR=/root/target-claude`, `RUSTDOCFLAGS=-D warnings`):

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test --workspace` | passed |
| `cargo test -p alo-models --test the_grade_the_weights_wait_on` | 6 passed |

The measurement itself, for the record and **not** as a gate — it earned no
grade, and its failure is the finding above:

```
ollama serve (0.33.3), OLLAMA_KEEP_ALIVE=60m
ollama pull mistral:7b-instruct-v0.3-q4_K_M
ALO_DRIVING_MODEL=mistral:7b-instruct-v0.3-q4_K_M ALO_DRIVING_ROUNDS=1 \
  cargo test -p alo-driving --test against_a_model_on_this_machine -- --ignored --nocapture
```

**The measuring box was left as it was found**, with one difference recorded
here: the five previously measured models' weights (11 GB) were deleted from
`/root/.ollama` to make room for the 7B pull, and the Mistral weights were
deleted afterwards. Those five grades are in the catalogue and `docs/quirks.md`
already; re-measuring any of them means re-fetching, which took about a minute
each at the link speed here. Ollama was not running when this task started and
is not running now.

## Limitations

- **No grade was earned for any of the three entries.** The task's acceptance
  asks for grades; what exists instead is a measured account of why this
  machine cannot produce them, and `not-measured` standing where guesses would
  otherwise go. Task 10, the weights-aboard task, therefore stays blocked and
  its blocker is unchanged.
- **The finding is about this box, not about 7B models.** A machine with
  16 GB for the runtime and more than four cores would very likely grade all
  three in an afternoon. Nothing here says what those grades would be.
- **Teuken cannot be measured anywhere yet**, for the artefact reason above,
  which is a question rather than a limitation of hardware.
- The image pins Ollama 0.34.0 (`image/Containerfile`, task 7) and the box has
  0.33.3, which is what the five existing grades were made against. That gap is
  unchanged by this task and is noted so nobody reads the version above as a
  regression.

## Proposed shared-document updates

Not edited here — `docs/autonomy/SHARED_MAIN.md` gives them one writer.

- **CHANGELOG.md:** *The catalogue's Teuken entry named a research-licensed
  release while stating an Apache-2.0 commercial licence; it now names the
  commercial release, and a test refuses the mismatch for every entry. Three
  seven-billion-parameter entries remain unmeasured, with the measurement
  attempt and its numbers recorded.*
- **QUEUE.md / STATE.md:** lane B task 9 done 2026-09-11 with no grade earned;
  task 10 still blocked on the catalogue; task 12 written and ready. The
  standing item worth carrying is that **the measuring box cannot hold any
  unmeasured entry**, so the catalogue's measured half is bounded by hardware
  rather than by effort.
