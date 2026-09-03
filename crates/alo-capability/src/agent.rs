//! Whether this machine has an agent at all, and everything it may reach.
//!
//! [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md):
//! setup's *"where should your AI run?"* has a fourth answer with the same
//! weight as the other three — **not at all** — and a person can give it at
//! setup or at any time afterwards. This is that answer as a value, and the two
//! things the ADR says must follow from it.
//!
//! # It is not a flag beside the grants
//!
//! The obvious shape is a `bool` next to [`Grants`], asked by whoever remembers
//! to ask. That is the shape this repository refuses everywhere else, and here
//! it fails twice. Something would have to remember to check it before every
//! question — the one that is asked in three places already
//! ([`crate::Authorised`]) — and the flag and the list could disagree, so
//! *the agent is off* and *there are four grants* would both be true of one
//! machine and nothing could say which was the truth.
//!
//! So the list is **inside** the choice. A machine that declined the agent does
//! not hold an empty [`Grants`]; it holds no `Grants` at all, and
//! [`Agent::grants`] and [`Agent::grants_mut`] answer `None`. There is nothing
//! to forget to ask, because there is nothing to ask: the only road to the
//! machine's grants runs through the choice, and on a declined machine it stops.
//!
//! That answers the question the queue item posed — a state, or the absence of
//! any grant and any grantee — with **both, made one value**. It is a state,
//! because something has to survive a restart for *turning it on later is a
//! setting, not a reinstall* to be true. And it is the absence of any grant,
//! because the state is what holds them.
//!
//! # The two things that must follow, and one that must not
//!
//! **Grants end at once.** [`Agent::declining`] replaces the list rather than
//! emptying it, so reach is gone on the next question — the immediacy
//! [`Grants::revoke`] already had, applied to the whole machine in one act.
//!
//! **And they do not come back.** [`Agent::accepting`] makes a machine with an
//! agent and nothing granted. A person who turned the agent off and changed
//! their mind in June is not handing it March's folders back; ADR 0009 says
//! grants *end*, and a suspension that restored itself would be a different and
//! weaker promise wearing the same sentence.
//!
//! **The record and the egress indicator stay.** Neither is an AI feature, and
//! somebody who declined an agent may want *more* than average to know what
//! left their machine. Nothing in this file touches either, and that is the
//! point: what a machine with no agent still writes down is what it did on its
//! own (`alo_record::Only::OnItsOwn`), which exists already and is not this
//! crate's to switch off.
//!
//! # There is no `Default`
//!
//! A machine has to be **told** which of the two it is. `Default` would be alo
//! OS answering setup's fourth question on the person's behalf, in the file
//! that exists because that question is theirs — and whichever answer it picked
//! would be the one nobody ever chose. See the `compile_fail` doctest on
//! [`Agent::present`].

use std::time::SystemTime;

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::grant::Grantee;
use crate::grants::{GrantId, Grants};
use crate::reach::Ask;
use crate::refusing::NotGranted;
use crate::words;

/// What this machine decided about having an agent, and what follows from it.
///
/// The durable value a machine keeps: the grants live inside it rather than
/// beside it, so the two can never be read as disagreeing about whether there
/// is an agent to hold them.
///
/// Deliberately not `PartialEq` — [`Grants`] is not, and two machines' lists
/// being "the same" is not a question anything here needs answered. Deliberately
/// not `Default`, which is the more important of the two: see the module notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    /// This machine has an agent, and these are the grants it holds.
    Present(Grants),
    /// The person declined (ADR 0009). There is no list, because there is
    /// nothing that could be on one.
    Declined,
}

impl Agent {
    /// A machine with an agent and nothing granted yet.
    ///
    /// What setup writes down when somebody picks one of ADR 0008's three
    /// places for a model to answer from.
    ///
    /// # Examples
    ///
    /// ```
    /// use alo_capability::Agent;
    ///
    /// let machine = Agent::present();
    /// assert!(machine.has_an_agent());
    /// assert!(machine.grants().is_some_and(alo_capability::Grants::is_empty));
    /// ```
    ///
    /// **There is no `Default`.** Which of the two a machine is, is the
    /// person's answer to setup's fourth question, and a default would be alo
    /// OS giving it for them:
    ///
    /// ```compile_fail
    /// use alo_capability::Agent;
    ///
    /// let machine: Agent = Default::default();
    /// ```
    #[must_use]
    pub fn present() -> Self {
        Self::Present(Grants::default())
    }

    /// A machine where the person said *not at all*.
    #[must_use]
    pub fn declined() -> Self {
        Self::Declined
    }

    /// Whether there is an agent on this machine.
    ///
    /// What the shell asks to know whether the hotkey does anything and whether
    /// Grants, Models and providers appear in Settings — ADR 0009 says those
    /// surfaces are **absent** rather than present and disabled, because a
    /// greyed-out panel is an advertisement.
    #[must_use]
    pub fn has_an_agent(&self) -> bool {
        matches!(self, Self::Present(_))
    }

    /// The grants this machine holds, and `None` where there is no agent.
    ///
    /// The only road to them. A declined machine has no list rather than an
    /// empty one, so *nothing is granted* is not a fact something has to keep
    /// true — it is the shape of the value.
    #[must_use]
    pub fn grants(&self) -> Option<&Grants> {
        match self {
            Self::Present(grants) => Some(grants),
            Self::Declined => None,
        }
    }

    /// The same, for granting something and for revoking one grant.
    ///
    /// **This is what makes *nothing can be granted while the agent is off*
    /// structural** rather than a rule somebody follows: a file chooser that
    /// wanted to add a grant needs a `&mut Grants`, and on a declined machine
    /// there is none to hand it.
    #[must_use]
    pub fn grants_mut(&mut self) -> Option<&mut Grants> {
        match self {
            Self::Present(grants) => Some(grants),
            Self::Declined => None,
        }
    }

    /// Turn the agent off, and say how many grants a person will see go.
    ///
    /// ADR 0009's *turning it off again removes the agent's reach at once —
    /// grants end*. The list is replaced rather than emptied, so there is no
    /// moment at which this machine has an agent holding nothing, and the next
    /// question asked of it is refused by [`NotGranted::NoAgent`].
    ///
    /// The count is of grants that were **active** at `now`, because that is
    /// the list the person has been looking at: expired ones go too, and
    /// reporting them would be telling somebody they lost something they had
    /// already lost. Nothing here reads the clock, as everywhere else in this
    /// crate.
    ///
    /// Turning off a machine that already has no agent takes nothing away and
    /// says so.
    pub fn declining(&mut self, now: SystemTime) -> usize {
        let ending = self
            .grants()
            .map_or(0, |grants| grants.active_at(now).count());
        *self = Self::Declined;
        ending
    }

    /// Turn the agent on, and say whether that changed anything.
    ///
    /// A machine that already has an agent keeps its grants — this is not a
    /// way to clear them, and one that quietly was would be a second door to
    /// [`Agent::declining`] with an innocent name.
    ///
    /// A machine that had declined comes back with **nothing granted**. ADR
    /// 0009 says turning it on later is a setting rather than a reinstall, and
    /// says grants end when it goes off; both are true at once only if what
    /// comes back is a machine with an agent, not a machine with March's
    /// folders in it.
    pub fn accepting(&mut self) -> bool {
        match self {
            Self::Present(_) => false,
            Self::Declined => {
                *self = Self::present();
                true
            }
        }
    }

    /// Which grant permits this agent to touch this thing, at this moment — or
    /// why none does.
    ///
    /// [`Grants::permitting`] with the fourth choice in front of it, and the
    /// reason it is here rather than left to each caller: a machine with no
    /// agent has to refuse *and say the right thing*, and the right thing is
    /// not the sentence about picking a folder.
    ///
    /// # Errors
    /// [`NotGranted`] — the grants' own refusal on a machine that has an agent,
    /// and [`NotGranted::NoAgent`] on one that does not.
    pub fn permitting(
        &self,
        grantee: &Grantee,
        ask: &Ask,
        now: SystemTime,
    ) -> Result<GrantId, NotGranted> {
        match self {
            Self::Present(grants) => grants.permitting(grantee, ask, now),
            Self::Declined => Err(NotGranted::NoAgent {
                agent: grantee.as_str().to_owned(),
                wanted: ask.clone(),
            }),
        }
    }

    /// Whether this agent may touch this thing, at this moment.
    #[must_use]
    pub fn permits(&self, grantee: &Grantee, ask: &Ask, now: SystemTime) -> bool {
        self.permitting(grantee, ask, now).is_ok()
    }

    /// The string this crate declares for what this machine is.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::Present(_) => words::HAS_AN_AGENT,
            Self::Declined => words::HAS_NO_AGENT,
        }
    }

    /// What this machine is, in the language the person reads.
    ///
    /// The one line ADR 0009 leaves in Settings on a machine that declined —
    /// *Settings can offer it in one place without following them around* — and
    /// its twin on a machine that did not. Each says what the act would do,
    /// because whoever is reading one is deciding whether to perform it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::grant::Grant;
    use crate::reach::Reach;
    use crate::testing::{in_english, translated};
    use std::path::PathBuf;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn files() -> Grantee {
        Grantee::named("@files")
    }

    fn march() -> Ask {
        Ask::path("/home/anna/Invoices/march.pdf")
    }

    /// A machine with an agent, one folder granted, and one grant that has
    /// already run out — so the count `declining` answers with is a number that
    /// could be got wrong.
    fn a_working_machine() -> Agent {
        let mut machine = Agent::present();
        let grants = machine.grants_mut().unwrap();
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from("/home/anna/Invoices")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        grants.grant(
            Grant::checked(
                "@mail",
                Reach::Folder(PathBuf::from("/home/anna/Mail")),
                noon() - hour() * 2,
                hour(),
            )
            .unwrap(),
        );
        machine
    }

    /// **A machine with no agent permits nothing.** Not by holding an empty
    /// list, which something could add to, but by holding no list at all.
    #[test]
    fn a_machine_with_no_agent_permits_nothing() {
        let machine = Agent::declined();
        assert!(!machine.has_an_agent());
        assert!(machine.grants().is_none());
        for ask in [
            march(),
            Ask::path("/etc/shadow"),
            Ask::application("org.blender.Blender"),
        ] {
            assert!(!machine.permits(&files(), &ask, noon()), "{ask:?}");
        }
    }

    /// **Turning the agent off ends every grant at once**, which is ADR 0009's
    /// sharpest sentence: the immediacy a revoked grant has always had, applied
    /// to the whole machine in one act.
    #[test]
    fn turning_the_agent_off_ends_every_grant_at_once() {
        let mut machine = a_working_machine();
        assert!(machine.permits(&files(), &march(), noon()));

        // Two grants are on the list and one of them has expired, so what a
        // person is told is the one they can still see.
        assert_eq!(machine.grants().unwrap().len(), 2);
        assert_eq!(machine.declining(noon()), 1);

        assert!(!machine.has_an_agent());
        assert!(!machine.permits(&files(), &march(), noon()));
        assert!(machine.grants().is_none());
    }

    /// **And they do not come back.** A person who declined in March and
    /// changed their mind in June is not handed March's folders: grants *end*,
    /// and a suspension that restored itself would be a weaker promise wearing
    /// the same sentence.
    #[test]
    fn turning_it_back_on_does_not_bring_the_grants_back() {
        let mut machine = a_working_machine();
        machine.declining(noon());

        assert!(machine.accepting());
        assert!(machine.has_an_agent());
        assert!(machine.grants().unwrap().is_empty());
        assert!(!machine.permits(&files(), &march(), noon()));
    }

    /// Turning on a machine that is already on takes nothing away. A second
    /// door to `declining` with an innocent name is exactly the shape of
    /// mistake this test exists to catch.
    #[test]
    fn turning_it_on_when_it_is_already_on_takes_nothing_away() {
        let mut machine = a_working_machine();
        assert!(!machine.accepting());
        assert_eq!(machine.grants().unwrap().len(), 2);
        assert!(machine.permits(&files(), &march(), noon()));
    }

    /// And turning off a machine that has already declined is not an event.
    #[test]
    fn turning_it_off_twice_takes_nothing_away_the_second_time() {
        let mut machine = a_working_machine();
        assert_eq!(machine.declining(noon()), 1);
        assert_eq!(machine.declining(noon()), 0);
        assert!(!machine.has_an_agent());
    }

    /// **Nothing can be granted on a machine with no agent**, and it is the
    /// shape of the value that says so rather than a check: whatever wanted to
    /// add a grant needs a `&mut Grants`, and there is none to hand it.
    #[test]
    fn nothing_can_be_granted_on_a_machine_with_no_agent() {
        let mut machine = Agent::declined();
        assert!(machine.grants_mut().is_none());

        // The grant itself is still a perfectly good value — it is reaching the
        // machine's list with it that there is no road to.
        let grant = Grant::checked(
            "@files",
            Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            noon(),
            hour(),
        )
        .unwrap();
        assert!(grant.is_active_at(noon()));
        assert!(!machine.permits(&files(), &march(), noon()));
    }

    /// **The choice survives being written down and read back**, which is what
    /// *turning it on later is a setting, not a reinstall* needs to be true —
    /// and a declined machine reads back as declined rather than as one with an
    /// agent and an empty list.
    #[test]
    fn the_choice_survives_being_written_down_and_read_back() {
        let machine = a_working_machine();
        let written = serde_json::to_string(&machine).unwrap();
        let read: Agent = serde_json::from_str(&written).unwrap();
        assert!(read.permits(&files(), &march(), noon()));

        let mut machine = machine;
        machine.declining(noon());
        let written = serde_json::to_string(&machine).unwrap();
        assert_eq!(written, "\"declined\"");

        let read: Agent = serde_json::from_str(&written).unwrap();
        assert!(!read.has_an_agent());
        assert!(read.grants().is_none());
        assert!(!read.permits(&files(), &march(), noon()));
    }

    /// A machine with an agent refuses exactly as it did before: this file adds
    /// a fourth choice, not a fourth answer to the questions the other three
    /// ask.
    #[test]
    fn a_machine_with_an_agent_refuses_for_the_reasons_it_always_did() {
        let machine = a_working_machine();
        let permitting = machine.permitting(&files(), &Ask::path("/etc/shadow"), noon());
        assert!(matches!(permitting, Err(NotGranted::Never { .. })));

        let expired = machine.permitting(&files(), &march(), noon() + hour());
        assert!(matches!(expired, Err(NotGranted::Lapsed { .. })));

        assert_eq!(
            machine.permitting(&files(), &march(), noon()),
            machine
                .grants()
                .unwrap()
                .permitting(&files(), &march(), noon())
        );
    }

    /// **A refusal on a machine with no agent says so**, rather than sending
    /// somebody to a grants panel ADR 0009 makes absent on their machine.
    #[test]
    fn a_machine_with_no_agent_refuses_in_its_own_words() {
        let machine = Agent::declined();
        let why = machine.permitting(&files(), &march(), noon()).unwrap_err();
        assert!(matches!(why, NotGranted::NoAgent { .. }));
        assert_eq!(why.agent(), "@files");
        assert_eq!(why.wanted(), &march());

        let said = why.said(&in_english());
        assert!(said.text().contains("has no agent"), "{said}");
        assert!(!said.text().contains("picking a folder"), "{said}");
    }

    /// **What this machine is, is a sentence a person reads** — and each of the
    /// two says what the act would do, because whoever reads one is deciding
    /// whether to perform it.
    #[test]
    fn what_this_machine_is_is_said_in_the_language_the_person_reads() {
        let strings = in_english();
        let on = Agent::present().said(&strings);
        assert!(on.text().contains("has an agent"), "{on}");
        assert!(on.text().contains("ends at once"), "{on}");

        let off = Agent::declined().said(&strings);
        assert!(off.text().contains("has no agent"), "{off}");
        assert!(off.text().contains("reinstalled"), "{off}");
        assert_ne!(on.text(), off.text());
    }

    /// And both are translated like everything else here, with the marking that
    /// says when nobody has.
    #[test]
    fn what_this_machine_is_says_when_nobody_has_translated_it() {
        let german = translated(&[(
            words::HAS_NO_AGENT,
            "dieser Rechner hat keinen Agenten — schalten Sie jederzeit einen ein, es wird nichts \
             neu installiert",
        )]);
        let off = Agent::declined().said(&german);
        assert!(off.is_translated(), "{off}");
        assert!(off.text().contains("keinen Agenten"), "{off}");

        // The other one was not translated, and says so rather than looking the
        // same as the one that was.
        let on = Agent::present().said(&german);
        assert!(!on.is_translated(), "{on}");

        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        assert!(Agent::declined().said(&nothing).is_a_bug());
    }
}
