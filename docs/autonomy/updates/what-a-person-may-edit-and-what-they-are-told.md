# What a person may edit, and what they are told when it did not read

**Date:** 2026-09-15
**Workstream:** v0.5 — where a person's settings are kept
(`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 4)
**Contributor:** Claude (worker in `C:\dev\alo-os-b`)
**Status:** ready for integration

## What changed

`appearance.toml`, `dock.toml` and `shortcuts.toml` sit in a person's own
folder, so a person will open one in an editor and a portal will one day read
one. Until this change the contract listed their keys in one table row each and
said the shape was *still to be written*.

- **`docs/contracts/person-settings.md`** gains three sections, one per file:
  the keys, the `format` number, the exact file alo OS writes, every value a
  person may type (names matched exactly, ranges, what is refused), what a
  missing file means, what a file that does not read is told — with worked
  examples that are refused and the sentence each is refused with — and how the
  file is written.
- **Each of those sections is held to its crate by a test**:
  `crates/alo-{appearance,dock,shortcuts}/tests/the_contract_describes_this_file.rs`.
  The keys the section lists must be exactly the top-level keys the crate
  writes; the file the section says alo OS writes must be byte for byte the file
  `keeping::keep` puts on a real disk; every ```` ```toml ```` example must read
  from a real file; every ```` ```toml refused ```` example must be refused, in
  the sentence key the prose after it names, naming the key or the line where
  the prose says it does, with `at_sign_in` then drawing the release's
  defaults. A missing file is checked to be *changed nothing* and to write
  nothing.
- **`alo-choosing/tests/no_english_outside_the_vocabulary.rs`** reads the
  shipped source of all five crates — `alo-appearance`, `alo-dock`,
  `alo-shortcuts`, `alo-choosing`, `alo-changing` — for English written outside
  `alo-strings`. It lexes Rust (comments, doc comments, raw strings, escaped
  line continuations, character literals, `#[cfg(test)]` items and the modules
  they declare) and refuses any string literal holding two ordinary words side
  by side, unless it is the vocabulary (`saying`/`noting`/`counting` in a
  crate's `src/words.rs`), a lint's reason, rustdoc, or on a short exception
  list where each entry carries its argument. Stale exceptions are refused too.
  A calibration test shows the search finds every crate's declared vocabulary
  (one `saying` key and sentence per phrase `alo-saying` collects under that
  crate's area), and fixture tests show it finding a sentence in a plain call,
  a raw string, a continued line and an `#[error]`, and not finding comments,
  keys, serde names or marks.
- **`alo-choosing/tests/every_sentence_about_a_persons_settings_carries_a_note.rs`**
  asks the machine's one vocabulary that every phrase and plural under the five
  crates' areas carries a non-empty translator's note; that each
  `<area>.kept.unknown-key` names `{path}` and `{key}` with a note saying
  neither is translated; and that every sentence about one of these files names
  the file.
- **`alo-shortcuts`**: `shortcuts.action.close-window` was the one sentence of
  the five crates without a note. It has one, and the crate's own word tests
  now require a note on every word rather than a chosen list.

### In words a person outside this repository can read

The settings files for how your machine looks, where your dock is and which
shortcuts you changed are now documented key by key, with examples of what reads
and what does not and the exact message you will see for each — and that
documentation is checked against the software on every build, so it cannot
quietly drift. Every message about these files names the file, and the key when
a key was wrong, and every one of them is ready for translation.

## Decisions

- **The contract said something the crate does not do, and the contract
  changed.** `settings.toml`'s sections promised that a key nobody declared is
  refused *naming it*. `alo-choosing` deliberately never repeats an undeclared
  key back (`crates/alo-choosing/src/unreadable.rs`): it is where a pasted
  credential lands, and a refusal quoting it would carry it into logs. The plan
  says the crate is right where the two disagree, so three sentences in the
  `settings.toml` sections now say the file is named and the key is not, and
  why. No crate behaviour changed.
- **Where the scan test lives: `alo-choosing`.** It already dev-depends on
  `alo-saying`, which the calibration test needs, and it owns the person's
  folder. No new dependency was added anywhere.
- **What counts as English.** Two adjacent ordinary words (a letter then
  lower-case letters). A single word cannot be told from an identifier by
  reading; one-word labels are held by each crate's own list tests. The
  heuristic is documented in the test's header.
- **Nine exceptions, each argued.** `alo_changing::NotChanged`'s five
  `#[error]` displays (for a log; the person reads `NotChanged::said`),
  `alo_shortcuts::DefaultsError`'s two (a defect in the release's own list),
  and `alo-choosing`'s two `NotExpressible` details (for whoever fixes alo OS; a
  person reads `choosing.change.not-expressible`). Moving that English into the
  vocabulary would put developer diagnostics in front of translators; removing
  the `Display`s is a change to those crates' error types the plan did not ask
  for.
- **A contract example is a fenced block, and whether it reads is in its info
  string** (`toml` or `toml refused`), so the document and the test agree on
  which is which without a second list.
- **Two appearance contract tests run on Unix only.** The file alo OS writes
  names a picture by `/home/ada/harbour.jpg`, which the crate rightly refuses as
  relative on a Windows host. Every example with no path in it, and all refusal
  examples, run everywhere. The gates run on Linux.
- **Values documented as the crate reads them, including two that surprised
  me:** a display named twice in `displays`, and an action named twice in
  `[[changed]]`, are read as their last entry rather than refused; a shortcut
  clash does not refuse the file but is reported by `Shortcuts::clashes`.
- **A gap found, and written as task 5 rather than fixed here.** ADR 0038
  clause 3 says Settings does not write over a file that did not read. None of
  the three `keeping::keep` functions holds that: they replace whatever is at
  the path. The task's constraint is that nothing here changes what the
  sections describe, so the three *Writing it* sections state the current
  behaviour plainly, and the plan gains task 5, *A file that did not read is
  not written over by the next click*.

## Acceptance criteria

| Criterion | Evidence |
|---|---|
| The contract gains a section per file — keys, `format`, a missing file, a file that does not read — as sections of the existing contract | `alo-appearance`, `alo-dock`, `alo-shortcuts`: `the_contract_describes_this_file` (four tests each) |
| Every sentence these crates can say is in the vocabulary with a translator's note, including the one naming which file and which key | `alo-choosing`: `every_sentence_about_a_persons_settings_carries_a_note` |
| A test reads the shipped source of all five crates for English written outside `alo-strings` | `alo-choosing`: `no_english_outside_the_vocabulary` |

## Verification

All on WSL Ubuntu (the gates' platform), from `/mnt/c/dev/alo-os-b`, with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean, exit 0 |
| `cargo test -p alo-appearance` | all pass (107 unit, and every integration target) |
| `cargo test -p alo-dock` | all pass |
| `cargo test -p alo-shortcuts` | all pass (72 unit, including the new `every_word_carries_a_note`) |
| `cargo test -p alo-choosing` | all pass (136 unit; the two new targets: 3 and 7 tests) |
| Mutation: `edge` renamed `edges` in the dock table, and a refused example's sentence changed | both `alo-dock` contract tests failed naming the difference; reverted |

Not run: the full workspace suite (the supervisor runs it). `alo-changing` was
read and not edited, so its suite was not re-run. Nothing here touches hardware.

## Remaining limitations

- The scan cannot see a one-word English label outside `words.rs`.
- ADR 0038's *not written over* rule is task 5.
- No translations exist in the repository yet; the notes are what makes them
  possible.

## Proposed updates for the integration owner

- **CHANGELOG.md:** "The files for appearance, the dock and shortcuts are
  documented key by key in the person-settings contract, with examples of what
  reads and what is refused and the message for each, checked against the
  software by tests. Every message about them is ready for translation, and a
  test keeps English out of the five settings crates' code."
- **ROADMAP.md / QUEUE.md:** *Settings, as one place* — the plan's task 4 is
  done; task 5 (a file that did not read is not written over) is ready.
