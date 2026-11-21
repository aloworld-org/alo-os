//! **Judging a change's own requests**, the confirm round included (flag,
//! then confirm, approved 2026-11-20).

use alo_capability::Verbs;
use alo_strings::Strings;

use crate::a_road::ARequest;
use crate::a_try::{TheLevel, TheModel, Tried};
use crate::counted::{
    Counted, counted_with, summary_counting, tallying_its_own, the_flagged_of_its_own, with_more,
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
    let counted = counted_with(&tallied, own, reached);
    TheJudgement {
        tallied,
        said,
        counted,
    }
}

#[cfg(test)]
#[path = "judging_its_own_tests.rs"]
mod tests;
