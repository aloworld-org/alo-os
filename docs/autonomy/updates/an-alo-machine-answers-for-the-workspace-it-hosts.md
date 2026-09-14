# An alo machine that hosts a workspace answers for it, and says nothing more

**Date:** 2026-09-14
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 20; written as 19 and renumbered when the measuring lane took 19 first)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the owner
**Status:** ready for integration

## What changed, for a person reading the changelog

An alo machine that has a workspace server installed on it — `alo-workplace`'s mail,
files, chat and documents — now tells the office so, and nothing else. Colleagues'
machines find the workspace under the machine's own identity, at the address it
answered from, and a person there opens it by that identity (task 18). What the
machine says about itself is exactly what it said before; a machine with no workspace
server says exactly nothing more. Where the workspace answers is written by the
package that installs the server, as root, in one file, `/etc/alo/workspace.toml`,
holding one key: `port`. Nothing an agent or a person can send the daemon changes it.

## What changed, in the code

### `crates/alo-nearby`

- `src/answering.rs` (new, split out of `looking.rs` as it gained a second reason to
  change): `Answering` now also holds an optional hosted workspace.
  `Answering::hosting_a_workspace_at(NonZeroU16)` takes **a port and nothing else** and
  builds the `WorkspacePresence` from the responder's own presence, so the identity in a
  workspace answer is always the machine's own; `Answering::workspace` shows it.
  `answer_one` answers a question for machines with the presence (unchanged bytes), a
  question for workspaces with `advertising::about_a_workspace` only where a workspace
  is hosted, and a packet asking both with two packets. A `compile_fail` example holds
  that a `WorkspacePresence` under another identity cannot be handed to a responder.
- `src/reading.rs`: `a_question_for_workspaces_in`, beside `a_question_in`, both through
  one private `asks_for(packet, service)`.
- `src/looking.rs`: `Looking` and `Around` only; the answering test moved with `Answering`.
- `src/lib.rs`: re-exports and crate documentation.
- `tests/an_alo_machine_answers_for_the_workspace_it_hosts.rs` (new): the acceptance, over
  loopback sockets.

### `crates/alo-agentd`

- `src/hosting.rs` (new): `THE_HOSTED_WORKSPACE` (`/etc/alo/workspace.toml`),
  `hosted_at(path) -> Result<Option<NonZeroU16>, NotHosting>` and
  `advertised(path, told) -> Option<NonZeroU16>`, the one decision `main.rs` makes: a
  refusal is handed to the service log and advertises nothing.
- `src/refusing.rs`: `NotHosting` — `ALink`, `NotAFile`, `NotRoots`, `WritableByOthers`,
  `NotRead`, `NotTheShape`, `AnotherKey`, `NoPort`, `NotANumber`, `NotAPort`,
  `TheWiresOwnPort` — each naming the file and saying no workspace is advertised.
- `src/wire.rs`: `Wire::hosting(Option<NonZeroU16>)` (by value, before the service
  borrows the wire) and `Wire::hosts`; the end-to-end tests in
  `wire::a_hosted_workspace`.
- `src/main.rs`: reads the file once, before the wire is bound, and hands the port on.
- `src/starting.rs`, `src/lib.rs`: the order documented; exports.

### Contracts

- `docs/contracts/hosted-workspace-file.md` (new): where the file is, who writes it
  (the package that installs the workspace server, as root — `alo-workplace`'s), that
  `alo-agentd` only reads it, its one key, the rules, and what absence and refusal mean.
- `docs/contracts/local-network-wire.md`: *An alo machine that hosts a workspace answers
  for it*, additively — `alo-workplace`'s server on an alo machine runs no responder of
  its own and invents no identity.

## Decisions

**Read at the next start, not on a knock.** The plan left this to the crate. A knock is
a request on one of the daemon's two doors, and neither is the writer's: the file is
written by root's package, which is neither the person's shell nor the agent and holds
no peer credential either door accepts. A knock from the person's door would let the
person's side make the machine start advertising the moment a file appeared, and the
thing a root package installing a server can already do is restart the service beside
it. Reading at start keeps what the network is told a fact about the service that is
running, fixed for its lifetime, with nothing on the socket able to move it. A change
waits for the next start of `alo-agentd` (a restart by the package, or the next
sign-in).

**Believed from root alone**, where the pairings file is believed from root or the
person. The acceptance names *a file that is not root's* as a refusal, and the reason is
sound: this file speaks to the whole network in the machine's name, and a file the
person's login could write is one any program running as the person could point the
office at. The rest of the pairings file's trust is kept: not a link, a regular file,
not writable by group or world, all asked of the opened handle.

**`/etc/alo/workspace.toml`, not `/var/lib/alo`.** `/var/lib/alo` holds what the person
granted, paired and named; this is installed configuration, beside the machine
description.

**A refused file does not stop the service.** Unlike an unbelievable pairings file —
which would make *tampered* look like *never paired* — a wrong workspace file hides
nothing about the person's own machine; it only means colleagues do not find the
workspace. The machine serves the person, advertises no workspace, and says why in the
service log. Advertising a guess is never an option.

**The port may not be `7610`.** Not in the acceptance, and added because a workspace
advertised on the daemon's own wire port would send every workspace client at the
pairing and verb wire. One arm, one test, in the contract.

**`Answering` takes a port, not a `WorkspacePresence`.** The acceptance says the
responder is *given a `WorkspacePresence`* and that *no constructor lets another
identity reach it*; those only agree if the responder builds the presence itself from
its own identity. So it does, and the `compile_fail` example is the refusal of the other
shape.

**`Answering` moved to its own file.** `looking.rs` held the asking and the answering;
the answering gained a second service to answer, which is a second reason to change
(law 4). Public paths (`alo_nearby::Answering`) are unchanged.

**No format key in the file.** Version one is the one key; anything a later version adds
is a key version one refuses, so an older machine advertises nothing rather than
half-reading a newer file.

## Acceptance, and the test behind each part

| Acceptance | Test |
|---|---|
| answers `_alo-workspace._tcp.local` with exactly the closed advertisement when given a workspace | `alo-nearby` `an_alo_machine_answers_for_the_workspace_it_hosts::a_machine_hosting_a_workspace_is_heard_as_one_under_its_own_identity`; unit: `answering::tests::a_machine_hosting_a_workspace_answers_the_question_for_workspaces_with_the_closed_advertisement` |
| steps over the question when not | `…::a_machine_hosting_no_workspace_is_heard_as_a_machine_and_nothing_more`; unit: `answering::tests::a_machine_hosting_no_workspace_steps_over_the_question_for_workspaces` |
| presence answer unchanged byte for byte either way | `…::what_a_machine_says_about_itself_is_the_same_bytes_whether_or_not_it_hosts`; unit: `answering::tests::the_presence_answer_is_the_same_bytes_whether_or_not_a_workspace_is_hosted`, `…one_packet_asking_both_questions_gets_both_answers_apart` |
| no constructor lets another identity reach the responder | `compile_fail` in `src/answering.rs`; `…::no_identity_but_the_machines_own_reaches_a_workspace_answer` |
| absent file advertises nothing | `alo-agentd` `hosting::tests::roots_file_naming_one_port_is_hosted_and_no_file_is_nothing_hosted`; `wire::a_hosted_workspace::a_machine_with_no_workspace_file_or_a_refused_one_is_found_and_hosts_nothing` |
| not root's → refused | `hosting::tests::a_file_that_is_not_roots_is_refused_and_nothing_is_advertised` (on disk, chowned to uid 989, and as the rule for the person's and the agent's logins) |
| writable by anyone else → refused | `hosting::tests::a_file_writable_by_anyone_else_is_refused_and_nothing_is_advertised` |
| any key but the port → refused | `hosting::tests::a_file_carrying_any_key_but_the_port_is_refused_and_nothing_is_advertised` |
| port outside 1–65535 → refused | `hosting::tests::a_port_outside_one_to_65535_is_refused_and_nothing_is_advertised` |
| refused → the machine advertises no workspace | `wire::a_hosted_workspace::a_machine_with_no_workspace_file_or_a_refused_one_is_found_and_hosts_nothing` (a `0666` file: found as a machine, `open-workspace` refused as no such workspace) |
| an agent cannot write, name or change it; no request on either door does | `wire::a_hosted_workspace::no_request_on_either_door_writes_names_or_changes_the_hosted_workspace`; `hosting::tests::nothing_but_the_start_names_the_file_and_nothing_here_writes_it` |
| found and opened by task 18's request from a second daemon over loopback | `wire::a_hosted_workspace::a_machine_hosting_a_workspace_is_found_and_opened_by_a_second_daemon_over_loopback` |

## Verification

All executed under WSL Ubuntu (kernel 6.18, as root, as the loop runs), from
`/mnt/c/dev/alo-os-claude` with `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`:

- `cargo fmt --all -- --check` — exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo test -p alo-nearby` — exit 0: 150 unit, 9 + 12 + 4 + 4 + 5 + 9 + 1 integration,
  3 doctests (both `compile_fail` examples included).
- `cargo test -p alo-agentd` — exit 0: 344 unit, and every integration target.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-nearby -p alo-agentd --no-deps` — exit 0.
  (With `--document-private-items` both crates fail on redundant and broken private
  links that are on `main` already — `deliberating.rs`, `pairing.rs`,
  `answering.rs:192` — none introduced here.)
- Each evidence test run alone with `--exact` — each `1 passed`.

Not run: the whole-workspace suite (the supervisor's).

**Rebased onto `773313c`, re-gated.** The first handoff was refused at `git rebase
origin/main`: the measuring lane had published its own task 19 (*A turn shows a model
the words the product wrote*) in this plan while this branch — this task and task 18,
which had not been published either — held 19 for this task. Both sections are kept;
the lane that published first keeps the number, so this task is **20** and the one it
wrote is **21**, with the cross-references in this report and in task 18's renumbered.
No code changed in the resolution. The gates above were run again on the rebased tree.

## Not shown, and owed

- **Two physical machines.** Both ends ran on one host over loopback datagrams; whether a
  second machine on an office network hears the workspace over multicast is owed, as for
  tasks 1 and 17.
- **`alo-workplace`'s package** writing `/etc/alo/workspace.toml` and restarting the
  service is that repository's, against the new contract.
- **The hosting tests need root** to hold a root-owned file (the loop runs as root); a
  non-root run fails them with a sentence saying so rather than passing a test about
  root's file on somebody else's.
- **The person on the hosting machine is not shown** what their machine advertises, nor
  a refused file in their language — written as plan task 21.

## Proposed updates for the integration owner

- **CHANGELOG.md:** *An alo machine with a workspace server installed now tells the
  office so — under its own identity, at a port root's installer writes in
  `/etc/alo/workspace.toml` — and nothing more. Colleagues find and open it without
  typing an address; nothing an agent or a person sends the machine changes what it
  advertises.*
- **QUEUE.md / STATE.md:** v0.5 local-network plan task 20 done; task 21 (*The person
  sees what their machine says about itself on the network*) written and ready.
- **ROADMAP.md:** no box moves; the physical two-machine acceptance for workspace
  discovery stays owed.
