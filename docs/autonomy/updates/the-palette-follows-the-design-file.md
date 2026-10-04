# The palette follows the design file

- Date: 2026-10-04
- Workstream: appearance and the design file (`alo-appearance`, `docs/design/`)
- Contributor: Claude Code, the third PC
- Task: follow the colours in the design file, as semantic roles
- Status: **landed together — documentation, tokens, consumer migration and
  tests.** Dark-theme parity is **not** claimed and has no measured evidence yet.

## What was asked

The owner's direction of 2026-10-04: *follow the colours in Figma, update all
documents to use the Figma colour*, and then, when offered the choice of
squeezing twelve design roles into six existing colour names: **use semantic
roles rather than forcing twelve design roles into six colour names.**

## The problem, measured before anything was changed

`docs/design/palette.toml` named six colours. The design file names twelve
roles. Hex by hex:

```text
navy        #102A43   identical
deep-teal   #0F6B72   identical           ADR 0067's colour
cream       #F8F6F2   the file says #FAF7F2
warm-stone  #7A6F62   the file says #596B78
porcelain   #F4F1EC   no counterpart — not one role
charcoal    #1F2529   no counterpart — not a design variable
```

Two of six had drifted. Seven of the design's roles had no name here at all.

**And `porcelain` is why names were the wrong unit.** Four consumers used it and
meant four different things: the workspace canvas in `shipped::THE_SURFACE`, a
light ground to measure contrast against in `accent.rs`, a card surface, and a
recessed area. A colour name cannot be wrong about its role because it never
claimed one.

## What was built

**`Role`, twelve variants, in a new `crates/alo-appearance/src/role.rs`.** The
drawing vocabulary, under the design file's own variable names. It carries no
`Word`, no vocabulary key and no translation, because **nobody reads
`bg/cool`** — which is the deliberate opposite of `Token`'s choice and is what
kept this change out of `the-vocabulary.txt` and the 24-language obligation
entirely.

**`Token` keeps all six names and delegates its values to `Role`.** It is
serialised into a person's settings and `docs/contracts/person-settings.md` is a
public surface, so renaming a variant would change the meaning of a value
somebody has already stored. Three of the six take a corrected value; the
delegation means **every hex in the crate is now written in exactly one place**:

```text
Token::Navy       -> Role::TextPrimary     #102A43   unchanged
Token::DeepTeal   -> Role::AccentDefault   #0F6B72   unchanged, ADR 0067
Token::Cream      -> Role::BgCanvas        #FAF7F2   corrected
Token::Porcelain  -> Role::BgSurface       #FFFFFF   corrected
Token::WarmStone  -> Role::TextMuted       #596B78   corrected
Token::Charcoal      the one literal       #1F2529   retained
```

**Each of `porcelain`'s consumers was audited** rather than mapped to the
nearest-looking hex. `shipped::THE_SURFACE` moved from `Token::Porcelain` to
`Token::Cream`, because it really is the canvas and the name `Porcelain` now
carries white — shipping that would have shipped a pure white desktop.

**`charcoal` is retained and is not a design variable.** `accent.rs` measures
every choosable accent against it, so removing it removes the dark half of every
contrast check.

**ADR 0092** records all of it. It does **not** supersede or amend ADR 0067: deep
teal stays reserved, stays about five percent of a screen, stays never-alone, and
stays **not the ordinary selection or button colour** — which is the one thing a
semantic name invites a reader to assume, so the source, the brief and `role.rs`
each say so where a reader meets them.

## The gate runs without Figma, which is the point

**Two of the three machines working on this repository have no Figma access at
all.** A check only one machine can run is a check nobody can verify.

So `docs/design/figma-snapshot/variables.toml` commits what was measured — which
node, which day, which method — and `a_palette_with_one_source.rs` reads that.
The laptop confirmed independently that **nothing in any crate read
`docs/design/figma-snapshot/`** and that the 2,246,045-byte page export contains
none of `text/primary`, `bg/canvas`, `accent/default`, `text/muted`, `bg/surface`
or `border/default`. So the variables file is not an extra on top of the
snapshot; it is the only thing that can answer a palette question offline. That
search is theirs.

**Three limits of the measurement, each measured rather than assumed**, and in
the file where a reader meets it before trusting the gate:

- **The set cannot be enumerated.** `get_variable_defs` returns what a node's
  subtree uses, never the collection. **Thirty-two names by accumulation** — 17
  colour, 8 typography, 5 spacing, 2 radius — and no way to ask for the list. The
  variables are local rather than published, so `search_design_system` finds
  nothing; Code Connect needs an Organization plan and this account is Pro.

  **The snapshot said twenty-four until it was re-measured.** That number was
  right about a different quantity than its label: it counted the colours, the
  spacing and the radii and did not look at the typography the same node returns.
  Corrected, and the file now gives the per-node table rather than only a total.
- **The map is not mode-coherent.** The swatch *alo on dark* returns
  `accent/default = #77C8C6` beside a light `bg/surface`. **Measured from the
  wrong node, this would have carried the dark teal in as the agent's reserved
  light colour and every other test would still have passed** — which is why
  that one value has an assertion of its own.
- **Collection, mode and variable identifiers are not exposed** by any tool
  here. The direction asks for them, so they are recorded as an explicit gap;
  the REST endpoint that returns them needs a token this machine does not hold.

**And one audit gap that is a gap:** `get_variable_defs` on the minimized rail
returns `{}` — its fills are not bound to variables. A surface agreeing with the
snapshot is therefore not the same as a surface being correct.

## Two earlier records in this repository agreed, which was not expected

Four of the twelve could not be read from a purely light node. Rather than
resting on approval, they were checked against what the repository already held:

- **`the-palette-foundations-states.md`** (#192, 2026-09-27) is the owner's
  palette dictated a week earlier. It carries *muted text #596B78*, *cool
  surface #EEF2F4*, *warning #8C5A0A* and *danger #B42318* — **all four of the
  weak ones at exactly the measured values** — and ten of the twelve in all.
- **`the-canvas-in-numbers.md`** carries the other two from frame `70:29`.

**Every one of the twelve is attested by at least two independent records.**

That document also said of itself that it was the authority *until `palette.toml`
carries it*, and that the migration was *a task of its own, to be taken when the
canvas work is between landings*. **This was that task**, so it now says so at
the top instead of being left to contradict `palette.toml`, and its open question
— whether the list gains `text/secondary` and `border/default` — is closed: it
does. `bg/subtle` was measured and **not** adopted, and is recorded as such so
that *absent from the palette* stays distinguishable from *absent from the
design*.

## What it cost, measured

**The accessibility floor rose.** Every accent against both light grounds, at
EN 301 549's 4.5:1 for text:

```text
            cream  porcelain      bg/canvas  bg/surface
indigo       6.07     5.81           6.13      6.55
violet       5.76     5.52           5.82      6.22
moss         4.96     4.75           5.01      5.35
rose         5.44     5.21           5.49      5.87

worst 4.75  ->  worst 5.01
```

**And one real failure was fixed rather than carried across.** The old
`warm-stone #7A6F62` measures **4.36:1** on `bg/cool` and on `accent/soft` —
below the threshold. The design's `text/muted #596B78` measures **4.91:1** on
both. Metadata on a recessed surface was not legible enough, and nothing had
measured that pair because `bg/cool` did not exist here.

All twelve roles clear 4.5:1 on the grounds they are drawn on.

**One property is stated rather than hidden.** `border/default #E7EBEF` measures
**1.12:1** on the canvas, far below the 3.0 a meaningful shape needs. That is
what a hairline is, and it is acceptable only because a hairline is decoration:
no border in this palette may be the only thing conveying structure or state.

**Two published contrast numbers were re-measured, not adjusted.** The crate's
own arithmetic was reproduced first and agreed with both old numbers to five
places before either was replaced: navy on the canvas 13.56 → **13.70**, deep
teal on the canvas 5.78 → **5.84**. Terracotta goes 2.87 → **2.89** and stays
under both thresholds, so nothing ADR 0067 concluded moves.

## What is not done, and is not claimed

**Dark-theme parity.** The design binds the same twelve names to different values
in a dark mode, where `accent/default` is `#77C8C6`. The dark values that were
observed are recorded in the snapshot and **adopted by nothing**. `charcoal`
remains the dark ground.

**The rail's material.** The light rail's base is `bg/surface #FFFFFF` by the
owner's direction. A glass treatment is a base colour, an opacity, a blur, a
border and a shadow; four of those five are in no file here and are to be read
from the design file's actual fills and effects rather than invented from a hex.

**The typeface, and the brief disagrees with the design file about it.** Found
while re-verifying this change's own provenance claims — the node the colours
came from returns **eight typography variables** too:

```text
Display/Large    Manrope SemiBold 48/56   -1.2
Display/Medium   Manrope SemiBold 36/44   -0.8
Heading/Large    Manrope SemiBold 28/36   -0.4
Heading/Medium   Manrope SemiBold 22/30   -0.2
Body/Large       Manrope Regular  16/25    0
Body/Medium      Manrope Regular  14/22    0
Label/Medium     Manrope SemiBold 13/18   +0.1
System/Mono      Geist Mono Medium 12/18   0
```

**`figma-brief.md` says *Inter throughout, EB Garamond for the few editorial
moments*.** Neither appears as a variable in any node queried.

**And that one is already built, which makes it the more serious of the two
findings.** Measured here, and confirmed independently by the laptop lane, which
searched for what actually ships:

```text
crates/alo-shell/fonts/Inter.ttf        the only font file in the tree
window_control_label.rs:55              include_bytes!("../fonts/Inter.ttf")
window_control_label.rs:53              "the bundled, OFL-licensed Inter face
                                         specified by the design brief"
booting.rs                              a Font boot-failure variant
docs/contracts/native-window-controls.md:240
                                        "loads bundled Inter, the design
                                         brief's operating typeface"
```

Seventeen genuine references to the font across the crates and documents. Four
more matched `Inter` on a word boundary and are `Inter-|` from `/proc/net/dev`
parsing — the same substring fault as *a Mac* inside *a Machine*, and worth
counting because the laptop's first search returned 107.

**So it is not a drift between two documents. It is a difference between what the
design specifies and what the machine will actually render**, and it is the first
thing found in this work that a person would eventually see. The typeface is
compiled into the binary and **a public contract names the design brief as its
authority** — the same authority this change has just reversed for colour.

**The reversal is deliberately not extended to type.** The owner approved twelve
**colour** roles and said nothing about type; a typeface is a licensing decision
and a rendering decision as well as a design one; and changing it would move a
public contract and a boot path on an inference. The eight are recorded in the
snapshot as measured and unadopted, **no font is changed**, and this is a
question for the owner.

**Neither lane changed anything**, and both of us checked before saying so. The
Figma measurement is this lane's; what ships is the laptop's.

**Which Figma file is canonical.** `figma-brief.md` and `README.md` link
`8q0JVtnLroZYNdDkIQeJni`, which has one page and 43 frames and is this brief's
own output. Every measurement in `docs/design/` is from
`nDxyF5Ho9oC4RObjVzwBNJ`, which has 196 screens and 20,100 nodes. Both are real.
**Neither file was repointed**, because that is the owner's to say; the brief now
states the discrepancy where a reader meets it, and `README.md` is outside this
change's scope and untouched.

**Ten present-tense figures outside this change's scope, now off by 0.06.**
Counted rather than estimated, and the two cases separated because they are not
the same thing:

```text
"5.78:1"   10 occurrences, all present-tense claims about deep teal on cream
           9 in .rs doc comments:  alo-displays 2, alo-in-use 3,
                                   alo-notifying 2, alo-shell 2
           1 in docs/autonomy/evidence-a-person-can-work-on-it-all-day.md
           -> the pair now measures 5.84:1

"2.87:1"   16 occurrences, 10 of them attributed to what ADR 0010 measured
           -> still true as history; 2.87 was the value against cream #F8F6F2
```

**None of the twenty-six is an assertion.** Every occurrence was checked: all of
them are `//`, `///` or `//!` comments, or Markdown prose, so **no gate fails**
and the full suite is green. Every claim they make also remains true — deep teal
clears both thresholds at 5.84 as it did at 5.78, and terracotta misses both at
2.89 as it did at 2.87.

**They are left alone deliberately**, and listed here so the next lane does not
have to rediscover them. `alo-displays`, `alo-in-use`, `alo-notifying` and
`alo-shell` belong to other lanes, and `ROADMAP.md`, `QUEUE.md` and `STATE.md`
are reserved to the integration owner under `docs/autonomy/SHARED_MAIN.md`.
Editing another lane's doc comment to adjust a number that is not wrong is not
worth the collision; **the `5.78` figures are the ones a later change should
carry to 5.84**, and the `2.87` ones should be left as the historical
measurements they are.

## A document that was describing a file that no longer existed

Found while citing it. `docs/design/figma-snapshot/MANIFEST.md` recorded
2,246,045 bytes, 25,221 lines and 20,105 nodes; the `#489` refresh carried the
owner's Figma edits into `70-28.xml` — which was the point of taking a snapshot
— and left the manifest describing the file as it had been. The figures are
corrected to 2,247,251 / 25,217 / 20,100 and are now **asserted against the file
itself**, because a description a file cannot contradict is this repository's
recurring fault. The brief had copied the stale node count and is corrected with
it.

## Evidence

All nine gates, and the numbers in this report are from the run that landed it.
`crates/alo-appearance` is 146 tests, of which 9 are the palette gate.
