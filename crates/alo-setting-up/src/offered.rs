//! The four configurations setup offers, as one value.
//!
//! `docs/features.md` names four ways a machine can be run and holds them apart
//! on purpose — *on this machine*, *on a machine on your network*, *with a
//! provider you added*, *with none at all* — and
//! [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md)
//! gave the fourth the same weight as the other three with no persuasion
//! attached.
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! settled the order: the local one is listed first because it is the one that
//! needs nothing added, **and ordering is not weight**.
//!
//! # Why one enumerated value and not four screens
//!
//! Because *the four* is a fact about the product, and a surface that assembled
//! it from four independent buttons could be built with three of them. [`THE_FOUR`]
//! is the list, its order is fixed here, and a compositor drawing it cannot
//! leave one out without removing a line of this crate — where a reviewer would
//! see it.
//!
//! # alo's own service is not a fifth
//!
//! [ADR 0014](../../../docs/decisions/0014-alos-own-model-is-a-provider-like-any-other.md)
//! makes it **one more provider**, with no default, no pre-selection and no
//! special case anywhere in the code. So it is inside [`Offered::FromAProvider`]
//! and has no variant of its own, exactly as `alo_choosing::Picked` has none —
//! and its absence here is that decision being kept rather than an oversight.
//!
//! # Nothing here says which one is better
//!
//! Each choice has a name and one line saying **what it is**: where the question
//! goes, and what it needs. Not what it costs, not what it is good for, not what
//! most people do. `crate::nudging` is that rule as a check rather than a habit.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// One of the four configurations setup offers.
///
/// A closed list, because the four are what `docs/features.md` promises and a
/// fifth would be a promise nobody made. What is **not** here is a variant for
/// alo's own service: ADR 0014 makes it one more provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offered {
    /// A model on this machine's own disk, answered by the runtime that is
    /// already there.
    OnThisMachine,

    /// A machine this one has been paired with, on the same network.
    ///
    /// ADR 0003's one box serving an office. It is offered because it is one of
    /// the four ways alo OS runs; **this machine keeps no list of paired
    /// machines yet**, so answering with it is refused in words rather than
    /// written as a setting that would quietly do nothing —
    /// `crate::NotSetUp::NoPairedMachine`.
    OnAMachineOnThisNetwork,

    /// A service somewhere else, under a name, an address and a key the person
    /// gives.
    FromAProvider,

    /// No model, no provider and no agent.
    ///
    /// ADR 0009's fourth choice, with the same weight as the other three. It is
    /// an answer rather than a skip, and a machine that took it is finished
    /// with setup.
    NotAtAll,
}

/// The four, in the order setup lists them.
///
/// **The local one is first because it is the one that needs nothing added** —
/// no key, no pairing, no account — and that is ADR 0025's reason rather than a
/// preference. Ordering is not weight: nothing here is selected, the four carry
/// the same shape of sentence, and `crate::SettingUp` cannot be constructed
/// with any of them already chosen.
pub const THE_FOUR: [Offered; 4] = [
    Offered::OnThisMachine,
    Offered::OnAMachineOnThisNetwork,
    Offered::FromAProvider,
    Offered::NotAtAll,
];

impl Offered {
    /// What this choice is called, in the language the person reads.
    #[must_use]
    pub fn called(self, strings: &Strings) -> Said {
        strings.say(&self.name().key(), &Filling::nothing())
    }

    /// One line saying **what this choice is** — where the question goes, and
    /// what it needs — in the language the person reads.
    #[must_use]
    pub fn described(self, strings: &Strings) -> Said {
        strings.say(&self.description().key(), &Filling::nothing())
    }

    /// The string this crate declares for this choice's name.
    #[must_use]
    pub const fn name(self) -> words::Word {
        match self {
            Self::OnThisMachine => words::ON_THIS_MACHINE,
            Self::OnAMachineOnThisNetwork => words::ON_A_MACHINE_ON_THIS_NETWORK,
            Self::FromAProvider => words::FROM_A_PROVIDER,
            Self::NotAtAll => words::NOT_AT_ALL,
        }
    }

    /// The string this crate declares for this choice's one line.
    #[must_use]
    pub const fn description(self) -> words::Word {
        match self {
            Self::OnThisMachine => words::ON_THIS_MACHINE_IS,
            Self::OnAMachineOnThisNetwork => words::ON_A_MACHINE_ON_THIS_NETWORK_IS,
            Self::FromAProvider => words::FROM_A_PROVIDER_IS,
            Self::NotAtAll => words::NOT_AT_ALL_IS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use std::collections::BTreeSet;

    /// **Four, and the local one is first.** ADR 0025's ordering, held here
    /// rather than left to whoever draws the screen.
    #[test]
    fn the_local_choice_is_listed_first_and_there_are_four() {
        assert_eq!(THE_FOUR.len(), 4);
        assert_eq!(THE_FOUR[0], Offered::OnThisMachine);
    }

    /// **The fourth choice is one of the four**, rather than a smaller control
    /// somewhere else on the screen — ADR 0009's *same weight* made structural.
    #[test]
    fn declining_is_one_of_the_four_rather_than_something_beside_them() {
        assert!(THE_FOUR.contains(&Offered::NotAtAll));
    }

    /// **No two of them are the same choice**, which a list written by hand can
    /// stop being.
    #[test]
    fn the_four_are_four_different_choices() {
        let mut seen = BTreeSet::new();
        for offered in THE_FOUR {
            assert!(
                seen.insert(format!("{offered:?}")),
                "{offered:?} is here twice"
            );
        }
        assert_eq!(seen.len(), 4);
    }

    /// **Every one of the four has a name and a line, and no two share
    /// either.** A choice a person cannot read is a choice they cannot weigh
    /// against the others.
    #[test]
    fn each_of_the_four_reads_as_a_name_and_a_line_of_its_own() {
        let strings = in_english();
        let mut said = BTreeSet::new();
        for offered in THE_FOUR {
            let called = offered.called(&strings);
            let described = offered.described(&strings);
            assert!(!called.is_a_bug(), "{called}");
            assert!(!described.is_a_bug(), "{described}");
            assert!(said.insert(called.into_text()), "{offered:?}");
            assert!(said.insert(described.into_text()), "{offered:?}");
        }
        assert_eq!(said.len(), 8);
    }

    /// **alo's own service has no variant of its own.** ADR 0014 makes it one
    /// more provider, and a fifth choice here would be that decision reversed
    /// by a surface.
    #[test]
    fn alos_own_service_is_not_a_fifth_choice() {
        let strings = in_english();
        for offered in THE_FOUR {
            for said in [offered.called(&strings), offered.described(&strings)] {
                let text = said.text().to_lowercase();
                assert!(
                    !text.split_whitespace().any(|word| word
                        .trim_matches(|character: char| !character.is_ascii_alphanumeric())
                        == "alo"),
                    "{offered:?} names alo's own service as though it were a choice of its \
                     own: {said}"
                );
            }
        }
    }
}
