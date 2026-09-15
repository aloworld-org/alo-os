# Appearance, the dock and shortcuts keep their own files

**Date:** 2026-09-15
**Workstream:** v0.5 — where a person's settings are kept
(`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 2,
*Appearance, the dock and shortcuts, each keeping its own*), implementing
[ADR 0038](../../decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
by the rule `crates/alo-kept` holds.
**Contributor:** Claude Code worker, `C:\dev\alo-os-b`.
**Status:** ready for integration.

## What changed

**For a person:** once the shell calls these, a changed background, a moved
dock and a rebound key survive signing out. A file they edited by hand and got
wrong is not half-used: the machine looks the way alo OS ships it, and Settings
has a sentence naming the file — and the word they typed, when that was the
problem — to show them.

**In the code:**

- **`crates/alo-appearance`**
  - `src/keeping.rs` (new): `THE_FILE = "appearance.toml"`, `FORMAT = 1`,
    `impl alo_kept::Kept for Changes` with keys `background`, `displays`,
    `lock`, `following`, `text`, `accent`; `read(at)`, `keep(at, &changes)` and
    `at_sign_in(at) -> (Appearance, Option<FileNotRead>)`.
  - `src/unkept.rs` (new): `FileNotRead` and `FileNotWritten` — the file, the
    reason, `key()` for an unknown key, `word()` and `said(&Strings)`. No
    `Display`.
  - `src/words.rs`: seven sentences under `appearance.kept.*`, each with a
    translator's note and a `{path}` gap (`{line}` and `{key}` where they
    apply). `EVERY_WORD` 28 → 35, so `alo-saying` collects them through the
    existing `declare_into`.
  - `src/picture.rs`, `src/rotating.rs`: **a security fix found on the way.**
    `Picture` and `Rotating` derived `Deserialize` directly, so a hand-edited
    file could name `../../etc/shadow` as a *shipped* wallpaper, or a relative
    picture or folder, bypassing `Picture::shipped`, `Picture::file` and
    `Rotating::folder`. Harmless while nothing read a file; live the moment this
    task made the file real. Both now read back through those constructors
    (`serde(try_from)`), refusing with the key of the same refusal a panel
    shows, and with `deny_unknown_fields`.
  - `src/time.rs`, `src/scheme.rs`: `deny_unknown_fields` on the written shapes.
  - `src/lib.rs`: the sentence *which file it lives in and who writes it is the
    shell's* is gone; *it does not read the clock or the disk* now says the one
    file it keeps; module table lists `keeping` and `unkept`.
  - `tests/appearance_kept_in_its_own_file.rs` (new).
- **`crates/alo-dock`** — the same shape: `src/keeping.rs` (`dock.toml`,
  `FORMAT = 1`, key `edge`, `at_sign_in -> (Dock, …)`), `src/unkept.rs`, seven
  `dock.kept.*` words (`EVERY_WORD` 9 → 16), the header's *it does not read
  anything* corrected, and `tests/dock_kept_in_its_own_file.rs`. `toml` added
  as a dev-dependency for a unit test.
- **`crates/alo-shortcuts`** — `src/keeping.rs` (`shortcuts.toml`,
  `FORMAT = 1`, key `changed`, `at_sign_in -> (Shortcuts, …)`), `src/unkept.rs`,
  seven `shortcuts.kept.*` words (`EVERY_WORD` 38 → 45), the header's *nothing
  here has a side effect* corrected, `deny_unknown_fields` on `Changed` and on
  a chord's written parts, and `tests/shortcuts_kept_in_their_own_file.rs`.
  `toml` added as a dev-dependency.
- **`docs/contracts/person-settings.md`**: a table of the three files — owner,
  `format`, top-level keys — and what a file that did not read is answered
  with. The full shape of each value is still task 4's.
- **Plan:** task 2 marked done.

## Decisions

- **Each crate exposes free functions, not the trait.** `keeping::read`,
  `keeping::keep` and `keeping::at_sign_in` take and give the crate's own
  `Changes`, so the shell never names `alo_kept`. Appearance and the dock
  implement `Kept` on `Changes` directly.
- **Shortcuts sits under one key.** Its `Changes` is a list and a TOML file is a
  table, so the file is `[[changed]]` tables under a private wrapper,
  `InTheFile`. An action with no `chord` is a shortcut the person cleared. That
  is why a misspelt `chrod` must be refused rather than ignored, and why
  `Changed` now denies unknown fields.
- **Seven sentences per crate, not one.** A disk problem, a file that is not
  settings, one that stopped at a line, another format, an unknown key, a write
  the disk refused, and a write alo OS refused. Each sends a person somewhere
  different. A relative path is a caller's fault, never the person's, so it is
  said as *could not be read* on the way in and as a fault in alo OS on the way
  out.
- **A value refused inside the shape (`NotItsShape`) is said generally.** The
  deserialiser's refusals carry a key such as `appearance.picture.name-is-a-path`,
  but those sentences have gaps (the name, the path) the file reader cannot fill
  without quoting the file. Task 4 can decide whether to name them. This task
  names the top-level key, which is what the acceptance asks for.
- **`at_sign_in` returns the shipped value beside the refusal.** It does not
  return an error, because the machine must always draw something, and it
  never returns the half of a file that read. ADR 0038 point 3 (*Settings does
  not write over a file that did not read* unless the person puts the section
  back) belongs to the Settings surface, which holds the refusal this returns.
  It is not enforced here.
- **The contract gets a table now.** The gate says a public surface that moves
  updates its contract in the same change. Task 4 still owns the per-file
  sections, with their full shape and translator notes.

## Acceptance

| Criterion | Held by |
|---|---|
| A keeping module, a file constant, a `format` and reading/writing at a handed path, per crate | the `a_change_written_to_a_real_file_reads_back_as_itself` tests (they use `THE_FILE`, `FORMAT`, `read`, `keep` at a temp path) and `alo-choosing`'s existing `no_crate_that_keeps_a_file_depends_on_this_one_or_reads_an_environment` |
| A value read back from a real file is the value written, per crate | `a_change_written_to_a_real_file_reads_back_as_itself` in each crate's new test file |
| A hand-edited unknown key is refused whole with the key named, and the release is drawn | `a_key_that_is_not_on_the_list_is_refused_whole_and_the_release_is_drawn` in each; plus `a_file_that_is_not_this_format_is_refused_whole`, `a_refused_write_leaves_the_file_as_it_was`, `a_wallpaper_name_that_is_a_path_is_refused_whole`, `a_misspelt_chord_is_not_read_as_a_cleared_shortcut` |
| Each header loses the sentence that said this was somebody else's | `the_header_says_this_crate_keeps_its_own_file` in each |
| Every word these refusals can say is in the vocabulary `alo-saying` collects | `every_word_these_refusals_say_is_in_the_collected_vocabulary` in each, and the `unkept` unit tests that every reason says a declared, fully filled sentence naming the file |

## Verification (Windows host, this checkout)

Executed, all passing:

- `cargo fmt --all`
- `cargo clippy -p alo-appearance -p alo-dock -p alo-shortcuts --all-targets -- -D warnings` — clean
- `cargo test -p alo-appearance` — 107 unit, 7 new integration, existing suites, doctest: all pass
- `cargo test -p alo-dock` — 61 unit, 6 new integration, existing suites, doctest: all pass
- `cargo test -p alo-shortcuts` — 71 unit, 7 new integration, existing suites, doctest: all pass
- `cargo test -p alo-choosing --test the_folder_is_handed_rather_than_depended_on` — 2 pass (it scans these crates for an `alo-choosing` dependency or an environment read)
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-appearance -p alo-dock -p alo-shortcuts` — clean

Not run here: the full workspace suite. The supervisor runs it. The Unix mode
of a kept file (`0600`/`0700`) is `alo-kept`'s, tested there, and was not run
on Linux in this task.

## Limitations

- Nothing calls these yet: the shell's task 6 (*one place for settings*) is
  what hands each crate its path from `alo_choosing::where_the_folder_is` and
  draws from `at_sign_in`.
- A key refused *inside* a value, such as `fiting` in a picture table, gets the
  general *not settings alo OS can read* sentence, not a named key.
- No organisation bound over these files, as ADR 0038 point 7 and the plan say.

## Proposed shared-document updates

- **CHANGELOG.md:** "Appearance, the dock and keyboard shortcuts are now kept in
  the person's own folder (`appearance.toml`, `dock.toml`, `shortcuts.toml`),
  each read and written by the crate that owns it. A hand-edited file that is
  wrong is refused whole and named, and the machine uses what alo OS ships. A
  settings file can no longer name a path as a shipped wallpaper."
- **ROADMAP.md / QUEUE.md:** *Settings, as one place*: the storage half's
  task 2 is done. Tasks 3 (pairing revocation) and 4 (contract sections and
  vocabulary notes) remain, and the shell plan's task 6 no longer waits on
  appearance, dock or shortcuts storage.
