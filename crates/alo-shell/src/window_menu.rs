//! The window menu, and the title a person can select and copy from it.
//!
//! # The clause this exists for
//!
//! The owner's ruling of 2026-10-10: *the window menu also exposes the full,
//! selectable title, providing a persistent way to read and copy it with
//! pointer, keyboard or touch.*
//!
//! It is the third of three roads to the same string and the only one that
//! always exists. The ellipsis says there is more; `crate::window_edge_tooltip`
//! shows it to somebody reading now. Both of those fail a person who needs it
//! to **stay still** — to read slowly, to copy it, or to reach it on a touch
//! screen where there is no hover at all.
//!
//! # The content is `alo-menus`' and none of it is decided here
//!
//! `alo_menus::Menu::over(Subject::AWindow, …)` answers which entries a window
//! offers, from a list that crate calls **closed**: fourteen actions, no
//! application adds a row, and every one answers where else a person reaches
//! the same thing, so nothing is only in a right-click. This file lays that
//! answer out and adds the title block above it.
//!
//! **And `alo_menus` had no caller in this shell before this.** Its only
//! mentions outside its own crate were one test in `alo-dividing` and
//! `alo-saying`'s vocabulary collector — so *right-click context menus,
//! wherever a person expects one* was finished, tested, and reachable by
//! nobody. That is the fourth surface in this crate with that shape this week.
//!
//! # Selectable means boundaries, not a highlight
//!
//! A title a person can copy needs to be divisible where a person can divide
//! it, and that is **not** at every byte. [`TitleLine::boundaries`] holds the x
//! of each grapheme boundary, measured by shaping the prefix in the face the
//! menu draws in — so a pointer landing between two letters selects between
//! them, and a keyboard moving by one step moves by one letter rather than by
//! one byte of a Devanagari conjunct.
//!
//! The same reasoning as `crate::window_edge_title`'s cut, one layer along: the
//! scripts a byte index breaks are the ones with the least software already.

use alo_menus::{Action, Menu};
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::painted_text::how_wide;

/// How wide the menu is.
///
/// One figure rather than a fit, because a menu whose width followed its
/// longest entry would change width as a person's language changed — and a
/// person who has learned where *Close* is would find it somewhere else.
pub const THE_MENU_IS_WIDE: i32 = 260;

/// The space inside the menu's edge, on every side.
pub const WITHIN_THE_MENU: i32 = 8;

/// One entry's height, and the pointer target it gives.
///
/// 32, which is `window_edge`'s strip height and not a new figure: a row a
/// person presses should not be shorter than the smallest thing alo already
/// asks them to press.
pub const A_ROW_IS_TALL: i32 = 32;

/// One line of the title.
pub const A_TITLE_LINE_IS_TALL: i32 = 18;

/// The rule between the title and the entries.
pub const A_RULE_IS_TALL: i32 = 1;

/// The space above and below that rule.
pub const AROUND_THE_RULE: i32 = 6;

/// How many lines the title may take before it scrolls.
///
/// **Eight, where the tooltip's limit is four.** The tooltip is transient and
/// covers somebody's work; this is a surface they opened and can close, so it
/// may be taller. A title longer than eight lines of 244 logical pixels is
/// past what any layout helps with, and the selection still holds the whole
/// string.
pub const AT_MOST_TITLE_LINES: usize = 8;

/// One line of the title, and where a person may divide it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleLine {
    /// The line itself.
    pub text: String,
    /// Where its first glyph begins.
    pub at: Point<i32, Logical>,
    /// Where in the whole title this line starts, as a byte index.
    ///
    /// Carried so a selection across lines can be turned back into one
    /// substring of the title rather than a list of fragments.
    pub from: usize,
    /// The x of every grapheme boundary on this line, including both ends.
    ///
    /// `boundaries[0]` is `at.x` and the last is the line's right edge, so a
    /// line of *n* graphemes has *n + 1* entries. See this file's header for
    /// why these are graphemes and not bytes.
    pub boundaries: Vec<i32>,
}

/// One entry of the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// What choosing it does.
    pub does: Action,
    /// The whole row, which is what a pointer or a finger hits.
    pub area: Rectangle<i32, Logical>,
    /// Where the entry's word begins.
    pub word_at: Point<i32, Logical>,
}

/// The window menu, laid out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowMenu {
    /// The whole menu.
    pub panel: Rectangle<i32, Logical>,
    /// The full title, above the entries.
    pub title: Vec<TitleLine>,
    /// The rule between the title and the entries.
    pub rule: Rectangle<i32, Logical>,
    /// Every entry this window offers, in the order `alo_menus` gives them.
    pub rows: Vec<Row>,
    /// The whole title, for a reader and for a copy of all of it.
    pub whole: String,
}

/// Lay the window menu out at `from`, within `output`.
///
/// `menu` is `alo_menus`' answer about this window and `title` the
/// application's own untruncated string. `fonts` shapes both the wrap and the
/// grapheme boundaries, in the face the menu draws in.
///
/// Answers [`None`] when the menu offers no entry at all, which on a machine
/// with no agent and a subject with nothing to do would be an empty panel.
#[must_use]
pub fn window_menu(
    menu: &Menu,
    title: &str,
    from: Point<i32, Logical>,
    fonts: &mut cosmic_text::FontSystem,
    output: (i32, i32),
) -> Option<WindowMenu> {
    if menu.entries().is_empty() {
        return None;
    }
    let room = THE_MENU_IS_WIDE - 2 * WITHIN_THE_MENU;
    let lines = wrapped_title(title, room, fonts);

    let title_tall = i32::try_from(lines.len()).unwrap_or(0) * A_TITLE_LINE_IS_TALL;
    let rows_tall = i32::try_from(menu.entries().len()).unwrap_or(0) * A_ROW_IS_TALL;
    let rule_tall = if lines.is_empty() {
        0
    } else {
        A_RULE_IS_TALL + 2 * AROUND_THE_RULE
    };
    let size = Size::from((
        THE_MENU_IS_WIDE,
        2 * WITHIN_THE_MENU + title_tall + rule_tall + rows_tall,
    ));
    let panel = within(Rectangle::new(from, size), output);

    let mut y = panel.loc.y + WITHIN_THE_MENU;
    let mut at = 0usize;
    let mut title_lines = Vec::with_capacity(lines.len());
    for line in lines {
        let starts_at = Point::from((panel.loc.x + WITHIN_THE_MENU, y));
        let boundaries = boundaries_of(&line, starts_at.x, fonts);
        at += line.len();
        title_lines.push(TitleLine {
            from: at - line.len(),
            text: line,
            at: starts_at,
            boundaries,
        });
        y += A_TITLE_LINE_IS_TALL;
    }

    let rule = if title_lines.is_empty() {
        Rectangle::new(Point::from((panel.loc.x, y)), Size::from((0, 0)))
    } else {
        y += AROUND_THE_RULE;
        let it = Rectangle::new(
            Point::from((panel.loc.x + WITHIN_THE_MENU, y)),
            Size::from((room, A_RULE_IS_TALL)),
        );
        y += A_RULE_IS_TALL + AROUND_THE_RULE;
        it
    };

    let rows = menu
        .entries()
        .iter()
        .enumerate()
        .map(|(which, does)| {
            let down = y + i32::try_from(which).unwrap_or(0) * A_ROW_IS_TALL;
            Row {
                does: *does,
                // **The whole width**, so a press anywhere on the row chooses
                // it. A target the width of its word would be one a person has
                // to aim at.
                area: Rectangle::new(
                    Point::from((panel.loc.x, down)),
                    Size::from((panel.size.w, A_ROW_IS_TALL)),
                ),
                word_at: Point::from((
                    panel.loc.x + WITHIN_THE_MENU,
                    down + (A_ROW_IS_TALL - A_TITLE_LINE_IS_TALL) / 2,
                )),
            }
        })
        .collect();

    Some(WindowMenu {
        panel,
        title: title_lines,
        rule,
        rows,
        whole: title.to_owned(),
    })
}

impl WindowMenu {
    /// The part of the title between two points, as one string.
    ///
    /// **One substring and not a list of lines**, because what a person copies
    /// is the title and not the shape it happened to be wrapped into. Points
    /// are clamped to the title, so a drag that left the panel selects to its
    /// end rather than refusing.
    #[must_use]
    pub fn selected(&self, from: usize, to: usize) -> &str {
        let (from, to) = if from <= to { (from, to) } else { (to, from) };
        let end = to.min(self.whole.len());
        let start = from.min(end);
        // **Both ends moved to a boundary rather than trusted.** A byte index
        // from a pointer is not a character boundary, and slicing on one would
        // panic on every script that needs more than one byte a letter.
        let start = floor_boundary(&self.whole, start);
        let end = floor_boundary(&self.whole, end);
        self.whole.get(start..end).unwrap_or("")
    }

    /// Which grapheme boundary of the whole title is nearest `at`.
    ///
    /// What a pointer landing in the title block means, as a byte index into
    /// [`Self::whole`] — so a press and a release become a selection.
    #[must_use]
    pub fn boundary_at(&self, at: Point<i32, Logical>) -> Option<usize> {
        use unicode_segmentation::UnicodeSegmentation;
        let line = self
            .title
            .iter()
            .find(|line| at.y >= line.at.y && at.y < line.at.y + A_TITLE_LINE_IS_TALL)?;
        let mut nearest = line.from;
        let mut closest = i32::MAX;
        let mut walked = line.from;
        let mut edges = line.boundaries.iter().copied();
        if let Some(first) = edges.next()
            && (at.x - first).abs() < closest
        {
            closest = (at.x - first).abs();
            nearest = walked;
        }
        for (grapheme, x) in line.text.graphemes(true).zip(edges) {
            walked += grapheme.len();
            if (at.x - x).abs() < closest {
                closest = (at.x - x).abs();
                nearest = walked;
            }
        }
        Some(nearest)
    }
}

/// Move `at` back to the nearest character boundary at or before it.
fn floor_boundary(whole: &str, at: usize) -> usize {
    let mut at = at.min(whole.len());
    while at > 0 && !whole.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// The x of every grapheme boundary in `line`, starting at `from`.
fn boundaries_of(line: &str, from: i32, fonts: &mut cosmic_text::FontSystem) -> Vec<i32> {
    use unicode_segmentation::UnicodeSegmentation;
    let mut edges = Vec::with_capacity(line.graphemes(true).count() + 1);
    edges.push(from);
    let mut kept = String::with_capacity(line.len());
    for grapheme in line.graphemes(true) {
        kept.push_str(grapheme);
        edges.push(from + how_wide(fonts, &kept, the_metrics()));
    }
    edges
}

/// The metrics the menu's words are drawn with, which are the edge's.
fn the_metrics() -> cosmic_text::Metrics {
    cosmic_text::Metrics::new(12.0, 16.0)
}

/// Move `panel` so it is inside `output`.
///
/// Moved and never shrunk, for `window_edge_tooltip`'s reason: a narrower menu
/// would re-wrap the title, and a surface whose shape depended on where it was
/// opened would be one a person could not learn.
fn within(panel: Rectangle<i32, Logical>, output: (i32, i32)) -> Rectangle<i32, Logical> {
    let x = panel.loc.x.min(output.0 - panel.size.w).max(0);
    let y = panel.loc.y.min(output.1 - panel.size.h).max(0);
    Rectangle::new(Point::from((x, y)), panel.size)
}

/// `title`, broken into lines that each fit `room`.
fn wrapped_title(title: &str, room: i32, fonts: &mut cosmic_text::FontSystem) -> Vec<String> {
    use unicode_segmentation::UnicodeSegmentation;
    if title.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    for grapheme in title.graphemes(true) {
        let candidate = format!("{line}{grapheme}");
        if how_wide(fonts, &candidate, the_metrics()) <= room {
            line = candidate;
            continue;
        }
        if line.is_empty() {
            // Not one grapheme fits; nothing can be laid out in this room.
            return lines;
        }
        lines.push(std::mem::take(&mut line));
        if lines.len() >= AT_MOST_TITLE_LINES {
            return lines;
        }
        line.push_str(grapheme);
    }
    if !line.is_empty() && lines.len() < AT_MOST_TITLE_LINES {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
#[path = "window_menu_tests.rs"]
mod tests;
