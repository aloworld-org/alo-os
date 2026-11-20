//! **Which requests a change owns**: read off two declarations of what the
//! model is offered (`crate::the_declaration`) — the base's committed file and
//! the change's own code — never guessed from which files it touched.
//!
//! A request is the change's own when it is new or its own line changed. And
//! **every request is the change's own** when what every turn is told
//! changed, or when any verb offered is new, taken away or reads otherwise:
//! every turn is shown every verb offered, and the request a change to one
//! verb breaks is often one whose road never uses it, choosing it (the
//! coordinator, 2026-11-20, on #1413). A request taken out of the set is
//! nothing to try.

use crate::the_declaration::Declared;

/// Why a request is a change's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyItsOwn {
    /// The change adds it.
    ItIsNew,
    /// Its definition changed: its words, its road, what a step expects.
    ItsRoadChanged,
    /// What every turn is told changed, which every request is shown.
    EveryTurnIsToldOtherwise,
    /// Every turn is shown this verb, which the change adds, alters or takes
    /// away.
    EveryTurnIsShownAVerbOtherwise(String),
}

impl WhyItsOwn {
    /// The reason, in words for the summary.
    #[must_use]
    pub fn said(&self) -> String {
        match self {
            Self::ItIsNew => "it is new".to_owned(),
            Self::ItsRoadChanged => "its road changed".to_owned(),
            Self::EveryTurnIsToldOtherwise => "what every turn is told changed".to_owned(),
            Self::EveryTurnIsShownAVerbOtherwise(verb) => {
                format!("every turn is shown `{verb}`, which changed")
            }
        }
    }
}

/// One request a change owns, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItsOwn {
    /// The request's name.
    named: String,
    /// Why it is the change's own.
    why: WhyItsOwn,
}

impl ItsOwn {
    /// The request called `named` is the change's own, because `why`.
    #[must_use]
    pub fn because(named: &str, why: WhyItsOwn) -> Self {
        Self {
            named: named.to_owned(),
            why,
        }
    }

    /// The request's name.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// Why it is the change's own.
    #[must_use]
    pub const fn why(&self) -> &WhyItsOwn {
        &self.why
    }
}

/// The requests the change from `before` to `after` owns, in `after`'s order,
/// each once.
#[must_use]
pub fn its_own(before: &Declared, after: &Declared) -> Vec<ItsOwn> {
    let every_turn_changed = before.every_turn() != after.every_turn();
    // **The first verb, by name, whose line is not the same on both sides** —
    // new, taken away or reading otherwise.
    let a_verb_changed = before
        .verb_names()
        .chain(after.verb_names())
        .collect::<std::collections::BTreeSet<&str>>()
        .into_iter()
        .find(|verb| before.verb(verb) != after.verb(verb))
        .map(str::to_owned);
    after
        .requests()
        .iter()
        .filter_map(|request| {
            let was = before
                .requests()
                .iter()
                .find(|one| one.named() == request.named());
            let why = match was {
                None => Some(WhyItsOwn::ItIsNew),
                Some(was) if was.line() != request.line() => Some(WhyItsOwn::ItsRoadChanged),
                Some(_) => a_verb_changed
                    .clone()
                    .map(WhyItsOwn::EveryTurnIsShownAVerbOtherwise)
                    .or(every_turn_changed.then_some(WhyItsOwn::EveryTurnIsToldOtherwise)),
            };
            why.map(|why| ItsOwn::because(request.named(), why))
        })
        .collect()
}

/// The requests a change owns against `before`, the base's declaration as
/// its file holds it, with this code's `set` and `verbs` as the change.
///
/// # Errors
/// Why the base's declaration does not read, or the change's would not
/// build.
pub fn its_own_against(
    before: &str,
    set: &crate::the_set::TheSet,
    verbs: &alo_capability::Verbs,
) -> Result<Vec<ItsOwn>, String> {
    let before = Declared::read(before).map_err(|why| why.to_string())?;
    // **In the base's own scheme**, so a change that moves the scheme owns
    // only what it changed (`crate::the_first_fingerprints`).
    let after =
        Declared::by_the_scheme(set, verbs, before.scheme()).map_err(|why| why.to_string())?;
    Ok(its_own(&before, &after))
}

#[cfg(test)]
#[path = "its_own_tests.rs"]
mod tests;
