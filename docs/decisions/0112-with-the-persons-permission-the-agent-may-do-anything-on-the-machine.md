# ADR 0112 — With the person's permission, the agent may do anything on the machine

**Status:** **accepted, 2026-10-14, by the owner**, in these words: *"the agent
should be able to do anything on the machine, if the users gives it permission
then it can do anything."*

**This is law 5 stated as a rule a lane can apply**, and it is recorded because
the law alone has not been enough. Law 5 has been in `CLAUDE.md` since
2026-09-30 and
[ADR 0043](0043-the-terminal-is-a-persons-and-never-an-agents.md) still walled
an entire capability off with no setting behind it, for a month, enforced in
code. One general rule, written down once, is what that record needed and did
not have.

[ADR 0111](0111-the-person-chooses-whether-their-agent-may-use-the-terminal.md)
is this rule's first application, decided the same day. This record is the rule;
0111 is the worked example.

## The rule

**There is no act on a person's own machine that is reserved from their agent.
If the person grants it, the agent may do it.**

A lane that finds itself writing *the agent may never …* about a person's own
machine has found a bug in what it is writing, not a protection. The question is
never *should this be allowed*; it is **what does the person have to be told,
and how do they switch it on and off.**

## What "permission" means, so that "anything" is not formless

The rule widens *what* may be granted and changes nothing about *how*. Every
grant keeps ADR 0001 §3's terms and law 5's:

- **Enumerated** — a list, not a rule. A grant names what it is over.
- **Visible where the person can find them**, not buried.
- **Revocable in one action**, and **expiring**.
- **The risk in plain words, in the sentence that grants it.** Law 5: *anything
  with a risk is offered with the risk in plain words and switched on by the
  person.*
- **Off until the person turns it on.**
  [ADR 0025](0025-the-default-is-what-a-machine-arrives-able-to-do.md): nobody
  chooses for the person.
- **Revocation is honest about its limits.** It prevents further access. It
  cannot undo what already ran or recall what already left, and the surface says
  so rather than implying otherwise.

So *anything* is reached by **granting a capability**, under those terms — not by
loosening any of them.

## The four things a person's permission does not reach, and why none is a wall

Law 5 names all four. They are listed here with the reason each is not the
person being overruled, because *that* is what makes them legitimate rather than
exceptions nibbling at the rule.

### 1. Protections that restrict nothing

The record, undo, the egress indicator, plain warnings.

**A person who switched these off would gain no capability.** They would lose
the ability to see what happened and to put it back. There is nothing here for
them to choose between, so leaving them on takes nothing away — and
[ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
already makes them always-on at every level, including full trust: *every
command and its output are written to the record; the project or the affected
files are snapshotted before each run … and everything that leaves the machine
is shown.*

**These are also what make the rule safe to state so broadly.** *Anything*
survives as a default-off capability precisely because whatever is granted is
recorded, reversible and visible.

### 2. Protections that guard the person from others

No telemetry. Never a silent fallback. Helpdesk only when invited.

**These do not guard a person from themselves; they guard them from us and from
third parties.** Removing one would not be the person choosing something — it
would be somebody else taking something, with the person's switch as the
pretext. A setting that let alo send telemetry is not a freedom a person gains;
it is a freedom *we* gain over them.

### 3. The law

Law 5's own words. A person cannot grant what the law does not permit, and the
most common shape of that is not about them at all: other people's data,
reachable from their machine, is not theirs to hand over.

### 4. An organisation's policy on machines it owns

[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md), unchanged.
On its own property an organisation bounds what is available. On a person's own
machine the person chooses. A managed machine says so at first sign-in — there is
no silent enrollment, and no administrator may watch a screen or act as a person
([ADR 0004](0004-the-organisations-machine.md)).

## What does not change, and it is what keeps this reviewable

**Running code is not a verb on the broker's list, and no `exec` is ever added
to it.** That is law 2 and ADR 0064, and this rule does not touch it.

This matters more than it looks. Law 2's reason for a closed, typed, compiler-
walked verb list is that *a model that can write code that runs has escaped
every other control in this repository.* ADR 0064's answer was not to put `exec`
on the list; it was to make running code **a separate capability the person
grants, at one of three levels, with the three always-ons.**

So this rule is satisfied **without widening the broker at all**: the verb list
stays small and reviewable, and *anything* lives in granted capabilities beside
it. A future record that proposes reaching *anything* by adding a verb that runs
commands has misread this one.

## What this forbids us from doing

**Inventing a wall.** When a lane asks *may an agent do X on a person's own
machine?*, the answer is **yes, if the person grants it**, unless X is one of
the four above. A record that answers otherwise is a bug, *whatever the reason
given for it* — law 5's own words.

**ADR 0043 is the worked example**, and it is worth keeping in view because none
of its authors were careless. It had a real risk, stated well — *a shell is the
whole machine by another road* — and a careful mechanism, three refusals rather
than one. What it did not have was a setting. The risk was correct and the
conclusion was a wall, and it stood for a month inside a repository whose first
law on the subject says only the law may truly forbid.

**So the failure mode is not malice or sloppiness. It is a good reason.** Law 5
says as much: *whatever the reason given for it … it is addressed to good ones.*

## What it costs, said plainly

This rule makes the product **more permissive and more work at the same time.**
Every *anything* a person may grant needs:

- a grant with those terms, and a surface to give and revoke it on;
- a sentence saying what it costs, in words a person reads rather than a
  category;
- a record entry, an undo, and an egress line — because those are always on;
- **enforcement in the path that acts, not in the surface that displays.** A
  switch drawn over an unenforced capability is worse than no switch: it is a
  promise a person relies on.

A lane that cannot yet build the second half should say the capability is not
built rather than draw the switch.

## Consequences

- **ADR 0043 is superseded** by ADR 0111, the same day, as this rule's first
  application.
- **Records that gate an agent's reach are read against this rule**, including
  [ADR 0042](0042-installing-an-application-is-an-errand-and-an-agent-only-proposes-it.md)
  (an agent proposes an install) and
  [ADR 0079](0079-a-person-may-hand-their-agent-a-whole-machine.md) (a whole
  remote machine, already permitted on the owner's direction). 0079 is this rule
  arrived at early and for one case; nothing in it needs changing.
- **`docs/features.md`'s Non-goals** are read against it too. A non-goal that
  says *the agent may never* about a person's own machine is now a non-goal
  about what we have **not built**, not about what a person may not have.
- **Nothing here is built.** This record changes no code and ships no
  capability. What it changes is the answer a lane gives when it next asks the
  question, which is the thing ADR 0043 got wrong with every reason to get it
  right.
