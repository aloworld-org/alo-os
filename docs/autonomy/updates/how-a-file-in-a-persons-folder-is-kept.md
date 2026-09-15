# How a file in a person's folder is kept

**Date:** 2026-09-15
**Workstream:** v0.5 — where a person's settings are kept
(`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 1,
*What keeping one of these files means*), implementing
[ADR 0038](../../decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md).
**Contributor:** Claude Code worker, `C:\dev\alo-os-b`.
**Status:** ready for integration.

## What changed

**For a person:** nothing visible yet. This is the rule their appearance, dock
and shortcut settings will be kept by once task 2 lands. Together, the rule and
task 2 are what make a changed background survive a sign-out.

**In the code:**

- **`crates/alo-kept` (new).** A small crate that holds the keeping rule in
  type. A crate keeps a file in the person's folder by implementing
  `alo_kept::Kept`: its file name, its own `FORMAT`, the `KEYS` its file may
  have, what untouched means, and two functions that turn a refusal into
  *that crate's* refusal type. `alo_kept::read` and `alo_kept::keep` are the
  only way in and the only way out.
  - `src/kept.rs`: the trait.
  - `src/reading.rs`: a missing file is `Kept::untouched()`. A file that is
    there is checked in this order: text, TOML, `format`, unknown top-level key
    (named), then the shape's own deserialiser. It is refused whole at the
    first failure.
  - `src/writing.rs`: the text is `format = N` plus only the keys the value
    serialises. It is refused if the untouched value would write anything
    more, if the shape is not a table of keys or has its own `format` key, or
    if the text does not read back as the same value.
  - `src/disk.rs`: writes the text to a sibling `<file>.new` and syncs it.
    Then it **reads the sibling back off the disk and asks again whether the
    bytes are the same value**, and only then renames it over the old file.
    Any refusal removes the sibling and leaves the file as it was. The file
    gets mode `0600` and the folder `0700` on Unix.
  - `src/unread.rs`, `src/unwritten.rs`: why a read or a write was refused.
    Neither has a `Display`, and neither quotes the file.
- **`alo_choosing::where_the_folder_is`** (`crates/alo-choosing/src/place.rs`):
  the person's folder on its own, using the same `$XDG_CONFIG_HOME`/`$HOME`
  rule, still without reading the environment. `where_it_is` is now that
  folder joined with `settings.toml`, so the two cannot disagree.
- **`docs/contracts/person-settings.md`**: a short subsection under *Where it
  is* naming the folder, the new function, and the rule every other file in the
  folder follows. The per-file sections are task 4's.
- **Workspace:** `crates/alo-kept` is added to `Cargo.toml` members, and
  `Cargo.lock` is updated.

## Decisions

1. **The rule lives in a new crate, `alo-kept`, not in `alo-choosing`.** The
   plan requires both that the rule be *carried in type* for the crates in
   task 2, and that *no crate gains a dependency on `alo-choosing` for this*.
   Both can hold only if the type lives somewhere those crates can depend on
   that is not `alo-choosing`. Putting it in any of the three settings crates
   would make the other two depend on it for a reason unrelated to their
   shapes. `alo-strings` is a vocabulary, not a file keeper. A crate with one
   responsibility is the Law 4 answer. It is not on the plan's list of owned
   crates, because the plan did not foresee a sixth. It serves only this plan.
2. **`alo-kept` has no words.** "In the owning crate's own words" is enforced
   by the associated types `Kept::NotRead` and `Kept::NotWritten`. `alo-kept`
   hands over `Unread` or `Unwritten` and the owning crate builds its own
   refusal from it. So the crate has no vocabulary, `alo-saying` has nothing new
   to collect, and no sentence gets composed in two places.
3. **Unknown keys are named from a declared list (`Kept::KEYS`).** They are not
   parsed out of `serde`'s error text, which is English and could change. A
   release that adds a key to a shape but not to `KEYS` cannot write it: the
   round trip refuses with `ReadBackRefused(UnknownKey)`. A test covers this.
   Unknown keys *inside* a nested table are still refused whole, through the
   shape's own `deny_unknown_fields`, as `NotItsShape`.
4. **`format` is checked before keys.** A file from a release with a later
   format is reported as another format, not as a typo in a key that is valid
   in that format.
5. **An empty file is refused (`NoFormat`), not treated as untouched.** A whole
   write never leaves an empty file, so an empty one was made by something
   else. Only a missing file means the person changed nothing.
6. **Untouched is written as the format line; the file is not deleted.** Both
   read back as untouched. Deleting would add a second kind of write with its
   own failure modes. ADR 0038's "put everything back" is a write like any
   other.
7. **A relative path is refused on both read and write**
   (`NotWhereItBelongs`), for the same reason `where_it_is` ignores a relative
   `$XDG_CONFIG_HOME`.
8. **The written file is read back off the disk, not just the text.** The plan
   says "read back before it counts". Checking the staged sibling's bytes
   before the rename is the strongest form of that which still leaves the old
   file untouched on refusal.
9. **`alo_choosing::Choosing` is unchanged.** `settings.toml` reads more than
   one format (`ALSO_READ`) and a daemon reads it too, so it does not fit
   `Kept`'s single format. It already follows the same rule, and `disk.rs`
   says it borrows those mechanics.

## Acceptance criteria and evidence

| Criterion (plan task 1) | Test |
|---|---|
| `alo-choosing` exposes the person's folder on its own, without reading the environment | `alo-choosing` `the_folder_is_handed_rather_than_depended_on` `the_folder_is_the_one_the_settings_file_is_in` (and unit test `place::tests::the_folder_is_the_one_the_settings_are_in_for_every_session`) |
| No crate that keeps a file depends on `alo-choosing` or reads an environment | `alo-choosing` `the_folder_is_handed_rather_than_depended_on` `no_crate_that_keeps_a_file_depends_on_this_one_or_reads_an_environment`; `alo-kept` `the_keeping_rule` `a_keeper_is_handed_its_path_and_reads_no_environment` |
| Only the difference is written, under a `format` number of its own per file | `alo-kept` `the_keeping_rule` `only_the_difference_is_written_under_a_format_of_its_own` |
| No file means the person has changed nothing, never an error | `alo-kept` `the_keeping_rule` `no_file_means_the_person_has_changed_nothing` |
| A file that is there and wrong is refused whole, in the owning crate's words, nothing honoured | `alo-kept` `the_keeping_rule` `a_file_that_is_there_and_wrong_is_refused_whole_in_the_owning_crates_words` |
| Whole or not at all, to a sibling renamed over the old, refused unless it reads back as the same value | `alo-kept` `the_keeping_rule` `a_write_is_whole_and_read_back_before_it_counts` |

Refusal paths covered by unit tests in `crates/alo-kept/src`: not text, not TOML
(with the line number), no format (including an empty file), another format,
unknown key, a value the shape refuses, a folder where the file should be, a
relative path, a shape that writes what nobody changed, a shape that is not a
table, a shape with its own `format` key, a value that does not read back,
bytes on disk that do not hold (old file kept, sibling removed), a blocked path,
and a stale sibling.

## Verification

Run on 2026-09-15:

- **WSL Ubuntu** (root), `CARGO_TARGET_DIR=/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`:
  - `cargo fmt --all -- --check`: clean.
  - `cargo clippy -p alo-kept -p alo-choosing --all-targets -- -D warnings`:
    clean (exit 0).
  - `cargo test -p alo-kept`: 24 unit + 5 integration tests passed. The
    Unix-only `disk::tests::a_kept_file_is_its_owners_alone` ran here.
  - `cargo test -p alo-choosing`: every target passed, including 136 unit tests
    and the new integration test (2 tests).
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-kept -p alo-choosing --no-deps`: clean.
  - Each evidence test below run on its own with `--exact`: one test passed each.
- **Windows host:** `cargo test -p alo-kept` passed 23 unit + 5 integration
  tests; the mode test is Unix-only. `cargo clippy -D warnings` on the host
  fails in `alo-remembering` (`names.rs`, dead code on a non-Linux host). That
  failure was already there and is outside this change; the Linux gate is
  clean.
- Not run: the full workspace suite, which the supervisor runs. Nothing here
  touches the machine beyond temporary files, so there is no hardware
  measurement.

No test here uses a `0o000` permission to force a refusal, so running as root
does not hide a refusal branch.

## Limitations

- No crate uses `alo-kept` yet. `alo-appearance`, `alo-dock` and
  `alo-shortcuts` adopt it in task 2. `alo-shortcuts`' `Changes` serialises as
  a list, so task 2 needs to wrap it in a table shape. `text_of` refuses a
  non-table shape, so this cannot be missed.
- `Unread::NotItsShape { said }` carries the deserialiser's message for
  maintainers, and it may include a value from the file. It is never shown to
  a person. Owning crates map it to their own key.

## Proposed shared-document updates

- **CHANGELOG.md:** "A person's settings files beside `settings.toml` now have
  one keeping rule: only what changed is written, under a format of its own;
  no file means nothing changed; a file that is wrong is refused whole; and a
  write is read back off the disk before it replaces the old file
  (`crates/alo-kept`). `alo_choosing::where_the_folder_is` names the person's
  settings folder."
- **ROADMAP.md / QUEUE.md:** v0.5 *Settings, as one place*: the keeping rule
  (task 1 of the where-a-person's-settings-are-kept plan) is done. Tasks 2
  and 3 are unblocked.
- **STATE.md:** reference this report.
