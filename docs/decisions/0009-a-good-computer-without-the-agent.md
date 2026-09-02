# ADR 0009 — alo OS is a good computer without the agent

**Status:** accepted — completes [ADR 0007](0007-the-cpu-is-the-default.md) and
[ADR 0008](0008-where-inference-happens.md)
**Date:** 2026-09-02
**Context:** `docs/features.md`, `README.md`, the setup flow,
`docs/design/figma-brief.md`

## The decision in one line

A person can run alo OS with **no model, no provider and no agent at all** — and
what is left must be a desktop worth using on its own merits, not a crippled
version of one.

## What was actually wrong

Everything decided so far assumed the agent. ADR 0007 chose where the model runs;
ADR 0008 chose between three places it could answer from. Neither offered a
fourth answer that a great many people will want: **nowhere. I do not want this.**

That gap contradicts the product's own premise. alo OS is sold on control —
your models, your hardware, your building — and a system that *requires* an agent
has taken the largest decision out of the owner's hands before they have touched
it. "Sovereign, and you must use the AI" is not a coherent sentence.

It is also a quality problem. A system whose ordinary parts are only ever
experienced alongside an agent never has to be good on its own. Windows,
printing, tiling, the file manager: if those are only tolerable because something
clever sits on top, they are not finished, and the day the model is slow or
switched off the machine is revealed.

## The decision

**AI is a capability you can decline, at setup and at any time afterwards.**
Setup's "Where should your AI run?" gains a fourth choice — *not at all* — with
the same weight as the other three and no persuasion attached.

**With AI off, nothing essential is missing.** Files, windows, tiling, printing,
settings, updates, applications, the browser, accessibility — the whole system in
`docs/features.md` outside the agent sections — works exactly as it does with AI
on. There is no feature reachable only by asking for it.

**The agent's surfaces disappear rather than nag.** The hotkey does nothing, the
overlay does not exist, and Grants, Models and providers are absent from Settings
rather than present-but-disabled. A greyed-out feature is an advertisement.

**Turning it on later is a setting, not a reinstall.** And turning it off again
removes the agent's reach at once — grants end, nothing further is recorded as
agent activity.

**The record and the egress indicator stay.** They are not AI features. A person
who wants no agent may want *more* than average to know what left their machine
and when it updated. What the record contains simply has no agent entries in it.

## Consequences

- **A quality bar, deliberately set high.** The desktop must be worth choosing
  with the agent switched off. If the honest answer to "would somebody use this
  if the AI never worked?" is no, the ordinary parts are not finished — which is
  the same standard `docs/features.md` already applies by calling that section
  "why it is usable" rather than a differentiator.
- **The reachable fleet widens again**, past ADR 0007 and ADR 0008: a machine
  with no GPU, no provider and no interest in AI is still a customer, and on the
  Windows 10 fleet a great many of them will be exactly that.
- **The marketing claim changes shape** and improves: not "an AI operating
  system" but "a good, private desktop that has an agent if you want one".
- Anything an agent verb can do, a person must also be able to do by hand. That
  was implicit; with AI off it becomes structural, and it is a useful check on
  any verb somebody proposes.

## Alternatives rejected

**Require the agent, because it is the product.** Rejected: it contradicts the
control the product is sold on, and hides the quality of everything else behind
whether a model happens to be good that day.

**Ship a separate "lite" edition without AI.** Rejected: two systems to build,
test, certify and support, a split fleet, and a person who changes their mind has
to reinstall. It is a setting, not an edition.

**Keep the AI surfaces visible but disabled, so people can find them later.**
Rejected: a greyed-out panel is an advertisement wearing a disabled state.
Somebody who declined has been asked once already; Settings can offer it in one
place without following them around.
