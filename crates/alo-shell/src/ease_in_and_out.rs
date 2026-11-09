//! **Ease in and out**, the one curve alo's own motion follows.
//!
//! Shared rather than copied: the view's glide (`crate::the_view_glides`) and a
//! window travelling to the put-aside panel (`crate::a_window_travels_aside`)
//! both move this way, and two copies of a curve are two curves the first time
//! somebody tunes one of them. `docs/design/where-a-new-window-opens.md` names
//! it for the glide — *eased in and out* — and the travel follows the glide.

/// Ease in and out, cubic: slow at both ends, fastest in the middle.
///
/// `t` is the share of the time gone, and is clamped to `0..=1`, so a moment
/// before the start is the start and a moment after the end is the end.
#[must_use]
pub(crate) fn eased(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let back = -2.0 * t + 2.0;
        1.0 - back * back * back / 2.0
    }
}
