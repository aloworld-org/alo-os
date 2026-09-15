# ADR 0042 — Installing an application is an errand, and an agent only proposes it

**Status:** accepted, 2026-09-15. Written by task 1 of
`docs/autonomy/v0-5-software-and-the-web-plan.md` (*Installing, updating and
removing an application*), whose code is built on it.
**Date:** 2026-09-15
**Context:** [ADR 0001](0001-the-capability-model.md) (a change waits for one
approval of one sentence); [ADR 0005](0005-applications-are-sandboxed-and-ask.md)
(applications install sandboxed, from places a person or an organisation
chooses); [ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)
(the organisation bounds, the person chooses);
[ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
(no lane edits another lane's crates); `docs/contracts/agent-verbs.md` rule 5 (*a
verb that requires no grant needs a written reason in its ADR*);
`crates/alo-egress/src/errand.rs` (the closed list of reasons alo OS reaches the
network on its own); `docs/features.md` v0.5 (*install applications, sandboxed,
from Flathub or a repository the organisation runs; update and remove them*).

## The questions in one line

**Installing and updating an application leave the machine, and the plan says
the indicator names them as what they are — but `alo_egress::Errand` had no
member for either, and the plan does not own `alo-egress`. And the plan's one
verb, *install an application*, is over nothing a grant can name.** What is
each, and who makes the change?

## 1. Three errands join the closed list

`Errand` is ★ *no telemetry* as a closed list: a reason alo OS reaches the
network on its own that is not a member cannot be shown, and one that is a
member is checkable by anybody watching the indicator. Installing an application
is exactly that kind of reason — somebody chose it, nothing about the person is
sent — and naming it as *fetching a model* or as the system's own update check
would be the indicator saying something untrue.

So three members are added, each its own line:

| Errand | The line |
|---|---|
| `InstallingAnApplication` | alo OS is installing an application from {destination} |
| `CheckingForApplicationUpdates` | alo OS is checking for application updates at {destination} |
| `UpdatingAnApplication` | alo OS is updating an application from {destination} |

**Three rather than one**, because *looking for* and *fetching* are different
things to see happening, and **none of them is `CheckingForAnUpdate`**, because
an application's update is not the system's (`alo-keeping-up`).

**The edit is additive and small, and it is made in this change.** It adds three
members and three words to `alo-egress`, adds the three to the one exhaustive
match outside it (`alo-keeping-up`'s `Offered::heard`, which refuses an answer
heard during any errand but its own), rewords `alo-record`'s documentation that
counted *three reasons*, and makes one test in `alo-shell` count the errands
rather than assume a number. No behaviour of any of those crates changes.
ADR 0028's partition is kept in the way ADR 0040 kept it: the change is named
here, in writing, with the crates it touches.

## 2. Removing is not an errand

Removing an application reaches no network, so it is not on the list and puts
nothing on the indicator. The plan's acceptance names removing beside
installing and updating as *errands that leave the machine*; that sentence is
true of the other two and not of this one, and an indicator line for something
that did not leave would break the indicator's only promise in the other
direction. What removing owes instead — ending the application's grants in the
same act — is kept, and tested.

## 3. `install_application` needs no grant, and this is the reason

`docs/contracts/agent-verbs.md` rule 5: a verb names the grant it requires,
over a path or an application, **or it carries a written reason for requiring
none, in its ADR.** This is that reason.

- **There is nothing on the machine for a grant to be over.** The application is
  not installed yet, and the place it comes from is the rented tool's
  configuration, which no grant names.
- **It arrives granted nothing.** An installed application reaches no folder,
  no device and no portal until a person allows it something, and no agent can
  open, focus, arrange or close it without a grant over it. The installation
  widens nobody's reach.
- **The approval is the protection, and it is the right one.** The verb is a
  `change`: it waits for one approval of *install {application} from {source}*,
  both named, both validated, and that approval installs once. A place not
  enabled, outside the organisation's bound, or not checking signatures is
  refused at the moment of installation, whatever was approved.
- **And it only proposes.** There is no verb that updates, removes or lists
  applications, and nothing in `alo-software` turns an agent's call into an
  installation except an `alo_capability::Authorised` that came from an
  approval.

The reason is carried in the declaration (`Requires::nothing_because`) as well
as here, as rule 5 asks.

## Consequences

- `Errand::EVERY` has six members, and every reader of it — the record, the
  shell's indicator, the promise sentence beside the list — reads six.
- A future *removing* line is refused by this ADR rather than forgotten.
- The organisation's list of permitted places is a bound in the machine's
  description (ADR 0016). `alo-software` holds the value (`Bound`) and the
  refusal that names who set it; reading the section out of
  `/etc/alo/agentd.toml` is `alo-agentd`'s and a contract change of its own,
  written into the plan as its own task.
