//! The record window's icons, from the design's own files
//! (`icons/the-record/where-they-came-from.toml`).
//!
//! **Each file's viewBox is the icon's whole frame**, so an icon is drawn by
//! stretching its file into the square the window gives it, at the display's
//! scale, in the colour the palette gives where it is used.

use std::sync::OnceLock;

use alo_vector_shapes::AShape;

use crate::settings_canvas::{Area, Canvas};

/// One of the record window's icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ARecordIcon {
    /// An entry that changed something of the person's.
    Changed,
    /// An entry where something left this computer.
    Sent,
    /// An entry that was refused.
    Refused,
    /// That the person said yes, and today's count of it.
    Allowed,
    /// *Refused*, beside its word and on today's count.
    RefusedSmall,
    /// Today's count of what left this computer.
    Left,
    /// Beside the agent's name at the head of an entry.
    Agent,
    /// The footer: kept on this computer.
    Padlock,
    /// The window's own icon, in its title strip.
    WhatAloDid,
    /// Close, at the strip's end.
    Close,
}

impl ARecordIcon {
    /// Every icon, in the order the provenance file lists them.
    pub(crate) const ALL: [Self; 10] = [
        Self::Changed,
        Self::Sent,
        Self::Refused,
        Self::Allowed,
        Self::RefusedSmall,
        Self::Left,
        Self::Agent,
        Self::Padlock,
        Self::WhatAloDid,
        Self::Close,
    ];

    /// The file, as the design exported it.
    const fn svg(self) -> &'static str {
        match self {
            Self::Changed => include_str!("../icons/the-record/changed.svg"),
            Self::Sent => include_str!("../icons/the-record/sent.svg"),
            Self::Refused => include_str!("../icons/the-record/refused.svg"),
            Self::Allowed => include_str!("../icons/the-record/allowed.svg"),
            Self::RefusedSmall => include_str!("../icons/the-record/refused-small.svg"),
            Self::Left => include_str!("../icons/the-record/left.svg"),
            Self::Agent => include_str!("../icons/the-record/agent.svg"),
            Self::Padlock => include_str!("../icons/the-record/kept.svg"),
            Self::WhatAloDid => include_str!("../icons/the-record/what-alo-did.svg"),
            Self::Close => include_str!("../icons/the-record/close.svg"),
        }
    }

    /// Its place in [`Self::ALL`].
    fn at(self) -> usize {
        Self::ALL
            .iter()
            .position(|it| *it == self)
            .unwrap_or_default()
    }

    /// The artwork, read once for the life of the shell; [`None`] for a file
    /// `alo_vector_shapes` refused, said once and refused by this file's tests.
    fn shape(self) -> Option<&'static AShape> {
        static READ: OnceLock<Vec<Option<AShape>>> = OnceLock::new();
        READ.get_or_init(|| {
            ARecordIcon::ALL
                .iter()
                .map(|icon| match AShape::read(icon.svg()) {
                    Ok(shape) => Some(shape),
                    Err(why) => {
                        eprintln!(
                            "alo-shell: a record window icon could not be read and is not drawn: {why}"
                        );
                        None
                    }
                })
                .collect()
        })
        .get(self.at())
        .and_then(Option::as_ref)
    }

    /// Draw this icon into `frame`, its square in the display's pixels, in
    /// `colour`.
    pub(crate) fn drawn(self, canvas: &mut Canvas, frame: Area, colour: [u8; 3]) {
        let Some(shape) = self.shape() else {
            return;
        };
        let (Ok(wide), Ok(tall)) = (u32::try_from(frame.size.w), u32::try_from(frame.size.h))
        else {
            return;
        };
        let covered = shape.coverage(wide, tall);
        canvas.artwork(frame.loc.x, frame.loc.y, frame.size.w, &covered, colour);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every icon is the design file's own, read whole, and recorded.**
    #[test]
    fn every_icon_reads_and_is_recorded_with_its_node() {
        let record = include_str!("../icons/the-record/where-they-came-from.toml");
        for icon in ARecordIcon::ALL {
            assert!(icon.shape().is_some(), "{icon:?} does not read");
        }
        for name in [
            "changed.svg",
            "sent.svg",
            "refused.svg",
            "allowed.svg",
            "refused-small.svg",
            "left.svg",
            "agent.svg",
            "kept.svg",
            "what-alo-did.svg",
            "close.svg",
        ] {
            assert!(record.contains(name), "{name} is not recorded");
        }
    }

    /// **Drawn into its square and no further.**
    #[test]
    fn an_icon_is_drawn_inside_its_square() {
        let area = Area::new((0, 0).into(), (40, 40).into());
        let mut canvas = Canvas::over(area);
        ARecordIcon::Close.drawn(
            &mut canvas,
            Area::new((10, 10).into(), (16, 16).into()),
            [0x10, 0x2A, 0x43],
        );
        let inked = |x: i32, y: i32| canvas.at(x, y).is_some_and(|it| it[3] > 0);
        assert!(inked(18, 18), "the cross is drawn at its middle");
        assert!(!inked(5, 5), "nothing outside the square");
        assert!(!inked(30, 30), "nothing outside the square");
    }
}
