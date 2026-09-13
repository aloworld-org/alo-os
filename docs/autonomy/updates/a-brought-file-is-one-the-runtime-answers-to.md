# A brought file is one the runtime answers to

- Date: 2026-09-13
- Workstream: v0.5 lane B, task 5 (`docs/autonomy/v0-5-lane-b-plan.md`)
- Contributor: Claude Code, by hand — no worker could run
- Status: **the last word of the promise, and the first place it could have become a download**

`docs/features.md` promises, at v0.5: *point alo OS at weights you already have
**and it runs them**.* Task 4 built the first half — a file is chosen, costed
and written into the person's own settings as a model with no catalogue entry,
its licence theirs. The last three words were still missing. `ModelRuntime` is
asked by id, and the runtime knows only what it already lists, so the first
question put to a `.gguf` somebody pointed at failed as *no model there*:
honest, and not the promise.

## What changed

`ModelRuntime` gains one door, `bring`, taking the `Weights` a person's
settings carry. For the pinned runtime that is `/api/create` with a Modelfile
of exactly one line — `FROM <the file's own path>` — and nothing else in it.
No template, no system prompt, no parameters: anything further would be this
machine putting words in a model somebody else brought.

It is called **after** `Choosing::bringing_a_file` has succeeded, never instead
of it, so a runtime that is down costs a person nothing they typed. Their file
stays chosen and telling the runtime can be done again.

## The refusal that matters most

**To the runtime, a bare name is an instruction to fetch from a publisher.**
`FROM mistral` downloads. So *run the weights you already have* had exactly one
way to become an egress nobody asked for, and the door refuses it before
anything is sent: a path that is not absolute and not a file on this disk is
`NotAPathOnThisDisk`, checked here because once the runtime holds the Modelfile
it is the runtime deciding.

The door deliberately does **not** lean on `Weights::at` having checked the
file when it was chosen. These weights come back out of a settings file, where
the path may since have been deleted or the line edited by hand, and a test
names that reason.

Weights that name no file at all are `NothingToBring` — a refusal about which
door was used rather than about anything the person did.

## What it is measured against, and what it is not

Every test here runs against a socket this repository opens and answers on,
which is how every adapter test in `alo-models` is written. They show that the
**request alo OS sends is the right one** and that an answer comes back through
it:

| Acceptance | Test |
|---|---|
| one `FROM`, the file's own path, nothing else | `bringing_a_file_tells_the_runtime_its_path_and_nothing_else` |
| weights with no file refused at the door | `weights_with_no_file_are_refused_without_asking_the_runtime` |
| a bare or relative name refused before anything is sent | `a_path_that_is_not_a_file_on_this_disk_is_refused_before_anything_is_sent` |
| the runtime's own refusal carried, not reworded | `a_file_the_runtime_will_not_take_is_carried_as_a_refusal` |
| told, then answering by that id, over one socket | `a_brought_file_is_told_to_the_runtime_and_then_answers_by_its_own_name` |
| nothing goes near a download | `what_the_runtime_is_told_names_a_path_on_this_disk_and_asks_for_no_download` |
| a runtime that is down costs nothing typed | `a_runtime_that_is_down_costs_the_person_nothing_they_typed` |

**What no test here shows is that the pinned runtime accepts that request.**
Ollama 0.34.0 is on the image (ADR 0025) and not on this development machine,
so whether its `/api/create` takes this exact body is owed to a machine with
the runtime on it — the VM that booted on 2026-09-12 is where that is answered,
and it is not answered yet. Nothing in this change claims otherwise.

## Two things the compiler and the linter caught, worth recording

**A new refusal is a question everywhere failures become sentences.**
`alo-asking`'s `what_went_wrong` stopped compiling until both new variants were
mapped, which is the boundary working: neither can arrive on that road — asking
a question never opens this door — and if one ever did, what is true for the
person is *no model there* rather than *something unusable came back*.

**A substring assertion that would have passed on Linux and failed on Windows.**
The first version of the main test searched the request for `FROM <path>`; a
path is JSON-escaped on the way out, so `C:\Users\…` becomes `C:\\Users\\…` and
the match fails for a request that is correct. The body is parsed now rather
than searched. Third time this week the platform split has produced a red test
for green code.

## Verified

Linux, under WSL2, in the loop's own build directory, after
`cargo fmt --all --check` clean and `cargo clippy --workspace --all-targets --
-D warnings` clean over the workspace:

| Crate | Result |
|---|---|
| `alo-models` | 227 passed, 0 failed |
| `alo-saying` | 68 passed, 0 failed |
| `alo-asking` | 126 passed, 0 failed |
| `alo-turn` | 85 passed, 0 failed |
| `alo-agentd` | 268 passed, 0 failed |
| `alo-choosing` | 175 passed, 0 failed |

Not executed: the full workspace suite, which is the supervisor's; nothing on a
certified machine, and no *On the machine* box moves.

**CHANGELOG.md** — nothing user-visible yet; there is no surface that calls this
door. **ROADMAP.md** — the v0.5 line *run a model we never catalogued* is now
built rather than partly built; no tick moves on the machine.
**QUEUE.md/STATE.md** — a brought file is told to the runtime after it is
chosen.
