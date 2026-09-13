# One catalogue entry, graded on a machine that can hold it

**Date:** 2026-09-13
**Workstream:** v0.5 — the models, measured
**Task:** *One catalogue entry, graded on a machine that can hold it*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 1)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3** (8 cores, 4 performance and 4 efficiency), **8 GB
unified memory**, macOS 26.5.2. Gates in Ubuntu 24.04 aarch64 under Lima 2.2.0
(6 CPUs, 4 GiB, 8 GiB swap, kernel 7.0.0-31-generic), as root.
**Status:** done.

## What changed, for somebody outside this repository

The model catalogue now says **which machine every verb-driving grade was
measured on**, and refuses a grade that does not. And the first 7B model has
been measured: **Qwen2.5 7B Instruct drives the verbs 4 times in 10, which is
`rarely`.** No local model clears the bar yet, and the catalogue now says so
about a model large enough that *too small* is no longer the explanation.

## The grade

| | |
|---|---|
| Entry | `qwen2.5-7b-instruct` |
| Artefact | `qwen2.5:7b-instruct-q4_K_M` — 4,683,087,332 bytes, `sha256:845dbda0ea48ed749caafd9e6037047aa19acfcfd82e704d7ca97d631a0b697e` |
| Runtime | Ollama **0.34.0**, the version `image/Containerfile` pins, on `127.0.0.1:11434`, weights on the GPU through Metal |
| Machine | Apple M3, 8 GB unified memory, macOS 26.5.2 (the Linux VM stopped for the run) |
| Run | 2026-09-13, 18:13:40–18:14:14 UTC, one round of the fixed ten, 33.02 s |
| Result | **4 of 10 drove** — `drives_verbs = "rarely"` |

**Why this entry first.** The plan says the entry the catalogue recommends. The
catalogue recommends nothing — `agent_for_cpu` refuses on every machine because
no entry clears the bar — so the entry measured is the one that method's own
ordering ranks highest among those nobody had measured: among entries that run on
a CPU and may be used commercially, comfortable before workable and then larger.
Every comfortable one was already `rarely`; `eurollm-9b-instruct` is `slow`
and never a candidate; of the workable three, `qwen2.5-7b-instruct` has the most
parameters. So the first fact is about the model the recommendation would reach
for next.

**Which door.** The plan names `Asking::to_a_service_on_this_machine`. That door
is for a service a person runs themselves, and an answer through it says
`AServiceAtThisMachinesAddress` (ADR 0021). The runtime alo OS ships is reached
by `Asking::to_this_machine`, which is the door the existing harness already
uses and the one whose answer says *on this machine*. Both show nothing on the
indicator because nothing leaves; the run used `to_this_machine`, under
`SourcePolicy::ThisMachineOnly`, and asserted every answer's source.

**What was not changed.** The ten exercises, the prompt, the scoring and the
five-minute wait are the crate's as they were. The one change to the harness
prints every answer whole, after the run, so the grade below can be re-derived.
`Measured::of` refuses a run that skipped an exercise, and this run skipped
none — it is a measurement.

## The ten attempts, verbatim

```text

----- round 1, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 1, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"format":1,"asks":{"find_in_folder":{"verb":"read","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}`
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"format":1,"asks":{"rename_file":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"format":1,"asks":{"move_file":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"format":1,"asks":{"archive_folder":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices.zip"}}}
----- end

----- round 1, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"format":1,"asks":{"close_application":{"verb":"close_application","given":[{"application":"org.alo.Writer"}]}}}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
{"format":1,"asks":{"arrange_application":{"verb":"propose","given":[{"application":"org.alo.Writer","where":"left_half"}]}}}
----- end
```

**How it failed.** The four that drove are the three reads and `open`/`focus`;
every change except those two named its verb where the door belongs —
`{"asks":{"rename_file":{"verb":"rename_file",…}}}` instead of
`{"asks":{"propose":{"verb":"rename_file",…}}}`. `find` swapped the two, and
`archive` also lost the argument list. The daemon's door could not read any of
the six, so the registry was never reached. `docs/quirks.md` has this as *A 7B
model gets the reads right and addresses every change to the wrong door*.

## The machine beside the grade

`crates/alo-models/src/measured_on.rs` is new: a `[model.measured]` block of
`machine` (which must state its memory in GB or GiB), `date` (a real
`YYYY-MM-DD`) and `runtime` (a name and a version). `Catalogue::parse` refuses
a grade without one, a block beside `not-measured`, and a block that says
nothing checkable; `deny_unknown_fields` keeps a misspelt block from vanishing.
The seven grades already in the catalogue were given their blocks from the
reports that made them — the development PC's WSL2 Ubuntu guest, four cores and
5.8 GiB, Ollama 0.33.3, on 2026-09-04 and 2026-09-11 — rather than being left to
fail the new rule.

Tests: `a_grade_with_no_machine_beside_it_is_refused`,
`a_machine_that_says_nothing_checkable_is_refused_by_the_loader`,
`every_grade_we_ship_names_its_machine`, and `measured_on`'s own three, which
walk every refusal. Test fixtures that write a grade now write a machine too, so
each keeps being refused for the reason it names. The tests that held the old
state moved with it: `the_grade_the_weights_wait_on.rs` now accepts a graded
entry only when the finding names the machine that graded it (and shows that
refusal), the carry-or-fetch table repeats the new grade, and
`from_a_prompt_to_what_a_machine_offers.rs` counts five measured of seven.

## Egress

One fetch: `ollama pull qwen2.5:7b-instruct-q4_K_M` from `registry.ollama.ai`,
4.7 GB, 17:36:41–17:38:29 UTC — what `alo-egress` calls *fetching a model*. The
runtime itself, `ollama-darwin.tgz` 0.34.0, came from the project's GitHub
release and was checked against its published digest
(`dd12b00b…ef734`). The measurement caused none.

## Gates

Run in the Lima VM as root, `CARGO_TARGET_DIR=/root/alo-builds/alo-os-main`,
`RUSTDOCFLAGS="-D warnings"`, the nine commands of `tools/kernel-loop`'s
`EVERY_GATE` verbatim, on this task's tree:

| Gate | Result |
|---|---|
| formatting | pass |
| clippy, warnings denied | pass — zero warnings |
| the workspace's tests | **fail — one test, not this task's** |
| the supervisor's formatting | pass |
| the supervisor's clippy | pass |
| the supervisor's own tests | pass — 106 |
| rustdoc, warnings denied | pass |
| the BPF target's formatting | pass |
| the BPF target's clippy | pass |

`cargo test --workspace --no-fail-fast` on the same tree: **4,124 passed, 1
failed, 23 ignored**. The one is `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, which
fails the same way on the untouched tree (`b83d95e`) in this VM and is in
`docs/quirks.md`. After rebasing onto `1e2fdba` (lane A's pairing keys) the
nine were run again on the combined tree: the same eight pass, and the tests are
**4,165 passed, 1 failed, 23 ignored**, the same one. Every test in `alo-models`, `alo-driving`, `alo-choosing`,
`alo-answering` and `alo-telling` passes.

**This task is published with that gate red on this machine, and that is a
decision, stated here rather than hidden.** The failure is in a Linux-only
crate this lane may not edit, it reproduces on a tree this task did not touch,
and the setup prompt's rule for exactly this case — a refusal on aarch64 in the
kernel's half, while the rest passes — is to record it and not weaken anything.
Nothing was ignored, excluded or narrowed: the gate was run whole and its
result is the one above. Lane A's gates on the PC are the ones that decide
whether `alo-bounding` is green on x86_64.

## Setting up the Mac, and what it found

- `~/Documents` on this Mac is iCloud Drive, a file-sync folder the
  constitution rules out for the checkout, so the checkout is `~/dev/alo-os`.
- The VM needed more than `a-loop-on-a-mac.md` listed; that document now says
  what: the desktop's libraries, `gnome-keyring`, a diverted dbus activation
  file, the HWE kernel, the BPF LSM on the command line, root, and swap.
- On the release kernel (6.8) the verifier refuses the boundary: *combined stack
  size of 3 calls is 544. Too large*. On 7.0 it loads.
- **Both BPF gates pass on aarch64.**
- Two workspace tests failed on the untouched tree in this VM, neither in this
  lane's crates, both in `docs/quirks.md` for their owners (the second passed on
  this task's tree, for no reason this task gave it): `alo-bounding`'s
  `ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`
  (*"an ordinary program can set a flag on its own files: Os { code: 95, kind:
  Unsupported, … }"*) and `alo-citing`'s
  `a_real_decision_taken_off_the_real_list_is_refused`, which depends on the
  order `read_dir` lists `docs/decisions/` in.

## What this does not claim

- Nothing about alo OS on certified hardware. A Mac is not the machine; no
  *On the machine* box moved, and `ROADMAP.md` is untouched because ADR 0028
  forbids this plan moving a v0.01 line.
- Nothing about `min_ram_gb` or `on_cpu` for this entry. The run was on a GPU
  in unified memory; those fields describe a CPU.
- One round is ten attempts, the plan's number. Earlier grades used twenty.
  4 of 10 is five short of the bar and one short of `sometimes`; a second round
  is a bigger sample, not a different method, and task 2 may run one.

## Next

Task 2 — every entry graded, or refused with the reason — depends only on this
and is next.
