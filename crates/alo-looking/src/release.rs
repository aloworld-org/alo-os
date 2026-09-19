//! One released version of alo OS, as the place updates come from names it.
//!
//! `image/pinned.toml` names a release `MAJOR.MINOR.PATCH` and nothing else,
//! and `alo-image` refuses anything that is not that shape, because *a tag is a
//! name somebody can move*. The place a machine asks holds names of every kind
//! beside those — a signature's name, a candidate somebody pushed, a word that
//! means whichever build came last — and this is the door that lets exactly the
//! releases through.
//!
//! **Refused rather than repaired.** Nothing is trimmed, nothing is lowered and
//! no `v` is stripped: a name that is not three numbers is not a release with
//! something wrong with it, it is not a release, and the place offering it is
//! offering something this machine has no opinion about.
//!
//! **A release is never shown to a person.** It is the name of a thing in a
//! place, which `docs/features.md` says a person never learns, so there is no
//! `Display` here and [`NotARelease`] has no sentence: a name that is not a
//! release is simply not one of the releases a place offers, and nobody is told
//! about it.

/// How many numbers a release is written with.
const HOW_MANY_NUMBERS: usize = 3;

/// One released version of alo OS.
///
/// Ordered by its three numbers, which is what *the newest release a place
/// offers* means. The text is carried so that the name asked for is the name
/// the place gave, rather than one written out again from the numbers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Release {
    /// The first number.
    major: u32,
    /// The second.
    minor: u32,
    /// The third.
    patch: u32,
    /// The name exactly as it was read.
    named: String,
}

/// Why some name is not a release.
///
/// Not a sentence anybody reads — see this file's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotARelease {
    /// Not three numbers separated by dots: a word, a version with something
    /// after it, two numbers, or four.
    #[error("a release is three numbers, and that is not")]
    NotThreeNumbers,
    /// Three numbers, one of which is larger than a number this counts with.
    #[error("that release counts higher than a release can")]
    TooLarge,
}

impl Release {
    /// Some name, read as a release.
    ///
    /// # Errors
    /// [`NotARelease`].
    pub fn named(text: &str) -> Result<Self, NotARelease> {
        let parts: Vec<&str> = text.split('.').collect();
        let [major, minor, patch] = parts.as_slice() else {
            return Err(NotARelease::NotThreeNumbers);
        };
        let mut numbers = [0_u32; HOW_MANY_NUMBERS];
        for (number, part) in numbers.iter_mut().zip([major, minor, patch]) {
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(NotARelease::NotThreeNumbers);
            }
            *number = part.parse().map_err(|_| NotARelease::TooLarge)?;
        }
        let [major, minor, patch] = numbers;
        Ok(Self {
            major,
            minor,
            patch,
            named: text.to_owned(),
        })
    }

    /// The name, exactly as the place wrote it.
    #[must_use]
    pub fn named_as(&self) -> &str {
        &self.named
    }

    /// The newest release among these names, ignoring every name that is not
    /// one.
    ///
    /// [`None`] when nothing in the list is a release at all, which is a place
    /// offering no version of this system rather than an answer that failed.
    #[must_use]
    pub fn newest_of<'a>(names: impl IntoIterator<Item = &'a str>) -> Option<Self> {
        names
            .into_iter()
            .filter_map(|name| Self::named(name).ok())
            .max()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Three numbers are a release, and it is written back as it was read.
    #[test]
    fn three_numbers_are_a_release_and_are_written_back_unchanged() {
        let release = Release::named("0.0.2").unwrap();
        assert_eq!(release.named_as(), "0.0.2");
        assert_eq!(Release::named("12.3.45").unwrap().named_as(), "12.3.45");
    }

    /// **A name that is moved on purpose is not a release**, and neither is a
    /// signature's name, half a version, or one with something on either end.
    /// The list is `alo-image`'s own, plus the two names this machine actually
    /// meets in the place it asks.
    #[test]
    fn a_name_that_is_not_three_numbers_is_refused() {
        for name in [
            "latest",
            "dev",
            "main",
            "",
            "0.1",
            "0.0.1-dev",
            "v0.0.1",
            "0..1",
            "0.0.1.2",
            "0.0.x",
            " 0.0.2",
            "0.0.2 ",
            "sha256-8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9",
        ] {
            assert_eq!(
                Release::named(name),
                Err(NotARelease::NotThreeNumbers),
                "`{name}` was read as a release"
            );
        }
    }

    /// **A number larger than this counts with is refused**, rather than
    /// wrapping into a release that would sort below the one running.
    #[test]
    fn a_number_larger_than_a_release_counts_with_is_refused() {
        assert_eq!(Release::named("0.0.4294967296"), Err(NotARelease::TooLarge));
    }

    /// Releases sort by their numbers rather than as text, which is the whole
    /// reason this is a type: `0.0.10` comes after `0.0.9`.
    #[test]
    fn releases_sort_by_their_numbers_and_not_as_text() {
        assert!(Release::named("0.0.10").unwrap() > Release::named("0.0.9").unwrap());
        assert!(Release::named("1.0.0").unwrap() > Release::named("0.99.99").unwrap());
        assert_eq!(
            Release::newest_of(["0.0.1", "0.0.10", "0.0.9"])
                .unwrap()
                .named_as(),
            "0.0.10"
        );
    }

    /// **A list with nothing in it that is a release offers nothing**, and the
    /// names that are not releases do not make it fail.
    #[test]
    fn a_list_holding_no_release_offers_none() {
        assert_eq!(Release::newest_of(["latest", "sha256-aa"]), None);
        assert_eq!(Release::newest_of([]), None);
        assert_eq!(
            Release::newest_of(["latest", "0.0.2", "sha256-aa"])
                .unwrap()
                .named_as(),
            "0.0.2"
        );
    }
}
