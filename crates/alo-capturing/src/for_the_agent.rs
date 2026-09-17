//! **The one road by which an agent may see the screen, and what it costs.**
//!
//! Task 6 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`. A screenshot is
//! the most harvest-shaped thing this machine has: one call and a program holds
//! everything a person was looking at — their mail open beside their bank, a
//! colleague's name, a photograph on a second screen. So the agent reaches it by
//! **one road, with one approval, and no other**.
//!
//! # The verb does not exist, and this file does not add it
//!
//! `docs/contracts/agent-verbs.md` lists no verb that captures the screen, and
//! the task says plainly that where it does not, **that is a finding and the
//! verb is not added here**. Adding one to reach this code would be the whole
//! failure this plan exists to prevent, arriving as a convenience.
//!
//! So [`ForTheAgent`] can be built **only** from an `alo_capability::Authorised`
//! — evidence that a verb's call was validated and a person approved it — and
//! today nothing in this repository can produce one for a capture, because no
//! such verb is declared. The type is written now so that the day somebody
//! declares that verb, the road it must come down already exists and already
//! refuses everything else.
//!
//! # What the agent gets, and for how long
//!
//! The picture goes to the turn. **It is not kept, not indexed, not written to
//! the record.** What the record says is that a picture of the screen was
//! taken — which is a thing that happened to the person's machine and belongs
//! in it — and never the picture, nor what was in it.
//!
//! [`ForTheAgent::to_the_turn`] takes the picture out and leaves nothing behind:
//! the value is consumed, so a caller cannot hold it and hand it over twice.
//!
//! # And the indicator says so while it happens
//!
//! [`crate::for_the_agent::ForTheAgent::in_use`] is the use `alo-in-use` shows
//! while the turn holds the picture: the screen, **by the agent**, which that
//! crate draws in terracotta with the agent's mark and word — ADR 0010 reserves
//! that colour for the agent and nothing else uses it.
//!
//! A capture the agent took that the indicator did not show is the failure this
//! whole workstream is built to prevent, so
//! `tests/the_agent_reaches_the_screen_by_one_road.rs` tests it directly rather
//! than trusting that the path is shared.

use alo_capability::Authorised;
use alo_in_use::{By, Use, UseId, Used};

use crate::picture::Picture;
use crate::screen::Screen;

/// **A capture an agent asked for, that a person approved.**
///
/// No public constructor but [`ForTheAgent::approved`], which takes the
/// authorisation a verb's door produces. There is no way to make one from a
/// path, a name, or an intention.
#[derive(Debug)]
pub struct ForTheAgent {
    /// Which screen it is of.
    screen: Screen,
    /// Which agent asked, as the grant names it.
    agent: String,
    /// The picture itself, until it is handed to the turn.
    picture: Picture,
}

impl ForTheAgent {
    /// **The only way to one**: an approved call of the screen verb.
    ///
    /// # Why there is no grant to check
    ///
    /// ADR 0040 keeps facilities — the camera, the screen, notifications — to
    /// **applications, never to agents**: *an agent is offered what it needs at
    /// the moment it is asked, and a durable grant to the camera would be a
    /// background reader by another name.* `alo-capability` refuses such a
    /// grant outright (`GrantError::NotForAnAgent`).
    ///
    /// So the agent holds nothing standing, and **the approval of the sentence
    /// is the whole of the authority** — which is why this checks that the
    /// authority came from an approval rather than from the read door.
    ///
    /// The `Authorised` is taken **by value and spent**. An approval is never a
    /// session (ADR 0001), and this is where that carries weight rather than
    /// tidiness: an agent that could take a second picture on this morning's
    /// approval is an agent watching the screen, which `docs/features.md`
    /// forbids in the same breath as it promises context on invocation. One
    /// approval is one picture, and there is no way to hold the authority and
    /// use it twice.
    ///
    /// # Errors
    /// [`NotForTheAgent::AnotherVerb`] for an authority from any other verb,
    /// and [`NotForTheAgent::NobodyApprovedIt`] for one that reached here
    /// without a person having approved the sentence.
    pub fn approved(
        approved: Authorised,
        screen: Screen,
        picture: Picture,
    ) -> Result<Self, NotForTheAgent> {
        if approved.verb() != crate::verbs::PICTURE_OF_THE_SCREEN {
            return Err(NotForTheAgent::AnotherVerb {
                verb: approved.verb().to_owned(),
            });
        }
        if approved.from_approval().is_none() {
            return Err(NotForTheAgent::NobodyApprovedIt);
        }
        Ok(Self {
            screen,
            agent: approved.under().as_str().to_owned(),
            picture,
        })
    }

    /// Which screen was captured.
    #[must_use]
    pub const fn screen(&self) -> Screen {
        self.screen
    }

    /// **What the indicator shows while the turn holds this.**
    ///
    /// The screen, in use by the agent. `alo-in-use` draws that in terracotta
    /// with the agent's mark, and nothing else in this system is terracotta
    /// (ADR 0010).
    ///
    /// [`None`] where the agent's name is not one a grant could name — which is
    /// `alo_in_use::By::the_agent`'s own refusal, not a judgement made here.
    #[must_use]
    pub fn in_use(&self, at: UseId) -> Option<Use> {
        let agent = alo_capability::Grantee::named(&self.agent);
        Some(Use::of(at, Used::Screen, By::the_agent(&agent)?))
    }

    /// **Hand the picture to the turn, and keep nothing.**
    ///
    /// Consumes the capture: there is no second copy to index, to write into
    /// the record, or to hand to anybody else.
    #[must_use]
    pub fn to_the_turn(self) -> Picture {
        self.picture
    }

    /// **What the record says about this**, and the whole of it.
    ///
    /// That a picture of the screen was taken, for which agent. Not the
    /// picture, not its size, not a thumbnail, not what was on the screen: a
    /// record that described the contents would be the harvest this road exists
    /// to prevent, written down by us.
    #[must_use]
    pub fn as_the_record_says_it(&self) -> String {
        format!("a picture of the screen was taken for {}", self.agent)
    }
}

/// Why a capture is not the agent's to have.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotForTheAgent {
    /// The authority came from some other verb. Nothing else opens this road:
    /// an approval for renaming a file is not an approval for a picture of
    /// everything on the screen.
    #[error("`{verb}` is not the screen verb, and no other verb produces a picture of the screen")]
    AnotherVerb {
        /// What the authority was for.
        verb: String,
    },
    /// The authority did not come from an approval.
    ///
    /// A read verb's authority is granted at the door and runs inside the turn;
    /// this verb is a change, and the sentence a person approved **is** the
    /// authority, because ADR 0040 gives an agent nothing standing over the
    /// screen to hold.
    #[error(
        "nobody approved a picture of the screen, and an agent holds nothing that stands in \
             for that"
    )]
    NobodyApprovedIt,
}

/// **A capture, as the road produces one, for a test that has no verb to call.**
///
/// A test cannot build an `Authorised`: that comes from a person approving a
/// proposal, through machinery no test here runs. So this makes the value the
/// honest road would make, and it is `#[doc(hidden)]` and takes no
/// authorisation — it is useless for reaching a screen, because it holds a
/// picture somebody already has.
#[doc(hidden)]
#[must_use]
pub fn for_a_test(agent: &str, screen: Screen, picture: Picture) -> ForTheAgent {
    ForTheAgent {
        screen,
        agent: agent.to_owned(),
        picture,
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_in_use::Line;

    /// A capture, as the road would produce one — built here from the parts
    /// rather than from an `Authorised`, because **no verb that could produce
    /// one exists** (see this module's header), and a test that invented one
    /// would be testing a door nothing opens.
    fn a_capture(bytes: Vec<u8>) -> ForTheAgent {
        for_a_test(
            "@the-agent",
            crate::testing::a_screen(),
            Picture::of(bytes).expect("a picture"),
        )
    }

    /// **The indicator shows the screen, in use by the agent, in terracotta.**
    #[test]
    fn while_the_turn_holds_it_the_indicator_says_the_agent_is_using_the_screen() {
        let capture = a_capture(vec![1, 2, 3]);
        let shown = capture
            .in_use(UseId::recorded(1))
            .expect("the agent is a grantee");
        let line = Line::of(&shown);

        assert_eq!(line.what(), Used::Screen);
        assert!(
            line.by().is_the_agents(),
            "the line does not say it is the agent"
        );
        assert_eq!(
            line.colour(),
            alo_appearance::Token::Terracotta,
            "the agent used the screen and the indicator did not say so in the agent's colour"
        );
        assert!(line.the_agents_dot(), "ADR 0010's mark is not drawn");
    }

    /// **The picture goes to the turn and the capture is spent.**
    #[test]
    fn handing_the_picture_to_the_turn_leaves_nothing_behind() {
        let capture = a_capture(vec![9, 9, 9]);
        let said = capture.as_the_record_says_it();
        let picture = capture.to_the_turn();
        assert_eq!(picture.bytes(), [9, 9, 9]);

        // What the record holds is that it happened, and nothing of what was
        // on the screen.
        assert!(said.starts_with("a picture of the screen was taken"));
        assert!(!said.contains("999") && !said.contains("bytes"));
    }
}
