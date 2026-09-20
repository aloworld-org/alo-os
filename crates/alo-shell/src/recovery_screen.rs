//! The recovery screen: what a person is offered when the machine they were
//! running will not do, and the one thing they can choose.
//!
//! `ROADMAP.md` v0.5: *recovery and rollback screen — reachable when the
//! workspace is not.*
//!
//! # It decides nothing
//!
//! Whether going back can be offered at all is
//! `alo_keeping_up::GoingBack::offered`, decided from what the base reports and
//! from the record's last change, **before anything is offered** — so a return
//! that cannot be done says so instead of being offered and failing halfway.
//! Every sentence this screen shows is that crate's: the offer is
//! `GoingBack::said`, the refusal is `CannotGoBack::said`, and the two moments
//! a person can choose are `GoingBack::when_word`. Nothing is worded here.
//!
//! # It carries nothing out
//!
//! Choosing hands back the `GoingBack` that was decided and the moment the
//! person chose, and the screen is gone. Making that happen is
//! `alo_keeping_up::Returning` carried out through the broker; a compositor
//! that replaced the operating system itself would be a drawing crate with the
//! deepest authority on the machine.
//!
//! # Nothing is preselected
//!
//! Going back replaces the whole system a person is running, and one of the two
//! moments restarts the machine underneath them. A screen that arrived with an
//! answer already selected would turn a single Enter — held down from whatever
//! failed a moment ago — into that. So nothing is selected until a person moves
//! to it, and Enter before then does nothing.
//!
//! # It is reachable before anybody has signed in
//!
//! Nothing here names an account, a session or a greeting, and nothing here
//! opens a file: the screen is handed what the base reported and the person's
//! vocabulary, both of which exist before sign-in. `tests/recovery_source.rs`
//! reads these files and holds them to it — which is the same test that holds
//! *it touches nothing a person owns*.

use alo_keeping_up::{Changed, Deployments, GoingBack, WhenItApplies};
use alo_strings::{Filling, Said, Strings};

use crate::recovery_keys::RecoveryKey;

/// The two moments going back can happen at, in reading order: the one that
/// leaves the machine alone until the person restarts it themselves, then the
/// one that restarts it now.
pub const THE_TWO_MOMENTS: [WhenItApplies; 2] = [
    WhenItApplies::AtTheNextRestart,
    WhenItApplies::NowBecauseThePersonAsked,
];

/// The recovery screen.
pub struct RecoveryScreen {
    /// What it stands at, decided once by `alo-keeping-up`.
    stands: Stands,
    /// The moment Enter would choose, which is none until a person moves.
    selected: Option<WhenItApplies>,
}

/// What the screen stands at.
enum Stands {
    /// Going back can be offered: what was decided, the sentence a person
    /// approves, and the two moments said in their language.
    Offered {
        /// What was decided, handed back whole when a person chooses.
        going_back: GoingBack,
        /// The sentence a person approves.
        offer: Said,
        /// Each moment and its words.
        moments: Vec<(WhenItApplies, Said)>,
    },
    /// It cannot be offered, and this is why — in `alo-keeping-up`'s words,
    /// never in a sentence written here.
    Cannot(Said),
}

/// What the screen draws now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryShows<'a> {
    /// Going back is offered: the sentence, the two moments, and which one
    /// Enter would choose.
    Offered {
        /// The sentence a person approves.
        offer: &'a Said,
        /// Each moment and its words, in reading order.
        moments: &'a [(WhenItApplies, Said)],
        /// The moment Enter would choose, or none.
        selected: Option<WhenItApplies>,
    },
    /// Going back cannot be offered, and this is why.
    CannotGoBack(&'a Said),
}

/// What became of one key press.
pub enum RecoveryChosen {
    /// Nothing was chosen; this is the screen to draw.
    Still(Box<RecoveryScreen>),
    /// The person chose to go back. There is no screen in here, on purpose:
    /// what happens next is `alo_keeping_up::Returning` carried out through
    /// the broker, and nothing on this screen is answered twice.
    GoBack {
        /// What was decided when the screen was made, unchanged.
        going_back: Box<GoingBack>,
        /// The moment the person chose.
        when: WhenItApplies,
    },
}

impl RecoveryScreen {
    /// The screen for a machine whose base reports `deployments` and whose
    /// record last says `last_changed`, in the person's language.
    ///
    /// Takes `alo_keeping_up::GoingBack::offered`'s answer as it is, so what a
    /// machine that cannot go back shows is that crate's decision rather than a
    /// second reading of the disk here.
    #[must_use]
    pub fn of(
        deployments: &Deployments,
        last_changed: Option<&Changed>,
        strings: &Strings,
    ) -> Self {
        Self {
            stands: match GoingBack::offered(deployments, last_changed) {
                Ok(going_back) => Stands::Offered {
                    offer: going_back.said(strings),
                    moments: THE_TWO_MOMENTS
                        .into_iter()
                        .map(|when| (when, said_for(when, strings)))
                        .collect(),
                    going_back,
                },
                Err(why) => Stands::Cannot(why.said(strings)),
            },
            selected: None,
        }
    }

    /// What the screen draws now.
    #[must_use]
    pub fn shows(&self) -> RecoveryShows<'_> {
        match &self.stands {
            Stands::Offered { offer, moments, .. } => RecoveryShows::Offered {
                offer,
                moments,
                selected: self.selected,
            },
            Stands::Cannot(said) => RecoveryShows::CannotGoBack(said),
        }
    }

    /// Whether going back is offered at all.
    #[must_use]
    pub const fn is_offered(&self) -> bool {
        matches!(self.stands, Stands::Offered { .. })
    }

    /// One key press, and what became of the screen.
    ///
    /// Moving selects a moment; choosing gives the selected one and ends the
    /// screen. Choosing while nothing is selected does nothing, and a screen
    /// that cannot offer going back takes no key that does anything at all —
    /// there is nothing on it to choose.
    #[must_use]
    pub fn pressed(mut self, key: RecoveryKey) -> RecoveryChosen {
        let order: Vec<WhenItApplies> = match &self.stands {
            Stands::Offered { moments, .. } => moments.iter().map(|(when, _)| *when).collect(),
            // Nothing is offered, so no key does anything at all.
            Stands::Cannot(_) => return RecoveryChosen::Still(Box::new(self)),
        };
        match key {
            RecoveryKey::Next => {
                self.selected = moved(&order, self.selected, true);
                RecoveryChosen::Still(Box::new(self))
            }
            RecoveryKey::Previous => {
                self.selected = moved(&order, self.selected, false);
                RecoveryChosen::Still(Box::new(self))
            }
            RecoveryKey::Nothing => RecoveryChosen::Still(Box::new(self)),
            RecoveryKey::Choose => {
                let Some(when) = self.selected else {
                    // Nothing is selected, so there is nothing to choose.
                    return RecoveryChosen::Still(Box::new(self));
                };
                let selected = self.selected;
                match self.stands {
                    Stands::Offered { going_back, .. } => RecoveryChosen::GoBack {
                        going_back: Box::new(going_back),
                        when,
                    },
                    // `order` was taken from an offer, so this cannot happen;
                    // the screen goes back exactly as it was rather than
                    // choosing anything.
                    stands @ Stands::Cannot(_) => {
                        RecoveryChosen::Still(Box::new(Self { stands, selected }))
                    }
                }
            }
        }
    }
}

/// The moment beside `selected` in `order`, wrapping — and the end a person
/// moved towards when nothing is selected yet, so Tab reaches the first moment
/// and Shift+Tab the last.
fn moved(
    order: &[WhenItApplies],
    selected: Option<WhenItApplies>,
    forward: bool,
) -> Option<WhenItApplies> {
    let count = order.len();
    if count == 0 {
        return None;
    }
    let at = match selected.and_then(|now| order.iter().position(|when| *when == now)) {
        Some(at) if forward => (at + 1) % count,
        Some(at) => (at + count - 1) % count,
        None if forward => 0,
        None => count - 1,
    };
    order.get(at).copied()
}

/// The words a person chooses a moment by, in their language.
fn said_for(when: WhenItApplies, strings: &Strings) -> Said {
    strings.say(&GoingBack::when_word(when).key(), &Filling::nothing())
}

#[cfg(test)]
#[path = "recovery_screen_tests.rs"]
mod tests;
