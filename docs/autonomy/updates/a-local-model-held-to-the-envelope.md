# A local model held to the envelope

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *Whether a turn asks a local model for the shape it must answer in*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 7)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama **0.34.0**
(pinned) on `127.0.0.1`, the Linux VM stopped during every run. Gates in Ubuntu
24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

A local model can now be asked for the agent's next request with the runtime
holding its answer to the protocol's outer shape — the version, and one of the
three doors an agent has. Asked that way, **Qwen 2.5 7B, running on an 8 GB
laptop, produced a request alo OS would act on 35 times in 40 (87.5%)**, where
asked freely it managed 40%. That is still just under the 90% alo OS requires
before handing a model the agent, so nothing is given the agent yet — but the
gap between a local model and a local agent is now five attempts in forty.

The decision is [ADR 0032](../../decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md).

## What was measured

| Model | Asked freely | Held to the whole call's schema (probe) | **Held to the envelope (harness)** |
|---|---|---|---|
| Qwen 2.5 7B, Q4_K_M | 8 of 20, `rarely` | 3 of 20 | **35 of 40 — `sometimes`** |
| Llama 3.1 8B, Q4_K_M | 1 of 10, `rarely` | 2 of 20 | **5 of 20 — `rarely`** |
| Mistral 7B v0.3, Q4_K_M | 0 of 10, `rarely` | 0 of 20 | **0 of 20 — `rarely`** |

- **The harness runs:** Qwen 2.5 7B 00:43:48–00:45:38 UTC, four rounds; a first
  two-round run at 00:38:41 gave 17 of 20. Mistral 00:39:41–00:40:50, Llama 3.1
  00:40:50–00:43:32, two rounds each. All on 2026-09-14.
- **The probe:** a scratch program outside the repository that built the same
  prompts with `alo_driving::Exercises` and scored with the same `attempt`,
  sending `format` directly. It measured the whole-call schema, and an envelope
  admitting two doors, which gave Qwen 19 of 20. **That 19 is not a grade**: it
  admitted two doors where an agent has three, and it was twenty samples.
- **Why the whole call's schema fails:** the runtime orders a schema's keys
  alphabetically, so a model writes an argument's value before its name
  (`docs/quirks.md`).
- **What is left in the envelope:**
  - Qwen's five failures in forty are three requests through the wrong door
    and two arguments out of shape;
  - Llama 3.1 mixes argument shapes and sometimes nests the verb as a door;
  - Mistral writes `given` as a plain object of name to value every time, which
    no envelope constraint reaches.

## What was built

- **ADR 0032**, accepted: an agent turn holds a local model to the envelope and
  the door, never the call; only for questions that must answer in the protocol,
  never a person's; only the pinned runtime; and a grade says how the model was
  asked.
- **`alo-models`**:
  - `in_the_envelope.rs`: `the_envelope()`, `THE_DOORS` and
    `THE_PROTOCOL_FORMAT`, with a test that the schema names every door once
    and nothing inside any of them;
  - `Ollama::answers_in_the_envelope`, and a test that only it carries a
    `format` and that a person's question never does;
  - the catalogue's `drives_verbs_in_the_envelope` and
    `[model.measured_in_the_envelope]`, placed and counted, never
    `not-measured`, and **not read by the recommendation** — a test builds an
    entry that is `reliably` in the envelope and `rarely` freely and shows the
    machine does not give it the agent.
- **`alo-driving`**:
  - the harness asks either way (`ALO_DRIVING_ASKED=in-the-envelope`);
  - `the_envelope_is_the_protocols.rs` holds the schema's version and doors to
    what `alo-protocol` reads from an agent, and to none of a person's;
  - the counts test covers envelope grades.
- **Also updated:**
  - the catalogue carries the three envelope grades;
  - lane B's task 10 now says how close the gap is.

## For lane A

**The agent turn does not ask this way yet**, and that is not this lane's to
change. `alo-asking`'s local door (`locally.rs`) calls `ModelRuntime::answers`,
and `alo-turn` decides what a turn asks. For a turn to use the envelope:

- a trait method (or a door) that reaches `Ollama::answers_in_the_envelope`
  for a question that must answer in the protocol;
- `alo-turn` using it for the agent's next request and never for a person's
  question;
- then the recommendation reads `drives_verbs_in_the_envelope` —
  ADR 0032 decision 5.

Until then no machine gives a local model the agent on an envelope grade, which
is what the catalogue's test holds.

## The answers, verbatim

### Qwen 2.5 7B, in the envelope, forty attempts

```text

----- round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}],"path":"/home/anna/Invoices"}},"format":1}
----- end

----- round 1, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 1, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 1, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 1, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 2, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 2, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 2, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 2, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 2, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"application":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 3, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 3, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 3, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 3, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 3, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 3, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 3, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 4, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 4, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 4, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 4, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 4, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 4, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 4, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end
```

### Mistral 7B, in the envelope, twenty attempts

```text

----- round 1, list: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"list_folder","given":{"folder":"/home/anna/Invoices"}} },
"format":1}
----- end

----- round 1, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":{"file":"/home/anna/Invoices/march.pdf"}} },"format":1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":{"folder":"/home/anna/Invoices","named":"october","most":20}} },"format":1}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}} },"format":1}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"asks":{"propose":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}}, "format":1}
----- end

----- round 1, archive: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"list_folder","given":{"folder":"/home/anna/Invoices"}}},
"format":1}
----- end

----- round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"application":"org.alo.Writer"}} },"format":1}
----- end

----- round 1, focus: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"focus_application","given":{"application":"org.alo.Writer"}} },
"format":1}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"close_application":{"application":"org.alo.Writer"}} },"format":1}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer","arrange_application":{"where":"left_half"}}}, "format":1}
----- end

----- round 2, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":{"folder":"/home/anna/Invoices"}} },"format":1}
----- end

----- round 2, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":{"file":"/home/anna/Invoices/march.pdf"}} },"format":1}
----- end

----- round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":{"folder":"/home/anna/Invoices","named":"october","most":20}} },"format":1}
----- end

----- round 2, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}}, "format":1}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}}, "format":1}
----- end

----- round 2, archive: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"list_folder","given":{"folder":"/home/anna/Invoices"}}},
"format":1}
----- end

----- round 2, open: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"open_application","given":{"application":"org.alo.Writer"}} }, "format":1}
----- end

----- round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"focus_application":{"application":"org.alo.Writer"}}}, "format":1}
----- end

----- round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"close_application":{"application":"org.alo.Writer"}} },"format":1}
----- end

----- round 2, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"arrange_application","given":{"application":"org.alo.Writer","where":"left_half"}} },"format":1}
----- end
```

### Llama 3.1 8B, in the envelope, twenty attempts

```text

----- round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}, "format":1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":"\"\""},{"folder":"/home/anna/Invoices","is":20}]}},"format":1}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}},"format":1}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"asks":{"propose":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}},"format":1}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"archive_folder":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices.zip"}}},"format":1}
----- end

----- round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"application":"org.alo.Writer"}}},"format":1}
----- end

----- round 1, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"focus_application":{"application":"org.alo.Writer"}}},"format":1}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"close_application":{"application":"org.alo.Writer"}}},"format":1}
----- end

----- round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}, "format":1}
----- end

----- round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":"\"\""},{"folder":"/home/anna/Invoices","is":1},{"most":20,"is":20}]}},"format":1}
----- end

----- round 2, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}},"format":1}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}},"format":1}
----- end

----- round 2, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"archive_folder":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}}},"format":1}
----- end

----- round 2, open: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"application":"org.alo.Writer"}}},"format":1}
----- end

----- round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"focus_application":{"application":"org.alo.Writer"}}},"format":1}
----- end

----- round 2, close: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/Anna"},{"named":"folder","is":"/Documents"}]}},"format":1}
----- end

----- round 2, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"arrange_application":{"verb":"arrange_application","given":[{"application":"org.alo.Writer","is":"left_half"}]}}}, "format":1}
----- end
```

## Egress

None.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root. The first run refused clippy for this task's own defect — the shared
measurement loop's envelope variant is unused in the brought-file harness — and
it was fixed with an `allow` that says why; `cargo clippy --workspace
--all-targets -- -D warnings` is then clean. Formatting, rustdoc, the
supervisor's three and both BPF gates pass. The workspace's tests, run whole with
`--no-fail-fast`: **4,425 passed, 1 failed, 25 ignored** — the one is
`alo-bounding`'s `ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`,
the aarch64 failure recorded in the first task's report and `docs/quirks.md`.
The publish script re-runs all nine on the tree combined with `main` and pushes
only if the result is this one.
