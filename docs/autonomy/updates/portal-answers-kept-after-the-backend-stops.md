# What an application was answered is kept after the backend stops

**Date:** 2026-09-15
**Workstream:** v0.5 applications and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 8)
**Contributor:** Claude worker in the Mac checkout, for the repository owner
**Status:** ready for integration

## Change description

The portal backend used to write every answer it gave an application only into
memory, and all of it was lost when the backend stopped. Every answer is now
kept in a file on the disk, `/var/lib/alo/portal-answers.jsonl`: when it was
given, which application asked (or that none could be named), which portal, and
what the application was told, refusals included. A backend started again adds
to the same file, and whoever reads it later finds everything in the order it
happened. If the file cannot take an answer, the application does not get that
answer. It gets the portal's refusal, and no secret is written, no file is
opened and no setting is sent.

## Where it is kept, and why

The plan left two choices: give `alo-record` an application kind of entry (which
needs lane A's agreement, so this task would stop at an ADR), or let
`alo-portals` keep its own file under `alo-remembering`'s rules. **I chose the
second.**

- ADR 0040 part 2 already rules out writing an application under the agent
  column. An additive application kind would need a new tag in
  `docs/contracts/record-file.md`, plus changes to `alo-record`, `alo-keeping`
  and every reader of `Only::ByAnAgent`. That is three crates this plan does not
  own, and it would put a person's application history into the file a SIEM
  export reads as *what an agent did*.
- A separate file beside the record answers the acceptance fully inside this
  plan's crate, with no edit to a crate another lane owns. The two files use the
  same notation (JSON lines under a `{"format":1}` line), so a reader or a later
  merge sees one shape.
- The rules it follows are `alo-remembering`'s (`believing.rs`): no symbolic
  link, a regular file, root's or this login's, not writable by others, made
  `0600`, and the folder never made. Those functions are private to a crate this
  plan may not change, so `crates/alo-portals/src/believed_file.rs` restates the
  three checks. That duplication is a known cost (see *Limitations*).

## What changed

- `crates/alo-portals/src/recording.rs`: `Recording::keep` now returns
  `Result<(), NotRecorded>`. `Kept` (in memory) always keeps.
- `crates/alo-portals/src/not_recorded.rs`: `NotRecorded`, the English refusals
  an administrator reads (`NotThere`, `ALink`, `SomebodyElses`,
  `WritableByOthers`, `NotAnAnswersFile`, `ANewerFormat`, `NotRead`,
  `NotWritten`).
- `crates/alo-portals/src/kept_outcome.rs`: `KeptOutcome` mirrors `Outcome`
  (and `RefusedAs`, `NothingOpensAs`) for the reason `alo_record::Written`
  mirrors `Value`: a decision read back off a disk would be one nothing decided.
  The file keeps identities, not sentences. The conversion is an exhaustive
  `From<&Outcome>`.
- `crates/alo-portals/src/kept_answer.rs`: `KeptAnswer`, one line (time,
  application or nobody, portal, outcome). A line whose application is not an
  identifier is read as unreadable.
- `crates/alo-portals/src/believed_file.rs` (Unix): opening under the rules
  above, with `O_NOFOLLOW | O_NONBLOCK`, so a named pipe at the path is refused
  at once rather than holding `open`.
- `crates/alo-portals/src/answers_file.rs` (Unix): `AnswersFile::opened` writes
  the format line into a new file, or checks it on an existing one (a newer
  format, or a missing format line, is refused and the file left as it was).
  It notices a torn last line. `keep` appends one line and runs `sync_data`
  before returning. `AnswersFile::read_back` returns the answers in order, plus
  the line numbers of lines that did not read. `THE_ANSWERS` and
  `THE_ANSWERS_FORMAT` are public.
- `Portal`, `NotARequest` and `Unanswered` gained `serde` derives with
  kebab-case names, which the contract lists.
- The backend's doors now act only on a kept answer:
  - `asked.rs`: `Asked::recorded` / `Asked::responded`. `answered` sends `2`
    when the record refused. A bad `handle_token` whose refusal could not be
    recorded gets `Failed` rather than `InvalidArgs`.
  - `secret_portal.rs`: the keyring writes the secret into a reserved in-memory
    buffer, the answer is recorded, and only then is the buffer written to the
    application's pipe. The buffer is zeroed afterwards. A pipe write that fails
    is recorded as `not-written` after the answer.
  - `open_uri_portal.rs`: `Opened` is recorded **before** the opener is asked
    over D-Bus activation, so no file is opened on an unrecorded request. An
    opener that does not answer is recorded as `not-opened` after it.
  - `settings_portal.rs`: an unkept answer or refusal is
    `org.freedesktop.portal.Error.Failed`. That includes `ReadAll` of an
    unanswered namespace, which would otherwise return an empty dictionary.
  - `watching_appearance.rs`: `SettingChanged` is sent only if
    `AppearanceSent` was kept.
- `words.rs`: `portals.answered.opened-in` now reads *{application} had a file
  it was granted handed to {opener} to open*. It is written before the opener
  answers, so the old *opened in* would have been false when a `not-opened`
  follows it. No translation carried the old text.
- `crates/alo-portals/Cargo.toml`: `serde` and `serde_json` as normal
  dependencies (both already in the workspace; `serde_json` was a
  dev-dependency here). `rustix` moves from Linux-only to `cfg(unix)`. No crate
  is added to the tree, and `Cargo.lock` changes by one line (`serde` in
  `alo-portals`' list).
- Docs: `docs/contracts/portal-answers-file.md` (new, the format),
  `docs/contracts/portals.md` (where the record lives, and what an unrecorded
  answer gets), rustdoc on every new public item, and the crate map in
  `lib.rs`.

## Decisions

1. **A separate file in `alo-portals`, not an `alo-record` kind.** See above.
2. **Record before delivering, and add a second line if delivery fails.** The
   acceptance is *an answer the record could not keep is never sent*. For
   Settings the answer is the reply, so recording first is enough. For Secret
   and OpenFile, handing over the secret or asking the opener is itself the
   delivery, so the decision is recorded first and a failed delivery follows it
   as `not-written` / `not-opened`. The alternative, a separate "about to
   deliver" outcome, would change the response semantics of `Outcome` and add
   an entry to every successful request. The contract documents the two-line
   case.
3. **The refusal sent when the record fails is not itself recorded.** It cannot
   be: the record just failed. That is the one thing the backend says
   unrecorded, and the contract and `asked.rs` say so.
4. **`/var/lib/alo/portal-answers.jsonl` is a constant, beside the record.** The
   same argument `alo-remembering` makes for `THE_GRANTS`. Nothing in this task
   starts the backend in a session, so nothing yet opens the constant on a
   machine. That is image and session work.
5. **A torn last line is ended, not repaired.** The next answer starts on a
   fresh line, and the torn one is reported by number when read back, never
   skipped silently.
6. **Refused files are left untouched.** A newer format, a first line that is
   not a format line, a link, a pipe, or a file others can write is neither read
   nor added to.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| Kept durably, read back in order with time, application or nobody, portal, outcome | `what_an_application_was_answered_is_kept` `the_file::every_kind_of_answer_is_kept_and_read_back_in_order` |
| A backend started again over the same record finds what the last wrote | `the_file::a_record_opened_again_adds_to_what_was_there`; `on_the_bus::a_backend_started_again_finds_what_the_last_one_wrote` (real bus, two backends) |
| Not granted and not identified written and read back | the two tests above (`nothing-granted`, `never-granted`, `lapsed`, `not-identified` with no application) |
| A caller gone written and read back | `a_caller_is_named_by_the_process_the_bus_holds` `on_the_bus::a_caller_gone_is_read_back_from_the_disk_after_the_backend_stops` (a real process killed and reaped while its sandbox is read, for a Secret request and a `SettingChanged`, read back after the backend stops) |
| An answer the record could not keep is never sent | `on_the_bus::an_answer_the_record_could_not_keep_is_never_sent` (secret not written, file not opened, Settings `Failed`, `ReadAll` not answered, no `SettingChanged`), with `on_the_bus::the_same_requests_are_answered_when_the_record_keeps_them` as its control |
| Kept under `alo-remembering`'s rules, format in `docs/contracts/` | `the_file::a_file_somebody_else_could_have_written_is_refused`, `the_file::a_file_that_is_not_an_answers_file_this_reads_is_left_as_it_was`, `the_file::a_missing_folder_is_refused_and_nothing_kept_is_not_there`, `the_file::a_named_pipe_is_refused_without_waiting`, `the_file::a_torn_last_line_is_reported_and_the_next_answer_is_whole`, `the_contract_describes_the_file` |

No approval was needed: no ADR is contradicted, no grant is widened, no crate
another lane owns is edited, and nothing in `docs/features.md` is narrowed.

## Verification

Run on the Mac lane's Lima VM (`alo`, Ubuntu 24.04 aarch64, `dbus-daemon`
1.14.10, as root), with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`:

- `cargo fmt --all` (on the Mac, the checkout both see): clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-portals`: clean.
- `cargo test -p alo-portals`: all pass (35 unit; integration 8 + 9 + 5 + 5 + 5
  + 11; 1 doc).
- `cargo test -p alo-secrets --test the_secret_portal_answers_on_a_real_bus`:
  2 pass. `alo-secrets` is not edited, but its keyring is what the reordered
  Secret door hands over from.

**Not run:** the full workspace suite, which the supervisor runs.
**Not measured:** a real full disk. The record failing is simulated by a
`Recording` that refuses, on a real bus with real clients. The file's own write
failure path (`torn` set, line ended by the next answer) is covered by the
torn-line test, not by an `ENOSPC`.
**Not physical acceptance:** no Flatpak application, and no session starting the
backend over `/var/lib/alo`.

## Limitations

- **The three ownership checks are restated** from `alo-remembering`'s private
  `believing.rs`. Proposal for that crate's owner: publish the believing open
  (or move it to `alo-kept`) so both files share one implementation.
- **`alo-keeping` already writes the agent's record** with a head line, damage
  reporting and shortening. Its types are built around `alo-record`'s entries,
  and it belongs to another lane, so the answers file does not share it.
  Retention for this file is the next task (task 9, written in the plan).
- **The file only grows** until task 9.
- **Nothing reads it for a person yet.** Showing what applications were refused
  belongs to a surface, not to this plan.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** applications plan task 8 is done. Task 9, *what
  applications were answered is kept as long as the machine's record, and no
  longer*, is written in the plan and ready.
- **ROADMAP.md:** nothing moves. This is v0.5 screenless work under ADR 0028.
