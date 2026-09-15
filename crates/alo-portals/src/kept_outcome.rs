//! What a request was answered with, as the answers file keeps it.
//!
//! A mirror of [`Outcome`], and deliberately a separate type, for
//! `alo_record::Written`'s reason. An [`Outcome`] carries the decision itself —
//! `alo-capability`'s refusal, `alo-applications`' opener — and those types have
//! no way back from a disk, because a decision read back off a disk would be
//! one nothing decided. The answers file has to be read back, so it needs a
//! type whose meaning is *this is what was written down*: which answer, why,
//! and the handles of the grants it was allowed by.
//!
//! The two are kept in step by [`From<&Outcome>`], whose match is exhaustive: a
//! new way of answering an application does not compile until somebody has
//! decided how the file keeps it.
//!
//! **Identities, never sentences.** A refusal is kept as `nothing-granted`, not
//! as the English a person read, so the file says the same thing on a Greek
//! machine as on a German one; [`crate::words`] says it when it is shown.
//!
//! **There is no way back** to an [`Outcome`], a grant or a request. A record is
//! evidence, not an instruction.

use alo_applications::{Because, NothingOpens};
use alo_capability::{GrantId, NotAllowed};
use serde::{Deserialize, Serialize};

use crate::answered::{Outcome, Unanswered};
use crate::not_a_request::NotARequest;
use crate::refused::Refused;

/// What a request was answered with, written down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "answer", rename_all = "kebab-case")]
pub enum KeptOutcome {
    /// The application was handed its own secret, allowed by these grants.
    SecretHandedOver {
        /// The grant behind each thing the request was for.
        against: Vec<GrantId>,
    },
    /// The file was handed to `opener` to open, allowed by these grants.
    Opened {
        /// The application that opens it.
        opener: String,
        /// Whether that was the person's choice, rather than a declaration.
        chosen: bool,
        /// The grants over the file and over the opener.
        against: Vec<GrantId>,
    },
    /// The application read the appearance settings, allowed by these grants.
    AppearanceRead {
        /// The grant behind each thing the request was for.
        against: Vec<GrantId>,
    },
    /// The application was sent the appearance settings that changed.
    AppearanceSent {
        /// The grant behind each thing the request was for.
        against: Vec<GrantId>,
    },
    /// The grants refused it.
    Refused {
        /// How.
        why: RefusedAs,
    },
    /// What arrived was never a request.
    NotARequest {
        /// Why not.
        why: NotARequest,
    },
    /// Nothing opens the file.
    NothingOpens {
        /// Why nothing does.
        why: NothingOpensAs,
    },
    /// It could not be answered with what it asked for.
    Unanswered {
        /// Why not.
        why: Unanswered,
    },
}

impl KeptOutcome {
    /// Whether the application was refused what it asked for — the same
    /// question [`Outcome::was_refused`] answers, asked of the file.
    #[must_use]
    pub const fn was_refused(&self) -> bool {
        match self {
            Self::SecretHandedOver { .. }
            | Self::Opened { .. }
            | Self::AppearanceRead { .. }
            | Self::AppearanceSent { .. } => false,
            Self::Refused { .. }
            | Self::NotARequest { .. }
            | Self::NothingOpens { .. }
            | Self::Unanswered { .. } => true,
        }
    }
}

/// How the grants refused a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefusedAs {
    /// The application holds no grant at all.
    NothingGranted,
    /// Nothing it holds has ever covered what it asked for.
    NeverGranted,
    /// A grant covered it, and has expired.
    Lapsed,
}

/// Why nothing opens a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NothingOpensAs {
    /// No installed application opens its kind.
    NoApplication {
        /// What the person chose for the kind, when that is not installed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        chosen: Option<String>,
    },
    /// The file is not a kind anything opens: a program, nothing, damaged,
    /// protected with a password, or unrecognised.
    TheFile,
    /// The file would not be read.
    Unreadable,
}

impl From<&Outcome> for KeptOutcome {
    /// Exhaustive on purpose: a new way of answering does not compile until
    /// somebody has decided how the file keeps it.
    fn from(outcome: &Outcome) -> Self {
        match outcome {
            Outcome::SecretHandedOver(allowed) => Self::SecretHandedOver {
                against: allowed.against().to_vec(),
            },
            Outcome::Opened(opens) => Self::Opened {
                opener: opens.opener().application().identifier().to_owned(),
                chosen: matches!(opens.opener().because(), Because::ThePersonChoseIt),
                against: opens.allowed().against().to_vec(),
            },
            Outcome::AppearanceRead(allowed) => Self::AppearanceRead {
                against: allowed.against().to_vec(),
            },
            Outcome::AppearanceSent(allowed) => Self::AppearanceSent {
                against: allowed.against().to_vec(),
            },
            Outcome::Refused(refused) => Self::Refused {
                why: RefusedAs::from(refused),
            },
            Outcome::NotARequest(not) => Self::NotARequest { why: *not },
            Outcome::NothingOpens(nothing) => Self::NothingOpens {
                why: NothingOpensAs::from(nothing),
            },
            Outcome::Unanswered(unanswered) => Self::Unanswered {
                why: unanswered.clone(),
            },
        }
    }
}

impl From<&Refused> for RefusedAs {
    fn from(refused: &Refused) -> Self {
        match refused {
            Refused::NothingGranted { .. } => Self::NothingGranted,
            Refused::NotAllowed {
                why: NotAllowed::Never { .. },
                ..
            } => Self::NeverGranted,
            Refused::NotAllowed {
                why: NotAllowed::Lapsed { .. },
                ..
            } => Self::Lapsed,
        }
    }
}

impl From<&NothingOpens> for NothingOpensAs {
    fn from(nothing: &NothingOpens) -> Self {
        match nothing {
            NothingOpens::NoApplication { chosen, .. } => Self::NoApplication {
                chosen: chosen.clone(),
            },
            NothingOpens::TheFile(_) => Self::TheFile,
            NothingOpens::Unreadable(_) => Self::Unreadable,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::portal::Portal;
    use alo_capability::{Applicant, Ask, Facility, Grant, Grants, Reach};
    use std::time::{Duration, SystemTime};

    /// **Refused as the grants refused it** — nothing granted, never granted,
    /// and lapsed are three different things for a person reading the file.
    #[test]
    fn each_refusal_of_the_grants_is_kept_as_itself() {
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
        let camera = Ask::Facility(Facility::Camera);
        let microphone = Ask::Facility(Facility::Microphone);
        let never = grants.allowing(&cheese, &microphone, noon).unwrap_err();
        let lapsed = grants
            .allowing(&cheese, &camera, noon + Duration::from_secs(120))
            .unwrap_err();
        for (refused, kept) in [
            (
                Refused::NothingGranted {
                    application: cheese.clone(),
                    portal: Portal::Camera,
                },
                RefusedAs::NothingGranted,
            ),
            (
                Refused::NotAllowed {
                    portal: Portal::Microphone,
                    why: never,
                },
                RefusedAs::NeverGranted,
            ),
            (
                Refused::NotAllowed {
                    portal: Portal::Camera,
                    why: lapsed,
                },
                RefusedAs::Lapsed,
            ),
        ] {
            let outcome = KeptOutcome::from(&Outcome::Refused(refused));
            assert_eq!(outcome, KeptOutcome::Refused { why: kept });
            assert!(outcome.was_refused());
        }
    }

    /// **Kept by identity, in kebab case**, the way the contract spells it.
    #[test]
    fn an_outcome_is_written_by_its_identity() {
        let written = serde_json::to_string(&KeptOutcome::Unanswered {
            why: Unanswered::NotOpened {
                opener: "org.gnome.Papers".to_owned(),
            },
        })
        .unwrap();
        assert_eq!(
            written,
            r#"{"answer":"unanswered","why":{"not-opened":{"opener":"org.gnome.Papers"}}}"#
        );
        let written = serde_json::to_string(&KeptOutcome::Refused {
            why: RefusedAs::NothingGranted,
        })
        .unwrap();
        assert_eq!(written, r#"{"answer":"refused","why":"nothing-granted"}"#);
        let written = serde_json::to_string(&KeptOutcome::SecretHandedOver {
            against: vec![GrantId::numbered(7)],
        })
        .unwrap();
        assert_eq!(written, r#"{"answer":"secret-handed-over","against":[7]}"#);
    }

    /// **Only what was done is not a refusal**, as the response codes say.
    #[test]
    fn a_kept_outcome_is_refused_exactly_when_its_response_was_two() {
        let outcomes = [
            Outcome::Unanswered(Unanswered::NotIdentified),
            Outcome::NotARequest(NotARequest::NotAnIdentifier),
            Outcome::Refused(Refused::NothingGranted {
                application: Applicant::named("org.gnome.Cheese"),
                portal: Portal::Secret,
            }),
        ];
        for outcome in outcomes {
            assert_eq!(
                KeptOutcome::from(&outcome).was_refused(),
                outcome.was_refused(),
                "{outcome:?}"
            );
        }
    }
}
