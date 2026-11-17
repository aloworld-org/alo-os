//! What a person would do with an application they install.
//!
//! # Every kind of application, since 2026-11-17
//!
//! The first eight were all kinds of **making** — the gap they closed was that a
//! fresh machine could look at a picture and could not make one. The owner
//! widened the market on 2026-11-17: *"Every kind of app"* — chat, calls, music,
//! notes, email, code, maps, books, games, passwords, remote desktop, 3D, music
//! making and screen recording — and diagrams, which this lane judged a kind of
//! its own because drawing a flow chart is neither illustration nor a document.
//! So a category is now what a person **does**, of which making is one kind, and
//! the naming rule below holds for all of them.
//!
//! [`crate::Role`] is what a fresh machine's eight applications are *there to
//! do*, so a caller can ask for *the terminal* without knowing today's
//! identifier. A category is the other question: a person who has not installed
//! anything is looking for the thing that does what they came to do, and they
//! are looking under what they want to **make**.
//!
//! # Named for the work, never for the product it replaces
//!
//! Every name here is a kind of thing a person does. None of them is the name of
//! somebody else's program, and that is a decision rather than an omission: a
//! category called `photoshop-alternative` would describe our list by another
//! company's catalogue, would age into a lie the day either program changed,
//! and would tell a person who had never used that program nothing at all.
//! *Painting* and *illustration* say what is behind the shelf; *the Illustrator
//! one* says only who got there first.
//!
//! [`Category::Documents`] is the one that was hard to name, and the first
//! spelling of it was `office` — which is a kind of work and also a product,
//! and a test that had to carve out its own category's name was a test being
//! argued with rather than passed. *Documents* is what a person makes, and it
//! says exactly where the catalogue picks up from the machine: a fresh machine
//! **reads** documents ([`crate::Role::DocumentViewer`]) and cannot write one.
//!
//! # These are not words a person reads
//!
//! A category's name here is an identifier in a data file — the spelling
//! [`crate::Catalogue`] reads out of `catalogue.toml` and refuses an unknown
//! one by. **A market screen may not draw it.** What a person reads is their
//! own language, through `crate::words` like every other string this crate
//! shows, and a screen that drew `page-layout` at somebody would be the
//! hardcoded-English bug wearing a data file.
//!
//! A category is a closed list rather than a string for [`crate::Role`]'s
//! reason: so the catalogue can refuse a shelf with nothing on it, and so a
//! caller can ask for *the video one* without knowing today's identifier.

use serde::Deserialize;

/// What a person would do with an application they install.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// A picture that exists already: cropped, corrected, cut out, combined.
    Pictures,
    /// A picture that does not exist yet, made with a brush.
    Painting,
    /// Lines that stay sharp at any size — a logo, a sign, a cut file.
    Illustration,
    /// What a camera recorded, developed and graded without altering it.
    Photographs,
    /// A page arranged to be printed, with the colour a printer asks for.
    PageLayout,
    /// Clips on a timeline, with sound, exported to play somewhere else.
    Video,
    /// Sound recorded and edited: an interview, a voice-over, a podcast.
    Audio,
    /// Something somebody else can open: written, calculated or presented.
    Documents,
    /// Boxes and arrows: a flow, a plan, an organisation, a network.
    Diagrams,
    /// A thing in three dimensions — an object to render, animate or make.
    Modelling,
    /// Music written, played in and recorded: a score, a track, a mix.
    MusicMaking,
    /// What is on the screen, recorded or streamed as it happens.
    ScreenRecording,
    /// Programs written, run and corrected.
    Code,
    /// Messages to other people, as they are written.
    Chat,
    /// Talking to other people, seeing them or not.
    Calls,
    /// Letters sent and received by address.
    Email,
    /// Notes kept for oneself: typed, linked or written by hand.
    Notes,
    /// Music and radio, listened to.
    Music,
    /// Books read on the screen, and a shelf of them kept.
    Books,
    /// Where a place is, and the way there.
    Maps,
    /// Games played, and the places they are bought or kept.
    Games,
    /// Passwords and other secrets, kept in one locked place.
    Passwords,
    /// Another computer, seen and used from this one.
    RemoteDesktop,
}

impl Category {
    /// Every category, in the order the catalogue is kept: the eight kinds of
    /// making first, as they were decided on 2026-10-16, and then the kinds the
    /// owner added on 2026-11-17.
    pub const EVERY: [Self; 23] = [
        Self::Pictures,
        Self::Painting,
        Self::Illustration,
        Self::Photographs,
        Self::PageLayout,
        Self::Video,
        Self::Audio,
        Self::Documents,
        Self::Diagrams,
        Self::Modelling,
        Self::MusicMaking,
        Self::ScreenRecording,
        Self::Code,
        Self::Chat,
        Self::Calls,
        Self::Email,
        Self::Notes,
        Self::Music,
        Self::Books,
        Self::Maps,
        Self::Games,
        Self::Passwords,
        Self::RemoteDesktop,
    ];

    /// The category as the catalogue names it.
    ///
    /// The spelling in `catalogue.toml`, for a refusal a person editing that
    /// file can act on. **Not a string to draw** — see this module's note.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Pictures => "pictures",
            Self::Painting => "painting",
            Self::Illustration => "illustration",
            Self::Photographs => "photographs",
            Self::PageLayout => "page-layout",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Documents => "documents",
            Self::Diagrams => "diagrams",
            Self::Modelling => "modelling",
            Self::MusicMaking => "music-making",
            Self::ScreenRecording => "screen-recording",
            Self::Code => "code",
            Self::Chat => "chat",
            Self::Calls => "calls",
            Self::Email => "email",
            Self::Notes => "notes",
            Self::Music => "music",
            Self::Books => "books",
            Self::Maps => "maps",
            Self::Games => "games",
            Self::Passwords => "passwords",
            Self::RemoteDesktop => "remote-desktop",
        }
    }

    /// The word this category is drawn through.
    ///
    /// **Never [`Category::named`]** — that is the spelling `catalogue.toml`
    /// uses and the one a refusal quotes, and it is English that nothing
    /// translates. This is the sentence a person reads, which the vocabulary
    /// owns like every other.
    #[must_use]
    pub const fn word(self) -> crate::words::Word {
        match self {
            Self::Pictures => crate::words::PICTURES,
            Self::Painting => crate::words::PAINTING,
            Self::Illustration => crate::words::ILLUSTRATION,
            Self::Photographs => crate::words::PHOTOGRAPHS,
            Self::PageLayout => crate::words::PAGE_LAYOUT,
            Self::Video => crate::words::VIDEO,
            Self::Audio => crate::words::AUDIO,
            Self::Documents => crate::words::DOCUMENTS,
            Self::Diagrams => crate::words::DIAGRAMS,
            Self::Modelling => crate::words::MODELLING,
            Self::MusicMaking => crate::words::MUSIC_MAKING,
            Self::ScreenRecording => crate::words::SCREEN_RECORDING,
            Self::Code => crate::words::CODE,
            Self::Chat => crate::words::CHAT,
            Self::Calls => crate::words::CALLS,
            Self::Email => crate::words::EMAIL,
            Self::Notes => crate::words::NOTES,
            Self::Music => crate::words::MUSIC,
            Self::Books => crate::words::BOOKS,
            Self::Maps => crate::words::MAPS,
            Self::Games => crate::words::GAMES,
            Self::Passwords => crate::words::PASSWORDS,
            Self::RemoteDesktop => crate::words::REMOTE_DESKTOP,
        }
    }
    /// The category as a verb's option names it.
    ///
    /// **Underscores, where [`Category::named`] has a hyphen**, and the two
    /// spellings are each right for their own surface rather than a drift:
    ///
    /// - `catalogue.toml` is a file a person edits, and `page-layout` is how
    ///   that file is written throughout — `a_category_is_named_in_the_spelling_the_file_uses`
    ///   holds it to that.
    /// - A verb's option is **what a model sends and what the record keeps**,
    ///   and `alo_capability` refuses anything else in as many words: *name the
    ///   option in lower-case words joined by underscores, like `left_half`.*
    ///   That rule exists because a model has to be able to send the name back
    ///   exactly.
    ///
    /// So the mapping is one function with a test either side of it, rather
    /// than a second list somebody has to remember to extend. Derived from
    /// [`Category::named`] and never typed again, so a category renamed in one
    /// place is renamed in both.
    #[must_use]
    pub fn as_an_option(self) -> String {
        self.named().replace('-', "_")
    }

    /// Where this category is in [`Category::EVERY`].
    pub(crate) const fn place(self) -> usize {
        match self {
            Self::Pictures => 0,
            Self::Painting => 1,
            Self::Illustration => 2,
            Self::Photographs => 3,
            Self::PageLayout => 4,
            Self::Video => 5,
            Self::Audio => 6,
            Self::Documents => 7,
            Self::Diagrams => 8,
            Self::Modelling => 9,
            Self::MusicMaking => 10,
            Self::ScreenRecording => 11,
            Self::Code => 12,
            Self::Chat => 13,
            Self::Calls => 14,
            Self::Email => 15,
            Self::Notes => 16,
            Self::Music => 17,
            Self::Books => 18,
            Self::Maps => 19,
            Self::Games => 20,
            Self::Passwords => 21,
            Self::RemoteDesktop => 22,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_category_is_in_its_own_place() {
        for (at, category) in Category::EVERY.into_iter().enumerate() {
            assert_eq!(category.place(), at, "{}", category.named());
        }
    }

    /// **Every category has its own word, and no two share one.** A category
    /// added without a word would be drawn as nothing at all, which looks like
    /// an empty shelf rather than a missing translation.
    #[test]
    fn every_category_is_drawn_through_a_word_of_its_own() {
        let mut keys = Vec::new();
        for category in Category::EVERY {
            let key = category.word().named().to_owned();
            assert!(
                !keys.contains(&key),
                "{} shares a word with another category",
                category.named()
            );
            keys.push(key);
        }
        assert_eq!(
            keys.len(),
            Category::EVERY.len(),
            "the loop read every category"
        );
        // `Everything` narrows nothing and so is not a `Category`, but the
        // window draws it beside them and the vocabulary carries it too.
        assert_eq!(crate::words::EVERY_CATEGORY.len(), keys.len() + 1);
    }

    /// **Every category's option name is one a verb will accept, and no two
    /// collide.**
    ///
    /// `alo_capability::Verb::checked` refuses an option that is not an
    /// identifier — lower-case, starting with a letter, underscores and digits
    /// — and refuses two options of one name. Both refusals happen at
    /// declaration time, so a category added with a name a verb will not take
    /// would stop the daemon declaring its verbs at all rather than failing
    /// later.
    ///
    /// **This test is why the rule is a function.** `page-layout` was passed
    /// straight to `Offered::called` and every software verb test failed at
    /// once: *name the option in lower-case words joined by underscores.*
    #[test]
    fn every_category_is_an_option_a_verb_will_take() {
        let mut names = Vec::new();
        for category in Category::EVERY {
            let option = category.as_an_option();
            assert!(
                option.starts_with(|c: char| c.is_ascii_lowercase()),
                "{option} does not start with a lower-case letter"
            );
            assert!(
                option
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{option} is not an identifier a model can send back"
            );
            assert!(!names.contains(&option), "{option} is offered twice");
            names.push(option);
        }
        assert_eq!(
            names.len(),
            Category::EVERY.len(),
            "the loop read every category"
        );
    }

    /// **The option name round-trips to the category it came from**, which is
    /// what `alo_software::looked_at` relies on: it matches what a model sent
    /// against this same function, so an option offered and an option
    /// understood cannot be two different sets.
    #[test]
    fn an_option_name_finds_its_own_category_again() {
        for category in Category::EVERY {
            let option = category.as_an_option();
            let found = Category::EVERY
                .into_iter()
                .find(|other| other.as_an_option() == option);
            assert_eq!(
                found,
                Some(category),
                "{option} did not find its way back to {}",
                category.named()
            );
        }
    }

    /// **No category is named after a product**, and no category is named for
    /// being an alternative to one. The list says what a person makes; a name
    /// borrowed from another company's catalogue describes our list by theirs.
    /// A rule nobody can fail is not a rule, so this one is a test — and the
    /// word `office` is in it because it was the first spelling of
    /// [`Category::Documents`] and had to be argued out.
    #[test]
    fn no_category_is_named_after_somebody_elses_program() {
        for category in Category::EVERY {
            let named = category.named();
            // **Word by word, not letter by letter.** `passwords` contains
            // `word` and is not named after a word processor; matching each
            // hyphen-separated part keeps the rule and stops it refusing a
            // plain English noun (2026-11-17).
            let parts: Vec<&str> = named.split('-').collect();
            for product in [
                "photoshop",
                "illustrator",
                "indesign",
                "lightroom",
                "premiere",
                "audition",
                "acrobat",
                "adobe",
                "office",
                "word",
                "excel",
                "alternative",
                "replacement",
                "outlook",
                "teams",
                "slack",
                "discord",
                "spotify",
                "zoom",
                "steam",
                "kindle",
                "autocad",
                "github",
            ] {
                assert!(
                    !parts.iter().any(|part| part.starts_with(product)),
                    "the category {named} is named after {product}"
                );
            }
        }
    }

    /// Every category is spelled the way the catalogue writes it and has at
    /// least one entry — so a refusal names something a person editing that
    /// file can find, and no shelf in the market is empty.
    #[test]
    fn a_category_is_named_in_the_spelling_the_file_uses() {
        for category in Category::EVERY {
            let named = category.named();
            assert!(!named.is_empty());
            assert!(
                named.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{named}"
            );
            assert!(
                crate::catalogue::THE_CATALOGUE.contains(&format!("category = \"{named}\"")),
                "no entry in the catalogue is a {named}"
            );
        }
    }
}
