//! **Which requests a change owns**: read off two declarations of what the
//! model is offered (`crate::the_declaration`) — the base's committed file and
//! the change's own code — never guessed from which files it touched.
//!
//! A request is the change's own when it is new, when its own line changed,
//! when a verb it asks for is new or its line changed, or when what every turn
//! is told changed, which owns every request. A request taken out of the set
//! is nothing to try.

use crate::the_declaration::Declared;

/// Why a request is a change's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyItsOwn {
    /// The change adds it.
    ItIsNew,
    /// Its definition changed: its words, its road, what a step expects.
    ItsRoadChanged,
    /// It asks for this verb, which the change adds or alters.
    ItAsksFor(String),
    /// What every turn is told changed, which every request is shown.
    EveryTurnIsToldOtherwise,
}

impl WhyItsOwn {
    /// The reason, in words for the summary.
    #[must_use]
    pub fn said(&self) -> String {
        match self {
            Self::ItIsNew => "it is new".to_owned(),
            Self::ItsRoadChanged => "its road changed".to_owned(),
            Self::ItAsksFor(verb) => format!("it asks for `{verb}`, which changed"),
            Self::EveryTurnIsToldOtherwise => "what every turn is told changed".to_owned(),
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
                Some(_) => request
                    .asks_for()
                    .iter()
                    .find(|verb| before.verb(verb) != after.verb(verb))
                    .map(|verb| WhyItsOwn::ItAsksFor(verb.clone()))
                    .or(every_turn_changed.then_some(WhyItsOwn::EveryTurnIsToldOtherwise)),
            };
            why.map(|why| ItsOwn::because(request.named(), why))
        })
        .collect()
}

#[cfg(test)]
#[path = "its_own_tests.rs"]
mod tests;
