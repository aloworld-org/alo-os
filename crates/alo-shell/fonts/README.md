# Native control typefaces

Two faces, and they are not interchangeable. **Manrope is what alo's own
surfaces are drawn in**, because it is what the design is drawn in. Inter
remains for everything not yet moved to it, and as fallback.

**Neither is ever applied to an application's own text.** An application keeps
its typography; these are the shell's.

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
