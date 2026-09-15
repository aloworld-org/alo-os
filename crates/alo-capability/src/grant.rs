//! One grant: who may reach what, and until when.
//!
//! A grant is the durable half of the capability model (ADR 0001 §3). Verbs
//! are what may be *done*; a grant is what they may be done *to*, and it is the
//! only thing in alo OS that decides reach.
//!
//! The properties a grant has to have are enforced here, at construction,
//! because a check somewhere else is a check somebody can forget to call:
//!
//! - it names one agent or one application — a grant to nobody grants
//!   nothing;
//! - its path is a full path with no `..` in it, so it means the same thing
//!   wherever it is read;
//! - it is not the whole machine ([`GrantError::TheWholeMachine`]);
//! - a [`Facility`](crate::Facility) — the camera, the screen — is granted to
//!   an application and never to an agent ([`GrantError::NotForAnAgent`]).
//!   What an agent sees of the person's machine is offered at the moment they
//!   ask it, never watched, and a grant to the camera would be the opposite;
//! - a terminal is granted to an agent never ([`GrantError::APersonsOwn`],
//!   ADR 0043), because whatever is typed into one runs;
//! - **it ends.** [`Grant::checked`] takes how long it lasts and refuses zero,
//!   and there is no variant meaning "for ever". A grant that outlives the
//!   reason it was made is the failure this crate exists to make impossible,
//!   and the reliable way to prevent it is to leave the type no way to express
//!   it.
//!
//! Nothing here reads the clock: the moment a grant starts is passed in, and so
//! is the moment any later question is asked about it.

use std::time::{Duration, SystemTime};

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

pub use crate::grantee::Grantee;
use crate::path::{is_a_root, is_usable};
use crate::persons_own::is_a_persons_own;
use crate::reach::{Ask, Reach};
use crate::words;

/// Why a grant could not be made.
///
/// The messages say what to do about it. Somebody reads these having just
/// picked the wrong thing in a dialog, not having just read this file.
///
/// **No `Display`, and therefore not a `std::error::Error`.** The only road to
/// words is [`GrantError::said`], which takes the strings the person in front
/// of the machine reads — a `Display` here would be an English sentence one
/// `to_string()` away from a settings panel whose author had no reason to think
/// about it. What is given up is `std::error::Error` on a type that was never
/// an error a programmer handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantError {
    /// No agent was named.
    Anonymous,
    /// No application was named.
    NoApplicationNamed,
    /// The folder, file or application was empty.
    NothingNamed,
    /// A grant to `/`, or to any other spelling of the whole machine.
    TheWholeMachine,
    /// A relative path, which means something different depending on where it
    /// is read from.
    NotAFullPath,
    /// A path containing `..`, which can leave the folder it appears to be in.
    CouldLeadElsewhere,
    /// A grant lasting no time at all.
    NoTime,
    /// A grant so long it has no end this machine can represent.
    NoEnd,
    /// A grant to an agent over something that is not a path — the camera,
    /// the screen, the person's notifications.
    ///
    /// Those are granted to applications (ADR 0040). An agent is offered what
    /// it needs at the moment it is asked, and a durable grant to the camera
    /// would be a background reader by another name.
    NotForAnAgent,
    /// A grant to an agent over an application that is a person's own — a
    /// terminal ([`crate::persons_own`], ADR 0043).
    ///
    /// Whatever is typed into a terminal runs, so a grant over one is a grant
    /// to the whole machine by another road, and it is refused for the same
    /// reason [`GrantError::TheWholeMachine`] is. An application may still be
    /// allowed one; only an agent is refused.
    APersonsOwn,
}

impl GrantError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::Anonymous => words::ANONYMOUS,
            Self::NoApplicationNamed => words::NO_APPLICATION_NAMED,
            Self::NothingNamed => words::NOTHING_NAMED,
            Self::TheWholeMachine => words::THE_WHOLE_MACHINE,
            Self::NotAFullPath => words::GRANT_NOT_A_FULL_PATH,
            Self::CouldLeadElsewhere => words::GRANT_COULD_LEAD_ELSEWHERE,
            Self::NoTime => words::GRANT_NO_TIME,
            Self::NoEnd => words::GRANT_NO_END,
            Self::NotForAnAgent => words::NOT_FOR_AN_AGENT,
            Self::APersonsOwn => words::PERSONS_OWN,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::capability_words`] answers with the key, marked.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// A grant a person made.
///
/// `Serialize` because the list of grants outlives the session that made it,
/// and a person who granted a folder on Monday should find it in the list on
/// Tuesday — still expiring at the moment it was always going to expire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    /// The agent or application this grant is for.
    pub grantee: Grantee,
    /// What it covers.
    pub reach: Reach,
    /// When it was made — shown in the list, so a person can see what they did
    /// and when.
    pub granted_at: SystemTime,
    /// When it stops. Not optional, on purpose.
    pub expires: SystemTime,
}

impl Grant {
    /// Make a grant to the agent of this name, checking everything that has
    /// to be true of one.
    ///
    /// `from` is the moment it starts, normally now; `lasting` is how long it
    /// runs for. The pair is the argument rather than an end time because
    /// every caller has a duration in hand — an hour, the working day, the
    /// turn — and a subtraction is one more place to get the arithmetic wrong.
    ///
    /// # Errors
    /// [`GrantError`], saying what to fix.
    pub fn checked(
        grantee: &str,
        reach: Reach,
        from: SystemTime,
        lasting: Duration,
    ) -> Result<Self, GrantError> {
        Self::checked_for(&Grantee::named(grantee), reach, from, lasting)
    }

    /// Make a grant to this grantee — an agent or an application — checking
    /// everything that has to be true of one.
    ///
    /// [`Grant::checked`] is this for an agent. An application's grant is made
    /// here with [`crate::Applicant::grantee`], and is held to every rule an
    /// agent's is, with one more for the agent: a [`crate::Facility`] is never
    /// granted to one.
    ///
    /// # Errors
    /// [`GrantError`], saying what to fix.
    pub fn checked_for(
        grantee: &Grantee,
        reach: Reach,
        from: SystemTime,
        lasting: Duration,
    ) -> Result<Self, GrantError> {
        if grantee.as_str().is_empty() {
            return Err(if grantee.is_an_application() {
                GrantError::NoApplicationNamed
            } else {
                GrantError::Anonymous
            });
        }
        if matches!(reach, Reach::Facility(_)) && !grantee.is_an_application() {
            return Err(GrantError::NotForAnAgent);
        }
        let grantee = grantee.clone();
        let reach = checked_reach(reach)?;
        if let Reach::Application(identifier) = &reach
            && !grantee.is_an_application()
            && is_a_persons_own(identifier)
        {
            return Err(GrantError::APersonsOwn);
        }
        if lasting.is_zero() {
            return Err(GrantError::NoTime);
        }
        let expires = from.checked_add(lasting).ok_or(GrantError::NoEnd)?;
        Ok(Self {
            grantee,
            reach,
            granted_at: from,
            expires,
        })
    }

    /// Whether this grant is still running at this moment.
    ///
    /// A grant that expires at five o'clock does not include five o'clock. The
    /// boundary has to fall somewhere, and it falls on the side that reaches
    /// less.
    #[must_use]
    pub fn is_active_at(&self, now: SystemTime) -> bool {
        self.expires > now
    }

    /// How much of the grant is left, or `None` once it has expired.
    #[must_use]
    pub fn expires_in(&self, now: SystemTime) -> Option<Duration> {
        self.expires
            .duration_since(now)
            .ok()
            .filter(|d| !d.is_zero())
    }

    /// Whether this grant is for that grantee.
    #[must_use]
    pub fn is_for(&self, grantee: &Grantee) -> bool {
        &self.grantee == grantee
    }

    /// Whether this grant, at this moment, permits that ask.
    ///
    /// Both halves matter and both are checked here so that no caller can hold
    /// one without the other: an expired grant covering the right folder
    /// permits nothing at all.
    #[must_use]
    pub fn permits(&self, ask: &Ask, now: SystemTime) -> bool {
        self.is_active_at(now) && self.reach.covers(ask)
    }
}

/// Check what a grant is over, before it becomes one.
fn checked_reach(reach: Reach) -> Result<Reach, GrantError> {
    match &reach {
        Reach::Folder(path) | Reach::File(path) => {
            if path.as_os_str().is_empty() {
                return Err(GrantError::NothingNamed);
            }
            // The whole-machine check comes before the shape checks so that a
            // grant to `/` is refused for the reason a person needs to hear,
            // rather than for a technicality about its spelling.
            if is_a_root(path) {
                return Err(GrantError::TheWholeMachine);
            }
            if !path.has_root() {
                return Err(GrantError::NotAFullPath);
            }
            if !is_usable(path) {
                return Err(GrantError::CouldLeadElsewhere);
            }
        }
        Reach::Application(id) => {
            if id.trim().is_empty() {
                return Err(GrantError::NothingNamed);
            }
            return Ok(Reach::Application(id.trim().to_owned()));
        }
        // A facility is one of a closed list, so there is nothing in it to be
        // empty, relative or the whole machine.
        Reach::Facility(_) => {}
    }
    Ok(reach)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use std::path::PathBuf;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn invoices() -> Reach {
        Reach::Folder(PathBuf::from("/home/anna/Invoices"))
    }

    fn granted() -> Grant {
        Grant::checked("@files", invoices(), noon(), hour()).unwrap()
    }

    /// ADR 0001 §3, stated as a test because it is the sentence somebody will
    /// one day want an exception to.
    #[test]
    fn there_is_no_grant_to_the_whole_machine() {
        for root in ["/", "//", "/."] {
            let err =
                Grant::checked("@files", Reach::Folder(root.into()), noon(), hour()).unwrap_err();
            assert_eq!(err, GrantError::TheWholeMachine, "{root}");
        }
        assert!(
            GrantError::TheWholeMachine
                .said(&in_english())
                .text()
                .contains("pick the folder you actually mean")
        );
    }

    /// An expired grant grants nothing. Not less, not read-only — nothing.
    #[test]
    fn an_expired_grant_permits_nothing() {
        let grant = granted();
        let ask = Ask::path("/home/anna/Invoices/march.pdf");
        assert!(grant.permits(&ask, noon()));
        assert!(grant.permits(&ask, noon() + Duration::from_secs(3_599)));
        assert!(!grant.permits(&ask, noon() + hour()));
        assert!(!grant.permits(&ask, noon() + hour() + Duration::from_secs(1)));
        assert!(!grant.is_active_at(noon() + hour()));
        assert!(grant.expires_in(noon() + hour()).is_none());
        assert_eq!(grant.expires_in(noon()), Some(hour()));
    }

    /// A grant cannot be made without an end. There is no variant for "for
    /// ever", and zero is refused rather than treated as "until revoked".
    #[test]
    fn a_grant_has_to_end() {
        assert_eq!(
            Grant::checked("@files", invoices(), noon(), Duration::ZERO).unwrap_err(),
            GrantError::NoTime
        );
        assert_eq!(
            Grant::checked("@files", invoices(), noon(), Duration::MAX).unwrap_err(),
            GrantError::NoEnd
        );
    }

    /// A path that could lead somewhere else is refused when the grant is
    /// made, not argued about when it is used.
    #[test]
    fn a_grant_is_refused_before_it_can_lead_elsewhere() {
        assert_eq!(
            Grant::checked(
                "@files",
                Reach::Folder("/home/anna/../root".into()),
                noon(),
                hour()
            )
            .unwrap_err(),
            GrantError::CouldLeadElsewhere
        );
        assert_eq!(
            Grant::checked("@files", Reach::Folder("Invoices".into()), noon(), hour()).unwrap_err(),
            GrantError::NotAFullPath
        );
        assert_eq!(
            Grant::checked("@files", Reach::Folder("".into()), noon(), hour()).unwrap_err(),
            GrantError::NothingNamed
        );
    }

    /// ADR 0043: **there is no grant to an agent over a terminal**, for the
    /// reason there is none to `/` — and beside the refusal, the two grants that
    /// are still made: an agent's over a text editor, and an application's over
    /// the terminal a person chose to open something in.
    #[test]
    fn there_is_no_grant_to_an_agent_over_a_terminal() {
        for terminal in crate::A_PERSONS_OWN {
            let reach = Reach::Application(format!("  {terminal} "));
            assert_eq!(
                Grant::checked("@alo", reach.clone(), noon(), hour()).unwrap_err(),
                GrantError::APersonsOwn,
                "{terminal}"
            );
            let mail = crate::Applicant::named("org.example.Mail").grantee();
            assert!(Grant::checked_for(&mail, reach, noon(), hour()).is_ok());
        }
        assert!(
            Grant::checked(
                "@alo",
                Reach::Application("org.gnome.TextEditor".to_owned()),
                noon(),
                hour()
            )
            .is_ok()
        );
        assert!(
            GrantError::APersonsOwn
                .said(&in_english())
                .text()
                .contains("no agent can be granted one")
        );
    }

    #[test]
    fn a_grant_names_the_agent_it_is_for() {
        assert_eq!(
            Grant::checked("   ", invoices(), noon(), hour()).unwrap_err(),
            GrantError::Anonymous
        );
        let grant = granted();
        assert!(grant.is_for(&Grantee::named("@files")));
        assert!(!grant.is_for(&Grantee::named("@Files")));
        assert!(!grant.is_for(&Grantee::named("@mail")));
    }

    /// The grant a person made on Monday is the same grant on Tuesday,
    /// expiring when it was always going to expire.
    #[test]
    fn a_grant_survives_being_written_down_and_read_back() {
        let grant = granted();
        let written = serde_json::to_string(&grant).unwrap();
        let read: Grant = serde_json::from_str(&written).unwrap();
        assert_eq!(read, grant);
        assert!(!read.is_active_at(noon() + hour()));
    }

    /// **An application's grant is held to every rule an agent's is**, and
    /// may be over what an agent's may not.
    #[test]
    fn an_application_is_granted_under_the_same_rules() {
        let cheese = crate::Applicant::named("org.gnome.Cheese").grantee();
        let camera = Grant::checked_for(
            &cheese,
            Reach::Facility(crate::Facility::Camera),
            noon(),
            hour(),
        )
        .unwrap();
        assert!(camera.is_for(&cheese));
        assert!(!camera.is_for(&Grantee::named("org.gnome.Cheese")));
        assert!(camera.permits(&Ask::facility(crate::Facility::Camera), noon()));
        assert!(!camera.permits(&Ask::facility(crate::Facility::Camera), noon() + hour()));

        for (reach, lasting, refused) in [
            (
                Reach::Folder("/".into()),
                hour(),
                GrantError::TheWholeMachine,
            ),
            (invoices(), Duration::ZERO, GrantError::NoTime),
            (invoices(), Duration::MAX, GrantError::NoEnd),
            (
                Reach::Folder("/home/anna/../root".into()),
                hour(),
                GrantError::CouldLeadElsewhere,
            ),
        ] {
            assert_eq!(
                Grant::checked_for(&cheese, reach, noon(), lasting).unwrap_err(),
                refused
            );
        }
        assert_eq!(
            Grant::checked_for(
                &crate::Applicant::named("  ").grantee(),
                invoices(),
                noon(),
                hour()
            )
            .unwrap_err(),
            GrantError::NoApplicationNamed
        );
    }

    /// **An agent is never granted the camera, the screen or anything else
    /// that is not a path.** Context is offered, never watched.
    #[test]
    fn an_agent_is_never_granted_a_facility() {
        for facility in crate::Facility::EVERY {
            assert_eq!(
                Grant::checked("@files", Reach::Facility(facility), noon(), hour()).unwrap_err(),
                GrantError::NotForAnAgent,
                "{facility:?}"
            );
        }
        let said = GrantError::NotForAnAgent.said(&in_english());
        assert!(said.text().contains("only to applications"), "{said}");
        assert!(
            !GrantError::NoApplicationNamed
                .said(&in_english())
                .text()
                .contains("agent")
        );
    }

    /// The errors say what to do, not what went wrong.
    #[test]
    fn the_errors_say_what_to_do() {
        let strings = in_english();
        assert!(
            GrantError::Anonymous
                .said(&strings)
                .text()
                .contains("say which agent")
        );
        assert!(
            GrantError::NoTime
                .said(&strings)
                .text()
                .contains("how long")
        );
        assert!(
            GrantError::NotAFullPath
                .said(&strings)
                .text()
                .contains("full path")
        );
    }

    /// And they say it in the language the person reads, which is what this
    /// type losing its `Display` is for: there is no way to a sentence that
    /// does not go past the strings.
    #[test]
    fn a_grant_refusal_is_read_in_the_readers_own_language() {
        let strings = translated(&[(
            crate::words::THE_WHOLE_MACHINE,
            "es gibt keine Berechtigung für den ganzen Rechner — wählen Sie den Ordner, den Sie \
             wirklich meinen",
        )]);
        let said = GrantError::TheWholeMachine.said(&strings);
        assert!(said.is_translated());
        assert!(said.text().contains("den ganzen Rechner"), "{said}");
    }
}
