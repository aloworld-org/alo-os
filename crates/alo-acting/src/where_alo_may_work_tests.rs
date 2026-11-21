//! The one answer, at every setting, for an application with a copy of its own
//! and one without.

use alo_capability::AloInApplications;

use super::alo_may_work_in_their_window;

/// Firefox has a copy of its own for alo; Text Editor has none.
const WITH_A_COPY: &str = "org.mozilla.firefox";
const WITH_NONE: &str = "org.gnome.TextEditor";

#[test]
fn the_answer_is_the_settings_rule_over_whether_there_is_a_copy() {
    assert!(alo_launching::has_a_copy_of_its_own(WITH_A_COPY));
    assert!(!alo_launching::has_a_copy_of_its_own(WITH_NONE));
    for chosen in AloInApplications::ALL {
        for application in [WITH_A_COPY, WITH_NONE] {
            assert_eq!(
                alo_may_work_in_their_window(chosen, application),
                chosen.may_work_in_their_window_of(application == WITH_A_COPY),
                "{chosen:?} {application}"
            );
        }
    }
    // And the three cases a person meets, said outright.
    assert!(alo_may_work_in_their_window(
        AloInApplications::InItsOwnCopyOrTheirWindow,
        WITH_NONE
    ));
    assert!(!alo_may_work_in_their_window(
        AloInApplications::InItsOwnCopy,
        WITH_NONE
    ));
    assert!(!alo_may_work_in_their_window(
        AloInApplications::InItsOwnCopyOrTheirWindow,
        WITH_A_COPY
    ));
}
