//! A panel of rows, laid out and rasterised in one region of one display: the
//! shape both desktop windows are drawn in — what is running, and what is
//! filling the disk.
//!
//! # What it is handed, and what it does with it
//!
//! Sentences to put above the rows, and rows of cells: text that a crate which
//! owns the words or the numbers already answered. Nothing here words, orders,
//! sums or leaves out anything. A row's cells are set out along one line in
//! columns as wide as the widest cell of that column in view, and a cell that
//! does not fit moves to the next line rather than being cut. A row that does
//! not fit whole waits for the view to move, and a rail on the trailing edge
//! says what is out of view — a shape, so no sentence has to be invented for it.
//!
//! A row may be opened or closed (a folder, in the window of what is filling
//! the disk): a mark drawn in shapes says which, a minus for open and a plus
//! for closed. One row may be selected, and it is marked in the person's
//! accent — the only place the accent is spent in a window.
//!
//! # A refusal is a sentence in the panel
//!
//! Never an empty list: a machine with nothing running and a machine whose
//! kernel could not be read do not look the same.

use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;
use crate::desktop_look::{DesktopPalette, Measure};
use crate::painted::{Inked, Solid};
use crate::painted_text::{Shaped, sentence};

/// One row of a panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListRow {
    /// How deep it is in a tree: zero for a list.
    pub(crate) depth: usize,
    /// Whether it is open, when it can be opened at all.
    pub(crate) opened: Option<bool>,
    /// Its cells, in reading order.
    pub(crate) cells: Vec<String>,
}

/// What a panel shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListShows {
    /// The window is closed: nothing is drawn.
    Nothing,
    /// A refusal, in the words of whoever refused.
    Refusal(String),
    /// Rows, under sentences that stand above them.
    Rows {
        /// Drawn above the rows, and never moved by the view.
        remarks: Vec<String>,
        /// Every row, in the order they were handed over.
        rows: Vec<ListRow>,
        /// The first row the view starts from, when nothing is selected.
        first: usize,
        /// The selected row, which the view always holds.
        selected: Option<usize>,
    },
}

/// One piece of text as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CellDrawn {
    /// The text inked, exactly as it arrived.
    pub(crate) text: String,
    /// Its box.
    pub(crate) area: Rectangle<i32, Physical>,
}

/// One row as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowDrawn {
    /// Which row of those handed over.
    pub(crate) index: usize,
    /// The whole row.
    pub(crate) area: Rectangle<i32, Physical>,
    /// Its cells, in reading order.
    pub(crate) cells: Vec<CellDrawn>,
    /// The open-or-closed mark, when the row has one, and whether it is open.
    pub(crate) mark: Option<(Rectangle<i32, Physical>, bool)>,
}

/// A panel for one region of one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The panel, when anything is drawn.
    pub(crate) panel: Option<Rectangle<i32, Physical>>,
    /// The refusal, when one is drawn instead of rows.
    pub(crate) refusal: Option<CellDrawn>,
    /// The sentences above the rows.
    pub(crate) remarks: Vec<CellDrawn>,
    /// The rows in view, in order.
    pub(crate) rows: Vec<RowDrawn>,
    /// Whether rows before the view are out of it.
    pub(crate) earlier_out_of_view: bool,
    /// Whether rows after the view are out of it.
    pub(crate) later_out_of_view: bool,
    /// The rail's track and where the view is, when anything is out of view.
    pub(crate) rail: Option<(Rectangle<i32, Physical>, Rectangle<i32, Physical>)>,
    /// The selected row's accent bar, when it is in view.
    pub(crate) selection: Option<Rectangle<i32, Physical>>,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Text, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl ListPicture {
    /// Nothing drawn.
    pub(crate) fn empty(size: (i32, i32)) -> Self {
        Self {
            size,
            panel: None,
            refusal: None,
            remarks: Vec::new(),
            rows: Vec::new(),
            earlier_out_of_view: false,
            later_out_of_view: false,
            rail: None,
            selection: None,
            solids: Vec::new(),
            inked: Vec::new(),
        }
    }

    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.panel.is_none() && self.solids.is_empty() && self.inked.is_empty()
    }
}

/// How a panel is set out.
#[derive(Clone, Copy)]
pub(crate) struct ListLook {
    /// Measures at the person's text size.
    pub(crate) measure: Measure,
    /// The colours.
    pub(crate) palette: DesktopPalette,
    /// Whether the person reads right to left.
    pub(crate) right_to_left: bool,
}

/// Where things go inside a panel.
#[derive(Clone, Copy)]
struct Room {
    /// Where text starts across, on the left.
    text_x: i32,
    /// How wide text may be.
    inner: i32,
    /// The rail's left edge.
    rail_x: i32,
    /// The rail's width.
    rail: i32,
    /// Inside padding.
    pad: i32,
    /// Space between two things.
    gap: i32,
    /// How far each level of a tree is set in.
    indent: i32,
    /// The mark's side.
    mark: i32,
}

impl Room {
    /// The room for a panel filling `region`, or the refusal for one too small.
    fn in_region(region: Rectangle<i32, Physical>, look: ListLook) -> Result<Self, RenderError> {
        let measure = look.measure;
        let pad = measure.px(16);
        let rail = measure.px(10);
        let rail_gap = measure.px(10);
        let inner = region.size.w - 2 * pad - rail - rail_gap;
        if inner < measure.px(200) || region.size.h - 2 * pad < measure.px(120) {
            return Err(RenderError::DesktopScene);
        }
        let (text_x, rail_x) = if look.right_to_left {
            (region.loc.x + pad + rail + rail_gap, region.loc.x + pad)
        } else {
            (
                region.loc.x + pad,
                region.loc.x + region.size.w - pad - rail,
            )
        };
        Ok(Self {
            text_x,
            inner,
            rail_x,
            rail,
            pad,
            gap: measure.px(6),
            indent: measure.px(20),
            mark: measure.px(12),
        })
    }
}

/// A text shaped as narrow as its ink, and no wider than `most`.
fn tight(fonts: &mut FontSystem, text: &str, most: i32, look: ListLook) -> Shaped {
    let most = most.max(1);
    let (ground, ink) = (look.palette.ground, look.palette.ink);
    let first = sentence(fonts, text, most, look.measure.metrics(), ground, ink);
    if first.width > 0 && first.width + 1 < most {
        sentence(
            fonts,
            text,
            first.width + 1,
            look.measure.metrics(),
            ground,
            ink,
        )
    } else {
        first
    }
}

/// Lay a panel out in `region` of a display of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::DesktopScene`] when the region is outside the display, too
/// small for a panel, or cannot hold the sentences above the rows and the row
/// the view must hold, whole.
pub(crate) fn picture(
    shows: &ListShows,
    fonts: &mut FontSystem,
    size: (i32, i32),
    region: Rectangle<i32, Physical>,
    look: ListLook,
) -> Result<ListPicture, RenderError> {
    let mut drawn = ListPicture::empty(size);
    if matches!(shows, ListShows::Nothing) {
        return Ok(drawn);
    }
    let display = Rectangle::<i32, Physical>::from_size(size.into());
    if display.intersection(region) != Some(region) {
        return Err(RenderError::DesktopScene);
    }
    let room = Room::in_region(region, look)?;
    let edge = look.measure.px(2);
    match shows {
        ListShows::Nothing => Ok(drawn),
        ListShows::Refusal(said) => {
            let shaped = sentence(
                fonts,
                said,
                room.inner,
                look.measure.metrics(),
                look.palette.ground,
                look.palette.ink,
            );
            let height = 2 * room.pad + shaped.height;
            if height > region.size.h {
                return Err(RenderError::DesktopScene);
            }
            let top = region.loc.y + (region.size.h - height) / 2;
            let panel = Rectangle::new((region.loc.x, top).into(), (region.size.w, height).into());
            framed(&mut drawn.solids, panel, edge, look.palette);
            drawn.panel = Some(panel);
            drawn.refusal = Some(placed(
                &mut drawn.inked,
                shaped,
                said.clone(),
                room.text_x,
                top + room.pad,
            ));
            Ok(drawn)
        }
        ListShows::Rows {
            remarks,
            rows,
            first,
            selected,
        } => {
            framed(&mut drawn.solids, region, edge, look.palette);
            drawn.panel = Some(region);
            let bottom = region.loc.y + region.size.h - room.pad;
            let mut y = region.loc.y + room.pad;
            for remark in remarks {
                let shaped = sentence(
                    fonts,
                    remark,
                    room.inner,
                    look.measure.metrics(),
                    look.palette.ground,
                    look.palette.ink,
                );
                if y + shaped.height > bottom {
                    return Err(RenderError::DesktopScene);
                }
                let height = shaped.height;
                let at_x = room.text_x;
                drawn
                    .remarks
                    .push(placed(&mut drawn.inked, shaped, remark.clone(), at_x, y));
                y += height + room.gap;
            }
            if !remarks.is_empty() {
                drawn.solids.push(Solid {
                    area: Rectangle::new((room.text_x, y).into(), (room.inner, 1).into()),
                    colour: look.palette.ink,
                });
                y += 1 + room.gap;
            }
            let rows_top = y;
            let selected = selected.filter(|at| *at < rows.len());
            let mut cache: Vec<Option<ShapedRow>> = Vec::new();
            cache.resize_with(rows.len(), || None);
            let mut start = match selected {
                Some(at) => (*first).min(at),
                None => (*first).min(rows.len().saturating_sub(1)),
            };
            let laid = loop {
                let laid = laid_out(fonts, rows, &mut cache, start, rows_top, bottom, room, look);
                let holds = match selected {
                    Some(at) => laid.iter().any(|row| row.index == at),
                    None => !laid.is_empty() || rows.is_empty(),
                };
                if holds {
                    break laid;
                }
                if laid.is_empty() || selected.is_none_or(|at| start >= at) {
                    return Err(RenderError::DesktopScene);
                }
                start += 1;
            };
            for row in laid {
                if Some(row.index) == selected {
                    let bar = Rectangle::new(
                        (
                            if look.right_to_left {
                                room.text_x + room.inner - look.measure.px(4)
                            } else {
                                room.text_x
                            },
                            row.area.loc.y,
                        )
                            .into(),
                        (look.measure.px(4), row.area.size.h).into(),
                    );
                    drawn.solids.push(Solid {
                        area: bar,
                        colour: look.palette.accent,
                    });
                    drawn.selection = Some(bar);
                }
                if let Some((area, open)) = row.mark {
                    mark(&mut drawn.solids, area, open, look);
                }
                drawn.inked.extend(row.inked);
                drawn.rows.push(RowDrawn {
                    index: row.index,
                    area: row.area,
                    cells: row.cells,
                    mark: row.mark,
                });
            }
            let total = rows.len();
            drawn.earlier_out_of_view = start > 0 && total > 0;
            drawn.later_out_of_view = start + drawn.rows.len() < total;
            if drawn.earlier_out_of_view || drawn.later_out_of_view {
                let track = Rectangle::new(
                    (room.rail_x, rows_top).into(),
                    (room.rail, (bottom - rows_top).max(room.rail)).into(),
                );
                let height = track.size.h;
                let at = |rows: usize| (height as usize * rows / total.max(1)) as i32;
                let thumb = Rectangle::new(
                    (room.rail_x, rows_top + at(start)).into(),
                    (room.rail, at(drawn.rows.len()).max(look.measure.px(8))).into(),
                );
                framed(&mut drawn.solids, track, look.measure.px(1), look.palette);
                drawn.solids.push(Solid {
                    area: thumb,
                    colour: look.palette.ink,
                });
                drawn.rail = Some((track, thumb));
            }
            Ok(drawn)
        }
    }
}

/// A row's cells shaped, not yet placed.
struct ShapedRow {
    /// Each cell, shaped as narrow as its ink.
    cells: Vec<Shaped>,
}

/// A row laid out.
struct LaidRow {
    /// Which row.
    index: usize,
    /// The whole row.
    area: Rectangle<i32, Physical>,
    /// Its cells as drawn.
    cells: Vec<CellDrawn>,
    /// Its mark.
    mark: Option<(Rectangle<i32, Physical>, bool)>,
    /// Its ink.
    inked: Vec<Inked>,
}

/// Every row that fits whole from `start`, laid out from `top` to `bottom`.
#[expect(
    clippy::too_many_arguments,
    reason = "one layout pass: fonts, the rows, their shapes, the view and the room"
)]
fn laid_out(
    fonts: &mut FontSystem,
    rows: &[ListRow],
    cache: &mut [Option<ShapedRow>],
    start: usize,
    top: i32,
    bottom: i32,
    room: Room,
    look: ListLook,
) -> Vec<LaidRow> {
    let line = look.measure.metrics().line_height.ceil() as i32;
    let could_fit = usize::try_from(((bottom - top) / line.max(1)).max(1)).unwrap_or(1);
    let end = rows.len().min(start.saturating_add(could_fit));
    let set_in = |row: &ListRow| {
        let depth = i32::try_from(row.depth).unwrap_or(i32::MAX / 2);
        room.indent.saturating_mul(depth).min(room.inner / 2) + room.mark + room.gap
    };
    for (index, row) in rows.iter().enumerate().take(end).skip(start) {
        if let Some(slot) = cache.get_mut(index)
            && slot.is_none()
        {
            let most = room.inner - set_in(row);
            *slot = Some(ShapedRow {
                cells: row
                    .cells
                    .iter()
                    .map(|cell| tight(fonts, cell, most, look))
                    .collect(),
            });
        }
    }
    let mut widths: Vec<i32> = Vec::new();
    for shaped in cache.iter().take(end).skip(start).flatten() {
        for (column, cell) in shaped.cells.iter().enumerate() {
            if widths.len() <= column {
                widths.push(0);
            }
            if let Some(width) = widths.get_mut(column) {
                *width = (*width).max(cell.width_of_box);
            }
        }
    }

    let column_gap = look.measure.px(16);
    let mut laid = Vec::new();
    let mut y = top;
    for index in start..end {
        let (Some(row), Some(Some(shaped))) = (rows.get(index), cache.get(index)) else {
            break;
        };
        let begins = set_in(row);
        let mut x = begins;
        let mut line_top = 0;
        let mut line_height = 0;
        let mut boxes = Vec::new();
        for (column, cell) in shaped.cells.iter().enumerate() {
            let width = widths
                .get(column)
                .copied()
                .unwrap_or(0)
                .max(cell.width_of_box);
            if x > begins && x + cell.width_of_box > room.inner {
                x = begins;
                line_top += line_height;
                line_height = 0;
            }
            boxes.push((x, line_top));
            x += width + column_gap;
            line_height = line_height.max(cell.height);
        }
        let height = (line_top + line_height).max(line);
        if y + height > bottom {
            break;
        }
        let mut cells = Vec::new();
        let mut inked = Vec::new();
        for ((dx, dy), (cell, text)) in boxes
            .into_iter()
            .zip(shaped.cells.iter().zip(row.cells.iter()))
        {
            let x = if look.right_to_left {
                room.text_x + room.inner - dx - cell.width_of_box
            } else {
                room.text_x + dx
            };
            let area = Rectangle::new((x, y + dy).into(), (cell.width_of_box, cell.height).into());
            inked.push(Inked {
                area,
                pixels: cell.pixels.clone(),
            });
            cells.push(CellDrawn {
                text: text.clone(),
                area,
            });
        }
        let mark = row.opened.map(|open| {
            let before = begins - room.mark - room.gap;
            let x = if look.right_to_left {
                room.text_x + room.inner - before - room.mark
            } else {
                room.text_x + before
            };
            let y = y + (line - room.mark) / 2;
            (
                Rectangle::new((x, y).into(), (room.mark, room.mark).into()),
                open,
            )
        });
        laid.push(LaidRow {
            index,
            area: Rectangle::new((room.text_x, y).into(), (room.inner, height).into()),
            cells,
            mark,
            inked,
        });
        y += height + room.gap;
    }
    laid
}

/// Text placed whole at (`x`, `y`): it was measured to fit.
fn placed(inked: &mut Vec<Inked>, shaped: Shaped, text: String, x: i32, y: i32) -> CellDrawn {
    let area = Rectangle::new((x, y).into(), (shaped.width_of_box, shaped.height).into());
    let bottom = y + shaped.height;
    inked.extend(shaped.placed(x, y, bottom));
    CellDrawn { text, area }
}

/// An edge `edge` pixels thick in ink, and the ground inside it.
fn framed(
    solids: &mut Vec<Solid>,
    area: Rectangle<i32, Physical>,
    edge: i32,
    palette: DesktopPalette,
) {
    solids.push(Solid {
        area,
        colour: palette.ink,
    });
    solids.push(Solid {
        area: Rectangle::new(
            (area.loc.x + edge, area.loc.y + edge).into(),
            (area.size.w - 2 * edge, area.size.h - 2 * edge).into(),
        ),
        colour: palette.ground,
    });
}

/// The open-or-closed mark: a square edge, a bar across, and a bar down when
/// the row is closed — so which it is does not depend on a colour.
fn mark(solids: &mut Vec<Solid>, area: Rectangle<i32, Physical>, open: bool, look: ListLook) {
    let thin = look.measure.px(2).min(area.size.w / 4).max(1);
    framed(solids, area, (thin / 2).max(1), look.palette);
    let middle = area.size.w / 2;
    let inset = thin * 2;
    solids.push(Solid {
        area: Rectangle::new(
            (area.loc.x + inset, area.loc.y + middle - thin / 2).into(),
            (area.size.w - 2 * inset, thin).into(),
        ),
        colour: look.palette.ink,
    });
    if !open {
        solids.push(Solid {
            area: Rectangle::new(
                (area.loc.x + middle - thin / 2, area.loc.y + inset).into(),
                (thin, area.size.h - 2 * inset).into(),
            ),
            colour: look.palette.ink,
        });
    }
}

#[cfg(test)]
#[path = "desktop_list_tests.rs"]
mod tests;
