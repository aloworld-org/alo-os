//! One FNV-1a hash of everything a picture paints, for a test that holds a
//! surface to the bytes it drew before a change.
//!
//! **A test helper and nothing else**: compiled under `cfg(test)` only. The
//! Dock's own copy lives in `crate::dock_raster_scale_tests`, written first;
//! this is the same arithmetic for the surfaces that took their display's scale
//! after it — the Bar, the put-aside panel and the notification cards — so
//! three files do not each carry a fourth copy.

use smithay::utils::{Physical, Rectangle};

use crate::painted::{Inked, Solid, Washed};

/// Something with a place and a size: `alo-drawing`'s area, or the
/// compositor's rectangle the shell still lays out in.
pub(crate) trait Placed {
    /// Left, top, width and height.
    fn corner_and_size(self) -> [i32; 4];
}

impl Placed for Rectangle<i32, Physical> {
    fn corner_and_size(self) -> [i32; 4] {
        [self.loc.x, self.loc.y, self.size.w, self.size.h]
    }
}

impl Placed for alo_drawing::area::Area {
    fn corner_and_size(self) -> [i32; 4] {
        [self.loc.x, self.loc.y, self.size.w, self.size.h]
    }
}

/// A running hash a test feeds a picture into, part by part.
pub(crate) struct Hashed {
    /// The hash so far.
    hash: u64,
}

impl Hashed {
    /// A hash of nothing yet.
    pub(crate) const fn begun() -> Self {
        Self {
            hash: 0xcbf2_9ce4_8422_2325,
        }
    }

    /// Some bytes.
    fn eat(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x0100_0000_01b3);
        }
    }

    /// Bytes as they are, for a picture that is one raster already: the lock
    /// screen's.
    pub(crate) fn raw(&mut self, bytes: &[u8]) {
        self.eat(bytes);
    }

    /// Where something is and how large, in either the drawing's area or the
    /// compositor's rectangle: the same four numbers, hashed the same way.
    pub(crate) fn area(&mut self, at: impl Placed) {
        for n in at.corner_and_size() {
            self.eat(&n.to_le_bytes());
        }
    }

    /// A raster with its alpha, and where it goes.
    pub(crate) fn washed(&mut self, washed: &Washed) {
        self.area(washed.area);
        self.eat(&washed.pixels);
    }

    /// Flat shapes, in painting order.
    pub(crate) fn solids(&mut self, solids: &[Solid]) {
        for solid in solids {
            self.area(solid.area);
            self.eat(&solid.colour);
        }
    }

    /// Words, in painting order.
    pub(crate) fn inked(&mut self, inked: &[Inked]) {
        for one in inked {
            self.area(one.area);
            for pixel in &one.pixels {
                self.eat(pixel);
            }
        }
    }

    /// The hash.
    pub(crate) const fn done(&self) -> u64 {
        self.hash
    }
}
