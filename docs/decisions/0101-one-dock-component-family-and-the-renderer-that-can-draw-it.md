# ADR 0101 — One Dock component family, and the renderer that can draw it

**Status:** **accepted, 2026-10-10**, by the owner, in these words:

> **1. One canonical component family**
>
> Use the newer Dock-edge family, including `348:23874`, as the baseline for
> shared visual styling. Preserve the bottom Dock's composer and system actions
> from `321:17430` / `77:107`.
>
> Normalise the shared specifications:
>
> - Application and overflow targets: **48×48** logical pixels, non-overlapping.
> - Application artwork: **28×28**, centred. This supersedes the earlier 32px
>   artwork proposal.
> - Horizontal Dock height: **76px**, with 14px above and below the targets.
> - Dock corner radius: **18px**.
> - Border: **1px**, using the corresponding semantic border token.
> - Shadow: the measured newer reference, **0 8px 24px rgba(7,19,31,.08)**.
> - Backdrop blur: **18**, with the designed opaque alternative when reduced
>   transparency is enabled.
> - Composer: preserve the measured **344×48**, radius **12** at the reference
>   size, with its text and actions.
>
> The 52×48 bottom measurement may include spacing around a target; distinguish
> that from the target itself. Do not silently treat all these measurements as
> interchangeable.
>
> Update the Figma components and visual contract to reflect this ruling. Keep
> the distinct edge layouts, including the horizontal composer for side Docks.
>
> **2. Approve the icon dependency work**
>
> You have approval to select and integrate a suitable maintained Rust SVG
> rasteriser through the normal dependency and ADR process, after checking
> whether the existing graphics stack already provides it.
>
> Use the approved artwork for alo's system controls. Real applications retain
> their actual application icons; the illustrative icons in Figma are not a
> replacement for every installed application's identity. First letters are a
> missing-icon fallback, not the finished Dock.
>
> **3. Complete the renderer integration**
>
> Implement the required transparency, rounded surfaces, shadow, icons and
> backdrop treatment. Check the production rendering path with the renderer
> owner before choosing the implementation.
>
> The opaque painter you inspected establishes its current limitation. It does
> not establish that changing one alpha argument completes shadows, or that
> framebuffer read-back is the required blur approach. Choose the mechanism
> against the actual graphics pipeline and verify it on screen.
>
> **4. Make the snapshot useful for visual verification**
>
> Extend the design capture to include fills and variable bindings, strokes,
> radii, effects, typography, visibility and component relationships, alongside
> geometry. State what the export cannot represent and keep reference
> screenshots for comparison.
>
> **5. Finish against the complete design**
>
> Include the alo Bar, divider, Show all, History, More, individual
> running/focus indicators and the styled overflow panel. Keep human
> interaction indicators separate from teal alo activity.
>
> Present matching Figma and production views for bottom/resting and
> top/overflow, with real applications. Verify the side variants too. Geometry
> tests remain necessary; visual and interaction verification are also required
> before this is complete.

Quoted whole because the normalised table *is* the decision, and a summary would
be a lane choosing which row bound it.

## What this settles that was open

`docs/design/the-docks-visual-specification.md` reported two Dock designs in the
file disagreeing on slot size, artwork size, radius, shadow and blur, and said
**neither was the 48 × 48 the owner had ruled**. This reconciles them: the edge
family is the baseline, the bottom Dock keeps its composer and system actions,
and the shared numbers are normalised above.

**The 52 × 48 is not a target and must not be read as one.** The owner's
sentence is the general rule and it is the one most likely to be lost: *the
52×48 bottom measurement may include spacing around a target; distinguish that
from the target itself.* A slot, the artwork in it, the pressable area and the
pitch from one to the next are four numbers, and this repository has already
published a contradiction by comparing two of them — see
`crates/alo-dock/src/measures.rs` on `GLYPH` against `ICON`.

So: **target 48 × 48**, **artwork 28 × 28 centred in it**, and whatever spacing a
layout puts between targets is a fifth number that belongs to the layout.

## The renderer: what this lane reported, and what is actually true

**This lane reported that the compositor can only draw opaque rectangles, and
that corner radius and shadow were therefore not buildable.** That was wrong, and
the owner's instinct about it was right. The report generalised from one file —
`crates/alo-shell/src/painted.rs`, whose `solid_run` does pass `alpha: 1.0` — to
the whole pipeline, without looking at the other paths in the same crate.

Measured afterwards:

| | |
|---|---|
| the renderer | **`GlesRenderer`** — OpenGL ES, through smithay `0.7.0` with `renderer_gl`, `renderer_pixman`, `use_system_lib` |
| textures | already used: `crates/alo-shell/src/lock_texture.rs` imports an `Abgr8888` buffer with `ImportMem::import_memory` and draws it with `Frame::render_texture_from_to` |
| alpha | that call takes an opacity argument, and `Abgr8888` is a **per-pixel alpha** format |

So the lock screen already does the thing that was called impossible. A rounded
surface, a shadow and a 28 × 28 icon are **one RGBA texture composited over the
scene**, which is a road this crate has had working all along.

**`painted.rs`'s opacity is a choice that file made**, not a property of the
machine. It is the right choice for what it draws — flat bands and text on a
known ground, where a texture upload per frame would be waste — and the wrong
one for a surface with a radius and a shadow.

**The blur is the one that is still open**, and this ADR deliberately does not
choose its mechanism. Framebuffer read-back is **not** established as required;
sampling what is behind the Dock is a GLES problem with more than one answer, and
the owner's instruction is to settle it with the renderer owner against the real
pipeline rather than from a reading of one file. **The opaque alternative is not
a fallback for a hard problem** — it is a designed state the owner named, for
when a person has asked for reduced transparency, and it ships whatever the blur
costs.

## Consequences

**The first letter is demoted to what it always was.** The owner: *first letters
are a missing-icon fallback, not the finished Dock.* `the-alo-dock.md`'s ruling
of 2026-10-10 stands for an application with no artwork and stops being what the
Dock shows.

**Illustrative is not canonical.** The seventeen `Dock edges / Button / *` icons
are alo's own system controls — Search, History, More, Settings — and the
applications drawn beside them in the file are illustration. A real machine
draws a real application's own icon, which is the host integration's to supply,
beside ADR-0101's artwork for alo's own controls.

**The snapshot gets a job it could not do.** `70-28.xml` holds five attributes
per node and can never show a colour or a corner changing. Item 4 extends it to
fills and bindings, strokes, radii, effects, typography, visibility and component
relationships — **and to say what it cannot represent**, which is the half that
keeps it honest, with reference screenshots kept for comparison.

**Done is not a green suite here.** The owner's last sentence: *geometry tests
remain necessary; visual and interaction verification are also required before
this is complete.* Matching Figma and production views for bottom/resting and
top/overflow, with real applications, and the side variants verified.
