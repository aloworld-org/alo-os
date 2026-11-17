//! **Every request, tried several times**, the verdict, and the summary a
//! person reads: each request, how many tries took the road, and where the
//! others went.

use alo_capability::Verbs;
use alo_strings::Strings;

use crate::a_road::ARequest;
use crate::a_try::{TheLevel, TheModel, Tried, Went, one_try_at};
use crate::the_set::TheSet;

/// One request and every try of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tallied {
    /// The request.
    request: ARequest,
    /// Every try of it, in order.
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
        self.tries
            .iter()
            .filter(|tried| tried.took_the_road())
            .count()
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

impl Verdict {
    /// The exit code a run ends with: 0 when every try took the road, 1 when
    /// any left it, and 2 when nothing was measured — **never 0 for a run that
    /// could not ask**, so a provider that is down cannot read as a pass.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::EveryTryTookTheRoad => 0,
            Self::SomeLeftTheRoad => 1,
            Self::NotMeasured => 2,
        }
    }
}

/// The most of a model's line a summary shows.
const AS_MUCH_OF_A_LINE_AS_A_SUMMARY_SHOWS: usize = 400;

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
    tallying_at(
        TheLevel::FullAccess,
        set,
        tries,
        verbs,
        strings,
        model,
        told,
    )
}

/// [`tallying`] at a level (`crate::a_try::TheLevel`).
pub fn tallying_at(
    level: TheLevel,
    set: &TheSet,
    tries: usize,
    verbs: &Verbs,
    strings: &Strings,
    model: &mut dyn TheModel,
    told: &mut dyn FnMut(&ARequest, usize, &Tried),
) -> Vec<Tallied> {
    set.requests()
        .iter()
        .map(|request| {
            let tried = (1..=tries)
                .map(|number| {
                    let tried = one_try_at(level, request, set.may_reach(), verbs, strings, model);
                    told(request, number, &tried);
                    tried
                })
                .collect();
            Tallied::of(request.clone(), tried)
        })
        .collect()
}

/// What `tallied` says as a whole.
#[must_use]
pub fn verdict(tallied: &[Tallied]) -> Verdict {
    let every: Vec<&Tried> = tallied.iter().flat_map(|one| one.tries.iter()).collect();
    let tried_each = tallied.iter().all(|one| !one.tries.is_empty());
    if every
        .iter()
        .any(|tried| matches!(tried.went(), Went::WentWrong(_)))
    {
        Verdict::SomeLeftTheRoad
    } else if every.is_empty() || !tried_each || !every.iter().all(|tried| tried.took_the_road()) {
        Verdict::NotMeasured
    } else {
        Verdict::EveryTryTookTheRoad
    }
}

/// The summary, in plain words, headed by what was asked of what.
#[must_use]
pub fn summary(tallied: &[Tallied], asked_of: &str) -> String {
    let tries = tallied.first().map_or(0, |one| one.tries.len());
    let calls: usize = tallied
        .iter()
        .flat_map(|one| one.tries.iter())
        .map(Tried::asked)
        .sum();
    let mut said = format!(
        "## What alo's model chose, tried {tries} times per request\n\n\
         Set version {}, asked of {asked_of}, with the instructions and verbs a turn \
         is given. Nothing was carried out on any machine. {calls} model calls in all.\n\n",
        crate::the_set::THE_VERSION
    );
    said.push_str(match verdict(tallied) {
        Verdict::EveryTryTookTheRoad => {
            "**Every try of every request took the expected road.**\n\n"
        }
        Verdict::SomeLeftTheRoad => {
            "**The check did not pass**: at least one try left the expected road.\n\n"
        }
        Verdict::NotMeasured => {
            "**Nothing was measured**: the model could not be asked every time, so this \
             says nothing about the road.\n\n"
        }
    });
    said.push_str("| Request | Took the road | Where the other tries went |\n|---|---|---|\n");
    let mut looked: Vec<String> = Vec::new();
    let mut refused: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut the_other_way: Vec<String> = Vec::new();
    for one in tallied {
        let passes_refused = one
            .tries
            .iter()
            .filter(|tried| tried.took_the_road() && tried.refused_before_asking() > 0)
            .count();
        if passes_refused > 0 {
            refused.push(format!(
                "- {}: {passes_refused} of its passes {} refused first and went on",
                one.request.named(),
                if passes_refused == 1 { "was" } else { "were" }
            ));
        }
        let passes_the_other_way = one
            .tries
            .iter()
            .filter(|tried| tried.took_the_road() && tried.by_the_other_road())
            .count();
        if passes_the_other_way > 0 {
            the_other_way.push(format!(
                "- {}: {passes_the_other_way} of its passes took another road",
                one.request.named()
            ));
        }
        let passes_that_looked = one
            .tries
            .iter()
            .filter(|tried| tried.took_the_road() && tried.looked_aside() > 0)
            .count();
        if passes_that_looked > 0 {
            looked.push(format!(
                "- {}: {passes_that_looked} of its passes looked aside",
                one.request.named()
            ));
        }
        let passes_that_skipped = one
            .tries
            .iter()
            .filter(|tried| tried.took_the_road() && tried.skipped_looks() > 0)
            .count();
        if passes_that_skipped > 0 {
            skipped.push(format!(
                "- {}: {passes_that_skipped} of {} passes",
                one.request.named(),
                one.on_the_road()
            ));
        }
        let elsewhere: Vec<String> = one
            .tries
            .iter()
            .enumerate()
            .filter_map(|(at, tried)| match tried.went() {
                Went::TheRoad => None,
                Went::WentWrong(why) => Some(format!("try {}: {why}", at + 1)),
                Went::NotAnswered(why) => Some(format!(
                    "try {}: the model could not be asked ({why})",
                    at + 1
                )),
            })
            .collect();
        said.push_str(&format!(
            "| {} | {} of {} | {} |\n",
            one.request.named(),
            one.on_the_road(),
            one.tries.len(),
            if elsewhere.is_empty() {
                "—".to_owned()
            } else {
                elsewhere.join("; ").replace('|', "\\|")
            }
        ));
    }
    // **A pass that needed a look aside is said**, with the cap, so it is not
    // counted silently as a clean one.
    said.push_str(&format!(
        "\nA try may take a read the machine would answer aside of its road, at most {} a \
         try, and must still take every step after it. ",
        crate::a_try::THE_MOST_LOOKS_ASIDE
    ));
    if looked.is_empty() {
        said.push_str("No pass looked aside.\n");
    } else {
        said.push_str("Passes that looked aside:\n\n");
        said.push_str(&looked.join("\n"));
        said.push('\n');
    }
    // **A pass that skipped a look is counted**: going straight to an act the
    // person named is a road the machine takes too — it finds the control by
    // name — so it passes, and how often is said.
    said.push_str(
        "\nA look on the road — reading a window before acting in it — may be skipped, as the \
         machine finds what is named without one. ",
    );
    if skipped.is_empty() {
        said.push_str("No pass skipped a look.\n");
    } else {
        said.push_str("Passes that went straight to the act:\n\n");
        said.push_str(&skipped.join("\n"));
        said.push('\n');
    }
    // **A pass the machine refused first is said** too: on a machine that
    // refusal is shown to the model before anybody is asked, so the pass is
    // real, and it is still not a clean one.
    said.push_str(
        "\nA step the machine refuses before asking anybody is answered with the refusal, \
         as on a machine, and the try may go on from it. ",
    );
    if refused.is_empty() {
        said.push_str("No pass was refused first.\n");
    } else {
        said.push_str("Passes refused first:\n\n");
        said.push_str(&refused.join("\n"));
        said.push('\n');
    }
    // **A pass by a request's other road is said**, so the longer way to the
    // same place is counted rather than hidden among the passes.
    said.push_str(
        "\nA request may name other roads to the same place, which a try may take from where \
         they part. ",
    );
    if the_other_way.is_empty() {
        said.push_str("No pass took another road.\n");
    } else {
        said.push_str("Passes by another road:\n\n");
        said.push_str(&the_other_way.join("\n"));
        said.push('\n');
    }
    for one in tallied {
        said.push_str(&format!(
            "\n### {}\n\nAsked: *{}*\n\nExpected: {}\n\n",
            one.request.named(),
            one.request.words(),
            one.request.the_road_written()
        ));
        for (at, tried) in one.tries.iter().enumerate() {
            said.push_str(&format!(
                "- try {}: {} ({} model calls)\n",
                at + 1,
                if tried.chose().is_empty() {
                    "nothing".to_owned()
                } else {
                    tried.chose().join(" → ")
                },
                tried.asked()
            ));
            if let (Went::WentWrong(_), Some(line)) = (tried.went(), tried.said_last()) {
                let line: String = line
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(AS_MUCH_OF_A_LINE_AS_A_SUMMARY_SHOWS)
                    .collect();
                said.push_str(&format!(
                    "  - its last line: `{}`\n",
                    line.replace('`', "'")
                ));
            }
        }
    }
    said
}

#[cfg(test)]
#[path = "tallied_tests.rs"]
mod tests;
