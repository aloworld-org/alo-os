//! A request passes only when every try took the road; the summary says, for
//! each, how many did and where the others went.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use alo_applications::verbs::{APPLICATION, OPEN_APPLICATION};

use super::{Tallied, Verdict, summary, tallying, verdict};
use crate::a_road::ARequest;
use crate::a_try::{Tried, Went};
use crate::testing::{Writing, answering, proposing, running, the_verbs, the_words};
use crate::the_set::the_set;
use crate::went_wrong::WentWrong;

const FIREFOX: &str = "org.mozilla.firefox";

fn opening_firefox() -> Vec<String> {
    vec![
        proposing(OPEN_APPLICATION, &[(APPLICATION, FIREFOX)]),
        answering("Firefox is open."),
    ]
}

fn tallied_with(lines: &[String], tries: usize) -> Vec<Tallied> {
    let set = the_set()
        .expect("the set builds")
        .only("open an application by name")
        .expect("it is in the set");
    let mut model = Writing::these(lines);
    let mut said = 0;
    let tallied = tallying(
        &set,
        tries,
        &the_verbs(),
        &the_words(),
        &mut model,
        &mut |_, _, _| said += 1,
    );
    assert_eq!(said, tries, "every try is told as it ends");
    tallied
}

fn the_road() -> Tried {
    Tried::of(vec![OPEN_APPLICATION.to_owned()], Went::TheRoad, 2)
}

fn off_the_road() -> Tried {
    Tried::of(
        vec!["run: firefox".to_owned()],
        Went::WentWrong(WentWrong::RanACommand {
            step: 1,
            expected: OPEN_APPLICATION.to_owned(),
            command: "firefox".to_owned(),
        }),
        1,
    )
}

fn a_request() -> ARequest {
    ARequest::asking("open it", "Open Firefox.", Vec::new())
}

#[test]
fn every_try_on_the_road_passes() {
    let lines: Vec<String> = (0..5).flat_map(|_| opening_firefox()).collect();
    let tallied = tallied_with(&lines, 5);
    assert_eq!(tallied.len(), 1);
    assert_eq!(tallied[0].tries().len(), 5);
    assert_eq!(tallied[0].on_the_road(), 5);
    assert_eq!(verdict(&tallied), Verdict::EveryTryTookTheRoad);
}

/// **The refusal case**: four tries on the road and one that ran a command is
/// a request that did not pass.
#[test]
fn one_try_off_the_road_fails_the_request() {
    let mut lines: Vec<String> = Vec::new();
    lines.extend(opening_firefox());
    lines.push(running("firefox &"));
    for _ in 0..3 {
        lines.extend(opening_firefox());
    }
    let tallied = tallied_with(&lines, 5);
    assert_eq!(tallied[0].on_the_road(), 4);
    assert_eq!(verdict(&tallied), Verdict::SomeLeftTheRoad);
}

#[test]
fn a_try_that_could_not_be_asked_leaves_the_run_unmeasured_not_passed() {
    let tallied = [Tallied::of(
        a_request(),
        vec![
            the_road(),
            Tried::of(Vec::new(), Went::NotAnswered("down".to_owned()), 1),
        ],
    )];
    assert_eq!(verdict(&tallied), Verdict::NotMeasured);
}

#[test]
fn a_try_off_the_road_outweighs_one_not_asked() {
    let tallied = [Tallied::of(
        a_request(),
        vec![
            off_the_road(),
            Tried::of(Vec::new(), Went::NotAnswered("down".to_owned()), 1),
        ],
    )];
    assert_eq!(verdict(&tallied), Verdict::SomeLeftTheRoad);
}

#[test]
fn nothing_tried_is_nothing_measured() {
    assert_eq!(verdict(&[]), Verdict::NotMeasured);
    assert_eq!(
        verdict(&[Tallied::of(a_request(), Vec::new())]),
        Verdict::NotMeasured
    );
}

#[test]
fn the_summary_says_each_request_its_count_and_where_the_others_went() {
    let tallied = [Tallied::of(
        a_request(),
        vec![the_road(), off_the_road(), the_road()],
    )];
    let said = summary(&tallied, "gpt-4o-mini at https://api.openai.com");
    assert!(said.contains("open it"), "{said}");
    assert!(said.contains("2 of 3"), "{said}");
    assert!(
        said.contains("at step 1 it ran the command `firefox`"),
        "{said}"
    );
    assert!(
        said.contains("gpt-4o-mini at https://api.openai.com"),
        "{said}"
    );
    assert!(said.contains("5 model calls"), "{said}");
    assert!(said.contains("did not pass"), "{said}");
}

#[test]
fn the_summary_of_a_pass_says_it_passed() {
    let tallied = [Tallied::of(a_request(), vec![the_road(), the_road()])];
    let said = summary(&tallied, "a model");
    assert!(said.contains("2 of 2"), "{said}");
    assert!(
        said.contains("Every try of every request took the expected road"),
        "{said}"
    );
}

#[test]
fn a_request_never_tried_beside_one_that_passed_is_not_a_pass() {
    let tallied = [
        Tallied::of(a_request(), vec![the_road()]),
        Tallied::of(a_request(), Vec::new()),
    ];
    assert_eq!(verdict(&tallied), Verdict::NotMeasured);
}

/// **A run that could not ask never exits 0**, so a provider that is down
/// cannot read as a pass on the pull request.
#[test]
fn only_a_pass_exits_zero() {
    assert_eq!(Verdict::EveryTryTookTheRoad.exit_code(), 0);
    assert_eq!(Verdict::SomeLeftTheRoad.exit_code(), 1);
    assert_eq!(Verdict::NotMeasured.exit_code(), 2);
}

/// **A summary says how many passes took a look aside, and the cap**, so a
/// pass that needed one is visible rather than counted as a clean one.
#[test]
fn the_summary_counts_the_passes_that_looked_aside_and_names_the_cap() {
    let written = summary(&tallied_with(&opening_firefox(), 2), "a test model");
    assert!(
        written.contains(&format!(
            "at most {} a try",
            crate::a_try::THE_MOST_LOOKS_ASIDE
        )),
        "{written}"
    );
    assert!(written.contains("looked aside"), "{written}");
}

/// **Only a pass that looked aside is counted as one**: a pass with no look is
/// not, and a try that looked and still left the road is not a pass.
#[test]
fn only_passes_that_looked_aside_are_counted_as_such() {
    let request = the_set()
        .expect("the set builds")
        .only("open an application by name")
        .expect("it is in the set")
        .requests()
        .first()
        .cloned()
        .expect("one request");
    let with_looks = vec![Tallied::of(
        request.clone(),
        vec![
            the_road().looking_aside(1),
            the_road(),
            off_the_road().looking_aside(2),
        ],
    )];
    let written = summary(&with_looks, "a test model");
    assert!(
        written.contains("open an application by name: 1 of its passes looked aside"),
        "{written}"
    );
    let without = vec![Tallied::of(request, vec![the_road(), off_the_road()])];
    let written = summary(&without, "a test model");
    assert!(written.contains("No pass looked aside."), "{written}");
}
