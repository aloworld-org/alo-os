# A file a person brings, measured against the bar

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *Does a file a person brings clear the bar*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 4)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama **0.34.0**
(pinned) on `127.0.0.1`, the Linux VM stopped during the run. Gates in Ubuntu
24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

When somebody points alo OS at weights they already have, those weights can now
be measured the same way the catalogue's models are, and the result is written
into **their own settings**, beside the file, with the machine and date it was
measured on. Weights that drive the verbs dependably can then be given agent
turns; weights that do not keep answering questions and are told so in a
sentence of their own. A grade typed into a settings file by hand, with no
machine beside it, is shown but no longer gives the agent. The grade goes
nowhere else — a test reads the code to prove it.

**The measurement on this machine:** the Qwen 2.5 7B GGUF on this disk, brought
by file, drove **3 of 10 — `rarely`**. No file on this machine clears the bar,
so lane B's task 10 stays blocked, with the truer reason now written into it.

## The run

`against_a_file_brought_to_this_machine.rs`, 2026-09-13 22:10:42–22:11:37 UTC
(2026-09-14 on the machine's clock, which is the date recorded):

- **The file:** `Qwen2.5-7B-Instruct-Q4_K_M.gguf`, 4,683,073,952 bytes, which is
  the blob `sha256:2bada8a7…3730` already on this disk. It was not fetched for
  this task, and it is the only kind of file the machine has: every GGUF on it
  is also a catalogue entry. The road is what differs — id by the file's own
  name, handed over by `bring` — and not the bytes.
- **The runtime answered by that id**, through `Asking::to_this_machine` under
  `SourcePolicy::ThisMachineOnly`, and every answer's source was asserted.
- **Written** to a settings file through `Choosing::measuring`:

```toml
format = 3

[[brought]]
id = "Qwen2.5-7B-Instruct-Q4_K_M.gguf"
bytes-on-disk = 4683073952
drives-verbs = "rarely"
file = "/Users/disanssebowabasalidde/dev/alo-vm/brought/Qwen2.5-7B-Instruct-Q4_K_M.gguf"

[brought.measured]
machine = "Apple M3, 8 GB unified memory, macOS 26.5.2, weights on the GPU through Metal"
date = "2026-09-14"
runtime = "Ollama 0.34.0"
```

  and read back as `can be the agent: false`. The settings file is a scratch
  one: no alo OS runs on this Mac, so there is no person's file to write into.

**The same bytes drove 4 of 10 asked by their catalogue name the day before.**
The runtime samples at its default temperature (0.8) because alo OS asks the
way a turn asks, so one round of ten is ten samples; `docs/quirks.md` has *The
runtime samples every answer*. Both results are `rarely`.

### The ten attempts, verbatim

```text

----- round 1, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 1, read: NotAMessage(NotReadable)
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}}
----- end

----- round 1, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"format":1,"asks":{"rename_file":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"format":1,"asks":{"move_file":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"format":1,"asks":{"archive_folder":{"verb":"propose","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 1, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, focus: NotAMessage(NotReadable)
{"format":1,"asks":{"focus_application":{"verb":"focus_application","given":[{"application":"org.alo.Writer"}]}}}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"format":1,"asks":{"close_application":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
{"format":1,"asks":{"arrange_application":{"verb":"propose","given":[{"application":"org.alo.Writer","where":"left_half"}]}}}
----- end
```

## What was built

- **`alo-models`**:
  - `Weights` carries `measured: Option<MeasuredOn>`;
  - `Weights::measured` takes the machine, so there is no road to a grade without one;
  - `grade_is_placed`, and `can_be_the_agent` requires a placed grade;
  - two outcome sentences — `WEIGHTS_MEASURED_THE_AGENT` and
    `WEIGHTS_MEASURED_NOT_THE_AGENT` — replacing the single `WEIGHTS_MEASURED`,
    and `WEIGHTS_GRADE_NOT_PLACED` for a grade that says no machine;
  - all three on the list the nudge rule reads.
- **`alo-choosing`**:
  - `Choosing::measuring(id, grade, machine)`, which refuses weights nobody
    brought (`NotWritten::NothingToMeasure`) and a machine that states
    nothing checkable (`GradeNotPlaced`), writing nothing in either case;
  - `[brought.measured]` in the settings file;
  - `tests/a_brought_file_measured_is_the_agent_or_offered_for_what_it_is.rs`,
    one test per acceptance;
  - `tests/a_grade_travels_nowhere.rs`, which reads the crate's shipped source
    and manifest for any way off the machine, and shows the check catching one.
- **`alo-driving`**:
  - the measurement loop moved to `tests/measuring/mod.rs`, so a catalogue
    entry and a brought file are measured by one piece of code;
  - the brought-file harness itself.
- **`docs/contracts/person-settings.md`**: `measured`, documented as additive.

**One decision, stated.** Refusing a settings file with a grade and no machine
was built first, and then taken out: the settings file is a public contract, and
the constitution says a contract changes additively. So a file written before
this reads exactly as it did; what changed is that an unplaced grade does not
give the agent, and the contract says so. The refusal lives on the one road that
**writes** a grade.

## Lane B's task 10, and what it is really waiting on

Its block said a catalogue entry that clears the bar. The truer sentence, now in
`docs/autonomy/v0-01-lane-b-plan.md`: the 7B and 8B entries have been measured
on a machine that holds them, and brought by file, and every one is `rarely` —
failing the call's grammar (door and verb confused), not its size.

## Egress

None. The file was already on this disk.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root: formatting, clippy with warnings denied, rustdoc, the supervisor's
three and both BPF gates pass. The workspace's tests, run whole with
`--no-fail-fast`: **4,337 passed, 1 failed, 25 ignored** — the one is
`alo-bounding`'s `ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`,
the aarch64 failure recorded in the first task's report and `docs/quirks.md`.
Every test in `alo-models`, `alo-choosing`, `alo-driving`, `alo-telling` and
`alo-answering` passes. The publish script re-runs all nine on the tree combined
with `main` and pushes only if the result is this one.

## What this does not claim

Nothing about a person's real settings file or a certified machine: the file
written here is a scratch one, and a Mac is not the machine.
