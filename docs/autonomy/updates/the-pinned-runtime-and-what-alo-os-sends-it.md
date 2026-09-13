# The pinned runtime, and what alo OS sends it

**Date:** 2026-09-13
**Workstream:** v0.5 — the models, measured
**Task:** *Does the pinned runtime accept what alo OS sends it*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 3)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, running
**Ollama 0.34.0** — the release `image/Containerfile` pins, its macOS build
checked against the project's published digest — on `127.0.0.1:11434`. Gates in
Ubuntu 24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

*Point alo OS at weights you already have and it runs them* did not work against
the model runtime alo OS actually ships: the runtime refused the request alo OS
used to hand it a file. It does now — measured on the real runtime, end to end:
a file on this disk handed over, a question answered by its name, and the
listing read back. The runtime's version is written down in one place, and a
test fails if the image ever installs a different one.

## The three requests, and what the runtime answered

**1. Bringing a file.** Refused as it was sent, and fixed.

- Sent, before: `POST /api/create`
  `{"model":"alo-brought-probe","modelfile":"FROM <path>.gguf","stream":false}`
- Answered: **`HTTP/1.1 400 Bad Request` `{"error":"neither 'from' or 'files' was specified"}`**
- Sent, now: `HEAD /api/blobs/sha256:<digest>` → `200` (held) or `404`; if
  `404`, `POST` the file to the same address → `201`; then `POST /api/create`
  `{"model":<id>,"files":{<file name>:"sha256:<digest>"},"stream":false}` →
  **`200 {"status":"success"}`**.

Every road tried, with each answer verbatim, is in `docs/quirks.md` under *The
pinned runtime refuses the Modelfile a brought file was handed over with* —
including `from` naming a path (`400 {"error":"invalid model name"}`), which is
the reassuring one: no create spelling turns this road into a download.

**2. A question by the id the door taught.** Accepted as sent.
`POST /api/chat` `{"model":"alo-brought-files","stream":false,"messages":[{"role":"user","content":"Answer with the single word: ready."}]}`
→ `{"model":"alo-brought-files","content":"Ready.","done_reason":"stop"}` (the
fields shown). The full reply for `qwen2.5:7b-instruct-q4_K_M`, verbatim, is now a
fixture: `the_answer_ollama_0_34_0_sends_is_read`.

**3. The listing.** Accepted and read as the fixtures assumed. A real `/api/tags`
entry from 0.34.0 carries `model`, `modified_at`, `digest`, `details.format`,
`family`, `families`, `parameter_size`, `context_length`, `embedding_length` and
`capabilities` beside the three fields this crate reads; unknown fields are
ignored rather than refused, so nothing broke. The entry is now a fixture,
verbatim: `the_listing_ollama_0_34_0_sends_is_read`.

**The road walked end to end, through the crate's own code:**
`the_pinned_runtime_accepts_what_alo_os_sends.rs`, run with
`ALO_BROUGHT_FILE` naming a real 4.7 GB GGUF on this disk:

```text
test the_image_pins_the_runtime_these_requests_were_measured_against ... ok
brought /Users/disanssebowabasalidde/dev/alo-vm/brought/qwen-brought.gguf as qwen-brought.gguf
answered: "Ready."
listed: Installed { id: "qwen-brought.gguf", bytes_on_disk: 4683074103, quantisation: Some("Q4_K_M") }
test the_three_requests_put_to_the_real_runtime ... ok
```

It checks `/api/version` is `0.34.0` before anything else, and removes the model
it made afterwards. That file's bytes were already in the runtime's store (the
same weights were fetched for task 1), so this run took the `HEAD 200` branch;
the upload branch — `404`, `POST`, `201`, and the digest-mismatch refusal — was
walked by hand against the same runtime with a file it did not hold, and is in
the quirks table.

## What was built

- **`crates/alo-models/src/handing_over.rs`**: the file's SHA-256 (through
  `ring`, already compiled for every build via `rustls`; the root `Cargo.toml`
  comment now says `alo-models` reaches it, and only from this file), the blob
  hand-over that sends bytes only when the runtime does not hold them, and the
  create body. Tests: the digest against published SHA-256 vectors, a vanished
  file refused naming its path, a create that carries nothing but the file and
  never `from`.
- **`bring`** uses it. Its unit tests now assert the three requests, that held
  bytes are not sent again, that a create the runtime refuses and bytes it will
  not store both come back `Unusable` with nothing made, and the path refusals
  that were there before.
- **`crates/alo-models/src/pinned.rs`**: `THE_PINNED_RUNTIME = "0.34.0"`, and the
  test that it equals `image/Containerfile`'s `ARG THE_RUNTIME`.
- **`testing::serving_each`**: a fixture that answers each request with its own
  status, because *not held*, *taken* and *created* are the point.

## Egress

None. The file brought is one already on this disk; nothing was fetched.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root. The first run refused this task twice, and both were looked at:

- **clippy** denied a `panic!` in the new integration test — this task's
  defect, fixed.
- **the workspace's tests** had, beside the known aarch64 failure, one more:
  `alo-bounding`'s `a_directory_opened_before_the_turn_began_is_not_a_key_to_what_is_in_it`.
  Run alone on the same tree it passed twice, 2 of 2 each time, so it is the
  transient under contention `tools/kernel-loop/src/gates.rs` runs a gate a
  second time for, in a crate this task does not touch.

The publish script then ran all nine again on the tree combined with `main`, and
published only because the result was eight passing and the workspace's tests
failing on `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` alone —
the aarch64 failure recorded in the first task's report and `docs/quirks.md`.
Every test in `alo-models`, including the new ones, passes.

## What this does not claim

- Nothing about the runtime on Linux or on the certified machine: the same
  release, a different build. The API is the same program's; the report says
  which build answered.
- Nothing about Teuken: its missing chat template (`docs/quirks.md`) is a fact
  about a file and this runtime, and choosing a template for it is a decision
  with a name on it that this task does not make.
