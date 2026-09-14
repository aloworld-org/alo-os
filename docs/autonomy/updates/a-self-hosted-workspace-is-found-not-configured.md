# A self-hosted workspace on the network is found, not configured

**Date:** 2026-09-14
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 17)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository owner
**Status:** ready for integration

## What changed, for a person

When someone joins an office that runs its own workspace — mail, files, chat and
documents — their alo machine can now find that workspace on the local network by
itself. Nobody has to tell them an address or set up DNS. The person's shell can ask
which workspaces are nearby and is shown each one with the address it was actually
heard from, and, where it is hosted by a machine they have paired with and named,
that machine's name.

Finding a workspace gives nothing to anyone. Nothing on the machine connects to a
workspace because it was found, nothing is paired, and an agent cannot even ask what
is nearby. A workspace that advertises anything beyond which one it is, where it
answers and the version it speaks is not found at all.

## What changed, in the code

**`crates/alo-nearby`**
- `src/workspace.rs` (new): `WORKSPACE_SERVICE` = `_alo-workspace._tcp.local`,
  exported beside `SERVICE`; `WorkspacePresence` (identity + port, the closed list a
  host advertises); `FoundWorkspace` (identity, port, measured address, version,
  `Standing::NotPaired`) with **no public constructor** — a `compile_fail` example
  holds that an address has no road into it.
- `src/advertising.rs`: `about_a_workspace` and `a_question_for_workspaces`, written
  by the same private record writer as a machine's presence (`an_answer`,
  `a_question_for`), so the two packets cannot drift in shape.
- `src/reading.rs`: `a_workspace_in`, sharing one parser (`an_instance_in`) with
  `a_machine_in`. Any `TXT` entry but `v=1` is `SaysMoreThanPresence`; a machine's
  presence read as a workspace is the new `NotNearby::NotAWorkspace` (a stranger's
  fault, like `NotAnAloMachine`). The `SRV` target is never used — where a workspace
  answers is the answer's source address.
- `src/looking.rs`: `Looking::ask_for_workspaces` and `Looking::around`, which hears
  machines and workspaces in one window and returns `Around { machines, workspaces }`.
  `Looking::found` is now `around(..).machines` — unchanged behaviour.
- `tests/a_workspace_on_the_network_is_found_not_configured.rs` (new).

**`crates/alo-protocol`**
- `src/workspaces.rs` (new): `FoundWorkspace` as a shell draws it — `machine`,
  `answers_at`, `speaks`, optional `called`; `deny_unknown_fields`.
- `workspaces {}` as `FromAPerson::Workspaces` (`asked.rs`, `person.rs`), refused on
  the agent's door as `NotForAnAgent` (`agent.rs`); the answer `workspaces {found}` as
  `ToAPerson::Workspaces` (`told.rs`, `to_a_person.rs`), refused as an agent's answer
  (`to_an_agent.rs`).

**`crates/alo-agentd`**
- `src/listing_workspaces.rs` (new): the person's door. Looks around before any
  lock; writes each workspace as heard; adds `called` only where the machine of that
  identity is paired now **and** answered from the same address in the same look.
  Writes nothing, contacts nothing.
- `src/looking.rs`: `LookingFor::look_around` (required method) and `around_at`,
  which the shipped `Wire` calls (`src/wire.rs`). Test fixtures implementing
  `LookingFor` gained the method (`testing.rs`, `pairing.rs`, `corridor.rs`).
- `src/answering.rs` dispatches `Workspaces` before the turn is consulted;
  `reaching.rs` and `pairing.rs` gained the arm their exhaustive matches need.

**`crates/alo-changing/src/door.rs`**: one arm added to an exhaustive match on
`ToAPerson`. This crate is not this plan's; the change is the minimum that keeps it
compiling and it changes no behaviour (an unexpected answer is still read as no
daemon).

**Contracts, additively:** `docs/contracts/local-network-wire.md` gains *A workspace
on the network* — the service, the three records, the closed `TXT`, the identity
rule for non-alo hosts, the measured address — which is what `alo-workplace`
advertises against. `docs/contracts/daemon-protocol.md` gains `workspaces`.

## Decisions

**A workspace has its own service rather than a key on `_alo-os`.** A key would
change what an alo machine advertises depending on whether it serves a workspace —
presence that leaks what a machine is doing, which task 1 forbids — and would force a
non-alo host to pretend to be an alo machine.

**Which workspace is a 32-hex identity; on an alo machine it is that machine's.**
That is what lets task 16's names apply without a second naming scheme, and it keeps
organisation names and hostnames out of the advertisement. A host that is not an alo
machine keeps a random identity of the same shape; the contract says so.

**A name is shown only where the named machine answered from the same address in
the same look.** An advertisement is unproven; without this, anything on the network
could advertise a workspace under a paired machine's identity and be shown as *the
studio machine's workspace*, borrowing trust the person placed in that machine. This
is still not cryptographic proof — the address check is discovery-grade — which is
why the name is presentation only and no reach is granted by it.

**Only `v=1` is read.** Consistent with task 1's machine presence: a different
version is a different closed list. A later OS that reads more versions is an
additive change (`speaks` is already on the wire).

**The machine question and the workspace question go out together** (`around_at`),
costing one two-second window per listing rather than two, and making the
same-address check possible.

**Whether reaching a found workspace requires a pairing permitting
`MayAskIts::Workspace`, and so whether the host must be an alo machine.** Decided:

- **The person's own act does not require a pairing, and the host need not be an alo
  machine.** A person opening their organisation's workspace signs in with that
  workspace's own account, and the workspace answers them exactly as it would from
  any network in the world. Being on the LAN confers nothing in that exchange —
  the account does — which is ADR 0003's rule kept rather than bent: the network is
  not the authority. Requiring an alo host would narrow *a self-hosted workspace is
  discovered* to *an alo-hosted workspace is discovered*, which the promise does not
  say; no ADR is needed because nothing is narrowed.
- **Anything this machine or its agents do with a workspace on another alo machine
  requires a pairing permitting `MayAskIts::Workspace`**, made by both people — the
  arm's own documentation (*this is what says a person meant to use it*) and ADR
  0003's *every use of another machine requires a pairing*. That is cross-machine
  agent work, which `docs/features.md` places at v1, and nothing in this task builds it.
- In this task **nothing reaches a workspace at all**, so the decision governs what
  comes next rather than any code here. Task 18 in the plan builds the first half.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| discovery finds a workspace advertised under its own service, reading exactly which workspace, where it answers and the version | `alo-nearby` · `a_workspace_on_the_network_is_found_not_configured` · `a_workspace_advertised_on_the_network_is_found_with_exactly_the_closed_list` |
| an advertisement carrying one field more is refused, not read around | `alo-nearby` · same file · `a_workspace_advertisement_carrying_one_field_more_is_refused_and_not_found` (and `reading::tests::a_workspace_advertisement_carrying_one_field_more_is_refused` for five keys and another version) |
| the person's door lists the workspaces found, each with the measured address and the name of the paired machine hosting it | `alo-agentd` · `lib` · `listing_workspaces::tests::a_workspace_hosted_by_a_paired_machine_is_listed_by_the_name_its_person_gave_it` |
| … and a claimed identity does not borrow a name | `alo-agentd` · `lib` · `listing_workspaces::tests::a_workspace_claiming_a_named_machine_is_not_named_when_that_machine_did_not_answer` |
| refused on the agent's door in the words an approval gets | `alo-agentd` · `lib` · `listing_workspaces::tests::an_agent_asking_which_workspaces_are_nearby_is_refused_as_an_approval_would_be`; `alo-protocol` · `lib` · `person::tests::asking_which_workspaces_are_nearby_is_a_persons_and_refused_to_an_agent` |
| finding a workspace confers nothing: advertised on a network where nothing is paired, nothing is contacted | `alo-agentd` · `lib` · `listing_workspaces::tests::a_workspace_on_a_network_where_nothing_is_paired_is_listed_and_nothing_is_contacted`; `alo-nearby` · `finding_a_workspace_contacts_nothing_and_pairs_nothing` |
| an address a person types is never dialled as a workspace, tested by its refusal | `alo-protocol` · `lib` · `person::tests::an_address_typed_as_a_workspace_is_not_a_request`; `alo-nearby` · `only_an_advertisement_heard_makes_a_found_workspace` (plus the `compile_fail` example in `workspace.rs`) |

## Verification — executed

On WSL Ubuntu, `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`,
from `/mnt/c/dev/alo-os-claude`:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --all-targets -- -D warnings` (whole workspace) — clean, exit 0.
- `cargo test -p alo-nearby` — 145 unit, all integration targets and both doctests
  (including the new `compile_fail`) pass.
- `cargo test -p alo-protocol` — 117 unit and all integration targets pass.
- `cargo test -p alo-agentd` — 324 unit and all five integration targets pass
  (the kernel-bounded ones under the shared kernel lock).
- `cargo test -p alo-changing` — all targets pass.
- Each evidence test above run on its own with `--exact`: one passed each.
- `cargo doc --no-deps -p alo-nearby -p alo-protocol -p alo-agentd` — no warnings.

The full workspace suite was not run here; the supervisor runs it.

**The first handoff was refused before any gate ran.** The supervisor's reserve
check reported *this machine could not say how much room there is in
`$HOME/alo-builds/alo-os-claude-bd192ccccbc3745b`* with nothing after the colon.
Reproduced by a second worker: WSL's service itself was not answering
(`Wsl/Service/0x8007274c`, a connection timeout), so `df` never ran and no stderr
came back from Linux. A few minutes later the same command answered 865 GiB free on
`/`. No code was at fault, so no code was changed. The second worker then ran every
gate again from the start: `cargo fmt --all` (no changes), `cargo clippy
--all-targets -- -D warnings` (exit 0), and `cargo test -p` for `alo-nearby`,
`alo-protocol`, `alo-agentd` and `alo-changing` (all pass), then each of the ten
evidence tests alone with `--exact` (one passed each).

## Not shown, and owed

- **Two physical machines.** Every test puts both ends on one host over loopback
  datagrams. Whether a second machine on an office network hears a workspace over
  multicast is owed to two machines, as for task 1.
- **`alo-workplace`'s responder.** That repository has to advertise against the
  contract; nothing here can test it.
- **Reaching a workspace** is not built (by design): see task 18.
- No `unsafe`, no new dependency, no setting, no new user-facing sentence (the only
  refusal reuses `NotForAnAgent`'s existing words).

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "An alo machine now finds self-hosted workspaces on the local
  network by itself — no address to type, no DNS step — and shows each with where it
  answers and, when a paired machine hosts it, that machine's name. Finding a
  workspace connects to nothing and grants nothing."
- **ROADMAP.md / QUEUE.md:** v0.5 *A self-hosted workspace on the network is
  discovered, not configured* — screenless part done (discovery, the person's door,
  the contract); physical two-machine acceptance and `alo-workplace`'s responder owed.
  Next: plan task 18, *The person opens a found workspace, at the address measured at
  that moment*.
- **STATE.md:** reference this report.
