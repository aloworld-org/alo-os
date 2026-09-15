# Portal requests judged as grants

**Date:** 2026-09-15
**Workstream:** v0.5 — applications, and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`), task 1:
*A portal request is a grant, and is refused like one*.
**Contributor:** Claude Code worker under the kernel-loop supervisor (Mac lane,
gates in the `alo` Lima VM).
**Status:** ready for integration.

## What changed

Built on [ADR 0040](../../decisions/0040-what-an-applications-grant-is-over.md)
(accepted 2026-09-15, option C, all four parts). Its four parts went in first,
additively, then the portal crate.

### `crates/alo-capability` — ADR 0040, parts 1–3

- **`src/facility.rs`** (new): `Facility`, the closed list of eleven things a
  machine has that are not a path. They are the camera, the microphone, the
  screen once, the screen continuously, notifications, the clipboard, the
  desktop background, appearance settings, sleep, network state and power
  profile. Each has a name it is written down by and a word. The v1 portals are
  absent.
- **`src/reach.rs`**: `Reach::Facility` and `Ask::Facility`. A facility is
  matched exactly and covers nothing of any other kind.
- **`src/grantee.rs`** (new, moved out of `grant.rs`): `Grantee` is an agent or
  an application. An agent and an application with the same name are two
  grantees. An agent still serialises as its bare name. An application is
  written as `{ "application": … }`. `Applicant` is the application's identity
  when it asks.
- **`src/grant.rs`**: `Grant::checked_for(&Grantee, …)`. `Grant::checked` now
  calls it for an agent. Two new refusals: `GrantError::NoApplicationNamed`,
  and `GrantError::NotForAnAgent` for a facility granted to an agent.
- **`src/allowing.rs`** (new): `Grants::allowing(&Applicant, &Ask, now)` and
  `Grants::allows_anything`. The refusal is `NotAllowed` (`Lapsed` / `Never`),
  and its words never say *agent* or *folder*.
- **`src/grants.rs`**: `permitting` and `allowing` share one crate-private
  search. `permitting` refuses an application's grantee.
- **`src/agent.rs`**: `Agent::Declined` now holds the applications' grants.
  - `declining` ends every agent's grant, keeps every application's grant with
    its handle, and counts only the agent's grants.
  - `accepting` brings back the applications' grants and none of the agent's.
  - New: `allowed()`, `allow(grant)` (refuses an agent's grant in both states),
    `revoke_allowed(id)` (touches only an application's grant) and
    `allowing(…)`.
  - `grants_mut()` still answers `None` on a declined machine.
- **`src/words.rs`**: 15 new words (2 grant refusals, 11 facilities, 2
  application refusals).

### `crates/alo-remembering` — ADR 0040, part 4

- **`src/written.rs`**: the grants file reads formats 1 and 2.
  - Format 2 adds `applicant` and `facility`.
  - Each grant names exactly one grantee and exactly one reach.
  - **The lowest format that holds the list is written.** A list with no
    application's grant is written byte for byte as before.
  - A format-1 file naming a format-2 key is refused.
- **`src/refusing.rs`**: `NewerThanItsFormat`, `GrantedToNobody`,
  `GrantedToTwo`, `NoSuchFacility`. `AnotherFormat` names the formats it
  reads.
- **`docs/contracts/grants-file.md`** (new): the file's contract. There was
  none, and the format is a public surface.

### `crates/alo-portals` (new) — the task

- `Portal`: the fifteen v0.5 portals, closed, each with a sentence.
  `Portal::over` is ADR 0040's table as a `match`: four portals are about a
  file and eleven are about one facility each.
- `Request::of` (facility portals), `Request::over` (path portals) and
  `Request::opening_with` (the file and the opener). What arrives is checked
  first (`NotARequest`): an identifier with no spaces or control characters
  and at most 255 bytes, and a full path with no `..`.
- `Request::judged(&Grants, now)` checks two things in order:
  1. `Grants::allows_anything`. If false, the answer is
     `Refused::NothingGranted`, before anything the request is for is looked
     at.
  2. `Grants::allowing` for each ask. A refusal is
     `Refused::NotAllowed { why: alo_capability::NotAllowed }`.

  If both pass, the answer is `Allowed` with the grant behind each ask.
- Words collected by `alo-saying` (`Cargo.toml`, `collecting.rs`). The crate is
  a workspace member.

### Retired

- `crates/alo-granted/tests/a_portal_grant_waits_on_its_decision.rs`. It held
  the ADR in place while the ADR was *proposed*, and its own report asked the
  worker who built task 1 to remove or rewrite it. What still matters moved to
  `crates/alo-portals/tests/a_portal_request_is_a_grant.rs`: the features line
  held whole, a single row per portal in ADR 0040's table, and the **none**
  rows matching the facility portals exactly.

### In words a person outside this repository can read

Applications on alo OS can now be granted things the way agents are: the camera,
the microphone, the screen, notifications, or a file. Both live in the same list
of permissions. An application asking for something it was not granted is
refused. An application nobody has granted anything is refused without a
question ever being put on the screen. Turning the AI agent off no longer takes
away what applications were allowed, so a video-call application keeps its
camera. Agents can never be granted the camera or the screen. The file where
permissions are kept gained a second format. It is used only when an
application holds a permission, so rolling back an update keeps every folder
granted to the agent.

## Decisions made here

- **Two doors onto one search, not one door with two refusals.** ADR 0040
  says the portal judges with `Grants::permitting`. That would need
  `NotGranted` to learn a new variant or field. `NotGranted` is matched
  exhaustively and built in `crates/alo-turn/src/arriving.rs`, which is lane
  A's crate and which this plan may not edit. So applications ask through
  `Grants::allowing`. It runs the same crate-private search and returns its
  own refusal type. Nothing about coverage, expiry or identity is written
  twice. The agent's door refuses an application's grantee, so a grant made to
  an application can never authorise an agent. The part of the ADR that
  matters holds: one list, one search, and no refusal that calls an
  application an agent.
- **A facility is never granted to an agent** (`GrantError::NotForAnAgent`).
  No agent could be granted one before, because there was no such reach. So
  this narrows nothing. It keeps *context is offered, never watched* true in
  the type.
- **`Agent::Declined` holds the applications' list rather than a second
  store.** Handles stay unique across both kinds, so revoking by handle stays
  one mechanism, and task 2 can list both kinds from one place. The agent's
  road (`grants_mut`) is still absent on a declined machine.
- **The lowest format is written.** Always writing format 2 would make every
  rollback past this change drop the whole grants file. Writing format 1 when
  it is enough costs one branch.
- **Unknown facility names get their own refusal**
  (`NotRemembered::NoSuchFacility`), not a parser error. The file stores the
  name as text and resolves it with `Facility::by_name`.
- **Open-with names two asks**, the file and the opener, and both must be
  granted. A request cannot ask one portal for another portal's facility,
  because a facility portal's request names nothing.
- **Judged against `&Grants`, not `&Agent`.** The daemon holds a `Grants` read
  off the disk. A machine value gives its list through `Agent::allowed()`.
- **ADR 0040 is left as accepted.** Its sentence about the hold test describes
  the time before the decision, and rewriting an accepted decision is not a
  worker's call.

## Acceptance, and the test behind each clause

| Criterion | Test |
|---|---|
| The v0.5 portals are a closed enum, each with a sentence in the vocabulary; v1 absent | `alo-portals` `a_portal_request_is_a_grant::the_portals_are_the_closed_list_the_promise_names` |
| Evaluated like a verb: against a grant naming application and reach, refused outside it, never widened | `…::a_request_is_judged_against_a_grant_naming_the_application_and_the_reach` |
| Reuses `alo-capability`'s types rather than restating its rules | `…::the_evaluation_is_alo_capabilitys_and_is_not_restated_here` |
| An application nobody granted anything is refused before any dialog | `…::an_application_granted_nothing_is_refused_before_any_dialog` |
| What a request is for is the same `Reach` an agent's grant names; one list holds both | `…::what_a_request_is_for_is_the_reach_an_agents_grant_names` |
| ADR 0040 part 1: a closed facility reach, matched exactly | `alo-capability` lib `reach::tests::a_facility_covers_itself_and_nothing_else`, `grant::tests::an_agent_is_never_granted_a_facility` |
| Part 2: grantee is agent or application; refusal words differ | `grantee::tests::an_agent_and_an_application_never_answer_for_each_other`, `allowing::tests::an_application_is_refused_in_its_own_words` |
| Part 3: declining ends agents' grants and keeps applications' | `agent::tests::declining_the_agent_keeps_every_applications_grant`, `agent::tests::a_declined_machine_grants_applications_and_never_an_agent`, and `alo-portals` `…::a_machine_with_no_agent_still_judges_what_applications_ask` |
| Part 4: the grants file gets a new format number, additively | `alo-remembering` lib `written::tests::an_applications_grant_over_a_facility_is_kept_in_format_two`, `…::a_list_with_no_applications_grant_is_written_as_it_was_before`, `…::a_first_format_file_naming_a_later_key_is_refused` |

The other refusal paths are tested as well:

- an expired or revoked grant;
- another application's grant;
- an agent of the same name;
- a malformed identifier or path;
- a facility that does not exist in the file;
- a file with two grantees or none;
- a file that grants an agent the camera.

## Verification

Run from the checkout on 2026-09-15. Everything except formatting ran inside
the `alo` VM (`limactl shell alo sudo bash -lc …`), with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`, the loop's own
build directory.

- `cargo fmt --all`, then `cargo fmt --all -- --check`: clean (on the Mac).
- `cargo clippy --all-targets -- -D warnings` (whole workspace, 55 crates
  checked, including `alo-turn`, `alo-agentd`, `alo-granted`): exit 0, no
  warnings.
- `cargo test -p alo-capability`: exit 0 (unit 156, integration 8 + 2, doc
  2).
- `cargo test -p alo-remembering`: exit 0 (unit 42, integration 3 + 4 + 4).
- `cargo test -p alo-portals`: exit 0 (unit 10, integration 9, doc 1).
- `cargo test -p alo-granted`: exit 0 (26, 6, doc 3 + 3) after the retired
  test was removed.
- `cargo test -p alo-saying`: exit 0 (63, 4, 1).
- `cargo test -p alo-collected`: exit 0 (8, 11). It holds the crates that
  declare words to the workspace members.
- `cargo test -p alo-citing`: exit 0 (21, 10).
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-capability -p
  alo-remembering -p alo-portals -p alo-saying`: exit 0.
- `tools/kernel-loop`: `cargo test plan::` exit 0 (12 passed) against the
  edited plan, with target `/root/alo-builds/alo-os-a96435785b5e7d7d-kernel-loop`.
- Every evidence test was run on its own with `--exact`, and each reported
  1 passed.

**Not run:** the full workspace test suite, which the supervisor runs. No
hardware acceptance applies, because nothing here touches the machine: there is
no D-Bus, no socket and no dialog.

## Remaining limitations

- **Nothing serves a portal yet.** Task 5 serves `org.freedesktop.portal.*`
  from these decisions.
- **The one list does not yet word application rows.** `alo-granted`'s row
  says *{agent} can reach {what}*, and its empty-list sentence speaks of
  agents. Task 2 (depends on 1, now ready) puts applications on the list in
  their own words. Until then, a machine does not yet make application grants
  anywhere a person can see them, because no surface grants them.
- **Nothing in `alo-agentd` makes application grants.** The daemon reads the
  grants file, and an application's grant in it now permits no agent verb.
- **`Agent`'s JSON form changed** for a declined machine: from `"declined"` to
  `{"declined": {…}}`. No crate and no contract persists `Agent` through serde.
  The grants file is what is kept.

## Proposed updates for the integration owner

- **CHANGELOG.md:** *Applications can be granted things the way agents are,
  including the camera, the microphone, the screen and notifications, on the
  same list. An application nobody granted anything is refused without asking
  the person. Turning the agent off keeps what applications were allowed.
  Agents can never be granted the camera or the screen. The grants file gains
  format 2, written only when an application holds a grant.*
- **ROADMAP.md:** no box moves. The v0.5 portal item stays unticked until task
  5 serves portals.
- **QUEUE.md / STATE.md:** applications plan task 1 is done. Tasks 2, 3 and 4
  are now selectable. Reference this report.
