//! **Every surface the shell draws, held to the pixels it drew on `main` before
//! the UI kit moved anything** — task 1 of `docs/autonomy/the-alo-ui-kit-plan.md`.
//!
//! The kit moves the shell's drawing into crates of its own. A move is meant to
//! change nothing a person sees, and *meant to* is not a check, so each surface
//! below is drawn at three display scales (100, 150 and 200 per cent) and at two
//! text sizes (100 and 125 per cent), every picture is hashed with
//! [`crate::a_picture_hashed::Hashed`], and the hashes are held to a table
//! written into the surface's own file.
//!
//! **Where the tables came from.** Taken once, on 2026-11-11, by running these
//! tests against `main` at `4bf7722e` with nothing in the shell changed, and
//! reading the table each failure printed; the test runs on every build from
//! then on, in the gate's whole-workspace test step.
//!
//! **A picture that changes on purpose is a design change**, and the plan says
//! none is made by the kit. A lane whose change is meant to move a surface's
//! pixels re-takes that surface's table from the failure message, which prints
//! it ready to paste, and says so in its commit body. A lane that is moving code
//! and sees this fail has changed what a person sees.
//!
//! **Two surfaces take no text size**: the Bar lays itself out at
//! `TextScale::ordinary()` whatever the person chose (`crate::bar_raster`), and
//! the window band has no text size among its inputs. Each is held at three
//! scales and one text size, and its file says so.
//!
//! **What would make this pass while the desktop is broken:** a fixture that
//! drew nothing, or one that ignored the scale or the text size, would hash the
//! same at every setting and still match its table. So [`held`] also refuses a
//! table with two equal hashes in it.

#[path = "pinned_band.rs"]
mod band;
#[path = "pinned_bar.rs"]
mod bar;
#[path = "pinned_egress_status.rs"]
mod egress_status;
#[path = "pinned_lock.rs"]
mod lock;
#[path = "pinned_record.rs"]
mod record;
#[path = "pinned_settings.rs"]
mod settings;
#[path = "pinned_sign_in.rs"]
mod sign_in;
#[path = "pinned_task_window.rs"]
mod task_window;

use alo_appearance::TextScale;

use crate::display_scale::DisplayScale;

/// One picture of one surface, by what it was drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Drawn {
    /// The display's scale, per cent.
    scale: u16,
    /// The person's text size, per cent.
    text: u16,
    /// Everything the surface painted, hashed.
    hash: u64,
}

/// The display scales every surface is drawn at.
const SCALES: [u16; 3] = [100, 150, 200];

/// The text sizes every surface that takes one is drawn at.
///
/// **125 rather than 150**, measured on the first run: the egress status's
/// pill is laid out at the display's scale times the text size, so its picture
/// at text 150 on a display at 100 was the same picture as at text 100 on a
/// display at 150, and [`held`] rightly refused two equal hashes. At 125 no two
/// of the six products are equal.
const TEXT_SIZES: [u16; 2] = [100, 125];

/// The display every surface is drawn on, in logical pixels: the design's
/// frames are 1440 × 960.
const LOGICAL: (i32, i32) = (1440, 960);

/// `logical` on a display at `scale`, in that display's pixels.
fn in_pixels(logical: (i32, i32), scale: DisplayScale) -> (i32, i32) {
    let per_cent = i32::from(scale.per_cent());
    (logical.0 * per_cent / 100, logical.1 * per_cent / 100)
}

/// A surface drawn at every scale and every text size, in that order.
fn every_way(mut draw: impl FnMut(DisplayScale, TextScale) -> u64) -> Vec<Drawn> {
    let mut drawn = Vec::new();
    for scale in SCALES {
        for text in TEXT_SIZES {
            let hash = draw(
                DisplayScale::of_per_cent(scale),
                TextScale::percent(text)
                    .unwrap_or_else(|_| unreachable!("100 and 125 are text sizes")),
            );
            drawn.push(Drawn { scale, text, hash });
        }
    }
    drawn
}

/// A surface that takes no text size, drawn at every scale.
fn every_scale(mut draw: impl FnMut(DisplayScale) -> u64) -> Vec<Drawn> {
    SCALES
        .into_iter()
        .map(|scale| Drawn {
            scale,
            text: 100,
            hash: draw(DisplayScale::of_per_cent(scale)),
        })
        .collect()
}

/// **`now` is the table `before` holds**, and no two of its pictures are the
/// same. A failure prints the table as drawn now, ready to paste.
fn held(surface: &str, now: &[Drawn], before: &[Drawn]) {
    let table: String = now
        .iter()
        .map(|it| {
            let h = it.hash;
            format!(
                "    Drawn {{ scale: {}, text: {}, hash: 0x{:04x}_{:04x}_{:04x}_{:04x} }},\n",
                it.scale,
                it.text,
                h >> 48,
                (h >> 32) & 0xffff,
                (h >> 16) & 0xffff,
                h & 0xffff,
            )
        })
        .collect();
    let mut different: Vec<u64> = now.iter().map(|it| it.hash).collect();
    different.sort_unstable();
    different.dedup();
    assert_eq!(
        different.len(),
        now.len(),
        "two of {surface}'s pictures are the same, so a setting did not reach it:\n{table}"
    );
    assert_eq!(
        now, before,
        "{surface} no longer draws the pixels it drew before; as drawn now:\n{table}"
    );
}
