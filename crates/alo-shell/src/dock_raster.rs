//! The dock on one display: a floating bar, centred, clear of its own edge.
//!
//! # Whose decisions these are
//!
//! How thick it is and how long it needs to be are `alo-dock`'s answers for this
//! display's own size and how many icons there are. **Which edge it is on is the
//! person's**, since 2026-10-10: `Dock::edge` is their choice if they made one
//! and what the release ships otherwise, and [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
//! made the bottom the default rather than the only. This file turns those
//! answers into pixels and decides none of them. Asked once per display, so a
//! laptop and the screen beside it each get the dock laid out for their own size.
//!
//! **The text size reaches nothing here.** It did, through a row of names under
//! the icons that the owner removed on 2026-10-10 — the thickness is the measured
//! 76 across and 70 down a side at every text size. A *name* still scales, because
//! a name is a sentence: it is shown on hover and on keyboard focus, outside the
//! bar, at `DesktopLook::measure`'s size.
//!
//! # A bar, not a band
//!
//! It drew a band across the whole width, flush to the bottom edge, from before
//! ADR 0076 until now. **The designs show a bar**: as wide as what it holds,
//! centred, with room underneath it and either side.
//!
//! That is not decoration. A band is part of the screen's frame; a bar is a
//! thing lying on the canvas, which is what the Dock is — the canvas moves under
//! it and it does not move with the canvas. The gap beneath it is
//! `alo_dock::measures::FLOATING_ABOVE_THE_EDGE`.
//!
//! **What it holds is passed in.** This file does not count icons: how many
//! there are is `alo_dock::Holding`'s answer, so a bar drawn here cannot
//! disagree with the list the Dock decided to show.
//!
//! # When it will not fit, which is not yet handled
//!
//! A bar longer than the edge it runs along is **clamped** to that edge less its
//! margins, and the icons past the clamp are drawn outside the band or not at
//! all. That is the whole of what happens today.
//!
//! **This section said otherwise and was wrong**: *what does not fit went into
//! the overflow before it ever reached this file*, and *how many fit is
//! `alo_dock::fit`'s*. `alo_dock::fit` exists, is complete and is tested — and
//! **has no caller outside its own tests**, so nothing has ever put anything into
//! an overflow. Measured 2026-10-10: three calls in `alo-dock`'s own test module,
//! none anywhere else in the workspace. The sentence described the design and
//! read as a description of the code, which is the fault class `CLAUDE.md` names
//! as *declared place versus actual place*.
//!
//! What is right about it is the direction, and it stands as the rule for the
//! change that wires `fit` up: clamping is the last resort rather than the
//! mechanism, because shrinking icons to fit more is how a dock becomes unusable
//! exactly when somebody has the most open. `docs/features.md`'s **Where the Dock
//! goes** names the overflow as the part of that promise still owed.
//!
//! # The status area is not drawn here any more
//!
//! This file drew a segment at the far end of the band, set off by a rule in ink,
//! and called it the status area. ADR 0076 took the status area off the Dock: a
//! clock is not something a person opens or brings into focus, so it is not the
//! Dock's, and *where it goes instead* is handed to the shell's own plan rather
//! than answered here.
//!
//! **The egress indicator was not left floating with it.** It still sits where it
//! sat, and `crate::egress_status_place` now works its corner out from the screen
//! and the dock's thickness directly — which is what that file always did, minus
//! one indirection through a type the Dock no longer has. Nothing about where a
//! person looks for *nothing has left this machine* changed in this record.
//!
//! # Furniture, not authority
//!
//! Nothing here grants, approves or revokes, and nothing here measures: the
//! dock is a band of the person's colours with the accent along its inside
//! edge. `tests/desktop_source.rs` reads these files to hold that.

use alo_dock::measures::{FLOATING_ABOVE_THE_EDGE, GLYPH, ICON, MARGIN};
use alo_dock::places::Places;
use alo_dock::{Dock, Layout, OnTheDock, Room, Screen};
use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;
use crate::desktop_look::DesktopLook;
use crate::painted::{Inked, Solid};

/// The largest display side, in pixels, the dock is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The dock for one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// `alo-dock`'s layout for this display.
    pub(crate) layout: Layout,
    /// The whole band, along the bottom.
    pub(crate) band: Rectangle<i32, Physical>,
    /// The accent along the band's inside — that is, top — edge.
    pub(crate) accent: Rectangle<i32, Physical>,
    /// Flat shapes, in painting order.
    pub(crate) solids: Vec<Solid>,
    /// One letter per application on the Dock, inked into its icon's box.
    ///
    /// Empty when the Dock holds nothing, which is a Dock a person sees as a
    /// bare band — and is what every machine showed until 2026-10-10, because
    /// nothing drew an icon at all.
    pub(crate) inked: Vec<Inked>,
}

/// Where the bar sits on this edge: `along` units down its edge, `thickness`
/// across, lifted `floating` clear of the screen's own edge.
///
/// **One function for four edges rather than four placements.** The bar is the
/// same rectangle each time — as long as what it holds, as thick as its lane — and
/// only two things vary: which axis the length runs along, and which end of the
/// other axis it is lifted from. Written as four `if`s in the caller, the two that
/// are never drawn in this release would be the two nobody notices going wrong.
///
/// **`floating` is the gap, not the position.** The bar is inset from its edge
/// rather than flush to it, which is the design file's rule for every edge and the
/// reason the egress corner has to subtract it too.
fn the_band_on(
    edge: alo_dock::Edge,
    size: (i32, i32),
    along: i32,
    thickness: i32,
    floating: i32,
) -> Rectangle<i32, Physical> {
    let (width, height) = size;
    let origin = match edge {
        alo_dock::Edge::Bottom => ((width - along) / 2, height - thickness - floating),
        alo_dock::Edge::Top => ((width - along) / 2, floating),
        alo_dock::Edge::Left => (floating, (height - along) / 2),
        alo_dock::Edge::Right => (width - thickness - floating, (height - along) / 2),
    };
    let extent = if edge.runs_across() {
        (along, thickness)
    } else {
        (thickness, along)
    };
    Rectangle::new(origin.into(), extent.into())
}

/// The accent along the band's **inside** edge: the one facing the canvas.
///
/// **A picture found this and three tests did not.** The accent was
/// `(bar_width, rule)` at the band's own corner for every edge — right for the
/// bottom, and for the other three a line of the wrong length on the wrong side:
/// on the left edge it ran **176 pixels out across the canvas** above a bar 70
/// wide, because the length was taken along the screen's width while the bar runs
/// down its height. Measured off a drawn frame on 2026-10-10, where it is
/// impossible to miss and where a reader of the code had missed it.
///
/// `the_bar_sits_on_whichever_edge_it_was_laid_along` asks `the_band_on` about
/// all four edges and never looks at the accent;
/// `the_dock_is_a_centred_bar_clear_of_the_bottom_edge` checks the accent and
/// only ever on the bottom. Each was right about its own subject, and the gap
/// between them was the whole of the defect.
///
/// **Inside means facing the canvas**, which is the opposite end of the thickness
/// from the screen edge the bar is lifted off. So it is the band's top for a dock
/// along the bottom and the band's bottom for one along the top — not the same
/// corner twice.
fn the_accent_on(
    edge: alo_dock::Edge,
    band: Rectangle<i32, Physical>,
    rule: i32,
) -> Rectangle<i32, Physical> {
    let (origin, extent) = match edge {
        alo_dock::Edge::Bottom => ((band.loc.x, band.loc.y), (band.size.w, rule)),
        alo_dock::Edge::Top => (
            (band.loc.x, band.loc.y + band.size.h - rule),
            (band.size.w, rule),
        ),
        alo_dock::Edge::Left => (
            (band.loc.x + band.size.w - rule, band.loc.y),
            (rule, band.size.h),
        ),
        alo_dock::Edge::Right => ((band.loc.x, band.loc.y), (rule, band.size.h)),
    };
    Rectangle::new(origin.into(), extent.into())
}

/// Lay `dock` out on a display of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::DesktopScene`] for a display `alo-dock` refuses to lay a dock
/// out on, or one larger than any this dock is laid out for;
/// [`RenderError::AccentRefused`] for a look whose accent is not one a person
/// can choose.
pub(crate) fn picture(
    dock: &Dock,
    look: DesktopLook,
    size: (i32, i32),
    on_the_dock: &[OnTheDock],
    fonts: &mut FontSystem,
) -> Result<DockPicture, RenderError> {
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::DesktopScene);
    }
    let screen = Screen::of(
        u32::try_from(width).map_err(|_| RenderError::DesktopScene)?,
        u32::try_from(height).map_err(|_| RenderError::DesktopScene)?,
    )
    .map_err(|_| RenderError::DesktopScene)?;
    let palette = look.palette().map_err(|_| RenderError::AccentRefused)?;
    let measure = look.measure();
    let layout = dock.layout_on(screen);
    let thickness = i32::try_from(layout.thickness().as_pixels())
        .map_err(|_| RenderError::DesktopScene)?
        .clamp(1, width.min(height));
    let rule = measure.px(2).min(thickness);

    // As wide as what it holds, clamped to the screen less its margins. The
    // count comes from whoever decided what the Dock shows, so the bar drawn
    // and the list decided cannot disagree.
    let margin = i32::try_from(MARGIN).unwrap_or(i32::MAX);
    let floating = i32::try_from(FLOATING_ABOVE_THE_EDGE).unwrap_or(i32::MAX);
    let wanted = i32::try_from(Room::a_bar_holding(on_the_dock.len()).as_pixels())
        .map_err(|_| RenderError::DesktopScene)?;
    // **The edge the bar runs along, not always the width.** A dock down a side is
    // as long as the screen is tall, and clamping it to the width would make a
    // portrait screen's side dock short for a reason that has nothing to do with
    // where it is — which is the mistake `alo_dock::layout`'s own header warns
    // about one level up, for thickness.
    let along = if layout.edge().runs_across() {
        width
    } else {
        height
    };
    let widest = (along - 2 * margin).max(1);
    let bar_width = wanted.clamp(1, widest);

    // Centred along its edge, and lifted clear of it.
    let band = the_band_on(layout.edge(), size, bar_width, thickness, floating);
    let accent = the_accent_on(layout.edge(), band, rule);

    let solids = vec![
        Solid {
            area: band,
            colour: palette.dock,
        },
        Solid {
            area: accent,
            colour: palette.accent,
        },
    ];

    // **One letter per application, which is what a Dock icon is today.** The
    // owner ruled on 2026-10-10 that an application with no artwork shows its
    // first letter, and `alo-applications` names an icon in none of its files —
    // so this is every application rather than a fallback anybody will see
    // rarely. `docs/design/the-alo-dock.md` carries the ruling and the three
    // decisions inside it.
    //
    // **`alo_dock::Places` says where they go and this says nothing about it.**
    // That crate lays the icons out along the bar without knowing which edge the
    // bar is on — `from_the_start` is a distance **along**, not an `x` — so the
    // one thing left here is which axis *along* is, which is the same question
    // `the_band_on` above already answers for the band.
    let places = Places::of(on_the_dock);
    let icon = i32::try_from(ICON).unwrap_or(i32::MAX);
    let glyph = i32::try_from(GLYPH).unwrap_or(i32::MAX);
    // **Centred across the bar, which is a rule rather than a number.** This read
    // `MARGIN` — 8 — until 2026-10-10, and the bar is 76 thick with a 48 icon in
    // it, so every letter sat six logical pixels above where the design puts it.
    // The owner's ruling that day says where it goes: *the measured 48px
    // application target centred vertically: 14px above and below.*
    //
    // Written as the half of what is left over rather than as 14, because the two
    // orientations have different thicknesses — the measured 76 across, the
    // measured 70 down a side, where the same rule gives 11 — and a constant
    // taken off one frame would be wrong on the other. `CLAUDE.md` names this as
    // one of the three honest kinds of number: a rule that was never a number.
    //
    // `MARGIN` stays where it belongs, which is the room at the bar's **ends**:
    // `alo_dock::Places` has already put it into `from_the_start`. The two were
    // one number while the thickness was `MARGIN + ICON + MARGIN`, and that is
    // what let this line look derived for as long as it did.
    let across = ((thickness - icon) / 2).max(0);
    let mut inked = Vec::new();
    for place in places.each() {
        let along_the_bar = i32::try_from(place.from_the_start()).unwrap_or(i32::MAX);
        // The icon's own box, `ICON` square: `from_the_start` along the bar's
        // edge, and centred across its thickness.
        let (left, top) = if layout.edge().runs_across() {
            (band.loc.x + along_the_bar, band.loc.y + across)
        } else {
            (band.loc.x + across, band.loc.y + along_the_bar)
        };
        // **The letter is inked on the dock's own colour**, so the box it
        // arrives in disappears into the band rather than drawing a tile the
        // design does not ask for. `docs/design/the-alo-dock.md`: a control at
        // rest contributes artwork and nothing else.
        let shaped = crate::painted_text::centred(
            fonts,
            place.app().first_letter(),
            icon,
            // **`GLYPH`, which is artwork's size and not text's.** The owner's
            // ruling of 2026-10-10 is explicit that a fixed bar height must not
            // become fixed-size text elsewhere — so this is worth saying
            // outright: the letter stands in for an application's icon, the
            // design gives that artwork 32 inside a 48 target, and *icon size
            // becomes a person's setting* is `[v0.5]` in `docs/features.md`.
            // It does not scale with the person's text size for the same reason
            // a company's logo does not.
            //
            // **A name is a different thing and must scale.** When the tooltip
            // that shows one is drawn — on hover and on keyboard focus, outside
            // the bar — it takes its size from `DesktopLook::measure`, as every
            // sentence on this machine does. `alo_dock::Room::a_line_at` is
            // what sizes it, and it still takes a `TextScale`.
            Metrics::new(glyph as f32, glyph as f32),
            palette.dock,
            palette.ink,
        );
        // Centred down the icon as well as across it. `centred` does the one
        // axis it can know about; this is the other.
        let down = top + ((icon - shaped.height) / 2).max(0);
        if let Some(letter) = shaped.placed(left, down, height) {
            inked.push(letter);
        }
    }

    Ok(DockPicture {
        size,
        layout,
        band,
        accent,
        solids,
        inked,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    /// `how_many` applications on the Dock, one window each.
    ///
    /// Built through `Holding::showing` rather than by hand, because
    /// `OnTheDock` has no public constructor and should not gain one for a
    /// test: the list a person sees is the one `alo-dock` decides, and a
    /// fixture that assembled its own would be drawing something the Dock would
    /// never show.
    fn on_the_dock(how_many: usize) -> Vec<alo_dock::OnTheDock> {
        let patch =
            alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("one by one is a patch");
        let mut windows = alo_dock::Windows::none();
        for number in 0..how_many {
            let named = format!("app-{number}");
            windows.opened(alo_dock::Window::of(
                alo_dock::WindowId::numbered(number as u64),
                Some(alo_dock::AppId::named(&named).expect("a named application")),
                "",
                patch,
                alo_dock::HowItSits::OnTheCanvas,
            ));
        }
        alo_dock::Holding::nothing().showing(&windows)
    }

    /// The bundled faces, as the shell loads them.
    fn fonts() -> cosmic_text::FontSystem {
        let mut fonts = cosmic_text::FontSystem::new();
        fonts
            .db_mut()
            .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
        fonts
            .db_mut()
            .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
        fonts
    }

    use super::*;
    use crate::desktop_testing::{an_appearance, noon_look};
    use alo_appearance::{Accent, TextScale};
    use alo_strings::Direction;

    /// Whether `inner` lies wholly inside `outer`.
    fn inside(inner: Rectangle<i32, Physical>, outer: Rectangle<i32, Physical>) -> bool {
        outer.intersection(inner) == Some(inner)
    }

    /// **The bar is placed on all four edges, and only the bottom is ever drawn.**
    ///
    /// `Dock::shipped` is on the bottom and nothing can change it, by the owner's
    /// order of work — so a test that went through `picture` could only ever
    /// exercise one of the four placements, and the other three would be reached
    /// for the first time by whoever turns the setting on.
    ///
    /// So this asks the placement directly. **It is the half of *all four edges
    /// work* that a drawing test cannot reach**, and the three untried branches are
    /// exactly where a wrong sign or a swapped axis would sit unnoticed.
    #[test]
    fn the_bar_sits_on_whichever_edge_it_was_laid_along() {
        use alo_dock::Edge;
        let size = (1920, 1080);
        let (along, thick, floating) = (600, 70, 8);

        for edge in Edge::EVERY {
            let band = the_band_on(edge, size, along, thick, floating);

            // The extent follows the orientation: long way along its edge.
            let (expect_w, expect_h) = if edge.runs_across() {
                (along, thick)
            } else {
                (thick, along)
            };
            assert_eq!(
                (band.size.w, band.size.h),
                (expect_w, expect_h),
                "{edge:?} is laid out across the wrong axis"
            );

            // Inset from its own edge by the floating gap, never flush to it.
            let gap = match edge {
                Edge::Bottom => 1080 - (band.loc.y + band.size.h),
                Edge::Top => band.loc.y,
                Edge::Left => band.loc.x,
                Edge::Right => 1920 - (band.loc.x + band.size.w),
            };
            assert_eq!(gap, floating, "{edge:?} is not floating clear of its edge");

            // Centred along the edge it runs down, to within a pixel.
            let (before, after) = if edge.runs_across() {
                (band.loc.x, 1920 - (band.loc.x + band.size.w))
            } else {
                (band.loc.y, 1080 - (band.loc.y + band.size.h))
            };
            assert!(
                (before - after).abs() <= 1,
                "{edge:?} is not centred: {before} before, {after} after"
            );

            // And wholly on the screen, which the arithmetic above could satisfy
            // while putting a negative origin somewhere.
            assert!(
                band.loc.x >= 0 && band.loc.y >= 0,
                "{edge:?} starts off the screen at {:?}",
                band.loc
            );
            assert!(
                band.loc.x + band.size.w <= 1920 && band.loc.y + band.size.h <= 1080,
                "{edge:?} runs off the screen"
            );
        }
    }

    /// **The accent lies on the band's inside edge, on all four edges.**
    ///
    /// It did not. It was the band's corner plus `(bar_width, rule)` whatever the
    /// edge, so on a dock down the left of the screen it drew a line 176 pixels
    /// long across the canvas beside a bar 70 wide, and on a dock along the top it
    /// drew on the band's outer edge rather than the one facing the canvas.
    ///
    /// **Found in a drawn frame rather than here**, which is the point worth
    /// keeping: two tests already covered the two halves — one asked
    /// `the_band_on` about four edges and never looked at the accent, the other
    /// looked at the accent on one edge — and the defect lived in the gap between
    /// their subjects. Each was correct.
    ///
    /// Four claims, because each catches a different way of being wrong: the
    /// accent is **inside** the band (length and axis), it is `rule` across and
    /// the band's **whole length** along (not a shorter or longer line), and it is
    /// flush with the **inside** face (not the outer one, which `inside` alone
    /// would allow).
    #[test]
    fn the_accent_lies_along_the_bands_inside_face_on_every_edge() {
        use alo_dock::Edge;
        let rule = 2;
        let band_for = |edge| the_band_on(edge, (1920, 1080), 600, 70, 8);

        for edge in Edge::EVERY {
            let band = band_for(edge);
            let accent = the_accent_on(edge, band, rule);

            assert!(
                inside(accent, band),
                "{edge:?}: the accent at {accent:?} leaves the band at {band:?}, so it draws on                  the canvas"
            );

            // `rule` across the thickness, the band's whole length along it.
            let (across, along, band_along) = if edge.runs_across() {
                (accent.size.h, accent.size.w, band.size.w)
            } else {
                (accent.size.w, accent.size.h, band.size.h)
            };
            assert_eq!(across, rule, "{edge:?}: the accent is not {rule} across");
            assert_eq!(
                along, band_along,
                "{edge:?}: the accent runs {along} along a band that runs {band_along}"
            );

            // Flush with the face that looks at the canvas, which is the far end
            // of the thickness from the screen edge the bar is lifted off.
            let (accent_edge, band_inside) = match edge {
                Edge::Bottom => (accent.loc.y, band.loc.y),
                Edge::Top => (accent.loc.y + accent.size.h, band.loc.y + band.size.h),
                Edge::Left => (accent.loc.x + accent.size.w, band.loc.x + band.size.w),
                Edge::Right => (accent.loc.x, band.loc.x),
            };
            assert_eq!(
                accent_edge, band_inside,
                "{edge:?}: the accent is not on the face that looks at the canvas"
            );
        }
    }

    /// **And the drawn picture's accent is that one**, which is the claim the two
    /// tests above cannot make.
    ///
    /// They ask `the_accent_on` directly, so they say *this helper is right*.
    /// Mutation-tested on 2026-10-10 by putting the original fault back in
    /// `picture` — `Rectangle::new(band.loc, (bar_width, rule).into())` — and
    /// **all fourteen tests passed**, the two new ones included. A correct helper
    /// nothing calls is the fault class `CLAUDE.md` names first: *a check that
    /// stands in for the thing is not the thing.*
    ///
    /// So this one draws, on all four edges, and asks the picture.
    #[test]
    fn the_drawn_accent_is_the_one_the_edge_asks_for() {
        use alo_dock::Edge;
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let held = on_the_dock(4);
        let size = (1920, 1080);

        for edge in Edge::EVERY {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);
            let drawn = picture(&dock, look, size, &held, &mut fonts()).unwrap();

            assert_eq!(
                drawn.accent,
                the_accent_on(
                    edge,
                    drawn.band,
                    drawn.accent.size.w.min(drawn.accent.size.h)
                ),
                "{edge:?}: the picture's accent is not the one this edge asks for"
            );
            assert!(
                inside(drawn.accent, drawn.band),
                "{edge:?}: the drawn accent at {:?} leaves the band at {:?} and draws on the \
                 canvas — which is what it did on the left edge until 2026-10-10, a line 176 \
                 long beside a bar 70 wide",
                drawn.accent,
                drawn.band
            );
            // And it is painted, not merely computed: the solid a person sees.
            assert!(
                drawn.solids.iter().any(|solid| solid.area == drawn.accent),
                "{edge:?}: nothing is painted at the accent's place"
            );
        }
    }

    /// **No two edges put the accent in the same place either.**
    ///
    /// The companion to `the_four_edges_are_four_different_places`, and for the
    /// same reason: four arms that each satisfy the checks above could be two arms
    /// written twice, and a copied arm with its edge not changed is how that
    /// happens.
    #[test]
    fn the_four_edges_are_four_different_accents() {
        use alo_dock::Edge;
        let mut seen: Vec<Rectangle<i32, Physical>> = Vec::new();
        for edge in Edge::EVERY {
            let band = the_band_on(edge, (1920, 1080), 600, 70, 8);
            let accent = the_accent_on(edge, band, 2);
            assert!(
                !seen.contains(&accent),
                "{edge:?} puts the accent exactly where an earlier edge does: {accent:?}"
            );
            seen.push(accent);
        }
        assert_eq!(seen.len(), 4);
    }

    /// **No two edges put the bar in the same place.**
    ///
    /// Four placements that each pass the checks above could still be two
    /// placements written twice — a copied arm with its edge not changed is the
    /// likeliest way this goes wrong, and every assertion above would hold.
    #[test]
    fn the_four_edges_are_four_different_places() {
        use alo_dock::Edge;
        let mut seen: Vec<Rectangle<i32, Physical>> = Vec::new();
        for edge in Edge::EVERY {
            let band = the_band_on(edge, (1920, 1080), 600, 70, 8);
            assert!(
                !seen.contains(&band),
                "{edge:?} lands exactly where an earlier edge does: {band:?}"
            );
            seen.push(band);
        }
        assert_eq!(seen.len(), 4);
    }

    /// **The dock is a bar: centred, clear of the bottom edge, and as wide as
    /// what it holds** — the same bar whichever way the person reads.
    #[test]
    fn the_dock_is_a_centred_bar_clear_of_the_bottom_edge() {
        let size = (1920, 1080);
        let dock = Dock::shipped();
        let mut whichever_way_read: Option<Rectangle<i32, Physical>> = None;
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let look = noon_look(&an_appearance(), reading);
            let drawn = picture(&dock, look, size, &on_the_dock(4), &mut fonts()).unwrap();
            let layout = dock.layout_on(Screen::of(1920, 1080).unwrap());
            assert_eq!(drawn.layout, layout);

            let thick = i32::try_from(layout.thickness().as_pixels()).unwrap();
            let floating = i32::try_from(FLOATING_ABOVE_THE_EDGE).unwrap();

            // Clear of the edge, with room underneath it.
            assert_eq!(
                drawn.band.loc.y + drawn.band.size.h,
                1080 - floating,
                "{reading:?}: it is not floating"
            );
            assert_eq!(drawn.band.size.h, thick, "{reading:?}");

            // As wide as what it holds, and nothing like the whole screen.
            let wanted = i32::try_from(Room::a_bar_holding(4).as_pixels()).unwrap();
            assert_eq!(drawn.band.size.w, wanted, "{reading:?}");
            assert!(drawn.band.size.w < 1920, "{reading:?}: it spans the screen");

            // Centred: the room either side is equal to within a pixel.
            let left = drawn.band.loc.x;
            let right = 1920 - (drawn.band.loc.x + drawn.band.size.w);
            assert!((left - right).abs() <= 1, "{reading:?}: {left} vs {right}");

            assert!(inside(drawn.accent, drawn.band), "{reading:?}");
            assert_eq!(drawn.accent.loc.y, drawn.band.loc.y, "the inside edge");

            match whichever_way_read {
                None => whichever_way_read = Some(drawn.band),
                Some(first) => {
                    assert_eq!(first, drawn.band, "which way a person reads moved the dock");
                }
            }
        }
        assert!(whichever_way_read.is_some(), "neither way round was drawn");
    }

    /// **A bar holding more is wider**, which is what says the width comes from
    /// the contents rather than from the screen.
    #[test]
    fn a_bar_holding_more_is_wider() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let dock = Dock::shipped();
        let mut last = 0;
        for holding in [0_usize, 1, 4, 9] {
            let drawn = picture(
                &dock,
                look,
                (1920, 1080),
                &on_the_dock(holding),
                &mut fonts(),
            )
            .unwrap();
            assert!(
                drawn.band.size.w > last,
                "holding {holding} was not wider than the one before"
            );
            last = drawn.band.size.w;
        }
    }

    /// **A bar too wide for the screen is clamped rather than drawn off it**,
    /// and what did not fit went into the overflow before it reached this file.
    #[test]
    fn a_bar_wider_than_the_screen_is_clamped_to_it() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let drawn = picture(
            &Dock::shipped(),
            look,
            (1366, 768),
            &on_the_dock(200),
            &mut fonts(),
        )
        .unwrap();
        let margin = i32::try_from(MARGIN).unwrap();

        assert!(drawn.band.size.w <= 1366 - 2 * margin);
        assert!(drawn.band.loc.x >= 0);
        assert!(drawn.band.loc.x + drawn.band.size.w <= 1366);
    }

    /// **Per display.** The same dock on a laptop and on a large screen beside
    /// it is laid out for each display's own size, and each band is as thick as
    /// `alo-dock` says for it.
    ///
    /// **The band is the same thickness on both, and that is the point now
    /// rather than a weakening.** This asserted that at 300% text the names
    /// gave way on the laptop and were kept on the desk, so the two bands were
    /// different heights. The owner removed the row of names on 2026-10-10 —
    /// the verified design has none — so the bar is the measured 76 on every
    /// screen at every text size, and what differs between displays is its
    /// **length** and where it sits.
    #[test]
    fn each_display_gets_the_dock_laid_out_for_its_own_size() {
        let mut appearance = an_appearance();
        appearance.set_text(TextScale::percent(300).unwrap());
        let look = noon_look(&appearance, Direction::LeftToRight);
        let dock = Dock::shipped();

        let laptop = picture(&dock, look, (1366, 768), &on_the_dock(4), &mut fonts()).unwrap();
        let desk = picture(&dock, look, (3840, 2160), &on_the_dock(4), &mut fonts()).unwrap();
        let portrait = picture(&dock, look, (1080, 1920), &on_the_dock(4), &mut fonts()).unwrap();
        assert!(laptop.layout.labels().are_shown());
        assert!(desk.layout.labels().are_shown());
        assert_eq!(
            laptop.band.size.h, desk.band.size.h,
            "a 300% text size moved one band and not the other, so something still reads the \
             text to decide a thickness"
        );
        assert_ne!(
            laptop.band.loc.y, desk.band.loc.y,
            "the premise: these are different displays, so the bands sit at different heights \
             even though they are equally thick"
        );
        for drawn in [&laptop, &desk, &portrait] {
            assert_eq!(
                i64::from(drawn.band.size.h),
                i64::from(drawn.layout.thickness().as_pixels())
            );
            assert_eq!(
                drawn.band.loc.y + drawn.band.size.h,
                drawn.size.1 - i32::try_from(FLOATING_ABOVE_THE_EDGE).unwrap(),
                "it is clear of the bottom edge"
            );
        }
    }

    /// **The accent is `alo-appearance`'s**: the rule along the band's inside
    /// edge is exactly the accent the person chose, for the scheme on screen,
    /// and the band is never deep teal.
    #[test]
    fn the_dock_carries_the_persons_accent() {
        for accent in Accent::ALL {
            let mut appearance = an_appearance();
            appearance.set_accent(accent);
            let look = noon_look(&appearance, Direction::LeftToRight);
            let drawn = picture(
                &Dock::shipped(),
                look,
                (1920, 1080),
                &on_the_dock(4),
                &mut fonts(),
            )
            .unwrap();
            let rule = drawn
                .solids
                .iter()
                .find(|solid| solid.area == drawn.accent)
                .unwrap();
            assert_eq!(
                rule.colour,
                crate::desktop_look::rgb(accent.on(look.scheme()))
            );
        }
    }

    /// **A display `alo-dock` refuses is refused**, and so is one larger than any
    /// this dock is laid out for, rather than a dock drawn over somebody's whole
    /// screen.
    #[test]
    fn a_display_too_small_or_too_large_for_a_dock_is_refused() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        for size in [(0, 0), (100, 100), (-5, 800), (20_000, 1080)] {
            assert!(
                matches!(
                    picture(&Dock::shipped(), look, size, &on_the_dock(4), &mut fonts()),
                    Err(RenderError::DesktopScene)
                ),
                "{size:?}"
            );
        }
    }
    /// **One letter per application, inside the band.**
    ///
    /// The Dock drew a coloured band and nothing else until 2026-10-10 — no
    /// file in this crate asked `alo_dock::places` where an icon goes, and
    /// `alo-applications` names an icon in none of its seventeen files, so
    /// there was no artwork to put in one either. The owner's ruling gave every
    /// application a letter, and this is the test that it reaches the picture.
    #[test]
    fn every_application_on_the_dock_gets_a_letter_inside_the_band() {
        let dock = Dock::shipped();
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let held = on_the_dock(3);
        let drawn = picture(&dock, look, (1920, 1080), &held, &mut fonts()).unwrap();

        assert_eq!(
            drawn.inked.len(),
            held.len(),
            "three applications on the Dock and {} letters drawn",
            drawn.inked.len()
        );
        for letter in &drawn.inked {
            assert!(
                !letter.pixels.is_empty(),
                "a letter with no pixels is a letter nobody sees"
            );
            let inside = letter.area.loc.x >= drawn.band.loc.x
                && letter.area.loc.y >= drawn.band.loc.y
                && letter.area.loc.x + letter.area.size.w <= drawn.band.loc.x + drawn.band.size.w
                && letter.area.loc.y + letter.area.size.h <= drawn.band.loc.y + drawn.band.size.h;
            assert!(
                inside,
                "a letter at {:?} falls outside the band at {:?}, so it draws on the canvas",
                letter.area, drawn.band
            );
        }
    }

    /// **Every letter sits on the bar's centre line, on all four edges.**
    ///
    /// The icon's box was inset across the bar by `MARGIN` — 8 — until
    /// 2026-10-10. The bar is the measured 76 thick with a 48 icon in it, so the
    /// right inset is 14 and every letter on every machine sat **six logical
    /// pixels high**. The owner's ruling says so outright: *the measured 48px
    /// application target centred vertically: 14px above and below.*
    ///
    /// # Why the test that existed could not see it
    ///
    /// `every_application_on_the_dock_gets_a_letter_inside_the_band` asks that a
    /// letter is **inside** the band, and a letter six pixels high is inside it.
    /// *Inside* is the weakest claim a placement can make and it is the one that
    /// is easy to write; where a thing **is** takes a second number to compare
    /// against, and the second number here is the band's own centre.
    ///
    /// # Asserted as centred rather than as 14
    ///
    /// Fourteen is the bottom and top edges' answer; a dock down a side is the
    /// measured 70 wide and its answer is 11. Writing 14 would pass on two edges
    /// and fail on two, and writing both would be two constants where the design
    /// has one rule. So this asserts the rule — *centred across the thickness* —
    /// and then checks that the rule and `MARGIN` really do disagree, because a
    /// bar whose thickness happened to be `MARGIN + ICON + MARGIN` would make
    /// the fault invisible again. It was, until the bar was measured.
    #[test]
    fn a_letter_sits_on_the_bars_centre_line_on_every_edge() {
        use alo_dock::Edge;
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let held = on_the_dock(3);
        let size = (1920, 1080);

        for edge in Edge::EVERY {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);
            let drawn = picture(&dock, look, size, &held, &mut fonts()).unwrap();
            assert_eq!(drawn.inked.len(), held.len(), "{edge:?}");

            // The band's centre across its thickness, and each letter's.
            let thickness = i32::try_from(drawn.layout.thickness().as_pixels()).unwrap();
            let inset = (thickness - i32::try_from(ICON).unwrap()) / 2;
            // **The premise**: the rule and `MARGIN` give different answers on
            // this edge, so a draw still reading `MARGIN` would fail below. If
            // they ever agree this assertion says so rather than passing
            // vacuously.
            assert_ne!(
                inset,
                i32::try_from(MARGIN).unwrap(),
                "{edge:?} is {thickness} thick, where centring an icon and insetting it by                  MARGIN give the same answer — so this test cannot tell the two apart and the                  fault it was written for would be invisible"
            );

            for letter in &drawn.inked {
                let (band_middle, letter_middle) = if edge.runs_across() {
                    (
                        drawn.band.loc.y + drawn.band.size.h / 2,
                        letter.area.loc.y + letter.area.size.h / 2,
                    )
                } else {
                    (
                        drawn.band.loc.x + drawn.band.size.w / 2,
                        letter.area.loc.x + letter.area.size.w / 2,
                    )
                };
                assert!(
                    (band_middle - letter_middle).abs() <= 1,
                    "on the {edge:?} edge a letter's middle is {letter_middle} and the bar's is                      {band_middle}, {} away. The icon is inset {inset} across a bar {thickness}                      thick; MARGIN is {MARGIN} and was what this read until 2026-10-10",
                    (band_middle - letter_middle).abs()
                );
            }
        }
    }

    /// **No application, no letter**, which is what every machine showed before
    /// the ruling and what a fresh one still shows.
    #[test]
    fn a_dock_holding_nothing_draws_no_letters() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let drawn = picture(&Dock::shipped(), look, (1920, 1080), &[], &mut fonts()).unwrap();
        assert!(drawn.inked.is_empty());
        assert!(
            !drawn.solids.is_empty(),
            "the premise: the band is still drawn, so an empty `inked` is about letters and \
             not about nothing being drawn at all"
        );
    }

    /// **The letters are a person's applications and not one letter repeated.**
    ///
    /// `on_the_dock` names its applications `app-0`, `app-1`, `app-2`, so their
    /// first letters are all `a` — which is exactly the case that would let a
    /// draw ignoring `APlace::app` pass the test above. So this one asks that
    /// three *different* applications produce three *different* inks.
    #[test]
    fn each_icon_shows_its_own_applications_letter() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let patch =
            alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("one by one is a patch");
        let mut windows = alo_dock::Windows::none();
        for (number, named) in ["alpha", "beta", "gamma"].into_iter().enumerate() {
            windows.opened(alo_dock::Window::of(
                alo_dock::WindowId::numbered(number as u64),
                Some(alo_dock::AppId::named(named).expect("a named application")),
                "",
                patch,
                alo_dock::HowItSits::OnTheCanvas,
            ));
        }
        let held = alo_dock::Holding::nothing().showing(&windows);
        let drawn = picture(&Dock::shipped(), look, (1920, 1080), &held, &mut fonts()).unwrap();

        assert_eq!(drawn.inked.len(), 3);
        // **No indexing**, which this workspace denies: consecutive pairs
        // instead, which is the same claim and says which pair differed.
        let inks: Vec<&Vec<[u8; 3]>> = drawn.inked.iter().map(|it| &it.pixels).collect();
        for (which, pair) in inks.windows(2).enumerate() {
            let [before, after] = pair else {
                unreachable!("windows(2) yields pairs")
            };
            assert_ne!(
                before,
                after,
                "letters {which} and {} inked identically, so the draw is not reading each \
                 place's own application",
                which + 1
            );
        }
    }
}
