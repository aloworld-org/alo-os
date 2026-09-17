# Network, through the broker

**Date:** 2026-09-16
**Workstream:** v0.5 the broker and the disk — task 3 of
`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` (`ROADMAP.md`: ★ *System verbs
through the privileged broker*, and *Corporate proxy support, machine-wide*)
**Contributor:** Claude, as a worker in `C:\dev\alo-os-2`
**Status:** ready for integration. The code and its tests, run in WSL. Nothing has
run on certified hardware; the image carries neither the broker nor NetworkManager;
no turn offers these verbs yet; and the shell neither draws the Wi-Fi password
surface nor registers the secret agent yet. Each is said below.

## What changed, for a person

alo OS can now join a Wi-Fi network, forget a saved one, turn Wi-Fi on or off, and
set the machine's proxy through the one part of the system allowed to change
settings for the whole machine.

An assistant can propose the first three by the network's own name. The sentence a
person approves also says what the change does to the conversation they are
having, for example *turn Wi-Fi off, and this conversation loses its connection
until this machine is connected again*. The machine checks that this is true before
the person is asked, and again before the change is made. If it is not true, the
change is not made.

**A Wi-Fi password never goes through the assistant.** Joining a network that asks
for one makes the machine ask the person for it themselves, in their own session.
The only thing answered with it is the machine's network settings. No verb has
anywhere to put a password.

A person does all of this in Settings too. That takes the same road into the
machine and is written into the same history, marked as a change they made
themselves. The proxy is set by a person only: an assistant cannot propose one.
If no network has the approved name, or two do, nothing is changed. That includes
an open network using the name of a protected one next door.

## What changed, in the repository

**`crates/alo-networks`** (new). What a network is, as NetworkManager reports it:

| File | What it holds |
|---|---|
| `src/reported.rs` | `NetworkName` (the bytes announced; `called()` only when it is text), `Protection` (open, password, organisation sign-in), `Visible`, `Saved`, `Primary`, `TheNetworks`; `as_reported()` is what an identity is digested from: name **and protection** for a network in range, the network manager's identifier for a saved one |
| `src/service.rs` | `Networks` (what there is now) and `NetworkService` (join, forget, switch Wi-Fi); `NotAnswering`, `NotDone` in English for logs |
| `src/bus.rs` | NetworkManager's published D-Bus names, one method call and one property read/write |
| `src/network_manager.rs` | `NetworkManager` over `unix:path=/run/dbus/system_bus_socket` (never an environment variable): one visible network per name and protection through its strongest access point, saved Wi-Fi connections only, the primary connection; join activates a saved connection or `AddAndActivateConnection` with **no settings of ours**, then waits until activated or failed (90 s); forget is `Delete`; Wi-Fi is `WirelessEnabled`; 30 s per call; `OnThisMachine` connects afresh per request |
| `src/secret_agent.rs` | `SecretAgent` implementing `org.freedesktop.NetworkManager.SecretAgent`: answers `GetSecrets` only for the connection that owns the network manager's name **and** runs as its user, only when interaction is allowed, only for `802-11-wireless-security`; asks `ThePersonsOwnSurface`; `registered(address, agent)` serves then registers |
| `src/password.rs` | `WifiPassword`: 8–63 printable ASCII or 64 hex, no `Display`/`Clone`/`Serialize`, one crate-private way out |
| `src/proxy_file.rs` | `THE_MACHINES_PROXY` (`/etc/alo-proxy/proxy.json`), `THE_WANTED_PROXY` (`/run/alo-broker/wanted/proxy.json`), the bytes of each, and `rechecked`/`kept_on_this_machine`, which rebuild every value through `alo-proxy`'s checked constructors and refuse anything that does not come back identical |

**`crates/alo-brokerd`** (new). The broker's process, which task 1 left to the
first verb carried out:

| File | What it holds |
|---|---|
| `src/starting.rs`, `src/describing.rs`, `src/recording.rs` | Taken from the held task 2 branch, changed only to take the carriers after the logins are read and to make `/run/alo-broker/wanted` `0770` in the broker's group |
| `src/carrying.rs` | `Carriers`: the four network verbs to their carriers; printers, updates and storage refused `not-carried` by name, no wildcard |
| `src/network.rs` | `Network<S>`: join, forget, radio against what the network manager reports now; no match, two matches, or an organisation sign-in is `not-carried` |
| `src/proxy.rs` | `Proxy`: opens the handed-over file `O_NOFOLLOW`, believes only a plain file owned by the person of at most 64 KiB, requires its digest to be the approved identity, rebuilds it, refuses over an organisation's proxy, writes the machine's file whole at `0644` |
| `src/main.rs`, `alo-brokerd.service` | The thin process; the unit (`User=root`, `Group=alo`, both capability lines empty, `NoNewPrivileges`, `Wants=`/`After=NetworkManager.service`, `RuntimeDirectory=alo-broker` 0750, `StateDirectory=alo-broker` 0700, `ConfigurationDirectory=alo-proxy` 0755) |

**`crates/alo-changing-network`** (new). The agent's verbs and the one road:

| File | What it holds |
|---|---|
| `src/verbs.rs` | `join_network`, `forget_network` (`network`: name ≤ 32), `switch_wireless` (`wireless`: `on`/`off`); each also `this_conversation` (`keeps_its_connection`/`loses_its_connection`), named in the sentence; all `change`, `Requires::nothing_because`; `wanted(&Call)`, `approved(&Authorised)` |
| `src/cutting.rs` | `Answered` and `would`: a change loses the conversation's connection only when it is answered over the network and the change takes the machine off the Wi-Fi network it sends through |
| `src/proposing.rs` | `proposable`: refuses a call whose `this_conversation` is untrue, or whose network is not exactly one |
| `src/choosing.rs` | `chosen`: exactly one network by name, or `NoneVisibleCalled`, `NoneSavedCalled`, `MoreThanOneCalled`, `AsksForASignIn` |
| `src/by_hand.rs` | `Listed::in_range`, `Listed::saved`, `picked()`: a person's pick as the same broker verb, including a network no sentence could name |
| `src/carrying_out.rs` | `TheBroker`, `carry_out_approved` (re-checks what the change does to the conversation before asking), `carry_out_by_hand`, `set_proxy_by_hand` |
| `src/refusing.rs`, `src/words.rs`, `src/wanted.rs` | `NotChanged` and its sentences, `changed_said`, `proxy_set_said`; 31 strings with translator's notes; `Change`, `ThisConversation`, `Wanted` |

**`crates/alo-broker`** (the door). Taken unchanged from the held task 2 branch,
because none of it touches printers: `src/place.rs` (`THE_DOOR`, `THE_KEY`),
`src/handing_over.rs`, `src/asking.rs`, `BY_HAND` and `fresh_bytes` in
`src/approving.rs`, the line ceilings raised to 1,000 and 2,450, the race fix in
the door test's helper, and the two tests `the_key_reaches_the_turn_and_nobody_else`
and `the_turn_asks_at_the_door`. Doc lines now name the network as the first verbs
carried out.

**Registrations:** `Cargo.toml` members (three); `alo-saying` collects the new words
(46 lists); `alo-by-hand` is handed the new verbs (9 declaring crates);
`docs/by-hand.md` answers all three verbs from *Settings, as one place* and the
status area.

**Documents:** `docs/decisions/0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md`
(new, accepted with this task as ADR 0042 was); `docs/contracts/machine-proxy-file.md`
(new); `docs/contracts/agent-verbs.md` (*The network verbs*; the broker's process,
door, key, `BY_HAND` and network carriers; the verb classes table; the declaring
crates); the plan marks task 3 done and records what tasks 2 and 4 inherit.

## Decisions, and why

1. **The broker's process arrived with this task, taken from the held task 2
   branch.** Task 1 left the process to the first verb carried out, and task 2 is
   held back for changing `alo-printing`, so the network verbs are the first. The
   parts of that branch that do not touch printers (process start, record, unit,
   key hand-over, asking side) were taken **unchanged**, so that when task 2 is
   released the conflict is the same files with the printers' carrier beside the
   network's in `Carriers`, not two designs. The plan now says so.
2. **Three new crates, although the plan names only `alo-broker` and
   `alo-encrypting`.** This follows task 2's reasoning. The door stays auditable
   only if everything that carries a verb out lives outside it (`alo-brokerd`).
   What a network is must be shared by the privileged side and the person's side,
   and must not depend on either (`alo-networks`, the counterpart of
   `alo-printing`). The agent's verbs and road are `alo-changing-network`. This is a
   deviation, flagged for the integration owner.
3. **A network's identity is its name and its protection** (ADR 0049 §1). Without
   the protection, an open network announcing a protected network's name would be
   the same identity. With it, the two are two networks. A name alone then matches
   two, and nothing is joined.
4. **No proxy verb for the agent** (ADR 0049 §3). The broker has `network.set-proxy`,
   and the task requires it. A person reaches it from Settings. An agent proposing
   the address every connection goes through is the approval defending against an
   attack aimed at the approval. The door takes no text, so the proxy is handed over
   as a file in a folder the broker makes for the person's group, and asked for by
   the digest of its bytes.
5. **The Wi-Fi password is asked for by NetworkManager's own secret-agent
   protocol** (ADR 0049 §4). A password argument is forbidden. A password handed to
   the broker would give the door a text field. The agent answers only the name's
   owner running as the network manager's user. Without that check, the agent's own
   login could send `GetSecrets` to the person's connection and be handed what they
   typed.
6. **What a change does to the conversation is an argument the sentence names**
   (ADR 0049 §5). What a person approves is the sentence, so a warning drawn beside it
   would not be what was approved and would not be in the record. The value is
   checked both ways before proposing, and again before carrying out. A conversation
   answered on this machine is never told it will lose its connection.
7. **ADR number 0049.** 0047 went to the fine-tuning decision and 0048 to the adapters decision, both published while this task was in flight.
   Taking it here would collide when that branch is published.
8. **Joining waits until joined.** NetworkManager accepts an activation at once, so
   answering `carried` then would tell a person they are connected while the
   password is still being asked for. The client waits until the connection is
   activated or has failed, for up to 90 s. The asking side's 180 s wait (from task 2)
   covers that.
9. **A bus-library race found and designed around.** In gating, the secret-agent bus
   test hung 4 times in 15 runs. The cause: a call arriving at an object added to a
   connection that is already answering can be missed. `registered` now builds its
   connection with the agent already served and registers afterwards. The test
   builds every serving connection the same way, and each test connection has a
   20 s method timeout, so a regression fails instead of hanging. After the change
   the test passed 40 of 40 runs. Every NetworkManager call has a 30 s timeout, so a
   stuck service is a change not made, never a broker waiting for ever.
10. **Enterprise (802.1X) networks are not joined in v0.5**, and there is no VPN
    verb. Both are recorded in ADR 0049 §6.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| The broker's network verbs take closed types: a network by the identity the network manager reported | `alo-brokerd` `only_the_network_approved_is_changed` `each_network_verb_changes_exactly_what_was_approved` (refusals: `a_network_not_reported_now_is_not_changed_and_nothing_else_is`, `two_networks_answering_to_one_identity_are_neither_and_a_sign_in_is_not_joined`, `a_verb_that_is_not_the_networks_is_not_carried_out_here`, `a_network_change_under_no_genuine_approval_changes_nothing`) |
| …against NetworkManager's own interface, rented | `alo-networks` `the_network_manager_on_a_bus` `each_change_is_made_against_exactly_what_the_network_manager_reported` (and `the_network_manager_reports_what_it_sees_has_saved_and_sends_through`, `a_join_that_did_not_happen_is_not_said_to_have_happened`) |
| Set the proxy `alo-proxy` holds | `alo-brokerd` `the_proxy_set_is_the_proxy_handed_over` `the_proxy_handed_over_becomes_the_machines_as_the_persons` (refusals: `only_a_plain_file_of_the_persons_is_read`, `a_proxy_changed_after_it_was_approved_is_not_set`, `a_setting_alo_proxy_would_not_have_made_is_not_set`, `an_organisations_proxy_is_not_replaced_by_a_persons`) |
| A Wi-Fi password never passes through the agent: the verbs' arguments have no password field | `alo-changing-network` lib `verbs::tests::no_network_verb_has_anywhere_to_put_a_password` |
| …joining a protected network asks the person in a surface the agent cannot read | `alo-networks` `the_network_manager_on_a_bus` `the_password_is_given_to_the_network_manager_and_nobody_else` |
| A change that would cut a turn's own connection says so before it is approved | `alo-changing-network` `the_network_changes_only_through_the_broker` `a_proposal_wrong_about_this_conversation_is_never_put_to_the_person`, `an_approved_proposal_joins_exactly_that_network_and_said_what_it_cuts`, `an_approval_no_longer_true_about_this_conversation_changes_nothing` |
| No VPN configuration verb in v0.5 | `alo-changing-network` lib `verbs::tests::no_verb_configures_a_vpn_sets_a_proxy_or_lists_networks` |
| A person does the same by hand (ADR 0009) | `alo-changing-network` `the_network_changes_only_through_the_broker` `a_person_in_settings_changes_the_network_through_the_same_verbs`; `alo-by-hand` `every_verb_can_be_done_by_hand` `every_verb_this_machine_ships_can_be_done_by_hand` |
| A token under any key but the broker's changes nothing | `alo-changing-network` `the_network_changes_only_through_the_broker` `a_token_under_any_key_but_the_brokers_changes_nothing` |
| Inherited: the process, its record, its unit | `alo-brokerd` `the_broker_starts_only_as_it_must` `a_broker_started_as_it_must_be_writes_down_what_it_answers_on_the_disk`, `a_broker_in_roots_group_or_the_agents_opens_no_door_and_hands_over_no_key`; `the_unit_is_the_process` `the_broker_runs_as_root_in_the_persons_group_holding_no_capability` |
| Inherited: the key reaches the turn and nobody else | `alo-broker` `the_key_reaches_the_turn_and_nobody_else` `a_key_anybody_else_could_have_written_or_read_is_not_believed` |
| The broker stays auditable | `alo-broker` `small_enough_to_audit_in_an_afternoon` `the_broker_is_no_longer_than_an_afternoon` |

## Verification

Run in WSL Ubuntu on this machine, as root, with target
`$HOME/alo-builds/alo-os-2-72aa7fda7f7de151`:

- `cargo fmt --all` then `cargo fmt --all --check`: clean.
- `cargo clippy -p alo-broker -p alo-brokerd -p alo-networks -p alo-changing-network -p alo-saying -p alo-by-hand --all-targets -- -D warnings`: passed.
- `cargo test -p alo-broker -p alo-brokerd -p alo-networks -p alo-changing-network -p alo-saying -p alo-by-hand`: all passed (alo-networks 10 + 4; alo-brokerd 2 + 19; alo-changing-network 20 + 7; alo-broker, alo-saying, alo-by-hand all passed).
- `cargo test -p alo-networks --test the_network_manager_on_a_bus`: 40 consecutive runs, all passed, after the fix in decision 9.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-broker -p alo-brokerd -p alo-networks -p alo-changing-network`: passed.
- `cargo test -p alo-citing -p alo-collected` (not edited; they check the new ADR's citations and the new words' collection): passed.

The broker's `src/` is 2,207 lines, within the ceiling of 2,450.

**Not run:** the whole-workspace suite, which the supervisor runs. **Nothing ran on
certified hardware.** The network manager in every test is a stand-in: in memory
for the broker and road tests, and a `zbus` service with NetworkManager's name,
objects and interfaces on a private `dbus-daemon` for the client and secret agent.
It is not NetworkManager. **Not measured:**

- whether NetworkManager asks a secret agent in the person's session for the
  secrets of a connection that root activated;
- whether polkit allows root with an empty capability set to call
  `AddAndActivateConnection`, `Delete` and set `WirelessEnabled`;
- the exact `Flags`/`WpaFlags`/`RsnFlags` a real access point reports.

These are the first things to check on a booted image, and they belong in
`docs/quirks.md` whichever way they answer.

### Second pass, after the gates refused the first handoff

The whole-workspace suite refused the first handoff twice, both times at
`alo-software`'s `the_terminal_is_a_persons_and_no_verb_reaches_it`. That test
reads the workspace to find every crate that declares verbs and holds the list
to the verbs it hands the terminal's check. `alo-changing-network` declares
verbs, and the test had not been given them. The fix is the one the test asks
for, and it weakens nothing. `alo-changing-network` is now a dev-dependency of
`alo-software` and is on the test's list, and its three verbs are declared into
the `Verbs` the check walks. None of the three names an application, so none
of them can reach the terminal. The check now proves that for them as it does
for every other verb, rather than their being left out.

That pass ran these in WSL on the same target:

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --no-fail-fast -p alo-software -p alo-changing-network -p alo-brokerd -p alo-broker -p alo-networks -p alo-by-hand -p alo-saying`: all passed.

One run of `alo-software` failed once in
`the_proxy_on_the_road_out::the_program_the_tool_starts_really_receives_the_proxy_and_nothing_of_ours`
with *Text file busy (os error 26)*. That test is already on `main`, and this
change does not touch it. It writes a script to a fixed path in `/tmp` and
starts it, and another cargo process was running on the machine at the time.
The next run passed, and the error was not reproduced after that. It is recorded
here because it is flaky. A per-process path for that script would remove the
race, and it belongs to the proxy work's owner, not to this change.

## Limitations and follow-ups

- **No turn offers these verbs yet.** This is the same as `print_document` and
  `install_application`. `alo-turn` and `alo-agentd` are not this plan's to edit.
  **Proposed** queue item: hand a network verb's call to `proposable` with the
  turn's `Answered`, and its redeemed approval to `carry_out_approved` with
  `OnThisMachine` and `TheBroker::on_this_machine()`.
- **The shell** draws the password surface (`ThePersonsOwnSurface`), calls
  `secret_agent::registered(THE_SYSTEM_BUS, …)` at sign-in, and gives Settings its
  network pane (`Listed`, `carry_out_by_hand`, `set_proxy_by_hand`). **Proposed**
  for the shell lane. Until the agent is registered, joining a protected network
  with no saved password is `not-carried`.
- **The image.** **Proposed** for the image lane: install NetworkManager, build
  `--package alo-brokerd`, copy it to `/usr/libexec/alo-brokerd` and the unit into
  `image/usr/lib/systemd/system/`, and teach `alo-image` to hold the unit.
- **Nothing reads the machine's proxy file yet.** `alo-software`, `alo-updating` and
  `alo-models` are handed a `TheProxy` by their callers. **Proposed**: each reader
  reads `/etc/alo-proxy/proxy.json` through `alo_networks::proxy_file::kept_on_this_machine`.
- **Held task 2 conflicts with this** in `alo-brokerd` (`lib.rs`, `main.rs`, the
  unit and its test, `starting.rs`) and in `alo-broker`'s `lib.rs`/`keeping.rs` doc
  lines. The resolution is to add `Printers` to `Carriers` and keep both carriers'
  documentation.
- **Plan crate lists.** `alo-saying` and `alo-by-hand` were edited for registration
  only, as task 2 did. The plan lists `alo-saying` as read-only.
- **Carried from task 1:** the kernel cannot tell `alo-agentd` from another program
  the person runs. The `SO_PEERPIDFD` cgroup narrowing is still proposed.

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "alo OS can now join and forget Wi-Fi networks, turn Wi-Fi on
  or off, and set the machine's proxy through the one part of the system allowed to
  change settings for the whole machine. An assistant proposes a network change by
  the network's name, and the sentence the person approves says whether the
  conversation will lose its connection. The machine checks that this is true. A
  Wi-Fi password is only ever asked of the person, never of the assistant. A person
  does the same in Settings, on the same road, and only a person sets the proxy.
  Nothing has run on a real network yet."
- **ROADMAP.md:** under ★ *System verbs through the privileged broker*, the network is
  done (the code). The broker's process exists and is not yet in the image. Under
  *Corporate proxy support*, the machine-wide file and its writer exist, and its
  readers are owed.
- **QUEUE.md / STATE.md:** broker plan task 3 done (ADR 0049 accepted with it).
  Task 4 adds its carriers to `alo_brokerd::Carriers`. Task 2, when released,
  resolves against this change. New items: offer the network verbs from a turn;
  register the secret agent and draw the password surface in the shell; put the
  broker and NetworkManager in the image and measure the three unmeasured points;
  read the machine's proxy file on every road out.
