# The broker's closed list and its door

**Date:** 2026-09-16
**Workstream:** v0.5 the broker and the disk — task 1 of
`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` (`ROADMAP.md`: ★ *System verbs
through the privileged broker*)
**Contributor:** Claude, as a worker in `C:\dev\alo-os-2`
**Status:** ready for integration. The code and its tests; no verb is carried out
and nothing runs on a machine yet, by the task's own constraint.

## What changed, for a person

Nothing a person can see yet. Underneath, alo OS now has the part that will change
settings for the whole machine: printers, the network, updates and drives. It has
a fixed list of the changes it will ever make. None of them takes a name, a path or
a command. It only acts when a person has approved that exact change, once. Each
approval works one time and expires after a minute. Every request it gets is written
into the machine's history before it answers, including the ones it refuses. Reading
that history back now says *a change to settings for the whole machine was passed on
to be made, as the person approved*, or *…was refused, and nothing was changed*.

## What changed, in the repository

**`crates/alo-broker`** (new; library only, 744 lines of code in 1,904 lines):

| File | What it holds |
|---|---|
| `src/verbs.rs` | `SystemVerb`, the closed list: eleven verbs, one argument each; held to `Copy` by the compiler; `EVERY_NAME`, `read`, `one_of_each` |
| `src/arguments.rs` | The only two argument shapes: `Identity` (32 bytes, the SHA-256 of what the rented service reported) and `Switch` (on/off); `Argument` and the bytes a proof is made over |
| `src/hex.rs` | Lowercase hexadecimal, one spelling |
| `src/approving.rs` | `ApprovingKey` (HMAC-SHA-256 via `ring`, fresh from `getrandom`, never printable) and `Token` (approval, issued, proof over the exact verb and argument); `LIFETIME` of 60 s |
| `src/spent.rs` | `Spent`: a genuine token is used once, and forgotten once it could only lapse |
| `src/door.rs` | `Door`: the user the kernel must name; refuses to be handed to root at start-up |
| `src/request.rs` | `Request`: one line, five words, at most 256 bytes; `NotRead` |
| `src/answer.rs` | `Answer`: `carried`, `refused <why>`, `not-kept`, spelt as the record spells the reason |
| `src/keeping.rs` | The two seams: `Recording` (implemented for `alo_record::Record`) and `Carrying` (implemented by nothing here) |
| `src/broker.rs` | `Broker::heard`: the seven checks in order, each answer written down before it is given, and a broker that could not write anything down stops |
| `src/unix.rs` | `SO_PEERCRED`, `chown`, `geteuid`/`getegid` through `rustix` — the only file naming it |
| `src/listening.rs` | `Listening`: one socket, mode 0660, handed to a group; the kernel asked before a byte is read; bounded line; ten seconds' patience |

**`crates/alo-record`** (additive): `src/brokered.rs` adds `AtTheBroker`, the closed
list of refusal reasons, and `Entry::handed_on_by_the_broker` /
`Entry::refused_by_the_broker`. `Happened::Brokered { verb, from_approval, refused }`
is the new kind; it names no agent, is not egress, counts as stopped only when
refused, and answers `from_approval` only for a genuine token. `format` stays `1`.

**`crates/alo-recounting`**: `Outcome::MachineChangeHandedOn` and
`MachineChangeRefused`, two new sentences with translator's notes
(`recounting.outcome.machine-change-handed-on`, `…-refused`), and a test that neither
names the machinery.

**Documents:** `docs/contracts/agent-verbs.md` (the list, the identity, the wire and the
answers, under *The privileged broker*); `docs/contracts/record-file.md` (`brokered`);
the plan marks task 1 done and records what task 2 inherits.

## Decisions, and why

1. **Arguments are an identity digest or a switch, and nothing else.** The plan forbids
   any string that becomes a path, command, configuration line or device name. A
   32-byte SHA-256 of the identity the rented service reported cannot be any of those.
   Nothing ever reads it as anything but bytes to compare. The verb that carries it
   out asks the service what it has now, and acts on the match or on nothing. That
   makes a hostile argument harmless rather than merely validated. It also means the
   later tasks' "`alo-printing`'s own types" arrive as a digest of those types' text.
   The broker never receives the text.
2. **Eleven verbs, the plan's own list.** Printers: add, remove, set default. Network:
   join, forget, radio, set proxy. Updates: apply, roll back. Storage: mount, eject.
   **Checking a disk's health is left out.** It is a read, and reads answer inside a
   turn without an approval (ADR 0001 §5). Putting it behind a door that demands an
   approval token would have decided task 4's question the wrong way. No verb takes a
   password (task 3's rule is already true of the shape). No verb formats,
   repartitions or erases anything.
3. **The token is an HMAC over the exact request, used once, good for 60 seconds.**
   It covers the verb name, the argument's shape and bytes, the turn's approval
   number (`alo_capability::ProposalId`, so the broker's entry and the turn's entry
   share a number) and the moment. Changing any of them fails the proof, which
   `ring` compares in constant time. A genuine token is remembered until it could
   only be refused as lapsed, so memory is bounded without a count. A restarted
   broker has a new key, so forgetting everything on restart is safe.
4. **The door compares the kernel's user, not a group.** `alo-agentd` runs as the
   person's login (`User=alo`, uid 1000; ADR 0001 §2). The *agent* is a login of its
   own (60989), and the person is a member of the agent's group. So a group check
   would be a door the agent's login could knock on. A user check refuses the agent,
   the greeter, the model service, the converter and root.
5. **What the kernel cannot tell apart, said plainly.** On this image `alo-agentd`'s
   credentials are the person's. The kernel therefore cannot tell it from another
   program the person runs. This is a real limit, and no in-task change removes it
   without contradicting ADR 0001 §2. What makes it acceptable: law 2 binds the agent,
   never the person, and task 2 already has the person doing the same by hand through
   the same verbs (ADR 0009). The agent's own login reaches neither the door nor the
   turn's memory, so nothing the model says becomes a token. A future narrowing would
   check that the peer's cgroup is `alo-agentd.service` (through `SO_PEERPIDFD`, so a
   reused pid cannot pass). It is proposed below, not built.
6. **An additive record kind in `alo-record`, although the plan says it reads and never
   edits that crate.** The acceptance needs every request, *permitted or refused*,
   recorded through `alo-record` before the answer. None of the existing kinds could
   record a permitted broker request honestly. `ran` needs an `alo_capability::Authorised`,
   which the broker must not hold. Building one inside the broker would invent a local
   approval and put a false approval number in the record. `turned-away` means "never
   became a call". So the smallest honest change was a new kind, added the same way
   `rolled-back` was added the day before: additive, `format` unchanged, contract
   updated, `alo-recounting` given its clause. The reason is a closed tag, not a
   sentence, so the record carries no English and no caller text beyond a verb name
   through `Line`. This is a deviation from the plan's crate list, made on purpose. It
   is flagged here for the integration owner.
7. **Written down before carried out, and a failed write stops the broker.** A request
   that passes every check is recorded as handed on *before* `Carrying::carry` runs.
   A verb that then fails gets a second entry, `not-carried`, with its approval
   still spent. If any write fails, that request and every later one is answered
   `not-kept` and nothing is carried out. This is `alo-turn`'s rule, held by the
   component with more authority.
8. **No binary, no unit, no key hand-over yet.** A root service that can carry nothing
   out would be privilege on the machine with no function. The process, its unit, the
   machine's record it writes to, and how the approving key reaches the turn all
   arrive with the first verb (task 2). The plan now says so under task 2. The key's
   type (`ApprovingKey::of` / `fresh`) and the issuing call (`ApprovingKey::issue`) are
   ready for both sides.
9. **Afternoon-audit numbers.** `MOST_CODE = 900` lines of code (not blank, not a
   comment, not a file's tests) and `MOST_LINES = 2,200` in `src/`. Today the crate is
   744 and 1,904. Dependencies are held exactly: `alo-record`, `getrandom`, `ring`,
   `rustix` (unix), `serde_json` (dev). `ring` and `getrandom` were already in every
   build. Changing either number or the list is an edit to
   `tests/small_enough_to_audit_in_an_afternoon.rs`.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| Closed enum; argument types walked, failing on a free `String`/`PathBuf` | `alo-broker` `every_argument_is_a_closed_type` `no_argument_of_any_verb_can_hold_text_a_path_or_a_device_name` (plus `the_walk_finds_a_string_a_path_and_a_type_nobody_looked_at`, and `SystemVerb: Copy` held by the compiler) |
| One socket; only `alo-agentd`'s credentials, from the kernel | `alo-broker` `the_door_hears_only_the_agent_service` `a_caller_the_kernel_names_as_anybody_else_is_refused_unread` (plus the heard case, the socket mode and an over-long line) |
| Only for a verb a person approved, verified by the turn's token | `alo-broker` `only_what_a_person_approved_is_handed_on` `a_request_under_no_genuine_approval_is_refused_and_nothing_is_carried` (plus once-only and every-verb happy paths) |
| Every request, permitted or refused, recorded before it answers | `alo-broker` `every_answer_is_written_down_before_it_is_given` `every_request_permitted_or_refused_is_written_down_before_its_answer` (plus `not-carried` and a record that fills up) |
| Line count and dependency list held by a test | `alo-broker` `small_enough_to_audit_in_an_afternoon` `the_broker_is_no_longer_than_an_afternoon` and `the_broker_depends_on_exactly_what_it_names` |
| The record kind the above needs | `alo-record` lib `brokered::tests::a_refusal_is_a_refusal_and_is_found_as_one`; `alo-recounting` lib `told::tests::what_the_broker_answered_reads_back_as_a_change_to_the_whole_machine` |

## Verification

Run in WSL Ubuntu on this machine, target `$HOME/alo-builds/alo-os-2-72aa7fda7f7de151`:

- `cargo fmt --all` — clean.
- `cargo clippy -p alo-broker -p alo-record -p alo-recounting --all-targets -- -D warnings`: passed, no warnings.
- `cargo test -p alo-broker`: 26 unit tests and 14 integration tests passed.
- `cargo test -p alo-record` and `cargo test -p alo-recounting`: all passed.
- `cargo test -p alo-saying` (collects the new sentences, not edited): passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-broker -p alo-record -p alo-recounting`: passed.

**Not run:** the whole-workspace suite. The supervisor runs it. Nothing ran on
certified hardware. No verb exists to run there, and the door's socket test runs in
WSL, which is development evidence only.

## Limitations and follow-ups

- The kernel cannot tell `alo-agentd` from another program of the person's login
  (decision 5). **Proposed:** after task 2 ships the process, check that the peer's
  cgroup is `alo-agentd.service` through `SO_PEERPIDFD`.
- `alo-turn` does not issue tokens yet, and must not be edited under this plan. Wiring
  `ApprovingKey::issue` into the moment a system verb's approval is redeemed belongs
  to task 2, along with the key hand-over.
- The `not-carried` entry keeps the tag, not the machine's reason. Each verb writes
  down its own detail.

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "The part of alo OS that changes settings for the whole machine
  now exists as a fixed list with one door. It acts only on a change a person
  approved, once, within a minute, and writes down every request before answering,
  including the ones it refuses. No setting is changed through it yet."
- **ROADMAP.md:** under ★ *System verbs through the privileged broker*, note the list
  and door as done (the code), with no verb yet.
- **QUEUE.md / STATE.md:** broker plan task 1 done; tasks 2 and 4 ready (task 2
  inherits the process and key hand-over); task 3 still blocked on `alo-proxy`.
