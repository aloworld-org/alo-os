//! Reuse unchanged panel rasters without retaining application content.

use std::{cell::RefCell, collections::VecDeque};

use crate::{
    dock_surface::{ADocksSurface, AGround, AWell, InPrimary},
    painted::Washed,
};
use smithay::utils::{Physical, Rectangle};

const MAX_ENTRIES: usize = 8;
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_SHAPES: usize = 256;

thread_local! {
    static PANELS: RefCell<Panels> = RefCell::new(Panels::default());
}

pub(super) fn surface(of: ADocksSurface<'_>) -> Option<Washed> {
    PANELS.with(|panels| panels.borrow_mut().draw(of, super::dock_surface::rasterize))
}

/// Scalar inputs plus owned copies of every borrowed shape. Comparing the
/// reconstructed input uses its derived equality, including appearance, scale,
/// clipping, focus and position. No hash collision can reuse the wrong panel.
struct Inputs {
    plain: ADocksSurface<'static>,
    badges: Vec<Rectangle<i32, Physical>>,
    marks: Vec<Rectangle<i32, Physical>>,
    wells: Vec<AWell>,
    rings: Vec<Rectangle<i32, Physical>>,
    primary_wells: Vec<AWell>,
    outlines: Vec<Rectangle<i32, Physical>>,
    grounds: Vec<AGround>,
}

impl Inputs {
    fn own(of: ADocksSurface<'_>) -> Self {
        Self {
            plain: ADocksSurface {
                badges: &[],
                marks: &[],
                wells: &[],
                rings: &[],
                in_primary: InPrimary {
                    wells: &[],
                    outlines: &[],
                    grounds: &[],
                    ..of.in_primary
                },
                ..of
            },
            badges: of.badges.to_vec(),
            marks: of.marks.to_vec(),
            wells: of.wells.to_vec(),
            rings: of.rings.to_vec(),
            primary_wells: of.in_primary.wells.to_vec(),
            outlines: of.in_primary.outlines.to_vec(),
            grounds: of.in_primary.grounds.to_vec(),
        }
    }

    fn borrowed(&self) -> ADocksSurface<'_> {
        ADocksSurface {
            badges: &self.badges,
            marks: &self.marks,
            wells: &self.wells,
            rings: &self.rings,
            in_primary: InPrimary {
                wells: &self.primary_wells,
                outlines: &self.outlines,
                grounds: &self.grounds,
                ..self.plain.in_primary
            },
            ..self.plain
        }
    }

    fn bytes(&self) -> usize {
        self.badges.capacity() * size_of::<Rectangle<i32, Physical>>()
            + self.marks.capacity() * size_of::<Rectangle<i32, Physical>>()
            + self.wells.capacity() * size_of::<AWell>()
            + self.rings.capacity() * size_of::<Rectangle<i32, Physical>>()
            + self.primary_wells.capacity() * size_of::<AWell>()
            + self.outlines.capacity() * size_of::<Rectangle<i32, Physical>>()
            + self.grounds.capacity() * size_of::<AGround>()
    }
}

struct Entry {
    input: Inputs,
    image: Washed,
    bytes: usize,
}

#[derive(Default)]
struct Panels {
    entries: VecDeque<Entry>,
    bytes: usize,
}

impl Panels {
    fn draw(
        &mut self,
        of: ADocksSurface<'_>,
        rasterize: impl FnOnce(ADocksSurface<'_>) -> Option<Washed>,
    ) -> Option<Washed> {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.input.borrowed() == of)
        {
            let entry = self.entries.remove(index)?;
            let image = entry.image.clone();
            self.entries.push_back(entry);
            return Some(image);
        }
        let image = rasterize(of)?;
        // Arbitrarily large panels/shape lists may still draw, but are never
        // retained. Both entry count and total owned vector storage are bounded.
        let shapes = [
            of.badges.len(),
            of.marks.len(),
            of.wells.len(),
            of.rings.len(),
            of.in_primary.wells.len(),
            of.in_primary.outlines.len(),
            of.in_primary.grounds.len(),
        ];
        if image.pixels.len() > MAX_BYTES || shapes.into_iter().any(|length| length > MAX_SHAPES) {
            return Some(image);
        }
        let input = Inputs::own(of);
        let stored = image.clone();
        let bytes = input.bytes() + stored.pixels.capacity();
        if bytes > MAX_BYTES {
            return Some(image);
        }
        while self.entries.len() >= MAX_ENTRIES || self.bytes + bytes > MAX_BYTES {
            let entry = self.entries.pop_front()?;
            self.bytes -= entry.bytes;
        }
        self.bytes += bytes;
        self.entries.push_back(Entry {
            input,
            image: stored,
            bytes,
        });
        Some(image)
    }
}

#[cfg(test)]
#[path = "dock_surface_cache_tests.rs"]
mod tests;
