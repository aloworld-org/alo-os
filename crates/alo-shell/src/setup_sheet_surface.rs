//! The rounded shapes on a first-start sheet, rasterised with their corners.
//!
//! The choice rows are cards and the first action is a filled button, and a
//! flat [`Solid`](crate::painted::Solid) is an axis-aligned rectangle with no
//! corner at all. `crate::sign_in_surface` met this first for the sign-in
//! entry and answered it with a signed distance field antialiased by one
//! `0.5 - d` rule; this is that answer for the sheets, and it reuses that
//! file's `TheColours` so a sheet follows the person's scheme and contrast for
//! the same reason the sign-in screen does.
//!
//! # What this draws
//!
//! The wallpaper — the same artwork `alo_folded_light` already
//! rasterises, because the capture shows the first-start sheets carry the
//! identical five vectors — then a card behind each choice row, and the filled
//! ground behind the first action.
//!
//! **Not drawn: text.** That is `crate::painted_text`'s, inked over this.

use smithay::utils::{Physical, Rectangle, Scale};

use crate::painted::Washed;
use crate::setup_sheet_layout::SetupSheetLayout;
use crate::sign_in_surface::TheColours;

/// Where the keyboard's focus ring is.
///
/// **Its own mark, never the selection one.**
/// `docs/design/the-first-start.md`: *selecting is shown by a navy border and
/// the word `Selected`*, and *keyboard focus is a 2px navy outer ring with a
/// 4px gap, and is its own thing*. Two marks because they are two facts: where
/// a person is, and what they have picked. A screen that drew only the second
/// leaves somebody pressing an arrow key with nothing to show it worked —
/// which is what this sheet did until 2026-10-13.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheRing {
    /// On a choice row, by its place in the visible window.
    OnAChoice(usize),
    /// On an action, by its place in the row of them.
    OnAnAction(usize),
}

/// What a sheet marks: what is chosen, and where the ring is.
///
/// One argument rather than two, because they are read together and a caller
/// that could pass one without the other would be a caller able to draw a
/// chosen row nobody can reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct TheMarks {
    /// Which choice row a person has picked, and [`None`] before they do.
    pub(crate) chosen: Option<usize>,
    /// Where the focus ring is, and [`None`] on a sheet with no list.
    pub(crate) ring: Option<TheRing>,
    /// Whether each row holds a field a person types into, and which one has
    /// the keyboard.
    pub(crate) fields: TheFields,
}

/// The fields a grouped sheet's rows hold — `562:486`, `562:532`, `562:536`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TheFields {
    /// The rows hold no field.
    #[default]
    None,
    /// Every row holds one, and this one has the keyboard: a two-unit ink
    /// outline where every other has the hairline.
    InEveryRow {
        /// The row whose field has the keyboard, if any.
        focused: Option<usize>,
    },
}

/// A grouped card's corner, and a grouped sheet's buttons': `562:363`'s 12.
const A_GROUPED_CORNER_IS: i32 = 12;

/// A field's corner inside a grouped row: `562:486`'s 10.
const A_FIELDS_CORNER_IS: i32 = 10;

/// How far outside a card the focus ring sits.
const THE_RING_IS_OUT_BY: f32 = 4.0;

/// How thick the focus ring is.
const THE_RING_IS_THICK: f32 = 2.0;

/// A choice card's corner, and the filled action's.
const THE_CORNER_IS: i32 = 14;

/// How thick a card's hairline is.
const THE_HAIRLINE_IS: f32 = 1.0;

/// Rasterise a sheet's shapes for this layout, at this scale, onto an output
/// of `pixels`.
///
/// **The output's own pixels, not the logical sheet converted back.** At a
/// fractional scale a sheet laid out at its pixels divided by the scale comes
/// back a pixel or two short when multiplied again, and that strip at the
/// right or bottom edge would be painted by nothing — the rule
/// `crate::sign_in_surface` keeps for the same reason.
///
/// `marks` carries what is chosen and where the focus ring is. Nothing picked
/// is the state every setup sheet opens in — which is the state every setup sheet opens in
/// (ADR 0025 and ADR 0014 §4: nothing is pre-selected). A sheet whose rows are
/// a field and a picture rather than a choice passes [`None`] always.
///
/// [`None`] when the sheet has no area, which is the one case with nothing to
/// draw rather than a refusal worth reporting.
#[must_use]
pub(crate) fn setup_sheet_surface(
    at: &SetupSheetLayout,
    pixels: (i32, i32),
    colours: TheColours,
    marks: TheMarks,
    scale: Scale<f64>,
) -> Option<Washed> {
    let area: Rectangle<i32, Physical> = Rectangle::from_size(pixels.into());
    if area.size.w <= 0 || area.size.h <= 0 {
        return None;
    }
    let (w, h) = (area.size.w, area.size.h);

    // The same wallpaper the sign-in screen draws, on the same ground.
    let ground = alo_folded_light::the_wallpaper(w, h, colours.ground);

    let hairline = f64::from(THE_HAIRLINE_IS) * scale.x;
    let in_pixels = |it: &Rectangle<i32, smithay::utils::Logical>| {
        it.to_physical_precise_round::<f64, i32>(scale)
    };
    // **One card for a grouped sheet's rows**, its rows divided by a hairline
    // and none of them a card of its own (`562:363`).
    let group = at.group.as_ref().map(in_pixels);
    let corner = if group.is_some() {
        f64::from(A_GROUPED_CORNER_IS) * scale.x
    } else {
        f64::from(THE_CORNER_IS) * scale.x
    };
    let rows: Vec<Rectangle<i32, Physical>> = at.choices.iter().map(in_pixels).collect();
    let cards: Vec<Rectangle<i32, Physical>> = if group.is_some() {
        Vec::new()
    } else {
        rows.clone()
    };
    let dividers: Vec<Rectangle<i32, Physical>> = if group.is_some() {
        at.choices
            .iter()
            .skip(1)
            .map(|row| {
                in_pixels(&Rectangle::new(
                    row.loc,
                    smithay::utils::Size::from((row.size.w, 1)),
                ))
            })
            .collect()
    } else {
        Vec::new()
    };
    let fields: Vec<Rectangle<i32, Physical>> = match marks.fields {
        TheFields::None => Vec::new(),
        TheFields::InEveryRow { .. } => at
            .choices
            .iter()
            .map(|row| in_pixels(&crate::setup_sheet_layout::the_field_in(*row)))
            .collect(),
    };
    let accessibility = in_pixels(&at.accessibility);
    // **Only the first action is filled.** The design draws *Continue* and
    // *Save PIN* on a navy ground and *Back* and *Use password only* as an
    // outline, which is how a person tells the way on from the way back
    // without reading either.
    // Every action in order, for the ring to sit on whichever has focus.
    let buttons: Vec<Rectangle<i32, Physical>> = at
        .actions
        .iter()
        .map(|it| it.to_physical_precise_round::<f64, i32>(scale))
        .collect();
    let filled = at
        .actions
        .first()
        .map(|it| it.to_physical_precise_round::<f64, i32>(scale));
    let outlined: Vec<Rectangle<i32, Physical>> = at
        .actions
        .iter()
        .skip(1)
        .map(|it| it.to_physical_precise_round::<f64, i32>(scale))
        .collect();

    let mut pixels = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            let px = f64::from(x) + 0.5;
            let py = f64::from(y) + 0.5;
            let mut colour = ground
                .get(((y * w + x) as usize).min(ground.len().saturating_sub(1)))
                .copied()
                .unwrap_or(colours.ground);

            // **The grouped card**: the surface in a hairline, once, and a
            // hairline between each row and the one above it.
            if let Some(group) = group {
                colour = in_a_hairline(colour, (px, py), group, corner, hairline, colours);
                for divider in &dividers {
                    let on = crate::sign_in_surface::coverage(crate::sign_in_surface::how_far(
                        px, py, *divider, 0.0,
                    ));
                    if on > 0.0 {
                        colour = crate::sign_in_surface::mixed(colour, colours.border, on);
                    }
                }
            }
            // **A field in each row**, the hairline at rest and two of the ink
            // where the keyboard is.
            for (which, field) in fields.iter().enumerate() {
                let focused = matches!(
                    marks.fields,
                    TheFields::InEveryRow { focused: Some(it) } if it == which
                );
                let field_corner = f64::from(A_FIELDS_CORNER_IS) * scale.x;
                colour =
                    if focused {
                        let outer = crate::sign_in_surface::coverage(
                            crate::sign_in_surface::how_far(px, py, *field, field_corner),
                        );
                        let mut it = crate::sign_in_surface::mixed(colour, colours.entry, outer);
                        let inner =
                            crate::sign_in_surface::coverage(crate::sign_in_surface::how_far(
                                px,
                                py,
                                crate::sign_in_surface::inset(*field, hairline * 2.0),
                                (field_corner - hairline * 2.0).max(0.0),
                            ));
                        it = crate::sign_in_surface::mixed(it, colours.ink, outer - inner);
                        it
                    } else {
                        in_a_hairline(colour, (px, py), *field, field_corner, hairline, colours)
                    };
            }
            // **Accessibility, as a control in a hairline** (`562:350`).
            colour = in_a_hairline(colour, (px, py), accessibility, corner, hairline, colours);

            // A card behind each choice: the surface, then its edge.
            for (which, card) in cards.iter().enumerate() {
                // **Selected is 2 logical pixels of the ink, and the card
                // grows a pixel on every edge to carry it.** That is why the
                // design file measures 85 for an ordinary row and 87 for the
                // chosen one, and `docs/design/the-first-start.md` settled it
                // over all 77 frames on 2026-10-07 by measuring every card
                // rather than opening a few: a surviving 1.5px border would
                // have measured 86, and no card in the file measures 86.
                //
                // **The ink rather than a literal navy.** The rule names
                // `#102A43`, which is what `Contrast::ink` is in the light
                // scheme; a scheme that redefines the ink redefines this with
                // it, and a navy border on a dark ground would be a rule
                // followed into invisibility.
                //
                // **And never colour alone.** The word `Selected` goes beside
                // it, which is text and so belongs to whoever inks this.
                let this_one_is_chosen = marks.chosen == Some(which);
                let (box_of, stroke, edge) = if this_one_is_chosen {
                    (
                        crate::sign_in_surface::inset(*card, -hairline),
                        hairline * 2.0,
                        colours.ink,
                    )
                } else {
                    (*card, hairline, colours.border)
                };
                let outer = crate::sign_in_surface::how_far(px, py, box_of, corner);
                let inside = crate::sign_in_surface::coverage(outer);
                if inside > 0.0 {
                    colour = crate::sign_in_surface::mixed(colour, colours.entry, inside);
                    let inner = crate::sign_in_surface::how_far(
                        px,
                        py,
                        crate::sign_in_surface::inset(box_of, stroke),
                        (corner - stroke).max(0.0),
                    );
                    let ring = inside - crate::sign_in_surface::coverage(inner);
                    if ring > 0.0 {
                        colour = crate::sign_in_surface::mixed(colour, edge, ring);
                    }
                }
            }

            // The way on: filled with the ink itself.
            if let Some(filled) = filled {
                let inside = crate::sign_in_surface::coverage(crate::sign_in_surface::how_far(
                    px, py, filled, corner,
                ));
                if inside > 0.0 {
                    colour = crate::sign_in_surface::mixed(colour, colours.ink, inside);
                }
            }

            // The ways back: the sheet's own ground inside a hairline.
            for action in &outlined {
                let outer = crate::sign_in_surface::how_far(px, py, *action, corner);
                let inside = crate::sign_in_surface::coverage(outer);
                if inside > 0.0 {
                    colour = crate::sign_in_surface::mixed(colour, colours.entry, inside);
                    let inner = crate::sign_in_surface::how_far(
                        px,
                        py,
                        crate::sign_in_surface::inset(*action, hairline),
                        (corner - hairline).max(0.0),
                    );
                    let ring = inside - crate::sign_in_surface::coverage(inner);
                    if ring > 0.0 {
                        colour = crate::sign_in_surface::mixed(colour, colours.border, ring);
                    }
                }
            }

            // **The focus ring, drawn last and outside everything.** It sits
            // `THE_RING_IS_OUT_BY` beyond the card with its own gap, so it
            // never touches the selection border and the two are never one
            // mark. Last so that a row which is both chosen and focused shows
            // both rather than one painted over the other.
            if let Some(ring) = marks.ring {
                let around = match ring {
                    TheRing::OnAChoice(which) => rows.get(which),
                    TheRing::OnAnAction(which) => buttons.get(which),
                };
                // **On a grouped row the ring is the row's own edge**: two of
                // the ink round the row, inside the card (`562:364`), where a
                // card of its own wears it standing off.
                let on_a_grouped_row = group.is_some() && matches!(ring, TheRing::OnAChoice(_));
                if let (true, Some(row)) = (on_a_grouped_row, around) {
                    let thick = f64::from(THE_RING_IS_THICK) * scale.x;
                    let on_the_ring =
                        crate::sign_in_surface::coverage(crate::sign_in_surface::how_far(
                            px, py, *row, corner,
                        )) - crate::sign_in_surface::coverage(crate::sign_in_surface::how_far(
                            px,
                            py,
                            crate::sign_in_surface::inset(*row, thick),
                            (corner - thick).max(0.0),
                        ));
                    if on_the_ring > 0.0 {
                        colour = crate::sign_in_surface::mixed(colour, colours.ink, on_the_ring);
                    }
                } else if let Some(around) = around {
                    let out_by = f64::from(THE_RING_IS_OUT_BY) * scale.x;
                    let thick = f64::from(THE_RING_IS_THICK) * scale.x;
                    let outer = crate::sign_in_surface::inset(*around, -(out_by + thick));
                    let inner = crate::sign_in_surface::inset(*around, -out_by);
                    let on_the_ring = crate::sign_in_surface::coverage(
                        crate::sign_in_surface::how_far(px, py, outer, corner + out_by + thick),
                    ) - crate::sign_in_surface::coverage(
                        crate::sign_in_surface::how_far(px, py, inner, corner + out_by),
                    );
                    if on_the_ring > 0.0 {
                        colour = crate::sign_in_surface::mixed(colour, colours.ink, on_the_ring);
                    }
                }
            }

            pixels.extend_from_slice(&[colour[0], colour[1], colour[2], 0xFF]);
        }
    }
    Some(Washed { area, pixels })
}

/// `colour` with a surface in a hairline over it, where `(px, py)` is on
/// `shape`: the entry's own colour inside, the border's on its edge.
fn in_a_hairline(
    colour: [u8; 3],
    (px, py): (f64, f64),
    shape: Rectangle<i32, Physical>,
    corner: f64,
    hairline: f64,
    colours: TheColours,
) -> [u8; 3] {
    let inside =
        crate::sign_in_surface::coverage(crate::sign_in_surface::how_far(px, py, shape, corner));
    if inside <= 0.0 {
        return colour;
    }
    let colour = crate::sign_in_surface::mixed(colour, colours.entry, inside);
    let inner = crate::sign_in_surface::coverage(crate::sign_in_surface::how_far(
        px,
        py,
        crate::sign_in_surface::inset(shape, hairline),
        (corner - hairline).max(0.0),
    ));
    let ring = inside - inner;
    if ring > 0.0 {
        crate::sign_in_surface::mixed(colour, colours.border, ring)
    } else {
        colour
    }
}

#[cfg(test)]
#[path = "setup_sheet_surface_tests.rs"]
mod tests;
