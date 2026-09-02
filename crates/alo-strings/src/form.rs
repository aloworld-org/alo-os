//! One of the shapes a sentence takes when it counts something.
//!
//! *1 byte* and *2 bytes* is one sentence in English with two shapes. Polish
//! has three, Irish five, and Latvian has one for nothing at all. The names are
//! CLDR's — `zero`, `one`, `two`, `few`, `many`, `other` — and they are
//! deliberately not translated into anything more descriptive, because a
//! translator's own tools already call them these and a second vocabulary would
//! mean mapping between them somewhere.
//!
//! **A form is not a number and does not mean one.** Latvian's `zero` is used
//! for 0, 10, 11 and 20; Polish's `many` covers 0, 5 to 19, and 100. Which
//! numbers take which form is [`crate::cldr`]'s, and it is the whole reason
//! that file exists rather than a rule anybody writes from memory.
//!
//! A form reaches a key as its last part — `files.too-big.one` — which is how a
//! translator's file holds one line per form, sorted next to each other.

use std::fmt;

/// One shape a counted sentence takes.
///
/// The six CLDR cardinal categories. Every language uses `Other` or a subset
/// including it — except, for whole numbers, Polish, which is why nothing here
/// treats `Other` as a form that must always be present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Form {
    /// Nothing at all, where a language has a word for it. Latvian does, and
    /// uses it for far more numbers than nought.
    Zero,
    /// One of the thing, in the languages that single one out.
    One,
    /// Two of them: Irish, Maltese and Slovene.
    Two,
    /// A small number of them, where "small" is the language's business.
    Few,
    /// A large number of them, which in French and Spanish means whole
    /// millions and in Polish means nine.
    Many,
    /// Everything the language has no other word for. Most languages spend most
    /// of their numbers here.
    Other,
}

/// Every form there is, in the order CLDR writes them, which is the order a
/// translator's file is sorted into.
pub const EVERY_FORM: [Form; 6] = [
    Form::Zero,
    Form::One,
    Form::Two,
    Form::Few,
    Form::Many,
    Form::Other,
];

impl Form {
    /// What this form is called, which is the last part of the key it makes.
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }

    /// The form by that name, if it is one.
    #[must_use]
    pub fn of_tag(tag: &str) -> Option<Self> {
        EVERY_FORM.into_iter().find(|form| form.tag() == tag)
    }
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

/// Forms in the order they are written, for a sentence that lists them.
///
/// Used where a refusal has to tell a translator which forms their language
/// actually uses; being told *one, few, many* in that order is being told
/// something they can act on.
pub(crate) fn listed(forms: &[Form]) -> String {
    let tags: Vec<&str> = forms.iter().map(|form| form.tag()).collect();
    match tags.split_last() {
        None => String::new(),
        Some((last, [])) => (*last).to_owned(),
        Some((last, before)) => format!("{} and {last}", before.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_form_is_named_the_way_a_translators_tools_name_it() {
        assert_eq!(Form::One.tag(), "one");
        assert_eq!(Form::Other.to_string(), "other");
        assert_eq!(Form::of_tag("few"), Some(Form::Few));
        assert_eq!(Form::of_tag("plural"), None);
        assert_eq!(Form::of_tag("One"), None, "a key is lowercase");
    }

    #[test]
    fn every_form_round_trips_through_its_name() {
        for form in EVERY_FORM {
            assert_eq!(Form::of_tag(form.tag()), Some(form), "{form}");
        }
    }

    /// The forms are listed in one order, everywhere, because a translator
    /// reading *one, few and many* in two different orders in two messages has
    /// to work out whether they are the same list.
    #[test]
    fn a_list_of_forms_reads_as_a_sentence() {
        assert_eq!(listed(&[]), "");
        assert_eq!(listed(&[Form::Other]), "other");
        assert_eq!(listed(&[Form::One, Form::Other]), "one and other");
        assert_eq!(
            listed(&[Form::One, Form::Few, Form::Many]),
            "one, few and many"
        );
    }
}
