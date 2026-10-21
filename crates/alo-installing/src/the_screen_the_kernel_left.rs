//! **Opening the screen the firmware left, which nothing did.**
//!
//! [`crate::TheSurface::from_what_the_kernel_says`] takes the three things the
//! kernel publishes **as strings** and turns them into a surface. Reading those
//! three files and opening the device was nobody's, so a surface could be
//! described and never drawn on.
//!
//! # Three faults a person would meet differently
//!
//! | | what it means |
//! |---|---|
//! | the sysfs files are not there | this machine has **no framebuffer**, and the environment cannot draw at all |
//! | they are there and `/dev/fb0` is not | the kernel describes a screen and gives nothing to write to |
//! | `/dev/fb0` is there and will not open | almost always **not running as root**, which is a different thing to tell somebody |
//!
//! They are separate variants because the third is the one that happens on a
//! developer's machine and the first is the one that happens on a machine with
//! no graphics at all. Collapsing them would report *no screen* to somebody
//! whose screen is fine.
//!
//! # The root is an argument, and joining an absolute path to it does nothing
//!
//! `a_surface_to_draw_on`'s constants are absolute, because that is where the
//! kernel puts them. **`Path::join` with an absolute path throws the root
//! away** — `root.join("/sys/class/graphics/fb0/stride")` is
//! `/sys/class/graphics/fb0/stride`, whatever `root` was — so the leading
//! separator is stripped here and a test holds that it is.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use crate::{
    NoSurface, TheSurface, WHERE_THE_DEPTH_IS, WHERE_THE_PIXELS_GO, WHERE_THE_SIZE_IS,
    WHERE_THE_STRIDE_IS,
};

/// Why there is no screen to draw on.
#[derive(Debug, thiserror::Error)]
pub enum NoScreen {
    /// A file the kernel would publish about a framebuffer is not there.
    #[error("this machine publishes no framebuffer: {at} could not be read")]
    TheKernelPublishesNothing {
        /// Which file.
        at: String,
    },
    /// The three values do not describe a surface this can draw on.
    #[error("the kernel describes a screen this cannot draw on: {0}")]
    NotASurface(#[source] NoSurface),
    /// The device the pixels go to is not there.
    #[error("the kernel describes a screen and {at} is not there to write it to")]
    NoDeviceToWriteTo {
        /// Which device.
        at: String,
    },
    /// The device is there and will not take a write.
    #[error(
        "{at} is there and will not open for writing: {why} — this program is almost certainly not running as root"
    )]
    ItWillNotTakeAWrite {
        /// Which device.
        at: String,
        /// What the system said.
        why: String,
    },
}

/// The screen, and the file its pixels go to.
///
/// Held together because neither is any use alone: the surface is the
/// arithmetic and the file is where it lands, and every drawing call in this
/// crate takes both.
#[derive(Debug)]
pub struct TheScreen {
    /// How big it is and how its rows are laid out.
    surface: TheSurface,
    /// Where the pixels go.
    at: File,
}

impl TheScreen {
    /// How big it is and how its rows are laid out.
    #[must_use]
    pub const fn surface(&self) -> TheSurface {
        self.surface
    }

    /// Where the pixels go.
    #[must_use]
    pub const fn at(&self) -> &File {
        &self.at
    }
}

/// Everything this reads, in the order it reads it.
///
/// Public so that a caller reporting *this machine has no framebuffer* can say
/// which file it looked for, and so a test can assert the set rather than
/// rediscover it.
pub const WHAT_THE_KERNEL_PUBLISHES: [&str; 3] =
    [WHERE_THE_SIZE_IS, WHERE_THE_STRIDE_IS, WHERE_THE_DEPTH_IS];

/// An absolute path the kernel publishes, under the root this is asked about.
///
/// **The leading separator is stripped**, because `Path::join` with an absolute
/// path discards what it is joined to.
fn under(root: &Path, absolute: &str) -> PathBuf {
    root.join(absolute.trim_start_matches('/'))
}

/// Open the screen the firmware left, under this root.
///
/// # Errors
///
/// [`NoScreen`], whose variants separate *no framebuffer at all* from *a
/// framebuffer described and no device* from *a device that will not open* —
/// which are three different things to tell somebody.
pub fn the_screen_the_kernel_left(root: &Path) -> Result<TheScreen, NoScreen> {
    let mut said: Vec<String> = Vec::with_capacity(WHAT_THE_KERNEL_PUBLISHES.len());
    for publishes in WHAT_THE_KERNEL_PUBLISHES {
        let at = under(root, publishes);
        let Ok(text) = std::fs::read_to_string(&at) else {
            return Err(NoScreen::TheKernelPublishesNothing {
                at: at.display().to_string(),
            });
        };
        said.push(text);
    }
    let [size, stride, depth] = said.as_slice() else {
        // Unreachable while the loop above pushes one per path, and written as
        // a refusal rather than an index so that it cannot become a panic.
        return Err(NoScreen::TheKernelPublishesNothing {
            at: root.display().to_string(),
        });
    };

    let surface = TheSurface::from_what_the_kernel_says(size, stride, depth)
        .map_err(NoScreen::NotASurface)?;

    let pixels = under(root, WHERE_THE_PIXELS_GO);
    if !pixels.exists() {
        return Err(NoScreen::NoDeviceToWriteTo {
            at: pixels.display().to_string(),
        });
    }
    let at = OpenOptions::new()
        .write(true)
        .open(&pixels)
        .map_err(|why| NoScreen::ItWillNotTakeAWrite {
            at: pixels.display().to_string(),
            why: why.to_string(),
        })?;

    Ok(TheScreen { surface, at })
}

#[cfg(test)]
#[path = "the_screen_the_kernel_left_tests.rs"]
mod tests;
