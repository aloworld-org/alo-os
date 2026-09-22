//! **Which road this machine's answers take, and why** — never inferred from
//! how long one took.
//!
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md): *a GPU
//! changes speed, not capability*. Two things follow, and this file is both of
//! them.
//!
//! **A machine that runs on its processor says so, and says why.** A card of
//! the wrong vendor, a card with no driver, a card too small for the weights —
//! each is a machine that runs on its processor and a sentence a person can
//! read, rather than a failure, and rather than a silence somebody works out
//! from a stopwatch. ADR 0007 rejected *CPU support as a degraded mode, with
//! warnings*, so none of these sentences apologises: they state what is so.
//!
//! **Which road was taken is asked for, not timed.** [`Road::of`] answers from
//! what the runtime reports having loaded and from what the kernel says is on
//! the bus. Nothing here measures a duration, and nothing here has anything to
//! measure one with.
//!
//! # The card is the runtime's answer; the reason is the machine's
//!
//! *Was a card used* is a question only the runtime can answer, because only it
//! knows what it put where — [`crate::Loaded`] carries `/api/ps`'s own figures,
//! and `on_the_gpu_bytes` above zero **is** the card road. Nothing here decides
//! it from a vendor table, which would be a claim about a runtime rather than a
//! measurement of one.
//!
//! *Why not* is the machine's answer, and it comes from [`crate::card`]. The
//! two are kept apart on purpose: a machine with a usable card and a runtime
//! that loaded nothing onto it is a real state
//! ([`WhyTheProcessor::TheRuntimeLeftItThere`]), and a design that worked the
//! road out from the bus alone could not say it.
//!
//! # `min_vram_gb` does not come back as a judge
//!
//! ADR 0007 took the catalogue's `min_vram_gb` out of the offering decision:
//! it answers *will this run well on a card* and cannot answer *will this run
//! at all on this laptop*. Nothing in this file reads it. The figure a card's
//! memory is held against is the **weights' own size**, passed in by whoever is
//! deciding which road to expect, and it decides a road rather than an offer —
//! what a machine offers is still
//! [`crate::Catalogue::to_choose_from_on_cpu`], whatever is or is not on the
//! bus.

use alo_strings::{Filling, Said, Strings};

use crate::card::{ACard, Vendor, WhatDrawsHere};
use crate::measured_on::MeasuredOn;
use crate::runtime::Loaded;
use crate::words::{self, Word};

/// **Which road a question is to be put on.**
///
/// Not a setting and not a preference: there is no place in alo OS where a
/// person chooses this, and there is not going to be one — a machine uses what
/// it has. It exists so that one question can be put to **both** roads on one
/// machine and the two answers held to each other, which is the only way
/// ADR 0007's *a GPU changes speed, not capability* is a measurement rather
/// than a sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichRoad {
    /// Whatever this machine has, which is what every question asks.
    AsTheMachineIs,
    /// This machine's processor, whatever is on its bus.
    TheProcessor,
}

/// **Which road an answer took.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Road {
    /// The runtime put the weights on a graphics card, and this is what it
    /// reported: how much it loaded, and how much of that went to the card.
    OnTheCard {
        /// What the runtime loaded in all.
        loaded_bytes: u64,
        /// How much of that was on the graphics processor.
        on_the_gpu_bytes: u64,
    },
    /// The weights ran on this machine's processor, for this reason.
    OnTheProcessor(WhyTheProcessor),
}

/// **Why a machine runs a model on its processor.**
///
/// Six answers, and not one of them is a failure. ADR 0007's *the CPU is the
/// default* is what that means in the product: a machine with no card runs
/// alo OS and runs its agents, and a machine with a card it cannot use does the
/// same thing for a reason somebody can read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyTheProcessor {
    /// Nothing on this machine's bus draws. The ordinary state of the fleet
    /// this product exists for.
    NoCard,
    /// A card is here, and nothing on this machine can reach it.
    ///
    /// **Never reported as [`NoCard`](Self::NoCard)**, which is the distinction
    /// [`crate::card`] exists to keep: the base is rented and ships no
    /// proprietary driver (ADR 0011), so this is the likely state of every
    /// NVIDIA machine alo OS meets, and a person told *no card* would go
    /// looking for hardware they already own.
    NoDriverForTheCard {
        /// Who made it.
        made_by: Vendor,
    },
    /// A card is here and reachable, and the pinned runtime does not put
    /// weights on this vendor's.
    TheRuntimeCannotUseThatCard {
        /// Who made it.
        made_by: Vendor,
    },
    /// A card the runtime could use, with less memory than the weights need.
    ///
    /// The figures are beside the sentence rather than inside it, which is this
    /// crate's rule about counting in somebody else's language.
    NotEnoughOnTheCard {
        /// What the card said it has.
        the_card_has: u64,
        /// What the weights need.
        the_weights_need: u64,
    },
    /// A card the runtime can use, and it loaded nothing onto it.
    ///
    /// Not a fault of this machine's and not a claim about one: the runtime is
    /// pinned and unpatched (ADR 0006, ADR 0011), and what it did with a card
    /// it could have used is its own business. Said rather than hidden, because
    /// the alternative is a person with a card wondering why nothing changed.
    TheRuntimeLeftItThere,
    /// This machine's own list of what draws was not there to read, so nothing
    /// is claimed about a card either way.
    TheMachineWouldNotSay,
}

impl Road {
    /// **Which road this run took**, from what the runtime loaded and what the
    /// machine says is on its bus.
    ///
    /// `residency` is what the runtime reported for these weights — the entry
    /// [`crate::ModelRuntime::loaded`] answered with, or [`None`] where the
    /// runtime is holding nothing. `cards_it_can_use` is
    /// [`crate::ModelRuntime::cards_it_can_use`], asked of the runtime that
    /// served the run rather than assumed here. `the_weights_need` is the size
    /// of the weights themselves, never the catalogue's `min_vram_gb`.
    #[must_use]
    pub fn of(
        residency: Option<&Loaded>,
        draws_here: &WhatDrawsHere,
        cards_it_can_use: &[Vendor],
        the_weights_need: u64,
    ) -> Self {
        // The runtime's own answer first, and it is the only thing that can
        // say *yes*. A card that is on the bus, reachable and large enough is
        // still a card the runtime may not have used.
        if let Some(loaded) = residency
            && loaded.on_the_gpu_bytes > 0
        {
            return Self::OnTheCard {
                loaded_bytes: loaded.loaded_bytes,
                on_the_gpu_bytes: loaded.on_the_gpu_bytes,
            };
        }
        Self::OnTheProcessor(why(draws_here, cards_it_can_use, the_weights_need))
    }

    /// Whether this is the card road.
    #[must_use]
    pub fn is_the_card(&self) -> bool {
        matches!(self, Self::OnTheCard { .. })
    }

    /// **What the run put where**, as [`MeasuredOn`] states a residency: what
    /// was loaded, and how much of it was on the graphics processor.
    ///
    /// [`None`] on the processor road, because a residency nobody read is not a
    /// residency of nought — and `MeasuredOn` refuses half of one, so a grade
    /// earned without a reading states neither figure.
    #[must_use]
    pub fn residency(&self) -> Option<(u64, u64)> {
        match self {
            Self::OnTheCard {
                loaded_bytes,
                on_the_gpu_bytes,
            } => Some((*loaded_bytes, *on_the_gpu_bytes)),
            Self::OnTheProcessor(_) => None,
        }
    }

    /// **Write this road into a grade**, which is what puts a real device's
    /// figure into `on_the_gpu_bytes` instead of the [`None`] every site in
    /// this repository carried before.
    ///
    /// A run that did not read a residency leaves both figures alone rather
    /// than writing nought into them, for [`residency`](Self::residency)'s
    /// reason.
    pub fn recorded_in(&self, grade: &mut MeasuredOn) {
        if let Some((loaded, on_the_gpu)) = self.residency() {
            grade.loaded_bytes = Some(loaded);
            grade.on_the_gpu_bytes = Some(on_the_gpu);
        }
    }

    /// The string this crate declares for this road.
    #[must_use]
    pub fn word(&self) -> Word {
        match self {
            Self::OnTheCard { .. } => words::ANSWERED_ON_THE_CARD,
            Self::OnTheProcessor(why) => why.word(),
        }
    }

    /// **What a person reads when they ask which road this machine takes.**
    ///
    /// A [`Said`] rather than a [`String`], for
    /// [`crate::InferenceSource::said`]'s reason: this clause goes inside other
    /// sentences, and a sentence is only as translated as its least translated
    /// piece.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &self.filling())
    }

    /// The same clause on a line of its own.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> String {
        self.said(strings).into_text()
    }

    /// What fills the gaps in this road's sentence.
    fn filling(&self) -> Filling {
        match self {
            Self::OnTheCard { .. } => Filling::nothing(),
            Self::OnTheProcessor(why) => why.filling(),
        }
    }
}

impl WhyTheProcessor {
    /// The string this crate declares for this reason.
    #[must_use]
    pub fn word(&self) -> Word {
        match self {
            Self::NoCard => words::THE_PROCESSOR_NO_CARD,
            Self::NoDriverForTheCard { .. } => words::THE_PROCESSOR_NO_DRIVER,
            Self::TheRuntimeCannotUseThatCard { .. } => words::THE_PROCESSOR_ANOTHER_VENDOR,
            Self::NotEnoughOnTheCard { .. } => words::THE_PROCESSOR_NOT_ENOUGH_ON_THE_CARD,
            Self::TheRuntimeLeftItThere => words::THE_PROCESSOR_THE_RUNTIME_LEFT_IT_THERE,
            Self::TheMachineWouldNotSay => words::THE_PROCESSOR_THE_MACHINE_WOULD_NOT_SAY,
        }
    }

    /// What fills the gaps in this reason's sentence.
    fn filling(&self) -> Filling {
        match self {
            Self::NoDriverForTheCard { made_by }
            | Self::TheRuntimeCannotUseThatCard { made_by } => {
                // A vendor is a proper noun and is never translated, which is
                // the rule a filename is held to in `alo-files`.
                Filling::of("vendor", made_by.named())
            }
            Self::NoCard
            | Self::NotEnoughOnTheCard { .. }
            | Self::TheRuntimeLeftItThere
            | Self::TheMachineWouldNotSay => Filling::nothing(),
        }
    }
}

/// Why this machine's weights are on its processor.
///
/// Every card is looked at and the **nearest to usable** decides the sentence,
/// because that is the one a person could act on: a machine with an integrated
/// processor and an NVIDIA card with no driver is told about the driver, not
/// about the vendor of a device that was never going to run a model.
fn why(
    draws_here: &WhatDrawsHere,
    cards_it_can_use: &[Vendor],
    the_weights_need: u64,
) -> WhyTheProcessor {
    match draws_here {
        WhatDrawsHere::TheMachineWouldNotSay => WhyTheProcessor::TheMachineWouldNotSay,
        WhatDrawsHere::NoCard => WhyTheProcessor::NoCard,
        WhatDrawsHere::These(cards) => cards
            .iter()
            .map(|card| about(card, cards_it_can_use, the_weights_need))
            .min_by_key(how_near_usable)
            // `These` is never empty — `WhatDrawsHere::among` answers `NoCard`
            // for an empty list — but a reason is owed either way, and the
            // honest one for a list with nothing in it is that nothing draws.
            .unwrap_or(WhyTheProcessor::NoCard),
    }
}

/// What stands between one card and the weights.
fn about(card: &ACard, cards_it_can_use: &[Vendor], the_weights_need: u64) -> WhyTheProcessor {
    if !cards_it_can_use.contains(&card.made_by()) {
        // A driver would not help, so the vendor is what is said — including
        // for a card with no driver, where naming the driver would send
        // somebody to install one for a card the runtime still could not use.
        return WhyTheProcessor::TheRuntimeCannotUseThatCard {
            made_by: card.made_by(),
        };
    }
    if !card.is_reachable() {
        return WhyTheProcessor::NoDriverForTheCard {
            made_by: card.made_by(),
        };
    }
    match card.memory_bytes() {
        // A card that would not say how much memory it has is not a card that
        // said it has too little: nothing is refused for a figure nobody read.
        Some(has) if has < the_weights_need => WhyTheProcessor::NotEnoughOnTheCard {
            the_card_has: has,
            the_weights_need,
        },
        _ => WhyTheProcessor::TheRuntimeLeftItThere,
    }
}

/// How near one card is to being the one the weights run on — lower is nearer,
/// and the nearest decides what a person is told.
fn how_near_usable(why: &WhyTheProcessor) -> u8 {
    match why {
        WhyTheProcessor::TheRuntimeLeftItThere => 0,
        WhyTheProcessor::NotEnoughOnTheCard { .. } => 1,
        WhyTheProcessor::NoDriverForTheCard { .. } => 2,
        WhyTheProcessor::TheRuntimeCannotUseThatCard { .. } => 3,
        WhyTheProcessor::NoCard | WhyTheProcessor::TheMachineWouldNotSay => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{a_list_of_what_draws, a_machine, in_english, translated};

    /// What the pinned runtime answers, written once so the tests below are
    /// about the reasoning rather than about a list.
    const IT_CAN_USE: [Vendor; 2] = [Vendor::Nvidia, Vendor::Amd];

    /// Four gigabytes of weights, which is about what the pinned entry costs.
    const THE_WEIGHTS: u64 = 4_000_000_000;

    fn loaded(loaded_bytes: u64, on_the_gpu_bytes: u64) -> Loaded {
        Loaded {
            id: "phi-3-mini-instruct".to_owned(),
            loaded_bytes,
            on_the_gpu_bytes,
        }
    }

    /// **A machine with nothing on its bus runs on its processor and says
    /// exactly that.**
    #[test]
    fn a_machine_with_no_card_takes_the_processor_road_and_is_not_a_failure() {
        let drm = a_list_of_what_draws();
        let road = Road::of(
            None,
            &WhatDrawsHere::among(drm.at()),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            road,
            Road::OnTheProcessor(WhyTheProcessor::NoCard),
            "a machine with no card"
        );
        assert!(!road.is_the_card());
        assert_eq!(road.residency(), None);
    }

    /// **A card with no driver is never reported as no card** — the
    /// distinction the owner named, held as a test rather than as a comment.
    #[test]
    fn an_unreachable_card_reads_as_no_driver_and_never_as_no_card() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x10de", None, None);
        let road = Road::of(
            None,
            &WhatDrawsHere::among(drm.at()),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            road,
            Road::OnTheProcessor(WhyTheProcessor::NoDriverForTheCard {
                made_by: Vendor::Nvidia
            }),
        );
    }

    /// **An integrated processor beside a driverless card is told about the
    /// driver**, because that is the one somebody can act on.
    #[test]
    fn the_nearest_card_to_usable_is_the_one_a_person_is_told_about() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x8086", Some("i915"), None);
        drm.holding("card1", "0x10de", None, None);
        let road = Road::of(
            None,
            &WhatDrawsHere::among(drm.at()),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            road,
            Road::OnTheProcessor(WhyTheProcessor::NoDriverForTheCard {
                made_by: Vendor::Nvidia
            }),
        );
    }

    /// **A card of a vendor the runtime does not use is said as that**, not as
    /// a missing driver: installing one would change nothing.
    #[test]
    fn a_card_the_runtime_does_not_use_is_said_by_its_vendor() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x8086", Some("i915"), Some(THE_WEIGHTS * 4));
        let road = Road::of(
            None,
            &WhatDrawsHere::among(drm.at()),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            road,
            Road::OnTheProcessor(WhyTheProcessor::TheRuntimeCannotUseThatCard {
                made_by: Vendor::Intel
            }),
        );
    }

    /// **A card with less memory than the weights need says so**, with both
    /// figures beside the sentence.
    #[test]
    fn a_card_too_small_for_the_weights_is_a_reason_with_both_figures() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x1002", Some("amdgpu"), Some(2_000_000_000));
        let road = Road::of(
            None,
            &WhatDrawsHere::among(drm.at()),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            road,
            Road::OnTheProcessor(WhyTheProcessor::NotEnoughOnTheCard {
                the_card_has: 2_000_000_000,
                the_weights_need: THE_WEIGHTS,
            }),
        );
    }

    /// **A card that would not say how much memory it has is not refused for
    /// a figure nobody read.**
    #[test]
    fn a_silent_card_is_not_treated_as_a_card_that_is_too_small() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x10de", Some("nvidia"), None);
        let road = Road::of(
            None,
            &WhatDrawsHere::among(drm.at()),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            road,
            Road::OnTheProcessor(WhyTheProcessor::TheRuntimeLeftItThere),
        );
    }

    /// **The runtime's own reading is what says the card road was taken**, and
    /// it outranks everything the bus says.
    #[test]
    fn what_the_runtime_loaded_is_what_decides_the_card_road() {
        let drm = a_list_of_what_draws();
        let on_a_machine_that_says_nothing = Road::of(
            Some(&loaded(4_100_000_000, 4_100_000_000)),
            &WhatDrawsHere::among(drm.at()),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            on_a_machine_that_says_nothing,
            Road::OnTheCard {
                loaded_bytes: 4_100_000_000,
                on_the_gpu_bytes: 4_100_000_000,
            },
        );
        assert!(on_a_machine_that_says_nothing.is_the_card());
    }

    /// **A runtime holding weights with nothing on the card is the processor
    /// road**, and the reason still comes from the machine.
    #[test]
    fn nothing_on_the_card_is_the_processor_road_however_much_was_loaded() {
        let drm = a_list_of_what_draws();
        drm.holding("card0", "0x10de", None, None);
        let road = Road::of(
            Some(&loaded(4_100_000_000, 0)),
            &WhatDrawsHere::among(drm.at()),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            road,
            Road::OnTheProcessor(WhyTheProcessor::NoDriverForTheCard {
                made_by: Vendor::Nvidia
            }),
        );
    }

    /// **A machine whose kernel keeps no such list claims nothing either
    /// way.**
    #[test]
    fn a_machine_that_would_not_say_is_not_a_machine_with_no_card() {
        let road = Road::of(
            None,
            &WhatDrawsHere::among(std::path::Path::new("/there-is-no-such-place")),
            &IT_CAN_USE,
            THE_WEIGHTS,
        );
        assert_eq!(
            road,
            Road::OnTheProcessor(WhyTheProcessor::TheMachineWouldNotSay),
        );
    }

    /// **The card road writes a real device's figures into a grade**, which is
    /// what `on_the_gpu_bytes` was waiting for; the processor road writes
    /// neither, because half a residency is refused.
    #[test]
    fn a_grade_carries_what_the_card_held_and_never_half_of_it() {
        let mut grade = a_machine();
        Road::OnTheCard {
            loaded_bytes: 4_583_210_351,
            on_the_gpu_bytes: 4_100_000_000,
        }
        .recorded_in(&mut grade);
        assert_eq!(grade.loaded_bytes, Some(4_583_210_351));
        assert_eq!(grade.on_the_gpu_bytes, Some(4_100_000_000));
        assert_eq!(grade.what_is_wrong_with_it(), None, "{grade:?}");

        let mut untouched = a_machine();
        Road::OnTheProcessor(WhyTheProcessor::NoCard).recorded_in(&mut untouched);
        assert_eq!(untouched.loaded_bytes, None);
        assert_eq!(untouched.on_the_gpu_bytes, None);
        assert_eq!(untouched.what_is_wrong_with_it(), None, "{untouched:?}");
    }

    /// **Every road has a sentence, every sentence is distinct, and the vendor
    /// is in the two that name one.**
    #[test]
    fn every_road_says_something_a_person_can_read() {
        let strings = in_english();
        let every = [
            Road::OnTheCard {
                loaded_bytes: 1,
                on_the_gpu_bytes: 1,
            },
            Road::OnTheProcessor(WhyTheProcessor::NoCard),
            Road::OnTheProcessor(WhyTheProcessor::NoDriverForTheCard {
                made_by: Vendor::Nvidia,
            }),
            Road::OnTheProcessor(WhyTheProcessor::TheRuntimeCannotUseThatCard {
                made_by: Vendor::Intel,
            }),
            Road::OnTheProcessor(WhyTheProcessor::NotEnoughOnTheCard {
                the_card_has: 2,
                the_weights_need: 4,
            }),
            Road::OnTheProcessor(WhyTheProcessor::TheRuntimeLeftItThere),
            Road::OnTheProcessor(WhyTheProcessor::TheMachineWouldNotSay),
        ];
        let said: Vec<String> = every.iter().map(|road| road.shown(&strings)).collect();
        for line in &said {
            assert!(!line.trim().is_empty(), "a road with nothing to say");
            assert!(!line.contains('{'), "a gap nothing filled: {line}");
        }
        let mut distinct = said.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), said.len(), "two roads reading the same");
        assert!(
            said.get(2).is_some_and(|line| line.contains("NVIDIA")),
            "{said:?}"
        );
        assert!(
            said.get(3).is_some_and(|line| line.contains("Intel")),
            "{said:?}"
        );
    }

    /// **The sentence is in the reader's language, and the vendor inside it is
    /// not translated** — a vendor is a proper noun.
    #[test]
    fn the_reason_is_read_in_the_readers_language_with_the_vendor_left_alone() {
        let strings = translated(&[(
            words::THE_PROCESSOR_NO_DRIVER,
            "auf diesem Rechner steckt eine Grafikkarte von {vendor}, die kein Treiber erreichen \
             kann, daher läuft das Modell auf dem Prozessor",
        )]);
        let said = Road::OnTheProcessor(WhyTheProcessor::NoDriverForTheCard {
            made_by: Vendor::Nvidia,
        })
        .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("NVIDIA"), "{said}");
        assert!(said.text().contains("Prozessor"), "{said}");
    }

    /// **Nothing a person reads about a road mentions how long anything took.**
    ///
    /// ADR 0007's *a GPU changes speed, not capability*, read the other way
    /// round: which road was taken is asked for, and a sentence that offered
    /// speed as the answer would teach somebody to infer it from a stopwatch.
    #[test]
    fn no_road_is_described_by_how_long_it_took() {
        for word in words::ABOUT_THE_ROAD {
            let read =
                format!("{} {}", word.says(), word.note().unwrap_or_default()).to_ascii_lowercase();
            for timed in [
                "second", "minute", "faster", "slower", "speed", "quick", "wait",
            ] {
                assert!(
                    !read.contains(timed),
                    "`{}` describes a road by `{timed}`",
                    word.named()
                );
            }
        }
    }

    /// **And none of them apologises.** ADR 0007 rejected CPU support as a
    /// degraded mode with warnings: a default that apologises for itself
    /// teaches people the product is not for them.
    #[test]
    fn no_road_apologises_for_the_machine_it_is_on() {
        for word in words::ABOUT_THE_ROAD {
            let read =
                format!("{} {}", word.says(), word.note().unwrap_or_default()).to_ascii_lowercase();
            for apology in [
                "sorry",
                "unfortunately",
                "only",
                "limited",
                "degraded",
                "fall back",
                "fallback",
                "unsupported",
            ] {
                assert!(
                    !read.contains(apology),
                    "`{}` apologises with `{apology}`",
                    word.named()
                );
            }
        }
    }
}
