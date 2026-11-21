//! The bar is one declared setting.

use super::{THE_BAR, TheBar};

/// **Today's bar** (the owner, 2026-11-17 and 2026-11-20): ten tries, nine on
/// the road. Law 9 as written is every try of twenty, which is one line:
/// `TheBar::of(20, 20)`.
#[test]
fn the_bar_is_nine_of_ten_for_now() {
    assert_eq!((THE_BAR.tries(), THE_BAR.at_least()), (10, 9));
}

#[test]
fn the_bar_holds_in_proportion_and_all_means_all() {
    let bar = TheBar::of(10, 9);
    assert!(bar.holds(9, 10, false));
    assert!(!bar.holds(8, 10, false));
    assert!(bar.holds(18, 20, false));
    assert!(!bar.holds(17, 20, false));
    assert!(bar.holds(10, 10, true));
    assert!(
        !bar.holds(9, 10, true),
        "a request that reached all must keep all"
    );
    let law = TheBar::of(20, 20);
    assert!(law.holds(20, 20, false));
    assert!(!law.holds(19, 20, false));
}
