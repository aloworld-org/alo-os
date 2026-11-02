//! **Settings as a frame drew it**: where its picture landed on the display,
//! and where each category, row and list entry is in that picture — kept for
//! the press after the frame.
//!
//! # The positions Settings already records
//!
//! `crate::settings_raster::SettingsPicture` has always kept where it drew
//! every category, every target on the page that is in view, the list a picker
//! opened and each of its entries, in the picture's own pixels; until now only
//! tests read them. A press is carried back into the picture through the
//! camera's arithmetic run the other way, and then tested against those very
//! rectangles — so what a person presses is what they see, at any pan and
//! zoom, and a row scrolled out of the page's view is not pressed through the
//! window's edge.

use smithay::utils::{Physical, Point, Rectangle};

use crate::SettingsRow;
use crate::settings_canvas::Area;
use crate::settings_categories::SettingsCategory;
use crate::settings_pointed_at::PointedAt;
use crate::settings_raster::SettingsPicture;

/// Settings as one frame drew it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SettingsAsDrawn {
    /// Where the picture's whole room landed on the display.
    room_on_screen: Rectangle<i32, Physical>,
    /// The room's size, in the picture's own pixels.
    room: (i32, i32),
    /// The window, in the picture's pixels.
    panel: Area,
    /// Every category, where it is drawn.
    categories: Vec<(SettingsCategory, Area)>,
    /// Every row on the page in view, where it is drawn.
    targets: Vec<(SettingsRow, Area)>,
    /// The list a picker opened, while one is open.
    list: Option<Area>,
    /// Every entry of that list in view.
    entries: Vec<(SettingsRow, Area)>,
}

impl SettingsAsDrawn {
    /// What `picture` drew, with its room put at `room_on_screen` by the
    /// camera. [`None`] for a picture with no window in it.
    pub(crate) fn of(
        picture: &SettingsPicture,
        room_on_screen: Rectangle<i32, Physical>,
    ) -> Option<Self> {
        let panel = picture.panel?;
        Some(Self {
            room_on_screen,
            room: picture.size,
            panel,
            categories: picture.categories.clone(),
            targets: picture
                .targets
                .iter()
                .map(|target| (target.row.clone(), target.area))
                .collect(),
            list: picture.list,
            entries: picture.entries.clone(),
        })
    }

    /// A point on the display, carried back into the picture.
    fn into_the_picture(&self, at: Point<i32, Physical>) -> Option<Point<i32, Physical>> {
        let seen = self.room_on_screen;
        if seen.size.w <= 0 || seen.size.h <= 0 {
            return None;
        }
        let back = |from: i32, start: i32, seen: i32, room: i32| -> Option<i32> {
            i32::try_from(i64::from(from - start) * i64::from(room) / i64::from(seen)).ok()
        };
        Some(Point::from((
            back(at.x, seen.loc.x, seen.size.w, self.room.0)?,
            back(at.y, seen.loc.y, seen.size.h, self.room.1)?,
        )))
    }

    /// A rectangle in the picture, where it is seen on the display.
    fn on_the_screen(&self, area: Area) -> Rectangle<i32, Physical> {
        let seen = self.room_on_screen;
        let (across, down) = (self.room.0.max(1), self.room.1.max(1));
        let out = |from: i32, start: i32, seen: i32, room: i32| -> i32 {
            i32::try_from(i64::from(start) + i64::from(from) * i64::from(seen) / i64::from(room))
                .unwrap_or(start)
        };
        let left = out(area.loc.x, seen.loc.x, seen.size.w, across);
        let top = out(area.loc.y, seen.loc.y, seen.size.h, down);
        let right = out(area.loc.x + area.size.w, seen.loc.x, seen.size.w, across);
        let bottom = out(area.loc.y + area.size.h, seen.loc.y, seen.size.h, down);
        Rectangle::new((left, top).into(), (right - left, bottom - top).into())
    }

    /// The window, where it is seen on the display: what its edge sits on.
    pub(crate) fn the_window_on_the_screen(&self) -> Rectangle<i32, Physical> {
        self.on_the_screen(self.panel)
    }

    /// Whether the window is under `at`, a point on the display.
    pub(crate) fn is_under(&self, at: Point<i32, Physical>) -> bool {
        self.into_the_picture(at)
            .is_some_and(|point| self.panel.contains(point))
    }

    /// What a press at `at`, a point on the display, is on — or [`None`] off
    /// the window.
    ///
    /// **The list first**, because it is drawn over the page; then the side
    /// column's categories, then the page's rows. A row is only found inside
    /// the window, so the part of one scrolled under the window's edge is not.
    pub(crate) fn what_is_under(&self, at: Point<i32, Physical>) -> Option<PointedAt> {
        let point = self.into_the_picture(at)?;
        if !self.panel.contains(point) {
            return None;
        }
        if let Some(list) = self.list {
            if !list.contains(point) {
                return Some(PointedAt::OutsideTheList);
            }
            return Some(
                self.entries
                    .iter()
                    .find(|(_, area)| area.contains(point))
                    .map_or(PointedAt::TheWindow, |(row, _)| {
                        PointedAt::AnEntry(row.clone())
                    }),
            );
        }
        if let Some((category, _)) = self
            .categories
            .iter()
            .find(|(_, area)| area.contains(point))
        {
            return Some(PointedAt::ACategory(*category));
        }
        if let Some((row, _)) = self.targets.iter().find(|(_, area)| area.contains(point)) {
            return Some(PointedAt::ARow(row.clone()));
        }
        Some(PointedAt::TheWindow)
    }

    /// Where a category is seen on the display, for a test that presses it.
    #[cfg(test)]
    pub(crate) fn where_the_category_is(
        &self,
        which: SettingsCategory,
    ) -> Option<Rectangle<i32, Physical>> {
        self.categories
            .iter()
            .find(|(category, _)| *category == which)
            .map(|(_, area)| self.on_the_screen(*area))
    }

    /// Where a row is seen on the display, for a test that presses it.
    #[cfg(test)]
    pub(crate) fn where_the_row_is(&self, which: &SettingsRow) -> Option<Rectangle<i32, Physical>> {
        self.targets
            .iter()
            .find(|(row, _)| row == which)
            .map(|(_, area)| self.on_the_screen(*area))
    }

    /// Every row in view, for a test that picks one.
    #[cfg(test)]
    pub(crate) fn the_rows_in_view(&self) -> Vec<SettingsRow> {
        self.targets.iter().map(|(row, _)| row.clone()).collect()
    }
}

#[cfg(test)]
#[path = "settings_as_drawn_tests.rs"]
mod tests;
