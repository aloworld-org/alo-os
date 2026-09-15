# One list of grants, for agents and applications

**Date:** 2026-09-15
**Workstream:** v0.5 — applications, and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`), task 2:
*One list of what has been granted to what*.
**Contributor:** Claude Code worker under the kernel-loop supervisor (Mac lane,
gates in the `alo` Lima VM).
**Status:** ready for integration.

## What changed

Task 1 put applications' grants on the same `alo_capability::Grants` as agents'
grants. `alo-granted` already built its list from that value, so application
grants were already showing up as rows. Those rows were worded for agents,
though: *{agent} can reach {what}* above a video-call application's camera. The
empty list also said *No agent can reach…*, and a revocation said *the next
thing the agent asks is refused*. This change makes the list one list in fact,
and holds each clause of the acceptance with a test.

### `crates/alo-granted`

- **`src/words.rs`**: none of the four words is worded for one kind of grantee
  any more.
  - The row is now `{who} has been granted {what}`. The gap is `who` and the
    constant `words::WHO` replaces `words::AGENT`.
  - The empty list reads *Nothing is granted right now. No agent and no
    application has been granted anything on this machine, and there is
    nothing here to revoke*.
  - A revocation reads *…the next thing asked under it is refused…*.
  - The translator notes describe both kinds.
  - New test `no_word_is_worded_for_only_one_kind_of_grantee`: the row names
    no kind, and any sentence that says *agent* also says *application*.
- **`src/seen.rs`**: a row still keeps the grantee's name as a `String` and
  nothing else about the grantee. This is on purpose, and the module
  documentation now says why: a surface has nothing to sort agents from
  applications by.
  - New: `Seen::revoke_on(&mut Agent)`. On a machine with an agent it is
    `Seen::revoke` on that machine's list. On a declined machine it is
    `Agent::revoke_allowed`, because that machine holds only applications'
    grants and gives out no `&mut Grants` (ADR 0009, ADR 0040).
  - Both answer `Revoked::Now` or `Revoked::AlreadyGone`, through one private
    `answered`.
- **`src/listing.rs`**: the documentation now covers one order for both kinds,
  a declined machine's list (`Agent::allowed`), and why an application granted
  nothing has no row. Two new unit tests.
- **`src/lib.rs`**, **`src/testing.rs`**: documentation, and a fixture with a
  folder for `@files` and the camera for `org.gnome.Cheese`.
- **`tests/applications_on_the_one_list.rs`** (new): the acceptance test.
- **`tests/the_grants_a_person_can_see.rs`**: the row's expected English
  follows the new wording.
- **`Cargo.toml`**: two new dev-dependencies, both inside this workspace.
  `alo-portals` lets the in-flight request be judged by the real decision.
  `alo-remembering` lets the mixed list be read back through the grants file's
  own format-2 text.

No production dependency was added. Nothing outside `alo-granted` changed:
`alo-changing` and `alo-saying` build and pass unchanged.

### In words a person outside this repository can read

The list of what has been granted now shows what applications may reach beside
what AI agents may reach. It is one list, in the order things were granted,
and each row reads the same way: *org.gnome.Cheese has been granted the
camera*, *@files has been granted /home/anna/Invoices and everything in it*.
Taking an application's permission away is the same action as taking an
agent's away, and it applies to the application's very next request, including
one already on its way. An application that holds nothing does not show up on
the list at all.

## Decisions made here

- **A row does not know its grantee's kind.** Adding `Seen::is_an_application`
  would have been easy, and it would give every surface a reason to show two
  lists. The plan says rows should be *a person cannot tell from an agent's
  except by the name*. So the type holds nothing else to tell them apart by,
  and the test checks the row's whole `Debug` form, not just its accessors.
- ***has been granted*, not *can reach*.** The *what* is sometimes a folder and
  sometimes *sending you notifications* or *keeping the machine awake*, which
  are `alo-capability`'s facility clauses. *Can reach sending you
  notifications* is not English. *Has been granted* reads correctly before
  every kind of reach. It also matches `alo-capability`'s own refusal of an
  application (*has not been granted the camera*).
- **Same key, new gap name.** `granted.one-grant` keeps its key, and its gap
  changes from `{agent}` to `{who}`. The repository holds no translation of
  this key, so nothing breaks. A new key would have left a dead entry behind.
  `words::AGENT` was public, but nothing outside the crate used it.
- **`Seen::revoke_on` as well as `Seen::revoke`, not a second mechanism.** On a
  machine with an agent both call `Grants::revoke` with the row's handle. A
  declined machine has no `&mut Grants` (ADR 0009's structural guarantee), so
  its applications' grants can only be revoked through `Agent::revoke_allowed`.
  Without this door, a person who declined the agent could see Cheese's camera
  on the list and have no way to take it away.
  - A stale agent's row used on a declined machine lands on nothing and changes
    nothing. That is tested.
  - `alo-changing`, which revokes against the kept `Grants`, is unaffected and
    already revokes applications' rows the same way.
- ***In flight* is a real second thread.** A backend thread holds the machine's
  list behind a `RwLock` and receives a request, and then that request is held
  until the test releases it. The person revokes during that gap. The request
  is judged after release and refused with `NotAllowed::Never`. The grant that
  was not revoked is still allowed. The agent's verb, revoked by the same
  method, is refused too. When the application's last row goes, its next
  request is refused with `Refused::NothingGranted`. Nothing is remembered
  between requests, so there is no race where an answer from before the
  revocation gets reused.
- **Agent and application rows are compared over the same folder.** Only a
  folder or a file can be granted to both kinds. An agent can never hold a
  facility (`GrantError::NotForAnAgent`). Comparing the two over one reach is
  the only comparison where *anything but the name* is really tested. The
  handle is left out because every grant has a different one by construction,
  and it is what a revocation targets, not a sign of whose grant it is.

## Acceptance, and the test behind each clause

| Criterion | Test |
|---|---|
| An application's grants beside an agent's, in one order, one shape per row (who, what, until when), and no row told apart by anything but the name | `alo-granted` `applications_on_the_one_list::a_mixed_list_has_no_row_told_apart_but_by_the_name` |
| Revoking an application's grant is the same action as revoking an agent's, takes effect at the next portal request, tested with a request in flight | `…::revoking_an_applications_grant_stops_the_request_in_flight_the_way_an_agents_verb_is_stopped` |
| An application that has been granted nothing does not appear | `…::an_application_granted_nothing_does_not_appear` |
| Every sentence the list shows is in the vocabulary `alo-saying` collects | `…::every_sentence_the_one_list_shows_is_collected` |
| No word is worded for one kind of grantee | `alo-granted` lib `words::tests::no_word_is_worded_for_only_one_kind_of_grantee` |
| Revocation reaches a declined machine's applications, and nothing else | `alo-granted` lib `seen::tests::revoking_on_a_declined_machine_reaches_only_what_it_holds` |
| Constraint: derived, never a second store | Every list in the acceptance test is read back through `alo_remembering::written` / `read`, and each portal request is judged against the same value the list came from. `Listing` and `Seen` still have no public constructor (compile-fail doctests unchanged). |

The refusal paths tested are:

- a stale row revoked a second time (`AlreadyGone`, grants unchanged);
- an agent's row from before declining, used on a declined machine;
- a stranger asking a hundred times, which leaves the grants byte for byte
  unchanged and adds no row;
- an expired application grant;
- a revoked application grant;
- a revoked grant asked for while a request is in flight.

## Verification

Run from the checkout on 2026-09-15. Formatting ran on the Mac. Everything else
ran inside the `alo` VM (`limactl shell alo sudo bash -lc …`) with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`, the loop's own
build directory. The Mac cannot build `alo-sessiond`, which `alo-saying` pulls
in, because `socket_peercred` exists only on Linux.

- `cargo fmt --all`, then `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets -- -D warnings`, whole workspace: exit 0.
- `cargo test -p alo-granted`: exit 0. Unit 32, `applications_on_the_one_list`
  4, `the_grants_a_person_can_see` 6, doc 3 + 3.
- `cargo test -p alo-saying -p alo-changing`: exit 0. These are the dependents
  that collect these words and revoke these rows. `alo-changing`: 20, 12, 10,
  1, 3, 3. `alo-saying`: 63, 4, 1.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-granted`: exit 0.
- `tools/kernel-loop`: `cargo test plan::`, run against the edited plan.
- Every evidence test was run on its own with `--exact`.

**Not run:** the full workspace test suite, which the supervisor runs. No
hardware acceptance applies: nothing here touches the machine, and nothing
here draws.

## Remaining limitations

- **Nothing draws the list.** The desktop lane's `crates/alo-shell` still has to
  wire `Listing` into a surface.
- **Nothing on a real machine makes application grants yet.** A person has no
  act that grants an application something. That act, and the portal backend
  that would answer with it, is task 5 and the desktop lane's chooser.
- **The Settings surface on a declined machine.** ADR 0009 says the Grants
  surface is absent on a machine with no agent. ADR 0040 keeps applications'
  grants on such a machine. This crate now lists and revokes those grants
  either way (`Listing::of(agent.allowed(), …)`, `Seen::revoke_on`). Whether
  the surface appears on a declined machine that holds application grants is
  the surface's decision. No ADR is contradicted here, but whoever draws the
  surface should reconcile the two sentences.

## Proposed updates for the integration owner

- **CHANGELOG.md:** *The list of what has been granted shows applications
  beside agents, in one order and in the same words. Each row reads "…has been
  granted…". Revoking an application's permission is the same action as
  revoking an agent's, and applies to its very next request, even one already
  on its way. An application that holds nothing is not listed.*
- **ROADMAP.md:** no box moves. The ★ *one list* item stays unticked until a
  surface draws it and task 5 lets applications ask.
- **QUEUE.md / STATE.md:** applications plan task 2 is done. Tasks 3 and 4
  remain selectable, and task 5 still waits on 3 and 4. Reference this report.
