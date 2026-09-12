//! What a person reads when weights they chose are larger than this machine's
//! memory — the whole of it, once.
//!
//! Two lines, in one value, in the order they are read. The first is
//! `alo-models`', because what a model costs on this machine is that crate's
//! knowledge and its sentence already says the model runs; the second is this
//! crate's, and it is the *once*: alo OS will run these weights whenever they
//! are chosen and will not raise their size again by itself. The reason they
//! are gathered here rather than assembled by whoever is drawing them is the
//! guarantee this crate exists for — **a warning that was suppressed produces
//! no sentence anywhere.** A surface that built its own two lines out of the
//! cost would be a surface that could draw them on the turn
//! [`crate::Warning`] said nothing about.
//!
//! # What it does not say
//!
//! Nothing about the licence and nothing about the measurement. Those two
//! lines are `alo_models::Weights::lines`' and belong at the moment the
//! weights are added, where they are true and where the person is deciding;
//! a warning about size that repeated them would be the machine arguing with
//! somebody about a model they have already chosen. And nothing that leans:
//! not toward a catalogued model, not toward a smaller one, not toward a
//! provider. The cost is said, the promise to run is said, and that is all.

use alo_models::{Cost, Weights};
use alo_strings::{Filling, Said, Strings};

use crate::too_large::TooLarge;
use crate::words;

/// How many lines one warning is.
///
/// Written down so that [`WarnedOnce::lines`] and whoever reads its answer
/// cannot disagree about how many there are.
pub const HOW_MANY_WARNING_LINES: usize = 2;

/// What a person is told, once, about weights larger than this machine's
/// memory.
///
/// Made by [`crate::Warning::about`] and by nothing else. There is no
/// constructor from a sentence, no public field, no `From` and no
/// deserialiser — so what reaches a screen came from weights that were really
/// costed on this machine, through the one place that knows whether it has
/// been said before.
///
/// **Deliberately not `Clone`**, which is [`crate::ToldOnce`]'s rule for its
/// reason: a clone would be a second copy of a warning that was permitted
/// once.
///
/// A warning cannot be made out of a sentence somebody wrote, and this is what
/// says so — there is no such constructor to call:
///
/// ```compile_fail
/// let _ = alo_telling::WarnedOnce::saying("these weights are too large");
/// ```
///
/// It fails with **E0599, no function or associated item named `saying`**,
/// and not on a typo.
#[derive(Debug, PartialEq)]
pub struct WarnedOnce {
    /// The weights this is about, whole, so the sentence about their cost is
    /// the one `alo-models` words and this crate cannot grow a second account
    /// of one moment.
    weights: Weights,
    /// What they cost on the machine this warning is about.
    cost: Cost,
}

impl WarnedOnce {
    /// Made by [`crate::Warning::about`] and by nothing else.
    pub(crate) fn about(weights: Weights, cost: Cost) -> Self {
        Self { weights, cost }
    }

    /// Which warning this is: these weights, at this cost.
    ///
    /// Always [`Some`] in practice, because this value is only made for the
    /// case worth saying; it is an [`Option`] because [`TooLarge::of`] is the
    /// one road to an identity and it is honest about weights that fit.
    #[must_use]
    pub fn too_large(&self) -> Option<TooLarge> {
        TooLarge::of(&self.weights, self.cost.machine_gb())
    }

    /// The weights this warning is about.
    #[must_use]
    pub fn weights(&self) -> &Weights {
        &self.weights
    }

    /// What they cost here — both numbers, for whoever writes them the way
    /// this region writes a size.
    #[must_use]
    pub fn cost(&self) -> Cost {
        self.cost
    }

    /// What it will cost — `alo_models::Cost::said`, unchanged: *larger than
    /// the memory this machine has; alo OS will still run them*.
    #[must_use]
    pub fn what_it_costs(&self, strings: &Strings) -> Said {
        self.cost.said(strings)
    }

    /// That this is said once, and the weights run whenever they are chosen.
    ///
    /// This crate's own, and the *once — and then run anyway* half of the
    /// promise said out loud.
    #[must_use]
    pub fn runs_them_anyway(&self, strings: &Strings) -> Said {
        strings.say(&words::RUNS_THEM_ANYWAY.key(), &Filling::nothing())
    }

    /// The whole warning, in the order a person reads it.
    #[must_use]
    pub fn lines(&self, strings: &Strings) -> [Said; HOW_MANY_WARNING_LINES] {
        [self.what_it_costs(strings), self.runs_them_anyway(strings)]
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_warning_about, in_english};
    use alo_models::costing::GIGABYTE;

    /// **The whole warning, in the order it is read**: the cost, then the
    /// promise to run.
    #[test]
    fn a_warning_is_two_lines_in_the_order_a_person_reads_them() {
        let strings = in_english();
        let warned = a_warning_about("theirs", 40 * GIGABYTE, 16.0);
        let lines = warned.lines(&strings);

        assert!(
            lines[0].text().contains("larger than the memory"),
            "{}",
            lines[0]
        );
        assert!(
            lines[0].text().contains("will still run them"),
            "{}",
            lines[0]
        );
        assert!(
            lines[1].text().starts_with("That is said once"),
            "{}",
            lines[1]
        );
        assert!(
            lines[1].text().contains("whenever you choose them"),
            "{}",
            lines[1]
        );
        for line in &lines {
            assert!(!line.is_a_bug(), "{line}");
        }
    }

    /// **This crate says one of the two and no more.** With only its own
    /// vocabulary loaded, the cost line is a key nothing declares — which is
    /// the test that catches this crate starting to say something
    /// `alo-models` already says.
    #[test]
    fn only_one_of_the_two_lines_is_this_crates_own() {
        let ours = Strings::of(words::telling_words().unwrap());
        let warned = a_warning_about("theirs", 40 * GIGABYTE, 16.0);
        let said_by_us = warned
            .lines(&ours)
            .into_iter()
            .filter(|line| !line.is_a_bug())
            .count();
        assert_eq!(
            said_by_us, 1,
            "this crate has started saying something alo-models already says"
        );
    }

    /// **Neither line names the licence or the measurement**, which are said
    /// where the weights are added and not argued about afterwards.
    #[test]
    fn a_warning_about_size_says_nothing_about_the_licence_or_the_measurement() {
        let strings = in_english();
        for line in a_warning_about("theirs", 40 * GIGABYTE, 16.0).lines(&strings) {
            let read = line.text().to_ascii_lowercase();
            assert!(!read.contains("licen"), "{line}");
            assert!(!read.contains("measur"), "{line}");
        }
    }

    /// The identity a warning carries is the identity of the cost it is
    /// about, so what is remembered and what was shown cannot drift apart —
    /// and both numbers are beside the sentence for whoever writes them.
    #[test]
    fn a_warning_knows_which_weights_and_what_they_cost() {
        let warned = a_warning_about("theirs", 40 * GIGABYTE, 16.0);
        let identity = warned.too_large().unwrap();
        assert_eq!(identity.id(), "theirs");
        assert_eq!(warned.weights().id, "theirs");
        assert_eq!(warned.cost().machine_gb(), 16.0);
        assert!((warned.cost().needs_gb() - 40.0).abs() < f32::EPSILON);
    }
}
