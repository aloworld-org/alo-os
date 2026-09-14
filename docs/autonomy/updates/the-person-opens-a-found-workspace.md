# The person opens a found workspace, at the address measured at that moment

**Date:** 2026-09-14
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 18)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository owner
**Status:** ready for integration

## What changed, for a person

A person who has been shown a workspace on their office network — mail, files, chat
and documents — can now open it. Their shell asks the machine to open that workspace
by the identity it was found by, and the machine looks at the network again at that
moment and tells the shell where the workspace is answering *right now*. Nobody types
an address, and an address remembered from a list shown a minute ago is never used.

If the workspace is not answering, or if two different places on the network both
claim to be it, nothing is opened and the person is told why in one sentence. The
machine itself never connects to the workspace: it tells the person's session where
to go, and the workspace's own sign-in answers them. Each workspace opened is written
in the machine's record, with which workspace and where; an agent cannot open one.

## What changed, in the code

**`crates/alo-protocol`**
- `asked.rs`, `person.rs`: `{"open-workspace":{"machine":"<identity>"}}` as
  `FromAPerson::OpenWorkspace { machine }` — the identity and nothing else
  (`deny_unknown_fields`, so an address, port or URL beside or instead of it is
  `NotReadable`). `agent.rs`: refused on the agent's door as `NotForAnAgent`.
- `told.rs`, `to_a_person.rs`: the answer `workspace-opened`, as
  `ToAPerson::WorkspaceOpened(FoundWorkspace)` — the shape one entry of `workspaces`
  already has — with `workspace_opened` and `opened_workspace`. `to_an_agent.rs`
  refuses it as an agent's answer. `workspaces.rs` and `lib.rs` documentation updated.

**`crates/alo-record`**
- `happened.rs`: `Happened::WorkspaceOpened { workspace, answers_at }`, tag
  `workspace-opened`. No agent (`agent()` is `None`), not egress, not a refusal;
  every accessor's exhaustive match gained the arm. `entry.rs`:
  `Entry::a_workspace_was_opened(workspace, answers_at, at)`. `lib.rs` table.

**`crates/alo-turn`**
- `a_workspace_was_opened` on `Machine`, `Turning` and `Arriving`, beside
  `a_pairing_was_kept` and for the same reason: the person's door writes the record
  whichever of the three holds the machine. The remote turn's entry is not stamped
  with that turn's origin.

**`crates/alo-recounting`**
- `Outcome::WorkspaceOpened` and the clause `recounting.outcome.workspace-opened`
  (*the person at this machine opened a workspace found on the network*), so the
  exhaustive match stays a build failure rather than a blank line.

**`crates/alo-agentd`**
- `src/opening_workspaces.rs` (new), in this order: the identity
  (`MachineId::read`), refused before the link is asked; `LookingFor::look_around` at
  the moment; exactly one address for that identity; the workspace drawn by
  `listing_workspaces::drawn`; `Holding::a_workspace_was_opened`; then the answer.
- `src/words.rs`: `THAT_IS_NOT_A_WORKSPACE`, `NO_SUCH_WORKSPACE_ON_THE_NETWORK`,
  `A_WORKSPACE_ANSWERED_FROM_MORE_THAN_ONE_PLACE`, each with a translator's note.
- `src/listing_workspaces.rs`: the "name only where true" rule factored into
  `pub fn drawn`, used by listing and opening alike.
- `src/holding.rs`: `a_workspace_was_opened`. `src/answering.rs` dispatches the
  request before the turn is consulted; `reaching.rs`, `pairing.rs` and
  `answering.rs::answered_to` gained the arm their exhaustive matches need.

**`crates/alo-changing/src/door.rs`**: one arm on an exhaustive match of
`ToAPerson`; no behaviour changes (an unexpected answer is still read as no daemon).

**Contracts, additively:** `docs/contracts/daemon-protocol.md` gains
`open-workspace`; `docs/contracts/record-file.md` gains `workspace-opened` among the
entries with no `agent`; `docs/contracts/local-network-wire.md` says what reaching a
found workspace now is.

## Decisions

**The request field is `machine`, not `workspace`.** It is the same value as
`FoundWorkspace.machine` on the list a shell draws, so a shell copies one field and
there is one name for one value on the wire. In the record the field is `workspace`,
because a record entry is read alone and `with`/`machine` there already mean a paired
machine.

**The answer reuses `FoundWorkspace`**, including `called` under the listing's rule
(the paired host answered from the same address in the same look). One shape, one
rule, one function (`drawn`), so a workspace cannot be named on opening where it
would not be named on the list.

**"More than one address" means more than one distinct `address:port`.** The same
answer heard twice from one place is one place and is opened; `Looking::around`
already keeps two answers claiming one identity from two addresses as two things
heard. Refusing both, rather than opening the first to answer, is the point: a
spoofed advertisement could otherwise win by answering faster.

**Refusals write nothing to the record.** The record entry is *a workspace opened*;
a refused opening handed nothing anywhere and is not something that happened to the
machine — the precedent `paired` set for refused proposals. The gate's *every
refusal leaves a record* is about what an agent can reach, and an agent cannot reach
this request: its refusal on the agent's door is `alo-protocol`'s, before the daemon
reads anything.

**The record is written before the answer is sent.** A record that cannot be written
returns `NotKept`, which stops the service with no address handed over — an opened
workspace with no record of it is what law 1's *afterwards in a record* forbids.

**`workspace-opened` is not egress** (`caused_egress` is false). The service dialled
nothing; what connects is the workspace client in the person's session, the
person's own act under their own account. Agent-caused egress is what the indicator
covers, and nothing here is agent-caused.

**The identity is read after trimming whitespace**, as `naming_machines` reads one,
so the two doors that take an identity agree.

**Next task written:** the plan named nothing after 18, so task 19 is written — *An
alo machine that hosts a workspace answers for it, and says nothing more*. The
contract says a workspace on an alo machine is advertised under that machine's own
identity, but `alo-agentd` owns the discovery responder and answers only the machine
question, so nothing on an alo machine can advertise one without a second responder
inventing the identity.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| a request opens a found workspace by its identity, answered by looking around at that moment with the one address it answered from now | `alo-agentd` · `lib` · `opening_workspaces::tests::opening_a_found_workspace_answers_with_the_address_measured_records_it_and_connects_to_nothing`; `…::a_workspace_that_moved_since_it_was_listed_is_opened_where_it_answers_now` |
| refused when what is named is not an identity, contacting nothing | `alo-agentd` · `lib` · `opening_workspaces::tests::opening_something_that_is_not_an_identity_is_refused_and_contacts_nothing` |
| refused when no workspace of that identity answered, contacting nothing | `alo-agentd` · `lib` · `opening_workspaces::tests::opening_a_workspace_that_did_not_answer_is_refused_and_contacts_nothing` |
| refused when more than one address answered for the same identity, contacting nothing | `alo-agentd` · `lib` · `opening_workspaces::tests::a_workspace_answering_from_more_than_one_address_is_not_opened_and_neither_is_contacted` |
| a request carrying an address is not a request | `alo-protocol` · `lib` · `person::tests::opening_a_workspace_by_an_address_is_not_a_request`; `alo-agentd` · `lib` · `opening_workspaces::tests::a_request_to_open_a_workspace_carrying_an_address_is_not_a_request` |
| an agent sending it is refused in the words an approval gets | `alo-agentd` · `lib` · `opening_workspaces::tests::an_agent_opening_a_workspace_is_refused_as_an_approval_would_be`; `alo-protocol` · `lib` · `person::tests::opening_a_workspace_is_a_persons_by_identity_and_refused_to_an_agent` |
| opening writes *a workspace opened by the person*, naming the identity and the address, and the daemon connects to nothing (a listener at that address never connected to) | `alo-agentd` · `lib` · `opening_workspaces::tests::opening_a_found_workspace_answers_with_the_address_measured_records_it_and_connects_to_nothing`; `alo-record` · `lib` · `entry::tests::a_workspace_opened_by_the_person_names_the_identity_and_the_address_and_no_agent` |
| the answer is the person's and never an agent's | `alo-protocol` · `lib` · `to_a_person::tests::the_workspace_opened_is_told_to_the_person_and_not_to_an_agent` |

Beyond the criteria: `a_workspace_heard_twice_from_one_address_is_opened` and
`a_workspace_hosted_by_a_named_paired_machine_is_opened_under_its_name`.

## Verification — executed

On WSL Ubuntu, `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`,
from `/mnt/c/dev/alo-os-claude`:

- `cargo fmt --all` — applied, then clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo test -p alo-record -p alo-recounting -p alo-protocol -p alo-changing` —
  exit 0, every target.
- `cargo test -p alo-turn` — exit 0.
- `cargo test -p alo-agentd` — exit 0: 333 unit and every integration target
  (the kernel-bounded ones under the shared kernel lock), about two minutes.
- `cargo test -p alo-saying` (it collects `alo-recounting`'s words) — exit 0.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` for the six crates touched —
  exit 0. (With `--document-private-items`, `alo-agentd` reports one unresolved
  link, `rereading::read_again` in `answering.rs`, which predates this change and is
  not touched by it.)
- Each evidence test above run on its own with `--exact`: one passed each.

The full workspace suite was not run here; the supervisor runs it.

## Not shown, and owed

- **Two physical machines.** Both ends run on one host over loopback, as for tasks
  1 and 17.
- **The person's session and the workspace client.** The shell that sends
  `open-workspace` and hands the address on is `alo-shell`'s (outside this plan), and
  the client that connects is `alo-workplace`'s. Nothing here can test either.
- **An alo machine advertising a workspace** — plan task 19.
- No `unsafe`, no new dependency, no setting. Three new user-facing sentences in
  `alo-agentd` and one clause in `alo-recounting`, each declared with a note.

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "A person can now open a workspace their alo machine found on the
  local network. The machine checks where it is answering at that moment, refuses if
  it is gone or if more than one place claims to be it, writes the opening in the
  record, and hands the address to the person's session — it never connects to the
  workspace itself, and an agent cannot open one."
- **ROADMAP.md / QUEUE.md:** v0.5 *A self-hosted workspace on the network is
  discovered, not configured* — the person's door can now open a found workspace
  (screenless); the shell's use of it, `alo-workplace`'s client and responder, and
  two-machine acceptance owed. Next: plan task 19, *An alo machine that hosts a
  workspace answers for it, and says nothing more*.
- **STATE.md:** reference this report.
