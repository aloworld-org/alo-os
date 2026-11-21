//! **Judging a change's own requests**, the confirm round included (flag,
//! then confirm, approved 2026-11-20).

use alo_capability::Verbs;
use alo_strings::Strings;

use crate::a_road::ARequest;
use crate::a_try::{TheLevel, TheModel, Tried};
use crate::counted::{
    Counted, counted_with, summary_counting, tallying_its_own, the_flagged_of_its_own,
    what_one_says, with_more,
};
use crate::its_own::ItsOwn;
use crate::tallied::{Tallied, tallying_at};
use crate::the_bar::{THE_BAR, THE_CONFIRMING_TRIES};
use crate::the_set::TheSet;

/// What judging a change's own requests came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheJudgement {
    /// Every request tried, both rounds added.
    tallied: Vec<Tallied>,
    /// The summary, both rounds shown.
    said: String,
    /// The verdict.
    counted: Counted,
    /// Its own requests to be tried on the base and judged against it.
    to_compare: Vec<String>,
}

impl TheJudgement {
    /// Every request tried, both rounds added.
    #[must_use]
    pub fn tallied(&self) -> &[Tallied] {
        &self.tallied
    }

    /// The summary.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// The verdict.
    #[must_use]
    pub const fn counted(&self) -> Counted {
        self.counted
    }

    /// Its own requests below the bar on all their tries, to be tried on the
    /// base and judged against it.
    #[must_use]
    pub fn to_compare_with_its_base(&self) -> &[String] {
        &self.to_compare
    }
}

/// **Judge `own` at `level`**: each tried the bar's number of times and
/// every other request in `set` `others` times; each of its own that slipped
/// tried [`THE_CONFIRMING_TRIES`] more; the verdict on both rounds, with the
/// ratchet where `reached` says a request once took every try here.
#[expect(
    clippy::too_many_arguments,
    reason = "the arguments of a try, with which requests are its own and what they reached"
)]
pub fn judging_its_own(
    level: TheLevel,
    set: &TheSet,
    own: &[ItsOwn],
    others: usize,
    reached: &dyn Fn(&str) -> bool,
    verbs: &Verbs,
    strings: &Strings,
    model: &mut dyn TheModel,
    told: &mut dyn FnMut(&ARequest, usize, &Tried),
    asked_of: &str,
) -> TheJudgement {
    let mut tallied = tallying_its_own(
        level,
        set,
        own,
        // **The bar's tries**, so the bar is one declared setting.
        (THE_BAR.tries(), others),
        verbs,
        strings,
        model,
        told,
    );
    let mut said = summary_counting(&tallied, own, asked_of);
    let flagged = the_flagged_of_its_own(&tallied, own, reached);
    if !flagged.is_empty() {
        let again = tallying_at(
            level,
            &set.clone()
                .retaining(|request| flagged.iter().any(|named| named == request.named())),
            THE_CONFIRMING_TRIES,
            verbs,
            strings,
            model,
            told,
        );
        tallied = with_more(tallied, again);
        said.push_str(&format!(
            "\n### After {THE_CONFIRMING_TRIES} more tries of what was flagged\n\n"
        ));
        said.push_str(&summary_counting(&tallied, own, asked_of));
    }
    let held_to_every_try: Vec<&str> = own
        .iter()
        .map(ItsOwn::named)
        .filter(|named| reached(named))
        .collect();
    if !held_to_every_try.is_empty() {
        said.push_str(&format!(
            "\nHeld to every try at this level, because each once took every try here \
             (`what-each-request-has-reached.txt`): {}.\n",
            held_to_every_try.join(", ")
        ));
    }
    // **An own request below the bar on all its tries is compared with its
    // base** (the coordinator, 2026-11-20, keeping the owner's two decisions
    // consistent): it may be weak on the base too, and a change that did not
    // break it is not blocked for it. The ratchet still blocks outright.
    let says = |one: &ItsOwn, at_every_try: bool| {
        tallied
            .iter()
            .find(|tried| tried.request().named() == one.named())
            .map(|tried| what_one_says(tried, at_every_try))
    };
    let ratchet_broken = own
        .iter()
        .any(|one| reached(one.named()) && says(one, true) == Some(Counted::ItsOwnSlipped));
    let to_compare: Vec<String> = own
        .iter()
        .filter(|one| !reached(one.named()) && says(one, false) == Some(Counted::ItsOwnSlipped))
        .map(|one| one.named().to_owned())
        .collect();
    let counted = match counted_with(&tallied, own, reached) {
        Counted::ItsOwnSlipped if !ratchet_broken && !to_compare.is_empty() => {
            Counted::ToCompareWithItsBase
        }
        counted => counted,
    };
    TheJudgement {
        tallied,
        said,
        counted,
        to_compare,
    }
}

#[cfg(test)]
#[path = "judging_its_own_tests.rs"]
mod tests;
