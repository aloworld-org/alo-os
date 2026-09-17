# A session with no folder says so in Settings

**Date:** 2026-09-15
**Workstream:** v0.5 — where a person's settings are kept
(`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 7),
implementing ADR 0038.
**Contributor:** Claude (development worker, checkout `C:\dev\alo-os-b`).
**Status:** ready for integration.

## What changed, for a person

If you sign in to a session that has no home directory — so there is nowhere of
your own to keep settings — Settings now tells you so: a change you make takes
effect straight away, and nothing you change in that sign-in will be kept past
it. Before, the dock you moved simply came back to where it was at the next
sign-in, with nothing that said it would. Nothing is written anywhere in that
session: alo OS does not invent a folder under `/tmp` or wherever the session
started.

## What changed, in the code

All in `crates/alo-choosing`, plus the contract and the plan. No keeper crate
(`alo-appearance`, `alo-dock`, `alo-shortcuts`, `alo-kept`) changed, and nothing
in `crates/alo-shell`.

| File | What |
|---|---|
| `src/folder.rs` (new) | `the_persons_folder(config_home, home) -> Result<PersonsFolder, NoFolder>`; `PersonsFolder::{folder, path_of}`; `NoFolder::{home, said}`; `HomeWas::{Unset, NotAbsolute}` |
| `src/words.rs` | `SESSION_NO_FOLDER` (`choosing.session.no-folder`) with a translator's note; `EVERY_WORD` is 19; the list's own tests hold that this one sentence names no `{path}`, and accept its consequence clause, *nothing you change now will be kept* |
| `src/lib.rs` | Declares and re-exports the module and `SESSION_NO_FOLDER`; a crate-doc section on the folder or why there is none |
| `tests/a_session_with_no_folder_says_so.rs` (new) | One test per way the folder is missing, plus the legitimate road |
| `tests/one_persons_folder_from_sign_in_to_the_next_change.rs` | The walk now reaches the folder by `the_persons_folder` and `PersonsFolder::path_of`, walks a session with no `$HOME`, and holds the contract's section to naming `choosing.session.no-folder` and quoting the vocabulary's sentence word for word |
| `tests/no_english_outside_the_vocabulary.rs` | A test that `src/folder.rs` is among the shipped files read and holds no English of its own |
| `tests/every_sentence_about_a_persons_settings_carries_a_note.rs` | Every `choosing` sentence names `{path}`, except the no-folder one, which is held to naming none |
| `tests/a_persons_choice_reaches_the_machine.rs` | The word count is 19 |
| `docs/contracts/person-settings.md` | *A Settings surface, from sign-in to the next change* names `alo_choosing::the_persons_folder`, `PersonsFolder::path_of`, `NoFolder::said` and the sentence, and says what a session with no folder draws and writes |
| `docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md` | Task 7 marked done; task 8 written |

## Decisions, and why

- **`Result<PersonsFolder, NoFolder>` rather than a new enum.** The plan asks
  for a value that is not a bare `Option`. A `Result` with a refusal type is
  the idiom every other refusal in this crate uses, it is `#[must_use]` for
  free, and `PersonsFolder` has a private field — so the only way to a keeper's
  path is `path_of` on a value you got by matching the `Ok`.
- **`where_the_folder_is` stays public and unchanged.** It is a public surface
  (the contract's own §*where the folder is*, `alo-kept`'s docs, `alo-agentd`'s
  use of `where_it_is`) and changes additively. `the_persons_folder` asks it
  rather than restating the rule, and a test holds the two to agree. Steering a
  surface away from the `Option` is written as task 8 rather than done by
  deprecating it here.
- **The contract's section for the shell now names `the_persons_folder`
  instead of `where_the_folder_is`.** The walk holds that section to exactly
  the calls a surface makes, and a surface should make the new one. The walk's
  list of calls went from 13 to 15.
- **`NoFolder` records `HomeWas::Unset` or `HomeWas::NotAbsolute`, and says
  one sentence for both.** A relative `$XDG_CONFIG_HOME` with no `$HOME` is
  `Unset`: the configuration directory was ignored by rule, and what is missing
  is the home. The distinction is kept for whoever fixes the login. The person
  can do the same thing in either case, so they get one sentence.
- **The sentence names no file.** There is none to name. The list tests that
  said *every sentence names `{path}`* now say it of every sentence about a
  file, and hold this one to having no gap at all. That is the rule stated
  precisely, not relaxed: a `{path}` here would be a gap nothing could fill.
- **"Writes nothing anywhere" is held in type and measured.** `NoFolder` holds
  no path, so there is nothing to hand a keeper. The tests also check, before
  and after each refusal, the places a made-up folder would land: under the
  test's working directory at the relative value's `alo` and `.config/alo`,
  and under the system temporary directory. The walk checks that a real home
  which was not named stays empty.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| The sentence is declared with a translator's note and collected by `alo-saying` | `alo-choosing every_sentence_about_a_persons_settings_carries_a_note every_sentence_the_five_crates_say_carries_a_note` |
| No `$HOME` answers the refusal and writes nothing | `alo-choosing a_session_with_no_folder_says_so a_session_with_no_home_says_its_changes_will_not_be_kept` |
| A relative `$XDG_CONFIG_HOME` and no `$HOME` | `alo-choosing a_session_with_no_folder_says_so a_relative_configuration_directory_and_no_home_says_its_changes_will_not_be_kept` |
| A relative `$HOME` | `alo-choosing a_session_with_no_folder_says_so a_relative_home_says_its_changes_will_not_be_kept` |
| A value, not a bare `Option`: a keeper's path only through a folder | `alo-choosing a_session_with_no_folder_says_so a_session_with_a_home_is_handed_paths_inside_its_own_folder` |
| The contract's shell section names the call and the sentence, held by task 6's walk | `alo-choosing one_persons_folder_from_sign_in_to_the_next_change one_persons_folder_is_walked_from_sign_in_to_the_next_change` |
| The no-English test still reads all five crates clean | `alo-choosing no_english_outside_the_vocabulary the_five_crates_write_no_english_outside_the_vocabulary`, and `a_session_with_no_folder_is_said_only_from_the_vocabulary` |

## Verification

Run in WSL Ubuntu against `/mnt/c/dev/alo-os-b` with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`:

- `cargo fmt --all` then `cargo fmt --all -- --check`: clean.
- `cargo clippy -p alo-choosing --all-targets -- -D warnings`: clean.
- `cargo test -p alo-choosing`: every target passed (141 unit tests; all
  integration targets).
- `cargo test -p alo-saying`: passed (it collects the new word).
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-choosing --no-deps`: clean.
- Each evidence test above was also run on its own with `--exact`, and passed.

Not run here: the full workspace suite, which the supervisor runs. There is
nothing to measure on a machine: the change is pure logic and words.

## Remaining limitations

- No surface says the sentence yet. The shell plan's task 6 draws Settings, and
  this is the call it makes.
- The sentence is English only until a translation arrives, like every other
  sentence in the vocabulary.

## Proposed updates to shared documents

- **CHANGELOG.md:** "Settings says so when a sign-in has no home directory: a
  change takes effect now and will not be kept past this sign-in, and nothing
  is written to a folder that belongs to nobody."
- **QUEUE.md / STATE.md:** v0.5 *where a person's settings are kept*, task 7
  done. Task 8, *A Settings surface reaches the folder one way*, is ready.
- **ROADMAP.md:** no change.
