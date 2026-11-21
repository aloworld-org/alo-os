//! **What a change is held to**: its own requests (`crate::its_own`), each at
//! least nine in every ten tries on the road — the owner's rule of
//! 2026-11-17. Every other request tried is reported and not counted.

use alo_capability::Verbs;
use alo_strings::Strings;

use crate::a_road::ARequest;
use crate::a_try::{TheLevel, TheModel, Tried, Went};
use crate::its_own::ItsOwn;
use crate::tallied::{Tallied, tallying_at};
use crate::the_bar::{THE_BAR, THE_CONFIRMING_TRIES};
use crate::the_set::TheSet;

/// How many times a change's own request is tried, at each level: the bar's
/// (`crate::the_bar`).
pub const THE_TRIES_ITS_OWN_TAKES: usize = THE_BAR.tries();

/// What a change's own requests say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counted {
    /// The change owns no request: it touches nothing the model is offered.
    NothingOfItsOwn,
    /// Every request of its own took the road at least nine in every ten.
    ItsOwnHeld,
    /// A request of its own took the road fewer than nine in every ten.
    ItsOwnSlipped,
    /// A request of its own slipped on the first tries: flagged, to be tried
    /// again and judged on both rounds — not a verdict yet.
    ToConfirm,
    /// A request of its own is below the bar on all its tries, and is to be
    /// tried on the base and judged against it (`crate::compared_with_its_base`):
    /// not a verdict yet.
    ToCompareWithItsBase,
    /// A request of its own was not tried enough, or the model could not be
    /// asked often enough, to say.
    NotMeasured,
}

impl Counted {
    /// The exit code a run ends with: 0 when the change may merge, 1 when its
    /// own slipped, 2 when nothing could be said.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::NothingOfItsOwn | Self::ItsOwnHeld => 0,
            Self::ItsOwnSlipped => 1,
            Self::NotMeasured | Self::ToConfirm => 2,
            Self::ToCompareWithItsBase => 3,
        }
    }
}

/// [`counted`], with `reached` saying which of its own requests have reached
/// every try at this level (`crate::what_each_has_reached`).
#[must_use]
pub fn counted_with(
    tallied: &[Tallied],
    own: &[ItsOwn],
    reached: &dyn Fn(&str) -> bool,
) -> Counted {
    if own.is_empty() {
        return Counted::NothingOfItsOwn;
    }
    own.iter()
        .map(|one| {
            tallied
                .iter()
                .find(|tried| tried.request().named() == one.named())
                .map_or(Counted::NotMeasured, |tried| {
                    what_one_says(tried, reached(one.named()))
                })
        })
        .fold(Counted::ItsOwnHeld, |so_far, this| match (so_far, this) {
            (Counted::ItsOwnSlipped, _) | (_, Counted::ItsOwnSlipped) => Counted::ItsOwnSlipped,
            (Counted::ToConfirm, _) | (_, Counted::ToConfirm) => Counted::ToConfirm,
            (Counted::NotMeasured, _) | (_, Counted::NotMeasured) => Counted::NotMeasured,
            _ => Counted::ItsOwnHeld,
        })
}

/// What `tallied` says about the change owning `own`.
#[must_use]
pub fn counted(tallied: &[Tallied], own: &[ItsOwn]) -> Counted {
    counted_with(tallied, own, &|_| false)
}

/// What one request of the change's own says: held when it meets the bar
/// ([`THE_BAR`]) of at least the bar's tries — every try, where it has
/// `reached` every try before; not measured when it was tried fewer times, or
/// when the tries the model could not be asked would have decided it; slipped
/// otherwise.
pub(crate) fn what_one_says(tallied: &Tallied, reached: bool) -> Counted {
    let tries = tallied.tries().len();
    let on_the_road = tallied.on_the_road();
    let not_asked = tallied
        .tries()
        .iter()
        .filter(|tried| matches!(tried.went(), Went::NotAnswered(_)))
        .count();
    if tries < THE_TRIES_ITS_OWN_TAKES {
        Counted::NotMeasured
    } else if THE_BAR.holds(on_the_road, tries, reached) {
        Counted::ItsOwnHeld
    } else if THE_BAR.holds(on_the_road + not_asked, tries, reached) {
        Counted::NotMeasured
    } else if tries < THE_BAR.tries() + THE_CONFIRMING_TRIES {
        // **Flag, then confirm** (approved 2026-11-20): a slip on the first
        // tries is tried again, and the verdict is on both rounds.
        Counted::ToConfirm
    } else {
        Counted::ItsOwnSlipped
    }
}

/// The summary: the verdict on the change's own requests, why each is its
/// own, and then every request tried, the others marked as not counted.
#[must_use]
pub fn summary_counting(tallied: &[Tallied], own: &[ItsOwn], asked_of: &str) -> String {
    if own.is_empty() {
        return format!(
            "## What alo's model chose\n\nThis change alters nothing the model is offered: \
             `crates/alo-every-try/{}` is the base's, so it owns no request and nothing was \
             tried.\n",
            crate::the_declaration::THE_FILE
        );
    }
    let calls: usize = tallied
        .iter()
        .flat_map(|one| one.tries().iter())
        .map(Tried::asked)
        .sum();
    let mut said = format!(
        "## This change's own requests, held to at least 9 of every 10 tries\n\n\
         Set version {}, asked of {asked_of}, with the instructions and verbs a turn is given. \
         Nothing was carried out on any machine. {calls} model calls in all.\n\n",
        crate::the_set::THE_VERSION
    );
    said.push_str(match counted(tallied, own) {
        Counted::NothingOfItsOwn | Counted::ItsOwnHeld => {
            "**Its own requests held**: each took the road at least 9 of every 10 tries.\n\n"
        }
        Counted::ItsOwnSlipped => {
            "**The check did not pass**: a request of this change's own took the road fewer \
             than 9 of every 10 tries.\n\n"
        }
        Counted::ToCompareWithItsBase => {
            "**To be compared with its base**: a request of this change's own is below the bar \n             on all its tries, and is tried on the base to see whether it was already.

"
        }
        Counted::ToConfirm => {
            "**Flagged**: a request of this change's own slipped on these tries, and is tried \n             again; the verdict is on both rounds.

"
        }
        Counted::NotMeasured => {
            "**Nothing was measured**: a request of this change's own was not tried ten times, \
             or the model could not be asked often enough to say.\n\n"
        }
    });
    said.push_str(
        "| Its own request | Why it is this change's own | Took the road |\n|---|---|---|\n",
    );
    for one in own {
        let took = tallied
            .iter()
            .find(|tried| tried.request().named() == one.named())
            .map_or_else(
                || "not tried".to_owned(),
                |tried| format!("{} of {}", tried.on_the_road(), tried.tries().len()),
            );
        said.push_str(&format!(
            "| {} | {} | {took} |\n",
            one.named(),
            one.why().said()
        ));
    }
    said.push_str(
        "\nEvery other request below was tried fewer times and is reported, not counted: its \
         slips do not fail this change.\n\n",
    );
    said.push_str(&crate::tallied::the_requests_tried(tallied));
    said
}

/// **Try a change's own and then every other request**: each of `own` in
/// `set` `tries.0` times, then every other `tries.1` times — none when that
/// is zero — its own first, saying each try through `told`.
#[expect(
    clippy::too_many_arguments,
    reason = "the same arguments as `tallying_at`, with which requests are its own"
)]
pub fn tallying_its_own(
    level: TheLevel,
    set: &TheSet,
    own: &[ItsOwn],
    tries: (usize, usize),
    verbs: &Verbs,
    strings: &Strings,
    model: &mut dyn TheModel,
    told: &mut dyn FnMut(&ARequest, usize, &Tried),
) -> Vec<Tallied> {
    let is_its_own = |request: &ARequest| own.iter().any(|one| one.named() == request.named());
    let (its, others) = tries;
    let mut tallied = tallying_at(
        level,
        &set.clone().retaining(is_its_own),
        its,
        verbs,
        strings,
        model,
        told,
    );
    if others > 0 {
        tallied.extend(tallying_at(
            level,
            &set.clone().retaining(|request| !is_its_own(request)),
            others,
            verbs,
            strings,
            model,
            told,
        ));
    }
    tallied
}

/// Its own requests flagged on these tries.
#[must_use]
pub fn the_flagged_of_its_own(
    tallied: &[Tallied],
    own: &[ItsOwn],
    reached: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    own.iter()
        .filter(|one| {
            tallied
                .iter()
                .find(|tried| tried.request().named() == one.named())
                .is_some_and(|tried| {
                    what_one_says(tried, reached(one.named())) == Counted::ToConfirm
                })
        })
        .map(|one| one.named().to_owned())
        .collect()
}

/// `tallied` with the tries of `more` added to the request each is of.
#[must_use]
pub fn with_more(tallied: Vec<Tallied>, more: Vec<Tallied>) -> Vec<Tallied> {
    tallied
        .into_iter()
        .map(|one| {
            let again: Vec<Tried> = more
                .iter()
                .filter(|other| other.request().named() == one.request().named())
                .flat_map(|other| other.tries().iter().cloned())
                .collect();
            if again.is_empty() {
                one
            } else {
                let tries = one.tries().iter().cloned().chain(again).collect();
                Tallied::of(one.request().clone(), tries)
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "counted_tests.rs"]
mod tests;
