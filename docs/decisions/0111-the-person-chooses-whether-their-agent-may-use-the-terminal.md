# ADR 0111 — The person chooses whether their agent may use the terminal

**Status:** **accepted, 2026-10-14, by the owner**, in these words: *"we are not
dictators so the users should be able to choose if the agent can use the
terminal or not so it is not us to dictate the users choices, so we need a
terminal that is ai native."*

**Supersedes** [ADR 0043](0043-the-terminal-is-a-persons-and-never-an-agents.md)
— *the terminal is a person's, and never an agent's* — which stays in the record
because its risk sentence is reused below, and because a decision reversed
without its reasoning kept is a decision nobody can audit.

**And a second ruling, the same day, on the split this record proposed:**
*"agent can open terminal window application."* So of the two grants below, the
**opening** half is not a grant at all — it is an ordinary application verb, and
the terminal stops being excluded from the four. Only *running commands* is the
capability a person switches on.

**The general rule is
[ADR 0112](0112-with-the-persons-permission-the-agent-may-do-anything-on-the-machine.md)**,
decided the same day and in the owner's words: *the agent should be able to do
anything on the machine, if the users gives it permission then it can do
anything.* This record is that rule's first application; 0112 is the rule, and a
lane asking *may an agent do X* should read it rather than reason from here.

**What is accepted is the principle and that split.** The remaining terms under
*The decision* are proposed against it and the owner may correct any of them.
What is not open is whether the person chooses.

## Why ADR 0043 cannot stand, and it is not only law 5

Three independent reasons. The first is the owner's; the other two are this
repository disagreeing with itself.

### 1. Law 5 forbids the wall, and postdates the wall

ADR 0043 is dated **2026-09-15** and accepted **2026-09-16**. Law 5 — *the
person chooses; alo never takes the choice away* — reached `CLAUDE.md` later,
and says: *every protection is a default they can change, not a wall … **only
the law, or an organisation's policy on machines it owns (ADR 0016), may truly
forbid.** A change that takes a choice away from the person is a bug, whatever
the reason given for it.*

ADR 0043 is a wall. It offers no setting, no override and no grant. It is not
the law, and it is not an organisation's policy. By law 5's own review question
— *can the person choose this, knowing what it costs?* — the answer today is
no.

**And the "never offered" escape does not apply.** Law 5 says *never offering is
not taking away. Withdrawing is.* So the question is whether this repository
told a person they may decide. It did, six days after 0043 — which is reason 2.

### 2. ADR 0064 replaced 0043's foundation, and permits the greater act

ADR 0043's stated context is **ADR 0001 §1** — *no verb runs an arbitrary
command*.
[ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md),
accepted **2026-09-22**, says in its own words: *Law 2 read* **no verb runs an
arbitrary command***… **This ADR is that replacement, for §1 only.*** Six days
after 0043, the premise 0043 was built on stopped being the rule.

What replaced it is a choice at three levels, the third of which is **full
trust**: *the agent runs code directly on the machine, without asking.*

So the repository's position today is:

| | permitted? |
|---|---|
| an agent running **any command** on the machine, unattended, at full trust | **yes** — ADR 0064, the person's own choice |
| an agent **opening a terminal application** | **no** — ADR 0043, no choice offered |

**That is incoherent, and not marginally.** It forbids the lesser act and
permits the greater one. An agent at full trust already has everything a shell
would give it; what 0043 blocks is not a capability but one application's
identifier. A person who has granted full trust and is then told their agent may
not open Ptyxis is being guarded from a door while the wall beside it stands
open.

### 3. ADR 0079 already crossed this line, for strictly more, on the owner's direction

[ADR 0079](0079-a-person-may-hand-their-agent-a-whole-machine.md), 2026-09-29,
permits a person to grant their agent control of **another whole machine**. It
does not argue the grant away — it says *this grant is `/` on another machine,
reached by a third road*, and it names 0043 while doing so: *ADR 0043 named the
second road and refused it. This record names the third and, on the owner's
direction, permits it — deliberately, visibly, and under the narrowest terms
that still leave the feature real.*

So **the second road is the only one still walled off, and it is the smallest of
the three.** A local terminal is less than a whole remote machine by any measure
of blast radius. Keeping 0043 while 0079 stands is not caution. It is an
inconsistency that happens to be load-bearing in code.

## The decision

**Whether an agent may use the terminal is a capability the person grants. It is
off until they turn it on, the sentence that grants it says what it costs, it is
revocable in one action, and it expires.**

The protection ADR 0043 built is **kept as the default**, which is exactly what
law 5 asks of a protection: *a default they can change, not a wall.*

The terms are ADR 0079's, deliberately — the owner has already approved them for
a larger grant, and a smaller one should not be harder to give:

1. **Off until the person turns it on.** A fresh machine behaves exactly as it
   does today, and `A_PERSONS_OWN` keeps naming every terminal emulator this
   machine can install. What changes is that the list is consulted **alongside**
   the person's grant instead of **instead of** it.
2. **The risk in plain words, in the granting sentence** — and ADR 0043 already
   wrote that sentence: *a shell is the whole machine by another road.* It is
   reused verbatim rather than softened. Law 5: *anything with a risk is offered
   with the risk in plain words and switched on by the person.*
3. **Revocable in one action, and expiring.** That is ADR 0001 §3 for every
   grant and is not relaxed here. Revocation prevents further access; it cannot
   undo a command that already ran, and the surface says so rather than
   implying otherwise.
4. **An organisation may forbid it on machines it owns**
   ([ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)).
   That is the one party law 5 allows to truly forbid, and it is unchanged.
5. **ADR 0064's three always-ons apply, at every level**: every command and its
   output are written to the record; the project or the affected files are
   snapshotted before each run, so *undo what the agent did* works even at full
   trust; and everything that leaves the machine is shown. **None of those three
   is a setting**, because none of them takes a choice away — law 5's own
   carve-out for protections that restrict nothing.

## The two halves are not the same size, and only one is a grant

**0043's single refusal conflated two acts that differ enormously**, and the
owner's second ruling of 2026-10-14 separates them: *agent can open terminal
window application.*

### Opening a terminal window is an ordinary application verb

**It hands the agent nothing**, and that is the whole reason. A terminal window
opens onto a shell prompt waiting for a keystroke, and **there is no verb that
types.** The broker's list carries open, focus, close and arrange; none of them
carries text, and `alo-shell` has no road from an agent to a pty — ADR 0110's
own reasoning is that a pty cannot tell an agent from a hand, which is why
nothing may write into one. So an agent that opens a terminal has produced a
window **only a person can use**, exactly as it has when it opens a text editor.

So the terminal is no longer excluded from the application verbs. It is an
application, and `open` reaches it under the same grant over that application
that any other application needs — enumerated, visible, revocable, expiring.
**0043's risk sentence does not reach this half**: a shell is the whole machine
by another road *to whoever can drive it*, and opening a window is not driving
it.

Two things that keep it that way, and both are already true:

- **`open` carries an identifier and nothing else.** No directory, no
  environment, no command — the verbs are typed and validated, so there is no
  argument through which an agent could set up a shell to a person's
  disadvantage before they type in it.
- **Several windows is not a new power.** An agent that opens ten terminals is
  as annoying as one that opens ten editors, and the answer is the same answer:
  the record shows it, and undo closes them.

### Running commands in one is the capability a person grants

This is the half 0043 was right to be afraid of, and it keeps every term under
*The decision* above: off until the person turns it on, the risk in 0043's own
plain words, revocable in one action, expiring, and an organisation may forbid
it on machines it owns.

**And `A_PERSONS_OWN` is the wrong shape for either half.** It is a list of
terminal emulators, which is the right list for *which applications are a
person's own* and answers neither question here: `open` needs no such list now,
and *may this agent run commands* is not a question about an identifier at all.

## What this unlocks, which is the point of asking

An **AI-native terminal** — the thing the owner asked for, and the thing
[ADR 0110](0110-the-agent-writes-the-command-and-only-the-person-runs-it.md)
could only half-describe while 0043 stood. 0110's design worked *around* the
wall: a surface the compositor draws, because the agent could not be near a
terminal at all.

With the wall gone, that becomes a **choice**. A person who wants the agent
proposing commands they press Enter on gets exactly 0110. A person who grants
more gets an agent that runs them. Both are the same product at two settings,
which is what law 5 asks for.

**0110 is not withdrawn by this.** Its undo and its opt-in measurement are what
make *either* level survivable, and its reading of law 2 stands. What changes is
that its restriction becomes a default rather than the only shape available.

## What does not change

- **The broker's verb list.** Running code is not a verb on it — ADR 0064 — and
  no `exec` verb is added here. A capability the person grants is not an
  enumerated verb and is not reached through the broker's list.
- **There is no grant to `/`.** This grant is named, bounded, revocable and
  expiring, exactly as 0079's is. What ADR 0001 §3 forbids is an *unenumerated*
  grant to everything, not a person choosing to hand over something large with
  their eyes open.
- **The default.** A machine arrives with this off, and
  [ADR 0025](0025-the-default-is-what-a-machine-arrives-able-to-do.md) holds
  that nobody chooses for the person.
- **ADR 0043's reasoning.** It was right that a shell is the whole machine. It
  was wrong only about who decides.

## What is owed, and none of it is written here

- **`alo-capability`**: `is_a_persons_own` stops causing an unconditional
  refusal and becomes a default consulted beside the person's grant. Measured
  2026-10-14: **17 non-comment uses outside its own file**, and
  `GrantError::APersonsOwn` is asserted in `alo-software`'s tests — so this is a
  change to a real enforcement path and not a flag flip. **Whose change it is
  is a question for the integration owner**: ADR 0043 itself held a built change
  back because *`alo-capability` is the applications plan's, and a lane does not
  widen its own partition*, and that question was never answered.
- **The surface a person grants it on**, carrying the risk sentence, in Settings
  beside the other grants — and *shown* is not *enforced*. The refusal path
  belongs to the capability layer and must be tested as carefully as the grant.
- **The two grants named separately** — *open a terminal* and *run commands in
  one* — rather than one switch.
- **`docs/features.md`** and **`docs/contracts/`**, because this changes a
  published surface.

**Nothing above is built.** This record is the decision, and the account of why
0043 could not stand. A reader who wants the code should expect to write it.
