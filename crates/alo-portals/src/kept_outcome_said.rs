//! What a request was answered with, said from what the answers file kept.
//!
//! [`crate::Outcome::said`] words an answer while the backend holds the
//! decision. What survives the backend stopping is a [`KeptOutcome`] —
//! identities, never sentences — and this words that, **with the same words**:
//! the same keys, filled the same way, so a person reading the file later reads
//! what the backend would have said at the moment it answered.
//!
//! # Where the file keeps less than the decision held
//!
//! The file keeps no path, no kind of file and not which of `alo-opening`'s
//! findings made a file one nothing opens. Where a sentence needs one of those,
//! the same sentence is used with a clause standing in for what is not kept —
//! [`words::WHAT_IT_ASKED_FOR`], [`words::THAT_KIND_OF_FILE`] — and never a
//! value guessed at. Only `alo-opening`'s two findings, whose sentences are
//! chosen by the finding itself, are said with two sentences of this crate's.
//!
//! A portal over a facility loses nothing: what it asked for is the portal's
//! facility, and a grant that covered a facility was over that facility and
//! nothing else, so `alo-capability`'s refusal is filled exactly as it was.

use alo_capability::words as capability;
use alo_capability::{Ask, Reach};
use alo_strings::{Filling, Said, Strings};

use crate::kept_outcome::{KeptOutcome, NothingOpensAs, RefusedAs};
use crate::portal::{Over, Portal};
use crate::words;

impl KeptOutcome {
    /// What this says, in the language the person reads, for a request to
    /// `portal` from `application` — or from an application that could not be
    /// named, where that is [`None`].
    ///
    /// Reached through [`crate::KeptAnswer::said`], which holds all three.
    #[must_use]
    pub(crate) fn said_for(
        &self,
        application: Option<&str>,
        portal: Portal,
        strings: &Strings,
    ) -> Said {
        let asked = naming(application, strings);
        match self {
            Self::SecretHandedOver { .. } => strings.say(&words::SECRET_HANDED_OVER.key(), &asked),
            Self::Opened { opener, .. } => strings.say(
                &words::OPENED_IN.key(),
                &asked.and("opener", opener.as_str()),
            ),
            Self::AppearanceRead { .. } => strings.say(&words::APPEARANCE_READ.key(), &asked),
            Self::AppearanceSent { .. } => strings.say(&words::APPEARANCE_SENT.key(), &asked),
            Self::NetworkRead { .. } => strings.say(&words::NETWORK_READ.key(), &asked),
            Self::Refused { why } => refused(*why, asked, portal, strings),
            Self::NotARequest { why } => why.said(strings),
            Self::NothingOpens { why } => nothing_opens(why, strings),
            Self::Unanswered { why } => why.said(strings),
        }
    }
}

/// The filling that names who asked: the identifier as it was kept, or
/// [`words::NOBODY_NAMED`] where none was.
pub(crate) fn naming(application: Option<&str>, strings: &Strings) -> Filling {
    match application {
        Some(application) => Filling::of("application", application),
        None => Filling::nothing().and_said(
            "application",
            &strings.say(&words::NOBODY_NAMED.key(), &Filling::nothing()),
        ),
    }
}

/// A refusal of the grants, with the sentence the decision was said with.
fn refused(why: RefusedAs, asked: Filling, portal: Portal, strings: &Strings) -> Said {
    match why {
        RefusedAs::NothingGranted => strings.say(&words::NOTHING_GRANTED.key(), &asked),
        RefusedAs::NeverGranted => strings.say(
            &capability::APPLICATION_NEVER_GRANTED.key(),
            &wanted(portal, asked, strings),
        ),
        RefusedAs::Lapsed => {
            let reach = match portal.over() {
                Over::Facility(facility) => Reach::Facility(facility).said(strings),
                Over::APath => strings.say(&words::WHAT_IT_ASKED_FOR.key(), &Filling::nothing()),
            };
            strings.say(
                &capability::APPLICATION_LAPSED.key(),
                &wanted(portal, asked, strings).and_said("reach", &reach),
            )
        }
    }
}

/// `asked`, with what the portal was asked for in `{wanted}` — the portal's
/// facility through `alo-capability`'s own door, or the clause standing in for
/// a path the file does not keep.
fn wanted(portal: Portal, asked: Filling, strings: &Strings) -> Filling {
    match portal.over() {
        Over::Facility(facility) => Ask::Facility(facility).fills("wanted", asked, strings),
        Over::APath => asked.and_said(
            "wanted",
            &strings.say(&words::WHAT_IT_ASKED_FOR.key(), &Filling::nothing()),
        ),
    }
}

/// Why nothing opened the file, from what the file kept of it.
fn nothing_opens(why: &NothingOpensAs, strings: &Strings) -> Said {
    use alo_applications::words as applications;
    let what = Filling::nothing().and_said(
        "what",
        &strings.say(&words::THAT_KIND_OF_FILE.key(), &Filling::nothing()),
    );
    match why {
        NothingOpensAs::NoApplication { chosen: None } => {
            strings.say(&applications::NOTHING_OPENS.key(), &what)
        }
        NothingOpensAs::NoApplication {
            chosen: Some(chosen),
        } => strings.say(
            &applications::NOTHING_OPENS_CHOICE_NOT_INSTALLED.key(),
            &what.and("chosen", chosen.as_str()),
        ),
        NothingOpensAs::TheFile => {
            strings.say(&words::NOT_A_KIND_ANYTHING_OPENS.key(), &Filling::nothing())
        }
        NothingOpensAs::Unreadable => strings.say(&words::FILE_NOT_READ.key(), &Filling::nothing()),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::answered::Outcome;
    use crate::refused::Refused;
    use alo_capability::{Applicant, Facility, Grant, Grants};
    use alo_strings::Vocabulary;
    use std::time::{Duration, SystemTime};

    /// The three crates whose words an answer is said with.
    fn in_english() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::declare_into(&mut vocabulary).unwrap();
        alo_capability::words::declare_into(&mut vocabulary).unwrap();
        alo_applications::words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **A refusal of a facility is said exactly as it was decided** — never
    /// granted and lapsed both — because the file loses nothing about one.
    #[test]
    fn a_facility_refused_reads_back_as_the_decision_said_it() {
        let strings = in_english();
        let noon = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
        let cheese = Applicant::named("org.gnome.Cheese");
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked_for(
                &cheese.grantee(),
                Reach::Facility(Facility::Camera),
                noon,
                Duration::from_secs(60),
            )
            .unwrap(),
        );
        let never = Refused::NotAllowed {
            portal: Portal::Microphone,
            why: grants
                .allowing(&cheese, &Ask::Facility(Facility::Microphone), noon)
                .unwrap_err(),
        };
        let lapsed = Refused::NotAllowed {
            portal: Portal::Camera,
            why: grants
                .allowing(
                    &cheese,
                    &Ask::Facility(Facility::Camera),
                    noon + Duration::from_secs(120),
                )
                .unwrap_err(),
        };
        for refused in [never, lapsed] {
            let portal = refused.portal();
            let outcome = Outcome::Refused(refused);
            let kept =
                KeptOutcome::from(&outcome).said_for(Some(cheese.as_str()), portal, &strings);
            assert_eq!(kept, outcome.said(&strings));
            assert!(!kept.is_a_bug(), "{kept}");
        }
    }

    /// **Nobody named is said as nobody named**, never as an empty gap.
    #[test]
    fn an_application_nobody_named_fills_the_gap_with_words() {
        let strings = in_english();
        let said = KeptOutcome::Refused {
            why: RefusedAs::NothingGranted,
        }
        .said_for(None, Portal::Secret, &strings);
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(
            said.text()
                .contains("an application that could not be named"),
            "{said}"
        );
    }
}
