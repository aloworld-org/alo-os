# One place for settings, drawn with every section through its own crate

**Date:** 2026-09-16
**Workstream:** v0.5 — the shell (`docs/autonomy/v0-5-the-shell-plan.md`, task 6)
**Responsible contributor:** Claude worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration — **the code only.** A certified machine has
not seen Settings; it is measured by unit tests on real files and laid out and
rasterised the way the other nested surfaces are.

## What changed

| File | What it is |
|---|---|
| `crates/alo-shell/src/settings_window.rs` | `SettingsWindow`: the sections, the focus, and what each key reaches. Opened by hand with nothing but where things are kept. |
| `crates/alo-shell/src/settings_places.rs` | `SettingsPlaces`: the person's folder from `alo_choosing::where_the_folder_is`, each keeper's file by its own `THE_FILE`, and the machine's grants and pairings where the host says. |
| `crates/alo-shell/src/settings_kept.rs`, `settings_keepers.rs` | One road for appearance, the dock and shortcuts — *drawn at sign-in*, *a change*, *put back as shipped* — each a call into that crate's `keeping`. |
| `crates/alo-shell/src/settings_answering.rs`, `settings_paired.rs` | What answers questions: `alo-setting-up`'s four, and under each the choices the person's settings hold, changed through `alo_choosing::Choosing`. |
| `crates/alo-shell/src/settings_granted.rs` | Grants and pairings in one list of `alo_changing::Row`, revoked with `Changing::revoked` alone. |
| `crates/alo-shell/src/settings_lines.rs` | Each section as the words drawn for it — every piece a crate's answer. |
| `crates/alo-shell/src/settings_keys.rs`, `settings_chord.rs`, `settings_seat.rs` | Keyboard operation through the seat; a press named as an `alo_shortcuts` chord while Settings waits for one. |
| `crates/alo-shell/src/settings_raster.rs`, `settings_paint.rs`, `nested_settings.rs` | Laid out, rasterised and submitted above clients and the record, below a waiting question and the egress indicator. |
| `crates/alo-shell/tests/unit_fixtures/settings_testing.rs` (outside `src/`, since it writes the pairings file the daemon alone writes in shipped source), `settings_window_tests.rs`, `settings_raster_tests.rs` | A person's machine on a real disk, and the tests. |
| `crates/alo-shell/tests/settings_source.rs` | The existing four tests kept; two new ones hold the Settings files to the plan's constraint. |
| `crates/alo-shell/src/{lib,presentation,scene_native,scene_drawing,nested,nested_approval,nested_desktop,nested_record,offscreen}.rs` | Registration: modules and exports, `RenderError::SettingsScene`, a Settings layer in the native frame (`None` everywhere else). |
| `crates/alo-shell/Cargo.toml` | Reads `alo-changing`, `alo-choosing`, `alo-granted`, `alo-nearby`, `alo-remembering`, `alo-setting-up`. No `serde` or `toml`. |
| `docs/autonomy/v0-5-the-shell-plan.md` | Task 6 marked done, with the findings below. |

**User-readable change description:** *alo OS has a Settings window. In one
place a person can choose what answers their questions — a model, a provider
they added, a machine they paired with, or nothing at all — choose an accent,
move the dock to another edge, change or clear any keyboard shortcut, and see
everything they have granted to agents and applications, and every machine they
paired with, in one list they can revoke from. Each change is saved by the part
of alo OS that owns it, exactly as it would save it anyway. If one of their
settings files was edited by hand and no longer reads, Settings says which file
and what is wrong, and never writes over it — until they choose to put that
section back as alo OS ships it.*

## How it works

**Sections, in order:** what answers questions; appearance; the dock; shortcuts;
what has been granted to what. A row exists only for something a person can do
(choose it, rebind it, revoke it). Keys: Up/Down/Home/End move, Tab and
Shift+Tab move between sections, Enter or Space acts, Backspace puts one
shortcut back as shipped, Delete puts a section whose file did not read back as
shipped, Escape closes. While Settings waits for a chord, the next press is the
chord; Escape stops waiting and Backspace alone clears the shortcut.

**Every change goes through the owning crate, on a copy.** A kept section
changes a copy of what is drawn and hands its `changes()` whole to that crate's
`keeping::keep`; only a copy the crate wrote replaces what is drawn. What
answers questions calls `Choosing::answered_by` — *a person changing their mind
in Settings*, which leaves setup's own answer alone — or
`answered_by_a_paired_machine` for a machine. A grant's or a pairing's row goes
to `Changing::revoked`, and nothing else in the shell revokes anything.

## Decisions, and why

- **What answers questions is one section built from two crates.** The plan
  lists setup (`alo-setting-up`) and the model and provider (`alo-choosing`)
  separately, but they are one question in the person's file. The four ways are
  drawn with setup's own names and lines, and the choices sit under the way they
  belong to. Setup's `answer` is never called: it is asked once, and
  `alo-choosing` says Settings changes use `answered_by`.
- **A way with nothing to choose under it is absent.** The plan says a setting
  the machine does not have is absent rather than greyed out, and ADR 0009 says
  a disabled control is an advertisement. So *from a provider* appears only when
  there is a provider model to choose, and *on a machine on this network* only
  when a pairing lets that machine's models be asked. *Not at all* is always
  offered.
- **The one list comes from the files the plan names.** Grants come from
  `alo_remembering::remembered`, held as *the* list `Changing` needs. Pairings
  come from `alo_remembering::pairings_remembered`. A pairing the daemon revoked
  leaves the list at once, even *until a restart*, when the file still names it.
  The machine is also no longer offered as an answer.
- **A grants or pairings file that is there and does not read is not drawn as
  *nothing granted*.** That would be a lie. No crate has a sentence for it
  (finding), so the refusal goes to the host in `SettingsOpened` and the section
  draws no rows. Pairings can still be revoked, because that writes nothing on
  this side.
- **A session with no folder offers no change.** The contract says nothing is
  written in that case. Offering a change that would be forgotten silently is
  the gap the keeping plan's task 7 exists to close, so the three kept sections
  and what answers questions show no rows there. The compositor still draws the
  shipped values.
- **Chosen is a mark, focus is an edge, waiting is a doubled edge.** No crate
  declares a word for *chosen*. State must never be colour alone (EN 301 549).
  Everything is drawn in the scheme's two tokens: nothing is dimmed, and
  terracotta never appears.
- **The record window's room is reused** (`record_room::Room`) for measures and
  palette rather than copied. Settings is the same kind of tall, whole-row
  panel.
- **The contract's *A Settings surface* section is not edited.**
  `alo-choosing`'s walk test holds that section to exactly the thirteen calls it
  makes. The shell makes those calls and does not add to them.

## Acceptance criteria and evidence

| Criterion (plan, task 6) | Test |
|---|---|
| One surface holds every setting the machine has today | `alo-shell lib settings_window::tests::every_setting_the_machine_has_is_in_one_window` |
| What was chosen at setup / the model and provider: the surface changes nothing the crate would not | `alo-shell lib settings_window::tests::what_answers_questions_changes_nothing_alo_choosing_would_not` — refusals: `…::a_paired_machine_no_longer_permitted_is_refused_in_alo_choosings_words`, `…::settings_that_did_not_read_are_drawn_as_refused_and_never_written_over` |
| Appearance | `alo-shell lib settings_window::tests::appearance_changes_nothing_its_crate_would_not` |
| The dock (and a file that did not read) | `alo-shell lib settings_window::tests::the_dock_changes_nothing_its_crate_would_not`; `…::a_file_that_did_not_read_is_kept_until_the_section_is_put_back_as_shipped` |
| Shortcuts (and every chord refusal) | `alo-shell lib settings_window::tests::shortcuts_change_nothing_their_crate_would_not_and_refuse_in_its_words` |
| Grants and pairings in one list, revoked the same way | `alo-shell lib settings_window::tests::a_grant_and_a_pairing_are_revoked_the_same_way` — refusals: `…::a_pairing_the_daemon_did_not_revoke_stays_in_the_list_with_its_words`, `…::an_expired_grant_is_no_row_and_revokes_nothing` |
| A setting the machine does not have is absent, not greyed | `alo-shell lib settings_window::tests::a_way_with_nothing_to_choose_is_absent_rather_than_disabled`; `alo-shell lib settings_raster::tests::chosen_and_focused_are_shapes_in_two_colours_and_never_terracotta`; `…::a_session_with_no_folder_offers_nothing_it_could_not_keep` |
| Constraint: decides nothing, no second place to grant, every value through its crate | `alo-shell settings_source settings_words_nothing_grants_nothing_and_changes_only_through_the_owning_crates` — and its refusal path, `alo-shell settings_source a_settings_file_that_words_grants_or_writes_around_its_keeper_is_refused` |

"Changes nothing the crate would not" is tested **byte for byte**. Each test
makes the same change through the owning crate on its own, in a twin folder or
on a copy of the machine's file, and compares the two files.

## Verification

Executed 2026-09-16 on a Windows 11 host, with Ubuntu under WSL 2 and
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`.

- `cargo fmt --all -- --check`: clean.
- `cargo clippy -p alo-shell --all-targets -- -D warnings`: exit 0.
- `cargo clippy --all-targets -- -D warnings` (workspace): exit 0.
- `cargo test -p alo-citing`: exit 0 (21 and 10 passed). Every ADR this
  report and the plan cite resolves.
- `cargo test -p alo-shell`: exit 0. Results: lib 346 (29 new), `approval_source` 5,
  `client_lifecycle` 262, `desktop_source` 4, `egress_status_source` 3,
  `record_source` 5, `settings_source` 6 (2 new), `sign_in_source` 4,
  `socket_ownership` 3, and doc tests. 0 failed.
- `cargo doc -p alo-shell --no-deps` with `RUSTDOCFLAGS=-D warnings`: exit 0.
- Each evidence test above was run on its own with `--exact`, and each reported
  1 passed.
- **Mutation:** adding `const _PROBE: fn() -> String = || format!("chosen");` to
  `settings_lines.rs` failed
  `settings_words_nothing_grants_nothing_and_changes_only_through_the_owning_crates`
  with `src/settings_lines.rs:25 format!` and `a string a person could read`.
  The line was removed, and the test passes again.

Not run: the full workspace suite (the supervisor runs it), and anything on a
certified machine or under WSLg.

## Refused once, and what changed

The first hand-over was refused by the workspace suite:
`alo-changing`'s `no_shipped_source_is_a_second_writer_of_the_pairings_file`
named `crates/alo-shell/src/settings_testing.rs`. That fixture is `cfg(test)`
only, but it stands the machine up with `alo_remembering::pairings_kept`, and
the test reads every file under a crate's `src/` as shipped — which is the
right rule, because it cannot tell a fixture from a writer by reading.

**Decided:** the fixture moved to
`crates/alo-shell/tests/unit_fixtures/settings_testing.rs`, and `lib.rs` names
it with `#[path]` under `#[cfg(test)]`. It is still a module of the crate, so
the unit tests reach it unchanged, and it is outside `src/`, which is where the
repository's own rule says a test that writes a pairings file belongs. Not
chosen: an exemption in the one-writer test (a gate weakened), or writing the
file's text by hand in the fixture (the same write, spelt so the search misses
it). `unit_fixtures/` has no `main.rs`, so Cargo makes no test target of it.

Gates after the change, on Ubuntu under WSL 2 (`alo-shell` is Linux-only, so a
Windows run checks nothing of it): `cargo fmt --all --check` clean;
`cargo clippy --all-targets -- -D warnings` (workspace) exit 0;
`cargo test -p alo-shell` exit 0 (lib 346, `client_lifecycle` 262, every other
target passing); `cargo test -p alo-changing --test
the_pairings_file_has_one_writer` 3 passed; `cargo doc -p alo-shell --no-deps`
with `-D warnings` exit 0.

## Findings — what the surface needs that no crate has yet

1. **No section headings and no label for *put back as shipped*.** No crate
   declares a word for *Appearance*, *Dock*, *Shortcuts*, *Grants* or for the
   put-back act. Sections are separated by a rule. Delete performs the put-back,
   and the refusal sentence each keeper already declares tells the person they
   may *put … back as alo OS ships it*. Proposed owner: a vocabulary for the
   Settings frame (probably `alo-shortcuts`/`alo-dock`/`alo-appearance` each
   naming their own section, or a small crate for the frame).
2. **Nothing opens Settings.** `alo-shortcuts` declares no action for it and no
   dock item exists, so the host must call `opened_by_hand` itself. The record
   window has the same finding.
3. **A provider's models are not kept.** `[[provider]]` in
   `docs/contracts/person-settings.md` has no models. Once a person chooses
   something else, a provider can only be chosen again by typing a model name,
   and Settings has no typing. Only the currently chosen provider model is
   offered (plus any a `Provider` carries in memory). Brought weights and paired
   machines have no such gap.
4. **Appearance offers only the accent.** The scheme following, text scale,
   backgrounds and lock screen are kept by `alo-appearance`, but none has words
   for its choices, and backgrounds need a picker. They are absent, not
   disabled.
5. **A grants or pairings file that does not read has no person's sentence.**
   `alo_remembering::NotRemembered` has none. Settings reports it to the host
   and draws no rows.
6. **A session with no folder** waits on
   `v0-5-where-a-persons-settings-are-kept-plan.md` task 7 for the sentence that
   says changes will not be kept. Until then, Settings offers no change there.
7. **A grant's row has no *when it ends*.** `alo-granted`'s note says the
   duration is drawn beside the row "by whoever displays it". No crate decides
   how a moment or a duration is written for a region (the same finding as the
   record window's dates).

## Proposed changes to shared documents

**CHANGELOG.md** — under v0.5: *Settings, as one place: what answers your
questions, the accent, the dock's edge, every keyboard shortcut, and everything
granted to agents, applications and paired machines in one list you can revoke
from — each saved by the part of alo OS that owns it, and never written over a
file you edited by hand.*

**ROADMAP.md** — v0.5 *Settings, as one place*: the code, not ticked on the
machine. The network, display, sound, printers, storage and updates sections
named there have no crate for Settings to read yet.

**docs/autonomy/QUEUE.md** — shell plan task 6 done, the code only. Findings 1,
3, 4 and 5 are proposed as crate-side work outside the shell plan.

**docs/autonomy/STATE.md** — reference this report.
