//! Who sent a notification: an application, the agent, or alo OS itself.
//!
//! Three answers, and the second one is the one this file exists for. A
//! notification drawn in terracotta with the word *agent* beside it says the
//! machine is speaking on the person's behalf, and
//! [ADR 0010](../../../docs/decisions/0010-terracotta-is-reserved-and-never-alone.md)
//! reserves that signal for exactly that. So [`Sender`] is a struct around a
//! private enum — the shape `alo_capability::Grantee` and `alo_in_use::By` use,
//! for the same reason — and [`Sender::the_agent`] reads
//! `Grantee::is_an_application` and refuses. **An application cannot arrive
//! wearing the agent's colour and word**, because there is no constructor that
//! would let it.
//!
//! # And there is no fourth answer
//!
//! `alo_in_use::By` has one, *something on this machine*, because a use of the
//! camera that could not be attributed is still a use of the camera and hiding
//! it would be the failure that indicator exists to prevent. A notification is
//! the opposite case: it arrives through a portal, under a grant, and a grant
//! names its holder. Something that cannot say who it is has not asked for
//! anything a person granted, and what it gets is a refusal rather than a line
//! on a screen with a shrug in it.

use alo_applications::Application;
use alo_capability::Grantee;
use alo_strings::{Filling, Strings, Word};

use crate::words;

/// Who sent a notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sender(Who);

/// The three answers, private so that nothing builds one past its constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Who {
    /// An application, by the identifier this machine knows it by and the name
    /// a person recognises.
    Application(Application),
    /// An agent, by the name the machine knows it by. Never an application —
    /// [`Sender::the_agent`] is what makes that true.
    Agent(Grantee),
    /// alo OS itself, with no agent and no application behind it.
    AloOs,
}

impl Sender {
    /// An application, by the identifier this machine knows it by.
    ///
    /// **Not public.** A sender that is an application exists only where a
    /// grant was judged: `crate::arriving` builds one out of
    /// `alo_portals::Allowed` and nothing else does, so there is no road from
    /// an application's name to a notification that does not pass through the
    /// portal and the person's own grant.
    #[must_use]
    pub(crate) const fn an_application(application: Application) -> Self {
        Self(Who::Application(application))
    }

    /// An agent, by the name the machine knows it by.
    ///
    /// Answers [`None`] for a grantee that is an application. That is the one
    /// guarantee this type makes, and it is ADR 0010's: the terracotta
    /// notification and the word *agent* are reachable only from a grantee
    /// that really is one.
    #[must_use]
    pub fn the_agent(agent: &Grantee) -> Option<Self> {
        if agent.is_an_application() {
            return None;
        }
        Some(Self(Who::Agent(agent.clone())))
    }

    /// alo OS itself — the machine telling the person something about their
    /// own machine.
    #[must_use]
    pub const fn alo_os_itself() -> Self {
        Self(Who::AloOs)
    }

    /// The application this is, or [`None`].
    #[must_use]
    pub const fn application(&self) -> Option<&Application> {
        match &self.0 {
            Who::Application(application) => Some(application),
            Who::Agent(_) | Who::AloOs => None,
        }
    }

    /// The agent this is, or [`None`].
    #[must_use]
    pub const fn agent(&self) -> Option<&Grantee> {
        match &self.0 {
            Who::Agent(agent) => Some(agent),
            Who::Application(_) | Who::AloOs => None,
        }
    }

    /// Whether this is an agent, which is the one answer that is terracotta.
    #[must_use]
    pub const fn is_the_agents(&self) -> bool {
        matches!(self.0, Who::Agent(_))
    }

    /// Whether this is alo OS itself.
    #[must_use]
    pub const fn is_alo_os(&self) -> bool {
        matches!(self.0, Who::AloOs)
    }

    /// The identifier a refusal names this sender by.
    ///
    /// The identifier a grant was made over for an application, the agent's own
    /// name for an agent, and the product's name for alo OS. Never translated,
    /// and never the name a packager wrote: `alo_applications::Application`
    /// says why at length.
    #[must_use]
    pub fn named(&self) -> String {
        match &self.0 {
            Who::Application(application) => application.identifier().to_owned(),
            Who::Agent(agent) => agent.as_str().to_owned(),
            Who::AloOs => "alo OS".to_owned(),
        }
    }

    /// The string this crate declares for it, where it has one.
    ///
    /// [`None`] for an application, whose clause is its own name and is not a
    /// string anybody translates.
    #[must_use]
    pub const fn word(&self) -> Option<Word> {
        match self.0 {
            Who::Application(_) => None,
            Who::Agent(_) => Some(words::THE_AGENT),
            Who::AloOs => Some(words::ALO_OS_ITSELF),
        }
    }

    /// The clause the sentence saying who a notification is from puts where it
    /// says by whom, in the language the person reads.
    ///
    /// A `String` rather than a `Said`, because what comes out is a **fragment
    /// placed inside something** — `alo_applications::Application::shown` and
    /// `alo_in_use::By::shown` are the same shape for the same reason.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> String {
        self.filling(strings)
            .value(words::WHO)
            .unwrap_or_default()
            .to_owned()
    }

    /// What the sentence fills its gap with, and where those words came from.
    ///
    /// One place rather than two, because a whole sentence is only as
    /// translated as its least translated piece and a bare `String` has
    /// forgotten which piece that was.
    pub(crate) fn filling(&self, strings: &Strings) -> Filling {
        match &self.0 {
            Who::Application(application) => Filling::of(words::WHO, application.shown(strings)),
            Who::Agent(agent) => Filling::nothing().and_said(
                words::WHO,
                &strings.say(
                    &words::THE_AGENT.key(),
                    &Filling::of(words::AGENT, agent.as_str()),
                ),
            ),
            Who::AloOs => Filling::nothing().and_said(
                words::WHO,
                &strings.say(&words::ALO_OS_ITSELF.key(), &Filling::nothing()),
            ),
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
    use crate::testing::{a_mail_client, in_english, the_agent};

    /// **An application can never be the agent.** The one guarantee this type
    /// makes: a grantee that is an application is refused here, so no
    /// notification can be built that is drawn in the agent's colour with the
    /// agent's word beside it (ADR 0010).
    #[test]
    fn an_application_can_never_be_the_agent() {
        let application = alo_capability::Applicant::named("org.example.Mail").grantee();
        assert!(application.is_an_application(), "the fixture is wrong");
        assert_eq!(Sender::the_agent(&application), None);

        let agent = the_agent();
        assert!(!agent.is_an_application());
        let sender = Sender::the_agent(&agent).unwrap();
        assert!(sender.is_the_agents());
        assert_eq!(sender.agent(), Some(&agent));
    }

    /// **Exactly one of the three is the agent's**, so the colour and the word
    /// a notification carries have one answer rather than an overlapping set.
    #[test]
    fn exactly_one_answer_is_the_agents() {
        let every = [
            Sender::an_application(a_mail_client()),
            Sender::the_agent(&the_agent()).unwrap(),
            Sender::alo_os_itself(),
        ];
        assert_eq!(every.iter().filter(|by| by.is_the_agents()).count(), 1);
        assert_eq!(every.iter().filter(|by| by.is_alo_os()).count(), 1);
        assert_eq!(
            every.iter().filter(|by| by.application().is_some()).count(),
            1
        );
    }

    /// **Each answer reads as itself, and no two read the same.**
    #[test]
    fn every_answer_reads_and_no_two_read_the_same() {
        let strings = in_english();
        assert_eq!(
            Sender::an_application(a_mail_client()).shown(&strings),
            "Mail (org.example.Mail)"
        );
        assert_eq!(
            Sender::the_agent(&the_agent()).unwrap().shown(&strings),
            "@alo, the agent on this machine"
        );
        assert_eq!(Sender::alo_os_itself().shown(&strings), "alo OS itself");
    }

    /// **The agent's clause says *the agent* whatever the agent is called.**
    #[test]
    fn the_agents_clause_says_the_agent_whatever_it_is_called() {
        let strings = in_english();
        for named in ["@alo", "@files", "@mail"] {
            let sender = Sender::the_agent(&Grantee::named(named)).unwrap();
            let shown = sender.shown(&strings);
            assert!(shown.starts_with(named), "{shown}");
            assert!(shown.contains("the agent"), "{shown}");
            assert_eq!(sender.named(), named);
        }
    }

    /// **Two of the three have a declared string and one has a name.**
    #[test]
    fn two_answers_are_declared_and_one_is_a_name() {
        assert_eq!(Sender::an_application(a_mail_client()).word(), None);
        assert_eq!(
            Sender::the_agent(&the_agent()).unwrap().word(),
            Some(words::THE_AGENT)
        );
        assert_eq!(Sender::alo_os_itself().word(), Some(words::ALO_OS_ITSELF));
    }
}
