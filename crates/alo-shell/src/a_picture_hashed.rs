//! One FNV-1a hash of everything a picture paints, for a test that holds a
//! surface to the bytes it drew before a change.
//!
//! **A test helper and nothing else**: compiled under `cfg(test)` only. The
//! Dock's own copy lives in `crate::dock_raster_scale_tests`, written first;
//! this is the same arithmetic for the surfaces that took their display's scale
//! after it — the Bar, the put-aside panel and the notification cards — so
//! three files do not each carry a fourth copy.

use smithay::utils::{Physical, Rectangle};

use crate::painted::{Inked, Washed};

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

    /// Where something is and how large.
    pub(crate) fn area(&mut self, at: Rectangle<i32, Physical>) {
        for n in [at.loc.x, at.loc.y, at.size.w, at.size.h] {
            self.eat(&n.to_le_bytes());
        }
    }

    /// A raster with its alpha, and where it goes.
    pub(crate) fn washed(&mut self, washed: &Washed) {
        self.area(washed.area);
        self.eat(&washed.pixels);
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
