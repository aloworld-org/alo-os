# Settings explains when session changes cannot be kept

Date: 2026-09-18. Workstream: where a person's settings are kept.
Responsible contributor: lane B development worker in `C:/dev/alo-os-b`.
Task: A session with no folder says so in Settings.

Status: implementation complete, ready for supervisor validation. Publication
validation is pending. Local compilation, lint and test validation is deferred
to the supervisor's nine serialized gates, as required by the worker instructions.
This report and handoff do not certify passing gates.

The checkout already contained `the_persons_folder`, `PersonsFolder`, `NoFolder`
and the collected `choosing.session.no-folder` vocabulary entry. The remaining
gap was that the Settings contract and the complete folder walk still called
the bare-Option API. They now use the typed result and `PersonsFolder::path_of`;
the walk also renders `NoFolder::said` and holds the contract to the exact
sentence and calls. The notice now explicitly says changes take effect now as
well as explaining why they will not survive sign-out. Its translator note
describes both halves and requires displaying it before a change.

The three missing-folder tests retain their checks that no candidate fallback
folder appears, and now assert the immediate effect, cause and sign-out limit.
The legitimate test additionally covers ignoring a relative or empty config
override when HOME is usable, and preferring an absolute config directory when
HOME is relative. The vocabulary test checks the collected notice's translator
note; the existing scan of all five crates remains intact.

Decisions: reuse the existing typed API without breaking the older path helpers;
keep folder resolution pure and keep environment and folder knowledge out of the
keepers, in accordance with ADRs 0016 and 0038. No new public API, folder file,
fallback location or shell change is needed. The contract tells the shell to show
the refusal before edits and draw changes only for this sign-in. This task does
not claim physical Settings acceptance. No new task was appended to the completed
plan, as directed by the supervisor's worker instructions.

Prepared operator edits to `docs/autonomy/a-new-machine-becomes-a-lane.md` and
`docs/autonomy/v0-5-the-models-measured-plan.md` are preserved and included in the
handoff: they assign this task to lane B and release the historical choosing-crate
claim. No model measurement is marked complete by those edits.

Verification performed on Windows:

- `rustfmt --edition 2024 --check crates/alo-choosing/src/words.rs crates/alo-choosing/tests/a_session_with_no_folder_says_so.rs crates/alo-choosing/tests/no_english_outside_the_vocabulary.rs crates/alo-choosing/tests/one_persons_folder_from_sign_in_to_the_next_change.rs`
  initially found layout differences. Applied `rustfmt --edition 2024` to those
  same four files, then repeated the exact check: exit 0.
- `git diff --check`: exit 0 (Git notes CRLF normalization for the two prepared
  operator documents).

Pending supervisor commands include `cargo clippy --all-targets -- -D warnings`,
`cargo test -p alo-choosing`, formatting and the remaining serialized gates.
For each evidence row in `.kernel-loop/handoff.toml`, the exact individual test
command is `cargo test -p alo-choosing --test <target> <full-name> -- --exact`,
from workspace `.`. None of these cargo commands was run locally.

Acceptance evidence requested from the supervisor:

| Criterion | Test target | Full test name |
|---|---|---|
| Collected notice with translator note | `no_english_outside_the_vocabulary` | `a_session_with_no_folder_is_said_only_from_the_vocabulary` |
| Typed folder and legitimate path precedence | `a_session_with_no_folder_says_so` | `a_session_with_a_home_is_handed_paths_inside_its_own_folder` |
| No HOME, notice and no fallback write | `a_session_with_no_folder_says_so` | `a_session_with_no_home_says_its_changes_will_not_be_kept` |
| Relative config and no HOME, notice and no fallback write | `a_session_with_no_folder_says_so` | `a_relative_configuration_directory_and_no_home_says_its_changes_will_not_be_kept` |
| Relative HOME, notice and no fallback write | `a_session_with_no_folder_says_so` | `a_relative_home_says_its_changes_will_not_be_kept` |
| Contract names the call and exact sentence, held by the folder walk | `one_persons_folder_from_sign_in_to_the_next_change` | `one_persons_folder_is_walked_from_sign_in_to_the_next_change` |
| All five crates retain the no-English check | `no_english_outside_the_vocabulary` | `the_five_crates_write_no_english_outside_the_vocabulary` |

Proposed changelog: Settings can explain before an edit that a session without a
usable home directory applies changes now but cannot keep them after sign-out.
Proposed roadmap/queue update: close settings-storage task 7 after supervisor
validation and publication. No broader release or hardware gate is claimed.

## Direct integration by the operator

The owner requested manual completion on 2026-09-18. The original work and
handoff were preserved before integration. Published main c69a044 already
contains the power fixture repair, so no power file or power ownership claim
is included in this Settings change. Software task 10 is also integrated.

The operator runs the seven exact acceptance tests above and every required
Linux gate against the combined tree, with one existing build directory and
the machine gate lock. Earlier pending-supervisor statements describe the
original worker handoff; publication now belongs to the operator. Results
accompany the publication commit. No physical Settings or release certification
is claimed by this library/contract task.
