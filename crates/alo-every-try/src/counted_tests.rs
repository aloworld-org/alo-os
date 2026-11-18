//! The owner's rule, 2026-11-17: a change that adds or alters something the
//! model chooses is held to **its own** requests passing at least 9 of 10
//! tries; other requests' slips are reported and do not count.

use super::{Counted, THE_TRIES_ITS_OWN_TAKES, counted, summary_counting};
use crate::a_road::ARequest;
use crate::a_try::{Tried, Went};
use crate::its_own::{ItsOwn, WhyItsOwn};
use crate::tallied::Tallied;
use crate::went_wrong::WentWrong;

fn on_the_road() -> Tried {
    Tried::of(vec!["open_application".to_owned()], Went::TheRoad, 2)
}

fn off_the_road() -> Tried {
    Tried::of(
        vec!["run: firefox".to_owned()],
        Went::WentWrong(WentWrong::RanACommand {
            step: 1,
            expected: "open_application".to_owned(),
            command: "firefox".to_owned(),
        }),
        1,
    )
}

fn not_answered() -> Tried {
    Tried::of(Vec::new(), Went::NotAnswered("no answer".to_owned()), 2)
}

/// `named`, tried `passes` times on the road and `slips` off it.
fn tried(named: &'static str, passes: usize, slips: usize) -> Tallied {
    let tries = std::iter::repeat_with(on_the_road)
        .take(passes)
        .chain(std::iter::repeat_with(off_the_road).take(slips))
        .collect();
    Tallied::of(ARequest::asking(named, "Open Firefox.", Vec::new()), tries)
}

fn own(named: &str) -> ItsOwn {
    ItsOwn::because(named, WhyItsOwn::ItsRoadChanged)
}

#[test]
fn ten_tries_is_what_its_own_take() {
    assert_eq!(THE_TRIES_ITS_OWN_TAKES, 10);
}

/// **The refusal case**: its own request at 8 of 10 fails the change.
#[test]
fn its_own_request_at_eight_of_ten_fails() {
    let verdict = counted(&[tried("mine", 8, 2)], &[own("mine")]);
    assert_eq!(verdict, Counted::ItsOwnSlipped);
    assert_eq!(verdict.exit_code(), 1);
}

#[test]
fn its_own_request_at_nine_of_ten_passes() {
    let verdict = counted(&[tried("mine", 9, 1)], &[own("mine")]);
    assert_eq!(verdict, Counted::ItsOwnHeld);
    assert_eq!(verdict.exit_code(), 0);
}

/// One own request slipping fails the change, though another held.
#[test]
fn one_of_its_own_slipping_fails_though_another_held() {
    let verdict = counted(
        &[tried("mine", 10, 0), tried("also mine", 7, 3)],
        &[own("mine"), own("also mine")],
    );
    assert_eq!(verdict, Counted::ItsOwnSlipped);
}

/// **Another request at 0 of 10 does not fail it**: it is reported, not
/// counted.
#[test]
fn another_request_at_nothing_of_ten_does_not_fail_it() {
    let verdict = counted(
        &[tried("mine", 10, 0), tried("somebody else's", 0, 10)],
        &[own("mine")],
    );
    assert_eq!(verdict, Counted::ItsOwnHeld);
    assert_eq!(verdict.exit_code(), 0);
}

/// **A change that touches nothing the model chooses passes**, whatever the
/// other requests did — and without any of them having been tried.
#[test]
fn a_change_with_nothing_of_its_own_passes() {
    assert_eq!(counted(&[], &[]), Counted::NothingOfItsOwn);
    assert_eq!(Counted::NothingOfItsOwn.exit_code(), 0);
    assert_eq!(
        counted(&[tried("somebody else's", 0, 10)], &[]),
        Counted::NothingOfItsOwn
    );
}

/// Fewer than ten tries cannot show 9 of 10, so a run that tried its own
/// fewer times has not measured it — and that is not a pass.
#[test]
fn its_own_tried_fewer_than_ten_times_is_not_measured() {
    let verdict = counted(&[tried("mine", 5, 0)], &[own("mine")]);
    assert_eq!(verdict, Counted::NotMeasured);
    assert_ne!(verdict.exit_code(), 0);
}

/// Its own request missing from the run is not measured, never a pass.
#[test]
fn its_own_not_tried_at_all_is_not_measured() {
    let verdict = counted(&[tried("somebody else's", 10, 0)], &[own("mine")]);
    assert_eq!(verdict, Counted::NotMeasured);
    assert_ne!(verdict.exit_code(), 0);
}

/// **A provider that did not answer is not a pass and not a slip**: 8 on the
/// road and 2 not asked could have been 10, so nothing was measured; 7 on the
/// road, 1 not asked and 2 wrong is below 9 whatever the one would have done.
#[test]
fn tries_the_model_was_not_asked_are_neither_a_pass_nor_a_slip() {
    let mut eight_and_two_unasked = tried("mine", 8, 0);
    eight_and_two_unasked = Tallied::of(
        eight_and_two_unasked.request().clone(),
        eight_and_two_unasked
            .tries()
            .iter()
            .cloned()
            .chain([not_answered(), not_answered()])
            .collect(),
    );
    let verdict = counted(&[eight_and_two_unasked], &[own("mine")]);
    assert_eq!(verdict, Counted::NotMeasured);
    assert_eq!(verdict.exit_code(), 2);

    let seven = tried("mine", 7, 2);
    let seven = Tallied::of(
        seven.request().clone(),
        seven
            .tries()
            .iter()
            .cloned()
            .chain([not_answered()])
            .collect(),
    );
    assert_eq!(counted(&[seven], &[own("mine")]), Counted::ItsOwnSlipped);
}

/// **A provider that is down is not a slip**: ten tries the model was never
/// asked, or eight on the road, one unasked and one wrong, could each have been
/// nine — so nothing was measured, and the run says so rather than blaming the
/// change.
#[test]
fn a_provider_that_did_not_answer_is_not_blamed_on_the_change() {
    let down = Tallied::of(
        ARequest::asking("mine", "Open Firefox.", Vec::new()),
        std::iter::repeat_with(not_answered).take(10).collect(),
    );
    assert_eq!(counted(&[down], &[own("mine")]), Counted::NotMeasured);
    let eight = tried("mine", 8, 1);
    let eight = Tallied::of(
        eight.request().clone(),
        eight
            .tries()
            .iter()
            .cloned()
            .chain([not_answered()])
            .collect(),
    );
    assert_eq!(counted(&[eight], &[own("mine")]), Counted::NotMeasured);
}

/// Nine on the road and one the model was not asked is 9 of 10 whatever the
/// one would have done, so it holds.
#[test]
fn nine_on_the_road_and_one_unasked_holds() {
    let nine = tried("mine", 9, 0);
    let nine = Tallied::of(
        nine.request().clone(),
        nine.tries()
            .iter()
            .cloned()
            .chain([not_answered()])
            .collect(),
    );
    assert_eq!(counted(&[nine], &[own("mine")]), Counted::ItsOwnHeld);
}

/// **At least nine in every ten**: a run of twenty holds at 18 and slips at 17.
#[test]
fn more_than_ten_tries_are_held_to_nine_in_every_ten() {
    assert_eq!(
        counted(&[tried("mine", 18, 2)], &[own("mine")]),
        Counted::ItsOwnHeld
    );
    assert_eq!(
        counted(&[tried("mine", 17, 3)], &[own("mine")]),
        Counted::ItsOwnSlipped
    );
}

/// The summary says which requests were counted and why they are the change's
/// own, and that the others were reported and not counted.
#[test]
fn the_summary_names_what_was_counted_and_what_was_not() {
    let said = summary_counting(
        &[tried("somebody else's", 0, 3), tried("mine", 8, 2)],
        &[own("mine")],
        "a model",
    );
    assert!(said.contains("mine"), "{said}");
    assert!(
        said.contains("| mine | its road changed | 8 of 10 |"),
        "its own row reads its own tries, not another request's: {said}"
    );
    assert!(said.contains("somebody else's"), "{said}");
    assert!(said.contains("not counted"), "{said}");
    assert!(said.contains("did not pass"), "{said}");
    assert!(
        said.contains("its road changed"),
        "why it is the change's own: {said}"
    );
    let nothing = summary_counting(&[], &[], "a model");
    assert!(
        nothing.contains("nothing the model is offered"),
        "{nothing}"
    );
    assert!(!nothing.contains("did not pass"), "{nothing}");
}
