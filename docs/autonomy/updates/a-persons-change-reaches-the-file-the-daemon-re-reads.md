# A person's change to the grants reaches the file the daemon re-reads

- **Date:** 2026-09-11
- **Workstream:** v0.01 delivery plan (`docs/autonomy/v0-01-delivery-plan.md`), task 23
- **Contributor:** Claude (build loop worker, `C:\dev\alo-os-claude`)
- **Status:** ready for integration

## What changed

New crate `crates/alo-changing`: the person's half of a change to the grants,
as one tested value. The daemon's half already existed —
`alo-agentd/src/rereading.rs` answers `alo_protocol::FromAPerson::Granted` by
re-reading `/var/lib/alo/grants.toml` whole — but nothing outside test
fixtures ever wrote that file or sent that knock, so a grant made through
`alo-picking` or a revocation made through `alo-granted` changed a `Grants`
in one process's memory and reached no daemon and no next sign-in. Every
surface would have hand-written the composition — write the file, then knock
— and a knock sent before the write re-reads the old list.

User-readable change description: **when you grant a folder or revoke a
grant, the change is now saved to your machine's own grants file and the
running agent service is told at once — and if nothing is running, the change
simply applies from your next sign-in. A change that cannot be saved changes
nothing at all, and says so.**

### The shape

- `Changing` (`src/changing.rs`) — holds the person's list, the file, and the
  daemon's door. Two doors: `granted` takes `alo_picking::Granting` and a
  sealed `alo_picking::Chosen`; `revoked` takes an `alo_granted::Seen` row.
  Both end in one private function which is **the only caller of
  `Knocking::knock` in the crate**, and the knock stands on the far side of
  `alo_remembering::kept`'s `?` — the order (write, then knock) is held by
  shape, and measured anyway.
- The change is applied to a **copy** of the list; the copy is written whole
  (`kept` stages and renames, so the file is never half a file); only a copy
  the disk accepted replaces the caller's list; only then is the daemon
  knocked. A refused write therefore leaves the file, the memory and the
  daemon exactly as they were, and knocks nobody.
- `Stood` (`src/stood.rs`) — where a kept change stands with the running
  daemon: `Heard { holding }` (the daemon's own count, carried), or
  `AtTheNextSignIn` (nobody at the door — not an error; the state this
  repository shipped with), or `TurnedAway { told }` (a daemon answered that
  it could not re-read and serves under the old list — carried in the
  daemon's own sentence, never dressed up as either of the others, because it
  is the one case with something left to look at).
- `TheDaemonsDoor` (`src/door.rs`) — the real client: one connection to the
  ADR 0017 socket, the knock line, one answer, five seconds of patience per
  read/write. Every way the conversation fails to happen is `AtTheNextSignIn`,
  because by then the change is already on the disk.
- `Made` / `Gone` (`src/outcome.rs`) — each door's nothing is its own case:
  a closed picker (`Made::Nothing`) and a stale row (`Gone::AlreadyGone`)
  grant/revoke nothing, write nothing and knock nobody.
- `NotChanged` (`src/refusing.rs`) — a refused grant keeps
  `alo-capability`'s own sentence; a refused write keeps `alo-remembering`'s
  English inside for the log and hands the person one declared sentence.
- Two strings under a new `changing` area (`src/words.rs`), collected by
  `alo-saying`: twenty-five collected, twenty-six declaring, `alo-agentd`
  still the one deliberately apart.

## Decisions

- **The knock reads the daemon's answer.** The task only demanded the knock be
  sent once after the write, but a client that fired and forgot could not tell
  *the running agent has been told* from *nothing heard*, and a person
  revoking something worrying is owed that difference. The daemon's three
  possible answers map onto `Stood`'s three cases; anything unrecognisable is
  the honest floor, `AtTheNextSignIn`.
- **A daemon that refused the re-read is not "no daemon".** It is running and
  serving under the old list. `Stood::TurnedAway` carries the daemon's own
  sentence rather than rewording it — a machine with two accounts of one
  moment is a machine a person cannot check.
- **The changes arrive only as the sealed types the person's acts produce**:
  `Chosen` (a pick that really happened) and `Seen` (a row derived from the
  machine's own list). This crate adds no way to grant by path or revoke by
  number, so it is a road for the person's two existing acts and not a third
  act.
- **Bounded patience (5s) at the door.** A knock is a courtesy to a running
  daemon, not something a surface may hang on; a daemon that cannot answer in
  time is answered by the next sign-in, like one that is not there.
- **`AtTheNextSignIn` has a declared sentence** ending in *there is nothing
  more to do*, because silence there reads as *nothing happened* and gets the
  same change made twice.
- **The list this value holds must be the person's own, whole** — read off
  the same file at sign-in through `alo_remembering::remembered`, one owner a
  session. The file is replaced whole (that is what makes revocation-by-file
  work at all), so a surface that handed this a fresh empty `Grants` would
  write its emptiness over everything granted. Documented prominently on
  `Changing`; the one-owner discipline is the compositor lane's to keep when
  it wires this.

## Acceptance, test by test

All in `crates/alo-changing`, refusal paths beside the answers:

- *One value composes the person's half; the change reads back off the disk;
  the knock exactly once, after the write* —
  `a_grant_made_through_it_is_on_the_disk_before_the_one_knock` and
  `a_revocation_made_through_it_is_off_the_disk_before_its_knock`
  (`tests/a_persons_change_reaches_the_daemon.rs`): a door of the tests' own
  counts knocks and reads the file at the moment each arrives; the change is
  read back through `alo_remembering::remembered`, the daemon's own road in.
  Over a real Unix socket, with a thread standing where the daemon stands:
  `a_running_daemon_hears_the_knock_after_the_file_says_so` (the far side
  holds the knock to `FromAPerson::Granted` with nothing in it, re-reads the
  file, finds the grant already there, answers; the person's side reads
  `Stood::Heard`).
- *A write that fails leaves the file as it was, sends no knock, and is told
  in words* — `a_grant_the_disk_would_not_take_changes_nothing_and_knocks_nobody`
  (missing folder, refused rather than made) and
  `a_revocation_the_disk_would_not_take_leaves_the_grant_standing` (the
  staging path squatted by a directory — chosen over permission bits because
  the loop runs tests as root, and root ignores modes). File, memory and
  knock count all measured unchanged; the sentence is declared and says
  nothing moved.
- *A machine with no daemon is not an error* —
  `a_machine_with_no_daemon_keeps_the_change_for_the_next_sign_in`: the real
  `TheDaemonsDoor` at a socket nobody listens on; the change stands on the
  disk, the outcome is `AtTheNextSignIn`, the declared sentence names the
  next sign-in.
- *Nothing new can write the grants from an agent's door* —
  `tests/no_agents_door_reaches_this_writer.rs`, the way `alo-clipboard`
  shows the same fact: `alo-agentd` does not name `alo-changing` (no road
  from the socket to the writer), `alo-changing` names no daemon, turn,
  record, context, asking or answering, and the knock line is measured to
  carry nothing. The daemon's own refusal of an agent's knock is
  `rereading.rs`'s `an_agent_knocked`, already tested there.
- *Every string in the vocabulary* —
  `tests/every_sentence_here_is_collected.rs`, plus the refusals said in
  words: `a_daemon_that_turned_the_rereading_away_is_not_read_as_heard`,
  `a_grant_the_machine_refuses_is_not_written_and_not_knocked` (the grant's
  own words), and the `refusing.rs`/`stood.rs`/`words.rs` unit tests.
- The two nothings — `picking_nothing_writes_nothing_and_knocks_nobody`,
  `a_stale_row_writes_nothing_and_knocks_nobody` (file byte-for-byte
  unchanged).
- Constraint held: **no protocol change** (`the_knock_carries_nothing`), no
  file under `crates/alo-shell` touched, no verb added.

## Verification

- Windows (this host): `cargo fmt --all` clean; `cargo clippy --all-targets`
  with warnings denied clean; `cargo test --workspace` green except the six
  pre-existing Windows-only `alo-recounting` failures named by the task 17
  and 18 reports, unchanged here and deliberately not cut to green. The
  unix-only integration tests compile out on Windows; the manifest and
  collection tests run on both hosts.
- Linux (WSL Ubuntu, `CARGO_TARGET_DIR=/root/target-claude`): `cargo test
  -p alo-changing` — 30 tests, all green, including the real-socket pair.
  Full workspace fmt/clippy/test runs recorded below in the gate section of
  the handoff; exact commands:
  `cargo fmt --all`, `cargo clippy --all-targets` (workspace denies warnings
  via `[lints]`), `cargo test --workspace`.
- Physical acceptance: none claimed. No pixels are claimed and none are
  tested; wiring this into a surface is the compositor lane's, and *On the
  machine* does not move.

### Gate repair (second worker, 2026-09-11)

The supervisor's `rustdoc, warnings denied` gate refused the first handoff:
two module docs linked to private items (`Changing::kept_then_knocked` in
`src/changing.rs`, `PATIENCE` in `src/door.rs`), which
`-D rustdoc::private-intra-doc-links` denies. The prose was right and the
items are private on purpose — the private function *is* the shape holding
the order — so the fix is the smallest true one: the two names now render as
plain code rather than links, and every other link in the crate resolves.
Verified with `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` on WSL Ubuntu
alongside the full gates.

## Limitations and follow-ups

- Nothing calls `Changing` outside tests yet — the callers are the folder
  picker's surface and the grants list surface, both the compositor lane's.
  This is the value that lane wires to, with its refusals decided first.
- One owner per session of the person's list is documented, not enforced by a
  lock; the file itself is safe (replaced whole or not at all), but two
  concurrent surfaces could lose each other's in-memory view. The compositor
  lane holds the one-owner discipline the way `SHARED_MAIN.md` holds it for
  working trees.
- The same gap exists one file over for the person's settings
  (`alo-choosing` only reads), and is written as task 24 in the plan.

## Proposed shared-document updates

For the integration owner (not edited here): CHANGELOG entry — "Granting a
folder and revoking a grant now reach the machine's own grants file and the
running agent service immediately; with no service running the change applies
at the next sign-in; a change that cannot be saved changes nothing and says
so." Queue/state: task 23 done, task 24 (person's model choice written where
the machine reads it) added as ready.
