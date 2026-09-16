//! One line of the indicator: the mark, the words, the place and the colour.
//!
//! A [`Line`] is made from a [`crate::Use`] and from nothing else, and it has
//! no constructor that takes a colour, a mark, a place or a sentence. That is
//! the same provenance argument `alo_indicator::Lamp` makes about the other
//! indicator, made here about this one: what is drawn is what the machine found
//! on its media server, and there is no second door through which a line could
//! arrive that no use is behind.
//!
//! # Mark, word and position — and only then colour
//!
//! [ADR 0010](../../../docs/decisions/0010-terracotta-is-reserved-and-never-alone.md)
//! is categorical: the agent is never signalled by colour alone, and it
//! measured why — terracotta on cream is 2.87:1, under the 3.0:1 WCAG 2.1
//! §1.4.11 asks of a shape that carries meaning. So every line carries three
//! things that are not a colour:
//!
//! - a **mark**, one silhouette per thing ([`crate::Mark`]);
//! - a **word**: a whole sentence naming what is in use and who is using it,
//!   and saying *the agent* in so many words when it is the agent;
//! - a **position**, fixed by what is in use and never by who
//!   ([`crate::Position`]).
//!
//! [`Line::colour`] comes fourth and adds nothing that is not already there. A
//! machine whose colours were all one colour would still tell a person
//! everything this indicator has to say.
//!
//! # Terracotta, and only for the agent
//!
//! [`Line::colour`] answers `Token::Terracotta` exactly when
//! [`crate::By::is_the_agents`] does, and `Token::Navy` — the colour alo OS
//! draws structure and text in — for everything else. There is no third case
//! and no setting: an application using the camera is not the machine acting on
//! the person's behalf, and drawing it as though it were would spend the one
//! signal ADR 0010 reserved.

use alo_appearance::Token;
use alo_strings::{Said, Strings, Word};

use crate::by::By;
use crate::mark::Mark;
use crate::position::Position;
use crate::used::Used;
use crate::uses::Use;

/// One line of the in-use indicator, as a shell is handed it.
///
/// ```
/// use alo_applications::Application;
/// use alo_appearance::Token;
/// use alo_capability::Grantee;
/// use alo_in_use::{By, Mark, Position, Use, UseId, Used};
///
/// let camera = Use::of(
///     UseId::recorded(42),
///     Used::Camera,
///     By::an_application(Application::called("com.example.VideoCall", "Video Call")?),
/// );
/// let line = camera.line();
/// assert_eq!(line.mark(), Mark::Lens);
/// assert_eq!(line.position(), Position::Second);
/// assert_eq!(line.colour(), Token::Navy);
///
/// let agent = Use::of(
///     UseId::recorded(43),
///     Used::Screen,
///     By::the_agent(&Grantee::named("@alo")).expect("an agent, not an application"),
/// );
/// assert_eq!(agent.line().colour(), Token::Terracotta);
/// assert_eq!(agent.line().mark(), Mark::Rectangle);
/// # Ok::<(), alo_applications::NotAnApplication>(())
/// ```
///
/// A line cannot be drawn in a colour somebody chose, because there is no such
/// constructor to call:
///
/// ```compile_fail
/// let line = alo_in_use::Line::in_colour(alo_appearance::Token::Terracotta);
/// ```
///
/// nor can one be hidden, because there is nothing to call:
///
/// ```compile_fail
/// let hidden = alo_in_use::Line::hidden();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// What is in use.
    what: Used,
    /// Who is using it.
    by: By,
}

impl Line {
    /// The line for this use.
    ///
    /// The only way to a [`Line`].
    #[must_use]
    pub fn of(one: &Use) -> Self {
        Self {
            what: one.what(),
            by: one.by().clone(),
        }
    }

    /// What is in use.
    #[must_use]
    pub const fn what(&self) -> Used {
        self.what
    }

    /// Who is using it.
    #[must_use]
    pub const fn by(&self) -> &By {
        &self.by
    }

    /// The shape drawn beside it, one per thing in use.
    #[must_use]
    pub const fn mark(&self) -> Mark {
        self.what.mark()
    }

    /// Where it sits, which is fixed by what is in use and never by who.
    #[must_use]
    pub const fn position(&self) -> Position {
        self.what.position()
    }

    /// The colour it is drawn in: terracotta for the agent, and the colour
    /// everything else on the machine is written in for everything else.
    ///
    /// Reserved (ADR 0010) and carrying nothing the mark and the sentence do
    /// not already carry.
    #[must_use]
    pub const fn colour(&self) -> Token {
        if self.by.is_the_agents() {
            Token::Terracotta
        } else {
            Token::Navy
        }
    }

    /// Whether the agent's own mark — ADR 0010's small dot — is drawn beside
    /// it.
    ///
    /// True exactly when the line is terracotta, because ADR 0010 says the two
    /// arrive together or not at all.
    #[must_use]
    pub const fn the_agents_dot(&self) -> bool {
        self.by.is_the_agents()
    }

    /// The string this crate declares for the line, before who is filled in.
    #[must_use]
    pub const fn word(&self) -> Word {
        self.what.word()
    }

    /// What the line reads as, in the language the person reads.
    ///
    /// The whole of what one line of the indicator says: what is in use, and by
    /// what. A line with no words is a line that does not exist for somebody
    /// using a screen reader, and this indicator is not a promise alo OS keeps
    /// for people who can see a mark and breaks for people who cannot.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &self.by.filling(strings))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_video_call, every_shape_of_use, in_english, the_agent, translated};
    use crate::words;

    /// **Every line says what is in use and by what**, in one sentence a screen
    /// reader can announce on its own.
    #[test]
    fn every_line_says_what_is_in_use_and_by_what() {
        let strings = in_english();
        for one in every_shape_of_use() {
            let said = one.line().said(&strings);
            assert!(!said.is_a_bug(), "{one:?} is not declared");
            assert!(said.unfilled().is_empty(), "{said} has a gap left in it");
            assert!(
                said.text().starts_with("The "),
                "{said} does not begin the sentence a line begins"
            );
        }

        assert_eq!(
            the_camera_by_a_video_call().line().said(&strings).text(),
            "The camera is in use by Video Call (com.example.VideoCall)"
        );
    }

    /// **alo OS's own use reads like anybody else's**, on the same indicator,
    /// in the same shape of sentence. `docs/features.md` promises the indicator
    /// *by any application, including ours*, and a line that quietly left the
    /// machine's own captures off would be the comfortable answer and the false
    /// one.
    #[test]
    fn what_alo_os_does_itself_reads_like_anybody_elses_line() {
        let strings = in_english();
        let ours = Use::of(crate::UseId::recorded(7), Used::Screen, By::alo_os_itself());
        let theirs = Use::of(
            crate::UseId::recorded(8),
            Used::Screen,
            By::an_application(a_video_call()),
        );

        assert_eq!(
            ours.line().said(&strings).text(),
            "The screen is in use by alo OS itself"
        );
        assert_eq!(ours.line().mark(), theirs.line().mark());
        assert_eq!(ours.line().position(), theirs.line().position());
        assert_eq!(ours.line().colour(), theirs.line().colour());
        assert!(!ours.line().the_agents_dot());
    }

    /// **Terracotta is the agent's and nobody else's** (ADR 0010), and the
    /// agent's dot arrives with it rather than instead of it.
    #[test]
    fn no_line_is_terracotta_unless_the_agent_is_the_one_using_it() {
        for one in every_shape_of_use() {
            let line = one.line();
            assert_eq!(
                line.colour() == Token::Terracotta,
                line.by().is_the_agents(),
                "{one:?} is coloured for somebody it is not"
            );
            assert_eq!(
                line.the_agents_dot(),
                line.by().is_the_agents(),
                "{one:?} draws the agent's mark for somebody it is not"
            );
        }
    }

    /// **The agent's line says *the agent* in words**, so the signal survives a
    /// person who cannot tell terracotta from anything else — which ADR 0010
    /// measured as everybody, on the reading ground, at 2.87:1.
    #[test]
    fn the_agents_line_says_so_without_its_colour() {
        let strings = in_english();
        let line = Use::of(
            crate::UseId::recorded(1),
            Used::Screen,
            By::the_agent(&the_agent()).unwrap(),
        )
        .line();
        assert_eq!(
            line.said(&strings).text(),
            "The screen is in use by @alo, the agent on this machine"
        );
        assert!(line.said(&strings).text().contains("the agent"));
    }

    /// **The three things that are not a colour tell the three uses apart.**
    /// Mark, word and position, each on its own: any one of them collapsing
    /// would leave colour doing work ADR 0010 forbids it to do alone.
    #[test]
    fn mark_word_and_position_tell_the_lines_apart_with_no_colour_at_all() {
        let strings = in_english();
        let by = By::an_application(a_video_call());
        let lines: Vec<Line> = Used::EVERY
            .into_iter()
            .map(|what| Use::of(crate::UseId::recorded(1), what, by.clone()).line())
            .collect();

        let marks: Vec<Mark> = lines.iter().map(Line::mark).collect();
        let places: Vec<Position> = lines.iter().map(Line::position).collect();
        let sentences: Vec<String> = lines
            .iter()
            .map(|line| line.said(&strings).text().to_owned())
            .collect();
        let colours: Vec<Token> = lines.iter().map(Line::colour).collect();

        assert_eq!(marks.len(), 3);
        for (first, second) in [(0_usize, 1_usize), (1, 2), (0, 2)] {
            assert_ne!(marks.get(first), marks.get(second));
            assert_ne!(places.get(first), places.get(second));
            assert_ne!(sentences.get(first), sentences.get(second));
            // And the colours are all the same, which is the point: nothing
            // above was told apart by one.
            assert_eq!(colours.get(first), colours.get(second));
        }
    }

    /// **The line is read in the language the person reads**, and is only as
    /// translated as its least translated piece.
    #[test]
    fn a_line_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[
            (words::THE_CAMERA, "Die Kamera wird von {who} benutzt"),
            (words::ALO_OS_ITSELF, "alo OS selbst"),
        ]);
        let line = Use::of(crate::UseId::recorded(1), Used::Camera, By::alo_os_itself()).line();
        let said = line.said(&strings);
        assert!(said.is_translated());
        assert_eq!(said.text(), "Die Kamera wird von alo OS selbst benutzt");

        // With the clause left untranslated the whole line says so, rather than
        // claiming a translation it only half has.
        let half = translated(&[(words::THE_CAMERA, "Die Kamera wird von {who} benutzt")]);
        let half_said = Use::of(crate::UseId::recorded(1), Used::Camera, By::alo_os_itself())
            .line()
            .said(&half);
        assert!(!half_said.is_translated(), "{half_said}");
        assert_eq!(
            half_said.text(),
            "Die Kamera wird von alo OS itself benutzt"
        );
    }

    /// The camera, used by a video-call application — the plan's own example of
    /// a line.
    fn the_camera_by_a_video_call() -> Use {
        Use::of(
            crate::UseId::recorded(42),
            Used::Camera,
            By::an_application(a_video_call()),
        )
    }
}
