//! "Compare with main" (the owner, 2026-11-20): a change that owns the whole
//! set is judged against its base, tried the same number of times, and a
//! flagged request is confirmed on twenty tries before it blocks.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected Err or index is the failure being reported"
)]

use super::{Judged, compared, the_flagged, verdict_of};
use crate::counted::Counted;
use crate::what_each_has_reached::{Measured, Reached};

/// What a run measured, written as its file holds it.
fn measured(lines: &[(&str, &str, usize, usize)]) -> Measured {
    let text: String = lines
        .iter()
        .map(|(level, named, on, of)| format!("measured\t{level}\t{named}\t{on}\t{of}\n"))
        .collect();
    Measured::read(&text).expect("it reads")
}

fn judged(change: &[(&str, &str, usize, usize)], base: &[(&str, &str, usize, usize)]) -> Judged {
    judged_with(change, base, &Reached::default())
}

fn judged_with(
    change: &[(&str, &str, usize, usize)],
    base: &[(&str, &str, usize, usize)],
    record: &Reached,
) -> Judged {
    let rows = compared(&measured(change), &measured(base), record);
    assert_eq!(rows.len(), 1);
    rows[0].judged().clone()
}

const L: &str = "full-access";

#[test]
fn no_worse_than_its_base_holds() {
    assert_eq!(
        judged(&[(L, "r", 10, 10)], &[(L, "r", 10, 10)]),
        Judged::Held
    );
    assert_eq!(
        judged(&[(L, "r", 9, 10)], &[(L, "r", 10, 10)]),
        Judged::Held
    );
}

/// **A request below the bar on its base may stay below it**, no worse.
#[test]
fn a_request_weak_on_its_base_may_stay_weak_but_no_worse() {
    assert_eq!(
        judged(&[(L, "slider", 4, 10)], &[(L, "slider", 5, 10)]),
        Judged::Held
    );
    assert_eq!(
        judged(&[(L, "slider", 0, 10)], &[(L, "slider", 0, 10)]),
        Judged::Held
    );
}

/// **At ten tries, a flag, not a verdict**: a drop of two, falling below
/// nine where the base had nine, and a ratchet broken are each flagged to be
/// confirmed on twenty.
#[test]
fn at_ten_tries_a_fault_is_flagged_to_be_confirmed() {
    assert_eq!(
        judged(&[(L, "slider", 3, 10)], &[(L, "slider", 5, 10)]),
        Judged::ToConfirm,
        "a drop of two"
    );
    assert_eq!(
        judged(&[(L, "close", 8, 10)], &[(L, "close", 9, 10)]),
        Judged::ToConfirm,
        "below nine where the base had nine"
    );
    let record = Reached::read("reached\tfull-access\tclose\n").expect("it reads");
    assert_eq!(
        judged_with(&[(L, "close", 9, 10)], &[(L, "close", 10, 10)], &record),
        Judged::ToConfirm,
        "the ratchet"
    );
}

/// **On twenty tries, the verdict**: a drop of four, below eighteen where the
/// base had eighteen, and a recorded request below twenty each block; less
/// holds.
#[test]
fn on_twenty_tries_the_verdict() {
    assert_eq!(
        judged(&[(L, "slider", 6, 20)], &[(L, "slider", 10, 20)]),
        Judged::ClearlyWorse
    );
    assert_eq!(
        judged(&[(L, "slider", 7, 20)], &[(L, "slider", 10, 20)]),
        Judged::Held
    );
    assert_eq!(
        judged(&[(L, "close", 17, 20)], &[(L, "close", 18, 20)]),
        Judged::BelowTheBarWhereItsBaseMetIt
    );
    assert_eq!(
        judged(&[(L, "close", 18, 20)], &[(L, "close", 20, 20)]),
        Judged::Held
    );
    let record = Reached::read("reached\tfull-access\tclose\n").expect("it reads");
    assert_eq!(
        judged_with(&[(L, "close", 19, 20)], &[(L, "close", 20, 20)], &record),
        Judged::BrokeTheRatchet
    );
    assert_eq!(
        judged_with(&[(L, "close", 20, 20)], &[(L, "close", 17, 20)], &record),
        Judged::Held
    );
}

/// The two runs of one tree measured on 2026-11-20 (CI runs 44779963562 and
/// 44793910551, at full access): "close an application" 10 then 8. Judged on
/// ten alone it would block a change that altered nothing; it is flagged.
#[test]
fn the_measured_wobble_of_an_unchanged_tree_is_flagged_not_blocked() {
    assert_eq!(
        judged(&[(L, "close", 8, 10)], &[(L, "close", 10, 10)]),
        Judged::ToConfirm
    );
}

/// A request new in the change has no base: it is held to the bar.
#[test]
fn a_request_new_in_the_change_is_held_to_the_bar() {
    assert_eq!(judged(&[(L, "new", 9, 10)], &[]), Judged::Held);
    assert_eq!(judged(&[(L, "new", 8, 10)], &[]), Judged::ToConfirm);
    assert_eq!(judged(&[(L, "new", 17, 20)], &[]), Judged::BelowTheBar);
}

/// **Not measured, never a pass**: too few tries, or a base tried a different
/// number of times, cannot be compared.
#[test]
fn what_cannot_be_compared_is_not_measured() {
    assert_eq!(
        judged(&[(L, "r", 5, 5)], &[(L, "r", 5, 5)]),
        Judged::NotMeasured
    );
    assert_eq!(
        judged(&[(L, "r", 10, 10)], &[(L, "r", 18, 20)]),
        Judged::NotMeasured
    );
}

/// **Two rounds add up**: what was measured for one request at one level in
/// the first ten and the confirming ten is judged on the total.
#[test]
fn the_confirming_round_is_added_to_the_first() {
    let rows = compared(
        &measured(&[(L, "close", 8, 10), (L, "close", 10, 10)]),
        &measured(&[(L, "close", 10, 10), (L, "close", 9, 10)]),
        &Reached::default(),
    );
    assert_eq!(rows[0].change(), (18, 20));
    assert_eq!(rows[0].base(), Some((19, 20)));
    assert_eq!(rows[0].judged(), &Judged::Held);
}

#[test]
fn the_verdict_and_what_is_flagged() {
    let rows = compared(
        &measured(&[
            (L, "a", 10, 10),
            (L, "b", 7, 10),
            ("ask-each-time", "a", 10, 10),
        ]),
        &measured(&[
            (L, "a", 10, 10),
            (L, "b", 10, 10),
            ("ask-each-time", "a", 10, 10),
        ]),
        &Reached::default(),
    );
    assert_eq!(
        the_flagged(&rows),
        [(L.to_owned(), "b".to_owned())],
        "only what is flagged is tried again"
    );
    assert_eq!(
        verdict_of(&rows),
        Counted::NotMeasured,
        "a flag is not yet a verdict"
    );
    let held = compared(
        &measured(&[(L, "a", 10, 10)]),
        &measured(&[(L, "a", 10, 10)]),
        &Reached::default(),
    );
    assert_eq!(verdict_of(&held), Counted::ItsOwnHeld);
    let worse = compared(
        &measured(&[(L, "a", 10, 20)]),
        &measured(&[(L, "a", 20, 20)]),
        &Reached::default(),
    );
    assert_eq!(verdict_of(&worse), Counted::ItsOwnSlipped);
}

/// What `good` tries on the road and then a provider that runs out measured,
/// of ten: the model is asked by a script with only `good` tries' lines.
fn running_out_after(good: usize) -> Measured {
    use crate::testing::{Writing, answering, proposing, the_verbs, the_words};
    use alo_applications::verbs::{APPLICATION, OPEN_APPLICATION};

    let set = crate::the_set::the_set()
        .expect("the set builds")
        .retaining(|request| request.named() == "open an application by name");
    let lines: Vec<String> = (0..good)
        .flat_map(|_| {
            [
                proposing(OPEN_APPLICATION, &[(APPLICATION, "org.mozilla.firefox")]),
                answering("Firefox is open."),
            ]
        })
        .collect();
    let mut model = Writing::these(&lines);
    let tallied = crate::tallied::tallying_at(
        crate::a_try::TheLevel::FullAccess,
        &set,
        10,
        &the_verbs(),
        &the_words(),
        &mut model,
        &mut |_, _, _| {},
    );
    Measured::of(crate::a_try::TheLevel::FullAccess, &tallied)
}

/// **A provider that runs out on the change** (the coordinator, 2026-11-20:
/// the check failed open on #1423's run 44854527313): a try the model could
/// not be asked makes the comparison not measured, which fails the check.
#[test]
fn a_provider_that_runs_out_on_the_change_is_not_measured() {
    let rows = compared(
        &running_out_after(0),
        &running_out_after(10),
        &Reached::default(),
    );
    assert_eq!(rows[0].judged(), &Judged::NotMeasured);
    assert_eq!(verdict_of(&rows), Counted::NotMeasured);
    assert_ne!(verdict_of(&rows).exit_code(), 0);
    let one_short = compared(
        &running_out_after(9),
        &running_out_after(10),
        &Reached::default(),
    );
    assert_eq!(
        one_short[0].judged(),
        &Judged::NotMeasured,
        "one try unasked is enough"
    );
}

/// **And on the base**: a base the model could not be asked is no base to be
/// compared with.
#[test]
fn a_provider_that_runs_out_on_the_base_is_not_measured() {
    let rows = compared(
        &running_out_after(10),
        &running_out_after(0),
        &Reached::default(),
    );
    assert_eq!(rows[0].judged(), &Judged::NotMeasured);
    assert_eq!(verdict_of(&rows), Counted::NotMeasured);
}

/// **And on both**: nought of ten beside nought of ten is not "no worse than
/// its base" — it is nothing measured. This is the run that went green.
#[test]
fn a_provider_that_runs_out_on_both_is_not_measured() {
    let rows = compared(
        &running_out_after(0),
        &running_out_after(0),
        &Reached::default(),
    );
    assert_eq!(rows[0].judged(), &Judged::NotMeasured);
    assert_eq!(verdict_of(&rows), Counted::NotMeasured);
    assert_ne!(verdict_of(&rows).exit_code(), 0);
}

/// **And in the confirm round**: a first round measured and a second that ran
/// out add up to a comparison not measured.
#[test]
fn a_provider_that_runs_out_in_the_confirm_round_is_not_measured() {
    let first = Measured::read(&running_out_after(8).written()).expect("it reads");
    let both: String = first.written() + &running_out_after(0).written();
    let base: String = running_out_after(10).written() + &running_out_after(10).written();
    let rows = compared(
        &Measured::read(&both).expect("it reads"),
        &Measured::read(&base).expect("it reads"),
        &Reached::default(),
    );
    assert_eq!(rows[0].change().1, 20);
    assert_eq!(rows[0].judged(), &Judged::NotMeasured);
}

/// **A refused try is shown as refused, never as a wrong one**: the summary
/// says how many tries the model could not be asked on each side, and a base
/// refused alone is "not measured", never "better" or "worse".
#[test]
fn the_summary_shows_the_tries_not_asked_on_each_side() {
    let rows = compared(
        &running_out_after(10),
        &running_out_after(4),
        &Reached::default(),
    );
    assert_eq!(rows[0].judged(), &Judged::NotMeasured);
    assert!(!rows[0].judged().blocks(), "not worse");
    let said = super::summary_of(&rows, "a scripted model");
    assert!(said.contains("6 not asked"), "{said}");
    assert!(said.contains("Nothing was decided yet"), "{said}");
}

/// A side the model was asked on every try says nothing of tries not asked.
#[test]
fn a_side_asked_every_try_says_none_not_asked() {
    let rows = compared(
        &running_out_after(10),
        &running_out_after(10),
        &Reached::default(),
    );
    let said = super::summary_of(&rows, "a scripted model");
    assert!(!said.contains("not asked)"), "{said}");
}
