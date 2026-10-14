# ADR 0110 — The agent writes the command and only the person runs it

**Status:** **proposed, 2026-10-14**, by the Mac lane, after the owner said the
feature list and the decisions must not be what stops a terminal worth
switching for: *"the features and adr should not stop us from making the wow
features to the terminal, we can edit them."*

**What is still the owner's**, and is the only thing this document leaves open:
whether any of it joins the six-item launch list in `ROADMAP.md`. Everything
else here is decided. The earlier word, relayed through the desktop lane, was
that the terminal comes after Sunday; that is a scheduling call and not a
design one, and nothing below depends on which way it goes.

## What is stuck, and it is not a missing feature

`docs/features.md` promises **a terminal** and says why: *law 2 forbids the
agent running arbitrary commands; it says nothing about a person, and an
operating system that does not trust its owner with a shell is a toy.* Since
2026-10-14 a person can open one — `Ctrl`+`Alt`+`T`.

And that is all it is: a terminal, like any other terminal, on a machine whose
whole argument is that its interface is an agent. **Nothing about it is ours.**

Meanwhile every AI terminal on the market — Warp, Wave, Cursor, Codex in a
shell, Claude Code in a shell — is built on the same shape: **the agent
executes.** It proposes and runs, proposes and runs, and the person supervises
a stream of actions they did not individually choose.

We cannot build that, and **the first draft of this document got the reason
wrong.** It said law 2 forbids an agent running commands at all. Law 2 says
something narrower and more useful: *running code is a separate grant that only
the person gives, at one of three levels* —
[ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md).
An agent **may** run code, at the level its person picked, and *at every level
each run is recorded and can be undone*.

So the undo in part 2 below is not an idea somebody had. **It is already what
law 2 requires**, and nothing in this repository does it.

What is forbidden is narrower still, and it is
[ADR 0043](0043-the-terminal-is-a-persons-and-never-an-agents.md) — *the
terminal is a person's, and never an agent's* — which is **accepted and built**.
`crates/alo-capability/src/persons_own.rs` holds a closed list of every terminal
emulator this machine can install, and three refusals against it: an agent
cannot be granted one, cannot ask for one whatever the grants file says, and
cannot **name** one in any argument of any call. A shell is the whole machine by
another road, so *there is no grant to `/`* is met a second time.

**The claim here is that law 2 is the advantage and not the handicap**, and
that it unlocks a product the others are structurally unable to copy.

## Why the others cannot copy it, stated as a mechanism

Codex and Claude Code are **processes inside a terminal they do not own**. They
run with the person's privileges, see what they are shown, and own no part of
the machine. Everything they offer is best-effort from userspace.

alo OS is the operating system under the terminal. It owns four things no
userspace agent can obtain:

| | what it is | what no process can do |
|---|---|---|
| the filesystem | one filesystem, chosen at install so a snapshot can be taken ([ADR 0045](0045-what-undoing-rewinds-to.md)) | undo a command |
| the kernel boundary | our own program on the kernel's security hooks ([ADR 0015](0015-the-kernel-learns-what-a-turn-is.md)) | say what a command actually did |
| the compositor | every window, and the selection, offered at the moment of invocation | read a failure without being pasted it |
| the model on the disk | inference with no network at all ([ADR 0008](0008-where-inference-happens.md)) | work with the cable unplugged |

**And the reason the first one needs law 2.** An agent running three hundred
commands cannot be meaningfully undone — there is no moment to rewind to, and a
person cannot be asked which of three hundred they meant. **One command a
person chose can be.** So *the agent does not execute* is not a restriction that
costs us the feature; it is the precondition that makes the feature coherent.

*Cannot execute* is not a flag anybody adds to an executing agent. A competitor
would have to rebuild from the kernel up.

## The decision

**Three parts. The first is the product; the other two are what make it true.**

### 1. A command the agent wrote, which only a person can run

The agent answers a request with a **command and a sentence about it**, drawn
in a surface the compositor owns. The person reads it and presses Enter, or
does not.

**The agent cannot press Enter, and this is a mechanism rather than a
promise.** The surface is the compositor's; the agent reaches it through an
enumerated verb that carries text and nothing else. There is no verb that
submits, no verb that types into somebody else's terminal, and no verb that
starts a shell. What the agent produces is a **proposal**, which is the same
shape as every other change it can ask for: *reads answer, changes wait*, and
what a person approves is the sentence they read.

**Why not write into the terminal the person already has open.** Two reasons,
and the second is already settled law here.

Mechanically: typing into a terminal that executes on newline *is* executing,
whatever we call the verb. A pty accepts a newline from whoever holds the file
descriptor and cannot tell an agent from a hand.

And by decision:
[ADR 0043](0043-the-terminal-is-a-persons-and-never-an-agents.md) bars an agent
from the terminal **application** in the capability layer, built. An agent
cannot be granted it, ask for it, or name it. So a verb that reached into a
running terminal would not merely be unwise — it would have to defeat three
refusals that exist to stop exactly that.

So the proposal lives in a surface we draw, where pressing Enter is a keystroke
the compositor received from a seat — and a seat is a person. **The surface is
the compositor's and is not a terminal**, which is what keeps this on the right
side of 0043 rather than arguing with it.

### 2. The run is reversible

A command a person runs this way is **wrapped in a snapshot**, taken before it
runs, from the road [ADR 0045](0045-what-undoing-rewinds-to.md) already
established for *undo what the agent did*. One key puts the filesystem back.

**This is the moat and it should be said plainly**: `rm -rf` in Claude Code is
gone for ever, because a process cannot snapshot a filesystem it does not own.
Here it is one keystroke.

**What undo does not cover is stated rather than hidden.** A snapshot rewinds
this machine's files. It does not recall a push, un-send an email, or undo
anything that already left — and the surface says so at the moment of
offering, not in a settings page. A reversibility we overstate once is a
reversibility nobody trusts again.

### 3. What it did is measured on request, and nothing is watched

After a run, the person may ask what it did, and the answer comes **from the
kernel boundary rather than from parsing what the command printed**. A command
that lies in its output lies to every other tool; it cannot lie to the hooks.

**Opt-in, per command, and this is load-bearing.**
`docs/features.md` carries a starred promise: *it forgets everything that was
not an agent. A syscall outside a turn is checked and leaves no trace — no log
line, no counter, no timestamp. Your editor, your browser and your terminal
pass through and are forgotten.*

The owner has said this file may be edited where a feature earns it. **This
feature does not need it edited, and spending it would be a bad trade.**
`alo watch cargo build` measures **one** command because a person asked for
that one. Nothing is watched; something was measured on request. The forgetting
promise comes out of this *stronger*, because the contrast is the pitch: the
one system that could watch everything is the one that measures only what you
point at.

So the promise stands unamended, and a lane that later finds it genuinely in
the way should come back here rather than edit it in passing.

*The design of the opt-in form is the desktop lane's, who proposed a
per-command egress readout, checked line 389 before proposing it, and found
that the always-on version broke it.*

## What this is not

- **Not an agent with a shell.** No `exec`, no shell, no "advanced" verb. The
  broker's list gains a verb that carries **text to a surface**, and the law
  that forbids the rest is untouched.
- **Not a reason to weaken the three levels.** [ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
  decides how code runs at the level the person picked — sealed box, ask each
  time, or full trust. A command a person runs from this surface is subject to
  it exactly as any other is.
- **Not watching a terminal.** See part 3.
- **Not a claim that any of it is built.** See below.

## What is built, and what each part needs

Honest, because this document will be read as a plan:

- **A terminal opens.** `Ctrl`+`Alt`+`T`, since #687. It is an ordinary
  terminal and nothing about it is ours.
- **A person's words reach the agent.** The Dock's composer, since #684, and
  `Super`+`A` reaches it since #690.
- **The snapshot filesystem is chosen at install**, and
  `crates/alo-image/tests/one_filesystem_and_it_can_hold_an_undo.rs` holds the
  image to it. **No command has ever been snapshotted.**
- **The boundary exists as a prototype and is honest about it.**
  `crates/alo-bounding`'s own header: *the smallest thing that can carry that
  idea or disprove it. One program, on one hook, holding one grant.* It is not
  a finished observer and must not be described as one.
- **The egress indicator draws.** The rest of the status area is owed a place.
- **The local model is on the disk** and the weights ship imported.

So: **part 1 needs a surface and one verb. Part 2 needs the snapshot road
reached from that surface. Part 3 needs the boundary finished.** In that order,
because part 1 is useless without 2 and 3 only answers a question 1 raises.

## The demo this is for

Thirty seconds, on a stage, and each beat is a thing no competitor can do:

1. Ask for something. The command appears with a sentence about it.
2. Press Enter. It runs — because a person ran it.
3. *What did that do?* Four thousand files written, three hosts contacted,
   `.npmrc` read. From the kernel.
4. Undo. Gone.
5. **Unplug the ethernet cable and do it again.** It still works.

Step 5 is the one to rehearse, because it is the only one that needs no
explaining and it kills every competitor in the room at once.
