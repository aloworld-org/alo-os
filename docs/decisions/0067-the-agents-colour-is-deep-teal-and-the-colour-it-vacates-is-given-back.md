# ADR 0067 — The agent's colour is deep teal, and the colour it vacates is given back

**Status:** accepted — amends [ADR 0010](0010-terracotta-is-reserved-and-never-alone.md),
whose reasoning stands and whose colour does not
**Date:** 2026-09-26
**Context:** `docs/design/palette.toml`, `crates/alo-appearance`
(`token.rs`, `accent.rs`, `contrast.rs`), `docs/design/figma-brief.md`,
[ADR 0065](0065-the-interface-a-world-its-places-and-the-objects-in-them.md)

## The decision in one line

**The agent is deep teal `#0F6B72`.** Terracotta stops being the agent's, and
because it is no longer a signal it is **given back to people as an accent**;
**verdigris is retired from the accent set**, because `#22707E` is the new
agent colour's neighbour and would do to teal exactly what ADR 0010 was written
to stop happening to terracotta.

## What changed, and what did not

The owner's interface direction of 2026-09-26 replaces the agent's colour. That
is a designer's decision and this decision does not argue with it.

**ADR 0010's reasoning is untouched and all of it still applies.** A colour that
means the agent must be reserved; an accent somebody can set to it destroys the
signal on precisely the machines whose owners liked it; and **a signal carried by
hue alone is not a signal**, because around one man in twelve cannot rely on hue
and EN 301 549 has required otherwise for twenty years. The mark and the word
beside the agent's colour remain mandatory, unchanged and unweakened.

What changes is which hue is reserved — and therefore which accent collides
with it.

## The collision this fixes

`Accent::Verdigris` is `#22707E` on a light ground. The new agent colour is
`#0F6B72`. They are the same hue at the same depth: side by side on a dock badge
nobody could tell them apart, and a person who chose verdigris would be running
a machine whose *the agent is acting* signal is also the colour of their own
folders.

That is ADR 0010's failure, reintroduced by moving the reserved colour without
moving the set designed around it. **A reserved colour and the accent set are one
decision, and changing either alone breaks both.**

## The decision

**Deep teal `#0F6B72` is reserved.** It means alo — present, acting, proposing,
or waiting for an approval — and it means nothing else. It is not in the accent
set and no personal setting selects it or a hue near it.

**Terracotta `#E76F51` is released.** It was only ever withheld because it was a
signal; it is not one any more, so withholding it would be a rule outliving its
reason. It becomes an accent, which is the answer ADR 0010 wanted and could not
give: a warm hue in a set that had none.

**Verdigris is retired.** Not moved, not re-toned — retired, because its name
means the blue-green of weathered copper and a verdigris that is not blue-green
is a lie told to make a migration cheaper.

| | On a light ground | On a dark ground |
|---|---|---|
| **Terracotta** | `#E76F51` | to be measured |
| Indigo | `#3A5AA8` | `#8AA0E6` |
| Violet | `#7A4E99` | `#BE97DE` |
| Moss | `#4A7546` | `#8DBE85` |
| Rose | `#A0466A` | `#E093AF` |

**Indigo becomes the default.** Verdigris held that place; indigo is the nearest
surviving hue to it and the furthest from the agent.

## What a machine that already says `verdigris` does

Verdigris is a **name in settings files that exist**, and it is the current
default, so most of them say it. Contracts outlive code, so removing the variant
and letting those files fail to parse is not available.

**A settings file naming verdigris is read, not refused**, and resolves to
indigo, and the machine records that it was migrated rather than changing
somebody's colour in silence. The name is accepted for one release and removed
in the next — expand, migrate, contract — and `AccentError::Reserved` now names
deep teal rather than terracotta.

## What must be measured before this ships

ADR 0010's own lesson was that a designer's hexes are a proposal until
`contrast.rs` has been over them. Three measurements are owed, and each fails a
release rather than reaching somebody who cannot read it:

1. **Deep teal on cream, porcelain and charcoal**, at text sizes as well as for
   fills.
2. **Terracotta as an accent on a dark ground.** ADR 0010 recorded terracotta's
   behaviour on cream; nobody has needed it on charcoal before, and the dark
   value in the table above is deliberately blank rather than guessed.
3. **Deep teal against navy.** This one is new and is the risk in this change.
   Navy `#102A43` is human controls, text and selection; deep teal `#0F6B72` is
   alo. Both are dark, desaturated and blue — *you* and *the machine* are now
   told apart by two colours that a small dot at arm's length may not
   distinguish, where terracotta and navy could never be confused. The mark and
   the word already carry that signal for anyone who cannot rely on hue, which
   is why this is a risk and not a defect. But it is the thing to watch on a
   real screen, and if the two read as one colour the answer is to separate them
   in lightness, never to drop the mark.

   **How it is measured is [who is acting](../design/who-is-acting.md)**, added
   2026-09-26 on the owner's direction, and it is stricter than this ADR was.
   No alo state may be a small teal dot alone at any size: the teal arrives with
   the four-corner mark or a written **alo**. Work in progress is a named cursor
   and a boundary around the affected object. Human selection differs in
   **shape and control treatment**, not in being the navy version of the alo
   indicator. Both roles are tested on real displays at dock, window-control and
   selected-object sizes, in light, dark and **grayscale** — and the criterion is
   that a person can say who is acting **without naming the colour**, so an
   interface where the only difference is which blue it is has failed even when
   the observer answers correctly. If it fails, the lightness of the treatments
   or the surfaces behind them moves; the two brand anchors do not.

## Consequences

- `docs/design/palette.toml` is the source and changes there first; `token.rs`
  carries the constants and `a_palette_with_one_source.rs` refuses to let the
  two disagree, so the change cannot be half-made.
- `Token::Terracotta` stops being a palette token and becomes
  `Accent::Terracotta`. The palette keeps six colours.
- Every place the agent is drawn changes colour, and each one is checked for its
  mark and its word while it is open — ADR 0010's consequence, re-run.
- `docs/design/figma-brief.md` is held to `palette.toml` by the same test and
  moves with it.

## Alternatives rejected

**Keep verdigris and give the agent a hue further from it.** Rejected: the owner
chose the agent's colour, and an accent is not a reason to overrule a brand
decision.

**Keep verdigris and darken it away from the agent.** Rejected above — the name
stops being true, and the next person to read `#1A5560` and the word *verdigris*
learns not to trust colour names here.

**Leave terracotta reserved and simply unused.** Rejected: a colour withheld for
a reason that no longer exists is a rule nobody can explain, and the accent set
needed a warm hue.
