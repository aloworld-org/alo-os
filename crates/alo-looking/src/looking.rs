//! One check: the whole of it, on the indicator, from the first question to
//! the answer.
//!
//! **One act.** A check asks a place two questions and produces one answer, and
//! both questions happen inside one errand — so a person watching their machine
//! sees *alo OS is checking for an update at …* once, for as long as it takes,
//! rather than two lines appearing and vanishing. [`look`] is the only road to
//! a [`crate::Found`], and it cannot be walked without an
//! `alo_egress::Indicator`, because the only maker of the `Underway` an
//! `alo_keeping_up::Offered` is heard during is the indicator itself.
//!
//! **It fetches an answer and never a build.** The second question asks which
//! build a release *is*; the bytes it names are pulled by the base, later, only
//! if a person chooses to apply the update (`alo_updating::apply`). A check
//! that downloaded the build to be helpful would be exactly the traffic law 1
//! exists to make visible, arriving before anybody agreed to it.
//!
//! **Nothing here decides when.** [`look`] is a function; its caller is
//! whatever put the question — a person, or the machine starting — and there is
//! no timer, no thread and no retry in this crate that could call it again.
//!
//! # Which release is offered
//!
//! The newest one the place holds, and never one older than the release this
//! build was cut from ([`crate::Place::not_before`]).
//!
//! A machine cannot carry the name of a release that did not exist when it was
//! built, so the place it asks is the only thing that can tell it there is one
//! — and *the newest* is the only reading of *what that place offers* that does
//! not need a name somebody can move, which `image/pinned.toml` exists to
//! forbid. Whether the offered build is one this machine will actually accept
//! is not asked here and is not this crate's to answer: the build is staged
//! under the owner's signature policy (ADR 0036, `alo_keeping_up::Staging`),
//! and a build that policy refuses is refused there, with the machine
//! unchanged.
//!
//! Whether the offer *differs* from what is running is
//! `alo_keeping_up::Standing`'s answer and is deliberately *differ, not newer*.
//! The floor above is about which release this machine is willing to be
//! offered; going backwards on purpose is `alo_keeping_up::GoingBack`'s, which
//! a person chooses and is told the consequences of.

use std::time::SystemTime;

use alo_egress::{Indicator, Underway};
use alo_keeping_up::{Digest, Offered, Running, Standing, a_check_at};

use crate::asking::ThePlace;
use crate::because::Because;
use crate::found::Found;
use crate::place::Place;
use crate::refusing::NoAnswer;
use crate::release::Release;

/// Ask `place` whether there is a newer version of this machine's system.
///
/// The errand is on `indicator` from before the first question until after the
/// answer, whichever way it ends.
///
/// # Errors
/// [`NoAnswer`], each of which leaves the machine exactly as it was: a check
/// asks a question and changes nothing.
pub fn look(
    indicator: &mut Indicator,
    now: SystemTime,
    place: &Place,
    running: &Running,
    because: Because,
    asking: &impl ThePlace,
) -> Result<Found, NoAnswer> {
    let underway = indicator.beginning_on_its_own(a_check_at(place.destination().clone()), now);
    let answered = asked(place, running, &underway, asking);
    indicator.ended_on_its_own(underway);
    answered.map(|standing| Found::of(running.digest().clone(), standing, because, now))
}

/// The two questions, inside the errand that is already being shown.
fn asked(
    place: &Place,
    running: &Running,
    underway: &Underway,
    asking: &impl ThePlace,
) -> Result<Standing, NoAnswer> {
    let held = asking.every_name()?;
    let newest = Release::newest_of(held.iter().map(String::as_str))
        .filter(|newest| newest >= place.not_before())
        .ok_or(NoAnswer::NothingIsOffered)?;
    let named = asking.the_build_of(&newest)?;
    let build = Digest::read(&named).map_err(|_| NoAnswer::NotUnderstood)?;
    // The errand was made from `a_check_at` six lines up, so this cannot be an
    // answer heard during some other errand. It is still read rather than
    // unwrapped, and the branch nothing can reach answers with the refusal that
    // says the machine is unchanged — which is true of every road out of this
    // function.
    let offered = Offered::heard(underway, build).map_err(|_| NoAnswer::NotUnderstood)?;
    Ok(Standing::between(running, &offered))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{APlaceThatAnswers, a_moment, a_place_not_before, build, the_place};
    use alo_egress::Errand;

    /// The floor these tests are written against.
    ///
    /// **Stated here rather than read off `image/pinned.toml`.** Which releases
    /// a place holds and which of them is offered is what is being tested, and
    /// none of it is about the number this repository is pinned at: a floor
    /// taken from the shipped pin made every one of these tests a hostage of
    /// the release process, and on 2026-09-19 the pin moved to `0.0.3` and the
    /// test below holding `0.0.2` began answering *nothing is offered*. The one
    /// test that is about the shipped pin says so in its name and uses
    /// [`the_place`].
    const NOT_BEFORE: &str = "0.0.2";

    /// A newer release is an update, and it is `Standing`'s answer rather than
    /// one this crate made up.
    #[test]
    fn a_newer_release_at_the_place_is_an_update() {
        let mut indicator = Indicator::default();
        let place = a_place_not_before(NOT_BEFORE);
        let found = look(
            &mut indicator,
            a_moment(),
            &place,
            &Running::reported(build("aa")),
            Because::ThePersonAsked,
            &APlaceThatAnswers::holding(&["0.0.2", "0.0.3"])
                .whose_builds(&[("0.0.3", build("bb"))]),
        )
        .unwrap();

        assert!(found.is_ready());
        assert_eq!(found.about(), &build("aa"));
        assert!(indicator.is_quiet());
    }

    /// **The newest release is the one offered**, whatever order the place
    /// answered in and whatever else it holds.
    #[test]
    fn the_newest_release_the_place_holds_is_the_one_offered() {
        let mut indicator = Indicator::default();
        let found = look(
            &mut indicator,
            a_moment(),
            &a_place_not_before(NOT_BEFORE),
            &Running::reported(build("aa")),
            Because::ThisMachineStarted,
            &APlaceThatAnswers::holding(&["0.0.10", "latest", "0.0.9", "0.0.2"])
                .whose_builds(&[("0.0.10", build("cc"))]),
        )
        .unwrap();
        assert!(found.is_ready());
    }

    /// **The same build is no update**, which is the whole of *up to date*.
    #[test]
    fn the_build_this_machine_runs_is_not_an_update() {
        let mut indicator = Indicator::default();
        let found = look(
            &mut indicator,
            a_moment(),
            &a_place_not_before(NOT_BEFORE),
            &Running::reported(build("bb")),
            Because::ThePersonAsked,
            &APlaceThatAnswers::holding(&["0.0.2"]).whose_builds(&[("0.0.2", build("bb"))]),
        )
        .unwrap();
        assert!(!found.is_ready());
    }

    /// **A place holding nothing this machine recognises offers nothing**, and
    /// so does one holding only releases older than the one this build was cut
    /// from — a way backwards is not an update.
    #[test]
    fn a_place_offering_nothing_this_machine_would_take_is_refused() {
        for holding in [vec!["latest", "sha256-aa"], vec![], vec!["0.0.1"]] {
            let mut indicator = Indicator::default();
            let refused = look(
                &mut indicator,
                a_moment(),
                &a_place_not_before(NOT_BEFORE),
                &Running::reported(build("bb")),
                Because::ThePersonAsked,
                &APlaceThatAnswers::holding(&holding),
            )
            .unwrap_err();
            assert_eq!(refused, NoAnswer::NothingIsOffered, "{holding:?}");
            assert!(indicator.is_quiet());
        }
    }

    /// **Half a build is not an answer**, and the machine is left as it was.
    #[test]
    fn an_answer_naming_half_a_build_is_not_understood() {
        let mut indicator = Indicator::default();
        let refused = look(
            &mut indicator,
            a_moment(),
            &a_place_not_before(NOT_BEFORE),
            &Running::reported(build("bb")),
            Because::ThePersonAsked,
            &APlaceThatAnswers::holding(&["0.0.3"]).whose_names(&[("0.0.3", "sha256:beef")]),
        )
        .unwrap_err();
        assert_eq!(refused, NoAnswer::NotUnderstood);
        assert!(indicator.is_quiet());
    }

    /// **Both questions happen inside one errand.** The place is asked twice
    /// and the indicator is quiet on either side of the one call, so a person
    /// watching sees one line rather than two appearing and vanishing.
    #[test]
    fn both_questions_happen_inside_one_check() {
        let mut indicator = Indicator::default();
        let asked = APlaceThatAnswers::holding(&["0.0.3"]).whose_builds(&[("0.0.3", build("bb"))]);
        assert!(indicator.is_quiet());
        look(
            &mut indicator,
            a_moment(),
            &a_place_not_before(NOT_BEFORE),
            &Running::reported(build("aa")),
            Because::ThePersonAsked,
            &asked,
        )
        .unwrap();
        assert_eq!(asked.how_often_it_was_asked(), 2);
        assert!(indicator.is_quiet());
    }

    /// **A refusal takes the line down too.** An errand left showing after the
    /// answer came back would be an indicator saying the machine is still
    /// reaching the network when it is not.
    #[test]
    fn a_check_that_was_refused_leaves_nothing_on_the_indicator() {
        let mut indicator = Indicator::default();
        let refused = look(
            &mut indicator,
            a_moment(),
            &a_place_not_before(NOT_BEFORE),
            &Running::reported(build("aa")),
            Because::ThePersonAsked,
            &APlaceThatAnswers::that_cannot_be_reached(),
        )
        .unwrap_err();
        assert_eq!(refused, NoAnswer::NoWayOut);
        assert!(indicator.is_quiet());
        assert!(indicator.showing().is_empty());
    }

    /// **A place's refusal is handed back as it is**, and the line comes down
    /// either way. Nothing here turns one reason into another, because *there
    /// is no road out* and *it refused* are different things to tell somebody.
    #[test]
    fn every_refusal_a_place_answers_with_comes_back_as_it_is() {
        for refusal in NoAnswer::EVERY {
            let mut indicator = Indicator::default();
            let answered = look(
                &mut indicator,
                a_moment(),
                &a_place_not_before(NOT_BEFORE),
                &Running::reported(build("aa")),
                Because::ThisMachineStarted,
                &APlaceThatAnswers::that_answers_with(refusal),
            );
            assert_eq!(answered.unwrap_err(), refusal);
            assert!(indicator.is_quiet());
        }
    }

    /// The line a person reads while it happens names the check and the place
    /// the pin names, worded by `alo-egress` rather than here.
    #[test]
    fn the_line_a_person_reads_is_the_check_at_the_place_the_pin_names() {
        let place = the_place();
        let errand = a_check_at(place.destination().clone());
        assert_eq!(errand.errand(), Errand::CheckingForAnUpdate);
        assert_eq!(errand.destination(), place.destination());
    }

    /// **A machine carrying the pin this repository ships is offered the
    /// release that pin names**, whatever release that is today.
    ///
    /// The floor is the pinned release itself and not one above it, so a
    /// machine built from a release that was afterwards re-pushed is offered
    /// the build now behind that name rather than told there is nothing for it.
    /// This is the one test here that reads `image/pinned.toml`, and it reads
    /// the release out of the place rather than writing a number down, so
    /// pinning the next release moves nothing in this file.
    #[test]
    fn a_machine_at_the_pinned_release_is_offered_what_that_release_is_now() {
        let place = the_place();
        let pinned = place.not_before().named_as().to_owned();
        let mut indicator = Indicator::default();
        let found = look(
            &mut indicator,
            a_moment(),
            &place,
            &Running::reported(build("aa")),
            Because::ThisMachineStarted,
            &APlaceThatAnswers::holding(&[&pinned]).whose_builds(&[(&pinned, build("bb"))]),
        )
        .unwrap();

        assert!(found.is_ready(), "{pinned}");
        assert!(indicator.is_quiet());
    }
}
