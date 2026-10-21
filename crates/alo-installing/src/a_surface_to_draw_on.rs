//! A surface the environment can put pixels on, with no ioctl and no `unsafe`.
//!
//! [ADR 0128](../../../docs/decisions/0128-whether-a-wi-fi-password-may-be-written-to-the-staging-partition.md)
//! gave this environment screens of its own, and
//! `docs/design/figma-snapshot/ROOTS.md`'s `370:30206` has required one since
//! it was drawn. Item 2 of `docs/autonomy/the-environment-in-full-plan.md`.
//!
//! # Why the framebuffer and not KMS
//!
//! The kernel the environment boots has `CONFIG_DRM_SIMPLEDRM=y` and
//! `CONFIG_DRM_FBDEV_EMULATION=y`, both built in — so a DRM device appears from
//! the firmware's own framebuffer, and `/dev/fb0` appears beside it. Measured
//! out of the image's own kernel config on 2026-10-20.
//!
//! Going through `/dev/fb0` rather than `DRM_IOCTL_MODE_SETCRTC` buys three
//! things that matter more here than anywhere else in this product:
//!
//! - **no mode setting, which is the point rather than a limitation.** The
//!   firmware has already set a mode; that mode is what the person is looking
//!   at, and taking it is strictly safer than choosing another and being wrong
//!   about what the panel can do. A black screen at this moment is worse than
//!   an unfashionable resolution;
//! - **no hand-transcribed kernel structs.** `drm_mode_crtc` and friends are an
//!   ABI to get wrong in a program that is statically linked against musl and
//!   has no `libdrm` to borrow one from. There is no struct here at all;
//! - **no `unsafe` and no `mmap`.** fbdev supports ordinary writes at an
//!   offset, so this is `std::os::unix::fs::FileExt::write_all_at` and nothing
//!   else.
//!
//! What it gives up is a page flip, so a screen is drawn in place rather than
//! swapped. For six static screens with a progress bar that is a trade worth
//! making; for anything that animates it would not be.
//!
//! # The first thing drawn is grey, deliberately
//!
//! Nothing here can know the channel order. `/sys/class/graphics/fb0/` reports
//! the geometry and the depth but **not** which byte is red — that is
//! `FBIOGET_VSCREENINFO`, which is a 160-byte struct and the one thing this
//! file is avoiding. On x86 UEFI the firmware's framebuffer is `XRGB8888`,
//! which in memory is blue, green, red, unused — but *almost always* is not
//! measured, and this file does not pretend otherwise.
//!
//! So [`Grey`] exists: **a colour whose channels are equal cannot be wrong
//! about their order.** The first screen a photograph proves is grey, and the
//! order is confirmed by a coloured one afterwards on a machine. That is the
//! cheapest honest first step, rather than drawing something that might come up
//! with the red and blue exchanged and being unable to tell whether the surface
//! or the order was at fault.
//!
//! # What is measured and what is not
//!
//! **The parsers and the row arithmetic are tested; nothing has been written to
//! a framebuffer.** This build machine has no framebuffer and no DRM device at
//! all. Item 2 of the plan says what finishes it: *a solid colour fills a real
//! screen from the environment* — and until somebody has a photograph, this is
//! not done whatever these tests say.

#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::io;
#[cfg(unix)]
use std::os::unix::fs::FileExt as _;

/// Where the kernel says how large the framebuffer is.
pub const WHERE_THE_SIZE_IS: &str = "/sys/class/graphics/fb0/virtual_size";

/// Where it says how many bytes one row takes.
///
/// **Not the width times the depth.** A framebuffer's rows are padded to suit
/// the hardware, so a 1366-pixel row is commonly 5504 bytes rather than 5464,
/// and writing the narrower number puts every row after the first one a little
/// further left than the last — a picture that shears across the screen.
pub const WHERE_THE_STRIDE_IS: &str = "/sys/class/graphics/fb0/stride";

/// Where it says how many bits one pixel takes.
pub const WHERE_THE_DEPTH_IS: &str = "/sys/class/graphics/fb0/bits_per_pixel";

/// The framebuffer itself.
pub const WHERE_THE_PIXELS_GO: &str = "/dev/fb0";

/// A grey, as a value every channel takes.
///
/// **A colour that cannot be wrong about channel order**, which is why the
/// first thing drawn is one. See this module's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grey(u8);

impl Grey {
    /// This much of every channel.
    #[must_use]
    pub const fn of(value: u8) -> Self {
        Self(value)
    }

    /// This grey as the value every channel takes.
    ///
    /// What anything compositing onto the surface needs: a grey is one value
    /// repeated, so blending two of them is arithmetic on these.
    #[must_use]
    pub const fn as_a_value(self) -> u8 {
        self.0
    }

    /// Dark enough to be plainly deliberate and light enough to be plainly
    /// lit — so that a photograph of it cannot be confused with a screen that
    /// is off.
    #[must_use]
    pub const fn plainly_drawn() -> Self {
        Self(0x40)
    }
}

/// What could not be made of the surface.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NoSurface {
    /// A file the kernel publishes could not be read as what it publishes.
    #[error("{at} does not say what it was expected to")]
    NotWhatItSays {
        /// Which file.
        at: &'static str,
    },
    /// A depth this cannot put a pixel into.
    ///
    /// Not a refusal of the machine but of the guess: at 16 bits a pixel the
    /// channels are packed 5-6-5 and a byte per channel is simply wrong, so
    /// writing anything would draw colours nobody asked for.
    #[error("this screen is {bits} bits a pixel, and only 32 is understood")]
    ADepthNotUnderstood {
        /// What it said.
        bits: u32,
    },
}

/// How large the surface is, and how its rows are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheSurface {
    /// Pixels across.
    across: u32,
    /// Pixels down.
    down: u32,
    /// Bytes from the start of one row to the start of the next.
    stride: u32,
    /// Bytes one pixel takes.
    each_pixel: u32,
}

impl TheSurface {
    /// Read it from the three things the kernel publishes.
    ///
    /// # Errors
    ///
    /// [`NoSurface`] for a file that does not say what it publishes, or a depth
    /// this cannot put a pixel into.
    pub fn from_what_the_kernel_says(
        virtual_size: &str,
        stride: &str,
        bits_per_pixel: &str,
    ) -> Result<Self, NoSurface> {
        let (across, down) =
            virtual_size
                .trim()
                .split_once(',')
                .ok_or(NoSurface::NotWhatItSays {
                    at: WHERE_THE_SIZE_IS,
                })?;
        let across = across
            .trim()
            .parse::<u32>()
            .map_err(|_| NoSurface::NotWhatItSays {
                at: WHERE_THE_SIZE_IS,
            })?;
        let down = down
            .trim()
            .parse::<u32>()
            .map_err(|_| NoSurface::NotWhatItSays {
                at: WHERE_THE_SIZE_IS,
            })?;
        let stride = stride
            .trim()
            .parse::<u32>()
            .map_err(|_| NoSurface::NotWhatItSays {
                at: WHERE_THE_STRIDE_IS,
            })?;
        let bits = bits_per_pixel
            .trim()
            .parse::<u32>()
            .map_err(|_| NoSurface::NotWhatItSays {
                at: WHERE_THE_DEPTH_IS,
            })?;
        if bits != 32 {
            return Err(NoSurface::ADepthNotUnderstood { bits });
        }
        // A surface with no pixels, or rows too short to hold the pixels they
        // are said to have, is not a surface — and both come back as the file
        // not saying what it says rather than as a drawing that goes wrong
        // later.
        let each_pixel = bits / 8;
        if across == 0 || down == 0 {
            return Err(NoSurface::NotWhatItSays {
                at: WHERE_THE_SIZE_IS,
            });
        }
        if stride < across * each_pixel {
            return Err(NoSurface::NotWhatItSays {
                at: WHERE_THE_STRIDE_IS,
            });
        }
        Ok(Self {
            across,
            down,
            stride,
            each_pixel,
        })
    }

    /// Pixels across.
    #[must_use]
    pub const fn across(self) -> u32 {
        self.across
    }

    /// Pixels down.
    #[must_use]
    pub const fn down(self) -> u32 {
        self.down
    }

    /// Bytes one pixel takes.
    ///
    /// Needed by anything writing single pixels rather than whole rows of one
    /// colour, which is what drawing a letter is.
    #[must_use]
    pub const fn each_pixel(self) -> u32 {
        self.each_pixel
    }

    /// One row of this grey, as the bytes a framebuffer row holds.
    ///
    /// **The row is the stride long, not the width long.** The padding at the
    /// end of a row is written too, with the same value — so a screen filled
    /// this way has no unwritten gap down its right-hand edge holding whatever
    /// the firmware left there.
    #[must_use]
    pub fn a_row_of(self, grey: Grey) -> Vec<u8> {
        vec![grey.0; self.stride as usize]
    }

    /// Where row `down` begins, in bytes from the start of the framebuffer.
    #[must_use]
    pub const fn where_row_begins(self, down: u32) -> u64 {
        down as u64 * self.stride as u64
    }

    /// How many bytes the whole surface is.
    #[must_use]
    pub const fn altogether(self) -> u64 {
        self.down as u64 * self.stride as u64
    }
}

/// Fill the whole screen with one grey.
///
/// The one function here that touches the machine, kept to one job for that
/// reason: everything it needs was worked out by the tested code above it.
///
/// # Errors
///
/// Whatever the write failed with. A framebuffer that is not there, or is not
/// writable by this program, arrives here rather than as a wrong picture.
///
/// # Why this one function is gated and the rest of the file is not
///
/// `FileExt::write_all_at` is Unix's, and this crate **must cross-compile for
/// Windows** — `alo-installer`'s tests pull it in, so the gate's Windows step
/// builds it. Everything above is arithmetic and parsing and is portable; only
/// the write is not, so only the write is gated.
///
/// *This was not gated when it was written, and the gate step added in #853
/// caught it — on the author's own branch, in the hour after he skipped
/// running that step locally on three changes in a row.*
#[cfg(unix)]
pub fn fill(at: &File, surface: TheSurface, grey: Grey) -> io::Result<()> {
    let row = surface.a_row_of(grey);
    for down in 0..surface.down() {
        at.write_all_at(&row, surface.where_row_begins(down))?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// What a 1366-wide panel really publishes, padding and all.
    ///
    /// **Not invented numbers**: 1366 pixels at four bytes is 5464, and the
    /// stride is 5504 because rows are padded to 64 bytes. That gap is the
    /// whole reason `stride` is read rather than multiplied, and this fixture
    /// is the case that would shear.
    const A_PADDED_PANEL: (&str, &str, &str) = ("1366,768\n", "5504\n", "32\n");

    /// **A surface is read, and the padding is kept.**
    #[test]
    fn a_padded_panel_is_read_and_its_padding_kept() {
        let (size, stride, depth) = A_PADDED_PANEL;
        let surface = TheSurface::from_what_the_kernel_says(size, stride, depth)
            .expect("a panel that publishes all three is a surface");
        assert_eq!((surface.across(), surface.down()), (1366, 768));
        // The row is the stride, not the width times four — which is the one
        // number a reader would guess and the one that shears the picture.
        assert_eq!(surface.a_row_of(Grey::of(0x40)).len(), 5504);
        assert_ne!(surface.a_row_of(Grey::of(0x40)).len(), 1366 * 4);
        assert_eq!(surface.altogether(), 5504 * 768);
    }

    /// **Rows begin a stride apart**, so row 1 starts after row 0's padding
    /// rather than after its pixels.
    #[test]
    fn rows_begin_a_stride_apart() {
        let (size, stride, depth) = A_PADDED_PANEL;
        let surface =
            TheSurface::from_what_the_kernel_says(size, stride, depth).expect("a surface");
        assert_eq!(surface.where_row_begins(0), 0);
        assert_eq!(surface.where_row_begins(1), 5504);
        assert_eq!(surface.where_row_begins(767), 5504 * 767);
        // And the last row ends exactly at the end.
        assert_eq!(
            surface.where_row_begins(767) + 5504_u64,
            surface.altogether()
        );
    }

    /// **An unpadded panel is the ordinary case and still right.**
    #[test]
    fn an_unpadded_panel_is_read() {
        let surface = TheSurface::from_what_the_kernel_says("1920,1080\n", "7680\n", "32\n")
            .expect("a surface");
        assert_eq!(surface.a_row_of(Grey::of(0)).len(), 1920 * 4);
        assert_eq!(surface.altogether(), 7680 * 1080);
    }

    /// **A grey is one value in every byte of the row**, which is what makes
    /// it unable to be wrong about channel order.
    #[test]
    fn a_grey_is_the_same_in_every_channel() {
        let surface =
            TheSurface::from_what_the_kernel_says("8,2\n", "32\n", "32\n").expect("a surface");
        let row = surface.a_row_of(Grey::plainly_drawn());
        assert_eq!(row.len(), 32);
        assert!(row.iter().all(|byte| *byte == 0x40), "{row:?}");
    }

    /// **A depth that is not 32 is refused rather than drawn wrong.**
    ///
    /// At 16 bits the channels are packed 5-6-5 and a byte per channel is
    /// simply wrong, so writing anything would draw colours nobody asked for.
    #[test]
    fn a_depth_that_is_not_understood_is_refused() {
        for bits in ["16\n", "24\n", "8\n", "0\n"] {
            let refused = TheSurface::from_what_the_kernel_says("1920,1080\n", "7680\n", bits)
                .expect_err("only 32 is understood");
            assert!(
                matches!(refused, NoSurface::ADepthNotUnderstood { .. }),
                "{bits:?} gave {refused:?}"
            );
        }
    }

    /// **A stride too short for the row it describes is refused**, because
    /// drawing into it would run every row into the one above.
    #[test]
    fn a_stride_too_short_is_refused() {
        let refused = TheSurface::from_what_the_kernel_says("1920,1080\n", "4096\n", "32\n")
            .expect_err("a row cannot be shorter than its pixels");
        assert_eq!(
            refused,
            NoSurface::NotWhatItSays {
                at: WHERE_THE_STRIDE_IS
            }
        );
        // The control: the same panel with an honest stride is a surface, so
        // the refusal is about the stride and not about the rest of it.
        assert!(
            TheSurface::from_what_the_kernel_says("1920,1080\n", "7680\n", "32\n").is_ok(),
            "the control"
        );
    }

    /// **A surface with no pixels is not a surface.**
    #[test]
    fn a_surface_with_no_pixels_is_refused() {
        for size in ["0,1080\n", "1920,0\n", "0,0\n"] {
            let refused = TheSurface::from_what_the_kernel_says(size, "7680\n", "32\n")
                .expect_err("a surface needs pixels");
            assert_eq!(
                refused,
                NoSurface::NotWhatItSays {
                    at: WHERE_THE_SIZE_IS
                },
                "{size:?}"
            );
        }
    }

    /// **Nothing that is not what the kernel publishes becomes a surface**, and
    /// each refusal names the file it was reading.
    #[test]
    fn nothing_that_is_not_published_becomes_a_surface() {
        for (size, stride, depth, at) in [
            ("", "7680\n", "32\n", WHERE_THE_SIZE_IS),
            ("1920\n", "7680\n", "32\n", WHERE_THE_SIZE_IS),
            ("1920,\n", "7680\n", "32\n", WHERE_THE_SIZE_IS),
            ("wide,tall\n", "7680\n", "32\n", WHERE_THE_SIZE_IS),
            ("1920,1080\n", "", "32\n", WHERE_THE_STRIDE_IS),
            ("1920,1080\n", "lots\n", "32\n", WHERE_THE_STRIDE_IS),
            ("1920,1080\n", "7680\n", "deep\n", WHERE_THE_DEPTH_IS),
            ("1920,1080\n", "7680\n", "", WHERE_THE_DEPTH_IS),
        ] {
            let refused = TheSurface::from_what_the_kernel_says(size, stride, depth)
                .expect_err("this is not a surface");
            assert_eq!(
                refused,
                NoSurface::NotWhatItSays { at },
                "{size:?} {stride:?} {depth:?}"
            );
        }
    }
}
