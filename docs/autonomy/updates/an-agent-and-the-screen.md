# An agent and the screen

**Date:** 2026-09-17
**Workstream:** v0.5 — capture, and the room you are sitting in
**Task:** *An agent and the screen*
(`docs/autonomy/v0-5-capture-and-the-room-plan.md`, task 6)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
tested and linted in the Lima VM (Ubuntu 24.04 aarch64).
**Egress:** none.
**Status:** done.

## The finding first: the verb does not exist, and this task did not add one

`docs/contracts/agent-verbs.md` lists twenty-odd verbs — files, applications,
printing, networks, documents, software — and **not one of them captures the
screen.** The task says what to do about that: record it as a finding, and do
not add the verb here.

So no agent on any alo OS machine can see the screen today, by any road, and
nothing in this change makes one possible. What this change does is build **the
road it would have to come down**, so that the day somebody declares that verb,
the door already exists and already refuses everything else — rather than the
verb arriving first and a capture being wired to it in the same hurry.

## The one road

`alo_capturing::ForTheAgent` can be built only from an
`alo_capability::Authorised` — the evidence a verb's call was validated and a
person approved it. There is no constructor from a path, a name or an intention,
and since no capture verb is declared, **nothing in this repository can produce
one today.**

What it does when it exists:

- **The picture goes to the turn and nowhere else.** `to_the_turn` consumes the
  capture, so there is no second copy to index, to keep, or to hand over twice.
- **The record says it happened, and nothing of what was seen**: *a picture of
  the screen was taken for @the-agent*. A test asserts the sentence contains no
  window, title, text, pixel count or size — a record describing the contents
  would be the harvest this road exists to prevent, written down by us.
- **The indicator shows it, as the agent.**

## Tested directly, not assumed

The owner's instruction, and it changed what I wrote: *a capture the agent took
that the indicator did not show is the failure this whole workstream is built to
prevent, so test for it directly rather than assuming the path is shared.*

So the test asks the capture itself for the use it produces, and reads the line
`alo-in-use` draws from it:

| | |
|---|---|
| what is in use | the screen |
| by | the agent |
| colour | **terracotta** (ADR 0010) |
| the agent's mark | drawn |

And beside it, the test that keeps the colour meaning one thing: an application
using the screen is **not** terracotta and carries no agent's mark. If a change
ever made an application's line terracotta, the agent's colour would stop
identifying the agent, and both tests would have to be deleted to hide it.

## And no other road exists

`tests/the_agent_reaches_the_screen_by_one_road.rs` reads the shipped source of
the four crates a turn runs inside — `alo-agentd`, `alo-turn`, `alo-context`,
`alo-capability` — for six ways of reaching a picture of the screen, in code
rather than comments. None appears.

**If it stops you:** an agent sees the screen through a verb whose proposal says
*a picture of your screen* and which a person approved, or it does not see the
screen. A road added there would be this plan's failure arriving as a
convenience.

## Where task 4 stands, and why it is not done

**Task 4 is blocked, and not by this lane.** Its constraint says the encoder and
format are decided *with the devices plan's codec decision*, and that plan's own
constraint says tasks that record a format it has not settled wait on it, and
that no codec is added to the image before it is accepted.

That decision is task 1 of `v0-5-devices-and-media-plan.md`, it is `ready`, and
**no machine holds that plan** — the row now says so. Choosing an encoder here
would breach both constraints, and it is the same shape of thing as the speech
engine the owner has just taken as ADR 0050: a decision, not a task.

**Task 5 depends on task 4**, so it waits behind the same decision.

**What task 4 will owe when it is unblocked, and what this lane will say:**
recording the screen with sound records **whoever is speaking near the machine**
— a colleague at the next desk, someone on a call in the same room, a child in
the background — none of whom is looking at the screen, and none of whom agreed
to anything. The indicator shows the microphone in use, which is honest for the
person holding the machine and invisible to everybody else in the room. That is
the cost, it does not have a technical remedy, and the sentence a person reads
before recording with sound will say it plainly rather than listing *microphone*
as a checkbox.

## Handing over, as the owner routed today

- **The fine-tuning stack's pins, for the third PC's installer lane**, which
  owns `image/` — from `crates/alo-adapting/src/engine.rs`, so nobody re-derives
  them:

  | | |
  |---|---|
  | `torch` | 2.9.0+cpu |
  | `transformers` | 4.57.1 — `b10d05da8fa67dc41644dbbf9bc45a44cb86ae33da6f9295f5fbf5b7890bd267` |
  | `peft` | 0.18.0 — `624f69ca6393b765ccc6734adda7ca57d80b238f0900a42c357d8b67a03d62ff` |
  | `accelerate` | 1.11.0 |
  | `safetensors` | 0.6.2 |

- **The `alo_egress::Errand` variant** for sending an adapted model stays with
  this PC's lane A; `alo_adapting::leaving` says what it must carry.
- **The speech engine is decided** — ADR 0050, `espeak-ng` through
  `speech-dispatcher`, chosen for language coverage. Nothing here picks another.

## A mistake of this lane's, and what it cost

Three pointers in `alo-hosted` named ADR 0014 by the filename it had **before**
it was renamed. `alo-citing` refused them on `main` — not only for this lane, for
every machine — until the owner fixed it in `b158c3d`.

Why it was invisible: a renamed decision leaves every link to it reading exactly
as it did before. Nothing looks wrong until a check reads them. **This lane now
lists `docs/decisions/` and copies the filename rather than recalling it.**

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
