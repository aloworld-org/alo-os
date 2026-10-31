//! **The Settings window's icons**: one beside each category, the plus on
//! *Add a provider*, the chevron on a row that opens something, the sign on a
//! note and on what a choice risks.
//!
//! The files are in `crates/alo-shell/icons/settings/`, byte for byte as the
//! design file exported them on 2026-10-31, with the node each came from in
//! `where-they-came-from.toml` beside them. They are read and drawn by
//! `alo_vector_shapes`, the rasteriser the task window's reply box already
//! uses, so they stay sharp at every scale (ADR 0142).
//!
//! # Placed as the design places them
//!
//! Each file is its *Group*'s drawing with its stroke, so it is a little
//! larger than the group and centred on it. An icon is drawn into its frame —
//! 20 beside a category, 14 before a note — by putting the file's box centred
//! on the group's, at the frame's scale.
//!
//! # Inked in the palette's colour
//!
//! The colours in the files are the design's light values; what is drawn is
//! the colour the window's palette gives the place it is used in, so the icons
//! follow the person's accent and contrast as the words beside them do.

use std::sync::OnceLock;

use alo_vector_shapes::AShape;

use crate::settings_canvas::{Area, Canvas};

/// One of the window's icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ASettingsIcon {
    /// alo and AI.
    Chat,
    /// Folders alo can use.
    Folder,
    /// Running code.
    Code,
    /// Terminal.
    Terminal,
    /// Appearance.
    Contrast,
    /// Region and time.
    Globe,
    /// Keyboard and shortcuts.
    Keyboard,
    /// Accessibility.
    Access,
    /// A note that asks for attention.
    Info,
    /// A note saying where a service answers.
    Cloud,
    /// Adding something.
    Plus,
    /// A row that opens something.
    ChevronRight,
    /// What a choice risks.
    Warn,
}

impl ASettingsIcon {
    /// Every icon.
    pub(crate) const ALL: [Self; 13] = [
        Self::Chat,
        Self::Folder,
        Self::Code,
        Self::Terminal,
        Self::Contrast,
        Self::Globe,
        Self::Keyboard,
        Self::Access,
        Self::Info,
        Self::Cloud,
        Self::Plus,
        Self::ChevronRight,
        Self::Warn,
    ];

    /// The file, as the design exported it.
    const fn svg(self) -> &'static str {
        match self {
            Self::Chat => include_str!("../icons/settings/chat.svg"),
            Self::Folder => include_str!("../icons/settings/folder.svg"),
            Self::Code => include_str!("../icons/settings/code.svg"),
            Self::Terminal => include_str!("../icons/settings/terminal.svg"),
            Self::Contrast => include_str!("../icons/settings/contrast.svg"),
            Self::Globe => include_str!("../icons/settings/globe.svg"),
            Self::Keyboard => include_str!("../icons/settings/keyboard.svg"),
            Self::Access => include_str!("../icons/settings/access.svg"),
            Self::Info => include_str!("../icons/settings/info.svg"),
            Self::Cloud => include_str!("../icons/settings/cloud.svg"),
            Self::Plus => include_str!("../icons/settings/plus.svg"),
            Self::ChevronRight => include_str!("../icons/settings/chevron-right.svg"),
            Self::Warn => include_str!("../icons/settings/warn.svg"),
        }
    }

    /// The file's name, as `where-they-came-from.toml` records it.
    #[cfg(test)]
    const fn file_name(self) -> &'static str {
        match self {
            Self::Chat => "chat.svg",
            Self::Folder => "folder.svg",
            Self::Code => "code.svg",
            Self::Terminal => "terminal.svg",
            Self::Contrast => "contrast.svg",
            Self::Globe => "globe.svg",
            Self::Keyboard => "keyboard.svg",
            Self::Access => "access.svg",
            Self::Info => "info.svg",
            Self::Cloud => "cloud.svg",
            Self::Plus => "plus.svg",
            Self::ChevronRight => "chevron-right.svg",
            Self::Warn => "warn.svg",
        }
    }

    /// The icon's frame in the design, and where its group sits in it:
    /// left, top, width and height, in the frame's units — as
    /// `where-they-came-from.toml` records them.
    const fn placed(self) -> (f32, [f32; 4]) {
        match self {
            Self::Chat | Self::Contrast => (20.0, [2.5, 2.5, 15.0, 15.0]),
            Self::Folder => (20.0, [1.667, 2.5, 16.667, 14.167]),
            Self::Code => (20.0, [1.667, 5.0, 16.667, 10.0]),
            Self::Terminal => (20.0, [3.333, 4.167, 13.333, 11.667]),
            Self::Globe => (20.0, [1.667, 1.667, 16.667, 16.667]),
            Self::Keyboard => (20.0, [1.667, 4.167, 16.667, 11.667]),
            Self::Access => (20.0, [4.167, 2.5, 11.667, 15.0]),
            Self::Info => (14.0, [1.167, 1.167, 11.667, 11.667]),
            Self::Cloud => (14.0, [1.167, 2.917, 11.666, 8.167]),
            Self::Plus => (16.0, [3.333, 3.333, 9.333, 9.333]),
            Self::ChevronRight => (20.0, [7.5, 5.0, 5.0, 10.0]),
            Self::Warn => (18.0, [1.487, 2.239, 15.012, 13.511]),
        }
    }

    /// Its place in [`Self::ALL`].
    fn at(self) -> usize {
        Self::ALL
            .iter()
            .position(|it| *it == self)
            .unwrap_or_default()
    }

    /// The artwork, read once for the life of the shell.
    ///
    /// [`None`] for a file `alo_vector_shapes` refused, said once on the error
    /// stream, and refused on every build by this file's tests.
    fn shape(self) -> Option<&'static AShape> {
        static READ: OnceLock<Vec<Option<AShape>>> = OnceLock::new();
        READ.get_or_init(|| {
            ASettingsIcon::ALL
                .iter()
                .map(|icon| match AShape::read(icon.svg()) {
                    Ok(shape) => Some(shape),
                    Err(why) => {
                        eprintln!(
                            "alo-shell: a Settings icon could not be read and is not drawn: {why}"
                        );
                        None
                    }
                })
                .collect()
        })
        .get(self.at())
        .and_then(Option::as_ref)
    }

    /// Draw this icon into `frame`, a square in the display's pixels, in
    /// `colour`.
    pub(crate) fn drawn(self, canvas: &mut Canvas, frame: Area, colour: [u8; 3]) {
        let Some(shape) = self.shape() else {
            return;
        };
        let (side, [left, top, wide, tall]) = self.placed();
        let scale = frame.size.w as f32 / side;
        let (across, down) = shape.its_box();
        let (pixels_across, pixels_down) = (
            (across * scale).round().max(1.0),
            (down * scale).round().max(1.0),
        );
        let centre = (
            frame.loc.x as f32 + (left + wide / 2.0) * scale,
            frame.loc.y as f32 + (top + tall / 2.0) * scale,
        );
        let x = (centre.0 - pixels_across / 2.0).round() as i32;
        let y = (centre.1 - pixels_down / 2.0).round() as i32;
        let covered = shape.coverage(pixels_across as u32, pixels_down as u32);
        canvas.artwork(x, y, pixels_across as i32, &covered, colour);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The record of where the icons came from.
    const WHERE_THEY_CAME_FROM: &str = include_str!("../icons/settings/where-they-came-from.toml");

    /// **Every icon is the design file's own, read whole**, recorded with the
    /// node it came from and placed as that record says.
    #[test]
    fn every_icon_reads_whole_and_is_placed_as_recorded() {
        let record: toml::Table = WHERE_THEY_CAME_FROM
            .parse()
            .unwrap_or_else(|why| unreachable!("the record reads: {why}"));
        let icons = record
            .get("icon")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default();
        for icon in ASettingsIcon::ALL {
            assert!(icon.shape().is_some(), "{} does not read", icon.file_name());
            let Some(recorded) = icons
                .iter()
                .find(|it| it.get("svg").and_then(toml::Value::as_str) == Some(icon.file_name()))
            else {
                unreachable!("{} is not recorded", icon.file_name());
            };
            let (side, group) = icon.placed();
            let frame = recorded
                .get("frame")
                .and_then(toml::Value::as_integer)
                .unwrap_or_default();
            assert_eq!(frame as f32, side, "{}", icon.file_name());
            let written: Vec<f32> = recorded
                .get("group")
                .and_then(toml::Value::as_array)
                .map(|it| {
                    it.iter()
                        .filter_map(toml::Value::as_float)
                        .map(|it| it as f32)
                        .collect()
                })
                .unwrap_or_default();
            assert_eq!(written, group.to_vec(), "{}", icon.file_name());
            // And the drawing fits its frame, a stroke's width either side.
            let Some(shape) = icon.shape() else { continue };
            let (across, down) = shape.its_box();
            assert!(across <= side && down <= side, "{}", icon.file_name());
        }
    }

    /// **An icon is drawn inside its frame**, in the colour asked for.
    #[test]
    fn an_icon_is_drawn_inside_its_frame() {
        use smithay::utils::Rectangle;
        let mut canvas = Canvas::over(Rectangle::new((0, 0).into(), (40, 40).into()));
        let frame = Rectangle::new((10, 10).into(), (20, 20).into());
        ASettingsIcon::Globe.drawn(&mut canvas, frame, [200, 0, 0]);
        let inked = (0..40)
            .flat_map(|y| (0..40).map(move |x| (x, y)))
            .filter(|(x, y)| canvas.at(*x, *y).is_some_and(|it| it[3] > 0))
            .collect::<Vec<_>>();
        assert!(!inked.is_empty());
        assert!(
            inked
                .iter()
                .all(|(x, y)| (10..30).contains(x) && (10..30).contains(y)),
            "{inked:?}"
        );
    }
}
