# Native control typefaces

Five files, three families, and they are not interchangeable. **Manrope is what alo's own
surfaces are drawn in**, because it is what the design is drawn in. Inter
remains for everything not yet moved to it, and as fallback. **Geist Mono is
what a terminal's cells are drawn in**, and nothing else.

**None is ever applied to an application's own text.** An application keeps its
typography; these are the shell's. The one exception is the terminal, where the
program has no typography of its own — it emits characters into a grid, and the
grid's face is ours.

## Manrope — alo's shell controls

Manrope variable font (weight axis), under the SIL Open Font License 1.1 in
`Manrope-OFL.txt`. Unmodified from google/fonts revision
`b31870aff700ab7a1d74fa0c6887d95beb9e0037`, path `ofl/manrope/Manrope[wght].ttf`.

**Why it is here.** `docs/design/the-external-window-edge.md` specifies
`Label/Medium` as *Manrope SemiBold 13, line height 18, letter spacing 0.1*,
read off `398:26245`. Until 2026-10-09 this crate bundled only Inter and every
caller asked for `Family::SansSerif`, so **nothing in this product drew the
typeface its design is drawn in**. The owner authorised bundling it that day.

**This file alone could not draw `Label/Medium`, and the sentence that used to
sit here said it could.** It read: *it carries the weight axis, so SemiBold is
weight 600 chosen through `Attrs::weight` rather than a second file.* That was
false the day it was written. A variable font is registered by fontdb as **one**
face at its default instance — for `Manrope[wght].ttf` that is weight 200,
ExtraLight — and cosmic-text 0.14.2 has no variation support at all, so there is
no axis to choose along. Worse, cosmic-text honours a requested family only
among faces whose weight matches **exactly**, so asking for Manrope at 600 did
not fall back to Manrope at 200; it left the family and drew **Inter**.

Measured 2026-10-14. See `../../../docs/quirks.md`, *the family you ask for is
honoured only at an exact weight match*. So a second file is exactly what was
needed, and it is the next section.

SHA-256: `3ae11c49db0455a3cc33e37d380f20fdb8c7f8b41dc07625c177e3d87a9d6ae6`.
164,700 bytes.
Upstream: <https://github.com/google/fonts/tree/b31870aff700ab7a1d74fa0c6887d95beb9e0037/ofl/manrope>.

## Geist Mono — the terminal's cells

Geist Mono variable font (weight axis), under the SIL Open Font License 1.1 in
`GeistMono-OFL.txt`. Unmodified from google/fonts revision
`9e25e2ba265e5298f70f6182dd4e8a3ebf1b9123`, path
`ofl/geistmono/GeistMono[wght].ttf`.

**Why it is here.** [ADR 0093](../../../docs/decisions/0093-the-palette-is-twelve-semantic-roles-and-the-design-file-decides-their-values.md)
records the brief as Manrope for interface labels with `System/Mono` in **Geist
Mono**, and
[ADR 0114](../../../docs/decisions/0114-the-terminals-engine-is-pinned-and-its-face-is-bundled.md)
is the terminal that needs it. The owner authorised bundling it on 2026-10-14,
as they did Manrope.

**Measured before it was added**, with a control: this folder held two
proportional faces and **zero** files anywhere asked for a monospace family. A
terminal asking for one would have got what the sign-in screen got on
2026-10-09 — *a box of the right size with nothing in it*, no error, correct
geometry, no glyphs. See `../src/window_control_label.rs`, which records that
boot.

**It is verified monospace rather than assumed to be.** Its `post` table reports
`isFixedPitch = 1`. A proportional face in a terminal does not fail loudly; it
draws a grid whose columns do not line up, which a cell-dimension test would
pass.

**And the same correction applies here.** The sentence that used to sit at this
point — *it carries the weight axis, so a bold cell is weight 700 chosen through
`Attrs::weight` rather than a second file* — was false for the same reason. This
file registers at weight 400 only. The design's `Terminal/Command` is **Medium,
weight 500**, and asking for it drew **Inter**, which is not even monospace. A
bold cell will need its own file too, when a bold cell is built.

SHA-256: `d00e590b8eb3a59acc329b2d044fd143ae935090b7da33199ebee27cc7de8196`.
171,948 bytes.
Upstream: <https://github.com/google/fonts/tree/9e25e2ba265e5298f70f6182dd4e8a3ebf1b9123/ofl/geistmono>.

## Manrope SemiBold — the static cut `Label/Medium` needs

Static instance, **unmodified**, under the same SIL Open Font License 1.1 in
`Manrope-OFL.txt`. From the Manrope project's own repository — the one that
licence's copyright line names — at revision
`468c0dbe38efa331b80bfe9448256abe27be44c3`, path
`fonts/ttf/manrope-semibold.ttf`.

**Why a second Manrope file.** The design's `Label/Medium` is *Manrope SemiBold
13 / 18 / 0.1*, read from the file on 2026-10-14. The variable file cannot serve
it, for the reason recorded above. This one can, because it declares
`OS/2.usWeightClass = 600` — so `Family::Name("Manrope")` with `Weight(600)` is
an exact match and the family is kept.

**Verified before it was added**, not assumed: typographic family (name ID 16)
is `Manrope`, which is the name a caller must ask for and not the ID 1 name
(`Manrope SemiBold`); `usWeightClass` is 600; and there is no `fvar` table, so
it is genuinely static.

google/fonts carries no static cut of Manrope — only `Manrope[wght].ttf` — which
is why this comes from the upstream project rather than from the same place as
the variable file.

SHA-256: `f43edb46a08e1538413ea2383ef01ed93cdab191a2c57d2489d9590985858cf7`.
145,104 bytes.
Upstream: <https://github.com/googlefonts/manrope/tree/468c0dbe38efa331b80bfe9448256abe27be44c3/fonts/ttf>.

## Geist Mono Medium — the static cut `Terminal/Command` needs

Static instance, **unmodified**, under the same SIL Open Font License 1.1 in
`GeistMono-OFL.txt`. From the Geist project's own repository — the one that
licence's copyright line names — at revision
`a0a06a3d916dcf92fe96f12051a124f89056b36a`, path
`fonts/GeistMono/ttf/GeistMono-Medium.ttf`.

**Why a second Geist Mono file.** The design's `Terminal/Command` is *Geist Mono
Medium, size 14, weight 500, line height 22, letter spacing 0*, read from the
design file on 2026-10-14 off the transcript node `487:29034`. The variable file
registers at 400, so the design's own weight drew Inter.

**Verified before it was added**: typographic family (ID 16) is `Geist Mono`;
`usWeightClass` is 500; `post.isFixedPitch` is 1, so it declares itself
fixed-pitch as the variable file does; and no `fvar` table.

**And the flag is not what the tests trust.** `isFixedPitch` is a declaration a
font can get wrong. The guard in
`../src/window_control_label_tests.rs` shapes `i`, `W`, `l`, `.`, `8`, `@` and
`m` through the real font system and asserts every advance is equal, because a
proportional face in a terminal does not fail loudly — it draws a grid whose
columns do not line up, which a cell-dimension test passes.

SHA-256: `6fa6a795c17a11bcbef410f52a853e8503f8e25d92762d202d907c54f5c0c0fb`.
150,096 bytes.
Upstream: <https://github.com/vercel/geist-font/tree/a0a06a3d916dcf92fe96f12051a124f89056b36a/fonts/GeistMono/ttf>.

## Neither static cut is a derivative, and that was checked

Both are the upstream projects' own files, byte for byte, so the OFL's
modification clauses do not arise. **It was checked anyway**, because the first
plan was to generate the instances from the variable files with `fonttools`:
none of the three licences in this folder declares a Reserved Font Name — zero
occurrences of the declaration form `with Reserved Font Name` in any of them —
so a derivative could have kept the family name. Taking the upstream files is
simply better provenance than generating equivalents.

## Inter — everything not yet moved, and fallback

Inter variable font (optical size and weight), under the SIL Open Font License
1.1 in OFL.txt. Unmodified from google/fonts revision 0b58fb370093f9a9f4ff785d94405710b79de67c,
path ofl/inter/Inter[opsz,wght].ttf. Embedded for deterministic offline native
labels as required by docs/design/figma-brief.md. No host font scan is required.

SHA-256: `29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031`.
Upstream: <https://github.com/google/fonts/tree/0b58fb370093f9a9f4ff785d94405710b79de67c/ofl/inter>.

## Fallback is deliberate and stays

All five are loaded, so a script none covers falls through to whatever else the
font database holds. **Dropping Inter would not simplify this** — it would
narrow the scripts this machine can draw, and a person whose language stops
rendering is a worse outcome than two files in a folder. The owner's direction
of 2026-10-09 says so in as many words: *preserving appropriate fallback fonts
for unsupported scripts.*
