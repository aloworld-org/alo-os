//! Who draws a header, from what the application said: each road held.

use super::*;
use crate::window_edge::ItsOwnButtons;

/// **A window that agreed on the protocol gets the shell's whole band**, as
/// Qt and KDE applications do (KCalc, measured agreeing to server-side).
#[test]
fn a_window_that_agreed_gets_the_shells_band() {
    assert_eq!(
        who_draws_the_header(true, Some("org.kde.dolphin")),
        Decorations::TheShellDraws
    );
    assert_eq!(who_draws_the_header(true, None), Decorations::TheShellDraws);
}

/// **A window that never asked, with a measured row, gets what its row says**:
/// Firefox's own Close and no Minimise.
#[test]
fn a_window_that_never_asked_is_told_apart_by_its_row() {
    assert_eq!(
        who_draws_the_header(false, Some("org.mozilla.firefox")),
        Decorations::TheApplicationDraws(ItsOwnButtons {
            minimise: false,
            close: true,
        })
    );
    assert_eq!(
        who_draws_the_header(false, Some("org.gnome.TextEditor")),
        Decorations::TheApplicationDraws(ItsOwnButtons {
            minimise: true,
            close: true,
        })
    );
}

/// **A measured row wins over an agreement**, because the row is what was
/// seen on the screen. Nothing in the table agreed when it was measured. This
/// holds the order should one ever do so.
#[test]
fn a_measured_row_wins_over_an_agreement() {
    assert_eq!(
        who_draws_the_header(true, Some("org.mozilla.firefox")),
        who_draws_the_header(false, Some("org.mozilla.firefox"))
    );
}

/// **The refusal: an application alo knows nothing about is given no
/// duplicate.** It never asked and has no row, so alo cannot know, and
/// assumes the application shows both.
#[test]
fn a_window_alo_cannot_know_is_assumed_to_show_both() {
    for unknown in [Some("org.example.Unmeasured"), Some(""), None] {
        assert_eq!(
            who_draws_the_header(false, unknown),
            Decorations::TheApplicationDraws(ItsOwnButtons::NOT_KNOWN),
            "{unknown:?}"
        );
    }
}

/// **The table is matched exactly**: an `app_id` is the desktop entry's name
/// by the protocol's definition, and a near miss is a different application.
#[test]
fn a_row_is_matched_exactly_and_never_by_a_prefix() {
    for near in ["org.mozilla.firefox.Devel", "org.mozilla", "ORG.MOZILLA.FIREFOX"] {
        assert_eq!(
            who_draws_the_header(false, Some(near)),
            Decorations::TheApplicationDraws(ItsOwnButtons::NOT_KNOWN),
            "{near}"
        );
    }
}
