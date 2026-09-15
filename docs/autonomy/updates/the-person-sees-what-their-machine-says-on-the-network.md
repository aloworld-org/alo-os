# The person sees what their machine says about itself on the network

**Date:** 2026-09-15
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 21)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the owner
**Status:** ready for integration

## What changed, for a person reading the changelog

An alo machine tells everything on the local network that it exists and — where a
workspace server is installed — that it hosts a workspace. A person can now ask their
own machine exactly what it is saying: its identity, the port its presence names, and
the workspace it announces, that it announces none, or why a workspace installed on it
is not being announced. That last case used to be a line in a service log; it is now a
sentence in the person's own language saying what to do — reinstall the workspace
server, restart, or ask whoever looks after the machine — and it names no file path,
owner or permission. Asking changes nothing: nothing is re-read, nothing new is
announced, nothing is written to the record, and there is still no setting for what a
machine announces. An agent cannot ask.

## What changed, in the code

### `crates/alo-protocol`

- `src/advertised.rs` (new): `Advertised { machine, port, workspace }` and
  `HostedWorkspace` — `hosts { port }` (a `NonZeroU16`), `hosts-none {}`, or
  `not-advertised` carrying a `Wording`. `deny_unknown_fields` on both, so an answer
  carrying an address, hostname, model list or path does not read.
- `src/asked.rs`: `Asked::Advertised {}` — carries nothing.
- `src/person.rs`: `FromAPerson::Advertised`; `src/agent.rs` refuses it as
  `NotForAnAgent`, the words an approval gets.
- `src/told.rs`, `src/to_a_person.rs`: `Told::Advertised`, `ToAPerson::advertised` and
  `ToAPerson::advertisement`; `src/to_an_agent.rs` refuses it as an answer for an agent.
- `src/lib.rs`: exports `Advertised`, `HostedWorkspace`.

### `crates/alo-agentd`

- `src/unhosted.rs` (new): `Unhosted` — `NotTheSystems`, `NotRead`, `NoUsablePort` —
  `Unhosted::of(&NotHosting)` (exhaustive over all eleven arms) and
  `Unhosted::said(strings)`.
- `src/words.rs`: three words, `agentd.a-workspace-here-is-not-the-systems`,
  `agentd.a-workspace-here-could-not-be-read`,
  `agentd.a-workspace-here-names-no-usable-port`, each with a translator's note; in
  `EVERY_WORD` (now 30).
- `src/hosting.rs`: `Hosted` — `Nothing`, `At(NonZeroU16)`, `Refused(Unhosted)`;
  `advertised` returns it, still handing the English refusal to the service log first.
- `src/wire.rs`: `Wire::hosting(Hosted)` keeps the group of a refused file;
  `Wire::advertising()` reads identity, port and workspace off the responder and what
  the start kept.
- `src/what_is_advertised.rs` (new): `Advertising` and `told(&Advertising, &Strings)`,
  and the acceptance tests.
- `src/pairing.rs`: `Nearby` gains `advertising: &Advertising`; `src/serving.rs` hands
  it in from the wire; `src/answering.rs` dispatches `FromAPerson::Advertised`;
  `src/reaching.rs` and `src/pairing.rs` gained their exhaustive arms.
- `src/testing.rs`: `nothing_is_advertised()` for tests not about this; the existing
  `Nearby` constructions in `answering`, `choosing_to_answer`, `corridor`,
  `listing_workspaces`, `naming_machines`, `opening_workspaces`, `pairing`,
  `questioned`, `reaching` and `wire` pass it.
- `src/main.rs`, `src/lib.rs`: comment; module registration and exports.

### `crates/alo-changing`

- `src/door.rs`: the one arm its exhaustive match on `ToAPerson` needed.

### Contracts

- `docs/contracts/daemon-protocol.md`: `advertised`, new, additively.
- `docs/contracts/hosted-workspace-file.md`: the person is told either way, additively.

## Decisions

**What the answer is made from: the wire, handed in on `Nearby`.** The constraint says
the daemon reports what `Wire` holds, never the file. `Wire` already held the identity,
the presence port and the workspace port (inside `alo_nearby::Answering`); it did not
hold *why* nothing was advertised, because `hosting::advertised` turned a refusal into
`None`. So `advertised` now returns `Hosted`, which keeps the refusal's group, and
`Wire::hosting` takes it. The person's door reaches it through `Nearby`, the value the
dispatch already carries for "this machine's neighbours" — what this machine tells
those neighbours belongs beside them — rather than through `LookingFor`, which is about
looking and would have given that trait a second reason to change.

**The wire keeps the group, not the refusal.** `NotHosting` names the path and carries an
`io::Error`. Keeping `Unhosted` rather than `NotHosting` means the running service holds
nothing that could leak the path into an answer, and the full English still goes to the
service log once, at start, exactly as task 20 wrote it.

**Three groups, by what a person can do.**
- *Not the system's* — `ALink`, `NotAFile`, `NotRoots`, `WritableByOthers`: something
  other than the system could have written it; reinstalling the server writes it again.
- *Could not be read* — `NotRead`: restart, then reinstall.
- *Names no usable port* — `NotTheShape`, `AnotherKey`, `NoPort`, `NotANumber`,
  `NotAPort`, `TheWiresOwnPort`: the server's package wrote something wrong; reinstall
  or update it.
A person cannot fix a mode or an owner themselves on a machine they did not set up, so
more groups would be more sentences with the same action. None of the sentences says
"root": the test forbids "root", "uid", "mode", digits of modes and ports, the path and
the file name.

**Wire shape.** `advertised` answers with three fields only. The workspace is an enum
rather than an optional port plus an optional sentence so that "hosts a port *and* was
refused" cannot be written. `hosts-none` is an empty object, the idiom `workspaces {}`
already uses. A workspace port of `0` does not read.

**Word names.** The first names began `THE_HOSTED_WORKSPACE_…`, and the existing
`hosting::tests::nothing_but_the_start_names_the_file_and_nothing_here_writes_it` (which
looks for that string in shipped code) rightly failed. They are `A_WORKSPACE_HERE_…`
now; the test was not touched.

**Next task written.** The plan named none after 21. Task 22, *A machine on two networks
is found on each of them*: `Wire::bound` joins the discovery group with
`Ipv4Addr::UNSPECIFIED`, so the kernel picks one interface and a docked laptop on wired
and Wi-Fi is found on only one. It is inside *Machines find each other with zero
configuration* and adds no setting.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| a read-only request answering identity, presence port and the workspace port or none, with no field for anything else, tested by the answer's shape | `alo-protocol` · `lib` · `advertised::tests::the_answer_carries_the_identity_the_port_and_the_workspace_and_nothing_else`; `advertised::tests::an_answer_carrying_anything_the_advertisement_does_not_is_not_read`; `to_a_person::tests::what_this_machine_advertises_is_told_to_the_person_and_not_to_an_agent`; `alo-agentd` · `lib` · `what_is_advertised::tests::the_person_is_told_the_identity_the_port_and_the_workspace_and_nothing_else` |
| a refused file is said in the person's language, naming no path, owner or mode, for each `NotHosting` arm grouped | `alo-agentd` · `lib` · `unhosted::tests::every_refusal_of_the_workspace_file_is_grouped_into_what_a_person_can_act_on`; `unhosted::tests::what_the_person_is_told_names_no_path_owner_or_mode`; `what_is_advertised::tests::a_refused_workspace_file_is_told_in_the_persons_words_naming_no_path_owner_or_mode` |
| the request changes nothing — no file re-read, nothing newly advertised, nothing recorded | `alo-agentd` · `lib` · `what_is_advertised::tests::asking_reads_no_file_again_advertises_nothing_new_and_writes_nothing` |
| an agent sending it is refused in the words an approval gets | `alo-agentd` · `lib` · `what_is_advertised::tests::an_agent_asking_what_this_machine_advertises_is_refused_as_an_approval_would_be`; `alo-protocol` · `lib` · `person::tests::asking_what_this_machine_advertises_is_a_persons_and_refused_to_an_agent` |
| a request carrying a port or a path is not a request | `alo-protocol` · `lib` · `person::tests::asking_what_is_advertised_with_a_port_or_a_path_is_not_a_request`; `alo-agentd` · `lib` · `what_is_advertised::tests::a_request_carrying_a_port_or_a_path_is_refused_and_changes_nothing` |

The strings are declared in `alo-agentd`'s `EVERY_WORD` and loaded by the fixture that
loads the machine's whole vocabulary (`testing::in_english`); the tests assert none of
the three is a missing key. `alo-agentd`'s words are declared by the crate itself rather
than collected by `alo-saying` (`crates/alo-saying/src/collecting.rs` gives the reason),
which is unchanged.

## Verification

Run in WSL (Ubuntu, as root, `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`),
from `/mnt/c/dev/alo-os-claude`:

- `cargo fmt --all -- --check` — exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo test -p alo-protocol -p alo-changing` — all passing (protocol lib 125, plus
  integration and doc tests; changing 16 + 14).
- `cargo test -p alo-agentd` — lib 353 passed, and every integration target passed,
  including the kernel-bounded ones.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-agentd -p alo-protocol -p alo-changing` — clean.
- Each test in the table run alone with `--exact` — each passed.

Not run here: the full workspace suite (the supervisor runs it). Not shown: a real
machine on a real office network, and a shell drawing the answer — there is no shell
surface for it yet (nothing in `alo-shell` by the plan's terms).

## Remaining limitations

- No shell draws `advertised` yet; the answer exists for one to.
- A machine on more than one network advertises on the interface the kernel chose
  (task 22).

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "A person can ask their machine what it tells the local network
  about itself — its identity, its port, and the workspace it announces or why one
  installed is not announced — in their own language, without anything being changed."
- **STATE.md:** reference this report; v0.5 local-network task 21 done; task 22 written.
- **QUEUE.md / ROADMAP.md:** no box ticks; the promise *discovered, not configured*
  remains owed on physical machines.
