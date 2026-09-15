# One person's folder, from sign-in to the next change

**Date:** 2026-09-15
**Workstream:** v0.5 — where a person's settings are kept
(`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 6)
**Contributor:** Claude Code worker, `C:\dev\alo-os-b`
**Status:** ready for integration

## What changed

A person's settings folder is now tested the way a session actually uses it,
all files at once, instead of one crate at a time.

- `crates/alo-choosing/tests/one_persons_folder_from_sign_in_to_the_next_change.rs`
  — a single integration test. It gives `alo_choosing::where_the_folder_is` a
  temporary home directory (never read from the environment) and hands each of
  `alo_appearance::keeping`, `alo_dock::keeping` and `alo_shortcuts::keeping`
  the path its own `THE_FILE` names in that folder. Then it walks:
  1. first sign-in on a new login: all three sections are drawn as shipped, and
     drawing them creates nothing;
  2. a change in each section, plus a language chosen through
     `alo_choosing::Choosing`: exactly four files end up in one folder
     (`appearance.toml`, `dock.toml`, `settings.toml`, `shortcuts.toml`). On
     Unix the folder is `0700` and every file `0600`;
  3. second sign-in: every section is drawn as the person left it;
  4. one file broken by hand (a key that is not on the list, under the
     `format` line): that section is drawn as shipped, with a sentence naming
     the file and the key, from the vocabulary the whole machine collects. The
     other two sections are drawn as the person left them;
  5. the next change to the broken section is refused. The sentence names the
     file, `did_not_read` gives the same refusal as sign-in did, and the file's
     bytes are unchanged. A change to another section made at the same moment
     is written;
  6. putting the broken section back as shipped writes `format = 1` alone, the
     section then reads as shipped, and the next change is written and read
     back. `settings.toml` is untouched throughout, and no `.new` sibling is
     left behind.

  The walk runs three times, once with each file broken, so all three keepers
  are held to the same path.
- `docs/contracts/person-settings.md` — new section *A Settings surface, from
  sign-in to the next change*, for the shell. It covers the one call at
  sign-in, then a table with each section's path constant, the call that draws
  it, the call that writes a change and the call that puts it back as shipped.
  It also says what the surface does with each refusal. The test reads this
  section and requires its `alo_…` code spans to be exactly the calls the walk
  makes, in both directions.
- `crates/alo-choosing/Cargo.toml` — `alo-appearance`, `alo-dock` and
  `alo-shortcuts` added as **dev-dependencies** only; `Cargo.lock` follows.
  None of the three depends on `alo-choosing`, and the existing
  `the_folder_is_handed_rather_than_depended_on.rs` still checks that.
- The plan: task 6 marked done, and task 7 written (below).

**User-readable change description:** alo OS now has one test that follows a
person's settings folder from sign-in to the next change. It checks that
appearance, the dock and shortcuts are saved next to the model settings and
come back at the next sign-in. It also checks that a file broken by hand only
resets its own section, that Settings will not save over the broken file, and
that "put back as shipped" repairs it. The settings contract now tells the
shell which call to make for each section.

## Decisions

- **Which crate holds the walk: `alo-choosing`.** The folder is worked out
  there, and it already had `alo-saying` as a dev-dependency for the
  machine-wide vocabulary. Putting the walk in a keeper would have given that
  keeper a dependency on `alo-choosing` (forbidden). A new crate would have
  been a crate that holds only a test.
- **One test, three passes.** The acceptance asks for one integration test.
  Breaking only one file would leave two keepers never tested for the refusal
  on this road, so the walk repeats with each file broken. It uses a small
  in-test `Section` trait over the three crates' real calls.
- **How the contract is held.** The test does not match prose. It collects
  every `alo_…` code span in the shell's section and compares that set to the
  calls the walk makes. If the doc names a call the walk doesn't make, or the
  walk makes a call the doc doesn't name, the test fails. Method names used on
  values (`said`, `did_not_read`, `changes()`) are written without the `alo_`
  prefix, so they are described but not part of that set.
- **No new public surface was needed.** Every call the shell makes already
  existed. The one thing missing is a sentence (next paragraph), and I wrote
  that up as task 7 rather than adding it here, because the task says a new
  public surface is added only when the walk finds one missing and is then
  named as what the shell plan waits on.
- **What the shell plan waits on:** nothing blocks the shell plan's task 6. One
  gap is named: a session with **no folder** (no usable `$HOME`) draws every
  section as shipped and keeps nothing, and today no sentence tells the person
  so. That is task 7 in this plan.
- **Revoking a pairing and a grant is not in this walk.** The task's
  description mentions them, but its acceptance does not. They are not files
  in the person's folder, and task 3 already holds them to one call and one
  `Gone` (`crates/alo-changing/tests/a_pairing_is_revoked_the_way_a_grant_is.rs`).
  Adding them here would have made a folder test depend on the daemon's socket
  fake without testing anything new.

## Acceptance criteria and evidence

All are held by
`. alo-choosing one_persons_folder_from_sign_in_to_the_next_change one_persons_folder_is_walked_from_sign_in_to_the_next_change`:

| Criterion | Where in the walk |
|---|---|
| Folder from `where_the_folder_is` under a temporary `$HOME`, never the environment | `Folder::of` |
| Three files beside `settings.toml` under their own `THE_FILE`, read back at a second sign-in | `the_files_are_beside_the_settings`, `a_second_sign_in_draws_what_was_left` |
| A broken file leaves the other two as the person left them, and that one as shipped, with its sentence naming the file | `broken_and_put_back` (first block) and the per-pass asserts |
| The next change to it is refused with its bytes unchanged, while another section's change is written | `broken_and_put_back` (second and third blocks) |
| Putting it back as shipped lets the next change through | `broken_and_put_back` (last block) |
| The contract names the calls, held by the same test | `the_calls_the_contract_names` |
| Refusal paths | a change over a broken file, for each of the three files; mode checks on Unix |

## Verification

Run in WSL Ubuntu, in the supervisor's target directory
(`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`), in the foreground:

- `cargo fmt --all` then `cargo fmt --all -- --check` — clean.
- `cargo clippy -p alo-choosing --all-targets -- -D warnings` — clean (exit 0).
- `cargo test -p alo-choosing` — every target green, including the new test
  (1 passed), `no_english_outside_the_vocabulary` (7) and
  `the_folder_is_handed_rather_than_depended_on` (2).
- Because the contract gained a section after `shortcuts.toml`'s:
  `cargo test -p alo-appearance|alo-dock|alo-shortcuts --test the_contract_describes_this_file`
  — 5 passed each.

Not run by me: the full workspace suite (the supervisor runs it) and a Windows
run. The mode checks are `cfg(unix)`; everything else in the walk runs on any
host.

## Remaining limitations

- Nothing here runs on a machine: the walk uses a temporary folder, not a real
  login's home. Being drawn at sign-in by the compositor is the shell plan's
  task 6.
- A session with no folder has no sentence yet (task 7).

## Proposed shared-document updates

- **CHANGELOG.md:** "A person's settings folder is now tested from sign-in to
  the next change, all files at once, and the settings contract tells the
  shell which call to make for each section."
- **QUEUE.md / STATE.md:** v0.5 *where a person's settings are kept*, task 6
  done; task 7 (*A session with no folder says so in Settings*) ready. The shell
  plan's task 6 is no longer waiting on this plan.
- **ROADMAP.md:** no change.
