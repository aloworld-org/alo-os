# ADR 0093 — The palette is twelve semantic roles, and the design file decides their values

**Status:** accepted, 2026-10-04, by the owner's direction: *follow the colours
in the design file*, and *use semantic roles rather than forcing twelve design
roles into six colour names.*
**Date:** 2026-10-04
**Context:** `docs/design/palette.toml`, `docs/design/figma-brief.md`,
`docs/design/figma-snapshot/variables.toml`,
`docs/design/the-palette-foundations-states.md`,
`docs/design/the-canvas-in-numbers.md`, `crates/alo-appearance`
(`token.rs`, `role.rs`, `accent.rs`, `contrast.rs`, `shipped.rs`),
`docs/contracts/person-settings.md`,
[ADR 0010](0010-terracotta-is-reserved-and-never-alone.md),
[ADR 0067](0067-the-agents-colour-is-deep-teal-and-the-colour-it-vacates-is-given-back.md),
[ADR 0002](0002-the-shell-is-native.md)

**This does not supersede ADR 0067, and does not amend it.** Deep teal
`#0F6B72` remains reserved for the agent, under 0067's reasoning and 0010's
before it. Everything 0067 decided is still decided; this ADR is about the
other eleven colours and about where any of their values come from.

## The question in one line

**The design file names twelve colour roles; `palette.toml` named six colours;
two of the six had drifted from the file and one of them was not a role at all.
What is the palette, and what keeps it the same in three places?**

## What was measured, 2026-10-04

```text
palette.toml                   6 entries, format 1, named by colour
the design file's variables   32 names observed, no way to enumerate the set
alo_appearance::Token          6 Rust constants, the drawing authority
figma-brief.md                 a 6-row table, drifted in 2 values
```

Hex by hex, against the design file:

```text
navy        #102A43   identical
deep-teal   #0F6B72   identical          ADR 0067's colour, untouched
cream       #F8F6F2   the file says #FAF7F2
warm-stone  #7A6F62   the file says #596B78
porcelain   #F4F1EC   no counterpart — not one role
charcoal    #1F2529   no counterpart — not a design variable
```

And six of the file's roles had no name here at all: `text/secondary`,
`bg/cool`, `border/default`, `accent/soft`, `status/positive`,
`status/warning`, `status/danger` — seven, counting `bg/surface`.

**`porcelain` is the one that shows why names were the wrong unit.** Four
consumers used it and meant four things: the workspace canvas in
`shipped::THE_SURFACE`, a light ground in `accent.rs`, a card surface, and a
recessed area. A colour name cannot be wrong about its role because it never
claimed one.

## The decision

**1. The palette is the design file's twelve roles, under the design file's own
names.** `text/primary`, `text/secondary`, `text/muted`, `bg/canvas`,
`bg/surface`, `bg/cool`, `border/default`, `accent/default`, `accent/soft`,
`status/positive`, `status/warning`, `status/danger`. A role says where a
colour goes; a colour name says only what it looks like.

**2. `docs/design/palette.toml` stays the source, at `format = 2`.** The
format number moves because the shape changed, so a reader written for 1
refuses the file rather than reading `text/primary` as though it were `navy`.
`ROADMAP.md` v0.01 — *the colours come from a source this repository can read,
and that is not CSS* — and ADR 0002 are why the source is TOML in this
repository and not a stylesheet in another.

**3. The values come from the design file, and are read from a committed
snapshot.** `docs/design/figma-snapshot/variables.toml` records what was
measured, from which node, on which day, by which method. **The gate runs
against that file and never against a live Figma**, because two of the three
machines working on this repository have no Figma access at all, and a check
only one machine can run is a check nobody can verify.

**4. `Role` is the drawing vocabulary and is not person-facing.** It carries no
`Word`, no vocabulary key and no translation, because no person reads
`bg/cool`. This is the opposite choice from `Token` and deliberately so: a
colour *name* is shown to somebody choosing one, and a colour *role* is an
instruction to a compositor.

**5. `Token` is retained, by name, with the six names unchanged.** It is what a
person picks when they want a plain colour behind their windows, it is
serialised into their settings, and `docs/contracts/person-settings.md` is a
public surface third parties build against. Renaming or removing a variant
would change the meaning of a value somebody has already stored.

**Each `Token` now delegates its value to the `Role` it equals**, so every hex
is written in exactly one place:

```text
Token::Navy       -> Role::TextPrimary      #102A43   unchanged
Token::DeepTeal   -> Role::AccentDefault    #0F6B72   unchanged, ADR 0067
Token::Cream      -> Role::BgCanvas         #FAF7F2   value corrected
Token::Porcelain  -> Role::BgSurface        #FFFFFF   value corrected
Token::WarmStone  -> Role::TextMuted        #596B78   value corrected
Token::Charcoal      the one literal        #1F2529   retained, see 7
```

**The names are retained and not deprecated.** A person choosing "cream" for a
background is choosing a colour name, which is a legitimate thing to call a
colour; what was wrong was a *drawing* authority organised that way. Three
values moved, and that is a correction rather than a renaming: the entry a
person stored still means the colour it named.

**6. Each of `porcelain`'s consumers was audited and given the role it
actually had**, rather than mapped to the nearest-looking hex:

```text
shipped::THE_SURFACE      the workspace canvas   -> Role::BgCanvas
accent.rs's light grounds canvas and surface     -> BgCanvas, BgSurface
a document, card or panel                        -> Role::BgSurface
a recessed or secondary area                     -> Role::BgCool
```

Note that `THE_SURFACE` moves from `Porcelain` to the **canvas**, while
`Token::Porcelain` itself becomes `bg/surface`. That is not a contradiction:
`THE_SURFACE` was mis-named, and resolving by role is what separates them.

**7. `charcoal` is retained at `#1F2529` and is not a design variable.** Every
choosable accent is measured against it — a hex that reads on cream is
illegible on charcoal — so removing it removes the dark half of every contrast
check. **This file is the light theme.** The design binds the same names to
different values in a dark mode, where `accent/default` measures `#77C8C6`, and
the dark mode carries candidates for the dark ground (`bg/subtle #183B56`,
`navy/950 #07131F`). **Neither is adopted, and dark-theme parity is not claimed
until it has its own measured evidence.**

**8. Three places hold the palette, and a gate holds them to each other.**
`palette.toml` is the source; `role.rs` is the copy a compositor can use at the
moment it draws a frame; `figma-brief.md` is the copy a designer reads.
`a_palette_with_one_source.rs` reads the source and the snapshot and fails on
any disagreement. Copies are legitimate and drift is not.

## What this does not decide

**It does not touch ADR 0067.** Deep teal is the agent's, it is about five
percent of a screen, it never appears without the mark or the word, and it is
**not the ordinary selection or button colour** — which is the one thing a
semantic name invites a reader to assume, so the source, the brief and `role.rs`
each say so where a reader meets them.

**It does not decide the rail's material.** The owner's direction is that the
light rail's base is `bg/surface #FFFFFF`; a glass treatment is a base colour,
an opacity, a blur, a border and a shadow, and four of those five are not in any
file here. `get_variable_defs` on the minimized rail (`337:23664`) returns `{}`
— its fills are not bound to variables at all. Those remaining measurements are
to be read from the actual fills and effects and not invented from a hex.

**It does not adopt a thirteenth role.** `bg/subtle` was measured, light
`#F2F4F6` and dark `#183B56`, and twelve were approved. It is recorded in the
snapshot as measured and unadopted, so *absent from the palette* stays
distinguishable from *absent from the design*.

**It does not decide the typeface — and the same authority question is open
there, unanswered.** Found while re-verifying this change's own provenance: the
node the colours were measured from returns **eight typography variables**, and
the family is **Manrope**, with `System/Mono` in **Geist Mono**. Neither Inter
nor EB Garamond appears as a variable in any node queried.

**`figma-brief.md` says *Inter throughout, EB Garamond for the few editorial
moments* — and unlike the colour table, that one is already built.** Measured
here, and confirmed independently by the laptop lane:

```text
crates/alo-shell/fonts/Inter.ttf        the only font file in the tree
window_control_label.rs:55              include_bytes!("../fonts/Inter.ttf")
window_control_label.rs:53              "the bundled, OFL-licensed Inter face
                                         specified by the design brief"
booting.rs                              a Font boot-failure variant —
                                         "the screen's own font could not be loaded"
docs/contracts/native-window-controls.md:240
                                        "loads bundled Inter, the design
                                         brief's operating typeface"
```

**So this is not a disagreement between two documents.** The typeface is
compiled into the shell binary, every sentence a person reads is laid out in it,
and **a public contract names the design brief as the authority for it** — the
same authority this ADR has just reversed for colour. A reader could reasonably
conclude the reversal extends to type. **It does not, and that is deliberate:**
the owner approved twelve **colour** roles and said nothing about type, and a
typeface is a licensing decision and a rendering decision as well as a design
one. Changing it would move a public contract and a boot path on an inference.

The eight are recorded in the snapshot as measured and unadopted, **no font is
changed**, and this is a question for the owner. It is written here rather than
only in a report because this ADR is what a reader will cite when they ask why
the brief is no longer the authority, and they are owed the limit of that in the
same place.

**Nobody would catch it by eye**, which is why it is worth an entry: Inter and
Manrope are both grotesques and the difference is in the letterforms. It is the
same reason deep teal needs the mark beside it.

**It does not resolve which Figma file is canonical.** `figma-brief.md` and
`README.md` link `8q0JVtnLroZYNdDkIQeJni`; every measurement in `docs/design/`
is from `nDxyF5Ho9oC4RObjVzwBNJ`. Both exist. That is the owner's to say, and
the brief now states the discrepancy instead of either file being quietly
repointed.

## What it costs, measured rather than estimated

**The accessibility floor rises.** Every accent is measured against both light
grounds at EN 301 549's 4.5:1 for text:

```text
                old grounds              new grounds
            cream    porcelain        bg/canvas  bg/surface
indigo      6.07       5.81             6.13       6.55
violet      5.76       5.52             5.82       6.22
moss        4.96       4.75             5.01       5.35
rose        5.44       5.21             5.49       5.87

worst 4.75  ->  worst 5.01
```

**And one real failure is fixed rather than carried over.** The old
`warm-stone #7A6F62` measures **4.36:1** on `bg/cool #EEF2F4` and on
`accent/soft #E8F4F2` — below the threshold. The design's `text/muted #596B78`
measures **4.91:1** on both. Metadata on a recessed surface was not legible
enough, and nothing had measured that pair because `bg/cool` did not exist here.

All twelve roles clear 4.5:1 on the grounds they are drawn on; the lowest is
`text/muted` on `bg/cool` at 4.91.

**ADR 0067's recorded number moves by two hundredths and its conclusion does
not.** Terracotta on cream was 2.87:1; on `bg/canvas #FAF7F2` it is 2.89:1.
Still far below 4.5, so terracotta remains neither a token nor an accent. 0067
is not edited for this; the number is recorded here.

**One property is stated rather than hidden.** `border/default #E7EBEF`
measures **1.12:1** on `bg/canvas` and 1.20:1 on `bg/surface` — far below 3.0.
That is what a hairline divider is, and it is acceptable only because a divider
is decoration: **no border in this palette may be the only thing conveying
structure or state**, which is the same rule the status colours are already
under. A control that needs a visible boundary uses `text/muted` or darker, not
`border/default`.

**The migration is contained, because the names were kept.** The 2026-09-27
record predicted a rename reaching *every drawing site in the shell, the same
forty-one files the deep teal change touched*. Delegating `Token` to `Role`
instead of renaming it means the drawing sites in `alo-shell`, `alo-in-use`,
`alo-notifying` and `alo-displays` keep compiling against the names they use:

```text
Token::WarmStone   5 uses, all inside alo-appearance
Token::Porcelain  15 uses, 4 outside alo-appearance
Token::Cream      47 uses, 31 outside — value changes, name does not
hard-coded old hexes outside alo-appearance:  2, both in alo-shell
```

`Role` adds no words, so `the-vocabulary.txt`, `words.rs` and the 24-language
obligation are untouched by this change.

## The evidence, and why four values stand on more than approval

Four of the twelve could not be read from a node that was purely light, because
`get_variable_defs` is node-scoped and flattens whatever modes a subtree used —
the swatch *alo on dark* returns the dark `accent/default` beside a light
`bg/surface`. Rather than resting on the owner's approval, they were checked
against what this repository already held:

- **`the-palette-foundations-states.md`** (#192, 2026-09-27) is the owner's
  palette dictated a week earlier. It carries *muted text #596B78*, *cool
  surface #EEF2F4*, *warning #8C5A0A* and *danger #B42318* — all four of the
  weak ones, at exactly the measured values — and ten of the twelve in all.
- **`the-canvas-in-numbers.md`** carries the other two from frame `70:29`:
  `text/secondary #274C68` and `border/default #E7EBEF`.

**Every one of the twelve is attested by at least two independent records.**
That earlier document also said of itself that it was the authority *until
`palette.toml` carries it*, and that the migration was *a task of its own, to be
taken when the canvas work is between landings*. This is that task, and that
document's open question — whether the list gains `text/secondary` and
`border/default`, or the page adopts the older names — is answered here: the
list gains them.
