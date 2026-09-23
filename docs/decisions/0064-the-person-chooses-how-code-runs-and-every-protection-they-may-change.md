# ADR 0064 — The person chooses: how code runs, and every protection a person may change

**Status:** **accepted, 2026-09-22, by the owner.** **Nothing is built in the
change that adds this.** Each part below is built by its own task, and that task
changes, in the same change, the tests that hold the rule it replaces. Until a
part is built, the tests that hold the old rule keep holding it, and the
machine behaves as it did.
**Date:** 2026-09-22
**Proposed by:** the owner, in conversation; written down by the dev PC
**Context:** [ADR 0001](0001-the-capability-model.md) §1 (no verb runs an
arbitrary command); [ADR 0003](0003-the-network-is-not-authority.md) (no
trusted-network setting); [ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)
(the organisation bounds, the person chooses);
[ADR 0062](0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
(Fast Startup left open); the Non-goals in `docs/features.md`; task 1 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` (no setting turns update
checking off).

## The owner's words

> *We should always give our users the choice to choose what they want and
> never limit them … if they want the agent to run the codes on their PCs on
> their own we should not stop them — that is dictatorship, and not freedom.*

And, asked whether this is a rule of development: *never take away the choice
of the user to decide what he wants.*

## Law 5

A fifth law joins the four in `CLAUDE.md`:

> **The person chooses. alo never takes the choice away.** On their own
> machine, a person decides what alo OS does. Every protection is a default
> they can change, not a wall. Anything with a risk is offered with the risk in
> plain words and switched on by the person. It is never removed from them
> "for their own good". Two kinds of protection stay on at every setting,
> because neither takes a choice away: protections that restrict nothing (the
> record, undo, the egress indicator, plain warnings), and protections that
> guard the person from others (no telemetry, never a silent fallback, helpdesk
> only when invited). Only the law, or an organisation's policy on machines it
> owns (ADR 0016), may truly forbid. A change that takes a choice away from the
> person is a bug, whatever the reason given for it.

The review question for every feature from now on is: *can the person choose
this, knowing what it costs?*

## Running code: three levels, and the person picks

**Law 2 read** *no verb runs an arbitrary command*, and ADR 0001 §1 and the
Non-goals called it not revisitable without replacing that ADR. **This ADR is
that replacement, for §1 only.** The rest of ADR 0001 stands.

The broker's fixed verb list stays exactly what it was: typed, closed, and
walked by the compiler. **Running code is not a verb on that list.** It is a
separate capability that the person grants at one of three levels:

1. **The sealed box** (the default). Code the agent writes or fetches runs only
   inside a box that sees the project and nothing else. It has no network except
   destinations the person approved, and each one appears in the egress
   indicator. The lock is enforced by the kernel
   ([ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md),
   [ADR 0015](0015-the-kernel-learns-what-a-turn-is.md)), not by the agent's
   promise. The box has no powers of its own, no road to the broker, and limits
   on processor, memory and time, and it is thrown away after each run.
2. **Ask me each time.** The agent shows the exact command, and it runs on the
   person's machine once the person approves it.
3. **Full trust.** The agent runs code directly on the machine, without asking.
   The person switches this on themself, for one project or for everything, and
   can switch it off at any time. The switch says in plain words what it costs:
   *text in a web page, an email or a downloaded project can steer the agent,
   and at this level whatever it steers the agent to run, runs.* It informs and
   does not forbid.

**At every level, always on:** every command and its output are written to the
record; the project or the affected files are snapshotted before each run, so
*undo what the agent did* works even at full trust; and everything that leaves
the machine is shown. Anything public or costly, such as publishing a site,
pointing a domain or sending mail, is a verb like any other and asks as verbs
ask.

**An organisation** may limit which levels are available on machines it owns
(ADR 0016). That is its choice over its own property. On a person's own machine
the person chooses.

Law 2 becomes: **No code runs unless the person chose it.**

## The protections a person may now change

Each was written as a wall. Each becomes a default the person can change, with
the cost said once and plainly.

| | Written today as | Becomes |
|---|---|---|
| 1 | *No setting turns update checking off* (keeps-itself task 1) | a setting that turns checking off, saying that security fixes stop arriving while it is off |
| 2 | *There is no trusted-network setting* (ADR 0003) | an opt-in, off by default: *trust devices on this network*, saying that this also trusts guests' phones and every device on the network. Pairing stays the default. |
| 3 | A provider address that is not https is refused (`docs/features.md`) | allowed after a plain warning that the key crosses the network readable |
| 4 | Installing outside the sandbox, v1 only (`docs/features.md`) | brought forward to v0.5 as a deliberate, clearly marked act. Developers need their own tools from the first day. |
| 5 | *A person never learns the name of anything we rented* | plain words stay the default everywhere, and a **details** view shows each component's own name as data. The rule that no sentence in `alo-strings` names a rented component is unchanged; the names in the details view are data, not sentences. |
| 6 | Screenshot-and-click disabled by policy by default | still off by default; the person may switch it on for their own machine |
| 7 | A turn whose kernel boundary cannot be applied does not run | still refused by default; at level 3 the person may choose to run it anyway, told that the kernel is not watching this turn, and the record says so |
| 8 | Clipboard history *never synced anywhere* | still never synced to any server. An opt-in shares a copied item directly between the person's own paired devices: end to end encrypted, deleted after about two minutes, and never readable by the agent. |
| 9 | Fast Startup, left open by ADR 0062 | **the installer asks**: *"Windows' Fast Startup is on. It can make Windows and alo OS disagree about the disk. Turn it off? (Recommended when sharing a disk.)"* The choice sets `HiberbootEnabled` and never uses `powercfg /h off`. Both answers are safe, because alo OS never mounts the Windows partition read-write. |

## What does not change

- **Protections that guard the person from others.** No telemetry. Never a
  silent fallback from local to hosted inference. The agent never nags. A
  helpdesk session is one the person starts. alo's own service gets no
  exemption from the egress indicator. Removing any of these would hand power to
  alo, an employer or a stranger, not to the person.
- **Protections that restrict nothing.** The record, undo, the egress indicator
  and plain warnings stay on at every level of every setting.
- **The organisation's bound on machines it owns** (ADR 0016).

## Consequences

- `CLAUDE.md` has five laws, and Law 2 reads as above. `docs/autonomy/LOOP.md`
  and `docs/autonomy/a-new-machine-becomes-a-lane.md` are updated to match, so
  that no lane refuses work this ADR permits.
- ADR 0001's, ADR 0003's and ADR 0062's status lines point here.
- `docs/features.md`'s Non-goals are updated for items 2 and the three levels,
  and a section lists the rest until each is built.
- **Each part is its own task**, and none of them is written here. The sealed
  box needs a design task before any code, and the owner's demo depends on it.
  Every such task changes, in the same change, the tests that hold the old rule,
  so the repository never states one rule while its tests hold another.
