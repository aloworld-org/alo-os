//! Opaque CPU composition of bounded native shapes and text onto a lock texture.
use crate::painted::{Inked, Solid};

/// Composite bounded shapes first, then opaque text boxes, preserving alpha.
pub(crate) fn overlay(into: &mut [u8], size: (i32, i32), solids: &[Solid], inked: &[Inked]) {
    for solid in solids {
        let a = solid.area;
        for y in a.loc.y.max(0)..(a.loc.y + a.size.h).min(size.1) {
            for x in a.loc.x.max(0)..(a.loc.x + a.size.w).min(size.0) {
                let n = ((y * size.0 + x) as usize) * 4;
                if let Some(pixel) = into.get_mut(n..n + 3) {
                    pixel.copy_from_slice(&solid.colour);
                }
            }
        }
    }
    for ink in inked {
        let a = ink.area;
        if a.size.w <= 0 {
            continue;
        }
        for (n, p) in ink.pixels.iter().enumerate() {
            let x = a.loc.x + (n % a.size.w as usize) as i32;
            let y = a.loc.y + (n / a.size.w as usize) as i32;
            if x >= 0 && y >= 0 && x < size.0 && y < size.1 {
                let at = ((y * size.0 + x) as usize) * 4;
                if let Some(pixel) = into.get_mut(at..at + 3) {
                    pixel.copy_from_slice(p);
                }
            }
        }
    }
}
