//! What the overlay shows when the agent has nothing to say yet.
//!
//! The whole of it, in one value: the state the machine stands in, and the
//! three readings under it. `docs/autonomy/v0-01-delivery-plan.md`, task 3 —
//! *the three things `alo-agentd` already answers — what is granted, what
//! model would answer, and whether anything left the machine — are values this
//! repository has and no screen has ever shown.*
//!
//! # Four lines, in this order, and the order is content
//!
//! [`AtRest::lines`] answers the state first and the readings after it,
//! because that is the order the sentences are written for: the state is what
//! a person needs and the readings are what they check. It is reading order
//! and not layout — nothing here says where a line goes, how big it is or what
//! it looks like, which stays the compositor's exactly as it was for the
//! summoning seam.
//!
//! # Derived, and derived at the moment the key is pressed
//!
//! [`AtRest::of_this_machine`] takes the person's settings, the machine's
//! grants and the machine's indicator and reads all three. It is the only
//! constructor that goes to the machine, and it does so **when the overlay
//! opens** rather than continuously: `CLAUDE.md` says context is offered and
//! never watched, and a background reader keeping this value fresh would be a
//! bug in this product rather than a feature request. Nothing here opens a
//! socket or probes a runtime either — `crate::answering` says why the state
//! is what the machine is *set to* rather than what it would manage today.
//!
//! # It is not an invocation
//!
//! Opening the overlay offers the agent nothing: no focused window, no
//! selection, no open document. What this value holds is what the machine
//! says about **itself** — what it would answer with, what it may reach, what
//! is leaving it — and none of that is anybody's content. `alo-context` is
//! the law for the other kind, and it is untouched by this file.

use std::time::SystemTime;

use alo_capability::{Grantee, Grants};
use alo_choosing::Settings;
use alo_egress::Indicator;
use alo_strings::{Said, Strings};

use crate::answering::WouldAnswer;
use crate::granted::Granted;
use crate::quiet::Quiet;
use crate::standing::Standing;

/// Everything the overlay shows before a question has been asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtRest {
    /// What would answer, if a question were asked now.
    answering: WouldAnswer,
    /// How much the agent may reach at this moment.
    granted: Granted,
    /// What is leaving this machine at this moment.
    quiet: Quiet,
}

impl AtRest {
    /// The three readings, composed.
    ///
    /// For a shell that already holds them — one told what the machine holds
    /// over `alo-protocol`, rather than one sharing the daemon's memory. The
    /// state is not a parameter and cannot be: [`AtRest::standing`] derives
    /// it, so a shell cannot show *ready* over a machine that has chosen
    /// nothing.
    #[must_use]
    pub const fn of(answering: WouldAnswer, granted: Granted, quiet: Quiet) -> Self {
        Self {
            answering,
            granted,
            quiet,
        }
    }

    /// The same, read from what this machine holds.
    ///
    /// `agent` is the grantee the machine's description names — `alo-agentd`'s
    /// `Described::agent` — because a grant is for one agent and a count that
    /// added them all together would treat two agents as interchangeable.
    /// `now` is passed in rather than read from the clock, so what the overlay
    /// shows and what a record says cannot disagree about when it was true.
    #[must_use]
    pub fn of_this_machine(
        agent: &Grantee,
        grants: &Grants,
        settings: &Settings,
        indicator: &Indicator,
        now: SystemTime,
    ) -> Self {
        Self::of(
            WouldAnswer::of(settings),
            Granted::of(agent, grants, now),
            Quiet::of(indicator),
        )
    }

    /// Where the machine stands: one of the three the plan names.
    #[must_use]
    pub const fn standing(&self) -> Standing {
        Standing::of(&self.answering, self.granted)
    }

    /// What would answer, if a question were asked now.
    #[must_use]
    pub const fn answering(&self) -> &WouldAnswer {
        &self.answering
    }

    /// How much the agent may reach at this moment.
    #[must_use]
    pub const fn granted(&self) -> Granted {
        self.granted
    }

    /// What is leaving this machine at this moment.
    #[must_use]
    pub const fn quiet(&self) -> Quiet {
        self.quiet
    }

    /// The sentence a person reads first, in the language they read it in.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        self.standing().said(strings)
    }

    /// Every line the overlay shows at rest, in the order they are read.
    ///
    /// The state, and then what answers, what is granted and what is leaving.
    /// A fixed four rather than a list, because there are four: a machine
    /// that had nothing to say about one of them would be a machine hiding
    /// something, and the empty case of each is a sentence of its own.
    #[must_use]
    pub fn lines(&self, strings: &Strings) -> [Said; 4] {
        [
            self.said(strings),
            self.answering.said(strings),
            self.granted.said(strings),
            self.quiet.said(strings),
        ]
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use alo_capability::{Grant, Reach};
    use alo_choosing::{Chosen, Picked, Setup, Which};
    use alo_egress::{Destination, EgressPolicy, Leaving, Why};
    use alo_models::{Brought, InferenceSource, Providers};
    use std::path::PathBuf;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn the_agent() -> Grantee {
        Grantee::named("@files")
    }

    /// A person who chose a model that runs on their own machine.
    fn chose_locally() -> Settings {
        Settings::of(
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )),
            Brought::default(),
            Providers::default(),
            Vec::new(),
            Setup::NotAnswered,
        )
        .unwrap()
    }

    /// A machine where this agent has been granted one folder.
    fn one_folder() -> Grants {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from("/home/anna/Invoices")),
                noon(),
                Duration::from_secs(60 * 60),
            )
            .unwrap(),
        );
        grants
    }

    /// **A machine nobody has set up**: the state says what to do, and the
    /// three readings underneath are each a sentence rather than a blank.
    #[test]
    fn a_machine_nobody_has_set_up_reads_whole() {
        let at_rest = AtRest::of_this_machine(
            &the_agent(),
            &Grants::default(),
            &Settings::untouched(),
            &Indicator::default(),
            noon(),
        );
        assert_eq!(at_rest.standing(), Standing::NothingChosen);
        assert_eq!(at_rest.answering(), &WouldAnswer::Nothing);
        assert_eq!(at_rest.granted(), Granted::Nothing);
        assert_eq!(at_rest.quiet(), Quiet::NothingIsLeaving);

        let strings = in_english();
        let lines = at_rest.lines(&strings);
        for line in &lines {
            assert!(!line.is_a_bug(), "{line}");
            assert!(!line.text().trim().is_empty(), "a line was blank");
        }
        assert_eq!(
            lines.map(|line| line.into_text()),
            [
                "Nothing has been chosen to answer questions yet. Choose a model to run on this \
                 machine, or add a provider to send questions to, and the agent can start"
                    .to_owned(),
                "no model or provider has been chosen".to_owned(),
                "nothing is granted".to_owned(),
                "nothing is leaving this machine".to_owned(),
            ]
        );
    }

    /// **A machine that can answer and can reach nothing** is the second
    /// state, and its readings say which half is missing — the person is not
    /// left to work out which of the two sentences applies to them.
    #[test]
    fn a_machine_that_can_answer_and_reach_nothing_says_which_half_is_missing() {
        let at_rest = AtRest::of_this_machine(
            &the_agent(),
            &Grants::default(),
            &chose_locally(),
            &Indicator::default(),
            noon(),
        );
        assert_eq!(at_rest.standing(), Standing::NothingGranted);

        let lines = at_rest.lines(&in_english()).map(Said::into_text);
        assert!(lines[0].contains("Grant it a folder"), "{lines:?}");
        assert_eq!(lines[1], "mistral-small answers, on this machine");
        assert_eq!(lines[2], "nothing is granted");
    }

    /// **A machine that is ready**: the invitation, and three readings that
    /// each say something true about the machine underneath it.
    #[test]
    fn a_machine_that_is_ready_invites_a_question() {
        let at_rest = AtRest::of_this_machine(
            &the_agent(),
            &one_folder(),
            &chose_locally(),
            &Indicator::default(),
            noon(),
        );
        assert_eq!(at_rest.standing(), Standing::Ready);

        let lines = at_rest.lines(&in_english()).map(Said::into_text);
        assert_eq!(lines[0], "Ask the agent anything");
        assert_eq!(lines[1], "mistral-small answers, on this machine");
        assert_eq!(lines[2], "one thing is granted");
        assert_eq!(lines[3], "nothing is leaving this machine");
    }

    /// **A ready machine with something leaving it says so on the fourth
    /// line**, and being ready does not quieten it: the two readings are
    /// about different things and neither hides the other.
    #[test]
    fn a_ready_machine_still_says_what_is_leaving_it() {
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(
                &EgressPolicy::Anywhere,
                Leaving::because(
                    &the_agent(),
                    Why::Fetching,
                    Destination::at("alo.example").unwrap(),
                ),
                noon(),
            )
            .unwrap();
        let at_rest = AtRest::of_this_machine(
            &the_agent(),
            &one_folder(),
            &chose_locally(),
            &indicator,
            noon(),
        );
        assert_eq!(at_rest.standing(), Standing::Ready);
        assert_eq!(
            at_rest.lines(&in_english())[3].text(),
            "one thing is leaving this machine right now"
        );
        assert!(indicator.ended(departing));
    }

    /// **The state cannot be set from outside.** There is no constructor that
    /// takes a [`Standing`], so a shell holding readings it was told about
    /// still cannot show *ready* over a machine that has chosen nothing —
    /// which is the one line of this whole value that would be worth lying
    /// with.
    #[test]
    fn a_composed_value_still_derives_its_own_state() {
        let claimed_ready = AtRest::of(
            WouldAnswer::Nothing,
            Granted::of_how_many(9),
            Quiet::of_how_many(1),
        );
        assert_eq!(claimed_ready.standing(), Standing::NothingChosen);
        assert_eq!(
            claimed_ready.said(&in_english()).text(),
            Standing::NothingChosen.said(&in_english()).text()
        );
    }

    /// **What was composed is what is shown.** A shell told about the machine
    /// over a socket reads the same four lines as one that read the machine
    /// itself, which is what makes the two constructors one value.
    #[test]
    fn composing_the_readings_reads_the_same_as_reading_the_machine() {
        let read = AtRest::of_this_machine(
            &the_agent(),
            &one_folder(),
            &chose_locally(),
            &Indicator::default(),
            noon(),
        );
        let told = AtRest::of(
            WouldAnswer::Something {
                source: InferenceSource::ThisMachine,
                model: "mistral-small".to_owned(),
            },
            Granted::of_how_many(1),
            Quiet::of_how_many(0),
        );
        assert_eq!(read, told);
        let strings = in_english();
        assert_eq!(
            read.lines(&strings).map(Said::into_text),
            told.lines(&strings).map(Said::into_text)
        );
    }
}
