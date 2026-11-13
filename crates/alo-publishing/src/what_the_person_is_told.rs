//! **The line a person reads once a folder is published**, wherever they read
//! it: in the Bar at full access, on the desktop's card after they said yes,
//! and in their terminal.
//!
//! The owner's decision of 2026-11-13: *the line afterwards says plainly it
//! went to the internet and Put back cannot recall it.* So it is never the
//! *Done* every other change gets — that word, beside a change, reads as one
//! `alo-back` can take back (`alo-shell`'s `what_an_answer_did`), and a
//! publication is the one change it cannot. Each surface asks this file rather
//! than choosing its own words, so the three cannot drift into disagreeing.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// What a publication was, as a surface holds it: the folder, the address,
/// and whether the site already held exactly this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatWasPublished<'a> {
    /// The folder, as a person reads its path.
    pub folder: &'a str,
    /// Where anybody reads it.
    pub address: &'a str,
    /// Whether nothing new was sent.
    pub nothing_new: bool,
}

/// After alo published without asking, because the person chose full access.
#[must_use]
pub fn published_without_asking(what: WhatWasPublished<'_>, strings: &Strings) -> Said {
    if what.nothing_new {
        return nothing_new(what, strings);
    }
    strings.say(
        &words::PUBLISHED_WITHOUT_ASKING.key(),
        &Filling::of(words::FOLDER, what.folder).and("address", what.address),
    )
}

/// After the person said yes and it was published.
#[must_use]
pub fn published_after_your_ok(what: WhatWasPublished<'_>, strings: &Strings) -> Said {
    if what.nothing_new {
        return nothing_new(what, strings);
    }
    strings.say(
        &words::PUBLISHED_AFTER_YOUR_OK.key(),
        &Filling::of("address", what.address),
    )
}

/// The site already held exactly this.
fn nothing_new(what: WhatWasPublished<'_>, strings: &Strings) -> Said {
    strings.say(
        &words::NOTHING_NEW.key(),
        &Filling::of(words::FOLDER, what.folder).and("address", what.address),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn in_english() -> Strings {
        Strings::of(crate::words::publishing_words().unwrap())
    }

    const PUBLISHED: WhatWasPublished<'static> = WhatWasPublished {
        folder: "/home/anna/site",
        address: "https://anna.github.io/",
        nothing_new: false,
    };

    /// **Both lines say it is on the internet and Put back can't recall it**,
    /// and neither says *Done*.
    #[test]
    fn every_line_after_a_publication_says_it_is_on_the_internet_and_cannot_be_recalled() {
        let strings = in_english();
        for said in [
            published_without_asking(PUBLISHED, &strings),
            published_after_your_ok(PUBLISHED, &strings),
        ] {
            let text = said.text();
            assert!(text.contains("on the internet now"), "{text}");
            assert!(text.contains("https://anna.github.io/"), "{text}");
            assert!(text.contains("Put back can't recall it."), "{text}");
            assert!(!text.starts_with("Done"), "{text}");
        }
        assert!(
            published_without_asking(PUBLISHED, &strings)
                .text()
                .contains("because you chose full access")
        );
    }

    #[test]
    fn a_site_that_already_held_the_folder_is_told_nothing_new_was_published() {
        let strings = in_english();
        let same = WhatWasPublished {
            nothing_new: true,
            ..PUBLISHED
        };
        for said in [
            published_without_asking(same, &strings),
            published_after_your_ok(same, &strings),
        ] {
            assert!(
                said.text().starts_with("Nothing new was published"),
                "{said:?}"
            );
        }
    }
}
