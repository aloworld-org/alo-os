# Teuken's chat template, and the fetch that could not fetch

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *Teuken's chat template, decided before Teuken is graded*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 8)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama **0.34.0**
(pinned) on `127.0.0.1`, the Linux VM stopped during the runs. Gates in Ubuntu
24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

- **Teuken's template.** Teuken 7B, the European model the catalogue lists
  second, is now fetched with the chat template its publisher wrote for it. The
  file the catalogue names carries none, so it used to be asked in a shape it
  was never trained on.
- **Teuken's grade.** With the template it was measured for the first time:
  **0 of 20, and 1 of 20 with the protocol's envelope — `rarely` both ways.**
- **Fetching.** While doing this, a more basic fault turned up and was fixed:
  alo OS asked the model runtime to download a catalogue model by the catalogue's
  own name, which the registry does not know, so **no catalogued model could have
  been downloaded by alo OS at all**. It now downloads the file the entry names
  and makes it answer to the catalogue's name.

## The template, and where it comes from

openGPT-X publishes the template twice for
`openGPT-X/Teuken-7B-instruct-commercial-v0.4` (revision `e91cc26f`):

- in the model card, as `f"System: {system_messages[lang_code]}\nUser: {user}\nAssistant:"`,
  with the note that the model "must be used with the provided prompt template";
- in `gptx_tokenizer.py`, the code `apply_chat_template` runs, as a system line
  per language plus a Jinja template whose generation prompt is `Assistant: `.
  Its `default` is the English system message.

The catalogue carries the tokenizer's `default`, rendered for the one shape alo
OS asks in — a single message — in the runtime's own template language:

```text
System: A chat between a human and an artificial intelligence assistant.The assistant gives helpful and polite answers to the human's questions.
User: {{ .Prompt }}
Assistant: 
```

The two sources differ by one space (*"assistant.The"* in the code, *"assistant.
The"* in the card); the code is what the model is served with, so it is the one
used, and the catalogue's note says so. Nothing was written that openGPT-X did
not publish.

**What the runtime answered.** Without the template:

- the runtime logged *"model is missing tokenizer.chat_template"*;
- it answered `I'm ready.<|im_end|>`, with the stray token in the text.

With it: `" ready."`.

## The fetch that could not fetch

`fetch` sent `POST /api/pull {"model":"mistral-7b-instruct:latest"}`; the pinned
runtime answered `{"error":"pull model manifest: file does not exist"}`.

It now:

- pulls the entry's `artefact`;
- then `POST /api/create {"model": <id>:latest, "from": <artefact>}`, adding
  `"template"` only for an entry that states one.

Walked on the real runtime:

- `mistral-7b-instruct` was fetched as `mistral:7b-instruct-v0.3-q4_K_M`, was
  listed under its catalogue id, and answered by it;
- `teuken-7b-instruct` was fetched as its borrowed artefact with the template
  carried, and was listed under its catalogue id.

## Teuken, measured

Two rounds each, 2026-09-14:

- asked freely: 01:35:35–01:37:15 UTC;
- in the envelope: 01:37:28–01:39:21 UTC.

| | Drove | Grade |
|---|---|---|
| Asked freely | **0 of 20** | `rarely` |
| In the envelope (ADR 0032) | **1 of 20** | `rarely` |

Its answers cut paths short and misspell them (`/home/anna/Invoic`,
`/home/anna/Invoicnes`) and pass arguments as bare strings; no template or
envelope reaches that. Teuken's `too-large-for-the-measuring-machine` reason is
gone: the first run ran out of GPU memory, and the second loaded.

### Asked freely, verbatim

```text

----- round 1, list: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"list_folder","given":["/home/anna/Invoic"]}}}
----- end

----- round 1, read: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoic
----- end

----- round 1, find: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoic
----- end

----- round 1, rename: NotAMessage(NotReadable)
 {"format":1,"asks":{"propose":{"verb":"rename_file","given":["/home/anna/Invoic
----- end

----- round 1, move: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"move","given":["/home/anna/Invoic
----- end

----- round 1, archive: NotAMessage(NotReadable)
 {"format":1,"asks":{"proppose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoic
----- end

----- round 1, open: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"open_application","given":[{"named":"org.alo.Writers"}]}}}
----- end

----- round 1, focus: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"arrange_applicatio
----- end

----- round 1, close: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"close_applicatio
----- end

----- round 1, arrange: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"arrange_applicatio
----- end

----- round 2, list: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoic
----- end

----- round 2, read: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoic
----- end

----- round 2, find: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoic
----- end

----- round 2, rename: NotAMessage(NotReadable)
 {"format":1,"asks":{"propuse":{"verb":"rename_file","given":["/home/anna/Invoic
----- end

----- round 2, move: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"move","given":[{"named":"folder","is":"/home/anna/Archive"},{"named":"file","is":"/home/anna/Invoic
----- end

----- round 2, archive: NotAMessage(MoreThanOneMessage)
 {"format":1,"asks":{"read":{"verb":"move","given":[{"named":"into","is":"/home/anna/Archive"}],"argumets":[]}}}
{"format":1,"asks":{"proppose":{"verb":"archive_folder","given":[{"named":"into","is":"/home/anna/Archive"}],"arguments":["Invoic"]}}}
{"format":1,"asks":{"read":{"verb":"list_folder","given":[],"arguments":["/home/anna/Invoic"]}}}
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"Invoc"}],"arguments":["/home/anna/Invoc"]}}}
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[],"arguments":["/home/anna/Invoc",null,null]}}}
{"format":1,"asks":{"read":{"verb":"rename_file","given":[{"named":"Invoc"}],"arguments":["/home/anna/Invoc"]}}}
{"format":1,"asks":{"read":{"verb":"move_file","given":[{"named":"Invoc"}],"arguments":["/home/anna/Invoc","/home/anna/Archive"]}}}
{"format":1,"asks":{"read":{"verb":"open_application","given":[],"arguments":["Firefox"]}}}
{"format":1,"asks":{"read":{"verb":"focus_applicatio
----- end

----- round 2, open: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"open_application","given":["org.alo.Writ<unk>er"]}}}
----- end

----- round 2, focus: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"arrange_applicatio","given":[{"named":"org.alo.Writ
----- end

----- round 2, close: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"close_applicatio
----- end

----- round 2, arrange: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"arrange_applicaton","given":[{"named":"org.alo.Writ
----- end
```

### In the envelope, verbatim

```text

----- round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoicnes"}]}},"format":1}
----- end

----- round 1, read: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoicers/"}]}},"format":1}
----- end

----- round 1, find: ArgumentRefused(Missing { verb: "find_in_folder", argument: "named", purpose: Key { named: "files.verb.find-in-folder.argument.named" } })
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoicnes"}]}},"format":1}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"rename_file","given":["/home/anna/Invoicers/scan001.pdf","march"]}},"format":1}
----- end

----- round 1, move: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"move_file","given":[{"named":"anna"},{"named":"Invoic"},{"named":"march.pdf"}],"path":"/home/anna/Invoics","new_name":"Archive/march.pdf"}},
"format":1}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":[{"named":"/home/anna/Invoicés"}]}},"format":1}
----- end

----- round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"open_application","given":["org.alo.Writer"]}},"format":1}
----- end

----- round 1, focus: NoSuchVerb { named: "focus_applicatio" }
{"asks":{"propose":{"verb":"focus_applicatio","given":[{"named":"org.alo.Writer","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"close","given":[{"named":"org.alo.Writ<caret>er"}]}},"format":1}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"arrange_applicaton","given":[{"named":"org.alo.Writ"}],"arguments":{"where":"left_half"}}},"format":1}
----- end

----- round 2, list: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoicers"}]}},

"format":1}
----- end

----- round 2, read: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoicers/"}]}},"format":1}
----- end

----- round 2, find: ArgumentRefused(Missing { verb: "find_in_folder", argument: "named", purpose: Key { named: "files.verb.find-in-folder.argument.named" } })
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoic"}]}},"format":1}
----- end

----- round 2, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"rename_file","given":["/home/anna/Invoicers/scan001.pdf"]}},"format":1}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"move_file","given":["/home/anna/Invoicers/march.pdf","/home/anna/Archive"]}},"format":1}
----- end

----- round 2, archive: ArgumentRefused(NoSuchArgument { verb: "archive_folder", argument: "/home/anna/Invoic" })
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"/home/anna/Invoic","is":"/home/anna/Invoic/"}]}},"format":1}
----- end

----- round 2, open: ArgumentRefused(NoSuchArgument { verb: "open_application", argument: "org.alo.Writer" })
{"asks":{"propose":{"verb":"open_application","given":[{"named":"org.alo.Writer","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"focus_applicatio"}},"format":1}
----- end

----- round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_applicaiton","given":["org.alo.Writer"]}},"format":1}
----- end

----- round 2, arrange: NoSuchVerb { named: "arrange_applicaton" }
{"asks":{"propose":{"verb":"arrange_applicaton","given":[{"named":"org.alo.Writer","is":"left_half"}]}},"format":1}
----- end
```

## What was built

- **`crates/alo-models/src/chat_template.rs`**: `ChatTemplate { text,
  published_at }`, refused without `{{ .Prompt }}` or without an `https://`
  address its publisher published it at; the loader refuses one beside no
  artefact.
- **`Ollama::fetch`** pulls the artefact and names it for the catalogue.
- **Tests:**
  - the artefact is what is pulled;
  - the create names the id `from` the artefact, with no template for Mistral;
  - Teuken's create carries exactly the catalogue's template, and Teuken is the
    only entry with one;
  - the ignored `a_catalogue_entry_fetched_answers_by_its_catalogue_id` walks
    the real runtime.
- **Tests moved with the measurement.** Three tests froze *Teuken has no grade*
  as a fact. They now hold the rule behind it: a grade on that entry was earned
  against the file it names, with a run's counts beside it.

## Egress

- `registry.ollama.ai`: one refused manifest lookup, 01:32:21 UTC.
- `hf.co`: re-checking the two artefacts' manifests during the walked fetches,
  01:34:32–01:34:45 UTC. Both artefacts were already on disk.
- `huggingface.co`: reading openGPT-X's `tokenizer_config.json`, `README.md`,
  `gptx_tokenizer.py`, `special_tokens_map.json` and the model's API record.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root: formatting, clippy with warnings denied, rustdoc, the supervisor's
three and both BPF gates pass. The workspace's tests, run whole with
`--no-fail-fast`: **4,450 passed, 1 failed, 26 ignored** — the one is
`alo-bounding`'s `ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`,
the aarch64 failure recorded in the first task's report and `docs/quirks.md`.
The publish script re-runs all nine on the tree combined with `main` and pushes
only if the result is this one.
