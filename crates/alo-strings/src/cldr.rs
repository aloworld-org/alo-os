//! Which form a language uses for a number.
//!
//! **This file is a table somebody read, not a rule anybody remembered.** It is
//! the cardinal plural rules from CLDR's `common/supplemental/plurals.xml`,
//! taken from `unicode-org/cldr` on 2026-09-02, and every arm below quotes the
//! condition it came from so that the next person can check it against the
//! source rather than against this file's opinion of itself.
//! `docs/autonomy/QUEUE.md` said outright why: a plural table written from
//! memory and shipped as a tested promise is exactly what the gate exists to
//! stop, and getting Irish wrong is not a bug anybody here would notice.
//!
//! # Whole numbers, and what that removes
//!
//! **alo OS counts things, and a thing is a whole number.** Bytes, files,
//! models, shortcuts, minutes: there is no half a file. So [`Counting`] holds a
//! `u64` and there is no shape for a fraction to arrive in — the same move as
//! `alo-capability`'s closed `Takes`, where what cannot be expressed cannot be
//! got wrong.
//!
//! That decision does most of the work here. CLDR's rules are written over six
//! operands — `n`, `i`, `v`, `w`, `f`, `t` and the compact exponent `e` — and a
//! whole number written plainly fixes all but two of them at zero. So `i = 1
//! and v = 0` is `n == 1`, Czech's `many` (`v != 0`) cannot happen at all, and
//! French's `many` keeps only the half about whole millions. The conditions are
//! quoted here in full anyway, so that what was dropped is visible.
//!
//! **What this costs, stated rather than discovered:** a sentence that one day
//! wants to count *1.5 hours* is not covered, and would be a decision to
//! reopen — a new operand set and a wider [`Counting`] — rather than a form
//! quietly picked as though the number had been whole.
//!
//! # A language whose rules are not here
//!
//! Answers `None`, and everything downstream treats that as *we do not know*
//! rather than guessing English's two forms. [`crate::Vocabulary::check`]
//! refuses a countable string translated into such a language, in words
//! addressed to whoever is contributing it, because a translation shown through
//! rules nobody has is a sentence that is wrong for most numbers in a language
//! nobody here reads.

use crate::form::Form;
use crate::language::Language;

/// The number a sentence counts, and how it is written where the sentence says
/// it goes.
///
/// The two travel together because they must not disagree: the number that
/// chose *how many files* over *one file* is the number the person then reads.
/// Splitting them would make that a convention at every call site.
///
/// **How a number is written is still the region's business.** [`Counting::of`]
/// writes plain digits, which is right for most of Europe and wrong for a
/// person who writes `4 000 000`; [`Counting::written_as`] takes the text
/// whoever knows the region has already made, exactly as [`crate::Filling`]
/// does and for the same reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counting {
    /// How many, which is what picks the form.
    how_many: u64,
    /// How it is written where the sentence shows it.
    written: String,
}

impl Counting {
    /// This many, written in plain digits.
    #[must_use]
    pub fn of(how_many: u64) -> Self {
        Self {
            how_many,
            written: how_many.to_string(),
        }
    }

    /// This many, written the way the region writes it.
    ///
    /// The text is the caller's, unchecked, as a [`crate::Filling`]'s values
    /// are: this crate does not format numbers and does not second-guess
    /// somebody who does.
    #[must_use]
    pub fn written_as(how_many: u64, written: impl Into<String>) -> Self {
        Self {
            how_many,
            written: written.into(),
        }
    }

    /// How many, which is what picks the form.
    #[must_use]
    pub fn how_many(&self) -> u64 {
        self.how_many
    }

    /// How it is written where the sentence shows it.
    #[must_use]
    pub fn written(&self) -> &str {
        &self.written
    }
}

/// Which form this language uses for this many, or `None` when alo OS does not
/// have the language's rules.
#[must_use]
pub fn form_for(language: &Language, how_many: u64) -> Option<Form> {
    counts(language.primary()).map(|counts| counts.form_for(how_many))
}

/// Every form this language uses for whole numbers, in [`crate::form`]'s order,
/// or `None` when alo OS does not have the language's rules.
///
/// This is what a translator's file needs a line for, and nothing more: a form
/// no whole number reaches is a form nothing would ever show.
#[must_use]
pub fn forms(language: &Language) -> Option<&'static [Form]> {
    counts(language.primary()).map(Counts::forms)
}

/// Whether alo OS knows how this language counts.
#[must_use]
pub fn knows(language: &Language) -> bool {
    counts(language.primary()).is_some()
}

/// Whether exactly one whole number takes this form in this language.
///
/// **This is the only case where a sentence may spell the number out** — *one
/// file*, *ein Ordner* — instead of showing it, and it is far rarer than it
/// looks. English's *one* is 1 and nothing else, but Croatian's is 1, 21, 31
/// and 101, French's covers none as well as one, and Latvian's *zero* covers 0,
/// 10, 11 and 20 alike. A form that covers more than one number and does not
/// say which is a sentence that counts at somebody without telling them the
/// count, so [`crate::Vocabulary::check`] asks this before letting a
/// translation leave the number out.
#[must_use]
pub fn names_one_number(language: &Language, form: Form) -> bool {
    counts(language.primary()).is_some_and(|counts| counts.names_one_number(form))
}

/// One CLDR rule set, as it applies to whole numbers.
///
/// Rule sets that answer identically for every whole number are one variant,
/// because two arms that cannot differ are two places to fix one mistake. The
/// documentation on each says which CLDR blocks were folded together and quotes
/// their conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Counts {
    /// One when there is exactly one.
    ///
    /// Three CLDR blocks, identical over whole numbers: `i = 1 and v = 0` (de,
    /// en, et, fi, nl, sv), `n = 1` (bg, el, hu), and Danish's `n = 1 or t != 0
    /// and i = 0,1`, whose second half needs a fraction.
    Exactly,

    /// None or one is one; a whole million is many. French and Portuguese.
    ///
    /// `one`: `i = 0..1`. `many`: `e = 0 and i != 0 and i % 1000000 = 0 and
    /// v = 0 or e != 0..5` — the second half is compact notation, which is not
    /// how a count arrives here.
    NoneOrOneThenMillions,

    /// One is one; a whole million is many. Italian and Spanish.
    ///
    /// Italian's `one` is `i = 1 and v = 0` and Spanish's is `n = 1`, which are
    /// the same over whole numbers; both carry the `many` above.
    OneThenMillions,

    /// One, then two to four, then everything else. Czech and Slovak.
    ///
    /// `one`: `i = 1 and v = 0`. `few`: `i = 2..4 and v = 0`. `many` is
    /// `v != 0`, so no whole number reaches it.
    OneThenUpToFour,

    /// Polish, which for whole numbers never says `other` at all.
    ///
    /// `one`: `i = 1 and v = 0`. `few`: `v = 0 and i % 10 = 2..4 and
    /// i % 100 != 12..14`. `many`: `v = 0 and i != 1 and i % 10 = 0..1 or
    /// v = 0 and i % 10 = 5..9 or v = 0 and i % 100 = 12..14` — which is every
    /// remaining whole number, so `other` has decimal samples only.
    PolishTens,

    /// Croatian: the last digit decides, except in the teens.
    ///
    /// `one`: `v = 0 and i % 10 = 1 and i % 100 != 11 or f % 10 = 1 and
    /// f % 100 != 11`. `few`: the same shape over `2..4`. The `f` halves need a
    /// fraction.
    CroatianTens,

    /// Lithuanian: the last digit decides, except through the whole teens.
    ///
    /// `one`: `n % 10 = 1 and n % 100 != 11..19`. `few`: `n % 10 = 2..9 and
    /// n % 100 != 11..19`. `many` is `f != 0`.
    LithuanianTens,

    /// Latvian, which has a form for nothing — and spends it on 0, 10 to 20,
    /// 30 and 100 alike.
    ///
    /// `zero`: `n % 10 = 0 or n % 100 = 11..19 or v = 2 and f % 100 = 11..19`.
    /// `one`: `n % 10 = 1 and n % 100 != 11 or v = 2 and f % 10 = 1 and
    /// f % 100 != 11 or v != 2 and f % 10 = 1`.
    LatvianTens,

    /// Romanian: one, then nothing and the first hundred's teens, then the
    /// rest.
    ///
    /// `one`: `i = 1 and v = 0`. `few`: `v != 0 or n = 0 or n != 1 and
    /// n % 100 = 1..19`.
    RomanianTeens,

    /// Slovene, decided by the last two digits.
    ///
    /// `one`: `v = 0 and i % 100 = 1`. `two`: `i % 100 = 2`. `few`:
    /// `v = 0 and i % 100 = 3..4 or v != 0`.
    SloveneHundreds,

    /// Maltese: one, two, then nothing-and-the-small-ones, then the teens.
    ///
    /// `one`: `n = 1`. `two`: `n = 2`. `few`: `n = 0 or n % 100 = 3..10`.
    /// `many`: `n % 100 = 11..19`.
    MalteseTeens,

    /// Irish, which counts in runs: one, two, three to six, seven to ten.
    ///
    /// `one`: `n = 1`. `two`: `n = 2`. `few`: `n = 3..6`. `many`: `n = 7..10`.
    IrishRuns,
}

impl Counts {
    /// Which form this many takes, in CLDR's order of asking: the first
    /// condition that holds wins, and `other` is what is left.
    fn form_for(self, n: u64) -> Form {
        match self {
            Self::Exactly => {
                if n == 1 {
                    Form::One
                } else {
                    Form::Other
                }
            }
            Self::NoneOrOneThenMillions => {
                if n <= 1 {
                    Form::One
                } else if n.is_multiple_of(1_000_000) {
                    Form::Many
                } else {
                    Form::Other
                }
            }
            Self::OneThenMillions => {
                if n == 1 {
                    Form::One
                } else if n != 0 && n.is_multiple_of(1_000_000) {
                    Form::Many
                } else {
                    Form::Other
                }
            }
            Self::OneThenUpToFour => match n {
                1 => Form::One,
                2..=4 => Form::Few,
                _ => Form::Other,
            },
            Self::PolishTens => {
                if n == 1 {
                    Form::One
                } else if matches!(n % 10, 2..=4) && !matches!(n % 100, 12..=14) {
                    Form::Few
                } else {
                    Form::Many
                }
            }
            Self::CroatianTens => {
                if n % 10 == 1 && n % 100 != 11 {
                    Form::One
                } else if matches!(n % 10, 2..=4) && !matches!(n % 100, 12..=14) {
                    Form::Few
                } else {
                    Form::Other
                }
            }
            Self::LithuanianTens => {
                if n % 10 == 1 && !matches!(n % 100, 11..=19) {
                    Form::One
                } else if matches!(n % 10, 2..=9) && !matches!(n % 100, 11..=19) {
                    Form::Few
                } else {
                    Form::Other
                }
            }
            Self::LatvianTens => {
                if n.is_multiple_of(10) || matches!(n % 100, 11..=19) {
                    Form::Zero
                } else if n % 10 == 1 && n % 100 != 11 {
                    Form::One
                } else {
                    Form::Other
                }
            }
            Self::RomanianTeens => {
                if n == 1 {
                    Form::One
                } else if n == 0 || matches!(n % 100, 1..=19) {
                    Form::Few
                } else {
                    Form::Other
                }
            }
            Self::SloveneHundreds => match n % 100 {
                1 => Form::One,
                2 => Form::Two,
                3 | 4 => Form::Few,
                _ => Form::Other,
            },
            Self::MalteseTeens => {
                if n == 1 {
                    Form::One
                } else if n == 2 {
                    Form::Two
                } else if n == 0 || matches!(n % 100, 3..=10) {
                    Form::Few
                } else if matches!(n % 100, 11..=19) {
                    Form::Many
                } else {
                    Form::Other
                }
            }
            Self::IrishRuns => match n {
                1 => Form::One,
                2 => Form::Two,
                3..=6 => Form::Few,
                7..=10 => Form::Many,
                _ => Form::Other,
            },
        }
    }

    /// Whether exactly one whole number takes this form.
    ///
    /// Read off the conditions on each variant above, and asserted against
    /// [`Counts::form_for`] by `a_form_that_names_one_number_names_only_it`:
    /// `one` is a single number only where the rule says `n = 1` outright, and
    /// not where it says `i = 0..1` (French counts none as one) or
    /// `n % 10 = 1` (Croatian, Lithuanian and Latvian count 21 as one).
    fn names_one_number(self, form: Form) -> bool {
        matches!(
            (self, form),
            (
                Self::Exactly
                    | Self::OneThenMillions
                    | Self::OneThenUpToFour
                    | Self::PolishTens
                    | Self::RomanianTeens,
                Form::One
            ) | (Self::MalteseTeens | Self::IrishRuns, Form::One | Form::Two)
        )
    }

    /// Every form some whole number takes under these rules.
    ///
    /// Asserted against [`Counts::form_for`] by
    /// `every_form_listed_is_a_form_some_number_actually_takes`, so the two
    /// cannot drift: a form listed here that no number reaches would be a line
    /// a translator writes for nothing, and one reached but not listed would be
    /// a sentence with nothing to show.
    fn forms(self) -> &'static [Form] {
        match self {
            Self::Exactly => &[Form::One, Form::Other],
            Self::NoneOrOneThenMillions | Self::OneThenMillions => {
                &[Form::One, Form::Many, Form::Other]
            }
            Self::OneThenUpToFour
            | Self::CroatianTens
            | Self::LithuanianTens
            | Self::RomanianTeens => &[Form::One, Form::Few, Form::Other],
            Self::PolishTens => &[Form::One, Form::Few, Form::Many],
            Self::LatvianTens => &[Form::Zero, Form::One, Form::Other],
            Self::SloveneHundreds => &[Form::One, Form::Two, Form::Few, Form::Other],
            Self::MalteseTeens | Self::IrishRuns => {
                &[Form::One, Form::Two, Form::Few, Form::Many, Form::Other]
            }
        }
    }
}

/// The rules for a language, by its primary subtag.
///
/// Only the language itself decides: `pt-BR` counts as `pt` does, because a
/// region changes how a number is *written* and not how many words a language
/// has for it.
///
/// The 24 official languages are all here, which
/// `every_official_language_counts` asserts. A language somebody contributes is
/// added by reading CLDR for it — there is no fallback, and
/// [`crate::Vocabulary::check`] says so to whoever is contributing.
fn counts(primary: &str) -> Option<Counts> {
    let counts = match primary {
        "bg" | "da" | "de" | "el" | "en" | "et" | "fi" | "hu" | "nl" | "sv" => Counts::Exactly,
        "fr" | "pt" => Counts::NoneOrOneThenMillions,
        "es" | "it" => Counts::OneThenMillions,
        "cs" | "sk" => Counts::OneThenUpToFour,
        "pl" => Counts::PolishTens,
        "hr" => Counts::CroatianTens,
        "lt" => Counts::LithuanianTens,
        "lv" => Counts::LatvianTens,
        "ro" => Counts::RomanianTeens,
        "sl" => Counts::SloveneHundreds,
        "mt" => Counts::MalteseTeens,
        "ga" => Counts::IrishRuns,
        _ => return None,
    };
    Some(counts)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::union;

    fn language(tag: &str) -> Language {
        Language::written(tag).unwrap()
    }

    /// Numbers wide enough to reach every form every rule set has, including
    /// the whole millions French and Spanish single out.
    fn a_spread_of_numbers() -> Vec<u64> {
        let mut numbers: Vec<u64> = (0..=1_000).collect();
        numbers.extend([
            1_002, 1_011, 1_021, 10_000, 100_000, 123_456, 1_000_000, 1_000_001, 2_000_000,
        ]);
        numbers
    }

    /// Every sample CLDR publishes beside the rule, checked against what this
    /// file answers. These are the numbers the source itself names, so a
    /// mistake in an arm shows up here as the wrong form for a number somebody
    /// at Unicode chose to write down.
    #[test]
    fn the_samples_cldr_publishes_come_out_the_way_cldr_says() {
        let samples: [(&str, Form, &[u64]); 40] = [
            // i = 1 and v = 0 / n = 1 / Danish.
            ("de", Form::One, &[1]),
            ("de", Form::Other, &[0, 2, 16, 100, 1_000]),
            ("bg", Form::One, &[1]),
            ("bg", Form::Other, &[0, 2, 16, 100]),
            ("da", Form::One, &[1]),
            ("da", Form::Other, &[0, 2, 16, 100]),
            // fr, pt.
            ("fr", Form::One, &[0, 1]),
            ("fr", Form::Many, &[1_000_000, 2_000_000]),
            ("fr", Form::Other, &[2, 17, 100, 1_000, 10_000, 100_000]),
            ("pt", Form::One, &[0, 1]),
            ("pt", Form::Many, &[1_000_000]),
            ("pt", Form::Other, &[2, 17, 100_000]),
            // it, es.
            ("it", Form::One, &[1]),
            ("it", Form::Many, &[1_000_000]),
            ("it", Form::Other, &[0, 2, 16, 100, 1_000, 100_000]),
            ("es", Form::One, &[1]),
            ("es", Form::Many, &[1_000_000]),
            ("es", Form::Other, &[0, 2, 16, 100_000]),
            // cs, sk.
            ("cs", Form::One, &[1]),
            ("cs", Form::Few, &[2, 3, 4]),
            ("cs", Form::Other, &[0, 5, 19, 100, 1_000]),
            // pl.
            ("pl", Form::One, &[1]),
            (
                "pl",
                Form::Few,
                &[2, 4, 22, 24, 32, 34, 52, 54, 62, 102, 1_002],
            ),
            ("pl", Form::Many, &[0, 5, 19, 100, 1_000, 10_000]),
            // hr.
            ("hr", Form::One, &[1, 21, 31, 81, 101, 1_001]),
            ("hr", Form::Few, &[2, 4, 22, 24, 52, 62, 102, 1_002]),
            ("hr", Form::Other, &[0, 5, 19, 100, 1_000]),
            // lt.
            ("lt", Form::One, &[1, 21, 81, 101, 1_001]),
            ("lt", Form::Few, &[2, 9, 22, 29, 102, 1_002]),
            ("lt", Form::Other, &[0, 10, 20, 30, 100, 1_000]),
            // lv.
            ("lv", Form::Zero, &[0, 10, 20, 30, 60, 100, 1_000]),
            ("lv", Form::One, &[1, 21, 31, 81, 101, 1_001]),
            ("lv", Form::Other, &[2, 9, 22, 29, 102, 1_002]),
            // ro.
            ("ro", Form::One, &[1]),
            ("ro", Form::Few, &[0, 2, 16, 101, 1_001]),
            ("ro", Form::Other, &[20, 35, 100, 1_000]),
            // sl.
            ("sl", Form::One, &[1, 101, 501, 1_001]),
            ("sl", Form::Two, &[2, 102, 702, 1_002]),
            ("sl", Form::Few, &[3, 4, 103, 104, 703, 1_003]),
            ("sl", Form::Other, &[0, 5, 19, 100, 1_000]),
        ];
        for (tag, form, numbers) in samples {
            for how_many in numbers {
                assert_eq!(
                    form_for(&language(tag), *how_many),
                    Some(form),
                    "{tag} {how_many}"
                );
            }
        }
    }

    /// The two five-form languages, which are the ones a table written from
    /// memory gets wrong.
    #[test]
    fn maltese_and_irish_come_out_the_way_cldr_says() {
        let maltese = language("mt");
        for (form, numbers) in [
            (Form::One, vec![1]),
            (Form::Two, vec![2]),
            (Form::Few, vec![0, 3, 10, 103, 109, 1_003]),
            (Form::Many, vec![11, 19, 111, 117, 1_011]),
            (Form::Other, vec![20, 35, 100, 1_000, 10_000]),
        ] {
            for how_many in numbers {
                assert_eq!(form_for(&maltese, how_many), Some(form), "mt {how_many}");
            }
        }

        let irish = language("ga");
        for (form, numbers) in [
            (Form::One, vec![1]),
            (Form::Two, vec![2]),
            (Form::Few, vec![3, 4, 5, 6]),
            (Form::Many, vec![7, 8, 9, 10]),
            (Form::Other, vec![0, 11, 25, 100, 1_000]),
        ] {
            for how_many in numbers {
                assert_eq!(form_for(&irish, how_many), Some(form), "ga {how_many}");
            }
        }
    }

    /// **The list of forms and the rule cannot drift apart.** A form listed
    /// that no number reaches is a line a translator writes for nothing; one
    /// reached but not listed is a sentence with nothing to show. Both are
    /// checked here rather than trusted.
    #[test]
    fn every_form_listed_is_a_form_some_number_actually_takes() {
        for official in union::OFFICIAL {
            let language = language(official.tag);
            let listed = forms(&language).unwrap();
            let mut reached: Vec<Form> = Vec::new();
            for how_many in a_spread_of_numbers() {
                let form = form_for(&language, how_many).unwrap();
                assert!(
                    listed.contains(&form),
                    "{} answers {form} for {how_many} and does not list it",
                    official.in_english
                );
                if !reached.contains(&form) {
                    reached.push(form);
                }
            }
            reached.sort_unstable();
            let mut listed = listed.to_vec();
            listed.sort_unstable();
            assert_eq!(reached, listed, "{}", official.in_english);
        }
    }

    /// Every language `docs/features.md` promises has to be countable, or a
    /// sentence with a number in it is a sentence that language cannot have.
    #[test]
    fn every_official_language_counts() {
        for official in union::OFFICIAL {
            assert!(
                knows(&language(official.tag)),
                "{} has no plural rules",
                official.in_english
            );
        }
    }

    /// **Polish never says `other` about a whole number**, which is the finding
    /// that most argues for reading the rules rather than assuming them: a
    /// table built on *every language has `other`* would give Polish a form
    /// nothing shows and, worse, would let a Polish file look complete while
    /// `many` — 0, 5 to 19, 100 — was missing.
    #[test]
    fn polish_has_no_other_form_for_a_whole_number() {
        assert_eq!(
            forms(&language("pl")).unwrap(),
            [Form::One, Form::Few, Form::Many]
        );
        for how_many in a_spread_of_numbers() {
            assert_ne!(
                form_for(&language("pl"), how_many),
                Some(Form::Other),
                "pl {how_many}"
            );
        }
    }

    /// **A form that says it names one number names only it**, checked against
    /// the rules rather than trusted — this is what decides whether a
    /// translator may write *ein Ordner* instead of showing the number, and
    /// getting it wrong would let Croatian write *jedna datoteka* for 21 files.
    #[test]
    fn a_form_that_names_one_number_names_only_it() {
        for official in union::OFFICIAL {
            let language = language(official.tag);
            for form in crate::form::EVERY_FORM {
                let how_many = a_spread_of_numbers()
                    .into_iter()
                    .filter(|how_many| form_for(&language, *how_many) == Some(form))
                    .count();
                assert_eq!(
                    names_one_number(&language, form),
                    how_many == 1,
                    "{} {form} covers {how_many} of the numbers checked",
                    official.in_english
                );
            }
        }
    }

    /// The three ways *one* is not one number, which are the reason the
    /// question is asked of the rules rather than of the form's name.
    #[test]
    fn one_is_not_always_one_number() {
        assert!(names_one_number(&language("en"), Form::One));
        assert!(
            !names_one_number(&language("fr"), Form::One),
            "French counts none as one"
        );
        assert!(
            !names_one_number(&language("hr"), Form::One),
            "Croatian counts 21 as one"
        );
        assert!(
            !names_one_number(&language("lv"), Form::Zero),
            "Latvian's zero covers 0, 10, 11 and 20"
        );
        assert!(
            !names_one_number(&language("is"), Form::One),
            "a language nobody has read the rules for names nothing"
        );
    }

    /// A region changes how a number is written, not how many words a language
    /// has for it.
    #[test]
    fn a_region_does_not_change_how_a_language_counts() {
        assert_eq!(forms(&language("pt-BR")), forms(&language("pt")));
        assert_eq!(
            form_for(&language("de-AT"), 2),
            form_for(&language("de"), 2)
        );
    }

    /// **A language nobody has read the rules for is not guessed at.** English's
    /// two forms are English's, and lending them to Icelandic would be a
    /// sentence wrong for most numbers in a language nobody here reads.
    #[test]
    fn a_language_we_do_not_have_the_rules_for_says_so() {
        let icelandic = language("is");
        assert!(!knows(&icelandic));
        assert_eq!(forms(&icelandic), None);
        assert_eq!(form_for(&icelandic, 1), None);
    }

    /// The number that picks the form is the number the sentence shows, which
    /// is why they arrive together — and how it is written is still whoever
    /// knows the region's business.
    #[test]
    fn a_counting_carries_the_number_and_how_it_is_written() {
        let plain = Counting::of(4_000_000);
        assert_eq!(plain.how_many(), 4_000_000);
        assert_eq!(plain.written(), "4000000");

        let french = Counting::written_as(4_000_000, "4 000 000");
        assert_eq!(french.how_many(), 4_000_000);
        assert_eq!(french.written(), "4 000 000");
        assert_eq!(
            form_for(&language("fr"), french.how_many()),
            Some(Form::Many)
        );
    }
}
