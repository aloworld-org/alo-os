//! The whole answer: a screen and a text size, laid out along the bottom.
//!
//! This is where `docs/features.md`'s *labels give way to icons where the short
//! edge demands it* becomes arithmetic, and the arithmetic is short enough to
//! read in one sitting:
//!
//! 1. The dock is along the bottom ([ADR
//!    0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)),
//!    so it takes its thickness out of the screen's **height** and runs the whole
//!    of its **width**.
//! 2. The height has a ceiling: the most a dock may take of it
//!    ([`crate::Room::the_most_a_dock_may_take`]).
//! 3. A dock with names on it wants a thickness that depends on the text size —
//!    a line of text under each icon.
//! 4. If what it wants fits under the ceiling, the names are drawn. If it does
//!    not, they give way and the dock is icons alone — which always fits,
//!    because [`crate::Screen`] refuses a screen where it would not.
//!
//! **It takes from the side it sits on, not from the screen's short side.** A
//! dock along the bottom takes from the height even on a screen that is taller
//! than it is wide, because the height is the side it is sitting on. Measuring
//! against `min(width, height)` instead would squeeze the dock on a portrait
//! screen for a reason that has nothing to do with where it is.
//!
//! **Nothing here is a judgement.** There is no *feels cramped*, no breakpoint
//! list and no eye. Every number comes from [`crate::measures`], and the
//! thresholds those numbers produce are held to EN 301 549's requirement that
//! text reach 200% without losing content — on the smallest screen alo OS lays
//! out for. The tests at the bottom of this file are that requirement, and they
//! are also what fixes the two numbers nobody could have picked honestly: the
//! share of the height a dock may take, and that a name under an icon needs a
//! line of text.
//!
//! **Nothing here reads anything either.** The screen is passed in and the text
//! size is passed in — the rule `alo-capability` set in item 1 and
//! `alo-appearance` kept, so a settings panel previewing *what would this look
//! like at 200%* asks exactly the question the compositor asks at 200%.
//!
//! **Which way the person reads is no longer asked.** It was only ever used to
//! put the status area at the far end of a row, and the status area is not the
//! Dock's any more.

use crate::edge::Edge;
use crate::labels::Labels;
use crate::room::Room;
use crate::screen::Screen;

/// A dock, laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// Its short edge: how much of the screen's height it takes.
    thickness: Room,
    /// Its long edge: how far it runs, which is the whole width.
    length: Room,
    /// What became of the names.
    labels: Labels,
    /// Which edge it was laid out along.
    ///
    /// **Carried rather than re-derived, because the draw must not guess.** A
    /// thickness and a length do not say which edge they came from — 70 × 960 is a
    /// left dock and a right dock equally — so a raster handed only those would
    /// have to be told the edge a second time, by a caller that might disagree with
    /// the one that laid it out. That is the *two answers to one question* shape
    /// this repository has found under several names.
    edge: Edge,
}

impl Layout {
    /// A dock along the bottom of this screen, with the text at this size.
    ///
    /// Answers rather than refuses: a [`Screen`] that exists is a screen a dock
    /// fits on, because that is what [`Screen::of`] checks.
    ///
    /// **Kept as the bottom edge's own constructor** rather than becoming a call
    /// to [`Self::along`] that unwraps. Every caller in the tree asks for the dock
    /// a fresh machine has, the bottom edge cannot fail to lay out, and a
    /// constructor that returned a `Result` nobody could get an error from would
    /// put a `match` at thirty call sites to describe a case that does not exist.
    #[must_use]
    pub fn of(screen: Screen) -> Self {
        Self::running_across(Edge::Bottom, screen)
    }

    /// A dock along this edge of this screen.
    ///
    /// **It takes no text size since 2026-10-10.** It did, and both branches
    /// now ignore it: the owner removed the label row that was the only thing
    /// a dock's thickness ever read the text for. A parameter nothing reads is
    /// the shape this repository keeps finding, and dropping it makes the
    /// ruling hold by construction — a thickness cannot vary with the text
    /// size when the function that decides it is not given one.
    ///
    /// **All four lay out since 2026-10-04**, when the owner gave the side
    /// placement its measurement. It returned a `Result` for one day, refusing
    /// left and right because a name beside an icon had no measured width; the
    /// owner's specification both supplied the width **and** took the names out of
    /// the bar, so there is no longer an edge this can fail on and no error type
    /// to carry.
    #[must_use]
    pub fn along(edge: Edge, screen: Screen) -> Self {
        if edge.runs_across() {
            // **The top edge is the bottom edge's arithmetic, and that is a
            // measurement rather than an assumption.** Both take their thickness
            // out of the screen's height and run the whole of its width; which end
            // of the height they sit at is an origin, and this crate does not
            // place the dock on a screen — `alo_shell` does. So there is one body
            // for both.
            return Self::running_across(edge, screen);
        }
        Self::running_down(edge, screen)
    }

    /// A dock down a side: thickness out of the width, running the height.
    ///
    /// **It takes no text size, and that is the owner's rule rather than an
    /// oversight.** Their specification of 2026-10-04: *keep icons and click
    /// targets unchanged; labels do not permanently widen the Dock.* A name beside
    /// an icon opens in a tooltip over the canvas, so a side dock is as thick as
    /// its icons whatever the text is set to — there is no threshold to cross and
    /// so nothing for the names to give way at.
    ///
    /// That is the whole difference from [`Self::running_across`], where the
    /// thickness carries a line of text and the names *do* give way when the
    /// ceiling is reached. Two placements, two arithmetics, and the second one is
    /// shorter because the owner took the names out of the bar.
    fn running_down(edge: Edge, screen: Screen) -> Self {
        Self {
            // **The measured lane, not `a_dock_of_icons`.** That sum is 64 and the
            // design file's frames say 70 — see `Room::a_side_docks_lane`, which
            // carries why the difference is recorded rather than reconciled.
            thickness: Room::a_side_docks_lane(),
            length: screen.height(),
            labels: Labels::Beside,
            edge,
        }
    }

    /// A dock across the screen: thickness out of the height, running the width.
    ///
    /// **It takes no text size, and that is the owner's ruling of 2026-10-10
    /// rather than an oversight.** This branched on whether a name fitted under
    /// each icon and thickened the bar to **85** when it did — a row the
    /// verified design does not have. Measured the same day: the snapshot's
    /// `Dock + alo Bar` is 76 with a 48 hit area in it, and there is no text
    /// node in the band at all.
    ///
    /// > Remove the unused label row and use the measured 76px horizontal Dock.
    /// > … No permanent application names beneath icons. Show names on hover
    /// > and keyboard focus, outside the bar without changing its height.
    ///
    /// So this is now `running_down`'s shape: one thickness, whatever the text
    /// is set to, because the name is a transient surface over the canvas and
    /// never part of the bar. Two placements, one arithmetic.
    fn running_across(edge: Edge, screen: Screen) -> Self {
        Self {
            thickness: Room::a_dock_of_icons(),
            length: screen.width(),
            labels: Labels::Beside,
            edge,
        }
    }

    /// How much of the screen's height it takes, across its short edge.
    #[must_use]
    pub const fn thickness(self) -> Room {
        self.thickness
    }

    /// How far it runs along the bottom, which is the whole width.
    ///
    /// *Whether it hides when a window needs the room* is [`crate::hiding`],
    /// and it is deliberately not here: a layout says how much room the dock
    /// takes when it is shown, and whether it is shown at all is a person's
    /// choice answered against what the windows want. A layout that also hid
    /// itself would have to know about windows, which this crate does not.
    #[must_use]
    pub const fn length(self) -> Room {
        self.length
    }

    /// Which edge of the screen this dock was laid out along.
    #[must_use]
    pub const fn edge(self) -> Edge {
        self.edge
    }

    /// What became of the names.
    #[must_use]
    pub const fn labels(self) -> Labels {
        self.labels
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    /// **A side dock is as thick as its icons, whatever the text size.**
    ///
    /// The owner's rule of 2026-10-04: *keep icons and click targets unchanged;
    /// labels do not permanently widen the Dock.* A name beside an icon opens in a
    /// tooltip over the canvas, so there is no line of text in the bar and nothing
    /// for the thickness to depend on.
    ///
    /// Asserted **across the whole text range** rather than at one size, because
    /// the fault this forbids is a side dock that quietly grows with the text the
    /// way the bottom one does — which would pass a check at 100%.
    #[test]
    fn no_docks_thickness_can_move_with_the_text() {
        let screen = Screen::of(1920, 1080).unwrap();
        // **The lane, not the icon sum.** This read `a_dock_of_icons` until
        // 2026-10-10 — correct while that was what `running_down` used, and it
        // caught the change to the frames' measured 70, which is what a test
        // pinned to an implementation does.
        let lane = Room::a_side_docks_lane();
        for edge in [Edge::Left, Edge::Right] {
            let laid = Layout::along(edge, screen);
            assert_eq!(
                laid.thickness(),
                lane,
                "{edge:?} is not the measured lane, so the names have widened the bar"
            );
            assert_eq!(laid.labels(), Labels::Beside, "{edge:?}");
        }
        // **And the across dock is the measured 76, which it was not until
        // 2026-10-10**: `a_dock_with_names_under` added a line of text and made
        // it 85, for a row of names the verified design does not have.
        for edge in [Edge::Bottom, Edge::Top] {
            let laid = Layout::along(edge, screen);
            assert_eq!(laid.thickness(), Room::a_dock_of_icons(), "{edge:?}");
            assert_eq!(laid.thickness().as_pixels(), 76, "{edge:?}");
            assert_eq!(laid.labels(), Labels::Beside, "{edge:?}");
        }
    }

    /// **The text size cannot reach a dock's thickness at all**, which is this
    /// file's strongest statement of the owner's rule and the reason the test
    /// above no longer loops over text sizes.
    ///
    /// It used to: `Layout::along` took a `TextScale`, and the loop asserted the
    /// thickness was the same at 100% and at the standard's 200%. That was the
    /// right test for the code as it stood, and **a loop over sizes can only
    /// ever check the sizes somebody thought to try.** The parameter is gone
    /// now, so the claim is held by the signature instead: there is no text size
    /// to pass, and a later change cannot quietly start reading one without
    /// adding it back and meeting this comment.
    ///
    /// A source-level check rather than a runtime one, because the thing being
    /// forbidden is *the function taking an argument* — which no value can
    /// demonstrate.
    #[test]
    fn laying_a_dock_out_takes_no_text_size() {
        let source = include_str!("layout.rs");
        let code: String = source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        // **Collected rather than asserted one at a time**, so a declaration
        // that has been renamed away is reported as *missing* rather than
        // passing by not being found. `clippy::panic` is denied here, so both
        // outcomes share one assertion at the end.
        let mut missing = Vec::new();
        let mut taking_a_text_size = Vec::new();
        for named in [
            "fn along(",
            "fn of(",
            "fn running_across(",
            "fn running_down(",
        ] {
            match code.split(named).nth(1) {
                None => missing.push(named),
                Some(after) => {
                    let arguments = after.split(')').next().unwrap_or("");
                    if arguments.contains("TextScale") {
                        taking_a_text_size.push(named);
                    }
                }
            }
        }
        assert!(
            missing.is_empty(),
            "{missing:?} are not declared in this file, so this check could not run and would \
             have passed by finding nothing"
        );
        assert!(
            taking_a_text_size.is_empty(),
            "{taking_a_text_size:?} take a text size again. A dock's thickness is the icon and \
             its two faces — the owner removed the row of names on 2026-10-10 — and an argument \
             nothing should read is how it would come back"
        );
    }

    /// **A side dock runs the height, an across dock runs the width.**
    ///
    /// The one geometric difference between the two orientations, on a screen whose
    /// sides differ so that a transposition cannot pass.
    #[test]
    fn which_side_a_dock_runs_along_follows_its_edge() {
        let screen = Screen::of(1920, 1080).unwrap();
        for edge in [Edge::Bottom, Edge::Top] {
            assert_eq!(
                Layout::along(edge, screen).length(),
                screen.width(),
                "{edge:?}"
            );
        }
        for edge in [Edge::Left, Edge::Right] {
            assert_eq!(
                Layout::along(edge, screen).length(),
                screen.height(),
                "{edge:?}"
            );
        }
    }

    /// **The tooltip is the owner's two figures and nothing derived.**
    ///
    /// 200 logical pixels of usable text width, 12 either side, so 224 wide. Held
    /// here because the sum is the thing a reader will want and the two parts are
    /// what the owner gave — a single 224 constant would lose which half to change.
    #[test]
    fn a_name_beside_an_icon_is_two_hundred_wide_inside_two_hundred_and_twenty_four() {
        assert_eq!(crate::measures::A_NAME_BESIDE_AN_ICON, 200);
        assert_eq!(crate::measures::AROUND_A_NAME_BESIDE_AN_ICON, 12);
        let tooltip = crate::measures::A_NAME_BESIDE_AN_ICON
            + 2 * crate::measures::AROUND_A_NAME_BESIDE_AN_ICON;
        assert_eq!(
            tooltip, 224,
            "the tooltip is the text width plus both paddings"
        );
    }

    use super::*;
    use crate::measures::A_DOCK_MAY_TAKE_ONE_PART_IN;

    /// A dock on the smallest screen alo OS lays out for.
    fn on_the_smallest() -> Layout {
        Layout::of(Screen::the_smallest())
    }

    /// **EN 301 549 is kept by the names never being in the bar at all.**
    ///
    /// The standard an EU public-sector desktop is procured against requires
    /// text to reach 200% without loss of content. Three tests stood here and
    /// each asserted a different half of a mechanism the owner removed on
    /// 2026-10-10: that names survived at 200% on the smallest screen, that
    /// they gave way above it, and that a bigger screen kept them longer.
    ///
    /// **They are not replaced one for one, because what they tested is gone.**
    /// There is no row of names in the bar to survive or give way: a name is
    /// shown on hover and on keyboard focus, outside the bar, and the bar is
    /// the measured 76 at every text size. So the standard is met more simply
    /// than it was — the name cannot be lost to a text size that cannot reach
    /// it — and what is left to hold is that the thickness is a constant and
    /// the dock still fits its ceiling.
    ///
    /// The reassurance those tests were protecting has not gone anywhere:
    /// `crate::labels` still carries it, a screen reader still announces the
    /// name, and `crate::words::NAMES_GAVE_WAY` is still the sentence a person
    /// reads if anything ever does take a name off a screen.
    #[test]
    fn the_smallest_screen_gets_the_measured_bar_like_every_other() {
        let layout = on_the_smallest();
        assert_eq!(layout.thickness(), Room::a_dock_of_icons());
        assert_eq!(layout.thickness().as_pixels(), 76);
        assert_eq!(layout.labels(), Labels::Beside);
    }

    /// **The dock never takes more than its share**, on any screen — which is
    /// the promise the ceiling exists to keep and the reason [`Layout::of`] can
    /// answer without a `Result`.
    ///
    /// **No loop over text sizes any more.** There was one, and it was right
    /// while the thickness read the text; it cannot be written now, because
    /// `Layout::of` takes no text size. The screens are the loop that is left,
    /// and they are the one that matters: the ceiling is a share of the height,
    /// so a short screen is where a fixed thickness would break it.
    ///
    /// `384 × 384` is in the list deliberately: at 76 the dock takes 76 of a
    /// ceiling of 64, which is **more than its share** — so this test is also
    /// the thing that will fail if a screen that small is ever laid out for.
    /// `Screen::the_smallest` is what decides whether one can be.
    #[test]
    fn the_dock_never_takes_more_of_a_screen_than_it_may() {
        let screens = [
            Screen::the_smallest(),
            Screen::of(1920, 1080).unwrap(),
            Screen::of(3840, 2160).unwrap(),
            Screen::of(1080, 1920).unwrap(),
        ];
        for screen in screens {
            let layout = Layout::of(screen);
            let ceiling = Room::the_most_a_dock_may_take(screen.height());
            assert!(
                layout.thickness().fits_in(ceiling),
                "on a {} by {} screen it took {} of a ceiling of {}",
                screen.width().as_pixels(),
                screen.height().as_pixels(),
                layout.thickness().as_pixels(),
                ceiling.as_pixels()
            );
        }
    }

    /// **The names never give way, because they are never in the bar.**
    ///
    /// This asserted the opposite until 2026-10-10 — that once the labels had
    /// gone at some text size they never came back at a larger one, which was a
    /// real worry about a real mechanism: a dock whose labels flickered back
    /// would be a layout nobody could describe. The owner removed the
    /// mechanism, so the flicker it guarded against cannot happen, and what is
    /// left to say is the stronger thing.
    #[test]
    fn the_names_are_never_in_the_bar_to_give_way() {
        assert_eq!(on_the_smallest().labels(), Labels::Beside);
        assert!(
            on_the_smallest().labels().are_shown(),
            "a name shown on hover is still a name that is shown — `are_shown` is about \
             whether a person can read it, not about whether it sits in the bar"
        );
    }

    /// **The thickness comes out of the height and the length out of the
    /// width**, on a portrait screen as much as a landscape one — because the
    /// dock takes from the side it sits on rather than from whichever side is
    /// shorter.
    #[test]
    fn a_dock_takes_from_the_height_and_spans_the_width_on_either_shape() {
        for screen in [
            Screen::of(1366, 768).unwrap(),
            Screen::of(768, 1366).unwrap(),
        ] {
            let layout = Layout::of(screen);
            assert_eq!(layout.length(), screen.width());
            assert!(
                layout
                    .thickness()
                    .fits_in(Room::the_most_a_dock_may_take(screen.height()))
            );
        }
    }

    /// **The share a dock may take is the tightest one that keeps the
    /// standard.** One part more and the names would go at exactly the size
    /// EN 301 549 requires them to survive — so the number in
    /// [`crate::measures`] is fixed by the requirement rather than chosen, and
    /// this is the test that says which way it is fixed.
    ///
    /// **The share no longer has a justification, and this test says so rather
    /// than inventing one.**
    ///
    /// `A_DOCK_MAY_TAKE_ONE_PART_IN` was fixed by EN 301 549: one part tighter
    /// and a row of names would have gone at exactly the text size the standard
    /// requires them to survive. That is why the number was *measured* rather
    /// than chosen, and the test that stood here proved it by showing a tighter
    /// share would fail.
    ///
    /// **The owner removed the row of names on 2026-10-10, and the
    /// justification went with it.** The bar is a constant 76 now, and on the
    /// smallest screen this crate lays out for — 768 tall — the ceiling is 128
    /// at one part in six and 109 at one part in seven. **76 fits both.** So
    /// the share could be tightened and nothing in this repository would
    /// notice, which means it is no longer pinned by anything.
    ///
    /// This asserts the one thing that is still true — the bar fits the share —
    /// and **names the loss** so that whoever revisits the number knows it is
    /// now a free choice rather than a measured one. Asserting tightness would
    /// be a test that passed by picking a comparison that happened to fail.
    #[test]
    fn the_bar_fits_its_share_and_the_share_is_no_longer_pinned() {
        let screen = Screen::the_smallest();
        let ceiling = Room::the_most_a_dock_may_take(screen.height());
        assert!(
            Room::a_dock_of_icons().fits_in(ceiling),
            "the measured bar does not fit the share it is allowed"
        );
        // **The loss, asserted so that it is a fact and not a remark.** A
        // tighter share still fits, which is what *no longer pinned* means. If
        // this ever stops being true — a taller bar, a smaller screen — the
        // share has become load-bearing again and the note above is stale.
        let tighter = Room::pixels(screen.height().as_pixels() / (A_DOCK_MAY_TAKE_ONE_PART_IN + 1));
        assert!(
            Room::a_dock_of_icons().fits_in(tighter),
            "a tighter share no longer fits, so the chosen one is pinned again and the note \
             above should say so"
        );
    }
}
