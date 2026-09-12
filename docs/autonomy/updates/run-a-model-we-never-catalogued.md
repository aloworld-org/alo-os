# Run a model we never catalogued — the catalogue recommends, it does not gate

**Date:** 2026-09-12
**Workstream:** v0.5, lane B — providers and models, task 4 (*Run a model we
never catalogued — the catalogue recommends, it does not gate*)
**Contributor:** Claude (`C:\dev\alo-os-b`, kernel-loop worker)
**Status:** ready for integration

## What was wrong

Three sentences from `docs/features.md`, v0.5: *point alo OS at weights you
already have and it runs them; the catalogue recommends, it does not gate*;
*a model too large for the memory in this laptop is said so plainly, once —
and then run anyway*; *what you bring is yours, including its licence, and
alo OS does not pretend to have checked it.*

Most of the shape existed. `alo_models::Weights` is a model with no catalogue
entry and no licence field; `alo_models::Cost` warns and cannot refuse;
`alo_models::Brought` is the person's own list; `alo_choosing::Choosing::bringing`
writes an entry to the person's file; `Weights::lines` says the cost and whose
licence it is. What was missing:

- **No way to point at a file.** `Weights::checked` takes an id and a size
  somebody states; `Weights::found` takes what a runtime already lists. A
  person with a `.gguf` on their disk had no door that took the path and
  measured it — the size was typed or nothing.
- **Once was a sentence, not a mechanism.** *Said so plainly, once* was a
  rustdoc comment on `Weights::lines`. Nothing remembered that a person had
  been told, so a surface costing the chosen weights at every question would
  have repeated the sentence all afternoon — the nag `alo-telling` exists to
  prevent, for a different failure.
- **The measurement was not said.** `drives_verbs` for brought weights is
  `NotMeasured`, and the two lines a person read said nothing about it: a
  person could reasonably think alo OS had graded their file.
- **Nothing held the sentences to the promise.** No test read the sentences
  shown beside somebody's own weights for a lean toward the catalogue or away
  from their own.

## What changed

- `crates/alo-models/src/weights.rs` — `Weights::at(&Path)`: the size is
  `std::fs::metadata`'s answer for the file at that moment, the id is the
  file's own name exactly as the disk spells it, `file` is remembered, and
  `drives_verbs` is `NotMeasured`. Nothing opens the file, parses a header, or
  looks the name up in the catalogue. Three new refusals — `NoFileThere`,
  `NotAFile`, `FileNotRead` — each naming the path, with the machine's own
  reason kept beside `FileNotRead` and out of the sentence. `Weights` gains
  `file: Option<PathBuf>` (serde default, skipped when absent). `lines` is now
  three lines: cost, licence, measurement; `measurement` is the third alone.
- `crates/alo-models/src/words.rs` — five strings: the three path refusals,
  `models.brought.not-measured` and `models.brought.measured`. `EVERY_WORD`
  grows to 47. Two public lists: `ABOUT_BROUGHT_WEIGHTS`, every sentence shown
  beside brought weights, held by a test to be exactly the `models.brought.`
  area; and `NUDGES`, fourteen ways a sentence leans. A test reads every
  sentence *and its note* against the list, and refuses `catalogue` in any
  sentence a person reads there.
- `crates/alo-models/tests/a_model_we_never_catalogued.rs` — new: a real file
  is offered beside the catalogue with no entry in it and its size measured;
  a file whose name matches a catalogued entry inherits neither the entry's
  grade nor its licence, and the catalogue is byte for byte the same before
  and after; the three lines in German, translated whole; the path refusals
  named in German with the path carried through.
- `crates/alo-models/tests/what_this_crate_says.rs` — the existing weights
  test reads three lines instead of two.
- `crates/alo-choosing/src/choosing.rs` — `Choosing::bringing_a_file(&Path)`:
  `Weights::at` and then `bringing`, every refusal in `alo-models`' words,
  nothing written in any of them. Bringing is not choosing.
- `crates/alo-choosing/src/written.rs`, `writing.rs` — `[[brought]]` gains an
  optional `file` key, both directions. Additive; the format number stays 3.
  Not measured again on the way in, and the reader's `not_weights` names the
  three path refusals rather than wildcarding them.
- `crates/alo-choosing/tests/a_model_we_never_catalogued.rs` — new, against the
  vocabulary `alo-saying` collects: a file named as a model source and read
  back through the daemon's door with the path in the file; no catalogue entry
  and a licence that is theirs; a file larger than the machine chosen, written,
  and asked under no bound and under *this machine only*; a missing path and a
  folder refused naming the path with the settings byte for byte what they
  were; every brought sentence in the machine's vocabulary and none nudging;
  `drives-verbs = "not-measured"` in the file and the sentence saying nobody
  measured and nothing was guessed.
- `crates/alo-telling/src/too_large.rs`, `warned_once.rs`, `warning.rs` — new.
  `TooLarge` is the identity of a warning (which weights, what they cost on
  this machine), `None` for weights that fit. `WarnedOnce` is the two lines a
  person reads — `alo-models`' cost line and this crate's *that is said once:
  alo OS will run these weights whenever you choose them, and will not raise
  their size again by itself*. `Warning` is the session memory, beside
  `Telling` and built the same way: `Warn::Say`, `Warn::SaidAlready`,
  `Warn::Fits`; the person asking again never suppressed; bounded oldest-first
  by the same constant; no `Err` anywhere in it.
- `crates/alo-telling/src/words.rs`, `lib.rs`, `testing.rs` — one string,
  `telling.runs-them-anyway`; `EVERY_WORD` grows to 3; the agent-naming test
  now applies to the two lines that name the agent; a test holds the new line
  to `alo_models::words::NUDGES`; the crate doc gains the section and example.
- `crates/alo-telling/tests/a_model_too_large_for_this_machine_is_said_so_once.rs`
  — new, against the machine's vocabulary: said once across a hundred machine
  retries; nothing refuses and weights that fit are `Fits`; the person asking,
  other weights, or another memory figure are said; every line collected and
  none nudging.
- `docs/contracts/person-settings.md` — the `file` key, additive, and the two
  refusals at the write.
- `docs/autonomy/v0-5-lane-b-plan.md` — task 4 marked done; task 5 written
  (below).

### User-readable change description

You can point alo OS at a model file you already have on this machine. It
measures the file's size off the disk rather than asking you for a number,
keeps the file's own name, and puts it on your list beside the catalogue — with
no catalogue entry, no licence alo OS claims to have read, and no grade
guessed for it. If the file is larger than this machine's memory you are told
so once, in a sentence that also says alo OS will run it anyway; alo OS does
not bring it up again by itself. Pointing at a path with nothing there, or at
a folder, is refused in a sentence naming the path, and your settings are
untouched. Nothing about the catalogue's own entries changes.

## Decisions

- **The id is the file's own name.** The alternative — asking the person to
  name it — invents a name nothing else on the machine answers to. A runtime
  told about the file (task 5) is told under this name.
- **Measured once, when the person points.** The settings reader does not
  re-measure files its entries name: a drive not mounted this morning is not a
  settings file that is wrong, and a reader reaching out to every file it
  names is a reader with a second reason to fail.
- **`Warning` beside `Telling`, not inside it.** Two memories with one shape,
  one file each. A telling is made from an `alo_answering::Failed`; a warning
  from `Weights` and a memory figure. One `VecDeque` holding both would be a
  file with two reasons to change (law 4). What they share is `WhoAsked` and
  the bound.
- **The memory figure is part of the identity.** A laptop docked into more
  memory has changed the thing the sentence was about; a warning that ignored
  it would repeat a sentence that is now the same or swallow one that is now
  different.
- **`Warn::Fits` is a variant, not `None`.** A caller holding a `Warn` has
  nothing to put an `if` in front of except whether to draw two lines, and
  three named answers say which of three things happened.
- **The warning says nothing about licence or measurement.** Those are
  `Weights::lines`', at the moment of adding, where the person is deciding. A
  warning about size that repeated them would be arguing with somebody about a
  model they have chosen.
- **Machine memory is the caller's.** Nothing in the workspace measures it
  yet (`alo-image` carries the certified figure as a constant). `Cost::of` and
  `Warning::about` take gigabytes, as `Weights::costs_on` always has.
- **`NUDGES` lives in `alo-models`** so `alo-telling`'s line is held to the
  same list as the lines it is read beside, rather than to a copy.
- **Task 5 is the runtime.** The pinned runtime is asked by id and lists only
  what `/api/tags` knows, so a brought file is chosen, costed and asked, and
  the first question fails as *no model there* until the runtime is told about
  the file. That is honest and it is not the last word of the promise; it is
  written as task 5 rather than done here because it is a runtime adapter
  change against the crate's serving fixture, and this task's acceptance is
  already whole.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| A weights file on this machine is named as a model source through `alo-choosing`'s own shapes | `alo-choosing` `a_model_we_never_catalogued::a_weights_file_on_this_machine_is_named_as_a_model_source_and_read_back`; `alo-choosing` lib `choosing::tests::a_weights_file_pointed_at_is_brought_measured_and_can_then_be_chosen` |
| Offered by `alo-models` with no catalogue entry, licence *yours*, size measured off the file | `alo-models` `a_model_we_never_catalogued::a_file_is_offered_beside_the_catalogue_with_no_entry_in_it_and_its_size_measured`; `alo-choosing` `a_model_we_never_catalogued::what_is_offered_has_no_catalogue_entry_a_licence_that_is_theirs_and_a_measured_size`; `alo-models` lib `weights::tests::a_file_on_this_machine_is_brought_and_measured_off_the_disk` |
| A file larger than the machine's memory is told once through `alo-telling`, and run if still chosen | `alo-telling` `a_model_too_large_for_this_machine_is_said_so_once::weights_larger_than_this_machines_memory_are_said_so_once`; `alo-telling` `a_model_too_large_for_this_machine_is_said_so_once::nothing_here_refuses_and_weights_that_fit_are_nothing_to_say`; `alo-telling` `a_model_too_large_for_this_machine_is_said_so_once::saying_it_again_takes_the_person_asking_or_something_changing`; `alo-choosing` `a_model_we_never_catalogued::a_file_larger_than_this_machines_memory_is_still_chosen_and_still_asked` |
| Nothing about the catalogue's own entries changes | `alo-models` `a_model_we_never_catalogued::bringing_a_file_changes_nothing_about_the_catalogues_own_entries` |
| Every sentence is in the vocabulary `alo-saying` collects, and none nudges | `alo-choosing` `a_model_we_never_catalogued::every_sentence_about_their_own_weights_is_in_the_machines_vocabulary_and_none_nudges`; `alo-telling` `a_model_too_large_for_this_machine_is_said_so_once::every_line_is_in_the_vocabulary_this_machine_collects_and_none_nudges`; `alo-models` lib `words::tests::nothing_said_beside_brought_weights_nudges_toward_the_catalogue_or_away_from_their_own` |
| `drives_verbs` for a brought file is not measured, and the sentence says so | `alo-choosing` `a_model_we_never_catalogued::what_a_brought_file_drives_is_not_measured_and_the_sentence_says_so`; `alo-models` lib `weights::tests::the_measurement_line_says_whether_a_measurement_was_run_and_no_more` |
| Refusal: a path that is not a weights file is refused naming the path, and nothing is written | `alo-choosing` `a_model_we_never_catalogued::a_path_that_is_not_a_weights_file_is_refused_and_the_settings_are_byte_for_byte_what_they_were`; `alo-models` `a_model_we_never_catalogued::a_path_that_is_not_a_file_is_refused_in_words_naming_the_path`; `alo-models` lib `weights::tests::a_path_that_is_not_a_file_is_refused_naming_the_path` |

**Constraints held:** nothing downloads anything — no test opens a socket and
`Weights::at` calls `metadata` only. Nothing in `image/` or `crates/alo-shell`
was touched. The catalogue's grade and licence are not guessed for a file:
`bringing_a_file_changes_nothing_about_the_catalogues_own_entries` points at a
file named after a catalogued entry that carries a grade and asserts the file
did not inherit it.

## Verification

Run in WSL Ubuntu against `/mnt/c/dev/alo-os-b`, target directory
`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`:

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-models -p alo-choosing -p alo-telling
cargo test -p alo-models
cargo test -p alo-choosing
cargo test -p alo-telling
```

Results are in the section at the end of this report, filled in from the run
before the handoff was written. The full workspace suite was not run here; the
supervisor runs it before publication.

## Limitations

- **The runtime does not yet answer to a brought file.** See task 5 in the
  plan. Until it lands, a person who brings a file, chooses it and asks a
  question is told *no model there* once, through `alo-telling`, and the
  machine carries on. Nothing is refused and nothing is guessed; the last word
  of the promise is not yet true.
- Nothing physical: this task touches no device.
- Machine memory is not measured anywhere in the workspace; the figure is
  passed in by whoever draws the panel, as it always was.

## Proposed shared-document updates

- `CHANGELOG.md`, unreleased: the user-readable change description above.
- `docs/autonomy/QUEUE.md` / `STATE.md`: lane B task 4 done; task 5 (*A
  brought file is one the runtime answers to*) is next and depends on 4.
- `ROADMAP.md`, v0.5: the three `docs/features.md` lines may be ticked
  *built* per ADR 0028, with the runtime caveat above noted until task 5.

## Results

Run 2026-09-12 on the finished tree (WSL Ubuntu, target
`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`):

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-models -p alo-choosing -p alo-telling` | clean, exit 0 |
| `cargo test -p alo-models` | all targets pass (lib 165, `a_model_we_never_catalogued` 4, the rest unchanged) |
| `cargo test -p alo-choosing` | all targets pass (lib 128 including three new, `a_model_we_never_catalogued` 6, the rest unchanged) |
| `cargo test -p alo-telling` | all targets pass (lib 53 including the three new files' tests, `a_model_too_large_for_this_machine_is_said_so_once` 4, doctests) |

Not run here: the full workspace suite (the supervisor runs it), and anything
physical (nothing in this task touches a device).
