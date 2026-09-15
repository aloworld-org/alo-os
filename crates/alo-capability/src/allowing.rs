//! An application's question of the grants, and what it is told when the answer
//! is no.
//!
//! [ADR 0005](../../../docs/decisions/0005-applications-are-sandboxed-and-ask.md)
//! says a portal request is ADR 0001's sentence with *application* in place of
//! *agent*. [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md)
//! says what that takes: the same list, the same search, and a refusal of its
//! own. This file is the last of those.
//!
//! # The same search
//!
//! [`Grants::allowing`] asks exactly what [`Grants::permitting`] asks — which
//! grant, naming this grantee and covering this ask, is running at this moment
//! — through the one crate-private search behind both. Nothing about coverage,
//! expiry or identity is restated here, so an application's grant cannot come
//! to mean more than an agent's would.
//!
//! # A refusal of its own
//!
//! [`crate::NotGranted`] says *grants are made by picking a folder* and, on a
//! machine with no agent, *this machine has no agent*. Neither is true of a
//! video-call application asking for the microphone. [`NotAllowed`] has the two
//! refusals an application can get — *it expired* and *you never granted it* —
//! and no third: declining the agent does not touch an application's grants
//! (ADR 0040, part 3), so there is no *no agent* to tell it about.

use std::time::SystemTime;

use alo_strings::{Filling, Said, Strings};

use crate::grantee::Applicant;
use crate::grants::{GrantId, Grants};
use crate::reach::{Ask, Reach};
use crate::words;

/// Why no grant allowed an application something.
///
/// A value rather than a sentence, for [`crate::NotGranted`]'s reason: deciding
/// never depends on a vocabulary having been loaded, and the screen and the
/// record word the same value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAllowed {
    /// A grant to this application covered it and has expired.
    Lapsed {
        /// The application that asked.
        application: Applicant,
        /// What the expired grant was over.
        reach: Reach,
        /// What was asked for.
        wanted: Ask,
    },
    /// Nothing this application holds has ever covered it.
    Never {
        /// The application that asked.
        application: Applicant,
        /// What was asked for.
        wanted: Ask,
    },
}

impl NotAllowed {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::Lapsed { .. } => words::APPLICATION_LAPSED,
            Self::Never { .. } => words::APPLICATION_NEVER_GRANTED,
        }
    }

    /// The application that asked.
    #[must_use]
    pub fn application(&self) -> &Applicant {
        match self {
            Self::Lapsed { application, .. } | Self::Never { application, .. } => application,
        }
    }

    /// What was asked for.
    #[must_use]
    pub fn wanted(&self) -> &Ask {
        match self {
            Self::Lapsed { wanted, .. } | Self::Never { wanted, .. } => wanted,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::capability_words`] answers with the key, marked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = self.wanted().fills(
            "wanted",
            Filling::of("application", self.application().as_str().to_owned()),
            strings,
        );
        let filling = match self {
            Self::Lapsed { reach, .. } => filling.and_said("reach", &reach.said(strings)),
            Self::Never { .. } => filling,
        };
        strings.say(&self.word().key(), &filling)
    }
}

impl Grants {
    /// Which grant allows this application to reach this thing, at this moment
    /// — or why none does.
    ///
    /// The application's door onto the list [`Grants::permitting`] is the
    /// agent's door onto. One search behind both; only the refusal differs.
    ///
    /// # Errors
    /// [`NotAllowed`], carrying what was asked for and the grant that ran out
    /// where there was one.
    pub fn allowing(
        &self,
        application: &Applicant,
        ask: &Ask,
        now: SystemTime,
    ) -> Result<GrantId, NotAllowed> {
        self.searched(&application.grantee(), ask, now)
            .map_err(|lapsed| match lapsed {
                Some(held) => NotAllowed::Lapsed {
                    application: application.clone(),
                    reach: held.grant.reach.clone(),
                    wanted: ask.clone(),
                },
                None => NotAllowed::Never {
                    application: application.clone(),
                    wanted: ask.clone(),
                },
            })
    }

    /// Whether this application holds any grant at all, at this moment.
    ///
    /// The question a portal asks first: an application nobody has granted
    /// anything is refused before any dialog is imagined, rather than having
    /// its request turned into a question put to the person.
    #[must_use]
    pub fn allows_anything(&self, application: &Applicant, now: SystemTime) -> bool {
        let grantee = application.grantee();
        self.held_by(&grantee, now).next().is_some()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::facility::Facility;
    use crate::grant::{Grant, Grantee};
    use crate::testing::{in_english, translated};
    use std::path::PathBuf;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn cheese() -> Applicant {
        Applicant::named("org.gnome.Cheese")
    }

    fn camera() -> Ask {
        Ask::facility(Facility::Camera)
    }

    /// A list with the camera granted to Cheese for an hour, and a folder
    /// granted to an agent, so every answer below is read off a mixed list.
    fn granted() -> (Grants, GrantId) {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from("/home/anna/Invoices")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let id = grants.grant(
            Grant::checked_for(
                &cheese().grantee(),
                Reach::Facility(Facility::Camera),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        (grants, id)
    }

    /// **An application is allowed what it was granted, and nothing beside
    /// it** — not the microphone, not a path, not what an agent holds.
    #[test]
    fn an_application_is_allowed_what_it_was_granted_and_nothing_beside_it() {
        let (grants, id) = granted();
        assert_eq!(grants.allowing(&cheese(), &camera(), noon()), Ok(id));

        for wanted in [
            Ask::facility(Facility::Microphone),
            Ask::facility(Facility::ScreenContinuously),
            Ask::path("/dev/video0"),
            Ask::path("/home/anna/Invoices/march.pdf"),
        ] {
            assert_eq!(
                grants.allowing(&cheese(), &wanted, noon()),
                Err(NotAllowed::Never {
                    application: cheese(),
                    wanted: wanted.clone(),
                }),
                "{wanted:?}"
            );
        }

        // Another application is not allowed Cheese's camera.
        assert!(
            grants
                .allowing(&Applicant::named("org.example.Spy"), &camera(), noon())
                .is_err()
        );
        // And an application named like the agent is not allowed its folder.
        assert!(
            grants
                .allowing(
                    &Applicant::named("@files"),
                    &Ask::path("/home/anna/Invoices/march.pdf"),
                    noon()
                )
                .is_err()
        );
    }

    /// **An expired grant allows nothing, and says it expired**; a revoked one
    /// allows nothing on the next question.
    #[test]
    fn an_expired_or_revoked_grant_allows_nothing() {
        let (mut grants, id) = granted();
        assert_eq!(
            grants.allowing(&cheese(), &camera(), noon() + hour()),
            Err(NotAllowed::Lapsed {
                application: cheese(),
                reach: Reach::Facility(Facility::Camera),
                wanted: camera(),
            })
        );
        assert!(grants.revoke(id));
        assert!(matches!(
            grants.allowing(&cheese(), &camera(), noon()),
            Err(NotAllowed::Never { .. })
        ));
    }

    /// **Asking is not granting.** A refused question leaves the list as it
    /// was.
    #[test]
    fn asking_never_widens_what_an_application_holds() {
        let (grants, _) = granted();
        let before = serde_json::to_string(&grants).unwrap();
        for facility in Facility::EVERY {
            let _answer = grants.allowing(&cheese(), &Ask::facility(facility), noon());
            let _answer = grants.allowing(
                &Applicant::named("org.example.Spy"),
                &Ask::facility(facility),
                noon(),
            );
        }
        assert_eq!(serde_json::to_string(&grants).unwrap(), before);
    }

    /// **Holding anything** is asked of the application, at this moment.
    #[test]
    fn whether_an_application_holds_anything_is_asked_at_a_moment() {
        let (grants, _) = granted();
        assert!(grants.allows_anything(&cheese(), noon()));
        assert!(!grants.allows_anything(&cheese(), noon() + hour()));
        assert!(!grants.allows_anything(&Applicant::named("@files"), noon()));
        assert!(!grants.allows_anything(&Applicant::named("org.example.Spy"), noon()));
        // The agent's own grant is still the agent's.
        assert_eq!(grants.held_by(&Grantee::named("@files"), noon()).count(), 1);
    }

    /// **An application is never called an agent, and never told to pick a
    /// folder.**
    #[test]
    fn an_application_is_refused_in_its_own_words() {
        let strings = in_english();
        let never = NotAllowed::Never {
            application: cheese(),
            wanted: camera(),
        }
        .said(&strings);
        assert_eq!(
            never.text(),
            "org.gnome.Cheese has not been granted the camera — an application reaches only what \
             you grant it, never what it asks for"
        );
        let lapsed = NotAllowed::Lapsed {
            application: cheese(),
            reach: Reach::Facility(Facility::Camera),
            wanted: camera(),
        }
        .said(&strings);
        for said in [&never, &lapsed] {
            assert!(!said.text().contains("agent"), "{said}");
            assert!(!said.text().contains("folder"), "{said}");
        }
        assert!(lapsed.text().contains("has expired"), "{lapsed}");
    }

    /// And in the reader's language, only as translated as what it names.
    #[test]
    fn an_applications_refusal_is_as_translated_as_what_it_names() {
        let strings = translated(&[
            (
                words::APPLICATION_NEVER_GRANTED,
                "{application} hat keine Berechtigung für {wanted}",
            ),
            (words::THE_CAMERA, "die Kamera"),
        ]);
        let said = NotAllowed::Never {
            application: cheese(),
            wanted: camera(),
        }
        .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert_eq!(
            said.text(),
            "org.gnome.Cheese hat keine Berechtigung für die Kamera"
        );

        let half = translated(&[(
            words::APPLICATION_NEVER_GRANTED,
            "{application} hat keine Berechtigung für {wanted}",
        )]);
        let said = NotAllowed::Never {
            application: cheese(),
            wanted: camera(),
        }
        .said(&half);
        assert!(!said.is_translated(), "{said}");
    }
}
