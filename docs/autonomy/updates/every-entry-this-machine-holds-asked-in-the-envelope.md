# Every entry this machine holds, asked in the envelope

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *Every entry this machine can hold, asked in the envelope*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 10)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama **0.34.0**
(pinned) on `127.0.0.1`, the Linux VM stopped during the runs. Gates in Ubuntu
24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

Every model in the catalogue that this 8 GB machine can run has now been measured
the way ADR 0032 has an agent turn ask — with the runtime holding its answer to
the protocol's envelope. The seven smaller models, including the two chosen
because their publishers train them for tool calls, all stay `rarely`: holding
the envelope helps a model that already writes the call's grammar, and at three
billion parameters and under, none does. **Qwen 2.5 7B, at 87.5%, remains the
closest any catalogued local model has come to the bar.**

## The grades

Every entry fetched through `Ollama::fetch` — the road task 8 fixed — and
answering by its catalogue id; then two rounds of the fixed ten, in the envelope,
through the harness. 2026-09-14, 02:14:05–02:18:34 UTC.

| Entry | In the envelope | Asked freely (the development PC, 2026-09) |
|---|---|---|
| `qwen2.5-7b-instruct` | 35 of 40, `sometimes` (task 7) | 8 of 20, `rarely` |
| `llama-3.1-8b-instruct` | 5 of 20, `rarely` (task 7) | 1 of 10, `rarely` |
| `teuken-7b-instruct` | 1 of 20, `rarely` (task 8) | 0 of 20, `rarely` |
| `mistral-7b-instruct` | 0 of 20, `rarely` (task 7) | 0 of 10, `rarely` |
| `llama-3.2-3b-instruct` | **3 of 20**, `rarely` | 0 of 20, `rarely` |
| `qwen3-1.7b` | **1 of 20**, `rarely` | 3 of 20, `rarely` |
| `granite-3.2-2b-instruct` | **0 of 20**, `rarely` | 1 of 20, `rarely` |
| `qwen2.5-3b-instruct` | **0 of 20**, `rarely` | 1 of 20, `rarely` |
| `gemma-2-2b-instruct` | **0 of 20**, `rarely` | 2 of 20, `rarely` |
| `smollm2-1.7b-instruct` | **0 of 20**, `rarely` | 0 of 20, `rarely` |
| `phi-3-mini-instruct` | **0 of 20**, `rarely` | 0 of 20, `rarely` |

The three the machine cannot hold — `gemma-2-9b-instruct`, `eurollm-9b-instruct`
and `mixtral-8x7b-instruct` — are task 9's, still blocked on memory.

## How the small ones fail inside the envelope

- **The door is often the first one the schema lists.**
  - Qwen3 1.7B asked five changes and Llama 3.2 3B six through `read`, and
    Granite asked four: `{"asks":{"read":{"verb":"rename_file",…}}`.
  - The envelope names `read` first; whether that order leans a small model
    toward it was not tested. Reordering the doors to raise a grade would be
    tuning the method to the candidate, which the plan forbids.
- **The call inside the door keeps each model's own mistakes:**
  - Qwen 2.5 3B spreads the arguments beside `verb`;
  - Gemma 2 2B and SmolLM2 write `given` as an object;
  - Phi-3 Mini names each argument by its value and answers `"is": true`.

## Egress

The seven fetches, 02:09:48–02:13:56 UTC, from `registry.ollama.ai`: the seven
artefacts the catalogue names, about 11 GB. They were fetched to be measured,
through the product's own road. The measurements caused none.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root. Formatting, clippy with warnings denied, rustdoc, the supervisor's
three and both BPF gates pass. The first run of the workspace's tests failed four
`alo-agentd` boundary tests beside the known aarch64 one, all for the same
reason — a control group and two pins left by an earlier fixture whose processes
had gone (`docs/quirks.md`, *A boundary fixture takes a control group away while
its process is still leaving*). That debris was removed, only where its process
was gone, and the publish script ran all nine again on the tree combined with
`main`, publishing only when the one failure was `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` — the
aarch64 failure recorded in the first task's report. Every test in `alo-models`
and `alo-driving` passes.

## The answers

### qwen3-1.7b, in the envelope, verbatim

```text

----- round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":{"folder":"/home/anna/Invoices"}}},"format":1}
----- end

----- round 1, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":[{"file":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"folder":"/home/anna/Invoices"},{"named":"october"},{"most":20}]}},"format":1}
----- end

----- round 1, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 1, archive: TheWrongDoor
{"asks":{"read":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices"}]}},"format":1}
----- end

----- round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer"}},"format":1}
----- end

----- round 1, focus: NoSuchVerb { named: "NAME" }
{"asks":{"read":{"verb":"NAME","given":[{"named":"ARGUMENT","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: ArgumentRefused(NoSuchArgument { verb: "close_application", argument: "org.alo.Writer" })
{"asks":{"read":{"verb":"close_application","given":[{"named":"org.alo.Writer","is":1}]}},"format":1}
----- end

----- round 1, arrange: NoSuchVerb { named: "ARRANGE_APPLICATION" }
{"asks":{"read":{"verb":"ARRANGE_APPLICATION","given":[{"named":"APPLICATION","is":"org.alo.Writer"},{"named":"WHERE","is":"left_half"}]}},"format":1}
----- end

----- round 2, list: NoSuchVerb { named: "LIST_FOLDER" }
{"asks":{"read":{"verb":"LIST_FOLDER","given":[{"named":"FOLDER","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"folder":"/home/anna/Invoices","named":"october","most":20}]}},"format":1}
----- end

----- round 2, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 2, archive: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"archive_folder","given":[{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}]}},"format":1}
----- end

----- round 2, open: TheWrongDoor
{"asks":{"read":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, focus: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"focus_application","given":{"application":"org.alo.Writer"}}},"format":1}
----- end

----- round 2, close: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"propose","given":{"file":"org.alo.Writer","name":"close"}}},"format":1}
----- end

----- round 2, arrange: TheWrongDoor
{"asks":{"read":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}, "format":1}
----- end

```

### granite-3.2-2b-instruct, in the envelope, verbatim

```text

----- round 1, list: ArgumentRefused(NoSuchArgument { verb: "list_folder", argument: "/home/anna/Invoices" })
{"asks":{"read":{"verb":"list_folder","given":[{"named":"/home/anna/Invoices","is":"folder"}]}},"format":1}
----- end

----- round 1, read: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"/home/anna/Invoices","is":"files"}]}},"format":1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"folder":"/home/anna/Invoices","named":"october"},{"most":20}]}},"format":1}
----- end

----- round 1, rename: TheWrongDoor
{"asks": {"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"archive_folder","given":{"folder":"/home/anna/Invoices","name":"invoices"}}},"format":1}
----- end

----- round 1, open: ArgumentRefused(NoSuchArgument { verb: "open_application", argument: "org.alo.Writer" })
{"asks":{"read":{"verb":"open_application","given":[{"named":"org.alo.Writer","is":"application"}]}},"format":1}
----- end

----- round 1, focus: TheWrongDoor
{"asks":{"read":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"move_file","given":{"file":"org.alo.Writer","into":"left_half"}}},"format":1}
----- end

----- round 2, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":{"folder":"/home/anna/Invoices"}}},"format":1}
----- end

----- round 2, read: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"/home/anna/Invoices","is":"files"}]}},"format":1}
----- end

----- round 2, find: ArgumentRefused(NoSuchArgument { verb: "find_in_folder", argument: "october" })
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 2, rename: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"rename_file","given":[{"file":"/home/anna/Invoices/scan001.pdf","name":"scan001.pdf"}]}},"format":1}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 2, archive: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"archive_folder","given":{"folder":"/home/anna/Invoices","name":"invoices"}}},"format":1}
----- end

----- round 2, open: ArgumentRefused(NoSuchArgument { verb: "open_application", argument: "org.alo.Writer" })
{"asks":{"read":{"verb":"open_application","given":[{"named":"org.alo.Writer","is":"application"}]}},"format":1}
----- end

----- round 2, focus: TheWrongDoor
{"asks":{"read":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, close: NoSuchVerb { named: "close" }
{"asks":{"read":{"verb":"close","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, arrange: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"move_file","given":{"file":"org.alo.Writer","into":"left_half"}}},"format":1}
----- end

```

### qwen2.5-3b-instruct, in the envelope, verbatim

```text

----- round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":[{"folder":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 1, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","file":"/home/anna/Invoices/march.pdf"}},"format":1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","file":"/home/anna/Invoices","named":"october","most":20}},"format":1}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"rename_file","file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}},"format":1}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"move_file","file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive/","where":"whole_screen"}},"format":1}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"archive_folder","folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}},"format":1}
----- end

----- round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"open_application","given":[{"application":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"focus_application","given":[{"application":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","application":"org.alo.Writer"}},"format":1}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"arrange_application","application":"org.alo.Writer","where":"left_half"}},"format":1}
----- end

----- round 2, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":[{"folder":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 2, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","file":"/home/anna/Invoices/march.pdf"}},"format":1}
----- end

----- round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","file":"/home/anna/Invoices","named":"october","most":20}},"format":1}
----- end

----- round 2, rename: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","file":"/home/anna/Invoices/scan001.pdf"}},"format":1}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"move_file","file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive/","where":"whole_screen"}},"format":1}
----- end

----- round 2, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"archive_folder","file":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}},"format":1}
----- end

----- round 2, open: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":[{"folder":"/Applications"}]}}, "format":1}
----- end

----- round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"focus_application","given":[{"name":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","application":"org.alo.Writer"}},"format":1}
----- end

----- round 2, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"arrange_application","given":[{"application":"org.alo.Writer"},{"where":"left_half"}]}},"format":1}
----- end

```

### llama-3.2-3b-instruct, in the envelope, verbatim

```text

----- round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}, "format":1}
----- end

----- round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}, "format":1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":"10.10"},"most","value","20"]}},"format":1}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"asks":{"read":{"propose":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}}},"format":1}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf"},{"into":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 1, archive: TheWrongDoor
{"asks":{"read":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices"}]}},"format":1}
----- end

----- round 1, open: TheWrongDoor
{"asks":{"read":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, focus: ArgumentRefused(SameArgumentTwice { argument: "application" })
{"asks":{"read":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"arrange_application","given":[{"application":"org.alo.Writer","where":"left_half"}]}},"format":1}
----- end

----- round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 2, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":[{"file":"","name":"march.pdf"}]}},"format":1}
----- end

----- round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":"Anna wants to find files with October in their names"},{"named":"most","is":"20"}],"folder":"/home/anna/Invoices"}},"format":1}
----- end

----- round 2, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"move_file","given":[{"file":"\"/home/anna/Invoices/march.pdf\""},{"into":"\"/home/anna/Archive\""}]}},"format":1}
----- end

----- round 2, archive: ArgumentRefused(NoSuchArgument { verb: "archive_folder", argument: "extension" })
{"asks":{"read":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices"},{"named":"extension","is":".zip"}]}}, "format":1}
----- end

----- round 2, open: TheWrongDoor
{"asks":{"read":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, focus: ArgumentRefused(SameArgumentTwice { argument: "application" })
{"asks":{"read":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"application","is":"org.alo.Writer"}]}}, "format":1}
----- end

----- round 2, close: NoSuchVerb { named: "close" }
{"asks":{"read":{"verb":"close","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, arrange: TheWrongDoor
{"asks":{"read":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

```

### gemma-2-2b-instruct, in the envelope, verbatim

```text

----- round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","folder":"/home/anna/Invoices"}},"format":1}
----- end

----- round 1, read: NotAMessage(MoreThanOneMessage)
{"asks": {"read": {"file": "/home/anna/Invoices/march.pdf" }
},"format": 1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":{"folder":"/home/anna/Invoices","named":["october"],"most":20}}},"format":1}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"rename_file","given":{"file": "/home/anna/Invoices/scan001.pdf"}}},"format":1}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":{"folder":"/home/anna/Invoices","named":["march.pdf"]}} },"format":1}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"asks": {"read": {"verb": "list_folder", "folder": "/home/anna/Invoices" } }, "format": 1}
----- end

----- round 1, open: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"open_application","given":{"named":"application","is":"org.alo.Writer"}}},"format":1}
----- end

----- round 1, focus: NotAMessage(NotReadable)
{"asks": {"read": {"verb": "focus_application", "given": {"application": "org.alo.Writer"}}}, "format": 1}
----- end

----- round 1, close: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"close","given":{"named":"org.alo.Writer", "is": "Anna"}}
},"format":1}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"focus_application","given":{"named":"org.alo.Writer","is":"org.alo.Writer"}}},"format":1}
----- end

----- round 2, list: NotAMessage(NotReadable)
{"asks": {"read": {"folder": "/home/anna/Invoices" } },"format":1}
----- end

----- round 2, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","file":["/home/anna/Invoices/march.pdf"]}},"format":1}
----- end

----- round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":{"folder":"/home/anna/Invoices","named":["october"],"most":20}}},"format":1}
----- end

----- round 2, rename: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"rename_file","given":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}},"format":1}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"asks": {"read": {"verb": "rename_file", "arguments": {"file": "/home/anna/Invoices/march.pdf", "into": "/home/anna/Archive"}}},"format":1}
----- end

----- round 2, archive: NotAMessage(NotReadable)
{"asks": {"read": {"verb": "list_folder", "arguments": {"folder": "/home/anna/Invoices"}}}, "format": 1}
----- end

----- round 2, open: NotAMessage(NotReadable)
{"asks": {"read": {"verb": "open_application", "given": {"application": "org.alo.Writer"}}},"format": 1}
----- end

----- round 2, focus: TheWrongDoor
{"asks":{"read":{"verb":"focus_application","given":[{"named":"application","is": "org.alo.Writer"}]}},"format":1}
----- end

----- round 2, close: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"CLOSE","given":[{"named":"APPLICATION","is":"org.alo.Writer"}]}},
 	"format":1}
----- end

----- round 2, arrange: NotAMessage(NotReadable)
{"asks": {"read": {"verb": "focus_application", "given": {"named": "application", "is": "org.alo.Writer" } } },"format": 1}
----- end

```

### smollm2-1.7b-instruct, in the envelope, verbatim

```text

----- round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"NAME","given":{"folder":{"named":{"named":"/home/anna/Invoices","is":1},"is":1}}}},"format":1}
----- end

----- round 1, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"NAME","given":[{"named":"ARGUMENT","is":{"type":"string","format":1,"value":"read_file"}},{"named":"FILE","is":{"type":"string","format":1,"value":"march.pdf"}}]}},"format":1}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":{"folder":"/home/anna/Invoices","named":"october","most":20}}},"format":1}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"rename_file","given":{"folder":"/home/anna/Invoices/scan001.pdf","named":"march.pdf"}}},"format":1}
----- end

----- round 1, move: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"rename_file","given":["/home/anna/Invoices/march.pdf","/home/anna/Archive"]}},
 "format":1}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"rename_file","given":{"named":["/home/anna/Invoice", "invoices"],"is":["a full path"]}}},"format":1}
----- end

----- round 1, open: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"NAME","given":[{"named":"ARGUMENT","is":{"type":"string","description":"A value of type 'application' that represents the identifier of an installed application."}}]}},
"format":1}
----- end

----- round 1, focus: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"PROPOSE","given":{"named":"APPLICATION","is":["ALO.Writer"]}}},"format":1}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"close","given":{"named":"org.alo.Writer","is":true}}},"format":1}
----- end

----- round 1, arrange: NoSuchVerb { named: "name" }
{"asks":{"read":{"verb":"name","given":[{"named":"app_id","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, list: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"NAME","given":[{"named":"ARGUMENT","is":{"type":"string","value":"/home/anna/Invoices"}}]}},
"format":1
}
----- end

----- round 2, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read","given":{"folder":"/home/anna/Invoices","named":"march.pdf"}}},"format":1}
----- end

----- round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list","given":{"folder":{"named":"/home/anna/Invoices","most":20,"named":{"named":["october"],"most":20}}}}},"format":1}
----- end

----- round 2, rename: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"rename_file","given":{"folder":{"path":["/home/anna/Invoices"],"length":["20"]},"file":{"path":["scan001.pdf"],"length":["20"]},"named":{"path":["march.pdf"],"length":["20"]}}}},

"format":1

}
----- end

----- round 2, move: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":["/home/anna/Invoices/march.pdf"]},{"named":"into","is":["/home/anna/Archive/"]}]}},
"format":1}
----- end

----- round 2, archive: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"READ","given":{"folder":{"name":"invoices","path":{"name":"/home/anna/Archive","name":{"text":"/home/anna/Archive/invoices","is":{"type":"path","value":true,"max_length":100}}},"is":{"type":"path","value":true,"max_length":100}}}}

},"format":1

}
----- end

----- round 2, open: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"NAME","given":{"named":"application","is":{"named":"org.alo.Writer"}}}},"format":1}
----- end

----- round 2, focus: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"open_application","given":{"application":{"name":"org.alo.Writer","identifier":"org.alo.Writer"},"focused":false}}},"format":1}
----- end

----- round 2, close: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"close","given":{"named":"org.alo.Writer"}}},"format":1}
----- end

----- round 2, arrange: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"OPEN_APPLICATION","given":{"name":"org.alo.Writer"}}},"format":1}
----- end

```

### phi-3-mini-instruct, in the envelope, verbatim

```text

----- round 1, list: NotAMessage(MoreThanOneMessage)
{
  "asks": {
    "read": {
      "verb": "list_folder",
      "given": [
        {"folder": "/home/anna/Invoices"}
      ]
    }
  },
  "format": 1
}
----- end

----- round 1, read: ArgumentRefused(NoSuchArgument { verb: "read_file", argument: "/home/anna/Invoices/march.pdf" })
{"asks":{"read":{"verb":"read_file","given":[{"named":"/home/anna/Invoices/march.pdf","is":"1"}]}},"format":1}
----- end

----- round 1, find: ArgumentRefused(NoSuchArgument { verb: "find_in_folder", argument: "october" })
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":20}]}},"format":1}
----- end

----- round 1, rename: NotAMessage(MoreThanOneMessage)
{
  "asks": {
    "propose": {
      "verb": "rename_file",
      "given": [
        {
          "named": "/home/anna/Invoices/scan001.pdf",
          "is": true
        },
        {
          "named": "march.pdf",
          "is": true
        }
      ]
    }
  },
  "format": 1
}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf"},{"into":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices.zip"}},"format":1}
----- end

----- round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer"}},"format":1}
----- end

----- round 1, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer","focus_application":"org.alo.Writer"}},"format":1}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer"}},"format":1}
----- end

----- round 1, arrange: NotAMessage(MoreThanOneMessage)
{
  "asks": {
    "propose": {
      "application": "org.alo.Writer",
      "where": "left_half"
    }
  },
  "format": 1
}
----- end

----- round 2, list: NotAMessage(MoreThanOneMessage)
{
  "asks": {
    "read": {
      "verb": "list_folder",
      "given": [
        {"named": "/home/anna/Invoices", "is": true}
      ]
    }
  },
  "format": 1
}
----- end

----- round 2, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":[{"named":"march.pdf","is":true}]}},"format":1}
----- end

----- round 2, find: NotAMessage(MoreThanOneMessage)
{
  "asks": {
    "read": {
      "verb": "find_in_folder",
      "given": [
        {"named": "october", "is": "20"},
        {"named": "Invoices", "is": "/home/anna/Invoices"}
      ]
    }
  },
  "format": 1
}
----- end

----- round 2, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"/home/anna/Invoices/scan001.pdf","is":false},{"named":"march.pdf","is":true}]}},"format":1}
----- end

----- round 2, move: NotAMessage(MoreThanOneMessage)
{
  "asks": {
    "propose": {
      "command": "move_file",
      "file": "/home/anna/Invoices/march.pdf",
      "into": "/home/anna/Archive"
    }
  },
  "format": 1
}
----- end

----- round 2, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"format":1,"verb":"archive_folder","given":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}}},"format":1}
----- end

----- round 2, open: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer"}},"format":1}
----- end

----- round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer","focus_application":"org.alo.Writer"}},"format":1}
----- end

----- round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer"}},"format":1}
----- end

----- round 2, arrange: NotAMessage(MoreThanOneMessage)
{
  "asks": {
    "propose": {
      "application": "org.alo.Writer",
      "where": "left_half"
    }
  },
  "format": 1
}
----- end

```
