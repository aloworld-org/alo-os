//! **A framebuffer built as a file, drawn on, and read back.**
//!
//! This machine has no framebuffer — no `/dev/fb0`, no `/sys/class/graphics`
//! entry but `fbcon`, and no DRM device at all — so the screen is a file of the
//! right size. That is not a pretence: `fill` writes at byte offsets through
//! `write_all_at`, and a regular file takes those writes exactly as the device
//! does. **So the bytes can be read back and asserted**, which is as close to
//! *somebody has seen it* as this lane can get without hardware, and it proves
//! the arithmetic rather than the driver.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::{NoScreen, WHAT_THE_KERNEL_PUBLISHES, the_screen_the_kernel_left, under};
use crate::{
    Grey, NoSurface, WHERE_THE_DEPTH_IS, WHERE_THE_PIXELS_GO, WHERE_THE_SIZE_IS,
    WHERE_THE_STRIDE_IS,
};

/// A small screen, so the whole of it can be read back and checked.
const ACROSS: u32 = 8;
/// The same.
const DOWN: u32 = 4;
/// Deliberately **not** `ACROSS * 4`: a real framebuffer pads its rows, and a
/// row written at `across * each_pixel` instead of at the stride is the bug
/// that draws a diagonal.
const STRIDE: u32 = 48;

/// A machine publishing these three things, with a device of this size.
fn a_machine_with(size: &str, stride: &str, depth: &str, device: Option<u64>) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-screen-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    for (publishes, what) in [
        (WHERE_THE_SIZE_IS, size),
        (WHERE_THE_STRIDE_IS, stride),
        (WHERE_THE_DEPTH_IS, depth),
    ] {
        let its = under(&at, publishes);
        std::fs::create_dir_all(its.parent().expect("a file has a parent")).expect("a directory");
        std::fs::write(&its, what).expect("writing what the kernel publishes");
    }
    if let Some(bytes) = device {
        let pixels = under(&at, WHERE_THE_PIXELS_GO);
        std::fs::create_dir_all(pixels.parent().expect("a file has a parent"))
            .expect("a directory");
        std::fs::write(
            &pixels,
            vec![0u8; usize::try_from(bytes).expect("a small screen")],
        )
        .expect("writing a framebuffer of our own");
    }
    at
}

/// A machine like the one the environment expects to find.
fn an_ordinary_machine() -> PathBuf {
    a_machine_with(
        &format!("{ACROSS},{DOWN}"),
        &STRIDE.to_string(),
        "32",
        Some(u64::from(STRIDE) * u64::from(DOWN)),
    )
}

/// **The three values come back as the surface they describe.**
#[test]
fn the_screen_is_the_size_the_kernel_published() {
    let root = an_ordinary_machine();
    let screen = the_screen_the_kernel_left(&root).expect("a screen");
    assert_eq!(screen.surface().across(), ACROSS);
    assert_eq!(screen.surface().down(), DOWN);
    assert_eq!(screen.surface().each_pixel(), 4);
    // The stride, read back through where the rows begin rather than through an
    // accessor, because that is what every write uses.
    assert_eq!(screen.surface().where_row_begins(1), u64::from(STRIDE));
}

/// **Filling it puts those bytes in the file**, which is the whole chain from
/// three sysfs files to pixels.
///
/// `a_row_of` produces a row **`stride` bytes long**, not `across *
/// each_pixel`, so a fill covers the padding between rows as well. That is the
/// design and it is harmless on a real device, where the padding is inside the
/// mapped region and off the screen.
///
/// **So the discriminator is the file's length, not its padding.** The first
/// version of this test asserted the padding stayed zero and failed — correctly,
/// against the code rather than against a bug. If rows were placed at `across *
/// each_pixel` instead of at the stride, the last row would end at
/// `DOWN * ACROSS * 4` = 128 and the bytes from there to `DOWN * STRIDE` = 192
/// would never be written. Asserting **every** byte is the grey catches that,
/// and nothing weaker does.
#[cfg(unix)]
#[test]
fn filling_the_screen_puts_the_grey_in_every_byte_of_the_file() {
    let root = an_ordinary_machine();
    let screen = the_screen_the_kernel_left(&root).expect("a screen");
    let grey = Grey::of(0x40);
    let altogether = usize::try_from(u64::from(STRIDE) * u64::from(DOWN)).expect("a small screen");

    crate::fill(screen.at(), screen.surface(), grey).expect("filling a file of our own");

    let written = std::fs::read(under(&root, WHERE_THE_PIXELS_GO)).expect("the framebuffer back");
    assert_eq!(written.len(), altogether, "the file changed size");
    assert_eq!(
        written.iter().filter(|byte| **byte == 0x40).count(),
        altogether,
        "{} of {altogether} bytes hold the grey. If the figure is {}, the rows \
         were laid out at across*each_pixel rather than at the stride and the \
         last {} bytes were never written",
        written.iter().filter(|byte| **byte == 0x40).count(),
        ACROSS * 4 * DOWN,
        altogether - usize::try_from(ACROSS * 4 * DOWN).expect("small")
    );
    // And the last row really does begin where the stride says, which is the
    // same fact from the other side.
    assert_eq!(
        screen.surface().where_row_begins(DOWN - 1),
        u64::from(STRIDE) * u64::from(DOWN - 1)
    );
}

/// **No sysfs is *no framebuffer*, and it names the file it looked for.**
///
/// The machine this was written on is exactly this case.
#[test]
fn a_machine_with_no_framebuffer_says_so_and_names_what_it_looked_for() {
    let nowhere = Path::new("/this/machine/has/no/graphics");
    let why = the_screen_the_kernel_left(nowhere).expect_err("there is no screen there");
    let NoScreen::TheKernelPublishesNothing { at } = why else {
        panic!("a machine with no sysfs reported {why:?}")
    };
    assert!(
        at.ends_with(WHAT_THE_KERNEL_PUBLISHES[0].trim_start_matches('/')),
        "it named {at}, which is not the first thing it reads"
    );
}

/// **A described screen with no device is its own fault**, not *no
/// framebuffer*.
///
/// The kernel saying how big a screen is and giving nothing to write to is a
/// different machine from one with no graphics, and a person told the wrong one
/// would look in the wrong place.
#[test]
fn a_screen_described_with_no_device_is_a_different_fault() {
    let root = a_machine_with(&format!("{ACROSS},{DOWN}"), &STRIDE.to_string(), "32", None);
    let why = the_screen_the_kernel_left(&root).expect_err("there is no device");
    assert!(
        matches!(why, NoScreen::NoDeviceToWriteTo { .. }),
        "a missing device reported {why:?}"
    );
}

/// **A depth this cannot draw into is refused as that**, and carries the
/// surface's own reason.
#[test]
fn a_depth_that_cannot_hold_a_pixel_is_refused_as_that() {
    let root = a_machine_with(
        &format!("{ACROSS},{DOWN}"),
        &STRIDE.to_string(),
        "16",
        Some(64),
    );
    let why = the_screen_the_kernel_left(&root).expect_err("16 bits a pixel is refused");
    let NoScreen::NotASurface(NoSurface::ADepthNotUnderstood { bits }) = why else {
        panic!("a 16-bit screen reported {why:?}")
    };
    assert_eq!(bits, 16);
}

/// **Nonsense in a published file is refused before anything is opened.**
#[test]
fn a_published_file_that_does_not_say_what_it_should_is_refused() {
    for (size, stride, depth) in [
        ("not,a,size", "48", "32"),
        ("8", "48", "32"), // no comma: a size is `across,down`
        ("8,4", "wide", "32"),
        ("8,4", "48", "deep"),
        ("0,4", "48", "32"), // no pixels at all
    ] {
        let root = a_machine_with(size, stride, depth, Some(192));
        let why = the_screen_the_kernel_left(&root).expect_err("nonsense is refused");
        assert!(
            matches!(why, NoScreen::NotASurface(_)),
            "{size:?} {stride:?} {depth:?} reported {why:?}"
        );
    }
}

/// **The root is used, and joining an absolute path to it does not throw it
/// away.**
///
/// `Path::join` with an absolute path discards what it is joined to, so
/// `root.join("/sys/class/graphics/fb0/stride")` is the real machine's file.
/// Without the strip this whole module would read **the host's** framebuffer
/// while believing it read the fixture's — and on a machine that has one, the
/// tests above would pass for the wrong reason.
#[test]
fn an_absolute_path_under_a_root_stays_under_it() {
    let root = Path::new("/somewhere/of/our/own");
    let joined = under(root, WHERE_THE_STRIDE_IS);
    assert!(
        joined.starts_with(root),
        "{joined:?} escaped the root it was given"
    );
    assert!(joined.ends_with("sys/class/graphics/fb0/stride"));
    // And the trap itself, asserted so the reason is visible rather than
    // remembered: the naive join really does discard the root.
    assert_eq!(
        root.join(WHERE_THE_STRIDE_IS),
        Path::new(WHERE_THE_STRIDE_IS),
        "Path::join no longer discards an absolute path, so the strip may be \
         unnecessary — check before removing it"
    );
}
