//! What the portal backend answered one request with, as the record keeps it.
//!
//! Every request that reaches the backend on the bus is answered, and every
//! answer is one [`Answered`] — the ones that handed something over and, as
//! carefully, the ones that were refused. Law 3's *every execution and every
//! refusal leaves a record*, said of applications.
//!
//! # Typed, and worded when read
//!
//! An [`Outcome`] carries the decision itself — `alo-capability`'s refusal,
//! `alo-applications`' *nothing opens this*, the grants an answer was allowed
//! by — rather than a sentence about it, so a test and a reviewer can ask the
//! record the same question the code answered. [`Outcome::said`] words it with
//! the crate that decided it, so the record and anything a person is shown
//! cannot be two accounts of one answer.
//!
//! # The application is named whenever there was one
//!
//! [`Answered::application`] is the application the sandbox named, once that
//! name was one. It is [`None`] only for a caller that is not a sandboxed
//! application ([`Unanswered::NotIdentified`]), or whose sandbox named
//! something that is not an identifier — and a record that invented a name for
//! either would be a record that can be made to accuse an application of what
//! something else asked.

use std::time::SystemTime;

use alo_applications::NothingOpens;
use alo_capability::Applicant;
use alo_strings::{Filling, Said, Strings};

use crate::judging::Allowed;
use crate::not_a_request::NotARequest;
use crate::open_with::OpensWith;
use crate::portal::Portal;
use crate::refused::Refused;
use crate::words;

/// One request the backend answered, and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answered {
    /// When it was answered.
    at: SystemTime,
    /// The application that asked, when there was one.
    application: Option<Applicant>,
    /// The portal it asked.
    portal: Portal,
    /// What it was answered with.
    outcome: Outcome,
}

impl Answered {
    /// A request to `portal`, from `application`, answered with `outcome` at `at`.
    #[must_use]
    pub const fn new(
        at: SystemTime,
        application: Option<Applicant>,
        portal: Portal,
        outcome: Outcome,
    ) -> Self {
        Self {
            at,
            application,
            portal,
            outcome,
        }
    }

    /// When it was answered.
    #[must_use]
    pub const fn at(&self) -> SystemTime {
        self.at
    }

    /// The application that asked, when there was one.
    #[must_use]
    pub const fn application(&self) -> Option<&Applicant> {
        self.application.as_ref()
    }

    /// The portal it asked.
    #[must_use]
    pub const fn portal(&self) -> Portal {
        self.portal
    }

    /// What it was answered with.
    #[must_use]
    pub const fn outcome(&self) -> &Outcome {
        &self.outcome
    }
}

/// What a request was answered with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The Secret portal handed the application its own secret.
    SecretHandedOver(Allowed),
    /// The file was opened in the application that opens its kind.
    Opened(OpensWith),
    /// The application read the person's appearance settings.
    AppearanceRead(Allowed),
    /// The application was sent the appearance settings that changed.
    AppearanceSent(Allowed),
    /// The grants refused it.
    Refused(Refused),
    /// What arrived was never a request.
    NotARequest(NotARequest),
    /// Nothing opens the file.
    NothingOpens(NothingOpens),
    /// It was allowed as far as it went, or never reached the grants, and could
    /// not be answered.
    Unanswered(Unanswered),
}

impl Outcome {
    /// The response code the portal specification gives this answer: `0` for
    /// one that did what was asked, and `2` — *the interaction was ended in
    /// some other way* — for every other.
    ///
    /// `1`, *cancelled by the user*, is never sent: nothing here puts a
    /// question to a person, so no person cancelled anything, and saying so
    /// would tell an application a person refused it when the grants did.
    #[must_use]
    pub const fn response(&self) -> u32 {
        match self {
            Self::SecretHandedOver(_)
            | Self::Opened(_)
            | Self::AppearanceRead(_)
            | Self::AppearanceSent(_) => 0,
            Self::Refused(_)
            | Self::NotARequest(_)
            | Self::NothingOpens(_)
            | Self::Unanswered(_) => 2,
        }
    }

    /// Whether the application was refused what it asked for.
    #[must_use]
    pub const fn was_refused(&self) -> bool {
        self.response() != 0
    }

    /// What this says, in the language the person reads — each the sentence of
    /// the crate that decided it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::SecretHandedOver(allowed) => strings.say(
                &words::SECRET_HANDED_OVER.key(),
                &Filling::of("application", allowed.application().as_str()),
            ),
            Self::Opened(opens) => strings.say(
                &words::OPENED_IN.key(),
                &Filling::of("application", opens.allowed().application().as_str())
                    .and("opener", opens.opener().application().identifier()),
            ),
            Self::AppearanceRead(allowed) => strings.say(
                &words::APPEARANCE_READ.key(),
                &Filling::of("application", allowed.application().as_str()),
            ),
            Self::AppearanceSent(allowed) => strings.say(
                &words::APPEARANCE_SENT.key(),
                &Filling::of("application", allowed.application().as_str()),
            ),
            Self::Refused(refused) => refused.said(strings),
            Self::NotARequest(not) => not.said(strings),
            Self::NothingOpens(nothing) => nothing.said(strings),
            Self::Unanswered(unanswered) => unanswered.said(strings),
        }
    }
}

/// Why a request could not be answered with what it asked for, where neither
/// the grants nor *what opens what* are the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unanswered {
    /// The caller is not a sandboxed application, so nothing could say which
    /// application it is.
    NotIdentified,
    /// The request's `handle_token` is not letters, digits and underscores.
    NotAToken,
    /// The machine's grants could not be read.
    GrantsUnread,
    /// What is installed, what applications declare or what the person chose
    /// could not be read.
    ApplicationsUnread,
    /// The keyring is not there, is locked, or refused.
    KeyringUnavailable,
    /// The secret could not be written to what the application handed over.
    NotWritten,
    /// What the application handed over to be opened is not a file.
    NotAFile,
    /// The application that opens the file did not open it when asked.
    NotOpened {
        /// Its identifier.
        opener: String,
    },
    /// A web link or a folder, which nothing on this machine decides yet.
    NotDecidedHere,
    /// A setting this machine does not share with applications: another
    /// namespace than appearance, or a key it does not hold.
    NoSuchSetting,
    /// The person's appearance settings, or the time of day they are answered
    /// at, could not be read.
    AppearanceUnread,
}

impl Unanswered {
    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let (word, filling) = match self {
            Self::NotIdentified => (words::NOT_IDENTIFIED, Filling::nothing()),
            Self::NotAToken => (words::NOT_A_TOKEN, Filling::nothing()),
            Self::GrantsUnread => (words::GRANTS_UNREAD, Filling::nothing()),
            Self::ApplicationsUnread => (words::APPLICATIONS_UNREAD, Filling::nothing()),
            Self::KeyringUnavailable => (words::KEYRING_UNAVAILABLE, Filling::nothing()),
            Self::NotWritten => (words::NOT_WRITTEN, Filling::nothing()),
            Self::NotAFile => (words::NOT_A_FILE, Filling::nothing()),
            Self::NotOpened { opener } => {
                (words::NOT_OPENED, Filling::of("opener", opener.as_str()))
            }
            Self::NotDecidedHere => (words::NOT_DECIDED_HERE, Filling::nothing()),
            Self::NoSuchSetting => (words::NO_SUCH_SETTING, Filling::nothing()),
            Self::AppearanceUnread => (words::APPEARANCE_UNREAD, Filling::nothing()),
        };
        strings.say(&word.key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Only an answer that did what was asked is a `0`**, and nothing is ever
    /// a person's cancellation.
    #[test]
    fn only_what_was_done_is_answered_as_done() {
        let refused = Outcome::Refused(Refused::NothingGranted {
            application: Applicant::named("org.example.Stranger"),
            portal: Portal::Secret,
        });
        assert_eq!(refused.response(), 2);
        assert!(refused.was_refused());
        for unanswered in [
            Unanswered::NotIdentified,
            Unanswered::NotAToken,
            Unanswered::GrantsUnread,
            Unanswered::ApplicationsUnread,
            Unanswered::KeyringUnavailable,
            Unanswered::NotWritten,
            Unanswered::NotAFile,
            Unanswered::NotOpened {
                opener: "org.gnome.Papers".to_owned(),
            },
            Unanswered::NotDecidedHere,
            Unanswered::NoSuchSetting,
            Unanswered::AppearanceUnread,
        ] {
            let outcome = Outcome::Unanswered(unanswered);
            assert_eq!(outcome.response(), 2, "{outcome:?}");
            assert!(outcome.was_refused());
        }
        assert_eq!(
            Outcome::NotARequest(NotARequest::NotAnIdentifier).response(),
            2
        );
    }

    /// **Every way of being unanswered is said in the vocabulary**, with the
    /// opener named where there is one.
    #[test]
    fn every_unanswered_request_is_a_sentence() {
        let strings = Strings::of(crate::portal_words().unwrap());
        let not_opened = Unanswered::NotOpened {
            opener: "org.gnome.Papers".to_owned(),
        }
        .said(&strings);
        assert!(!not_opened.is_a_bug(), "{not_opened}");
        assert!(
            not_opened.text().contains("org.gnome.Papers"),
            "{not_opened}"
        );
        for unanswered in [
            Unanswered::NotIdentified,
            Unanswered::NotAToken,
            Unanswered::GrantsUnread,
            Unanswered::ApplicationsUnread,
            Unanswered::KeyringUnavailable,
            Unanswered::NotWritten,
            Unanswered::NotAFile,
            Unanswered::NotDecidedHere,
            Unanswered::NoSuchSetting,
            Unanswered::AppearanceUnread,
        ] {
            let said = unanswered.said(&strings);
            assert!(!said.is_a_bug(), "{unanswered:?}: {said}");
        }
    }
}
