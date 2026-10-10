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

use alo_dock::measures::{A_UTILITY_GLYPH, GLYPH, ICON};
use alo_dock::{Dock, Layout, OnTheDock};

use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;
use crate::desktop_look::DesktopLook;
use crate::painted::{Inked, Solid};
use crate::where_the_dock_is::TheDocksPlaces;

/// What the overflow control shows: a horizontal ellipsis.
///
/// **A mark, not a word, and so not `alo_dock::words`.** The control's *name* is
/// a sentence a person reads — `alo_dock::words::SHOW_MORE_OPEN_APPS`, which a
/// screen reader speaks and a tooltip shows — and this is the thing drawn in its
/// 48 slot. U+2026 means *there is more here* in every language this product
/// ships in, which a translated string could not: a letter or an abbreviation
/// would have to be translated and would then be a different width in every
/// locale, inside a slot that is one size everywhere.
///
/// It is drawn through the same text path as an application's first letter,
/// because that is what this file already has and a second path would be a
/// second place for a mark to be mispositioned.
///
/// # A `char`, and written as a code point, for two guards rather than one
///
/// `tests/desktop_source.rs` holds that no file in this path *words a
/// sentence*: a line here may hold no `"` at all, because every sentence a
/// person meets comes from `alo-saying` and a literal is how that stops being
/// true. This is not an exception to that rule — the control's **name** is
/// `alo_dock::words::SHOW_MORE_OPEN_APPS`, in the vocabulary, translated, and
/// it is what a screen reader speaks and a tooltip shows. What is left here is
/// one character of artwork, and a `char` is the type that says so.
///
/// Written `'\u{2026}'` rather than pasted, so that a reader sees which code
/// point it is rather than three dots that might be a period repeated — which
/// is a different character, a different width, and the thing a screen reader
/// would read aloud as three full stops.
const THE_OVERFLOWS_MARK: char = '\u{2026}';

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
    /// What there was no room for, in order, behind the overflow control.
    ///
    /// **Empty is the ordinary case** and means there is no control on the bar
    /// either: `alo_dock::fit` puts nothing aside until the edge runs out, and
    /// `alo_dock::Places` draws no control for an empty overflow.
    pub(crate) over: Vec<OnTheDock>,
    /// Where this draw put the Dock, for the press that will ask later.
    ///
    /// **The same value the three fields above were taken from**, destructured
    /// in one expression and never written again — so this is one answer read
    /// two ways rather than a second home that can drift from the first. The
    /// draw hands it to `Server::the_docks_slots_were_drawn`, which is the one
    /// place a press can find out where the slots are: outside a frame they do
    /// not exist.
    pub(crate) standing: TheDocksPlaces,
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

/// What the overflow list needs drawing: everything but the pixels.
///
/// A struct rather than eight arguments, because every one of them is a
/// different type of number and a caller that swapped two would compile.
struct TheOverflowIsDrawn<'a> {
    /// The applications the bar had no room for, in order.
    over: &'a [OnTheDock],
    /// The Dock's band, which the list opens inward from.
    at: Rectangle<i32, Physical>,
    /// Which edge the Dock is on, so *inward* has a direction.
    edge: alo_dock::Edge,
    /// The display, so the list can be kept on it.
    room: (i32, i32),
    /// The person's text size, which the rows grow with.
    text: alo_appearance::TextScale,
    /// What the heading says, already said: this file may not word a sentence.
    heading: &'a str,
    /// The person's colours.
    palette: crate::desktop_look::DesktopPalette,
}

/// Draw the list the overflow control opens, into `solids` and `inked`.
///
/// **It opens inward from the bar**, which is the design file's own name for the
/// frame: `More apps / opens inward`, 248 wide beside a left dock, a right dock,
/// a top dock and the bottom one. So the width is a measurement and the side it
/// opens toward follows the Dock's edge.
///
/// **How tall it is comes from `alo_dock::overflow`**, not from here. That is the
/// owner's ruling of 2026-10-10 — rows at a 44 minimum, growing with the text,
/// the panel growing past the drawn 313 and then scrolling — and this file's job
/// is to put it where the bar is and ink the names into it.
fn the_overflow_list(
    solids: &mut Vec<Solid>,
    inked: &mut Vec<Inked>,
    drawn: TheOverflowIsDrawn<'_>,
    fonts: &mut FontSystem,
) {
    let TheOverflowIsDrawn {
        over,
        at,
        edge,
        room,
        text,
        heading,
        palette,
    } = drawn;
    let (width, height) = room;
    let beside = i32::try_from(alo_dock::measures::BESIDE_A_NAME_IN_THE_OVERFLOW).unwrap_or(0);
    let around = i32::try_from(alo_dock::measures::AROUND_THE_OVERFLOWS_HEADING).unwrap_or(0);
    let gap = i32::try_from(alo_dock::measures::GAP).unwrap_or(0);

    // **How much room there is between the bar and the far side of the screen**,
    // which is what the list may grow into before it has to scroll.
    let headroom = match edge {
        alo_dock::Edge::Bottom => at.loc.y - gap,
        alo_dock::Edge::Top => height - (at.loc.y + at.size.h) - gap,
        alo_dock::Edge::Left | alo_dock::Edge::Right => height - 2 * gap,
    }
    .max(1);

    let panel = alo_dock::overflow::ThePanel::of(
        over.len(),
        text,
        alo_dock::Room::pixels(u32::try_from(headroom).unwrap_or(0)),
    );
    let across = i32::try_from(panel.width().as_pixels()).unwrap_or(0);
    let down = i32::try_from(panel.height().as_pixels()).unwrap_or(0);

    // **Inward from the bar, and kept on the screen.** The list hangs off the
    // control's end of the bar on an edge that runs across, and beside the bar on
    // one that runs down — then is pulled back onto the display, because a list
    // half off the screen is a list a person cannot read.
    let origin = match edge {
        alo_dock::Edge::Bottom => (at.loc.x + at.size.w - across, at.loc.y - gap - down),
        alo_dock::Edge::Top => (at.loc.x + at.size.w - across, at.loc.y + at.size.h + gap),
        alo_dock::Edge::Left => (at.loc.x + at.size.w + gap, at.loc.y + at.size.h - down),
        alo_dock::Edge::Right => (at.loc.x - gap - across, at.loc.y + at.size.h - down),
    };
    let left = origin.0.clamp(0, (width - across).max(0));
    let top = origin.1.clamp(0, (height - down).max(0));

    solids.push(Solid {
        area: Rectangle::new((left, top).into(), (across, down).into()),
        colour: palette.dock,
    });

    let line = i32::try_from(alo_dock::Room::a_line_at(text).as_pixels()).unwrap_or(1);
    let size = i32::try_from(alo_dock::Room::text_at(text).as_pixels()).unwrap_or(1);
    let metrics = Metrics::new(size as f32, line as f32);

    // The heading, then the rule under it.
    let said = crate::painted_text::sentence(
        fonts,
        heading,
        across - 2 * beside,
        metrics,
        palette.dock,
        palette.ink,
    );
    if let Some(drawn) = said.placed(left + beside, top + around, height) {
        inked.push(drawn);
    }
    solids.push(Solid {
        area: Rectangle::new(
            (left + beside, top + around + line + around).into(),
            (
                across - 2 * beside,
                i32::try_from(alo_dock::measures::A_DIVIDER).unwrap_or(1),
            )
                .into(),
        ),
        colour: palette.accent,
    });

    // One row per application that is on the screen, its name inked into it.
    let (first, showing) = panel.showing();
    for which in first..first.saturating_add(showing) {
        let Some(application) = over.get(which) else {
            break;
        };
        let Some(sits) = panel.where_a_row_sits(which) else {
            continue;
        };
        let row = top + i32::try_from(sits.as_pixels()).unwrap_or(0);
        let a_row = i32::try_from(panel.a_row().as_pixels()).unwrap_or(1);
        let name = crate::painted_text::sentence(
            fonts,
            application.app().name(),
            across - 2 * beside,
            metrics,
            palette.dock,
            palette.ink,
        );
        // Centred down its row, which is taller than the text whenever the floor
        // is what decided it.
        let into = row + ((a_row - name.height) / 2).max(0);
        if let Some(drawn) = name.placed(left + beside, into, height) {
            inked.push(drawn);
        }
    }
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
    the_overflow: (bool, &str),
    fonts: &mut FontSystem,
) -> Result<DockPicture, RenderError> {
    let (width, height) = size;
    // **Where the Dock is, asked rather than worked out here.** This file
    // computed the band, the slots and the overflow itself and kept none of it,
    // so a press had no way to ask what was under the pointer without
    // rasterising a frame. `crate::where_the_dock_is` is the one answer both
    // read; neither can be right while the other is wrong.
    let standing = crate::where_the_dock_is::where_the_dock_is(dock, size, on_the_dock)?;
    let TheDocksPlaces {
        layout,
        band,
        thickness,
        ref places,
        ref over,
    } = standing;
    let (over, places) = (over.clone(), places.clone());
    let palette = look.palette().map_err(|_| RenderError::AccentRefused)?;
    let measure = look.measure();
    let rule = measure.px(2).min(thickness);

    let accent = the_accent_on(layout.edge(), band, rule);

    let mut solids = vec![
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
    let icon = i32::try_from(ICON).unwrap_or(i32::MAX);
    let glyph = i32::try_from(GLYPH).unwrap_or(i32::MAX);
    let utility = i32::try_from(A_UTILITY_GLYPH).unwrap_or(i32::MAX);
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
    let mut inked: Vec<Inked> = Vec::new();
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
        // **An application's letter, or the control's mark.** The slot is the
        // same size either way — the owner's ruling of 2026-10-10 makes the
        // overflow one application slot — and what is drawn in it is not: a
        // letter would read as an application a person could open, and there is
        // no application called *more*.
        //
        // `A_UTILITY_GLYPH` rather than `GLYPH`, which is the owner's own
        // distinction: *utility glyphs — search, overflow — 20 to 24, in the
        // same 48 target.*
        let mut one_character = [0_u8; 4];
        let (mark, size) = match place.app() {
            Some(app) => (app.first_letter(), glyph),
            None => (
                &*THE_OVERFLOWS_MARK.encode_utf8(&mut one_character),
                utility,
            ),
        };
        let shaped = crate::painted_text::centred(
            fonts,
            mark,
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
            Metrics::new(size as f32, size as f32),
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

    // **The list the control opens, when a person has opened it.**
    //
    // Drawn last so it sits over the bar and over the canvas, which is what a
    // list opened from a control has to do. `alo_dock::overflow::ThePanel` is
    // the layout — rows at the floor every control is held to, growing with the
    // text, scrolling when the room runs out — and this is the only thing that
    // turns it into pixels.
    let (the_overflow_is_open, heading) = the_overflow;
    if the_overflow_is_open && !over.is_empty() {
        the_overflow_list(
            &mut solids,
            &mut inked,
            TheOverflowIsDrawn {
                over: &over,
                at: band,
                edge: layout.edge(),
                room: (width, height),
                text: look.scale(),
                heading,
                palette,
            },
            fonts,
        );
    }

    Ok(DockPicture {
        size,
        layout,
        band,
        accent,
        solids,
        inked,
        over,
        standing,
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
    use crate::where_the_dock_is::the_band_on;

    /// A Dock whose overflow list a person has not opened, which is every frame
    /// but the one after they press the control.
    const fn closed() -> (bool, &'static str) {
        (false, "")
    }
    use alo_appearance::{Accent, TextScale};
    use alo_dock::measures::{FLOATING_ABOVE_THE_EDGE, MARGIN};
    use alo_dock::{Room, Screen};
    use alo_strings::Direction;

    /// Whether `inner` lies wholly inside `outer`.
    fn inside(inner: Rectangle<i32, Physical>, outer: Rectangle<i32, Physical>) -> bool {
        outer.intersection(inner) == Some(inner)
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
            let drawn = picture(&dock, look, size, &held, closed(), &mut fonts()).unwrap();

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

    /// **The dock is a bar: centred, clear of the bottom edge, and as wide as
    /// what it holds** — the same bar whichever way the person reads.
    #[test]
    fn the_dock_is_a_centred_bar_clear_of_the_bottom_edge() {
        let size = (1920, 1080);
        let dock = Dock::shipped();
        let mut whichever_way_read: Option<Rectangle<i32, Physical>> = None;
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let look = noon_look(&an_appearance(), reading);
            let drawn =
                picture(&dock, look, size, &on_the_dock(4), closed(), &mut fonts()).unwrap();
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
                closed(),
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

    /// **Two hundred applications stay on the screen, and every one of them
    /// stays reachable.**
    ///
    /// This was `a_bar_wider_than_the_screen_is_clamped_to_it`, and its doc
    /// comment said *what did not fit went into the overflow before it reached
    /// this file*. Nothing did: `alo_dock::fit` had no caller, so two hundred
    /// applications produced a bar 11 208 pixels wide that was **clamped** to
    /// the screen — and the clamp is a rectangle, so a hundred and seventy-odd
    /// icons were drawn past its end, on the canvas or off the display
    /// altogether, with no way to reach them.
    ///
    /// The clamp is still there and is now what the header claims it is: a last
    /// resort that cannot fire, because the slot count comes from the same
    /// `widest` the clamp would use.
    ///
    /// Four claims, and the fourth is the one that matters to a person: the bar
    /// is on the screen, it was **not** clamped, every slot drawn is inside it,
    /// and **nothing was dropped** — what is not on the bar is in `over`, in
    /// order, behind the control.
    #[test]
    fn two_hundred_applications_fit_the_screen_with_none_of_them_lost() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let held = on_the_dock(200);
        let drawn = picture(
            &Dock::shipped(),
            look,
            (1366, 768),
            &held,
            closed(),
            &mut fonts(),
        )
        .unwrap();
        let margin = i32::try_from(MARGIN).unwrap();
        let widest = 1366 - 2 * margin;

        assert!(drawn.band.size.w <= widest);
        assert!(drawn.band.loc.x >= 0);
        assert!(drawn.band.loc.x + drawn.band.size.w <= 1366);

        // Not clamped: the bar is exactly as wide as what it holds.
        let slots = drawn.inked.len();
        assert_eq!(
            drawn.band.size.w,
            i32::try_from(Room::a_bar_holding(slots).as_pixels()).unwrap(),
            "the bar was clamped, so it is not as wide as the {slots} slots it drew"
        );

        // Every letter inside the band, which the clamp never guaranteed.
        for letter in &drawn.inked {
            assert!(
                letter.area.loc.x + letter.area.size.w <= drawn.band.loc.x + drawn.band.size.w,
                "a slot is drawn past the end of the bar at {:?}",
                letter.area
            );
        }

        // **Nothing lost.** The slots are the applications on the bar plus the
        // one control, and the rest are behind it.
        assert!(!drawn.over.is_empty(), "nothing went into the overflow");
        assert_eq!(
            slots - 1 + drawn.over.len(),
            held.len(),
            "{} on the bar, {} over, and {} were held",
            slots - 1,
            drawn.over.len(),
            held.len()
        );
    }

    /// **The overflow works on all four edges**, and each edge's answer is its
    /// own.
    ///
    /// The owner's ruling of 2026-10-10 asks for the control to *use the same
    /// slot spacing as neighbouring applications on all four edges*, and the
    /// edges genuinely differ: a bar along the bottom of a 1366 × 768 display
    /// runs 1366 and one down the side runs 768, so a side dock overflows
    /// sooner and holds fewer. A test on one edge would say nothing about that.
    ///
    /// **Three claims per edge and one across them.** Per edge: the bar stays
    /// within the edge it runs along, something went into the overflow, and
    /// nothing was lost. Across them: the two orientations hold **different**
    /// numbers, so this cannot pass by the length being taken off the same axis
    /// four times — which is exactly the fault the accent had.
    #[test]
    fn the_overflow_holds_what_will_not_fit_on_every_edge() {
        use alo_dock::Edge;
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let held = on_the_dock(200);
        let size = (1366, 768);
        let margin = i32::try_from(MARGIN).unwrap();
        let mut on_the_bar: Vec<(Edge, usize)> = Vec::new();

        for edge in Edge::EVERY {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);
            let drawn = picture(&dock, look, size, &held, closed(), &mut fonts()).unwrap();

            // Within the edge it runs along, less the room either side.
            let (ran, edge_length) = if edge.runs_across() {
                (drawn.band.size.w, size.0)
            } else {
                (drawn.band.size.h, size.1)
            };
            assert!(
                ran <= edge_length - 2 * margin,
                "{edge:?}: the bar runs {ran} along an edge {edge_length} long"
            );
            assert!(
                drawn.band.loc.x >= 0
                    && drawn.band.loc.y >= 0
                    && drawn.band.loc.x + drawn.band.size.w <= size.0
                    && drawn.band.loc.y + drawn.band.size.h <= size.1,
                "{edge:?}: the bar leaves the screen at {:?}",
                drawn.band
            );

            let slots = drawn.inked.len();
            assert!(
                !drawn.over.is_empty(),
                "{edge:?}: two hundred applications and nothing in the overflow"
            );
            assert_eq!(
                slots - 1 + drawn.over.len(),
                held.len(),
                "{edge:?}: {} on the bar and {} over, of {}",
                slots - 1,
                drawn.over.len(),
                held.len()
            );
            on_the_bar.push((edge, slots));
        }

        // **The two orientations hold different numbers.** 1366 along the
        // bottom and 768 down a side is not the same bar, and a length taken
        // off the wrong axis would make all four agree.
        let across: Vec<usize> = on_the_bar
            .iter()
            .filter(|(edge, _)| edge.runs_across())
            .map(|(_, slots)| *slots)
            .collect();
        let down: Vec<usize> = on_the_bar
            .iter()
            .filter(|(edge, _)| !edge.runs_across())
            .map(|(_, slots)| *slots)
            .collect();
        assert_eq!(across.len(), 2, "the premise");
        assert_eq!(down.len(), 2, "the premise");
        assert!(
            across.iter().all(|holds| Some(holds) == across.first()),
            "the bottom and top edges of one display hold different numbers: {on_the_bar:?}"
        );
        assert!(
            down.iter().all(|holds| Some(holds) == down.first()),
            "the left and right edges of one display hold different numbers: {on_the_bar:?}"
        );
        assert!(
            across.first() > down.first(),
            "a bar along the 1366 edge does not hold more than one down the 768 edge, so the \
             length is being taken off the same axis for both: {on_the_bar:?}"
        );
    }

    /// **The list is drawn when a person opens it, and not before.**
    ///
    /// Three frames of one Dock, differing only in the thing a press turns over:
    /// closed draws the bar alone, open draws the bar and the list, and open on
    /// a Dock with nothing behind the control draws the bar alone again.
    ///
    /// The third is the one worth having. `the_overflow_is_open` lives on the
    /// `Server` and is not cleared when the Dock stops overflowing — a person
    /// closes a window and the bar fits again — so a draw that trusted the flag
    /// alone would paint an empty list over the canvas.
    #[test]
    fn the_list_is_drawn_only_when_it_is_open_and_has_something_in_it() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let crowded = on_the_dock(200);
        let open = (true, "More open apps");

        let shut = picture(
            &Dock::shipped(),
            look,
            (1366, 768),
            &crowded,
            closed(),
            &mut fonts(),
        )
        .unwrap();
        let opened = picture(
            &Dock::shipped(),
            look,
            (1366, 768),
            &crowded,
            open,
            &mut fonts(),
        )
        .unwrap();

        assert!(!shut.over.is_empty(), "the premise: something overflowed");
        assert!(
            opened.solids.len() > shut.solids.len(),
            "opening the list drew no more shapes: {} against {}",
            opened.solids.len(),
            shut.solids.len()
        );
        assert!(
            opened.inked.len() > shut.inked.len(),
            "opening the list inked no more text: {} against {}",
            opened.inked.len(),
            shut.inked.len()
        );

        // Open, on a Dock with room for everything: the flag says open and
        // there is nothing to show.
        let roomy = picture(
            &Dock::shipped(),
            look,
            (1920, 1080),
            &on_the_dock(4),
            open,
            &mut fonts(),
        )
        .unwrap();
        let roomy_shut = picture(
            &Dock::shipped(),
            look,
            (1920, 1080),
            &on_the_dock(4),
            closed(),
            &mut fonts(),
        )
        .unwrap();
        assert!(roomy.over.is_empty(), "the premise");
        assert_eq!(
            (roomy.solids.len(), roomy.inked.len()),
            (roomy_shut.solids.len(), roomy_shut.inked.len()),
            "an empty list was drawn over the canvas"
        );
    }

    /// **The list stays on the screen, on every edge.**
    ///
    /// It opens inward from the bar, and the bar is already near an edge — so
    /// the direction *inward* is the one thing that differs between the four,
    /// and a list that opened the wrong way would be half off the display. Every
    /// shape and every letter is checked, because a panel inside the screen with
    /// its rows outside it is the same bug one layer down.
    #[test]
    fn the_open_list_stays_on_the_screen_on_every_edge() {
        use alo_dock::Edge;
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let crowded = on_the_dock(200);
        let size = (1366, 768);

        for edge in Edge::EVERY {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);
            let drawn = picture(
                &dock,
                look,
                size,
                &crowded,
                (true, "More open apps"),
                &mut fonts(),
            )
            .unwrap();

            for solid in &drawn.solids {
                assert!(
                    solid.area.loc.x >= 0
                        && solid.area.loc.y >= 0
                        && solid.area.loc.x + solid.area.size.w <= size.0
                        && solid.area.loc.y + solid.area.size.h <= size.1,
                    "{edge:?}: a shape at {:?} is off a {size:?} screen",
                    solid.area
                );
            }
            for letter in &drawn.inked {
                assert!(
                    letter.area.loc.x >= 0
                        && letter.area.loc.y >= 0
                        && letter.area.loc.x + letter.area.size.w <= size.0
                        && letter.area.loc.y + letter.area.size.h <= size.1,
                    "{edge:?}: a letter at {:?} is off a {size:?} screen",
                    letter.area
                );
            }
        }
    }

    /// **A bar with room for everything has no control and nothing over.**
    ///
    /// The companion to the test above, and the premise that keeps it honest: if
    /// `fit` put something aside at four applications on a 1920 screen, that test
    /// would pass for the wrong reason.
    #[test]
    fn a_dock_that_fits_puts_nothing_aside() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let held = on_the_dock(4);
        let drawn = picture(
            &Dock::shipped(),
            look,
            (1920, 1080),
            &held,
            closed(),
            &mut fonts(),
        )
        .unwrap();
        assert!(
            drawn.over.is_empty(),
            "a Dock of four overflowed on a 1920 screen"
        );
        assert_eq!(
            drawn.inked.len(),
            held.len(),
            "a control was drawn for an empty overflow"
        );
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

        let laptop = picture(
            &dock,
            look,
            (1366, 768),
            &on_the_dock(4),
            closed(),
            &mut fonts(),
        )
        .unwrap();
        let desk = picture(
            &dock,
            look,
            (3840, 2160),
            &on_the_dock(4),
            closed(),
            &mut fonts(),
        )
        .unwrap();
        let portrait = picture(
            &dock,
            look,
            (1080, 1920),
            &on_the_dock(4),
            closed(),
            &mut fonts(),
        )
        .unwrap();
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
                closed(),
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
                    picture(
                        &Dock::shipped(),
                        look,
                        size,
                        &on_the_dock(4),
                        closed(),
                        &mut fonts()
                    ),
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
        let drawn = picture(&dock, look, (1920, 1080), &held, closed(), &mut fonts()).unwrap();

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
            let drawn = picture(&dock, look, size, &held, closed(), &mut fonts()).unwrap();
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
        let drawn = picture(
            &Dock::shipped(),
            look,
            (1920, 1080),
            &[],
            closed(),
            &mut fonts(),
        )
        .unwrap();
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
        let drawn = picture(
            &Dock::shipped(),
            look,
            (1920, 1080),
            &held,
            closed(),
            &mut fonts(),
        )
        .unwrap();

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
