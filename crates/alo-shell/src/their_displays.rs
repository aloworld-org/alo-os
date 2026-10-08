//! **What a person has settled about their displays**, handed to the shell by
//! the desktop — `docs/autonomy/more-than-one-display-plan.md` task 10.
//!
//! # The road task 3a declared was not travelable
//!
//! `TheDesktop::the_screens_of` took a `Vec<alo_displays::Reported>` and
//! answered with a [`crate::Screens`], and **`alo-desktop` has no
//! `alo-displays` dependency** — so the one crate that must implement it could
//! not name its argument or its return type. The default answered [`None`],
//! every running machine took the single-display fallback, and five tasks of
//! that plan were unreachable in production while reading as done.
//!
//! # So the shell builds the arrangement and asks for what it may not read
//!
//! Three things are the **person's**, kept in their folder, and a compositor
//! that opened that file would be a compositor measuring:
//!
//! - their [`Appearance`](alo_appearance::Appearance),
//! - their night light, resolved against a clock the desktop owns, as a
//!   [`Tonight`](alo_displays::Tonight),
//! - the arrangements they have kept, as [`Changes`](alo_displays::Changes).
//!
//! Two things are **facts about the machine**, which only the compositor
//! learns: which displays are present and how large each is, and
//! [`THE_SIZES_THIS_COMPOSITOR_DRAWS`].
//!
//! So the desktop hands over the first three and the shell supplies the last
//! two. The same division the canvas layout and the shortcuts already travel,
//! and **no new crate edge**: the types a desktop needs are re-exported from
//! this crate, which it already depends on.

use alo_displays::Support;

/// Which display sizes this compositor can actually draw.
///
/// `Support` is documented in `alo-displays` as *asked of the compositor,
/// because it is a fact about the machine rather than a decision this crate
/// makes* — so it is answered here, where the drawing happens, rather than by
/// whoever happens to be calling.
///
/// **Whole multiples only, and it is the strict reading of two honest ones.**
///
/// The loose reading is `Fractional`: this shell does draw a fractional scale
/// for its own furniture, and `division_raster`'s
/// `a_fractional_scale_draws_something_other_than_one_to_one` proves it.
///
/// The strict reading is this one, and it is about what an **application**
/// gets. `presentation.rs` advertises every output to every client as
/// `Scale::Integer(1)`, and `wp_fractional_scale_v1` is advertised nowhere, so
/// a client never renders natively at a fractional size — a fractional scale
/// would be an upscale of a one-to-one render rather than a sharp one.
///
/// **The strict one is chosen because `SHARED_MAIN.md` says to**: *where a flag
/// chooses between the strict and the loose reading, the strict one is the
/// default — a flag that fails open is a flag that will be forgotten in the
/// direction that matters.* And it is the kinder answer in practice: rounding
/// 150 % up to 200 % is **said** to the person, by `Note::SizeRounded`, where a
/// silently blurry 150 % is not.
///
/// **What would change this:** advertising `wp_fractional_scale_v1`, at which
/// point a client can render at the size it is told and `Fractional` becomes
/// the true answer. That is not this task.
pub const THE_SIZES_THIS_COMPOSITOR_DRAWS: Support = Support::WholeMultiplesOnly;

/// What one person has settled about their own displays.
///
/// Every field is a **reading** the shell may not perform: two settings out of
/// their folder and one resolution of a clock. A desktop that has none of them
/// answers [`None`] from
/// [`TheDesktop::their_displays`](crate::TheDesktop::their_displays) and every
/// surface lays out against one viewport exactly as before.
#[derive(Debug, Clone)]
pub struct TheirDisplays {
    /// How this person wants things to look.
    pub appearance: alo_appearance::Appearance,
    /// Their night light, already resolved against the desktop's own clock.
    ///
    /// **Resolved there rather than here**, because reaching a `Tonight` from a
    /// `NightLight` needs a [`Moment`](alo_displays::Moment) and a moment needs
    /// a clock and a time zone. `alo-desktop` holds both; this crate holds
    /// neither and must not.
    pub tonight: alo_displays::Tonight,
    /// The arrangements this person has kept, for every set of screens they
    /// have arranged.
    ///
    /// [`Changes::untouched`](alo_displays::Changes::untouched) is the honest
    /// answer for a person who has arranged nothing, and it is literally true
    /// on 2026-10-08 because nothing in production writes an arrangement yet.
    /// **Plan task 12 is a test that fails when that stops being true**, so
    /// that *they have changed nothing* cannot quietly become a lie.
    pub remembered: alo_displays::Changes,
}

impl TheirDisplays {
    /// The screens these displays make, laid out as this person arranged them.
    ///
    /// [`None`] when nothing is plugged in, which is a machine with nothing to
    /// draw on rather than an arrangement — `Attached::now` calls that
    /// `NotArranged::NoScreens` and it is not a fault to report.
    ///
    /// **A refusal about one display does not lose the others.** `Reported::of`
    /// has already dropped any display it could not describe, before this is
    /// reached.
    #[must_use]
    pub fn the_screens_of(&self, reported: Vec<alo_displays::Reported>) -> Option<crate::Screens> {
        let attached = alo_displays::Attached::now(
            reported,
            &self.remembered,
            THE_SIZES_THIS_COMPOSITOR_DRAWS,
        )
        .ok()?;
        Some(crate::Screens::of(
            attached,
            &self.appearance,
            &self.tonight,
        ))
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "their_displays_tests.rs"]
mod tests;
