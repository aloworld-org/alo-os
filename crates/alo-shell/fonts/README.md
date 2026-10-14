# Native control typefaces

Three faces, and they are not interchangeable. **Manrope is what alo's own
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

It carries the weight axis, so SemiBold is weight 600 chosen through
`Attrs::weight` rather than a second file.

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

It carries the weight axis, so a bold cell is weight 700 chosen through
`Attrs::weight` rather than a second file.

SHA-256: `d00e590b8eb3a59acc329b2d044fd143ae935090b7da33199ebee27cc7de8196`.
171,948 bytes.
Upstream: <https://github.com/google/fonts/tree/9e25e2ba265e5298f70f6182dd4e8a3ebf1b9123/ofl/geistmono>.

## Inter — everything not yet moved, and fallback

Inter variable font (optical size and weight), under the SIL Open Font License
1.1 in OFL.txt. Unmodified from google/fonts revision 0b58fb370093f9a9f4ff785d94405710b79de67c,
path ofl/inter/Inter[opsz,wght].ttf. Embedded for deterministic offline native
labels as required by docs/design/figma-brief.md. No host font scan is required.

SHA-256: `29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031`.
Upstream: <https://github.com/google/fonts/tree/0b58fb370093f9a9f4ff785d94405710b79de67c/ofl/inter>.

## Fallback is deliberate and stays

Both are loaded, so a script neither covers falls through to whatever else the
font database holds. **Dropping Inter would not simplify this** — it would
narrow the scripts this machine can draw, and a person whose language stops
rendering is a worse outcome than two files in a folder. The owner's direction
of 2026-10-09 says so in as many words: *preserving appropriate fallback fonts
for unsupported scripts.*
