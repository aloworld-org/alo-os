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
//! - **it ends, unless the person said otherwise.** [`Grant::checked`] takes
//!   how long it lasts and refuses zero; [`Grant::checked_until_removed`] makes
//!   the one a person asked to keep until they take it away.
//!
//! **That last clause said the opposite until 2026-10-25**, and said it as a
//! principle: *there is no variant meaning "for ever" … the reliable way to
//! prevent it is to leave the type no way to express it*. The owner amended
//! ADR 0001 that day, and the reasoning is in the amendment rather than
//! repeated here. What matters at this file's level is that the protection did
//! not go away, it **moved**: a grant that never ends used to be prevented by a
//! clock and is now carried by the list a person can see — it says *until you
//! remove it* where another says a time, and one press revokes it. [`Until`] is
//! one field rather than a moment beside a flag so that the list and the daemon
//! cannot disagree about which kind a grant is.
//!
//! The rest of the paragraph above is unchanged and still enforced.
//!
//! Nothing here reads the clock: the moment a grant starts is passed in, and so
//! is the moment any later question is asked about it.

use std::time::{Duration, SystemTime};

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::a_terminal_for_the_agent::ATerminalForTheAgent;
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
    /// When it stops, or that it does not until the person removes it.
    ///
    /// **One field rather than a moment beside a flag**, which is the whole
    /// reason this is an enum: two fields could disagree about whether a grant
    /// ends, and the one that goes wrong is a grant the daemon honours for ever
    /// while the list tells a person it ran out at five.
    pub expires: Until,
}

/// When a grant stops.
///
/// **Amended into ADR 0001 on 2026-10-25.** It was a `SystemTime` and nothing
/// else, so every grant ran out and a person had to come back and grant the
/// same folder again — which the owner ruled against: *I can give an agent full
/// access or remove it, but there is no time limit*.
///
/// *Expiring by default* stands: this is what law 5 calls a default a person
/// may change. What §3 protected with a clock is now protected by the list — an
/// endless grant is in it, says so in words, and one press revokes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Until {
    /// It stops at this moment.
    ///
    /// Serialises as the number `docs/contracts/grants-file.md` has always
    /// described, so every grant written before this reads back unchanged.
    AMoment(SystemTime),
    /// It runs until the person takes it away.
    ///
    /// Serialises as the one word `removed`, which is why this is `untagged`:
    /// a reader meeting a number gets a moment and a reader meeting the word
    /// gets no end, and neither needs a version field.
    Removed(TheWordRemoved),
}

/// The one word an endless grant is written as, and nothing else accepted.
///
/// A type of its own rather than a bare string, so that nothing can put another
/// word in the file and have it read back as *for ever*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheWordRemoved;

impl Serialize for TheWordRemoved {
    fn serialize<S: serde::Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        serialiser.serialize_str("removed")
    }
}

impl<'de> Deserialize<'de> for TheWordRemoved {
    fn deserialize<D: serde::Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        let said = String::deserialize(reader)?;
        if said == "removed" {
            return Ok(Self);
        }
        Err(serde::de::Error::custom(format!(
            "a grant stops at a moment or at \"removed\", and this says {said:?}"
        )))
    }
}

impl Until {
    /// A grant that runs until the person takes it away.
    #[must_use]
    pub const fn removed() -> Self {
        Self::Removed(TheWordRemoved)
    }

    /// Whether a grant carrying this is still running at that moment.
    #[must_use]
    pub fn reaches(self, now: SystemTime) -> bool {
        match self {
            Self::AMoment(moment) => moment > now,
            Self::Removed(_) => true,
        }
    }

    /// How much is left, or [`None`] for one that has run out **and** for one
    /// that never will.
    ///
    /// The two are told apart by [`Self::is_until_removed`] rather than by
    /// this, because *no time left* and *no end* are opposite facts and a
    /// person reading one as the other is the mistake this type exists to
    /// prevent.
    #[must_use]
    pub fn left(self, now: SystemTime) -> Option<Duration> {
        match self {
            Self::AMoment(moment) => moment
                .duration_since(now)
                .ok()
                .filter(|left| !left.is_zero()),
            Self::Removed(_) => None,
        }
    }

    /// Whether this runs until the person takes it away.
    #[must_use]
    pub const fn is_until_removed(self) -> bool {
        matches!(self, Self::Removed(_))
    }
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
        // **The strict reading, for every caller that has not been told
        // otherwise.** `CLAUDE.md`: *a flag that fails open is a flag that
        // will be forgotten in the direction that matters* — so this road
        // means *the person has not chosen*, and no caller is loosened by
        // being out of date.
        Self::checked_for_when(
            grantee,
            reach,
            from,
            lasting,
            ATerminalForTheAgent::NotAtAll,
        )
    }

    /// The same, told what the person has allowed their agent to do with a
    /// terminal.
    ///
    /// **The one road, with [`Self::checked_for`] as its default.** Two
    /// functions rather than a changed signature because `Grant::checked_for`
    /// has 63 call sites across 39 files, nearly all of them tests with no
    /// opinion about terminals — and `CLAUDE.md` says a contract changes
    /// **additively**. What makes this one road rather than two answers is that
    /// the default is named: the shorter name means `NotAtAll` and says so.
    ///
    /// # Errors
    /// As [`Self::checked_for`], and [`GrantError::APersonsOwn`] for a terminal
    /// the person has not allowed.
    pub fn checked_for_when(
        grantee: &Grantee,
        reach: Reach,
        from: SystemTime,
        lasting: Duration,
        terminal: ATerminalForTheAgent,
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
            && !terminal.may_open_one()
        {
            return Err(GrantError::APersonsOwn);
        }
        if lasting.is_zero() {
            return Err(GrantError::NoTime);
        }
        let expires = Until::AMoment(from.checked_add(lasting).ok_or(GrantError::NoEnd)?);
        Ok(Self {
            grantee,
            reach,
            granted_at: from,
            expires,
        })
    }

    /// A grant that runs **until the person removes it**.
    ///
    /// The fourth door onto this one constructor, beside [`Self::checked`],
    /// [`Self::checked_for`] and [`Self::checked_for_when`] — a door rather
    /// than a fifth argument, because 219 call sites pass a `Duration` and a
    /// grant with no end is a different thing to say rather than a different
    /// number to say it with.
    ///
    /// **Everything else about it is checked the same way.** It goes through
    /// the same validation by handing it a duration it is certain to accept and
    /// then replacing the moment that came out — so a nameless grantee, a
    /// relative path, a `..`, the whole machine and a person's own terminal are
    /// all refused here exactly as they are for a grant of an hour. The only
    /// difference between the two is when they stop.
    ///
    /// ADR 0001, amended 2026-10-25, is why this exists at all.
    ///
    /// # Errors
    /// As [`Self::checked_for_when`], less [`GrantError::NoTime`] and
    /// [`GrantError::NoEnd`], which are about a length and there is none.
    pub fn checked_until_removed(
        grantee: &Grantee,
        reach: Reach,
        from: SystemTime,
        terminal: ATerminalForTheAgent,
    ) -> Result<Self, GrantError> {
        // **An hour, thrown away.** Any non-zero length that cannot overflow
        // does, and the moment it produced is replaced below; what is being
        // borrowed is every other check in there, so that this road cannot
        // drift from the one beside it.
        let timed =
            Self::checked_for_when(grantee, reach, from, Duration::from_secs(60 * 60), terminal)?;
        Ok(Self {
            expires: Until::removed(),
            ..timed
        })
    }

    /// Whether this grant is still running at this moment.
    ///
    /// A grant that expires at five o'clock does not include five o'clock. The
    /// boundary has to fall somewhere, and it falls on the side that reaches
    /// less.
    #[must_use]
    pub fn is_active_at(&self, now: SystemTime) -> bool {
        self.expires.reaches(now)
    }

    /// How much of the grant is left, or `None` once it has expired — **and
    /// also `None` for one that runs until the person removes it.**
    ///
    /// Those two are opposite facts and this does not tell them apart, which
    /// `Until::left` says and [`Self::lasts_until_removed`] is for. A surface
    /// that showed *no time left* for an endless grant would tell a person the
    /// opposite of what their machine is doing.
    #[must_use]
    pub fn expires_in(&self, now: SystemTime) -> Option<Duration> {
        self.expires.left(now)
    }

    /// Whether this grant runs until the person takes it away.
    #[must_use]
    pub const fn lasts_until_removed(&self) -> bool {
        self.expires.is_until_removed()
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

    /// The same grant, given until the person takes it away.
    fn until_removed() -> Grant {
        Grant::checked_until_removed(
            &Grantee::named("@files"),
            invoices(),
            noon(),
            ATerminalForTheAgent::NotAtAll,
        )
        .unwrap()
    }

    /// **ADR 0001 as the owner amended it on 2026-10-25.** A grant given until
    /// it is removed is still running a century later, where an hour's grant
    /// stopped at one o'clock.
    #[test]
    fn a_grant_until_removed_outlives_any_clock() {
        let a_century = noon() + Duration::from_secs(100 * 365 * 24 * 60 * 60);
        assert!(until_removed().is_active_at(a_century));
        assert!(!granted().is_active_at(noon() + hour()));
    }

    /// **No time left and no end are opposite facts**, and nothing may read one
    /// as the other. Both answer `None` for how much is left, so the question
    /// that tells them apart is a different one.
    #[test]
    fn running_out_and_never_ending_are_never_confused() {
        let after = noon() + hour();
        assert_eq!(granted().expires_in(after), None);
        assert_eq!(until_removed().expires_in(after), None);

        assert!(until_removed().lasts_until_removed());
        assert!(!granted().lasts_until_removed());
    }

    /// Everything else about a grant is checked the same way, which is what
    /// borrowing the other road's validation buys.
    #[test]
    fn a_grant_until_removed_is_refused_for_every_reason_a_timed_one_is() {
        for (reach, why) in [
            (
                Reach::Folder(PathBuf::from("/")),
                GrantError::TheWholeMachine,
            ),
            (
                Reach::Folder(PathBuf::from("Invoices")),
                GrantError::NotAFullPath,
            ),
            (
                Reach::Folder(PathBuf::from("/home/anna/../etc")),
                GrantError::CouldLeadElsewhere,
            ),
        ] {
            let refused = Grant::checked_until_removed(
                &Grantee::named("@files"),
                reach,
                noon(),
                ATerminalForTheAgent::NotAtAll,
            );
            assert_eq!(refused, Err(why));
        }
        assert_eq!(
            Grant::checked_until_removed(
                &Grantee::named(""),
                invoices(),
                noon(),
                ATerminalForTheAgent::NotAtAll,
            ),
            Err(GrantError::Anonymous)
        );
    }

    /// **It reads back as what it was written as**, and the one word is the
    /// only word accepted — so nothing in a hand-edited file becomes *for ever*
    /// by being spelt almost right.
    #[test]
    fn an_endless_grant_survives_the_disk_and_nothing_else_does() {
        let written = serde_json::to_string(&until_removed()).unwrap();
        assert!(written.contains("removed"), "{written}");
        assert_eq!(
            serde_json::from_str::<Grant>(&written).unwrap(),
            until_removed()
        );

        let timed = serde_json::to_string(&granted()).unwrap();
        assert_eq!(serde_json::from_str::<Grant>(&timed).unwrap(), granted());

        for almost in ["Removed", "remove", "for ever", "never"] {
            let bad = written.replace("removed", almost);
            assert!(
                serde_json::from_str::<Grant>(&bad).is_err(),
                "{almost:?} was read as an endless grant"
            );
        }
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

    /// **A person may allow their agent to open a terminal**, which is the
    /// half ADR 0111 added and the owner ruled on: *agent can open terminal
    /// window application.*
    ///
    /// The grant is made for every terminal on the list, because a refusal
    /// that lifted for one and not another would be a list nobody could
    /// reason about.
    #[test]
    fn a_person_may_allow_their_agent_to_open_a_terminal() {
        for terminal in crate::A_PERSONS_OWN {
            let reach = Reach::Application((*terminal).to_owned());
            let agent = Grantee::named("@alo");
            let allowed = Grant::checked_for_when(
                &agent,
                reach.clone(),
                noon(),
                hour(),
                ATerminalForTheAgent::MayOpenOne,
            );
            assert!(
                allowed.is_ok(),
                "{terminal} was refused although the person allowed it: {:?}",
                allowed.err()
            );
            // **And the default still refuses the same grant**, which is what
            // makes the assertion above about the choice rather than about
            // the list.
            assert_eq!(
                Grant::checked_for(&agent, reach, noon(), hour()).unwrap_err(),
                GrantError::APersonsOwn,
                "{terminal} was permitted without the person choosing"
            );
        }
    }

    /// **Allowing a terminal does not allow the whole machine.**
    ///
    /// The refusal that must survive the loosening. ADR 0043's reasoning was
    /// *a terminal is the whole machine by another road*, and ADR 0111 answers
    /// that the road is **driving** a shell and not opening a window — so a
    /// grant over `/` stays refused however the terminal choice is set.
    #[test]
    fn allowing_a_terminal_still_does_not_allow_the_whole_machine() {
        for terminal in [
            ATerminalForTheAgent::NotAtAll,
            ATerminalForTheAgent::MayOpenOne,
        ] {
            let agent = Grantee::named("@alo");
            assert_eq!(
                Grant::checked_for_when(
                    &agent,
                    Reach::Folder("/".into()),
                    noon(),
                    hour(),
                    terminal,
                )
                .unwrap_err(),
                GrantError::TheWholeMachine,
                "a grant to the whole machine was permitted with {terminal:?}"
            );
        }
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

    /// **A terminal is refused to an agent until the person allows it** —
    /// ADR 0111, which supersedes ADR 0043's outright wall.
    ///
    /// This test asserted the wall and now asserts the **default**, which is
    /// the same refusal for the commonest case and a different claim about
    /// why: not *no agent can be granted one* but *nobody has allowed it on
    /// this machine*. `a_person_may_allow_their_agent_to_open_a_terminal`
    /// below is the other half, and neither means much without the other.
    ///
    /// Beside the refusal, the two grants that are still made: an agent's over
    /// a text editor, and an application's over the terminal a person chose to
    /// open something in.
    #[test]
    fn a_terminal_is_refused_to_an_agent_until_the_person_allows_it() {
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
        // **The refusal says nobody has allowed it, not that nobody may.**
        // The old wording — *whatever is typed into it runs, so no agent can
        // be granted one* — stated ADR 0043 as a rule of the product, and
        // ADR 0111 makes that false: it is a choice the person has not made.
        // A refusal that misstates a decision is worse than a terse one,
        // because a person reads it and believes it.
        let said = GrantError::APersonsOwn.said(&in_english());
        assert!(
            said.text().contains("have not allowed"),
            "the refusal says {:?}, which does not tell the person it is their choice",
            said.text()
        );
        assert!(
            !said.text().contains("no agent can be granted"),
            "the refusal still states ADR 0043, which ADR 0111 superseded"
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
