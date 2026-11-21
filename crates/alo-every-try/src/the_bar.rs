//! **The bar a change's own requests are held to**, declared once: how many
//! tries each is given at each level, and how many must take the road.
//!
//! Law 9 (`CLAUDE.md`) asks for **every** try to take the road. This is a
//! departure from it, decided by the owner and written down in ADR 0157:
//! nine of ten for now, because at ten tries a request that takes the road
//! nearly always still fails by chance one run in several; with a ratchet
//! (`crate::what_each_has_reached`) so that a request that once took every try
//! may not slip below it; and, once every request is reliably at ten, twenty
//! tries with all twenty required — the law as written. **That step is this
//! one line**: [`THE_BAR`] becomes `TheBar::of(20, 20)`.

/// How many tries, and how many of them must take the road.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheBar {
    /// Tries per own request, at each level.
    tries: usize,
    /// How many of those must take the road.
    at_least: usize,
}

/// The bar, today: ten tries, nine on the road.
pub const THE_BAR: TheBar = TheBar::of(10, 9);

/// **What "clearly worse" means** (the owner, 2026-11-20, *"Compare with
/// main"*): a drop of this many tries in every [`THE_BAR`]'s tries, below
/// the base, for a change that owns the whole set
/// (`crate::compared_with_its_base`) — two of ten, four of twenty.
pub const THE_CLEARLY_WORSE_DROP: usize = 2;

/// **How many more tries a flagged request is given**, on the change and on
/// its base, before it blocks: a fault on the first tries is a flag, and the
/// verdict is on the totals (approved 2026-11-20). Zero makes the first tries
/// the verdict.
pub const THE_CONFIRMING_TRIES: usize = 10;

impl TheBar {
    /// `at_least` of every `tries`.
    #[must_use]
    pub const fn of(tries: usize, at_least: usize) -> Self {
        Self { tries, at_least }
    }

    /// Tries per own request, at each level.
    #[must_use]
    pub const fn tries(self) -> usize {
        self.tries
    }

    /// How many of those must take the road.
    #[must_use]
    pub const fn at_least(self) -> usize {
        self.at_least
    }

    /// Whether `on_the_road` of `tried` holds: every one of them when the
    /// request has `reached_every_try` at this level, and otherwise at least
    /// this bar's share.
    #[must_use]
    pub const fn holds(self, on_the_road: usize, tried: usize, reached_every_try: bool) -> bool {
        if reached_every_try {
            on_the_road == tried
        } else {
            on_the_road.saturating_mul(self.tries) >= tried.saturating_mul(self.at_least)
        }
    }
}

#[cfg(test)]
#[path = "the_bar_tests.rs"]
mod tests;
