# Portal grants, decided before they are built

**Date:** 2026-09-15
**Workstream:** v0.5 — applications, and what they expect
(`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`), task 1:
*A portal request is a grant, and is refused like one*.
**Contributor:** Claude Code worker under the kernel-loop supervisor (Mac lane,
gates in the `alo` Lima VM).
**Status:** ready for integration, as a decision. **The code for task 1 is not
written and waits on ADR 0040.** Nothing here claims that a portal request is
judged against a grant yet.

## What changed

- **`docs/decisions/0040-what-an-applications-grant-is-over.md`** (new,
  *proposed*). It covers what an application's grant is over, where it is
  kept, and why a worker could not choose that alone. It sets out three
  options, what each costs, and a recommendation.
- **`crates/alo-granted/tests/a_portal_grant_waits_on_its_decision.rs`** (new).
  It holds the ADR in place, the same way
  `crates/alo-opening/tests/converting_waits_on_its_decision.rs` holds ADR
  0039.
- **`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`**. Task 1 is
  now `**Status:** blocked`, with the reason and the ADR written in, so the loop
  steps over it instead of sending the next worker at the same wall.

### In words a person outside this repository can read

Applications on alo OS will ask for things the way agents do: to open a file,
use the camera, or show a notification. The plan was for both kinds of request
to be answered by one permission model and one list. Before building that, we
found that today's model has no way to write down *the camera*, *the screen* or
*notifications*. It can only record folders, files and applications. If an
application's grants were kept in the agent's list, a person who turned the
agent off would also lose every application's permissions. We wrote the choice
down for the owner instead of guessing, and recommended a small extension to
the permission model. Nothing an application can do has changed.

## Why this became a decision

The acceptance for task 1 asks for a portal request judged "the way
`alo-capability` evaluates a verb", with "the same `Reach` an agent's grant
names", reusing that crate's types. The plan also says `alo-capability` belongs
to lane A and this plan never edits it (ADR 0028's partition). Reading
`alo-capability` on `main` turned up four facts that make both requirements
impossible at once:

1. **`Reach` is `Folder`, `File` or `Application`.** Eleven of the fifteen v0.5
   portals ask about none of those: wallpaper, notifications, screenshot, screen
   capture, camera, microphone, clipboard, settings, inhibit, network monitor
   and power-profile monitor. The ADR has a table with one row per portal.
2. **A device path is not an honest reach.** The camera portal hands over a
   PipeWire remote, not a file. `/dev/videoN` is numbered in probe order, so a
   grant over `/dev/video0` could come to cover a different camera. The stable
   names under `/dev/v4l/by-id` are links, and `alo-capability` (`path.rs`)
   requires an ask to arrive with links resolved, so a grant over a link covers
   nothing.
3. **Grants live inside `Agent`.** `Agent::Declined` holds no list (ADR 0009),
   so an application's grants kept there would end when the person declines
   the agent, and could never be made on such a machine.
4. **`Grantee` and its refusals speak of an agent.** The grants file stores the
   grantee under `agent`. The never-granted refusal says *grants are made by
   picking a folder*. The declined-machine refusal says *this machine has no
   agent*. Neither is true of a video-call application asking for the
   microphone.

Every way around these facts from inside `alo-portals` is one of two things. It
is a pun: a portal stored as an application identifier, or a device stored as a
drifting path. Or it is a second list, which breaks the ★ *one list* promise in
`docs/features.md` and ADR 0005. The worker's instructions rule out narrowing a
promise and contradicting an accepted ADR. So, as with ADR 0039, the decision is
the deliverable.

## Decisions made here

- **The recommendation is option C.** `alo-capability` gains four things:
  - one closed kind of reach for what a machine has that is not a path;
  - a grantee that is either an agent or an application;
  - application grants that outlive declining the agent;
  - a new grants-file format.
  The change is made by the lane that owns the crate, or by this plan if the
  owner moves it here in writing.
- **The test lives in `alo-granted`, not in a new `alo-portals`.** Creating the
  portal crate while the decision is proposed would pre-empt it: the crate's
  first line would have to pick an option. `alo-granted` is this plan's, and it
  is where whatever an application's grant turns out to be over will first be
  shown as a row.
- **The test pins the `docs/features.md` portal line exactly.** If a portal is
  added to or removed from the promise, the test fails and points at the ADR's
  table, which would otherwise argue about a list nobody promised any more.
- **Task 4's core does not need a portal reach.** The ADR notes this, so the
  owner can narrow task 4's dependency and keep work moving before answering.
  Changing that dependency is the owner's call, and this change does not make
  it.

## What the test holds

`crates/alo-granted/tests/a_portal_grant_waits_on_its_decision.rs`:

| Test | What it refuses |
|---|---|
| `the_decision_exists_once_under_its_number` | no ADR 0040, two of them, or one whose status no longer stands |
| `the_plan_points_at_the_decision_and_steps_over_the_task_while_it_waits` | task 1 not naming the ADR, or left *ready* or marked done while it is proposed |
| `the_decision_sets_out_the_options_their_costs_and_a_recommendation` | a decision without three costed options, a recommendation, the rule that no portal is answered yes by default, and the three things that must happen first |
| `the_decision_reads_every_portal_the_promise_lists` | a features line that changed under the ADR, a portal with no row or two rows, a count of reachless portals other than eleven, and a row for any v1 portal |
| `no_portal_is_modelled_while_the_decision_is_proposed` | `crates/alo-portals` existing, the workspace listing it, or shipped source naming an `org.freedesktop.portal.*` interface while the ADR is proposed |
| `the_checks_would_notice_what_they_read_going_missing` | a rename that would make the checks above pass by reading nothing, and the helpers' own behaviour, including an indented table row |

## Verification

All commands ran from the checkout, inside the Linux VM the loop names
(`limactl shell alo sudo bash -lc …`), with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-a96435785b5e7d7d` (the loop's own)
and `RUSTDOCFLAGS="-D warnings"`. They were never run on the Mac itself: an
earlier attempt there stopped in `alo-sessiond`, which is Linux-only, and that
is not a finding about this change.

- `cargo fmt --all` then `cargo fmt --all -- --check`: clean.
- `cargo clippy -p alo-granted --all-targets -- -D warnings`: exit 0, no
  warnings.
- `cargo test -p alo-granted`: exit 0. Unit 26 passed;
  `a_portal_grant_waits_on_its_decision` 6 passed;
  `the_grants_a_person_can_see` 6 passed; doctests 3 + 3 passed.
- `cargo test -p alo-citing`: exit 0 (21 + 10 passed). The ADR's and the test's
  citations resolve, and ADR 0040 records its status.
- `tools/kernel-loop`: `cargo test plan::` exit 0 (12 passed), including
  `every_plan_this_repository_drives_holds_only_tasks` against the edited plan.
  This used a separate build directory,
  `$HOME/alo-builds/alo-os-a96435785b5e7d7d-kernel-loop`, created by this run
  inside the VM. It holds only build output, and nothing else uses it.

The first run of `the_decision_reads_every_portal_the_promise_lists` failed. The
ADR's table is indented under a list item, and the row finder did not trim
leading space. The finder was fixed, the indented case was added to the
self-check test, and the suite was run again as shown above.

**Not run:** the full workspace suite, which the supervisor runs. No hardware
or physical acceptance applies: nothing here touches the machine.

## Remaining limitations

- Task 1's acceptance is **unmet**. There is no `alo-portals`, no portal enum,
  no evaluation and no refusal of an ungranted application. Tasks 2, 3, 4 and 5
  depend on task 1, so the plan has no selectable task until the owner answers.
- Once ADR 0040 is accepted, the test's
  `no_portal_is_modelled_while_the_decision_is_proposed` stops applying on its
  own, because it checks the status line. The worker who builds task 1 should
  then remove or rewrite the test to hold the accepted shape.

## Proposed updates for the integration owner

- **CHANGELOG.md:** *Decided, not yet built: what an application's permission is
  over. Applications will ask for the camera, the screen or notifications
  through the same permission model agents use. ADR 0040 proposes how that
  model records things that are not files, and how an application keeps its
  permissions on a machine whose owner turned the agent off.*
- **ROADMAP.md:** no box moves. The v0.5 portal and one-list items stay unticked.
- **QUEUE.md / STATE.md:** the applications plan's task 1 is blocked on ADR 0040
  (proposed). The owner needs to answer it, and lane A (or the owner, by
  re-partitioning) needs to make the `alo-capability` change. Reference this
  report.
