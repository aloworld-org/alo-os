//! **A first-start sheet laid out to fit the screen it is on**, by the
//! design's own rule for a short screen.
//!
//! Until 2026-10-30 a sheet whose blocks did not fit above *Accessibility* was
//! refused, and a refused sheet is a blank screen. On a 1024 × 768 display at
//! 100 per cent — and on 1366 × 768, the smallest screen alo OS lays its Dock
//! out for — the keyboard sheet was one: three keyboards, the test row, a
//! heading and an introduction need 730 down and the room ends at 684. A
//! person with that laptop could not choose a keyboard, and could not get past
//! the sheet to make an account.
//!
//! # Tighten, then scroll — and never shrink what a person reads
//!
//! `docs/design/the-first-start.md` (*Responsive · Height*) gives two
//! mechanisms, in this order, both measured on the design's 1280 × 720 frame:
//!
//! 1. **Tighten.** The heading comes down to 28 on 36 and the gaps close to 8
//!    (`crate::setup_sheet_layout::HowClose::Tightened`).
//! 2. **Scroll.** What will not fit scrolls, and the way on, the way back and
//!    *Accessibility* stay where they can be reached without scrolling.
//!
//! Here the second is **the list scrolling within its own area**: the sheet
//! shows fewer rows of its list at a time, the window follows the cursor
//! (`crate::a_window_onto_a_list`), and the arrow keys and the wheel move it.
//! The heading, the actions and *Accessibility* never move off the screen. The
//! cards stay 85 tall and the text in them stays 16 and 14, because the design
//! says a build *may tighten and it may scroll; it may not shrink what a person
//! reads.* How this differs from the frame the design draws — a whole column
//! scrolling in a 640-wide region — is recorded beside that rule.
//!
//! **A sheet drawn at its own gaps is drawn exactly as before.** Only a sheet
//! that did not fit is tightened, so every screen tall enough for the capture
//! is the picture it was, pixel for pixel.

use cosmic_text::FontSystem;

use crate::painted_text::{Shaped, sentence};
use crate::setup_sheet_in_pixels::{TheSheetIsLaidOut, laid_out_as, the_column_in_pixels};
use crate::setup_sheet_layout::{HowClose, TheTextIsTall};
use crate::sign_in_raster::Measure;
use crate::sign_in_surface::TheColours;

/// A sheet that fits: where everything goes, its heading as shaped for that,
/// and how many rows of its list there is room for.
pub(crate) struct ItFits {
    /// Where everything goes.
    pub(crate) laid: TheSheetIsLaidOut,
    /// The heading, shaped at the size the layout was made for.
    pub(crate) title: Shaped,
    /// How many choice rows the sheet has room for, at most what it asked.
    pub(crate) rows: usize,
}

/// **Lay a sheet out to fit**: as captured if it fits, else tightened with as
/// many of its `tall.choices` rows as fit, and never fewer than `fewest`.
///
/// `tall.title` is ignored: the heading is shaped here, because its size is
/// one of the things tightening changes. [`None`] where even the tightened
/// sheet with its fewest rows does not fit, which is a refusal rather than a
/// column drawn under *Accessibility*.
pub(crate) fn the_sheet_that_fits(
    fonts: &mut FontSystem,
    title: &str,
    colours: TheColours,
    measure: &Measure,
    size: (i32, i32),
    tall: TheTextIsTall,
    fewest: usize,
) -> Option<ItFits> {
    let display = measure.display();
    let column = the_column_in_pixels(size.0, display);
    let shaped = |fonts: &mut FontSystem, metrics| {
        sentence(fonts, title, column, metrics, colours.ground, colours.ink)
    };
    // **As captured, first**, so a screen with room is the picture it was.
    let as_captured = shaped(fonts, measure.title_metrics());
    if let Some(laid) = laid_out_as(
        size,
        TheTextIsTall {
            title: as_captured.height,
            ..tall
        },
        display,
        HowClose::AsCaptured,
    ) {
        return Some(ItFits {
            laid,
            title: as_captured,
            rows: tall.choices,
        });
    }
    let tightened = shaped(fonts, measure.tight_title_metrics());
    (fewest.min(tall.choices)..=tall.choices)
        .rev()
        .find_map(|rows| {
            laid_out_as(
                size,
                TheTextIsTall {
                    title: tightened.height,
                    choices: rows,
                    ..tall
                },
                display,
                HowClose::Tightened,
            )
            .map(|laid| (laid, rows))
        })
        .map(|(laid, rows)| ItFits {
            laid,
            title: tightened,
            rows,
        })
}
