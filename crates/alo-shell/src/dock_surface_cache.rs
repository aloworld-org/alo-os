//! Reuse unchanged panel rasters without retaining application content.

use std::{cell::RefCell, collections::VecDeque};

use crate::{
    dock_surface::{ADocksSurface, AGround, AWell, InPrimary},
    painted::Washed,
};
use smithay::utils::{Physical, Rectangle};

/// Limit entry metadata even when each panel is only a few pixels.
const MAX_ENTRIES: usize = 8;
/// Bound retained image and shape vectors; entry metadata is separately bounded.
const MAX_BYTES: usize = 16 * 1024 * 1024;
/// Refuse to retain an arbitrarily long borrowed list, even for a small image.
const MAX_SHAPES: usize = 256;

thread_local! {
    /// Each rendering thread owns its cache without a shared compositor lock.
    static PANELS: RefCell<Panels> = RefCell::new(Panels::default());
}

/// Draw a changed panel or return an independent copy of its unchanged raster.
pub(super) fn surface(of: ADocksSurface<'_>) -> Option<Washed> {
    PANELS.with(|panels| panels.borrow_mut().draw(of, super::dock_surface::rasterize))
}

/// Scalar inputs plus owned copies of every borrowed shape. Comparing the
/// reconstructed input uses its derived equality, including appearance, scale,
/// clipping, focus and position. No hash collision can reuse the wrong panel.
struct Inputs {
    /// Scalar inputs, with all borrowed shape lists replaced by empty slices.
    plain: ADocksSurface<'static>,
    /// Circular application badge bounds.
    badges: Vec<Rectangle<i32, Physical>>,
    /// Running and focused application marks.
    marks: Vec<Rectangle<i32, Physical>>,
    /// Rounded field and row bounds with their individual corners.
    wells: Vec<AWell>,
    /// Keyboard focus outlines in the panel's ring colour.
    rings: Vec<Rectangle<i32, Physical>>,
    /// Primary-answer grounds in the primary colour.
    primary_wells: Vec<AWell>,
    /// Primary-colour keyboard outlines.
    outlines: Vec<Rectangle<i32, Physical>>,
    /// Additional grounds carrying their own colours.
    grounds: Vec<AGround>,
}

impl Inputs {
    /// Own every borrowed shape; future borrowed fields cannot become static
    /// through the scalar struct update without an explicit ownership decision.
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

    /// Reconstruct the complete input for its derived field-by-field comparison.
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

    /// Heap storage owned by the bounded shape lists, excluding fixed metadata.
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

/// One unchanged panel and the exact visual inputs that produced it.
struct Entry {
    /// Owned visual key; contains no application text or captured content.
    input: Inputs,
    /// The pure panel raster, never handed to a caller mutably.
    image: Washed,
    /// Retained image and shape vector capacity for eviction accounting.
    bytes: usize,
}

/// Least-recently-used rasters with independent entry and heap-storage limits.
#[derive(Default)]
struct Panels {
    /// Oldest first; reuse moves an entry to the back.
    entries: VecDeque<Entry>,
    /// Sum of retained image and shape vector capacities.
    bytes: usize,
}

impl Panels {
    /// Reuse matching pixels; otherwise draw and evict before retaining a copy.
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
