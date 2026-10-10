//! **Every stage the offscreen probe walks**, in one place because two sides
//! disagree about it otherwise.
//!
//! Task 17 of `docs/autonomy/the-shell-plan.md` asks that *the probe's stage
//! count matches the stages it walks, checked by walking them rather than by a
//! number somebody maintains.*
//!
//! # The number that was maintained, and what it cost
//!
//! `offscreen_check.rs` ended with `assert_eq!(stages, 24)`. Task 16 removed
//! `window_tiling` on 2026-09-26 and six stages went with it; **the number was
//! left at thirty and this probe was broken on `main` for a day.** Nothing caught
//! it, because no gate ran this example — it needs a Wayland parent, and until one
//! was found for the Mac lane on 2026-09-26 there was no machine here that could
//! run it at all. Somebody noticed the number, not the breakage.
//!
//! A count is the weakest thing that can be asserted about a walk. It passes when
//! a stage is sent twice and another skipped, it passes when two stages swap, and
//! it is maintained by hand in a file that has no other reason to change — so it
//! goes stale exactly when the walk does.
//!
//! # What is checked instead
//!
//! The **set**, and each stage exactly once. The client drives from [`EVERY_STAGE`]
//! and the checker requires that it saw precisely these, each one time. That
//! catches a stage sent twice, a stage skipped, a stage nobody drives any more and
//! a stage number invented in one file and not the other — none of which a count
//! can see. Nothing anywhere has to be kept in step by a person: delete a stage
//! from this list and both sides stop walking it together.
//!
//! # Why the numbers have gaps, and why they are kept
//!
//! They are not an order and never were: the checker matches on them, and a stage
//! number in a log a fortnight old still means what it meant. 23 to 28 are the gap
//! task 16 left, and 31 onward are task 17's own — the division between two real
//! clients that replaced them.

/// Every stage of the offscreen walk, in the order the client drives them.
///
/// **Order is the order they are sent**, which is not numeric order. It was 29
/// and 30 before 1, 2 and 3 - the scene matrices first, because those three
/// acknowledged them - and with the strip retired the order no longer has that
/// reason. It stays a list rather than a range because the division stages at
/// 31 to 34 still come first, and because 23 to 28 are a gap.
pub const EVERY_STAGE: [u8; 26] = [
    // **Task 17's own, and they come first.** A division needs two windows and the
    // refusal needs exactly one, so these three want a known window population —
    // and the cleanest one is the empty display this probe starts on. Threading
    // them into the gap at 23 would have meant reasoning about whether the script
    // had one window mapped or two at that point, with one of them minimised.
    33, 31, 32, 34, // Everything the probe already walked, unchanged.
    // **29 and 30 left on 2026-10-10 with the control strip.** They drew it in
    // each scheme and checked its pixels against a literal mask; there is no
    // strip to draw. The numbers are not reused, for the reason the gaps above
    // are kept - a stage number in an old log still means what it meant.
    1, 2, 3, 8, 6, 7, 4, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 5,
];

/// Whether this stage is one of task 17's division stages.
///
/// Named rather than compared against numbers where it is used, because *which
/// stages are the division's* is this file's business and a range written at the
/// call site would be a second place that has to agree with the list above.
#[must_use]
pub fn is_a_division_stage(stage: u8) -> bool {
    (31..=34).contains(&stage)
}
