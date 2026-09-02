# ADR 0010 — Terracotta is reserved for the agent, and the agent is never signalled by colour alone

**Status:** accepted — resolves a contradiction between `docs/features.md` and
`docs/design/figma-brief.md`, and unblocks queue item 8a
**Date:** 2026-09-02
**Context:** `crates/alo-appearance`, `docs/features.md` ("Making it yours"),
`docs/design/figma-brief.md`, the Figma file's screen 30

## The decision in one line

**Terracotta means the agent and nothing else**, so it is not offered as a
personal accent; a person chooses from a **designed accent set** that excludes it
and its neighbours — and because a colour alone cannot carry a signal anybody
must rely on, **the agent always appears with a mark and a word beside it**.

## What was actually wrong

Two documents promised things that cannot both hold, and the build loop found it
rather than guessing:

- `docs/features.md` promised an accent "drawn from the design tokens so the
  whole shell follows it" — a colour a person picks.
- `docs/design/figma-brief.md` said terracotta appears only where the agent is
  present or acting, about five percent of any screen.

**An accent a person can choose that happens to be terracotta destroys the one
signal that says the machine is acting on their behalf.** And the palette offered
no way out: the remaining tokens are structure and grounds rather than accents —
navy is unreadable against the charcoal dock, cream against the cream ground — so
there was nothing left to offer.

The loop was right to halt on it. Picking a colour is a designer's decision, and
inventing one to get past a blocked item is how a product's signals quietly stop
meaning anything.

## The decision

**Terracotta `#E76F51` is reserved.** It is not in the accent set, and no
personal setting can select it or a hue near it. It means the agent — present,
acting, or waiting for an approval — and it means nothing else anywhere in the
system.

**The accent set is designed, not derived.** Five hues, each with a value for a
light ground and a value for a dark one, because a single hex that reads well on
cream is illegible on charcoal. All five sit far from terracotta's hue, so none
of them can be mistaken for the agent at a glance.

| | On a light ground | On a dark ground |
|---|---|---|
| Verdigris | `#22707E` | `#5FB3C2` |
| Indigo | `#3A5AA8` | `#8AA0E6` |
| Violet | `#7A4E99` | `#BE97DE` |
| Moss | `#4A7546` | `#8DBE85` |
| Rose | `#A0466A` | `#E093AF` |

Verdigris is the default, and is a small piece of continuity: it is the name
`alo-workplace`'s colour scale still carries from before the palette became
terracotta.

**And the agent is never signalled by colour alone.** Wherever the agent
appears, terracotta arrives with a **mark** — the small dot — and a **word**:
"alo", "ALO WOULD LIKE TO", "answered on this machine". This is the part that
would have been left out if the contradiction had been resolved by simply
dropping the accent promise.

## Why the mark and the word are not optional

**A signal carried by hue alone fails for anybody who cannot distinguish that
hue.** Around one man in twelve has some form of colour blindness; deuteranopia
makes terracotta and moss neighbours. EN 301 549 conformance is a hard
requirement for the public-sector procurement this product is aimed at, and its
web equivalent has said for twenty years that colour must not be the only visual
means of conveying information.

So the honest reading is that "terracotta means the agent" was **never
sufficient**, accent set or no accent set. Finding the contradiction is what
exposed that — which is a better outcome than resolving it quietly.

## Consequences

- `crates/alo-appearance`'s `Token` gains the accent set with both values per
  hue, and refuses terracotta as a personal accent rather than silently
  accepting it.
- Every place the agent appears must be checked for a mark and a word. The Figma
  file's screen 30 currently offers terracotta as a choosable accent — that is
  the bug this ADR fixes, and it is fixed there too.
- The five accents want contrast verified against both grounds before they ship,
  at text sizes as well as for fills. The values above are a designer's
  proposal, not a measurement.
- Anybody adding a colour to the system now has a question to answer: does it
  mean the agent, is it structure, or is it one of the five?

## Alternatives rejected

**Drop the accent promise; terracotta is the only accent.** Rejected: personal
appearance is where somebody decides whether the machine is theirs
(`docs/features.md`, "Making it yours"), and taking it away to protect a signal
that needed a mark and a word anyway trades something real for something
imaginary.

**Let people choose any colour, including terracotta.** Rejected: it costs the
agent signal on precisely the machines whose owners liked terracotta enough to
choose it.

**Derive the accents from the existing tokens.** Rejected — it is what
`docs/features.md` said, and the loop demonstrated it cannot be done: the tokens
that are not terracotta are grounds and structure, and neither is legible as an
accent on both grounds.
