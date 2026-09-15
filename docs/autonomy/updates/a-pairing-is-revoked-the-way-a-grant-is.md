# A pairing is revoked the way a grant is

**Date:** 2026-09-15
**Workstream:** v0.5 — where a person's settings are kept
(`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 3), implementing
ADR 0038 §8.
**Contributor:** Claude (development worker, checkout `C:\dev\alo-os-b`).
**Status:** ready for integration.

## What changed, for a person

In the list of everything this machine has granted — folders to agents, and
pairings with other machines — taking a pairing away now works the same way as
taking a grant away. The surface makes one call for either row and gets the same
kind of answer back. When the machine's agent service refuses to revoke a pairing,
the person sees the service's own sentence and is told the pairing was **not**
revoked. When no service is running, they are told nothing was revoked. When the
service was reached but never answered, they are asked to look at the list again
instead of being told either thing. When a revocation took effect but could not
be saved, they are told it will come back after a restart.

## What changed, in the code

All in `crates/alo-changing`. `alo-agentd`, `alo-nearby`, `alo-protocol` and
`alo-remembering` are read and not edited. The daemon's `revoke-pairing` and
`pairings` messages in `docs/contracts/daemon-protocol.md` are spoken as
documented, and the contract does not change.

| File | What |
|---|---|
| `src/row.rs` (new) | `Row::{Grant(Seen), Pairing(SeenPairing)}`: one row of the one list |
| `src/seen_pairing.rs` (new) | `SeenPairing`, which comes only from the daemon's `pairings` answer (`told`) or the daemon's file through `alo_remembering::pairings_remembered` (`of`, `remembered`) |
| `src/unpairing.rs` (new) | `RevokingPairings` trait and `Unpaired`, its five answers |
| `src/the_door.rs` (new) | `Door: Knocking + RevokingPairings`, the one door `Changing` holds |
| `src/changing.rs` | `Changing::revoked(&Row, now) -> Result<Gone, NotChanged>`. A grant's path is unchanged. A pairing is asked of the daemon and writes nothing |
| `src/door.rs` | `TheDaemonsDoor` speaks `revoke-pairing` and `pairings` through one shared line exchange. It tells *nobody there* apart from *reached and not answered* |
| `src/stood.rs` | `Stood::KeptByTheDaemon` and `Stood::UntilARestart` |
| `src/refusing.rs` | `NotChanged::PairingRefused { told }`, `NobodyKeepsPairings`, `PairingNotAnswered` |
| `src/words.rs` | Four new sentences with translator's notes, collected through `alo-saying`'s existing `declare_into` call |
| `Cargo.toml` | Depends on `alo-nearby` for `MachineId` and `Pairing`, read only |
| `tests/a_pairing_is_revoked_the_way_a_grant_is.rs` (new) | The acceptance tests, over a real Unix socket and a real file |
| `tests/the_pairings_file_has_one_writer.rs` (new) | The source test for a second writer |
| `tests/a_persons_change_reaches_the_daemon.rs` | Now calls `revoked(&Row::Grant(..))`. Its door implements `RevokingPairings` by failing, so a grant's change that asks about a pairing breaks the test |

## Decisions, and why

- **`Changing::revoked` takes a `Row`, replacing the version that took `Seen`.**
  The acceptance asks for *one call whichever kind of row it is*. Keeping both
  methods would leave the two code paths the plan exists to remove. The only
  callers were this crate's own tests. No crate in the workspace depends on
  `alo-changing` except `alo-saying`, which only collects its words.
- **A pairing is never `Gone::AlreadyGone`.** For a grant, the person's side can
  see a stale row on its own. For a pairing, only the daemon knows. Its answer
  to revoking a machine it is not paired with is a refusal in words, and that
  is carried back as `NotChanged::PairingRefused`. It is not converted into a
  success about nothing, which the contract also forbids.
- **Two new `Stood` variants rather than reusing `Heard { holding }`.** `holding`
  is the daemon's count of *grants*, so filling it for a pairing would put a
  number on screen that means nothing. `KeptByTheDaemon` and `UntilARestart`
  match the daemon's `AfterRevoking` one for one. `was_heard()` is true for
  both, because the running daemon took the pairing away at once in each case.
- **No daemon is a refusal for a pairing, but not for a grant.** A grant's change
  is already on the disk when the knock fails. A pairing's file is the daemon's
  alone, so with no daemon nothing was revoked, and nothing on this side writes
  the file instead.
- **A daemon that was reached and never answered is its own refusal**
  (`PairingNotAnswered`). The daemon may already have acted, so saying *revoked*
  or *not revoked* would be a guess. The sentence asks the person to look at the
  list.
- **A second trait (`RevokingPairings`) plus a blanket `Door`, rather than a
  second method on `Knocking`.** Each trait is one conversation with its own
  answers. `Knocking` stays exactly what `alo-greeting` modelled its own trait on.
- **`SeenPairing` carries the identity and, from the daemon, the given name.**
  The daemon's file holds no names (the names file is separate), so a row read
  from the file has none. A surface draws the rest of the row (what the pairing
  permits, when it ends) from the same answer it built the row from.
- **An identity the daemon lists that is not an identity is left off the list.**
  Nobody could revoke that row, and the daemon naming one is a fault in the
  daemon.
- **A missing pairings file is an empty list, and a file that does not read is
  an error, not an empty list.** *Nothing is paired* would otherwise be a claim
  nobody checked.
- **The second-writer test reads every `crates/*/src` and `tools/*/src`, skipping
  comment lines.** It refuses, in any crate other than `alo-agentd`,
  `alo-remembering` and `alo-nearby`, the names `pairings_kept`,
  `keeping::written` and `pairings.toml`, and `THE_PAIRINGS` beside a file
  write. A calibration test shows the scan finds the daemon's real write in
  `alo-agentd/src/keeping_pairings.rs`, so the scan is known to be looking.
- **No *revoke everything*, and no road for an agent.** A revocation takes one
  `Row`, and `alo-agentd` still does not depend on this crate, which
  `tests/no_agents_door_reaches_this_writer.rs` already holds.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| A pairing's revocation over `revoke-pairing`, answered as the same `Gone`, one call for both kinds (held by a test that both answer the same type) | `a_pairing_is_revoked_the_way_a_grant_is::both_kinds_of_row_are_revoked_with_one_call_and_answer_the_same_gone`; over the wire, `a_pairing_the_daemon_revoked_is_gone_and_kept` and `a_pairing_revoked_until_a_restart_says_so` |
| Listed pairings come from the daemon's `pairings` answer or `pairings_remembered` | `the_daemons_pairings_answer_is_the_list_of_rows`, `the_daemons_file_read_back_is_the_list_of_rows`, `no_pairings_file_is_an_empty_list`, `a_pairings_file_that_does_not_read_is_refused_rather_than_empty`, `with_nobody_at_the_door_there_is_no_daemons_list` |
| A test reads the shipped source for a second writer of `/var/lib/alo/pairings.toml` | `the_pairings_file_has_one_writer::no_shipped_source_is_a_second_writer_of_the_pairings_file` (and its calibration `the_search_finds_the_daemons_own_write`) |
| A revocation the daemon refuses says so in the daemon's words | `a_pairing_the_daemon_refuses_is_told_in_its_words_and_not_as_done`; also `with_nobody_at_the_door_no_pairing_is_revoked`, `a_daemon_that_never_answers_is_not_read_as_a_revocation`, `an_answer_to_another_question_is_not_read_as_a_revocation` |

## Verification

Executed in WSL Ubuntu (root) against `/mnt/c/dev/alo-os-b`, with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`:

- `cargo fmt --all` then `cargo fmt --all -- --check`: clean.
- `cargo clippy -p alo-changing -p alo-saying --all-targets -- -D warnings`: clean.
- `cargo test -p alo-changing`: 20 unit tests, 12 + 10 + 1 + 3 + 3 integration tests, all pass.
- `cargo test -p alo-saying`: 63 + 4 + 1, all pass. The four new keys are collected without collision.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-changing --no-deps`: clean.

Not run, by instruction: the full workspace suite. The supervisor runs it.

On Windows, `cargo clippy -p alo-changing` stops in `alo-remembering`
(`names.rs`: `read` and `written` never used on non-Unix). That is a fault
already in the tree, in a crate this task does not edit, and it is unaffected by
this change. `alo-changing` was already depending on `alo-remembering`.

No test here relies on file permissions refusing root, so running the gates as
root exercises every branch.

## Limitations

- A surface still has to be written to *use* this. That is shell plan task 6,
  which this and task 2 unblock.
- A row read from the daemon's file carries no name the person gave the
  machine. The daemon's `pairings` answer does.

## Proposed updates for the integration owner

- **CHANGELOG.md**: "Settings: a pairing with another machine is revoked from the
  same list, with the same call, as a folder granted to an agent. A revocation
  the agent service refuses is shown in its own words and never as done. One
  that lasts only until a restart says so."
- **ROADMAP.md** (*Settings, as one place*): the person-side road to revoke a
  pairing exists (`alo_changing::Changing::revoked`).
- **docs/autonomy/STATE.md**: reference this report.
- **QUEUE.md**: nothing beyond the plan's own status line.
