//! **A line another process's indicator is showing, as it was told.**
//!
//! The desktop draws what is leaving this machine, and since 2026-10-27 that
//! includes what `alo-agentd` is sending on an agent's behalf: the service
//! that opens a connection to a provider is the one holding the indicator it
//! lit (ADR 0125), and it tells the desktop what that indicator shows. This is
//! the shape a line takes on that road.
//!
//! # It deserialises, and it is not a departure
//!
//! [`Leaving`](crate::Leaving) and [`OnItsOwn`](crate::OnItsOwn) deliberately
//! do not deserialise: one read back off a wire would be an egress nothing had
//! decided about. A `Reported` is the other thing — **a sentence about an
//! egress somebody else decided about** — so it reads back, and it has no road
//! to a [`Departing`](crate::Departing), an [`Underway`](crate::Underway) or a
//! line on an [`Indicator`](crate::Indicator). Nothing holding one may open a
//! connection because of it. It can only be said.
//!
//! **No moment crosses the wire**, as nothing else on the daemon's door does:
//! when a line began is the indicator's, and a clock on the other side would be
//! a second answer to one question.

use alo_capability::Grantee;
use alo_strings::{Said, Strings};
use serde::{Deserialize, Serialize};

use crate::destination::Destination;
use crate::at_work::{HowItIsShown, said_of_alo_at_work};
use crate::during::said_of_their_command;
use crate::errand::Errand;
use crate::indicator::Shown;
use crate::itself::said_of_an_errand;
use crate::leaving::{Why, said_of_an_agents};
use crate::showing::Showing;

/// One line of what another process's indicator is showing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Reported {
    /// An agent's egress.
    Agent {
        /// Whose authority it is under.
        agent: Grantee,
        /// Why it is leaving.
        why: Why,
        /// Where it is going.
        destination: Destination,
    },
    /// An errand alo OS is running itself.
    Itself {
        /// Which errand.
        errand: Errand,
        /// Where it is reaching.
        destination: Destination,
    },
    /// An application sending while a person's own command runs (ADR 0148).
    ///
    /// No destination, because none is read; [`crate::DuringTheirCommand`]
    /// says why.
    TheirCommand {
        /// The application, by the identifier its sandbox gives it.
        application: String,
        /// What a person calls it.
        called: String,
    },
    /// An application alo is working in (ADR 0143 Q6). No destination, for
    /// the same reason.
    AloAtWork {
        /// The application, by its identifier.
        application: String,
        /// What a person calls it.
        called: String,
        /// Which of the three lines.
        how: HowItIsShown,
    },
}

impl Reported {
    /// What one line of this machine's own indicator says, to be told to
    /// another process.
    #[must_use]
    pub fn of(shown: &Shown) -> Self {
        match shown.showing() {
            Showing::Agent(leaving) => Self::Agent {
                agent: leaving.agent().clone(),
                why: leaving.why(),
                destination: leaving.destination().clone(),
            },
            Showing::Itself(errand) => Self::Itself {
                errand: errand.errand(),
                destination: errand.destination().clone(),
            },
            Showing::TheirCommand(during) => Self::TheirCommand {
                application: during.application().to_owned(),
                called: during.called().to_owned(),
            },
            Showing::AloAtWork(at_work) => Self::AloAtWork {
                application: at_work.application().to_owned(),
                called: at_work.called().to_owned(),
                how: at_work.how(),
            },
        }
    }

    /// Where it is going, when that is known — never for an application during
    /// a person's command.
    #[must_use]
    pub fn destination(&self) -> Option<&Destination> {
        match self {
            Self::Agent { destination, .. } | Self::Itself { destination, .. } => Some(destination),
            Self::TheirCommand { .. } | Self::AloAtWork { .. } => None,
        }
    }

    /// The line a person reads: **the same sentence** the indicator that
    /// showed it would have said.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::Agent {
                agent,
                why,
                destination,
            } => said_of_an_agents(agent, *why, destination, strings),
            Self::Itself {
                errand,
                destination,
            } => said_of_an_errand(*errand, destination, strings),
            Self::TheirCommand { called, .. } => said_of_their_command(called, strings),
            Self::AloAtWork { called, how, .. } => said_of_alo_at_work(called, *how, strings),
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
    use crate::indicator::Indicator;
    use crate::itself::OnItsOwn;
    use crate::leaving::Leaving;
    use crate::policy::EgressPolicy;
    use crate::testing::in_english;
    use alo_models::{InferenceSource, Region};
    use std::time::{Duration, SystemTime};

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
    }

    /// **A reported line says exactly what the line it came from says**, for
    /// both kinds, so the desktop and the service cannot word one departure
    /// two ways.
    #[test]
    fn a_reported_line_says_what_the_line_it_came_from_says() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let asking = Leaving::asking(
            &Grantee::named("@mail"),
            &InferenceSource::Hosted {
                provider: "openai".to_owned(),
                region: Region::Declared("the US".to_owned()),
            },
        )
        .unwrap();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking, noon())
            .unwrap();
        let underway = indicator.beginning_on_its_own(
            OnItsOwn::for_(
                Errand::FetchingAModel,
                Destination::at("models.alo.example").unwrap(),
            ),
            noon(),
        );
        let while_it_runs = indicator.beginning_during_their_command(
            crate::DuringTheirCommand::of("org.mozilla.Thunderbird", "Thunderbird").unwrap(),
            noon(),
        );
        assert_eq!(indicator.showing().len(), 3);
        for shown in indicator.showing() {
            assert_eq!(
                Reported::of(shown).said(&strings).text(),
                shown.said(&strings).text()
            );
        }
        assert!(indicator.ended(departing));
        assert!(indicator.ended_on_its_own(underway));
        assert!(indicator.ended_during_their_command(while_it_runs));
        assert!(indicator.is_quiet());
    }

    /// **An application's line crosses the wire whole and names no place.**
    #[test]
    fn an_application_during_a_command_reads_back_with_no_destination() {
        let line = Reported::TheirCommand {
            application: "org.mozilla.Thunderbird".to_owned(),
            called: "Thunderbird".to_owned(),
        };
        let written = serde_json::to_string(&line).unwrap();
        assert_eq!(serde_json::from_str::<Reported>(&written).unwrap(), line);
        assert_eq!(line.destination(), None);
        assert_eq!(
            line.said(&in_english()).text(),
            "Thunderbird is sending, during your command"
        );
        let widened = written.replacen("\"called\"", "\"host\":\"x\",\"called\"", 1);
        assert!(
            serde_json::from_str::<Reported>(&widened).is_err(),
            "{widened}"
        );
    }

    /// **It survives the wire whole**, and an unknown field is refused rather
    /// than ignored, as everything else on the daemon's door is.
    #[test]
    fn a_reported_line_reads_back_and_refuses_what_it_does_not_know() {
        let line = Reported::Agent {
            agent: Grantee::named("@mail"),
            why: Why::Asking,
            destination: Destination::at("api.openai.com").unwrap(),
        };
        let written = serde_json::to_string(&line).unwrap();
        assert_eq!(serde_json::from_str::<Reported>(&written).unwrap(), line);

        let widened = written.replacen("\"why\"", "\"when\":1,\"why\"", 1);
        assert!(
            serde_json::from_str::<Reported>(&widened).is_err(),
            "{widened}"
        );
    }
}
