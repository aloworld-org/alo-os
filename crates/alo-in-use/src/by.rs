//! Who is using the screen, the camera or the microphone.
//!
//! Four answers, and the fourth is the one that matters most. Three of them are
//! the plan's — *an application by its name, an agent's turn, or alo OS
//! itself* — and the fourth is **something this machine can see but cannot
//! name**, because the alternative to that answer is not a better answer. It is
//! a line that is not shown, and a use that is not shown is the whole failure
//! this indicator exists to prevent.
//!
//! # An application can never wear the agent's answer
//!
//! [`By`] is a struct around a private enum, the shape `alo_capability::Grantee`
//! uses and for the same reason: there is no way to build [`By::the_agent`] out
//! of a grantee that is an application, because the constructor reads
//! `Grantee::is_an_application` and refuses. This is not tidiness. A line that
//! says *the agent* is drawn in terracotta (ADR 0010), terracotta means the
//! machine is acting on the person's behalf and means nothing else anywhere in
//! the system, and an application able to borrow it would have taken the one
//! signal that says so.
//!
//! # What the media server can and cannot vouch for
//!
//! [`crate::heard`] is where a record becomes one of these, and it carries the
//! whole of that argument: a sandboxed application's identity is the server's
//! to state and the application cannot claim to be anything else, while a name
//! an unsandboxed process wrote about itself is only ever read as *which
//! application*.

use alo_applications::Application;
use alo_capability::Grantee;
use alo_strings::{Filling, Strings, Word};

use crate::words;

/// Who is using something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct By(Who);

/// The four answers, private so that nothing builds one past its constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Who {
    /// An application, by the identifier this machine knows it by and the name
    /// a person recognises.
    Application(Application),
    /// An agent, by the name the machine knows it by. Never an application —
    /// [`By::the_agent`] is what makes that true.
    Agent(Grantee),
    /// alo OS itself, with no agent and no application behind it.
    AloOs,
    /// Something the machine can see and cannot name.
    Something,
}

impl By {
    /// An application, by the identifier and the name this machine has for it.
    #[must_use]
    pub const fn an_application(application: Application) -> Self {
        Self(Who::Application(application))
    }

    /// An agent, by the name the machine knows it by.
    ///
    /// Answers [`None`] for a grantee that is an application. That is the one
    /// guarantee this type makes: the terracotta line and the words *the agent*
    /// are reachable only from a grantee that really is one, so an application
    /// cannot arrive wearing them (ADR 0010).
    #[must_use]
    pub fn the_agent(agent: &Grantee) -> Option<Self> {
        if agent.is_an_application() {
            return None;
        }
        Some(Self(Who::Agent(agent.clone())))
    }

    /// alo OS itself.
    ///
    /// What the machine does on its own goes on this indicator like anybody
    /// else's use, which is `docs/features.md`'s promise in full: *by any
    /// application, including ours*.
    #[must_use]
    pub const fn alo_os_itself() -> Self {
        Self(Who::AloOs)
    }

    /// Something this machine can see and cannot name.
    ///
    /// Not a failure and not an absence: the screen, the camera or the
    /// microphone really is in use, and this is the honest name for whatever is
    /// using it. A use that could not be attributed and was therefore hidden
    /// would be the one a person most needed to be told about.
    #[must_use]
    pub const fn something_on_this_machine() -> Self {
        Self(Who::Something)
    }

    /// The application this is, or [`None`].
    #[must_use]
    pub const fn application(&self) -> Option<&Application> {
        match &self.0 {
            Who::Application(application) => Some(application),
            Who::Agent(_) | Who::AloOs | Who::Something => None,
        }
    }

    /// The agent this is, or [`None`].
    #[must_use]
    pub const fn agent(&self) -> Option<&Grantee> {
        match &self.0 {
            Who::Agent(agent) => Some(agent),
            Who::Application(_) | Who::AloOs | Who::Something => None,
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

    /// Whether the machine could not say what this is.
    #[must_use]
    pub const fn is_something_it_cannot_name(&self) -> bool {
        matches!(self.0, Who::Something)
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
            Who::Something => Some(words::SOMETHING_ON_THIS_MACHINE),
        }
    }

    /// The clause a line puts where it says by whom, in the language the person
    /// reads.
    ///
    /// A `String` rather than a `Said`, because what comes out is a **fragment
    /// placed inside something** — `alo_applications::Application::shown` and
    /// `alo_capability::Reach::shown` are the same shape for the same reason.
    /// What a person actually reads is [`crate::Line::said`], which is the
    /// whole sentence with this inside it.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> String {
        self.filling(strings)
            .value(words::WHO)
            .unwrap_or_default()
            .to_owned()
    }

    /// What a line fills its gap with, and where those words came from.
    ///
    /// One place rather than two, because a whole line is only as translated as
    /// its least translated piece and a bare `String` has forgotten which piece
    /// that was. An application's name is not translated and never was, so it
    /// goes in as itself.
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
            Who::Something => Filling::nothing().and_said(
                words::WHO,
                &strings.say(&words::SOMETHING_ON_THIS_MACHINE.key(), &Filling::nothing()),
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
    use crate::testing::{a_video_call, in_english, the_agent, translated};

    /// **An application cannot be an agent.** The one guarantee this type
    /// makes, and the one that keeps terracotta meaning what ADR 0010 says it
    /// means: a grantee that is an application is refused here, so no line can
    /// be built that draws an application in the agent's colour with the
    /// agent's word beside it.
    #[test]
    fn an_application_can_never_be_the_agent() {
        let application = alo_capability::Applicant::named("org.gnome.Cheese").grantee();
        assert!(application.is_an_application(), "the fixture is wrong");
        assert_eq!(By::the_agent(&application), None);

        let agent = the_agent();
        assert!(!agent.is_an_application());
        let by = By::the_agent(&agent).unwrap();
        assert!(by.is_the_agents());
        assert_eq!(by.agent(), Some(&agent));
    }

    /// **Exactly one of the four is the agent's**, so the colour and the word a
    /// line carries have one answer rather than an overlapping set of them.
    #[test]
    fn exactly_one_answer_is_the_agents() {
        let every = [
            By::an_application(a_video_call()),
            By::the_agent(&the_agent()).unwrap(),
            By::alo_os_itself(),
            By::something_on_this_machine(),
        ];
        assert_eq!(
            every.iter().filter(|by| by.is_the_agents()).count(),
            1,
            "the agent's answer is not exactly one of the four"
        );
        assert_eq!(every.iter().filter(|by| by.is_alo_os()).count(), 1);
        assert_eq!(
            every
                .iter()
                .filter(|by| by.is_something_it_cannot_name())
                .count(),
            1
        );
    }

    /// **Each answer reads as itself, and no two read the same.** A person told
    /// the same clause for an application and for the agent would have been
    /// told nothing.
    #[test]
    fn every_answer_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let every = [
            By::an_application(a_video_call()),
            By::the_agent(&the_agent()).unwrap(),
            By::alo_os_itself(),
            By::something_on_this_machine(),
        ];
        let mut seen: Vec<String> = Vec::new();
        for by in &every {
            let shown = by.shown(&strings);
            assert!(!shown.is_empty(), "{by:?} says nothing");
            assert!(!seen.contains(&shown), "two answers both read {shown}");
            seen.push(shown);
        }

        assert_eq!(
            By::an_application(a_video_call()).shown(&strings),
            "Video Call (com.example.VideoCall)"
        );
        assert_eq!(
            By::the_agent(&the_agent()).unwrap().shown(&strings),
            "@alo, the agent on this machine"
        );
        assert_eq!(By::alo_os_itself().shown(&strings), "alo OS itself");
        assert_eq!(
            By::something_on_this_machine().shown(&strings),
            "something on this machine"
        );
    }

    /// **The agent's clause says *the agent* whatever the agent is called.**
    /// ADR 0010's word, which is what a person reads when they cannot tell one
    /// colour from another — so it cannot come from the name.
    #[test]
    fn the_agents_clause_says_the_agent_whatever_it_is_called() {
        let strings = in_english();
        for named in ["@alo", "@files", "@mail"] {
            let by = By::the_agent(&Grantee::named(named)).unwrap();
            let shown = by.shown(&strings);
            assert!(shown.starts_with(named), "{shown}");
            assert!(shown.contains("the agent"), "{shown}");
        }
    }

    /// **Three of the four have a declared string and one has a name.** An
    /// application's clause is its own name, which nobody translates; the other
    /// three are alo OS's own words and are answered in the person's language.
    #[test]
    fn three_answers_are_declared_and_one_is_a_name() {
        assert_eq!(By::an_application(a_video_call()).word(), None);
        assert_eq!(
            By::the_agent(&the_agent()).unwrap().word(),
            Some(words::THE_AGENT)
        );
        assert_eq!(By::alo_os_itself().word(), Some(words::ALO_OS_ITSELF));
        assert_eq!(
            By::something_on_this_machine().word(),
            Some(words::SOMETHING_ON_THIS_MACHINE)
        );
    }

    /// **The clause is read in the language the person reads**, and says so.
    #[test]
    fn a_clause_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(words::ALO_OS_ITSELF, "alo OS selbst")]);
        assert_eq!(By::alo_os_itself().shown(&strings), "alo OS selbst");

        // The one nobody translated is still English, and is not a bug.
        let said = strings.say(&words::SOMETHING_ON_THIS_MACHINE.key(), &Filling::nothing());
        assert!(!said.is_translated());
        assert!(!said.is_a_bug());
    }
}
