//! **Every request, tried several times**, the verdict, and the summary a
//! person reads: each request, how many tries took the road, and where the
//! others went.

use alo_capability::Verbs;
use alo_strings::Strings;

use crate::a_road::ARequest;
use crate::a_try::{TheModel, Tried};
use crate::the_set::TheSet;

/// One request and every try of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tallied {
    request: ARequest,
    tries: Vec<Tried>,
}

impl Tallied {
    /// The request and its tries.
    #[must_use]
    pub const fn of(request: ARequest, tries: Vec<Tried>) -> Self {
        Self { request, tries }
    }

    /// The request.
    #[must_use]
    pub const fn request(&self) -> &ARequest {
        &self.request
    }

    /// Every try, in order.
    #[must_use]
    pub fn tries(&self) -> &[Tried] {
        &self.tries
    }

    /// How many took the road.
    #[must_use]
    pub fn on_the_road(&self) -> usize {
        0
    }
}

/// What the whole run says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Every try of every request took the road.
    EveryTryTookTheRoad,
    /// At least one try left the road.
    SomeLeftTheRoad,
    /// Nothing left the road, and something could not be asked — or nothing
    /// was tried — so the run does not say the road holds.
    NotMeasured,
}

/// Try every request in `set` `tries` times against `model`, saying each try
/// as it ends through `told`.
pub fn tallying(
    set: &TheSet,
    tries: usize,
    verbs: &Verbs,
    strings: &Strings,
    model: &mut dyn TheModel,
    told: &mut dyn FnMut(&ARequest, usize, &Tried),
) -> Vec<Tallied> {
    let _ = (set, tries, verbs, strings, model, told);
    Vec::new()
}

/// What `tallied` says as a whole.
#[must_use]
pub fn verdict(tallied: &[Tallied]) -> Verdict {
    let _ = tallied;
    Verdict::EveryTryTookTheRoad
}

/// The summary, in plain words, headed by what was asked of what.
#[must_use]
pub fn summary(tallied: &[Tallied], asked_of: &str) -> String {
    let _ = (tallied, asked_of);
    String::new()
}

#[cfg(test)]
#[path = "tallied_tests.rs"]
mod tests;
