//! What is watching or listening, right now.
//!
//! The live value the plan's task 1 asks for: **every current use of the
//! screen, the camera and the microphone, each naming who is using it.** It is
//! a snapshot rather than a stream of events, for the reason
//! `alo_egress::Indicator` gives about the other indicator — a person glancing
//! at their machine is asking what is happening *now*, and a list answers that
//! after a missed notification where a stream of events does not.
//!
//! # There is one door, and it is the machine's own media server
//!
//! [`InUse::read_from`] is the only constructor. There is no public field, no
//! `From`, no deserialiser, and no way to build one out of a list somebody
//! assembled — so what a shell draws is what a machine's media server said was
//! open, and there is no second answer to *is my camera on*.
//!
//! That is deliberately the same argument `alo_indicator::Lamp` makes, and the
//! compile-fail examples below are it as tests: a second door added later stops
//! being a design discussion and starts being a failing build.
//!
//! # Nothing is hidden, and nothing is trusted
//!
//! [`InUse::read_from`] keeps **every** use the server answered with, in the
//! order it answered. There is no filter, no allow-list, no *except ours*, and
//! no setting anywhere in this crate that turns a line off — which is the
//! plan's acceptance stated three ways, and each of those three is a test in
//! this file. alo OS's own captures appear like anybody else's, because nothing
//! here can tell them apart in order to do otherwise.
//!
//! # And it never reads as though nothing were happening
//!
//! A machine that cannot reach its media server answers [`crate::NotHeard`]
//! rather than an empty list. An empty indicator means the room is quiet; it
//! must never also mean the machine could not tell.

use alo_strings::{Filling, Said, Strings};

use crate::line::Line;
use crate::refusing::NotHeard;
use crate::streams::Streams;
use crate::used::Used;
use crate::uses::Use;
use crate::words;

/// Everything using the screen, the camera or the microphone at this moment.
///
/// ```
/// use alo_applications::Application;
/// use alo_in_use::{By, InUse, NotHeard, Streams, Use, UseId, Used};
///
/// /// A media server with one camera stream open.
/// struct AServer;
/// impl Streams for AServer {
///     fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
///         Ok(vec![Use::of(
///             UseId::recorded(42),
///             Used::Camera,
///             By::an_application(
///                 Application::called("com.example.VideoCall", "Video Call")
///                     .expect("an identifier a verb could name"),
///             ),
///         )])
///     }
/// }
///
/// let in_use = InUse::read_from(&mut AServer)?;
/// assert!(!in_use.is_quiet());
/// assert!(in_use.is_in_use(Used::Camera));
/// assert!(!in_use.is_in_use(Used::Microphone));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// What is in use cannot be assembled from a list somebody made up, because
/// there is no such constructor to call:
///
/// ```compile_fail
/// let made_up = alo_in_use::InUse::of(vec![]);
/// ```
///
/// nor can a use be left off one:
///
/// ```compile_fail
/// let hiding = alo_in_use::InUse::read_from_except(&mut server, &["com.example.VideoCall"]);
/// ```
///
/// nor can the indicator be turned off:
///
/// ```compile_fail
/// let off = alo_in_use::InUse::shown(false);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InUse {
    /// Every use, in the order the media server answered with them.
    ///
    /// Filled by [`InUse::read_from`] and by nothing else, which is the whole
    /// guarantee this type makes.
    uses: Vec<Use>,
}

impl InUse {
    /// What this machine's media server says is in use, right now.
    ///
    /// The only way to an [`InUse`]. Everything the server answered is kept, in
    /// the order it answered: there is no argument here that could leave one
    /// out, and no branch inside that does.
    ///
    /// # Errors
    /// [`NotHeard`], when the media server is not there, does not answer, or
    /// answers something that cannot be read. Never an empty list standing in
    /// for one of those.
    pub fn read_from(streams: &mut impl Streams) -> Result<Self, NotHeard> {
        Ok(Self {
            uses: streams.in_use_now()?,
        })
    }

    /// Every current use, in the order the media server answered with them.
    #[must_use]
    pub fn uses(&self) -> &[Use] {
        &self.uses
    }

    /// Every current use of one thing.
    pub fn using(&self, what: Used) -> impl Iterator<Item = &Use> {
        self.uses.iter().filter(move |one| one.what() == what)
    }

    /// Whether this thing is in use at all.
    #[must_use]
    pub fn is_in_use(&self, what: Used) -> bool {
        self.using(what).next().is_some()
    }

    /// How many things are using the screen, the camera or the microphone.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.uses.len()
    }

    /// Whether nothing is watching or listening.
    ///
    /// The state a machine is in for most of a day — and the honest one, since
    /// a machine that could not ask its media server refuses rather than
    /// answering this.
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.uses.is_empty()
    }

    /// The lines the indicator shows, in the order they sit in.
    ///
    /// Grouped by what is in use — the screen, then the camera, then the
    /// microphone — and within each group in the order the server answered, so
    /// that a line does not move under somebody's eye because another one
    /// ended.
    #[must_use]
    pub fn lines(&self) -> Vec<Line> {
        let mut lines: Vec<Line> = self.uses.iter().map(Use::line).collect();
        lines.sort_by_key(Line::position);
        lines
    }

    /// What the indicator reads as, in the language the person reads.
    ///
    /// One sentence per line, or the single sentence for a quiet room when
    /// there are no lines. **Never empty**: an indicator that said nothing
    /// would be indistinguishable from one that was not there, and a person
    /// checking whether their microphone is on would have no answer either way.
    #[must_use]
    pub fn reads_as(&self, strings: &Strings) -> Vec<Said> {
        if self.uses.is_empty() {
            return vec![strings.say(&words::NOTHING_IS_IN_USE.key(), &Filling::nothing())];
        }
        self.lines().iter().map(|line| line.said(strings)).collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::by::By;
    use crate::testing::{
        a_server_answering, a_server_refusing, a_video_call, every_shape_of_use, in_english,
        the_agent, the_camera_by,
    };
    use crate::uses::UseId;

    /// **A machine with nothing watching or listening is quiet**, and says so
    /// in one sentence rather than in silence.
    #[test]
    fn a_machine_with_nothing_watching_or_listening_is_quiet() {
        let in_use = InUse::read_from(&mut a_server_answering(vec![])).unwrap();
        assert!(in_use.is_quiet());
        assert_eq!(in_use.how_many(), 0);
        assert!(in_use.lines().is_empty());
        for what in Used::EVERY {
            assert!(!in_use.is_in_use(what), "{what:?}");
        }

        let reads = in_use.reads_as(&in_english());
        assert_eq!(reads.len(), 1);
        assert_eq!(
            reads.first().map(Said::text),
            Some("Nothing is watching or listening right now")
        );
    }

    /// **Every use the server answered with is on the list**, in the order it
    /// answered — the plan's *no variant that hides a use*, at the one place a
    /// use could plausibly be dropped.
    #[test]
    fn every_use_the_server_answered_with_is_on_the_list() {
        let answered = every_shape_of_use();
        let how_many = answered.len();
        let in_use = InUse::read_from(&mut a_server_answering(answered.clone())).unwrap();
        assert_eq!(in_use.how_many(), how_many);
        assert_eq!(in_use.uses(), answered.as_slice());
    }

    /// **alo OS's own use is on the list like anybody else's.** The
    /// comfortable failure would be an indicator that showed applications and
    /// quietly left the machine's own captures off, and `docs/features.md`
    /// promises the opposite in as many words: *by any application, including
    /// ours*.
    #[test]
    fn what_alo_os_does_itself_is_on_the_list_like_anybody_elses() {
        let ours = Use::of(UseId::recorded(1), Used::Screen, By::alo_os_itself());
        let theirs = the_camera_by(By::an_application(a_video_call()), 2);
        let in_use =
            InUse::read_from(&mut a_server_answering(vec![ours.clone(), theirs.clone()])).unwrap();

        assert_eq!(in_use.how_many(), 2);
        assert!(in_use.is_in_use(Used::Screen));
        assert!(in_use.is_in_use(Used::Camera));
        assert!(in_use.uses().contains(&ours));
        assert_eq!(
            in_use.using(Used::Screen).next().map(Use::by),
            Some(&By::alo_os_itself())
        );
    }

    /// **The agent's use is on the same list too**, and is the only one drawn
    /// in terracotta.
    #[test]
    fn the_agents_use_is_on_the_same_list_and_is_the_only_terracotta_one() {
        let in_use = InUse::read_from(&mut a_server_answering(every_shape_of_use())).unwrap();
        let terracotta = in_use
            .lines()
            .into_iter()
            .filter(|line| line.colour() == alo_appearance::Token::Terracotta)
            .count();
        assert_eq!(terracotta, 1);
        assert!(
            in_use
                .uses()
                .iter()
                .any(|one| one.by().agent() == Some(&the_agent()))
        );
    }

    /// **Two things using one camera are two lines.** An indicator that
    /// collapsed them would answer *the camera is in use* and hide which of the
    /// two a person wanted to stop.
    #[test]
    fn two_things_using_one_camera_are_two_lines() {
        let in_use = InUse::read_from(&mut a_server_answering(vec![
            the_camera_by(By::an_application(a_video_call()), 1),
            the_camera_by(By::alo_os_itself(), 2),
        ]))
        .unwrap();
        assert_eq!(in_use.using(Used::Camera).count(), 2);
        assert_eq!(in_use.lines().len(), 2);
        assert_eq!(in_use.reads_as(&in_english()).len(), 2);
    }

    /// **The lines sit where their thing sits**, whoever is using it — so the
    /// microphone is in the same place on the indicator every time.
    #[test]
    fn the_lines_sit_in_the_order_their_things_sit_in() {
        let in_use = InUse::read_from(&mut a_server_answering(vec![
            Use::of(UseId::recorded(1), Used::Microphone, By::alo_os_itself()),
            Use::of(UseId::recorded(2), Used::Screen, By::alo_os_itself()),
            the_camera_by(By::an_application(a_video_call()), 3),
        ]))
        .unwrap();
        assert_eq!(
            in_use
                .lines()
                .into_iter()
                .map(|line| line.what())
                .collect::<Vec<_>>(),
            vec![Used::Screen, Used::Camera, Used::Microphone]
        );
    }

    /// **A machine that could not ask refuses rather than reading as quiet.**
    /// This is the refusal path that matters most in the whole crate: an empty
    /// indicator and an unanswerable one look identical on a screen, and only
    /// one of them is true.
    #[test]
    fn a_machine_that_cannot_ask_refuses_rather_than_looking_quiet() {
        for refusal in [
            NotHeard::NothingHandlesSoundAndVideo {
                said: "nothing to run".to_owned(),
            },
            NotHeard::NoAnswer {
                said: "no socket".to_owned(),
            },
            NotHeard::NotUnderstood {
                said: "not a list".to_owned(),
            },
        ] {
            let answered = InUse::read_from(&mut a_server_refusing(refusal.clone()));
            assert_eq!(answered, Err(refusal.clone()));
            assert!(
                refusal
                    .said(&in_english())
                    .text()
                    .contains("cannot be shown"),
                "{refusal:?}"
            );
        }
    }

    /// **The indicator always says something.** Whatever is or is not
    /// happening, a person glancing at it reads a sentence rather than an empty
    /// space they have to interpret.
    #[test]
    fn the_indicator_always_says_something() {
        let strings = in_english();
        for answered in [Vec::new(), every_shape_of_use()] {
            let in_use = InUse::read_from(&mut a_server_answering(answered)).unwrap();
            let reads = in_use.reads_as(&strings);
            assert!(!reads.is_empty());
            for said in reads {
                assert!(!said.text().is_empty());
                assert!(!said.is_a_bug());
                assert!(said.unfilled().is_empty(), "{said} has a gap left in it");
            }
        }
    }
}
