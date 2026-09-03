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

## Since it was accepted — the rule is wider than "off"

This ADR's standing rule was written as *anything an agent verb can do, a person
must also be able to do by hand*, and it was scoped to a machine where somebody
**chose** to have no agent. That scope is too narrow, and the case it misses is
the one ordinary people will actually meet.

**An agent is unavailable for many reasons, and only one of them is a choice:**

- it was declined at setup, which is what this ADR was written about;
- **the money ran out** — a subscription lapsed, a prepaid balance emptied, a
  provider returned *payment required* or *quota exceeded*;
- no model has been downloaded yet, or the download was corrupt;
- the machine is offline and the chosen source is not on it;
- the provider is down, or the key expired;
- the policy refuses this source and the person has not approved another.

**The rule holds in every one of them.** The agent is never the only path to any
capability. If a verb is the only way to do something, that thing is not
finished — it has a feature that disappears when somebody's card is declined.

### Why the money case matters most

It is the one that decides who alo OS is for. If the agent is the only way to
reach some part of the machine, then **the person who cannot pay is a
second-class user of a computer they own** — and this product is explicitly for
individuals as well as companies. A student, a pensioner, somebody between jobs,
somebody in a country where a card is not a given: none of them should meet a
machine that works less well because of it.

It is also the case most likely to be got wrong quietly, because it looks like a
technical failure and is not. `alo-answering`'s vocabulary today would report an
exhausted balance as `KeyNotAccepted` or `HavingTrouble(402)` — sending somebody
to check a key that is perfectly correct, or to read a number. Neither is true,
and neither tells them the one useful thing: *this will not work until you pay,
nothing else about your machine has changed, and here is what still does.*

### What follows

- **`alo-answering` needs a failure of its own for this.** Running out is not a
  fault, not a misconfiguration and not a transient error, and the three existing
  answers all send a person somewhere unhelpful. It also must never become a
  reason to ask somewhere else automatically — ADR 0008's *never a silent
  fallback* runs in both directions, and "we spent your money elsewhere because
  the first place was empty" would be the worst possible version of it.
- **No nagging.** A machine that cannot reach a model says so once, where it
  happened, and continues. It does not follow somebody around asking them to buy
  credit; that is the greyed-out panel this ADR already rejected, wearing a
  different disguise.
- **Whoever proposes a verb answers the question again**: what does a person do
  when there is no agent to run it? The honest answer must be a way to do it, not
  a reason it does not matter.

### The rule constrains what gets built, not only what verbs may do

Read as a rule about verbs it is too weak, because the way this promise is
actually broken is not by a verb doing too much. **It is by a surface never being
built, on the grounds that the agent covers it.** A settings panel with no search
because you can just ask. A mail client with no filters because the assistant
sorts it. Each omission is defensible alone and together they produce a machine
that cannot be operated by hand.

So: **no surface may be left out because an agent can do it instead.** Every ★
agent capability names the plain way to do the same thing, and if it cannot name
one, the plain way is missing work rather than an acceptable gap.

Applying that test to `docs/features.md` found three, immediately:

- **Searching files.** The agent could answer *"where is that file?"* at v0.5 and
  **nothing anywhere promised an ordinary search** — while a v1 line spoke of
  applications contributing to "one place to look", presupposing a search nobody
  had specified. As written, a person who could not use the agent could not find
  their own documents.
- **Why is it slow.** The agent could answer it; no window showed what was
  running.
- **What is filling my disk.** The agent could answer it; nothing showed sizes.

Three others passed: *I can't open this file* has archives and file
associations, *undo what the agent did* has the recovery screen, and *printers,
solved* has printers in Settings. The test is worth running on every ★ line that
is added from here, because two thirds passing is exactly the ratio that makes a
gap invisible.
