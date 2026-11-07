//! **What each named cursor is drawn as**: the one table from a name an
//! application asks for to the drawing alo answers with.
//!
//! Every shape the cursor-shape protocol names has an answer here, and **the
//! match below has no catch-all for a name it knows**, so a name added to the
//! list arrives as a decision rather than as a silent arrow. Several names share
//! one drawing, as they do in every cursor theme: `e-resize`, `w-resize`,
//! `ew-resize` and `col-resize` all say *this moves left and right*, and a
//! double-headed arrow says so for all four, which is also what a window's edge
//! shows (ADR 0071).
//!
//! Where a name has no drawing of its own yet — *help*, *context-menu*, *alias*,
//! *dnd-ask* — it is drawn as the arrow, and [`has_its_own_drawing`] says so,
//! so the list of what is approximated can be read rather than discovered.

use cursor_icon::CursorIcon;

use crate::Silhouette;

/// Every cursor name the protocol defines, in its own order.
pub const EVERY_SHAPE: [CursorIcon; 36] = [
    CursorIcon::Default,
    CursorIcon::ContextMenu,
    CursorIcon::Help,
    CursorIcon::Pointer,
    CursorIcon::Progress,
    CursorIcon::Wait,
    CursorIcon::Cell,
    CursorIcon::Crosshair,
    CursorIcon::Text,
    CursorIcon::VerticalText,
    CursorIcon::Alias,
    CursorIcon::Copy,
    CursorIcon::Move,
    CursorIcon::NoDrop,
    CursorIcon::NotAllowed,
    CursorIcon::Grab,
    CursorIcon::Grabbing,
    CursorIcon::EResize,
    CursorIcon::NResize,
    CursorIcon::NeResize,
    CursorIcon::NwResize,
    CursorIcon::SResize,
    CursorIcon::SeResize,
    CursorIcon::SwResize,
    CursorIcon::WResize,
    CursorIcon::EwResize,
    CursorIcon::NsResize,
    CursorIcon::NeswResize,
    CursorIcon::NwseResize,
    CursorIcon::ColResize,
    CursorIcon::RowResize,
    CursorIcon::AllScroll,
    CursorIcon::ZoomIn,
    CursorIcon::ZoomOut,
    CursorIcon::DndAsk,
    CursorIcon::AllResize,
];

/// One of the distinct pictures this crate draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Drawing {
    /// The person's arrow, in whichever [`Silhouette`] they have.
    Arrow,
    /// The arrow with a small turning ring: *working, and you can still click*.
    Progress,
    /// The arrow with a plus: *this will be copied*.
    Copy,
    /// The text cursor, an I-beam.
    Text,
    /// The I-beam on its side, for text written top to bottom.
    VerticalText,
    /// A hand pointing with one finger: *this is a link*.
    Pointer,
    /// An open hand: *this can be picked up*.
    Grab,
    /// A closed hand: *this is being held*.
    Grabbing,
    /// Four arrows: *this moves any way*.
    Move,
    /// Left and right.
    Across,
    /// Up and down.
    Down,
    /// Top-left to bottom-right.
    Falling,
    /// Top-right to bottom-left.
    Rising,
    /// A circle struck through: *not here*.
    NotAllowed,
    /// A turning ring and no arrow: *wait*.
    Wait,
    /// A fine cross for aiming.
    Crosshair,
    /// A magnifier with a plus.
    ZoomIn,
    /// A magnifier with a minus.
    ZoomOut,
}

/// What `icon` is drawn as.
///
/// `CursorIcon` is marked non-exhaustive by its crate, so a name it adds later
/// is drawn as the arrow; every name it has today is answered by name above
/// that arm, and `every_name_has_an_answer_of_its_own` holds the list.
#[must_use]
pub const fn drawing_for(icon: CursorIcon) -> Drawing {
    match icon {
        CursorIcon::Progress => Drawing::Progress,
        CursorIcon::Copy => Drawing::Copy,
        CursorIcon::Text => Drawing::Text,
        CursorIcon::VerticalText => Drawing::VerticalText,
        CursorIcon::Pointer => Drawing::Pointer,
        CursorIcon::Grab => Drawing::Grab,
        CursorIcon::Grabbing => Drawing::Grabbing,
        CursorIcon::Move | CursorIcon::AllScroll | CursorIcon::AllResize => Drawing::Move,
        CursorIcon::EResize
        | CursorIcon::WResize
        | CursorIcon::EwResize
        | CursorIcon::ColResize => Drawing::Across,
        CursorIcon::NResize
        | CursorIcon::SResize
        | CursorIcon::NsResize
        | CursorIcon::RowResize => Drawing::Down,
        CursorIcon::NwResize | CursorIcon::SeResize | CursorIcon::NwseResize => Drawing::Falling,
        CursorIcon::NeResize | CursorIcon::SwResize | CursorIcon::NeswResize => Drawing::Rising,
        CursorIcon::NotAllowed | CursorIcon::NoDrop => Drawing::NotAllowed,
        CursorIcon::Wait => Drawing::Wait,
        CursorIcon::Crosshair | CursorIcon::Cell => Drawing::Crosshair,
        CursorIcon::ZoomIn => Drawing::ZoomIn,
        CursorIcon::ZoomOut => Drawing::ZoomOut,
        // The arrow itself, the four names drawn as the arrow for want of their
        // own picture, and whatever the list grows to.
        _ => Drawing::Arrow,
    }
}

/// Whether `icon` has a picture that says what it means, rather than being
/// drawn as the plain arrow for want of one.
#[must_use]
pub const fn has_its_own_drawing(icon: CursorIcon) -> bool {
    !matches!(drawing_for(icon), Drawing::Arrow) || matches!(icon, CursorIcon::Default)
}

/// How one piece of a drawing is painted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    /// The inside of the outline.
    Filled,
    /// A line this many units wide along the path.
    Stroked {
        /// The line's width, in units.
        width: f32,
        /// Whether its ends are round rather than cut square.
        round: bool,
    },
}

/// One piece of a drawing: SVG path data on the 32-unit box, and its paint.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// The path.
    pub d: String,
    /// How it is painted.
    pub paint: Paint,
}

impl Part {
    /// A filled outline.
    pub(crate) fn filled(d: impl Into<String>) -> Self {
        Self {
            d: d.into(),
            paint: Paint::Filled,
        }
    }

    /// A line `width` units wide.
    pub(crate) fn stroked(d: impl Into<String>, width: f32, round: bool) -> Self {
        Self {
            d: d.into(),
            paint: Paint::Stroked { width, round },
        }
    }
}

/// A whole drawing and the point in it that clicks.
#[derive(Debug, Clone, PartialEq)]
pub struct Artwork {
    /// Its pieces, in no particular order: they are one colour, so the order
    /// they are painted in cannot show.
    pub parts: Vec<Part>,
    /// The hotspot, in units on the 32-unit box.
    pub hotspot: (f32, f32),
}

/// The artwork for `drawing`, with the person's arrow where it has one.
#[must_use]
pub fn artwork(drawing: Drawing, silhouette: Silhouette) -> Artwork {
    match drawing {
        Drawing::Arrow => the_arrow(silhouette, None),
        Drawing::Progress => the_arrow(silhouette, Some(crate::artwork_marks::a_small_ring())),
        Drawing::Copy => the_arrow(silhouette, Some(crate::artwork_marks::a_small_plus())),
        Drawing::Text => crate::artwork_marks::text(),
        Drawing::VerticalText => crate::artwork_marks::vertical_text(),
        Drawing::Pointer => crate::artwork_hands::pointer(),
        Drawing::Grab => crate::artwork_hands::grab(),
        Drawing::Grabbing => crate::artwork_hands::grabbing(),
        Drawing::Move => crate::artwork_arrows::four_ways(),
        Drawing::Across => crate::artwork_arrows::both_ways(0.0),
        Drawing::Down => crate::artwork_arrows::both_ways(90.0),
        Drawing::Falling => crate::artwork_arrows::both_ways(45.0),
        Drawing::Rising => crate::artwork_arrows::both_ways(-45.0),
        Drawing::NotAllowed => crate::artwork_marks::not_allowed(),
        Drawing::Wait => crate::artwork_marks::wait(),
        Drawing::Crosshair => crate::artwork_marks::crosshair(),
        Drawing::ZoomIn => crate::artwork_marks::magnifier(true),
        Drawing::ZoomOut => crate::artwork_marks::magnifier(false),
    }
}

/// The person's arrow, with a badge beside its tail when the shape has one.
fn the_arrow(silhouette: Silhouette, badge: Option<Part>) -> Artwork {
    let mut parts = vec![Part::filled(silhouette.path())];
    parts.extend(badge);
    Artwork {
        parts,
        hotspot: silhouette.tip(),
    }
}

/// SVG path data for a closed polygon through `points`.
pub(crate) fn polygon(points: &[(f32, f32)]) -> String {
    let mut d = String::new();
    for (index, (x, y)) in points.iter().enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        d.push_str(&format!("{command}{x:.3} {y:.3} "));
    }
    d.push('Z');
    d
}

#[cfg(test)]
#[path = "artwork_tests.rs"]
mod tests;
