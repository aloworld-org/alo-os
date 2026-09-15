# A file that did not read is not written over by the next click

- **Date:** 2026-09-15
- **Workstream:** v0.5 — where a person's settings are kept
  (`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 5)
- **Contributor:** Claude (development worker, `C:\dev\alo-os-b`)
- **Status:** ready for integration

## What changed, for a person

If you edit `appearance.toml`, `dock.toml` or `shortcuts.toml` by hand and
make a mistake, alo OS already refused the whole file at sign-in and showed the
release's settings. Until now, the next thing you changed in Settings then
replaced your file, and the edit you were about to fix was gone. Now the change
is refused instead: your file stays exactly as you left it, and you are told
which file could not be read and that you can correct it or put that section
back as alo OS ships it. Putting a section back is the one action that replaces
such a file.

## What changed, in the code

- `crates/alo-kept`
  - `Unwritten::OverAFileThatDidNotRead(Unread)` — the new refusal, carrying
    why the file did not read.
  - `src/replacing.rs` (new) — `Over::{WhatReads, Anything}`: whether the file
    at a path may be written over, asked of the file as it is on the disk.
  - `src/writing.rs` — `keep` writes with `Over::WhatReads`; the new
    `put_back_as_shipped::<K>(at)` writes `K::untouched()` with
    `Over::Anything`. Both go through one private `written`. The check is the
    last thing asked inside `disk::kept`'s read-back closure, i.e. **after the
    text is staged and read back, immediately before the rename**.
  - `src/reading.rs` — `read`'s body became `pub(crate) as_it_is`, returning
    `Unread`, so the reader and the check are one reading of a file.
  - `tests/a_file_that_did_not_read_is_not_written_over.rs` (new).
- `crates/alo-appearance`, `crates/alo-dock`, `crates/alo-shortcuts`, each:
  - `keeping::put_back_as_shipped(at)`, and `keep`'s rustdoc and the module
    header say a file that did not read is not written over.
  - `FileNotWritten::word` maps the new refusal to `KEPT_NOT_REPLACED`
    (`<area>.kept.not-replaced`), and `FileNotWritten::did_not_read()` answers
    the `FileNotRead` for the file as it was at the write, so a surface can say
    what to mend.
  - `words.rs`: `KEPT_NOT_REPLACED`, with a translator's note, in `EVERY_WORD`
    (so `alo-saying` collects it) and in the gap test.
  - `tests/*_kept_in_*_own_file.rs`: three real-file tests (refused with bytes
    unchanged; a file mended since sign-in takes the change; putting back
    replaces), and the new word in the collected-vocabulary test.
  - `tests/the_contract_describes_this_file.rs`:
    `the_contract_says_a_file_that_did_not_read_is_not_written_over`.
- `crates/alo-choosing/tests/every_sentence_about_a_persons_settings_carries_a_note.rs`
  — each kept file now has eight sentences, not seven.
- `docs/contracts/person-settings.md` — the three *Writing it* sections say
  what the crates now do, replacing the sentences that said the rule was not
  yet held.
- The plan: task 5 marked done; task 6 written (an end-to-end walk of one
  person's folder, and a short section naming the calls for the shell).

## Decisions

- **The rule lives in `alo-kept`, not in each crate.** Every `Kept` shape gets
  it through `keep`, and no crate can write a value over a broken file without
  going through the one door that takes no value. That is what "carried in
  type" means here.
- **The door takes no value.** `put_back_as_shipped` can only write
  `Kept::untouched()`, so it cannot be used to write a change around the check.
  It writes the format line rather than deleting the file: that is exactly what
  `keep(untouched)` has always left, it reads as *changed nothing*, and it goes
  through the same whole, read-back write.
- **`keep` of the untouched value is also refused over a broken file.** The
  plan says the put-back door is *the only one* that replaces such a file; a
  surface that builds "reset" out of `keep(&Changes::untouched())` gets the
  refusal and should call the door.
- **Checked immediately before the rename, not before staging.** The plan asks
  for the file *as it is at the moment of the write*. Asking last narrows the
  window to the rename. It is not atomic against another writer editing the
  file in that instant; there is no portable compare-and-rename, and ADR 0038's
  one-writer-per-file plus an editor's save racing a click by microseconds is
  not worth an unsafe or platform-specific lock. Stated here rather than hidden.
- **What counts as "did not read" is exactly what `read` refuses**, including
  another `format` (so an older alo OS does not overwrite a newer one's file),
  bytes that are not text, and a file the disk will not hand over (a
  permission, or a folder at the path). A missing file or one that reads is
  written as before.
- **One new sentence per crate, naming the file**, rather than one per reason:
  the reason is already a sentence (`did_not_read().said(..)`), and a surface
  can show both.

## Acceptance criteria → evidence

| Criterion | Test |
|---|---|
| `alo-kept` carries the rule in type; refused over a file that did not read, bytes unchanged | `alo-kept` `a_file_that_did_not_read_is_not_written_over::a_change_is_refused_over_a_file_that_did_not_read_and_the_bytes_stay` |
| Per crate, against a real file, in the crate's words naming the file, bytes unchanged | `alo-appearance` / `alo-dock` / `alo-shortcuts` `*_kept_in_*_own_file::a_change_is_not_written_over_a_file_that_did_not_read` |
| Putting back as shipped is the distinct door that replaces such a file | `alo-kept` `…::putting_back_as_shipped_replaces_a_file_that_did_not_read`, and the same name per crate |
| A missing or reading file is written as today; checked at the write, not at sign-in | `alo-kept` `…::a_file_that_is_not_there_or_that_reads_is_written_as_before`, `…::a_file_fixed_since_it_was_refused_takes_the_next_change`; per crate `a_file_mended_since_sign_in_takes_the_next_change` |
| Every new sentence has a note and is collected by `alo-saying` | `alo-choosing` `every_sentence_about_a_persons_settings_carries_a_note::every_sentence_about_a_file_names_which_file` |
| The three *Writing it* sections describe it, held per crate | per crate `the_contract_describes_this_file::the_contract_says_a_file_that_did_not_read_is_not_written_over` |

A mutation run (making `keep` use `Over::Anything`) failed
`alo-appearance`'s `a_change_is_not_written_over_a_file_that_did_not_read`
before cargo stopped; the change was reverted.

## Verification

Run in WSL Ubuntu against `/mnt/c/dev/alo-os-b`, target
`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo test -p alo-kept -p alo-appearance -p alo-dock -p alo-shortcuts -p alo-choosing`
  (each crate separately) — all pass.

Not run: the full workspace suite (the supervisor runs it). Nothing here
touches the machine, so there is no hardware acceptance.

## Limitations

- No surface calls `put_back_as_shipped` yet; the shell plan's Settings task
  owns the button. Its label is not declared here — the sentence tells the
  person the option exists, and the shell declares the control's words.
- The check-then-rename window described above.

## Proposed shared-document updates

- **CHANGELOG.md:** "Settings no longer replaces a settings file you edited by
  hand when it could not be read; your next change is refused and names the
  file, and putting that section back as shipped is the one way to replace it."
- **QUEUE.md / STATE.md:** plan *where a person's settings are kept*, task 5
  done; task 6 (one person's folder from sign-in to the next change) ready.
- **ROADMAP.md:** no gate ticked; *Settings, as one place* still waits on the
  shell's surface.
