//! **Compare with main** (the owner, 2026-11-20, *"Compare with main"*): a
//! change that owns the whole set — it alters what every turn is told, or a
//! verb as the model reads it — is not held to the bar absolutely, because
//! the whole set wobbles from run to run and the absolute rule would fail
//! good changes by chance. It is tried, with its base, the bar's number of
//! times, and blocks only where a request is **clearly worse** than its
//! base, **falls below the bar where its base met it**, or **breaks the
//! ratchet** (`crate::what_each_has_reached`). A request below the bar on its
//! base may stay below it, no worse.
//!
//! **Flag, then confirm** (approved 2026-11-20): on the first tries a fault
//! is only a flag. A flagged request is tried
//! [`crate::the_bar::THE_CONFIRMING_TRIES`] more on both
//! sides and judged on the totals, with every threshold scaled to them — at
//! ten and ten, a drop of four of twenty, a floor of eighteen of twenty where
//! the base had eighteen, and twenty of twenty where the record holds it.
//!
//! **Why**, measured on 2026-11-20: two whole-set runs at ten tries of one
//! model-facing tree (CI runs 44779963562 and 44793910551) disagreed by two or
//! more tries on 6 of 50 request-and-level pairs — "close an application"
//! 10 then 8, "type into a search box without saving" 9 then 1 — so judged on
//! ten tries alone, a change that altered nothing would have been blocked.

use std::collections::BTreeMap;

use crate::counted::Counted;
use crate::the_bar::{THE_BAR, THE_CLEARLY_WORSE_DROP, THE_CONFIRMING_TRIES};
use crate::what_each_has_reached::{Measured, Reached, the_level_named};

/// How one request at one level compares with its base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Judged {
    /// No worse than its base, by the rules below.
    Held,
    /// A fault on the first tries: flagged, to be confirmed on more.
    ToConfirm,
    /// Clearly worse than its base: a drop of at least
    /// [`THE_CLEARLY_WORSE_DROP`] in every [`THE_BAR`]'s tries.
    ClearlyWorse,
    /// Below the bar where its base met it.
    BelowTheBarWhereItsBaseMetIt,
    /// New in the change, and below the bar.
    BelowTheBar,
    /// Recorded as having taken every try at this level, and did not.
    BrokeTheRatchet,
    /// Too few tries, or a base tried another number of times, to compare.
    NotMeasured,
}

impl Judged {
    /// The judgement, in words for the summary.
    #[must_use]
    pub const fn said(&self) -> &'static str {
        match self {
            Self::Held => "no worse than its base",
            Self::ToConfirm => "flagged: to be tried again on both sides",
            Self::ClearlyWorse => "clearly worse than its base",
            Self::BelowTheBarWhereItsBaseMetIt => "below the bar, where its base met it",
            Self::BelowTheBar => "new, and below the bar",
            Self::BrokeTheRatchet => "below every try, where the record says it took every try",
            Self::NotMeasured => {
                "not measured: too few tries, or tries the model could not be asked"
            }
        }
    }

    /// Whether this blocks the change.
    #[must_use]
    pub const fn blocks(&self) -> bool {
        matches!(
            self,
            Self::ClearlyWorse
                | Self::BelowTheBarWhereItsBaseMetIt
                | Self::BelowTheBar
                | Self::BrokeTheRatchet
        )
    }
}

/// One request at one level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The level's name.
    level: String,
    /// The request.
    named: String,
    /// The change's tries on the road, of how many.
    change: (usize, usize),
    /// The base's, if it had the request.
    base: Option<(usize, usize)>,
    /// The tries the model could not be asked: the change's, and the base's.
    not_asked: (usize, usize),
    /// How it was judged.
    judged: Judged,
}

impl Row {
    /// How it was judged.
    #[must_use]
    pub const fn judged(&self) -> &Judged {
        &self.judged
    }

    /// The change's tries on the road, of how many.
    #[must_use]
    pub const fn change(&self) -> (usize, usize) {
        self.change
    }

    /// The base's, if it had the request.
    #[must_use]
    pub const fn base(&self) -> Option<(usize, usize)> {
        self.base
    }
}

/// A level's name and a request.
type TheRequestAt = (String, String);

/// On the road, tried, and not asked.
type Sums = (usize, usize, usize);

/// Each level and request's totals — on the road, tried, not asked — in the
/// order first measured.
fn totals(measured: &Measured) -> Vec<(TheRequestAt, Sums)> {
    let mut order: Vec<TheRequestAt> = Vec::new();
    let mut sums: BTreeMap<TheRequestAt, Sums> = BTreeMap::new();
    for ((level, named, on, of), not_asked) in measured.each().zip(measured.not_asked()) {
        let key = (level.to_owned(), named.to_owned());
        let sum = sums.entry(key.clone()).or_insert_with(|| {
            order.push(key);
            (0, 0, 0)
        });
        sum.0 += on;
        sum.1 += of;
        sum.2 += not_asked;
    }
    order
        .into_iter()
        .map(|key| {
            let sum = sums.get(&key).copied().unwrap_or((0, 0, 0));
            (key, sum)
        })
        .collect()
}

/// How `change` compares with `base`, `reached` saying whether the record
/// holds the request at this level.
fn judging(change: (usize, usize), base: Option<(usize, usize)>, reached: bool) -> Judged {
    let (on, of) = change;
    if of < THE_BAR.tries() || base.is_some_and(|(_, base_of)| base_of != of) {
        return Judged::NotMeasured;
    }
    let confirmed = of >= THE_BAR.tries() + THE_CONFIRMING_TRIES;
    let fault = if reached && on != of {
        Some(Judged::BrokeTheRatchet)
    } else {
        match base {
            None => (!THE_BAR.holds(on, of, false)).then_some(Judged::BelowTheBar),
            Some((base_on, _)) => {
                if THE_BAR.holds(base_on, of, false) && !THE_BAR.holds(on, of, false) {
                    Some(Judged::BelowTheBarWhereItsBaseMetIt)
                } else if base_on.saturating_sub(on).saturating_mul(THE_BAR.tries())
                    >= THE_CLEARLY_WORSE_DROP.saturating_mul(of)
                {
                    Some(Judged::ClearlyWorse)
                } else {
                    None
                }
            }
        }
    };
    match fault {
        None => Judged::Held,
        Some(_) if !confirmed => Judged::ToConfirm,
        Some(fault) => fault,
    }
}

/// Every request the change measured, compared with its base — each judged
/// on its totals over every round measured.
#[must_use]
pub fn compared(change: &Measured, base: &Measured, record: &Reached) -> Vec<Row> {
    let base: BTreeMap<TheRequestAt, Sums> = totals(base).into_iter().collect();
    totals(change)
        .into_iter()
        .map(|((level, named), (on, of, not_asked))| {
            let reached = the_level_named(&level).is_some_and(|at| record.has(at, &named));
            let base_sum = base.get(&(level.clone(), named.clone())).copied();
            let base = base_sum.map(|(base_on, base_of, _)| (base_on, base_of));
            // **A try the model could not be asked makes the comparison not
            // measured**, on either side (the coordinator, 2026-11-20): nought
            // of ten beside nought of ten, with the provider out of credit,
            // read as "no worse than its base" and passed (run 44854527313).
            let not_asked_anywhere =
                not_asked > 0 || base_sum.is_some_and(|(_, _, base_not_asked)| base_not_asked > 0);
            Row {
                judged: if not_asked_anywhere {
                    Judged::NotMeasured
                } else {
                    judging((on, of), base, reached)
                },
                level,
                named,
                change: (on, of),
                base,
                not_asked: (
                    not_asked,
                    base_sum.map_or(0, |(_, _, base_not_asked)| base_not_asked),
                ),
            }
        })
        .collect()
}

/// The level and request of each row flagged to be confirmed.
#[must_use]
pub fn the_flagged(rows: &[Row]) -> Vec<(String, String)> {
    rows.iter()
        .filter(|row| row.judged == Judged::ToConfirm)
        .map(|row| (row.level.clone(), row.named.clone()))
        .collect()
}

/// What the rows say as a whole: slipped where any blocks, not measured where
/// any is flagged or could not be compared, held otherwise.
#[must_use]
pub fn verdict_of(rows: &[Row]) -> Counted {
    if rows.iter().any(|row| row.judged.blocks()) {
        Counted::ItsOwnSlipped
    } else if rows.is_empty()
        || rows
            .iter()
            .any(|row| matches!(row.judged, Judged::ToConfirm | Judged::NotMeasured))
    {
        Counted::NotMeasured
    } else {
        Counted::ItsOwnHeld
    }
}

/// The summary: each request at each level, the change beside its base and
/// how it was judged, flagged ones and confirmed ones alike.
#[must_use]
pub fn summary_of(rows: &[Row], asked_of: &str) -> String {
    let mut said = format!(
        "## This change compared with its base\n\nIt owns the whole set, so every request was \
         tried on the change and on its base, asked of {asked_of}. A fault on the first {} tries \
         is a flag; a flagged request is tried {} more on both and judged on the totals.\n\n",
        THE_BAR.tries(),
        THE_CONFIRMING_TRIES
    );
    said.push_str(match verdict_of(rows) {
        Counted::ItsOwnSlipped => {
            "**The check did not pass**: a request is worse than its base.\n\n"
        }
        Counted::NotMeasured | Counted::ToConfirm | Counted::ToCompareWithItsBase => {
            "**Nothing was decided yet**: a request is flagged, or could not be compared.\n\n"
        }
        Counted::ItsOwnHeld | Counted::NothingOfItsOwn => {
            "**No request is worse than its base.**\n\n"
        }
    });
    said.push_str("| Level | Request | Change | Base | Judged |\n|---|---|---|---|---|\n");
    for row in rows {
        // **A refused try is said as refused**, never folded into the misses.
        let refused = |count: usize| {
            if count > 0 {
                format!(" ({count} not asked)")
            } else {
                String::new()
            }
        };
        let base = row.base.map_or_else(
            || "new".to_owned(),
            |(on, of)| format!("{on} of {of}{}", refused(row.not_asked.1)),
        );
        said.push_str(&format!(
            "| {} | {} | {} of {}{} | {base} | {} |\n",
            row.level,
            row.named,
            row.change.0,
            row.change.1,
            refused(row.not_asked.0),
            row.judged.said()
        ));
    }
    said
}

#[cfg(test)]
#[path = "compared_with_its_base_tests.rs"]
mod tests;
