//! That the three states answer the three questions consistently.

use super::WhatTheDockHasOpen;

/// An application, for a state that needs one.
fn app() -> alo_dock::AppId {
    #[expect(
        clippy::unwrap_used,
        reason = "in a test, a panic on an unexpected Err is the failure being reported"
    )]
    alo_dock::AppId::named("an.app").unwrap()
}

/// **Every state against every question**, as one table.
///
/// Three accessors over three states is nine answers, and the ones worth
/// holding are the pairs that must never both be true: nothing is ever both
/// *More apps* and a picker, and `is_anything` agrees with the other two rather
/// than being a fourth opinion.
#[test]
fn each_state_answers_every_question() {
    let cases = [
        (WhatTheDockHasOpen::Nothing, false, false, false),
        (WhatTheDockHasOpen::MoreApps, true, false, true),
        (WhatTheDockHasOpen::TheWindowsOf(app()), false, true, true),
    ];
    for (state, more_apps, a_picker, anything) in cases {
        assert_eq!(state.is_more_apps(), more_apps, "{state:?}: is_more_apps");
        assert_eq!(
            state.the_windows_of().is_some(),
            a_picker,
            "{state:?}: the_windows_of"
        );
        assert_eq!(state.is_anything(), anything, "{state:?}: is_anything");
        assert!(
            !(state.is_more_apps() && state.the_windows_of().is_some()),
            "{state:?}: both surfaces at once"
        );
        assert_eq!(
            state.is_anything(),
            state.is_more_apps() || state.the_windows_of().is_some(),
            "{state:?}: is_anything disagrees with the other two"
        );
    }
}

/// **A frame starts with neither open**, which is what every fixture and every
/// first draw relies on.
#[test]
fn the_default_is_nothing() {
    assert_eq!(WhatTheDockHasOpen::default(), WhatTheDockHasOpen::Nothing);
}

/// **The picker remembers which application**, so the list it opens is that
/// application's rather than whichever one the draw happens to reach first.
#[test]
fn a_picker_carries_the_application_it_is_for() {
    let open = WhatTheDockHasOpen::TheWindowsOf(app());
    assert_eq!(open.the_windows_of(), Some(&app()));
}
