# A paired machine is spoken of by the name its person gave it

**Date:** 2026-09-14
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 16)
**Contributor:** Claude (Opus 5), in the `C:\dev\alo-os-claude` checkout
**Status:** ready for integration

## What changed, for a person

Until now every place alo OS spoke of a machine you are paired with — the
indicator that lights when a question goes down the corridor, the record of what
happened, a change another machine proposed, the list of pairings, the machine
chosen to answer your questions — named it by thirty-two hexadecimal characters
nobody chose. You can now give a paired machine a name from your own shell
(*the studio machine*), and all of those say that name instead. You can take the
name away again. The name is kept on your machine only: the other machine never
hears it, and nothing uses it to find, reach or trust a machine — that is still
the identity and the pairing. A name is refused, and nothing changes, when:

- the machine is not one you are paired with right now (never paired, revoked, or
  run out);
- what was named is not a machine identity (for example, you typed a name where
  the identity goes);
- the name is empty, longer than 64 characters, has a line break or another
  invisible control character in it, or is spelt like a machine identity.

Revoking a pairing takes its name away with it. A name survives a restart, beside
the pairing it belongs to. An agent cannot name a machine.

## What changed, in the code

| Where | What |
|---|---|
| `crates/alo-remembering/src/named.rs` (new) | `MachineName::checked` and `NotAName`: the rule a name is held to, one arm per part (empty, `TooLong`, `AControlCharacter`, `AnIdentity`), `LONGEST_NAME = 64` |
| `crates/alo-remembering/src/names.rs` (new) | `MachineNames` (one name per identity; `name`, `forget`, `only_those_paired`) and the file's TOML shape — `read` keeps only names of machines paired at the moment, `NotNames` refuses the whole file |
| `crates/alo-remembering/src/machine_names.rs` (new) | `THE_MACHINE_NAMES = /var/lib/alo/machine-names.toml`, `machine_names_remembered`, `machine_names_kept` — under `believing.rs`'s three rules, like the pairings file |
| `crates/alo-remembering/src/refusing.rs`, `lib.rs` | `NotRemembered::NotMachineNames`; exports and the crate map |
| `crates/alo-remembering/tests/a_machine_name_survives_a_restart.rs` (new) | a name read again after a restart beside its pairing; gone when the pairing was revoked or ran out; an unbelievable file refused whole |
| `crates/alo-protocol/src/asked.rs`, `person.rs`, `agent.rs` | `name-machine {machine, called}` and `clear-machine-name {machine}` → `FromAPerson::NameMachine` / `ClearMachineName`, `is_about_a_name`; refused on the agent's door as `NotForAnAgent` |
| `crates/alo-protocol/src/told.rs`, `to_a_person.rs`, `to_an_agent.rs` | `machine-named {machine, called?, became}` → `ToAPerson::MachineNamed`, `machine_named`, `became_of_naming`, `called`; `chosen-to-answer` gains `called?`; `chosen_to_answer(machine, called)` |
| `crates/alo-protocol/src/pairing.rs`, `lib.rs` | `Paired::called` (`skip_serializing_if`, `default`), `that_is_called`; `AfterNaming { Kept, KeptUntilARestart }` |
| `crates/alo-agentd/src/names.rs` (new) | `TheNames`: the names behind their own lock, `impl alo_corridor::Naming`; `named`, `forgotten`; `KeepingNames`, `NothingKeepsNames` |
| `crates/alo-agentd/src/keeping_names.rs` (new) | `TheNamesFile`, writing through `alo_remembering::machine_names_kept` |
| `crates/alo-agentd/src/naming_machines.rs` (new) | the door: identity → the rule → a pairing standing now, under the network's lock → held and written while that lock is held |
| `crates/alo-agentd/src/network.rs` | `TheNetwork::calling(TheNames)` and `names()` |
| `crates/alo-agentd/src/answering.rs`, `reaching.rs`, `pairing.rs` | dispatch; exhaustive matches; `pairing.rs` takes the name away on revoke and on a pairing kept afresh (`forget_the_name`) and puts `called` on the list |
| `crates/alo-agentd/src/hearing.rs` | a pairing kept from the wire also starts with no name |
| `crates/alo-agentd/src/choosing_to_answer.rs` | `chosen-to-answer` carries the name |
| `crates/alo-agentd/src/starting.rs`, `main.rs`, `refusing.rs`, `terms.rs`, `lib.rs` | `main` reads the names file after the pairings, against them; `until_stopped` takes `TheNames` and answers `Terms::naming` with `network.names()` instead of `NoNameYet`; `NotStarted::NoMachineNames` for a file that cannot be believed |
| `crates/alo-agentd/src/words.rs` | five sentences: `agentd.only-a-paired-machine-is-named`, `a-name-has-nothing-in-it`, `a-name-is-too-long`, `a-name-has-a-line-break`, `a-name-cannot-be-an-identity` |
| `crates/alo-agentd/src/corridor.rs`, `reaching.rs`, `questioned.rs`, `serving.rs`, `testing.rs` | the through-the-door tests; `reaching`'s and `serving`'s harnesses answer naming with `network.names()`; `the_studio_answering_and_keeping` keeps what crossed |
| `crates/alo-changing/src/door.rs` | the exhaustive match takes `MachineNamed` |
| `docs/contracts/machine-names-file.md` (new), `daemon-protocol.md`, `pairings-file.md`, `local-network-wire.md` | the file, the requests and answers, and *no name crosses the wire*, additively |
| `docs/autonomy/v0-5-the-local-network-plan.md` | task 16 marked done; task 17 written |

## Decisions, and why

1. **The names live in a file of their own, beside the pairings file, not in
   it.** The plan left this to the crate that owns it. `docs/contracts/pairings-file.md`
   already says a row has *no field for a name a person gave the machine*, because
   a row is exactly what two people confirmed; the plan's constraint says *a
   remembered pairing is exactly the row two people made*. A name is one person's.
   So `/var/lib/alo/machine-names.toml`, in `alo-remembering`, under the same three
   rules (`believing.rs`): not a link, owned by root or the reader, writable by
   nobody else, `0600`, replaced whole. That is *the trust the pairings file has*,
   and the grants and pairings files already share the rule, so nothing new about
   trust was invented.
2. **A name outlives nothing its pairing does not.** The reader takes the pairings
   and the moment and keeps only names of machines still paired; the writer cuts
   the list to paired machines each time; revocation takes the name away under the
   same lock; and a pairing *kept afresh* (from the person's door or the wire)
   starts nameless. The last one closes a gap the acceptance did not name: without
   it, a pairing that expired during a session and was then made again would come
   back wearing a name given under the old agreement.
3. **An unbelievable names file stops the daemon, like the pairings file.** A name
   decides nothing, and I considered starting with no names. I did not: whoever can
   write that file can put one machine's name on another machine's evidence, and a
   daemon that shrugged would make *somebody edited your names* look like *you
   named nothing* — the same argument `main.rs` makes for the pairings.
4. **The rule a name is held to is about what a person reads.** Empty, over 64
   characters, and control characters are refused because a name appears on
   one-line surfaces (a line break could draw a second line nobody wrote). A name
   spelt as a machine's identity — bare or `machine:<identity>`, in any case — is
   refused because a person reading an identity reads *that* machine: allowing it
   would let one machine be labelled as another in the record. The rule lives in
   `alo-remembering` so the door and the file cannot disagree; each arm has its own
   translated sentence, and no sentence has a gap (the crate's own rule).
5. **`TheNames` has its own lock, always taken after the network's.** The names are
   asked while `TheNetwork`'s lock is held (a verb's origin is named inside
   `hearing::on_the_verb_wire` under it), so they could not sit behind that lock
   without a deadlock. One order — network, then names; nothing holding the names
   lock reaches for the network's — makes a deadlock impossible, and changes to a
   name are made while the network lock is held so a revocation cannot land between
   *is it paired?* and the write.
6. **Clearing a name on a paired machine that has none succeeds**, answered
   `machine-named` with no `called`. The person asked for the machine to have no
   name; it has none. Clearing on an unpaired machine is refused, like naming.
7. **One answer for naming and clearing**, `machine-named`, with `called` absent
   after a clear and `became` saying `kept` or `kept-until-a-restart` —
   `AfterRevoking`'s honesty about the disk, for the same reason.
8. **The field is `called`**, matching `alo_corridor::Naming::called` and
   `alo_nearby::Origin::called`, which already carry the name through the code.
9. **The through-the-door tests go through the daemon's own request handlers**
   (`what_a_person_said`, then `what_an_agent_said` or the network's doorway), the
   level tasks 12 and 15 used. The indicator is read directly on the answering
   side (`Machine::showing` while the answer is leaving); on the asking side the
   turn owns the indicator, so the test reads the record's `Left` entry — which
   `alo_record::Entry::left` makes only from the departure the indicator handed
   out — and the answer's *came from* sentence.
10. **`NoNameYet` stays**, re-documented as what a test that is not about names
    hands in. Nothing a machine runs uses it now.

## Acceptance criteria, and the test behind each

| Criterion | Test |
|---|---|
| A request names a paired machine by its identity; the list carries the name beside the identity | `alo-agentd` `naming_machines::tests::a_paired_machine_is_named_and_the_list_carries_the_name_beside_the_identity` |
| Naming and clearing refused when what is named is not an identity | `alo-agentd` `naming_machines::tests::naming_something_that_is_not_an_identity_is_refused_and_names_nothing` |
| Naming and clearing refused when it is not a machine this one is paired with | `alo-agentd` `naming_machines::tests::naming_a_machine_this_one_is_not_paired_with_is_refused_and_writes_nothing` |
| Refused on the agent's door in the words an approval gets | `alo-agentd` `naming_machines::tests::an_agent_naming_a_machine_is_refused_as_an_approval_would_be_and_names_nothing`; `alo-protocol` `person::tests::naming_a_machine_is_a_persons_and_refused_to_an_agent` |
| Kept in the person's own file with the pairings file's trust, read again at start, across a restart | `alo-agentd` `naming_machines::tests::a_name_given_on_the_door_is_read_again_at_start`; `alo-remembering` `a_machine_name_survives_a_restart::a_name_kept_before_a_restart_is_read_again_after_one`; `a_machine_name_survives_a_restart::a_names_file_that_cannot_be_believed_is_refused_whole` |
| The indicator and record entry of a question down the corridor name it by that name; the name is never sent to the other machine, never used to find or dial one | `alo-agentd` `corridor::tests::a_question_down_the_corridor_names_the_machine_by_its_name_and_the_name_never_crosses` |
| A change a paired machine proposed names it by that name; the name proves nothing | `alo-agentd` `reaching::tests::a_change_from_a_paired_machine_names_it_by_its_given_name_and_the_name_proves_nothing` |
| A question answered for a paired machine names it by that name (indicator, departure, record) | `alo-agentd` `questioned::tests::a_question_answered_for_a_paired_machine_names_it_by_its_given_name` |
| `chosen-to-answer` carries the name beside the identity | `alo-agentd` `choosing_to_answer::tests::the_machine_chosen_to_answer_is_told_by_its_name_beside_its_identity` |
| A revoked pairing's name goes with it | `alo-agentd` `naming_machines::tests::a_revoked_pairings_name_goes_with_it`; `alo-remembering` `a_machine_name_survives_a_restart::a_name_whose_pairing_was_revoked_or_ran_out_is_not_read_again` |

Also added: `naming_machines::tests::a_name_the_rule_does_not_allow_is_refused_in_its_own_words`,
`a_name_that_could_not_be_written_stands_until_a_restart_and_says_so`,
`names::tests::*`, `keeping_names::tests::*`, `alo-remembering`'s `named::tests::*`,
`names::tests::*` and `machine_names::tests::*`, and the wire tests
`asked::tests::naming_a_machine_carries_its_identity_and_the_name_and_nothing_else`,
`to_a_person::tests::a_machine_named_is_told_to_the_person_and_not_to_an_agent`
and `pairing::tests::a_pairing_carries_its_machines_name_beside_the_identity`.

## Verification (executed)

Platform: WSL2 Ubuntu on the development machine,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`, run from
`/mnt/c/dev/alo-os-claude`.

- `cargo fmt --all`: applied; `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-agentd -p alo-protocol -p alo-remembering`: exit 0.
- `cargo test -p alo-remembering`: all pass (unit, and the three integration targets).
- `cargo test -p alo-protocol`: all pass.
- `cargo test -p alo-changing`: all pass.
- `cargo test -p alo-agentd`: all pass (unit, and the kernel-bounded integration tests).
- `cargo test -p alo-choosing --test no_agents_door_reaches_these_settings`: pass
  (it reads the daemon's sources, and `choosing_to_answer.rs` changed).
- Each evidence test run on its own with `--exact`: one passed each.
- The full workspace suite was **not** run here; the supervisor runs it.

No hardware acceptance applies: nothing here touches a device, and the plan keeps
this work screenless.

## Limitations

- The shell surface where a person types a name is `alo-shell`'s and outside this
  plan. What this task owes it is the requests, the answers and the fields.
- A pairing that runs out *during* a session keeps its name in memory until the
  next write, restart or re-pairing. It is shown nowhere meanwhile: every surface
  that names a machine requires a standing pairing (a proof, the list, a choice),
  and a pairing kept afresh starts nameless.
- A proposal waiting to pair has no name, by design: a name is for a machine you
  are paired with.

## Proposed updates to shared documents (for the integration owner)

- **CHANGELOG.md:** "You can give a machine you are paired with a name from your
  own shell. The indicator, the record, the list of pairings and the machine
  chosen to answer your questions use that name instead of a long identity. The
  name stays on your machine, never reaches the other one, and goes away when
  the pairing is revoked. An agent cannot name a machine."
- **docs/autonomy/STATE.md:** reference this report; task 16 of the local-network
  plan done, task 17 (*a self-hosted workspace on the network is found, not
  configured*) ready.
- **docs/autonomy/QUEUE.md / ROADMAP.md:** no box moves. ADR 0003's *visible* now
  holds for the local network's surfaces in words a person can read.
