# A person's other machines

**Status:** a design note, not a decision. It names what has to be settled and
recommends answers; the decisions it asks for belong in ADRs of their own.
**Date:** 2026-09-29

Two things, and they share one door. A person's other machines can be **given
work**, and they can be **sat down at** — a window on the canvas, under the
person's own hands. Adding a machine is what grants both.

## The picture this serves

One person owns several computers. They give one goal to one of them. The
machines get the work done between themselves. The person sees what each is
doing, the result comes back to where they asked, and they can stop any of it
without getting up.

**And when they add a machine, that machine appears on the canvas as a window
they can work in.** Not a status tile and not a report — the machine itself,
under their hands, in the same plane as everything else they have open. So this
is two things and not one: machines that talk to each other, and machines the
person can sit down at without standing up.

That is a person running a company on their own with several machines, rather
than an administrator running a fleet. The difference matters and is the reason
this note exists: almost everything already built here assumes **two people**,
and the picture above has **one**.

## What is already true, and it is more than it looks

The wire is finished. This note adds nothing to it.

- **Machines find each other** — `alo-nearby` advertises presence and nothing
  else over DNS-SD, with an identity that is a file rather than a serial, so a
  reinstall is a new machine.
- **Pairing is mutual and deliberate** — two people confirm on their own
  machines, comparing six digits derived from both identities and both public
  halves, which is Bluetooth's numeric comparison and Signal's safety number.
- **Every message proves itself** ([ADR 0031](../decisions/0031-the-pairing-is-the-key.md)).
  A pairing leaves each side a per-pairing key; every message carries a proof
  made with it, checked at the moment it arrives, so a pairing revoked stops
  proofs at once and a proof seen before is refused.
- **The grants are enumerated, expiring and revocable in one action**
  ([ADR 0001](../decisions/0001-the-capability-model.md) §3), and outlive a
  restart in `/var/lib/alo/pairings.toml`.
- **Being on the same network is not authority**
  ([ADR 0003](../decisions/0003-the-network-is-not-authority.md)). Discovery is
  open; use requires the pairing. There is no trusted-network setting and there
  will not be one.

`docs/features.md` already carries the promise this note elaborates, at tier v1:
*Cross-machine agent work — an agent may **ask** a paired machine, and acts only
under a grant made **on that machine, by its person***. So this is scope that
exists, not scope being invented.

**What is not built is work.** A question can cross the wire and a verb can
arrive at a door. Nothing yet sends a *job* to another machine, watches it, or
brings a result back.

## The one thing that is genuinely different here

**Pairing's safety rests on there being two people, and here there is one.**

When two people pair, the second confirmation is a real check: someone who is
not you had to agree. When both machines are yours, that confirmation is you
agreeing with yourself. It still proves you were physically at both machines,
and it still defeats an interceptor — the six digits do work no second person is
needed for. But it stops being evidence that the pairing was *wanted by someone
else*, because there is no someone else.

This is not a reason to refuse the case. It is the reason the one-person case
needs its own decision rather than inheriting the two-person one by silence. The
same sentence — *a grant made on that machine, by its person* — means something
weaker when both machines have the same person, and a promise that quietly
changes strength is the failure this repository keeps finding in its own gates.

## What has to be settled, and what I would answer

Five questions about **work sent between machines**. The section after this one
asks the separate questions that only **control** raises. Each is a decision,
and none is answered by the code that exists.

### 1. Standing permission, or permission each time?

The picture requires standing permission. A person who must walk to the second
machine for every job is not running a company with several computers; they are
operating two computers by hand.

**Recommended:** the grant is made once, in advance, from the settings surface,
and it is the existing kind — enumerated, expiring, revocable in one action.
What changes is *when* it is made, not what it is. The machine doing the work
still checks a grant at the moment the work arrives; it simply finds one already
there.

**Rejected:** a machine that accepts work because the person owns both. Ownership
is not a grant. If ownership were sufficient, a stolen machine would carry its
owner's authority to every other machine they own, and the revocation that ADR
0001 makes immediate would have nothing to revoke.

### 2. What crosses the wire — a goal, or a command?

**Recommended:** a goal. The asking machine says what it wants done; the working
machine decides how, under its own rules, its own capability model and its own
person's grants. A command would make the second machine an extension of the
first, and then the first machine's compromise is every machine's compromise.

This also keeps [ADR 0009](../decisions/0009-a-good-computer-without-the-agent.md)'s
shape: the working machine's manual path and agent path are both still its own.

### 3. How does the person see what is happening?

**Recommended:** the state of every machine's work is visible from the machine
the person is sitting at, without asking each machine in turn. A person who has
to visit three computers to find out what three computers are doing has not been
given anything.

Two things this must not become: a dashboard that is only truthful when every
machine is reachable, and a summary that silently omits a machine it could not
reach. **A machine that cannot be reached is a state, and it is shown as one.**

### 4. Where does the result land?

**Recommended:** where the person asked, not where the work ran. The whole point
is that the person keeps one place to look.

### 5. Stopping it.

**Recommended:** one action, from where the person is sitting, and it reaches
work already running rather than only preventing new work. Revocation in this
repository already means *immediately*, because the key goes with the row. Work
in flight has to honour the same word or the word gets weaker without anyone
deciding that it should.

## The machine is a window on the canvas

Adding a machine puts it on the canvas as a window. The person pans to it,
focuses it, sets it aside, or fills the screen with it, exactly as they do with
anything else — the Dock's rule already covers it without amendment: *click an
app to return to where you last used it.* A machine is one more thing they last
used somewhere.

This is a better fit than it first looks. The canvas already holds windows at
signed coordinates on a plane that extends past every edge, already pans to a
window rather than moving it, and already knows how a window sits — on the
canvas, put aside, or filling the screen. A remote machine does not need a new
surface invented for it. It needs to be a window.

**Control means the keyboard and the pointer go there.** That is the part that
is not just a picture of a screen, and it brings two questions the work-sending
half does not have.

### Who is acting, when the window is another machine

`docs/design/who-is-acting.md` exists because a person must always be able to
tell whether they or the agent did something. A window showing another machine
raises the same question one step further out: **input crossing into that window
leaves this machine.** A person who cannot tell at a glance which window is here
and which is elsewhere can type a password into a machine they did not mean to.

So the distinction has to be carried by the window itself, and it has to hold
when the window fills the screen — which is exactly when every other cue is
gone.

### The danger that is not obvious, and I think it is the sharp one

**Full pointer-and-keyboard control is a general verb, and this repository has
no general verbs.**

[ADR 0001](../decisions/0001-the-capability-model.md) says an agent reaches the
machine only through enumerated verbs, and
[ADR 0043](../decisions/0043-the-terminal-is-a-persons-and-never-an-agents.md)
says the terminal is a person's and never an agent's — both for the same reason:
a general-purpose door has every verb behind it and none of them is on a list.

A pointer and a keyboard on another machine's screen are exactly such a door. An
agent that could drive one would hold every verb that machine's interface
offers, enumerated by nobody, revocable only by taking the whole thing away. It
would be ADR 0043's terminal reached by another road.

**An earlier draft concluded from that: refuse it, the person's path only. The
owner overruled it, and the overruling is right.** The standing rule is that a
person may choose what they prefer, that the danger is indicated, and that a
feature is not removed to prevent the choice — *not limiting; letting them
decide.* Nothing in the analysis above changes: a pointer really is a general
verb. That is why the choice must be **deliberate, informed and narrow**, not
why it must be withheld.

**Recommended:** the person decides, per machine, whether their agent may
control it.

- **Off until they turn it on.** A machine added is a machine the *person* can
  sit down at. The agent driving it is a second act and deserves a second
  decision — [ADR 0025](../decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
  already holds that a default is what a machine arrives able to do, and
  arriving able to be driven by an agent is not something anyone asked for.
- **The same kind of grant as every other** — enumerated, expiring, revocable in
  one action ([ADR 0001](../decisions/0001-the-capability-model.md) §3). But what
  is enumerated here is *the machine*, not the verbs, and that is the honest
  difference: **this grant cannot enumerate what it permits, because a pointer
  permits everything.** The record must say that in those words, or the grant
  looks like the others and is not.
- **Said plainly where the choice is made**, in the person's own language: this
  lets the agent do on that machine anything you could do there. Not a warning
  box to click past — the plain fact, at the moment of granting.
- **Visible while it happens, and stoppable from where the person is sitting.**
  `who-is-acting.md` already requires that a person can always tell whether they
  or the agent acted; a pointer moving on another machine is the hardest case
  that rule has met, and the one it exists for.

**This does not fit ADR 0043 as written, and a design note may not overrule an
accepted decision.** ADR 0043 keeps the terminal a person's and never an
agent's, and the force of the argument above is that this door reaches the same
place. So the ADR this needs must say in its own words either that a person may
open for their agent a door ADR 0043 keeps shut — with the owner's rule as the
reason — or how the two genuinely differ, if on inspection they do. **What it
may not do is land quietly beside ADR 0043 and leave a reader to assume they
agree.**

## The dangers, named rather than removed

The owner's standing rule is that a person may choose what they prefer, that the
dangers are indicated, and that a feature is not removed to prevent a choice. So
these are stated in the record and shown to the person, and none of them is a
refusal.

- **A machine that works for you unattended is a machine that can be used
  without you.** Standing permission is what makes the picture work and it is
  also its cost. The mitigation is that grants expire and are enumerated, not
  that they are withheld.
- **One person confirming twice is a weaker ceremony than two people confirming
  once.** Said plainly at the moment it happens, not buried.
- **Work sent to another machine is still egress, and the indicator still
  fires.** `docs/features.md` already refuses the exception for the office GPU
  box in as many words: *"it only went to the machine down the corridor"* is
  exactly the kind of exception that quietly ends a guarantee. The same applies
  here, and pairing is what makes the departure *wanted*, never what makes it
  silent.
- **A lost machine holds standing grants**, and now also a window somebody else
  can sit down in. Remote lock and wipe is already a v1 promise; this makes it
  load-bearing rather than optional, and that dependency belongs in the record.
- **A window that is elsewhere can be mistaken for one that is here.** The cost
  is a password typed into the wrong machine. Shown on the window, and shown
  when it fills the screen.
- **An agent allowed to drive another machine holds a permission that cannot be
  enumerated.** It is the only grant in this product of which that is true, and
  the person turning it on is told exactly that rather than being refused it.

## What this is not

- Not fleet management. `docs/features.md` keeps fleet enrollment and policy for
  alo OS machines at v1 and this is not that: there is no administrator here,
  and no machine that others obey.
- Not a server in the middle. Paired machines talk directly, as files and
  printers already do.
- Not a separate remote-desktop application. `docs/features.md` lists a remote
  desktop portal among the portals, and that is the mechanism; the surface is
  the canvas, not a program a person launches and then looks for a machine
  inside.

## What would have to be true before any of it is built

**Two physical machines on one office network**, which this project has never
had. ROADMAP records it as the open half of *machines find each other*: every
test so far puts both sides on one host. Building work-sending on top of a
pairing that has only ever been exercised between two processes on one machine
would put the first real test of both at the same moment.

That is a hardware condition, not an effort condition, and it is the same shape
as the disk that defers the installer's alongside walk.
