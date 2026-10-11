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

/// The mark on the control that goes back from a window list to *More apps*.
///
/// A single left-pointing angle quotation mark, which is what the design draws
/// — `‹`, written as its code point for `THE_OVERFLOWS_MARK`'s reason: pasted,
/// it is indistinguishable from a less-than sign, and the two are different
/// characters with different widths that a reader reads aloud differently.
///
/// **Artwork and not a word**, so it is here and not in `alo-dock`'s
/// vocabulary. What a person using a screen reader is told about this control
/// is its accessible name, which is a sentence and is somebody else's to say.
const BACK_TO_THE_APPS: char = '\u{2039}';

/// The dock for one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// `alo-dock`'s layout for this display.
    pub(crate) layout: Layout,
    /// The whole band, along the bottom.
    pub(crate) band: Rectangle<i32, Physical>,
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
    /// The Dock's own surface: the rounded panel, its hairline and its shadow.
    ///
    /// **Replaces the two flat rectangles this drew until 2026-10-11** — the
    /// band and the accent rule — because a radius and a shadow are partial
    /// coverage and a rectangle is all or nothing. ADR 0101 and
    /// `crate::dock_surface` carry the why.
    ///
    /// [`None`] only for a panel with no area, which is a Dock that gave way.
    pub(crate) surface: Option<crate::painted::Washed>,
    /// The overflow list's own rectangle, when it is open and has something in
    /// it.
    ///
    /// **Not [`Self::panels`]'s area, which is the raster's.** A `Washed`
    /// carries its shadow, so its bounds run `THE_SHADOW_SPREADS` beyond the
    /// surface on every side — 312 wide for a 248-wide list. That is right for
    /// a texture and wrong for every question about *where the list is*: on
    /// 2026-10-11 two tests about the list's position were both written against
    /// the raster and both failed by exactly the shadow's reach, reading as
    /// faults in the layout.
    ///
    /// So the rectangle the design specifies is recorded separately, and it is
    /// what a press on a row will be tested against.
    pub(crate) overflow_list: Option<Rectangle<i32, Physical>>,
    /// Every other rounded panel this Dock drew — today, the overflow list.
    ///
    /// Separate from [`Self::surface`] because they are painted at different
    /// times: the Dock's own surface goes under its contents, and a list opened
    /// from it goes over them.
    pub(crate) panels: Vec<crate::painted::Washed>,
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

/// What the overflow list needs drawing: everything but the pixels.
///
/// A struct rather than eight arguments, because every one of them is a
/// different type of number and a caller that swapped two would compile.
struct TheOverflowIsDrawn<'a> {
    /// Everything *More apps* lists, in order.
    ///
    /// **Not the applications that overflowed.** It was those until
    /// 2026-10-11, when the owner ruled that this surface is *an application
    /// overview, not merely a list of icons that overflowed* — so it carries
    /// every pinned application and every open one, whether or not the bar had
    /// room, and including those whose windows are all minimised and which are
    /// therefore not on the bar at all.
    over: &'a [OnTheDock],
    /// The Dock's band, which the list opens inward from.
    at: Rectangle<i32, Physical>,
    /// Which edge the Dock is on, so *inward* has a direction.
    edge: alo_dock::Edge,
    /// The display, so the list can be kept on it.
    room: (i32, i32),
    /// The column the put-aside panel owns, which the list opens clear of.
    ///
    /// `None` on a display too narrow to hold one — see
    /// `crate::panel_raster::the_column_it_reserves`. **Not the panel's
    /// contents and not its rail**: the two surfaces are separate and this is
    /// the only thing one needs to know about the other.
    clear_of: Option<Rectangle<i32, Physical>>,
    /// The person's text size, which the rows grow with.
    text: alo_appearance::TextScale,
    /// Everything the popover says, already said: this file may not word a
    /// sentence, and a bundle cannot be half-filled.
    words: &'a crate::the_docks_words::TheDocksWords,
    /// One application's windows, when that is the face a person asked for.
    ///
    /// [`None`] for the overview, which is almost every frame. The two faces
    /// share a frame and differ in what sits inside it — and in how tall it is,
    /// because the picker's heading holds a back control and is 16 taller. So
    /// this is read before the popover is placed and not only when it is
    /// filled.
    picker: Option<&'a alo_dock::picking::ThePicker>,
    /// The person's colours.
    palette: crate::desktop_look::DesktopPalette,
}

/// Draw *More apps*, into `panels`, `solids` and `inked`.
///
/// The owner's design of 2026-10-11 — `More apps / Popover`, captured in
/// `docs/design/figma-snapshot/406-26881.xml`. A 448-wide popover holding a
/// fixed heading, a fixed search field, a **scrolling grid** of applications
/// and a fixed line of words.
///
/// **How big it is comes from `alo_dock::more_apps`**, not from here. That
/// module holds the owner's two caps — 536 for the whole popover and 336 for
/// the list inside it — and this file's job is to put it where the control is
/// and ink the words into it.
///
/// **It replaced a 248-wide list of names.** That was `More apps / opens
/// inward`, the surface ADR 0103 retired, and nothing of its geometry is left
/// here.
fn the_overflow_list(
    panels: &mut Vec<crate::painted::Washed>,
    inked: &mut Vec<Inked>,
    drawn: TheOverflowIsDrawn<'_>,
    fonts: &mut FontSystem,
) -> Rectangle<i32, Physical> {
    let TheOverflowIsDrawn {
        over,
        at,
        edge,
        room,
        clear_of,
        text,
        words,
        picker,
        palette,
    } = drawn;
    let (width, height) = room;
    let pad = i32::try_from(alo_dock::measures::INSIDE_MORE_APPS).unwrap_or(24);
    let down = i32::try_from(alo_dock::measures::DOWN_MORE_APPS).unwrap_or(16);
    let gap = i32::try_from(alo_dock::measures::BESIDE_THE_DOCK).unwrap_or(12);

    // **How much room there is between the bar and the far side of the
    // display**, which is what the popover may grow into before its own cap
    // binds. The cap usually binds first; this matters on a short output.
    let headroom = match edge {
        alo_dock::Edge::Bottom => at.loc.y - gap,
        alo_dock::Edge::Top => height - (at.loc.y + at.size.h) - gap,
        alo_dock::Edge::Left | alo_dock::Edge::Right => height - 2 * gap,
    }
    .max(1);

    // Pinned first, then everything else that is open — the two groups the
    // design draws and the order the owner's ruling puts them in.
    let pinned = over
        .iter()
        .filter(|on| matches!(on.pinned(), alo_dock::Pinned::ByThePerson))
        .count();
    let overview = alo_dock::more_apps::TheOverview::of(
        pinned,
        over.len().saturating_sub(pinned),
        alo_dock::Room::pixels(u32::try_from(headroom).unwrap_or(0)),
    );
    // **The picker's panel is 16 shorter in its list and the same overall**,
    // because its heading holds a back control. Both are capped at the owner's
    // 536 and both shrink on a short output; only the split between the fixed
    // parts and the list differs.
    let windows = picker.map(|picker| {
        alo_dock::picking::the_pickers_panel(
            picker.rows().len(),
            alo_dock::Room::pixels(u32::try_from(headroom).unwrap_or(0)),
        )
    });
    let across = i32::try_from(alo_dock::more_apps::TheOverview::width().as_pixels()).unwrap_or(0);
    let tall = i32::try_from(
        windows
            .map_or_else(
                || overview.height(),
                alo_dock::picking::ThePickersPanel::height,
            )
            .as_pixels(),
    )
    .unwrap_or(0);

    // **Inward from the bar, and kept on the display and clear of the panel.**
    // Unchanged from the list this replaces, because the placement was never
    // what the ruling changed — only what is inside.
    let origin = match edge {
        alo_dock::Edge::Bottom => (at.loc.x + at.size.w - across, at.loc.y - gap - tall),
        alo_dock::Edge::Top => (at.loc.x + at.size.w - across, at.loc.y + at.size.h + gap),
        alo_dock::Edge::Left => (at.loc.x + at.size.w + gap, at.loc.y + at.size.h - tall),
        alo_dock::Edge::Right => (at.loc.x - gap - across, at.loc.y + at.size.h - tall),
    };
    let (from, upto) = match clear_of {
        None => (0, width),
        Some(column) if column.loc.x <= 0 => (column.size.w + gap, width),
        Some(column) => (0, column.loc.x - gap),
    };
    let left = origin.0.clamp(from, (upto - across).max(from));
    let top = origin.1.clamp(0, (height - tall).max(0));

    let area = Rectangle::new((left, top).into(), (across, tall).into());
    let inner = across - 2 * pad;
    // The rounded grounds on this panel: the search field, and the scroll
    // thumb when the list has one. Declared here because the field is drawn
    // before the list is.
    let mut wells: Vec<Rectangle<i32, Physical>> = Vec::new();
    let line = i32::try_from(alo_dock::Room::a_line_at(text).as_pixels()).unwrap_or(1);
    let size = i32::try_from(alo_dock::Room::text_at(text).as_pixels()).unwrap_or(1);
    let metrics = Metrics::new(size as f32, line as f32);
    // **A function rather than a closure.** A closure capturing `fonts` and
    // `inked` holds both borrows for as long as it exists, and the grid below
    // needs them again — so the one that shaped the heading would make the one
    // that shapes a name a borrow error. Passing them in says the same thing
    // and lends them a line at a time.
    #[expect(
        clippy::too_many_arguments,
        reason = "two borrows and what one line of text needs; a bundle here would be a type per call site"
    )]
    fn ink(
        fonts: &mut FontSystem,
        inked: &mut Vec<Inked>,
        what: &str,
        at: (i32, i32),
        wide: i32,
        metrics: Metrics,
        on: [u8; 3],
        colour: [u8; 3],
        height: i32,
    ) {
        let said = crate::painted_text::sentence(fonts, what, wide, metrics, on, colour);
        if let Some(drawn) = said.placed(at.0, at.1, height) {
            inked.push(drawn);
        }
    }

    // otherwise be inked over the footer and over the canvas beneath.
    // **One application's windows instead of every application.** The frame is
    // the same and only what sits between its heading and its footer differs,
    // which is what the design draws: one popover with two faces.
    if let (Some(said), Some(panel)) = (words.picker.as_ref(), windows) {
        let head = i32::try_from(alo_dock::measures::THE_PICKERS_HEADING_IS_TALL).unwrap_or(48);
        let count = i32::try_from(alo_dock::measures::THE_WINDOW_COUNT_IS_TALL).unwrap_or(48);
        let row_is = i32::try_from(alo_dock::measures::A_WINDOW_ROW_IS_TALL).unwrap_or(106);
        let between = i32::try_from(alo_dock::measures::BETWEEN_APP_CELLS).unwrap_or(8);
        let icon = i32::try_from(alo_dock::measures::ICON).unwrap_or(48);

        // **A back control, then the title beside it.** The control is a 48
        // target like every other on the Dock; `‹` is drawn as its mark.
        let mut one = [0_u8; 4];
        ink(
            fonts,
            inked,
            BACK_TO_THE_APPS.encode_utf8(&mut one),
            (left + pad, top + pad + (head - line) / 2),
            icon,
            metrics,
            palette.dock,
            palette.ink,
            height,
        );
        ink(
            fonts,
            inked,
            &said.heading,
            (left + pad + icon + between, top + pad + (head - line) / 2),
            inner - icon - between,
            metrics,
            palette.dock,
            palette.ink,
            height,
        );
        // How many there are, and what choosing one does.
        ink(
            fonts,
            inked,
            &said.count,
            (left + pad, top + pad + head + down + (count - line) / 2),
            inner,
            metrics,
            palette.dock,
            palette.ink_quiet,
            height,
        );

        let rows_at = top + pad + head + down + count + down;
        let viewport = i32::try_from(panel.the_list_is_tall().as_pixels()).unwrap_or(0);
        let mut wells: Vec<Rectangle<i32, Physical>> = Vec::new();
        for (which, window) in picker
            .map(alo_dock::picking::ThePicker::rows)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            let Some(sits) = panel.where_a_row_sits(which) else {
                continue;
            };
            let y = rows_at + i32::try_from(sits.as_pixels()).unwrap_or(0);
            if y + row_is <= rows_at || y >= rows_at + viewport {
                continue;
            }
            // The row's own rounded surface, in the raster beside the circles.
            wells.push(Rectangle::new(
                (left + pad, y).into(),
                (inner, row_is).into(),
            ));
            // Its title, and under it what state it is in and where it sits.
            ink(
                fonts,
                inked,
                window.called(),
                (left + pad + pad, y + pad - 8),
                inner - 2 * pad,
                metrics,
                palette.badge,
                palette.ink,
                height,
            );
            if let Some(state) = said.states.get(which) {
                ink(
                    fonts,
                    inked,
                    state,
                    (left + pad + pad, y + pad + line + 1),
                    inner - 2 * pad,
                    metrics,
                    palette.badge,
                    palette.ink_quiet,
                    height,
                );
            }
        }
        ink(
            fonts,
            inked,
            &said.footer,
            (
                left + pad,
                top + tall
                    - pad
                    - i32::try_from(alo_dock::measures::THE_MORE_APPS_FOOTER_IS_TALL).unwrap_or(22),
            ),
            inner,
            metrics,
            palette.dock,
            palette.ink_quiet,
            height,
        );
        if let Some(washed) = crate::dock_surface::a_panel(
            area,
            palette.dock,
            palette.edge,
            room,
            crate::dock_surface::APanelsShapes {
                circles: &[],
                circle: palette.badge,
                marks: &[],
                mark: palette.ink,
                // **Every row is a well.** The rasteriser takes a list for
                // exactly this: a search field is one of them and a window
                // list is a column.
                wells: &wells,
                well_is: palette.badge,
            },
        ) {
            panels.push(washed);
        }
        return area;
    }

    // The heading.
    ink(
        fonts,
        inked,
        &words.heading,
        (left + pad, top + pad),
        inner,
        metrics,
        palette.dock,
        palette.ink,
        height,
    );

    // The search field: a rounded well with its invitation inside it. Drawn as
    // a `Solid` because it is a filled rectangle with no partial coverage at
    // its corners that a person can see at this size.
    let field = Rectangle::new(
        (
            left + pad,
            top + pad
                + i32::try_from(alo_dock::measures::THE_MORE_APPS_HEADING_IS_TALL).unwrap_or(32)
                + down,
        )
            .into(),
        (
            inner,
            i32::try_from(alo_dock::measures::THE_SEARCH_FIELD_IS_TALL).unwrap_or(48),
        )
            .into(),
    );
    wells.push(field);
    ink(
        fonts,
        inked,
        &words.search,
        (field.loc.x + pad, field.loc.y + (field.size.h - line) / 2),
        inner - 2 * pad,
        metrics,
        palette.badge,
        palette.ink_quiet,
        height,
    );

    // **The grid, clipped to its viewport.** Everything below here is inside
    // the scrolling list, so a cell the overview says is out of view is not
    // drawn at all rather than drawn and covered: a row past the bottom would

    let list: Rectangle<i32, Physical> = Rectangle::new(
        (field.loc.x, field.loc.y + field.size.h + down).into(),
        (
            inner,
            i32::try_from(overview.the_list_is_tall().as_pixels()).unwrap_or(0),
        )
            .into(),
    );
    let scrolled = i32::try_from(overview.scrolled().as_pixels()).unwrap_or(0);
    // **Gathered before the panel is rasterised**, because they are part of it.
    // A circle and a rounded well are partial coverage at their edges, so they
    // belong in the raster beside the panel's own corners rather than as flat
    // rectangles drawn over it — which is what they were, and what made the
    // badges square.
    let mut circles: Vec<Rectangle<i32, Physical>> = Vec::new();
    let mut marks: Vec<Rectangle<i32, Physical>> = Vec::new();
    let cell = i32::try_from(alo_dock::measures::AN_APP_CELL_IS_WIDE).unwrap_or(94);
    let icon = i32::try_from(alo_dock::measures::ICON).unwrap_or(48);
    let glyph = i32::try_from(alo_dock::measures::GLYPH).unwrap_or(28);

    // Each group's heading, where the group has anything in it.
    for (group, what) in [
        (overview.pinned(), &words.pinned),
        (overview.others(), &words.others),
    ] {
        if group.how_many() == 0 {
            continue;
        }
        let y = list.loc.y + i32::try_from(group.at().as_pixels()).unwrap_or(0) - scrolled;
        if y >= list.loc.y && y < list.loc.y + list.size.h {
            ink(
                fonts,
                inked,
                what,
                (list.loc.x, y),
                inner,
                metrics,
                palette.dock,
                palette.accent,
                height,
            );
        }
    }

    for (which, application) in over.iter().enumerate() {
        if !overview.is_shown(which) {
            continue;
        }
        let Some((x, y)) = overview.where_a_cell_sits(which) else {
            continue;
        };
        let at = (
            list.loc.x + i32::try_from(x.as_pixels()).unwrap_or(0),
            list.loc.y + i32::try_from(y.as_pixels()).unwrap_or(0) - scrolled,
        );
        // **The artwork's box, centred across the cell.** The design's own
        // arrangement: a 48 icon area at the top of a 94-wide cell, with the
        // 28 artwork centred inside that.
        let box_at = (at.0 + (cell - icon) / 2, at.1);
        let art = Rectangle::new(
            (box_at.0 + (icon - glyph) / 2, box_at.1 + (icon - glyph) / 2).into(),
            (glyph, glyph).into(),
        );
        // No artwork exists yet, so every application shows the initial the
        // owner ruled: a circle with its first letter. `dock_surface` draws the
        // circle with the Dock's own badges; this is the same shape in a list.
        circles.push(art);
        // **`AppId::first_letter`, which the Dock's own badges already use.**
        // It takes the first *grapheme* and not the first `char`, which is the
        // owner's ruling — an accented letter and a combined character are one
        // thing a person sees and two or more `char`s. Taking `chars().next()`
        // here would have split them, and it would have been a second way of
        // answering a question this workspace had already answered.
        let initial = application.app().first_letter();
        // **Shaped into a box the size of the letter, not the size of the
        // circle.** `painted_text` hands back an **opaque** box filled with the
        // ground colour, and the Dock's own initials survive that because the
        // ground is the badge's own — a badge-coloured box on a badge-coloured
        // circle is invisible.
        //
        // It is invisible only while the box is inside the circle. Shaped to
        // the circle's own 28 the box reaches the corners of a square the
        // circle does not fill, and the badge is drawn as a **square**: which
        // is what the first picture of this popover showed, with the Dock's
        // round badges in the same frame for comparison.
        //
        // So the width is the initial's size, as it is for the Dock.
        let initial_is = i32::try_from(alo_dock::measures::AN_INITIAL_IS).unwrap_or(14);
        let letter = crate::painted_text::centred(
            fonts,
            initial,
            initial_is,
            Metrics::new(initial_is as f32, glyph as f32),
            palette.badge,
            palette.badge_ink,
        );
        let (wide, high) = (letter.width_of_box, letter.height);
        if let Some(drawn) = letter.placed(
            art.loc.x + (glyph - wide) / 2,
            art.loc.y + (glyph - high) / 2,
            height,
        ) {
            inked.push(drawn);
        }
        // **A mark for an application that has a window**, the same two shapes
        // the bar draws: a short dot for running, a longer bar for the one
        // whose window has the keyboard. Nothing yet knows which is focused, so
        // every open application gets the dot.
        if application.is_open() {
            let wide = i32::try_from(alo_dock::measures::AN_OPEN_MARK_IS_WIDE).unwrap_or(4);
            let thick = i32::try_from(alo_dock::measures::A_RUNNING_MARK_IS_TALL).unwrap_or(3);
            marks.push(Rectangle::new(
                (box_at.0 + (icon - wide) / 2, box_at.1 + icon - thick).into(),
                (wide, thick).into(),
            ));
        }
        // Its name under it, centred across the cell.
        let name = crate::painted_text::sentence(
            fonts,
            application.app().name(),
            cell,
            metrics,
            palette.dock,
            palette.ink,
        );
        let wide = name.width_of_box;
        if let Some(drawn) = name.placed(
            at.0 + (cell - wide) / 2,
            at.1 + icon + i32::try_from(alo_dock::measures::BETWEEN_APP_CELLS).unwrap_or(8) / 2,
            height,
        ) {
            inked.push(drawn);
        }
    }

    // **The thumb, when there is more below.** Drawn as a well, which is what
    // every rounded shape on a panel is: 3 wide makes its radius 1.5, so both
    // ends are semicircles.
    if let Some((down_the_track, long)) = alo_dock::more_apps::a_scroll_thumb(
        overview.the_list_is_tall(),
        overview.the_whole_list_is_tall(),
        overview.scrolled(),
    ) {
        let thumb = i32::try_from(alo_dock::measures::A_SCROLL_THUMB_IS_WIDE).unwrap_or(3);
        wells.push(Rectangle::new(
            (
                list.loc.x + list.size.w - thumb,
                list.loc.y + i32::try_from(down_the_track.as_pixels()).unwrap_or(0),
            )
                .into(),
            (thumb, i32::try_from(long.as_pixels()).unwrap_or(0)).into(),
        ));
    }

    // **The line under the list, and which of the two it is.** The design
    // replaces one with the other rather than showing both: the same place,
    // one sentence at a time.
    ink(
        fonts,
        inked,
        if overview.scrolls() {
            &words.scrolling
        } else {
            &words.footer
        },
        (
            left + pad,
            top + tall
                - pad
                - i32::try_from(alo_dock::measures::THE_MORE_APPS_FOOTER_IS_TALL).unwrap_or(22),
        ),
        inner,
        metrics,
        palette.dock,
        palette.ink_quiet,
        height,
    );

    // **Rasterised last, drawn first.** It is pushed after the walk because the
    // walk is what gathers the circles that belong in it; the painter draws
    // every panel before any ink, so the order these two vectors are filled in
    // does not decide what covers what.
    if let Some(washed) = crate::dock_surface::a_panel(
        area,
        palette.dock,
        palette.edge,
        room,
        crate::dock_surface::APanelsShapes {
            circles: &circles,
            circle: palette.badge,
            marks: &marks,
            mark: palette.ink,
            wells: &wells,
            well_is: palette.badge,
        },
    ) {
        panels.push(washed);
    }

    area
}

/// Where one slot's 48 × 48 target sits on the screen.
///
/// **Centred across the bar's thickness, `along` down its edge** — the rule
/// `a_letter_sits_on_the_bars_centre_line_on_every_edge` holds, written once
/// now that two things need it: the draw that inks an initial, and the badge
/// placement that has to agree with it exactly or a letter would sit beside its
/// own circle.
fn a_slot_at(
    band: Rectangle<i32, Physical>,
    edge: alo_dock::Edge,
    along: i32,
    icon: i32,
) -> (i32, i32) {
    let across = ((if edge.runs_across() {
        band.size.h
    } else {
        band.size.w
    }) - icon)
        / 2;
    if edge.runs_across() {
        (band.loc.x + along, band.loc.y + across)
    } else {
        (band.loc.x + across, band.loc.y + along)
    }
}

/// Lay `dock` out on a display of `size` and rasterise it.
///
/// # It takes no display scale, and that is deliberate
///
/// Every figure here is `alo_dock`'s logical measure used directly as a pixel:
/// the band, the slots, the radius, the hairline and the marks are all in one
/// space, and **none of them is scaled**. This took a `display_scale` for one
/// commit and passed it to the radius alone, which made a dense display draw an
/// 18 radius as 36 on a band still 76 tall. Scaling one measure of an otherwise
/// unscaled geometry is worse than scaling none.
///
/// Making the Dock scale-aware is one change that moves all of them together,
/// and `CLAUDE.md`'s rule — no figure reaches a display without passing through
/// its scale — is kept by them sharing a conversion rather than by one of them
/// having its own.
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
    in_more_apps: &[OnTheDock],
    the_overflow: (
        &crate::WhatTheDockHasOpen,
        &crate::the_docks_words::TheDocksWords,
    ),
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
        thickness: _,
        ref places,
        ref over,
    } = standing;
    let (over, places) = (over.clone(), places.clone());
    let palette = look.palette().map_err(|_| RenderError::AccentRefused)?;

    // **The band and the accent are not rectangles any more.** They were two
    // `Solid`s until 2026-10-11, which is why the Dock had square corners and no
    // shadow: both are partial coverage, and a `Solid` is one opaque colour over
    // a rectangle. `crate::dock_surface` rasterises them together with the
    // hairline ADR 0101 ruled, and the renderer composites the result.
    //
    // `the_accent_on` still decides which face looks at the canvas; the surface
    // is told the edge and draws the rule on that face.
    let icon = i32::try_from(ICON).unwrap_or(i32::MAX);
    let badge_is = i32::try_from(GLYPH).unwrap_or(i32::MAX);

    // **Where every fallback badge goes, worked out before the surface** —
    // because a badge is drawn *into* the surface's raster, which is the only
    // way a circle gets an edge that is not a staircase.
    //
    // Every application gets one today: the owner's ruling of 2026-10-11 keeps
    // the first letter as the fallback for an application with **no usable
    // icon**, and `alo-applications` names an icon in none of its files, so
    // that is still every application. A real icon replaces a badge without the
    // slot moving, because the badge **is** the artwork box.
    let badges: Vec<Rectangle<i32, Physical>> = places
        .each()
        .iter()
        .filter(|place| place.app().is_some())
        .map(|place| {
            let along = i32::try_from(place.from_the_start()).unwrap_or(i32::MAX);
            let (left, top) = a_slot_at(band, layout.edge(), along, icon);
            let inset = (icon - badge_is) / 2;
            Rectangle::new(
                (left + inset, top + inset).into(),
                (badge_is, badge_is).into(),
            )
        })
        .collect();

    // Where the composer's field is, if this Dock has one, and the rule before
    // the commands — both in the surface's raster, because one is rounded and
    // the other is a hairline's worth of the same token.
    let slot_of = |what: &alo_dock::places::WhatIsHere| {
        places
            .each()
            .iter()
            .find(|place| place.what() == what)
            .map(|place| {
                let along = i32::try_from(place.from_the_start()).unwrap_or(i32::MAX);
                let across = i32::try_from(place.across()).unwrap_or(icon);
                let (left, top) = a_slot_at(band, layout.edge(), along, icon);
                let long =
                    i32::try_from(alo_dock::measures::THE_COMMANDS_DIVIDER_IS_LONG).unwrap_or(28);
                if layout.edge().runs_across() {
                    Rectangle::new((left, top).into(), (across, icon).into())
                } else {
                    // Down a side the slot's long axis is the bar's, so a field
                    // 344 along is 344 **tall** — which is what the left Dock's
                    // own `Open alo Bar` is not, and is the open question this
                    // names rather than guesses at.
                    let _ = long;
                    Rectangle::new((left, top).into(), (icon, across).into())
                }
            })
    };
    let composer = slot_of(&alo_dock::places::WhatIsHere::TheComposer);
    let divider = slot_of(&alo_dock::places::WhatIsHere::TheDivider);

    let mut panels: Vec<crate::painted::Washed> = Vec::new();
    let mut solids: Vec<Solid> = Vec::new();
    // **The rule between the applications and the commands.** Measured at 1 × 28
    // on the top Dock and 28 × 1 on the left one — the same line turned to suit
    // the bar — so its long side runs **across** the bar's thickness and its
    // short side along it. A `Solid`, because a hairline has no curve in it.
    if let Some(at) = divider {
        let long = i32::try_from(alo_dock::measures::THE_COMMANDS_DIVIDER_IS_LONG).unwrap_or(28);
        let thin = i32::try_from(alo_dock::measures::THE_DOCKS_EDGE).unwrap_or(1);
        let (size, offset) = if layout.edge().runs_across() {
            ((thin, long), ((at.size.w - thin) / 2, (icon - long) / 2))
        } else {
            ((long, thin), ((icon - long) / 2, (at.size.h - thin) / 2))
        };
        solids.push(Solid {
            area: Rectangle::new(
                (at.loc.x + offset.0, at.loc.y + offset.1).into(),
                size.into(),
            ),
            colour: palette.edge,
        });
    }
    // **The composer's field is a well**, which is what every rounded ground on
    // a panel is now called: one on the Dock, one in *More apps*, and one per
    // row in a window list. It is already computed above.
    let wells: Vec<Rectangle<i32, Physical>> = composer.into_iter().collect();
    let surface = crate::dock_surface::surface(crate::dock_surface::ADocksSurface {
        // **The bar's own marks are still flat**, and visibly so: the Dock's
        // surface is painted before `solids`, so a mark pushed there lands on
        // top of it. That is why the bar's indicators have always been visible
        // and the popover's were not.
        //
        // They should be rounded, by the same measurement as the popover's —
        // 4 by 3 with a 1.5 radius — and moving them is a change to the bar's
        // own drawing rather than to this seam. Named here rather than done in
        // passing.
        marks: &[],
        mark: [0, 0, 0],
        badges: &badges,
        badge: palette.badge,
        wells: &wells,
        well_is: palette.badge,
        panel: band,
        fill: palette.dock,
        edge: palette.edge,
        accent: palette.accent,
        against: layout.edge(),
        // Nothing: the rule across the bar is gone, and the surface keeps the
        // parameter so that a focus outline — which *does* follow a target —
        // has somewhere to arrive when it is built.
        accent_is: 0,
        room: size,
    });

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
    let (the_dock_has_open, words) = the_overflow;
    let initial = i32::try_from(alo_dock::measures::AN_INITIAL_IS).unwrap_or(i32::MAX);
    let utility = i32::try_from(A_UTILITY_GLYPH).unwrap_or(i32::MAX);
    let mut inked: Vec<Inked> = Vec::new();
    for place in places.each() {
        let along_the_bar = i32::try_from(place.from_the_start()).unwrap_or(i32::MAX);
        // The icon's own box, `ICON` square: `from_the_start` along the bar's
        // edge, and centred across its thickness.
        let (left, top) = a_slot_at(band, layout.edge(), along_the_bar, icon);
        // **Its own width**, which is a target's for an application and a
        // command, and the composer's for the field. `APlace::across` has
        // always been a field rather than a constant and this is why.
        let its_width = i32::try_from(place.across()).unwrap_or(icon);
        // **An application's initial on its badge, or the control's mark on the
        // bar.** The slot is the same size either way — ADR 0101 makes the
        // overflow one application slot — and what is in it is not: a letter
        // there would read as an application a person could open, and there is
        // no application called *more*.
        //
        // **The ground each is inked on is the thing that matters here.**
        // `painted_text::centred` returns an **opaque** box filled with the
        // ground it is given, so the ground is what the box paints over the
        // surface. An initial given the Dock's own colour paints a
        // dock-coloured tile **over its own badge** — which is what happened
        // for one commit: the circles were in the raster, 210 of their ~1 800
        // pixels survived as a fringe, and the picture showed bare letters
        // exactly as the owner said it must not.
        //
        // So the initial's ground is the badge's and the control's is the
        // Dock's, and they are read together rather than defaulted.
        //
        // The sizes are the owner's two: `AN_INITIAL_IS` — fourteen, half its
        // badge — and `A_UTILITY_GLYPH` for a mark that is not artwork.
        let mut one_character = [0_u8; 4];
        let label = i32::try_from(alo_dock::measures::A_LABEL_IS).unwrap_or(13);
        let (mark, size, ground, ink) = match place.what() {
            alo_dock::places::WhatIsHere::Application(app) => (
                app.first_letter(),
                initial,
                palette.badge,
                palette.badge_ink,
            ),
            alo_dock::places::WhatIsHere::TheOverflow => (
                &*THE_OVERFLOWS_MARK.encode_utf8(&mut one_character),
                utility,
                palette.dock,
                palette.ink,
            ),
            // **The composer's own two strings are drawn below**, where its
            // field is: a prompt at its left and the key at its right, neither
            // of them centred in a slot.
            alo_dock::places::WhatIsHere::TheComposer
            | alo_dock::places::WhatIsHere::TheDivider => continue,
            alo_dock::places::WhatIsHere::ShowAll => {
                (words.show_all.as_str(), label, palette.dock, palette.ink)
            }
            alo_dock::places::WhatIsHere::History => {
                (words.history.as_str(), label, palette.dock, palette.ink)
            }
        };
        let shaped = crate::painted_text::centred(
            fonts,
            mark,
            // **The slot's own width, not an icon's.** `Show all` is 88 wide and
            // was being shaped into a 48-wide buffer, which clipped it to
            // *Show al* — visible in the first drawing that had it.
            its_width,
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
            ground,
            ink,
        );
        // **The mark that says this application has a window open.**
        //
        // Measured on the design file's `Open window indicator` — `348:23886`
        // on the top Dock, `348:23686` on the left one, the same rectangle in
        // both: 4 × 3, centred across the slot and flush to its bottom. **It
        // does not rotate with the edge**: a Dock down the left side puts the
        // mark under each icon exactly as a Dock along the bottom does, which
        // is a measurement rather than an assumption — both variants were read.
        //
        // **Pinned but not open gets nothing**, which is the whole of the
        // distinction the design calls *open indicators distinguish pinned from
        // open*, and it is told by **width** rather than by colour.
        //
        // **The 12-wide focused mark is not drawn**, and this is where it would
        // go. `alo_dock::OnTheDock` has no notion of focus — it knows pinned,
        // open, and how many windows are put aside — and the shell's keyboard
        // focus is a `WlSurface` that nothing maps back to an application here.
        // Drawing the open mark for a focused application is the honest subset:
        // it is never wrong, it is incomplete, and `A_FOCUSED_MARK_IS_WIDE` is
        // declared and unused until something can answer the question.
        if place.app().is_some_and(|_| {
            on_the_dock
                .iter()
                .any(|one| Some(one.app()) == place.app() && one.is_open())
        }) {
            let wide = i32::try_from(alo_dock::measures::AN_OPEN_MARK_IS_WIDE).unwrap_or(1);
            let tall = i32::try_from(alo_dock::measures::A_RUNNING_MARK_IS_TALL).unwrap_or(1);
            solids.push(Solid {
                area: Rectangle::new(
                    (left + (icon - wide) / 2, top + icon - tall).into(),
                    (wide, tall).into(),
                ),
                colour: palette.ink,
            });
        }
        // **Centred on the letter's own rendered bounds, both ways.**
        //
        // `centred` now hands back a box trimmed to the ink actually in it, so
        // these two lines are optical centring rather than box centring: `F`,
        // `M` and `B` are three different widths and three different heights,
        // and each one's own ink is the only thing that says where its middle
        // is. Centring the *text box* instead put every letter where a Latin
        // capital with a full descender would have been.
        let across_by = ((its_width - shaped.width_of_box) / 2).max(0);
        let down_by = ((icon - shaped.height) / 2).max(0);
        if let Some(letter) = shaped.placed(left + across_by, top + down_by, height) {
            inked.push(letter);
        }
    }

    // **The composer's own two strings**, which the slot loop above skips: they
    // are not centred in a slot. Measured on both Dock families — the prompt at
    // the field's left behind its padding, the key at its right behind the same
    // — so each is placed from its own end rather than centred.
    if let Some(field) = composer {
        let pad = i32::try_from(alo_dock::measures::BESIDE_A_NAME_IN_A_LIST).unwrap_or(12);
        let body = i32::try_from(alo_dock::measures::TEXT_AT_ORDINARY).unwrap_or(15);
        let small = i32::try_from(alo_dock::measures::A_SMALL_LABEL_IS).unwrap_or(11);
        for (what, size, from_the_left) in [
            (words.prompt.as_str(), body, true),
            (words.key.as_str(), small, false),
        ] {
            if what.is_empty() {
                continue;
            }
            let shaped = crate::painted_text::centred(
                fonts,
                what,
                field.size.w,
                Metrics::new(size as f32, size as f32),
                palette.badge,
                palette.ink,
            );
            let x = if from_the_left {
                field.loc.x + pad
            } else {
                field.loc.x + field.size.w - pad - shaped.width_of_box
            };
            let y = field.loc.y + ((field.size.h - shaped.height) / 2).max(0);
            if let Some(drawn) = shaped.placed(x, y, height) {
                inked.push(drawn);
            }
        }
    }

    // **The list the control opens, when a person has opened it.**
    //
    // Drawn last so it sits over the bar and over the canvas, which is what a
    // list opened from a control has to do. `alo_dock::a_list_the_dock_opens::ThePanel` is
    // the layout — rows at the floor every control is held to, growing with the
    // text, scrolling when the room runs out — and this is the only thing that
    // turns it into pixels.
    // **Opened whether or not anything overflowed.** `over` is no longer the
    // tail that did not fit: the ruling keeps the control available *so the
    // person has a predictable route to the overview, even when all eligible
    // icons fit in the Dock*. The old condition tested the overflow tail, which
    // made this surface appear and vanish with the width of a display.
    // **Either face opens it.** The overview needs something to list; the
    // picker already refused to be open with nothing to choose between, which
    // `what_that_means_now` decides and this does not repeat.
    let it_is_open = (the_dock_has_open.is_more_apps() && !in_more_apps.is_empty())
        || the_dock_has_open.the_windows_of().is_some();
    let overflow_list = if it_is_open {
        Some(the_overflow_list(
            &mut panels,
            &mut inked,
            TheOverflowIsDrawn {
                over: in_more_apps,
                at: band,
                edge: layout.edge(),
                room: (width, height),
                // **The same derivation `crate::desktop_raster` hands the
                // panel**, and for the owner's reason of 2026-09-30: the
                // panel's edge is read from the direction a person reads in
                // rather than stored, so nothing names an edge and the two
                // cannot disagree. `look` is the one fact needed and this file
                // already has it.
                clear_of: crate::panel_raster::the_column_it_reserves(
                    look,
                    (width, height),
                    match look.reading() {
                        alo_strings::Direction::LeftToRight => {
                            alo_put_aside::the_region_the_panel_claims::WhichEdge::Right
                        }
                        alo_strings::Direction::RightToLeft => {
                            alo_put_aside::the_region_the_panel_claims::WhichEdge::Left
                        }
                    },
                ),
                text: look.scale(),
                words,
                picker: the_dock_has_open.the_windows_of(),
                palette,
            },
            fonts,
        ))
    } else {
        None
    };

    Ok(DockPicture {
        size,
        layout,
        band,
        solids,
        inked,
        over,
        standing,
        surface,
        overflow_list,
        panels,
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
    use alo_appearance::TextScale;
    use alo_dock::Screen;
    use alo_dock::measures::{FLOATING_ABOVE_THE_EDGE, MARGIN};
    use alo_strings::Direction;

    /// How many applications this Dock drew.
    ///
    /// **The bar holds more than applications since 2026-10-11** — the composer
    /// and one or two commands — so `inked.len()` is no longer the number of
    /// letters on it, and every test below that compared the two was asking
    /// about a bar that no longer exists.
    fn applications(drawn: &DockPicture) -> usize {
        drawn
            .standing
            .places
            .each()
            .iter()
            .filter(|place| place.app().is_some())
            .count()
    }

    /// How many things this Dock inks: every slot but the composer and the rule.
    ///
    /// The composer's two strings are drawn beside its field rather than
    /// centred in a slot, and a rule has nothing to say.
    fn inks(drawn: &DockPicture) -> usize {
        use alo_dock::places::WhatIsHere;
        drawn
            .standing
            .places
            .each()
            .iter()
            .filter(|place| {
                !matches!(
                    place.what(),
                    WhatIsHere::TheComposer | WhatIsHere::TheDivider
                )
            })
            .count()
    }

    /// A Dock whose overflow list a person has not opened, which is every frame
    /// but the one after they press the control.
    fn closed() -> (
        &'static crate::WhatTheDockHasOpen,
        &'static crate::the_docks_words::TheDocksWords,
    ) {
        (
            &crate::what_the_dock_has_open::NOTHING,
            crate::desktop_testing::no_words(),
        )
    }

    /// The same Dock with its overflow list open.
    fn opened() -> (
        &'static crate::WhatTheDockHasOpen,
        &'static crate::the_docks_words::TheDocksWords,
    ) {
        (
            &crate::what_the_dock_has_open::MORE_APPS,
            crate::desktop_testing::no_words(),
        )
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
            let drawn = picture(
                &dock,
                look,
                size,
                &on_the_dock(4),
                &on_the_dock(4),
                closed(),
                &mut fonts(),
            )
            .unwrap();
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
            // **What it holds, which is no longer only icons.** The bar carries
            // the composer and the commands since 2026-10-11, so its width is
            // `Places::how_far_across` rather than a count of slots one size.
            let wanted = i32::try_from(drawn.standing.places.how_far_across().as_pixels()).unwrap();
            assert_eq!(drawn.band.size.w, wanted, "{reading:?}");
            assert!(drawn.band.size.w < 1920, "{reading:?}: it spans the screen");

            // Centred: the room either side is equal to within a pixel.
            let left = drawn.band.loc.x;
            let right = 1920 - (drawn.band.loc.x + drawn.band.size.w);
            assert!((left - right).abs() <= 1, "{reading:?}: {left} vs {right}");

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
        let slots = applications(&drawn);
        assert_eq!(
            drawn.band.size.w,
            i32::try_from(drawn.standing.places.how_far_across().as_pixels()).unwrap(),
            "the bar was clamped, so it is not as wide as what it holds"
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
            slots + drawn.over.len(),
            held.len(),
            "{slots} on the bar, {} over, and {} were held",
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
            let drawn = picture(&dock, look, size, &held, &held, closed(), &mut fonts()).unwrap();

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

            let slots = applications(&drawn);
            assert!(
                !drawn.over.is_empty(),
                "{edge:?}: two hundred applications and nothing in the overflow"
            );
            assert_eq!(
                slots + drawn.over.len(),
                held.len(),
                "{edge:?}: {slots} on the bar and {} over, of {}",
                drawn.over.len(),
                held.len()
            );
            on_the_bar.push((edge, slots));
        }

        let down: Vec<usize> = on_the_bar
            .iter()
            .filter(|(edge, _)| !edge.runs_across())
            .map(|(_, holds)| *holds)
            .collect();
        assert_eq!(down.len(), 2, "the premise: two side edges");

        // **The two orientations hold different numbers**, which is the claim
        // that catches a length taken off the wrong axis: 1366 along the bottom
        // and 768 down a side is not the same bar.
        //
        // **The bottom and the top do not agree with each other**, and that is
        // ADR 0101 rather than a fault: *keep the distinct edge layouts.* The
        // bottom's trailing group is the composer and *Show all*, 88 wide; the
        // edges' is a rule, the composer and History, 48 wide. Different room
        // left over, different number of applications — measured here as
        // Bottom 14 and Top 15, and asserted as *each holds more than a side*
        // rather than as *the two agree*, which they have no reason to.
        let down_holds = down.first().copied().unwrap_or_default();
        assert_eq!(
            down.first(),
            down.last(),
            "the left and right edges have the same layout and hold different numbers: \
             {on_the_bar:?}"
        );
        for (edge, holds) in &on_the_bar {
            if edge.runs_across() {
                assert!(
                    *holds > down_holds,
                    "{edge:?} holds {holds} and a side holds {down_holds}, so the length is \
                     being taken off the same axis for both: {on_the_bar:?}"
                );
            }
        }
    }

    /// **The list is drawn as a rounded panel, not a flat rectangle.**
    ///
    /// It was one `Solid` with square corners and no shadow until 2026-10-11:
    /// the surface treatment ADR 0101 ruled reached the Dock and stopped there,
    /// and the first drawing that had both showed a Dock with a radius beside a
    /// list without one.
    #[test]
    fn the_open_list_is_a_rounded_panel_like_the_dock() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let crowded = on_the_dock(200);
        let of = |how| {
            picture(
                &Dock::shipped(),
                look,
                (1366, 768),
                &crowded,
                &crowded,
                how,
                &mut fonts(),
            )
            .unwrap()
        };
        let shut = of(closed());
        let open = of(opened());

        assert!(shut.panels.is_empty(), "a closed list drew a panel");
        assert_eq!(
            open.panels.len(),
            1,
            "an open list drew {} panels",
            open.panels.len()
        );

        // **A rounded panel has transparent corners.** A flat rectangle does
        // not, and that is the whole of what changed.
        let panel = open.panels.first().expect("one panel");
        let corner = panel.pixels.get(3).copied();
        assert!(
            corner.is_some_and(|alpha| alpha < 64),
            "the panel's own corner pixel is {corner:?} — opaque, so it is still a rectangle"
        );
    }

    /// **The list is drawn when a person opens it, and not before.**
    ///
    /// Three frames of one Dock, differing only in the thing a press turns over:
    /// closed draws the bar alone, open draws the bar and the list, and open on
    /// a Dock with nothing behind the control draws the bar alone again.
    ///
    /// The third is the one worth having. `WhatTheDockHasOpen` lives on the
    /// `Server` and is not cleared when the Dock stops overflowing — a person
    /// closes a window and the bar fits again — so a draw that trusted the flag
    /// alone would paint an empty list over the canvas.
    #[test]
    fn the_list_is_drawn_only_when_it_is_open_and_has_something_in_it() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let crowded = on_the_dock(200);
        let open = opened();

        let shut = picture(
            &Dock::shipped(),
            look,
            (1366, 768),
            &crowded,
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
            &crowded,
            open,
            &mut fonts(),
        )
        .unwrap();

        assert!(!shut.over.is_empty(), "the premise: something overflowed");
        // **A panel, not more solids.** This asked for more `solids` until
        // 2026-10-11, when the popover's shapes — its badges' circles, its
        // search field's rounded well, its running marks — moved into the
        // raster where they belong. A circle and a rounded end are partial
        // coverage, which a `Solid` cannot express, and `crate::desktop_paint`
        // paints every solid *before* any panel, so a shape pushed there was
        // drawn and then covered by the popover it was part of.
        //
        // So the popover adds **no** solids at all now, and the thing to assert
        // is the panel it adds instead.
        assert!(
            opened.panels.len() > shut.panels.len(),
            "opening the list rasterised no panel: {} against {}",
            opened.panels.len(),
            shut.panels.len()
        );
        assert!(
            opened.inked.len() > shut.inked.len(),
            "opening the list inked no more text: {} against {}",
            opened.inked.len(),
            shut.inked.len()
        );

        // **Open on a Dock with room for everything: it is still drawn.**
        //
        // This asserted the opposite until 2026-10-11 — *the flag says open and
        // there is nothing to show* — because the surface was the overflow and
        // a Dock with room has no overflow. ADR 0103 made it an overview whose
        // control is present *even when all eligible icons fit in the Dock*, so
        // four applications that all fit are four rows in it.
        //
        // Measured here rather than argued: `over` is empty and the list is
        // drawn anyway, which is exactly the pair the old rule said could not
        // happen.
        let roomy = picture(
            &Dock::shipped(),
            look,
            (1920, 1080),
            &on_the_dock(4),
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
            &on_the_dock(4),
            closed(),
            &mut fonts(),
        )
        .unwrap();
        assert!(roomy.over.is_empty(), "the premise: nothing overflowed");
        assert!(
            roomy.overflow_list.is_some(),
            "nothing overflowed, so the overview did not open — which is the \
             rule ADR 0103 replaced"
        );
        assert!(
            roomy.inked.len() > roomy_shut.inked.len(),
            "the overview opened and inked no names: {} against {}",
            roomy.inked.len(),
            roomy_shut.inked.len()
        );

        // **And nothing to show really does draw nothing.** The other half, and
        // the one the clause above could otherwise be satisfied by a list that
        // always opens: a Dock whose overview is empty draws no list even with
        // the flag set.
        let empty = picture(
            &Dock::shipped(),
            look,
            (1920, 1080),
            &[],
            &[],
            open,
            &mut fonts(),
        )
        .unwrap();
        assert!(
            empty.overflow_list.is_none(),
            "an empty overview was drawn over the canvas"
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
                &crowded,
                opened(),
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

    /// **An application with a window open carries a mark; a pinned one does
    /// not.**
    ///
    /// The design file's own sentence for the component — *open indicators
    /// distinguish pinned from open* — and the distinction is told by **width**
    /// rather than by colour, which is what `docs/design/the-alo-dock.md`
    /// promises and had nothing behind it until 2026-10-11.
    ///
    /// Asserted on all four edges, because the mark is measured as **not**
    /// rotating with the edge: a Dock down the left puts it under each icon
    /// exactly as one along the bottom does. Both variants were read from the
    /// file — `348:23886` and `348:23686` are the same rectangle — so a draw
    /// that helpfully turned it sideways would be wrong against a measurement.
    #[test]
    fn an_open_application_carries_a_mark_and_a_pinned_one_does_not() {
        use alo_dock::Edge;
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let size = (1920, 1080);

        for edge in Edge::EVERY {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);

            // Three with windows open.
            let open = picture(
                &dock,
                look,
                size,
                &on_the_dock(3),
                &on_the_dock(3),
                closed(),
                &mut fonts(),
            )
            .unwrap();
            // The same three, pinned and closed.
            let mut holding = alo_dock::Holding::nothing();
            for number in 0..3 {
                holding.pin(alo_dock::AppId::named(&format!("app-{number}")).unwrap());
            }
            let pinned = holding.showing(&alo_dock::Windows::none());
            let shut =
                picture(&dock, look, size, &pinned, &pinned, closed(), &mut fonts()).unwrap();

            let wide = i32::try_from(alo_dock::measures::AN_OPEN_MARK_IS_WIDE).unwrap();
            let tall = i32::try_from(alo_dock::measures::A_RUNNING_MARK_IS_TALL).unwrap();
            let marks = |drawn: &DockPicture| {
                drawn
                    .solids
                    .iter()
                    .filter(|solid| solid.area.size == (wide, tall).into())
                    .count()
            };

            assert_eq!(
                marks(&open),
                3,
                "{edge:?}: three applications have windows open and {} marks were drawn",
                marks(&open)
            );
            assert_eq!(
                marks(&shut),
                0,
                "{edge:?}: a pinned application that is not open was given a mark"
            );

            // **The premise**: both Docks drew the same three slots, so the
            // difference above is the mark and not the Dock being empty.
            assert_eq!(applications(&open), applications(&shut), "{edge:?}");
            assert_eq!(applications(&open), 3, "{edge:?}");
        }
    }

    /// **The mark sits under its own icon, centred, flush to the slot's
    /// bottom.**
    ///
    /// Where, not whether. Measured at `x: 20, y: 40` in a 44 slot — centred
    /// across it, and three tall ending at the slot's edge — which normalises
    /// to centred in a 48 target ending at its bottom.
    #[test]
    fn the_mark_sits_centred_under_its_own_icon() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let drawn = picture(
            &Dock::shipped(),
            look,
            (1920, 1080),
            &on_the_dock(3),
            &on_the_dock(3),
            closed(),
            &mut fonts(),
        )
        .unwrap();

        let icon = i32::try_from(alo_dock::measures::ICON).unwrap();
        let wide = i32::try_from(alo_dock::measures::AN_OPEN_MARK_IS_WIDE).unwrap();
        let tall = i32::try_from(alo_dock::measures::A_RUNNING_MARK_IS_TALL).unwrap();
        let marks: Vec<_> = drawn
            .solids
            .iter()
            .filter(|solid| solid.area.size == (wide, tall).into())
            .collect();
        assert_eq!(marks.len(), 3, "the premise");

        for (which, place) in drawn
            .standing
            .places
            .each()
            .iter()
            .filter(|place| place.app().is_some())
            .enumerate()
        {
            let along = i32::try_from(place.from_the_start()).unwrap();
            let (left, top) = if drawn.layout.edge().runs_across() {
                (
                    drawn.band.loc.x + along,
                    drawn.band.loc.y + (drawn.band.size.h - icon) / 2,
                )
            } else {
                (
                    drawn.band.loc.x + (drawn.band.size.w - icon) / 2,
                    drawn.band.loc.y + along,
                )
            };
            let wanted = (left + (icon - wide) / 2, top + icon - tall);
            assert!(
                marks
                    .iter()
                    .any(|mark| (mark.area.loc.x, mark.area.loc.y) == wanted),
                "no mark at {wanted:?} for slot {which}; marks are at {:?}",
                marks
                    .iter()
                    .map(|mark| (mark.area.loc.x, mark.area.loc.y))
                    .collect::<Vec<_>>()
            );
        }
    }

    /// **Every initial sits on a complete circle**, in both schemes.
    ///
    /// The owner's ruling of 2026-10-11 — *draw a complete circle … render only
    /// the letter's foreground over the badge, preserving the circle
    /// underneath* — asked after a drawing in which each badge was two
    /// crescents, above and below its letter.
    ///
    /// **A count is the wrong test for this and was the first one written.** It
    /// counted badge-coloured pixels and passed on a raster whose circles were
    /// all there and all about to be covered; tightened to count *after* the
    /// letters, it still only said *enough pixels*, which a crescent pair also
    /// satisfies. What the owner asked for is that the circle is **whole**, so
    /// this walks each badge's circumference and asks every point on it.
    #[test]
    fn every_initial_sits_on_a_complete_circle_in_both_schemes() {
        use alo_appearance::{Following, Scheme};
        for scheme in [Scheme::Light, Scheme::Dark] {
            let mut always = an_appearance();
            always.follow(Following::from(scheme));
            let look = noon_look(&always, Direction::LeftToRight);
            assert_eq!(
                look.scheme(),
                scheme,
                "the premise: this look is {scheme:?}"
            );

            let drawn = picture(
                &Dock::shipped(),
                look,
                (1920, 1080),
                &on_the_dock(3),
                &on_the_dock(3),
                closed(),
                &mut fonts(),
            )
            .unwrap();
            let surface = drawn.surface.as_ref().expect("the Dock has a surface");
            let wanted = look.palette().unwrap().badge;

            // The surface with every letter drawn over it, as `crate::painted`
            // composes them — because the fault being forbidden is the letters
            // covering the circles.
            let mut composed = surface.pixels.clone();
            let (width, height) = (surface.area.size.w, surface.area.size.h);
            for ink in &drawn.inked {
                for (n, pixel) in ink.pixels.iter().enumerate() {
                    let wide = ink.area.size.w.max(1);
                    let x = ink.area.loc.x + (n as i32 % wide) - surface.area.loc.x;
                    let y = ink.area.loc.y + (n as i32 / wide) - surface.area.loc.y;
                    if x < 0 || y < 0 || x >= width || y >= height {
                        continue;
                    }
                    let at = ((y as usize) * (width as usize) + (x as usize)) * 4;
                    if let Some(into) = composed.get_mut(at..at + 3) {
                        into.copy_from_slice(pixel);
                    }
                }
            }
            let at = |x: i32, y: i32| -> Option<[u8; 3]> {
                if x < 0 || y < 0 || x >= width || y >= height {
                    return None;
                }
                let n = ((y as usize) * (width as usize) + (x as usize)) * 4;
                let bytes = composed.get(n..n + 3)?;
                let mut out = [0_u8; 3];
                out.copy_from_slice(bytes);
                Some(out)
            };

            // Each badge's own circumference, eight points round it, just
            // inside the edge so the anti-aliased rim is not what is sampled.
            let badge_is = i32::try_from(GLYPH).unwrap();
            let icon = i32::try_from(ICON).unwrap();
            let mut circles = 0;
            for place in drawn.standing.places.each() {
                if place.app().is_none() {
                    continue;
                }
                let along = i32::try_from(place.from_the_start()).unwrap();
                let (left, top) = a_slot_at(drawn.band, drawn.layout.edge(), along, icon);
                let inset = (icon - badge_is) / 2;
                let (cx, cy) = (
                    left + inset + badge_is / 2 - surface.area.loc.x,
                    top + inset + badge_is / 2 - surface.area.loc.y,
                );
                let r = badge_is / 2 - 3;
                for (dx, dy) in [
                    (r, 0),
                    (-r, 0),
                    (0, r),
                    (0, -r),
                    (r * 7 / 10, r * 7 / 10),
                    (-r * 7 / 10, r * 7 / 10),
                    (r * 7 / 10, -r * 7 / 10),
                    (-r * 7 / 10, -r * 7 / 10),
                ] {
                    assert_eq!(
                        at(cx + dx, cy + dy),
                        Some(wanted),
                        "{scheme:?}: the badge at ({cx}, {cy}) is not its own colour {dx} across \
                         and {dy} down — the circle is not complete there"
                    );
                }
                circles += 1;
            }
            assert_eq!(circles, 3, "{scheme:?}: the premise, three badges");
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
            applications(&drawn),
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
        let drawn = picture(
            &dock,
            look,
            (1920, 1080),
            &held,
            &held,
            closed(),
            &mut fonts(),
        )
        .unwrap();

        assert_eq!(
            applications(&drawn),
            held.len(),
            "three applications on the Dock and {} laid out",
            applications(&drawn)
        );
        assert_eq!(
            drawn.inked.len(),
            inks(&drawn),
            "something other than a slot inked, or a slot did not"
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
            let drawn = picture(&dock, look, size, &held, &held, closed(), &mut fonts()).unwrap();
            assert_eq!(applications(&drawn), held.len(), "{edge:?}");

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
            &[],
            closed(),
            &mut fonts(),
        )
        .unwrap();
        assert_eq!(
            applications(&drawn),
            0,
            "a Dock holding nothing laid out an application"
        );
        assert!(
            drawn.surface.is_some(),
            "the premise: the Dock's surface is still drawn, so an empty `inked` is about \
             letters and not about nothing being drawn at all"
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
            &held,
            closed(),
            &mut fonts(),
        )
        .unwrap();

        assert_eq!(applications(&drawn), 3);
        let inks: Vec<&Inked> = drawn.inked.iter().take(3).collect();
        let _ = &inks;
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
