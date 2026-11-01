//! A window's line, shadow and title strip, made once for a size and a look and
//! kept — so a frame moves and clips them rather than working out every pixel
//! of a shadow again.
//!
//! # Why
//!
//! Since the owner's answer of 2026-11-01 every application window is drawn
//! with a line round it, a shadow under it and a title strip above it
//! (`crate::a_window_raised`, `crate::the_title_strip_drawn`), and until this
//! file each of them was worked out pixel by pixel **on every frame**: a
//! distance and a smooth step for every pixel of a shadow up to 57 wide round
//! the whole window. That is the CPU's work, and alo OS promises that no
//! graphics card is required, so on a machine with nothing else it is the
//! frame's own time.
//!
//! # Why it can be kept
//!
//! Each raster depends on three things only: **the window's size, the
//! display's scale and the look** (focused or not). Where the window is moves
//! it; where the display ends clips it; neither changes a pixel's colour,
//! because every pixel is worked out from its distance to the window's own
//! edges. So each is made once with the window at the origin, kept under
//! those three, and per frame [`placed`] moves it and copies the rows the
//! display shows — the same pixels, measured by this file's tests.
//!
//! # How much is kept
//!
//! At most [`AT_MOST`] rasters. A window being resized makes a new size every
//! frame, and keeping each would be memory spent on sizes nobody will see
//! again, so when the list is full it is emptied and starts over: a resize
//! costs what every frame cost before, and a window at rest costs a copy.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;
use crate::window_edge_look::WhichLook;

/// How many kept rasters there may be before the list starts over: enough
/// for a dozen windows in both looks.
pub(crate) const AT_MOST: usize = 24;

/// Which of the two drawings a raster is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Which {
    /// The line round the window and the shadow under it.
    Round,
    /// The title strip.
    Strip,
}

/// What a raster is kept under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TheKey {
    /// Which drawing.
    pub(crate) which: Which,
    /// The size it was made for, in the display's pixels.
    pub(crate) size: (i32, i32),
    /// The display's scale, in thousandths, so it can be compared exactly.
    pub(crate) scale: u32,
    /// The look.
    pub(crate) look: WhichLook,
}

impl TheKey {
    /// The key for `which` at `size`, `scale` and `look`.
    pub(crate) fn of(which: Which, size: (i32, i32), scale: f64, look: WhichLook) -> Self {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a display's scale in thousandths is a small positive number"
        )]
        let scale = (scale * 1000.0).round().max(0.0) as u32;
        Self {
            which,
            size,
            scale,
            look,
        }
    }
}

/// The kept rasters.
type Kept = HashMap<TheKey, Arc<Vec<Washed>>>;

/// The one list, for the life of the shell.
fn the_list() -> &'static Mutex<Kept> {
    static KEPT: OnceLock<Mutex<Kept>> = OnceLock::new();
    KEPT.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The rasters kept under `key`, made by `make` the first time.
///
/// `make` draws them with the window at the origin. A list that cannot be
/// locked — another thread panicked holding it — is not a reason to draw
/// nothing: the rasters are made and handed back without being kept.
pub(crate) fn kept_or_made(key: TheKey, make: impl FnOnce() -> Vec<Washed>) -> Arc<Vec<Washed>> {
    let Ok(mut kept) = the_list().lock() else {
        return Arc::new(make());
    };
    if let Some(it) = kept.get(&key) {
        return Arc::clone(it);
    }
    if kept.len() >= AT_MOST {
        kept.clear();
    }
    let it = Arc::new(make());
    kept.insert(key, Arc::clone(&it));
    it
}

/// Rasters made at the origin, moved `by` and clipped to an output of
/// `output`: what a frame draws.
pub(crate) fn placed(kept: &[Washed], by: (i32, i32), output: (i32, i32)) -> Vec<Washed> {
    let screen = Rectangle::<i32, Physical>::from_size(output.into());
    kept.iter()
        .filter_map(|made| {
            let moved = Rectangle::<i32, Physical>::new(
                (made.area.loc.x + by.0, made.area.loc.y + by.1).into(),
                made.area.size,
            );
            let shown = moved.intersection(screen)?;
            if shown == moved {
                return Some(Washed {
                    area: moved,
                    pixels: made.pixels.clone(),
                });
            }
            let wide = usize::try_from(moved.size.w).ok()?;
            let from_x = usize::try_from(shown.loc.x - moved.loc.x).ok()?;
            let across = usize::try_from(shown.size.w).ok()?;
            let mut pixels = Vec::with_capacity(across * usize::try_from(shown.size.h).ok()? * 4);
            for y in shown.loc.y..shown.loc.y + shown.size.h {
                let row = usize::try_from(y - moved.loc.y).ok()?;
                let start = (row * wide + from_x) * 4;
                pixels.extend_from_slice(made.pixels.get(start..start + across * 4)?);
            }
            Some(Washed {
                area: shown,
                pixels,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "window_rasters_kept_tests.rs"]
mod tests;
