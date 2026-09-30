//! Reaching a surface at a screen edge when a window is filling the screen.
//!
//! True full screen covers the Dock, and it covers the panel of windows a
//! person put aside. Moving to that surface's edge reveals it over the content,
//! and **the pointer can travel onto the revealed surface and click without it
//! disappearing on the way.**
//!
//! # That last clause is why this is a state machine
//!
//! It reads like a detail and it is the entire difficulty. The obvious
//! implementation reveals while the pointer is in a strip at the edge and hides
//! when it leaves — and the surface is drawn *over* that strip, so moving
//! towards the thing that just appeared makes it vanish. Everybody has used a
//! system that does this.
//!
//! # One surface, one machine, and the edge is not in here
//!
//! **Which edge is the caller's.** This file consumes a classification — *at
//! the edge*, *on the surface*, *elsewhere* — and never a coordinate, so it
//! does not know how tall the Dock is, where the panel starts, or which side of
//! the screen either lives on. That was already true when it served only the
//! Dock, which is why serving the panel as well is not a rewrite of what it
//! consumes.
//!
//! **Each surface holds its own.** The Dock's reveal and the panel's are two
//! values, so revealing or dismissing one cannot move the other. Nothing here
//! is shared.
//!
//! `docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md`
//! is untouched by this. The Dock is still fixed to the bottom edge; the panel
//! reveals from the right. **An edge is a fact about a surface, not a setting
//! offered to a person**, and nothing here is to be read as giving the choice
//! of edge back.
//!
//! # What holds it open is a set, not the last thing that happened
//!
//! **This is the part that was wrong until 2026-09-30.** The machine held two
//! states and answered from the most recent event, so the pointer leaving
//! concealed it whatever the keyboard was doing, and the keyboard leaving
//! concealed it whatever the pointer was doing. The two roads overrode each
//! other: somebody who tabbed to the surface and then moved the mouse away lost
//! it.
//!
//! So the state is **which regions currently hold it open**, and it is revealed
//! while any of them does. Leaving one cannot dismiss it while another remains
//! — which is not a case in a match, it is what a set means.
//!
//! # Nothing here is a timer
//!
//! No delay before revealing, no grace period after leaving. A timer would make
//! this file's behaviour depend on how fast somebody moves, which is exactly
//! what a person with a tremor or a trackball cannot control — and a grace
//! period is the usual patch for the bug this state machine does not have.
//!
//! Whether the reveal is animated is the compositor's, read from the person's
//! motion preference.

/// Where the pointer is, as far as revealing is concerned.
///
/// Three places rather than a coordinate: this file should not know how tall a
/// surface is or where its strip ends, because then two files would have
/// opinions about the same edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThePointer {
    /// In the strip along this surface's edge that asks for it.
    AtTheEdge,
    /// Over the surface itself while it is revealed — including its previews,
    /// its controls and the path between them, which the caller classifies as
    /// one continuous region.
    OnTheSurface,
    /// Anywhere else.
    Elsewhere,
}

/// Whether the keyboard is on the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheKeyboard {
    /// The surface has keyboard focus.
    IsOnIt,
    /// It does not.
    IsElsewhere,
}

/// Whether something is being dragged that concerns the surface.
///
/// A drag holds it open on its own: letting go of a window over a panel that
/// vanished mid-drag is a person losing what they were carrying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ADrag {
    /// A drag is under way.
    IsHappening,
    /// None is.
    IsNot,
}

/// Whether a menu belonging to the surface is open.
///
/// A menu holds it open on its own: a menu whose surface disappeared is a menu
/// about nothing, and the pointer is usually off the surface while using one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AMenu {
    /// One of this surface's menus is open.
    IsOpen,
    /// None is.
    IsNot,
}

/// Where the keyboard goes when the surface is dismissed with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusGoes {
    /// Back to the window filling the screen, which is where it came from.
    ///
    /// **A person who tabbed to the surface and pressed Escape must land back
    /// in their work**, not nowhere. Focus that goes nowhere strands whoever
    /// took the keyboard road.
    BackToTheWindow,
    /// Nowhere: it did not have the keyboard, so there is nothing to give back.
    Nowhere,
}

/// What a dismissal left behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dismissed {
    /// The surface afterwards.
    pub revealing: Revealing,
    /// What the shell must do with the keyboard.
    pub focus: FocusGoes,
}

/// Whether a surface is revealed over a full-screen window, and what holds it
/// there.
///
/// Revealed while **any** region holds it; concealed when none does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Revealing {
    /// The pointer is in the strip at the edge.
    at_the_edge: bool,
    /// The pointer is on the surface.
    on_the_surface: bool,
    /// The surface holds the keyboard.
    has_the_keyboard: bool,
    /// A drag that concerns this surface is under way.
    dragging: bool,
    /// One of its menus is open.
    menu_open: bool,
}

impl Revealing {
    /// Covered, as a surface is at rest under a full-screen window.
    #[must_use]
    pub const fn covered() -> Self {
        Self {
            at_the_edge: false,
            on_the_surface: false,
            has_the_keyboard: false,
            dragging: false,
            menu_open: false,
        }
    }

    /// Whether the surface is on the screen at this moment.
    ///
    /// **Any one region is enough**, which is the rule that leaving one cannot
    /// dismiss it while another remains.
    #[must_use]
    pub const fn is_revealed(self) -> bool {
        self.at_the_edge
            || self.on_the_surface
            || self.has_the_keyboard
            || self.dragging
            || self.menu_open
    }

    /// Where things stand after the pointer moves here.
    ///
    /// The strip and the surface are two regions rather than one, so moving
    /// from the strip onto the surface hands the hold from one to the other
    /// without passing through nothing.
    ///
    /// **Being told the pointer is on a surface nothing is holding open does
    /// not reveal it.** There was nothing to be on, and answering otherwise
    /// would invent a reveal nobody asked for.
    #[must_use]
    pub const fn the_pointer_is(self, now: ThePointer) -> Self {
        match now {
            ThePointer::AtTheEdge => Self {
                at_the_edge: true,
                ..self
            },
            ThePointer::OnTheSurface => Self {
                at_the_edge: false,
                on_the_surface: self.is_revealed(),
                ..self
            },
            ThePointer::Elsewhere => Self {
                at_the_edge: false,
                on_the_surface: false,
                ..self
            },
        }
    }

    /// Where things stand when the keyboard moves.
    ///
    /// **A person who cannot point still reaches the surface.** Focusing it
    /// reveals it and holds it while it keeps focus — the same shape as the
    /// pointer's *stay while you are on it*, so neither road is a consolation.
    #[must_use]
    pub const fn the_keyboard(self, now: TheKeyboard) -> Self {
        Self {
            has_the_keyboard: matches!(now, TheKeyboard::IsOnIt),
            ..self
        }
    }

    /// Where things stand when a drag starts or ends.
    #[must_use]
    pub const fn a_drag(self, now: ADrag) -> Self {
        Self {
            dragging: matches!(now, ADrag::IsHappening),
            ..self
        }
    }

    /// Where things stand when one of its menus opens or closes.
    #[must_use]
    pub const fn a_menu(self, now: AMenu) -> Self {
        Self {
            menu_open: matches!(now, AMenu::IsOpen),
            ..self
        }
    }

    /// Dismissed from the keyboard — Escape, or whatever the shell binds.
    ///
    /// **Everything lets go at once.** A dismissal a person asked for is not a
    /// region leaving; it is the answer *no, close it*, and a surface that
    /// stayed open because the pointer happened to be over it would be refusing
    /// an instruction.
    ///
    /// The keyboard goes back to the window only if this surface had it. Asked
    /// of a surface that never held focus it answers [`FocusGoes::Nowhere`],
    /// rather than moving the keyboard on a guess.
    #[must_use]
    pub const fn dismissed_by_the_keyboard(self) -> Dismissed {
        Dismissed {
            revealing: Self::covered(),
            focus: if self.has_the_keyboard {
                FocusGoes::BackToTheWindow
            } else {
                FocusGoes::Nowhere
            },
        }
    }

    /// The window stopped filling the screen, so there is nothing to reveal
    /// over: the surface is simply there.
    #[must_use]
    pub const fn full_screen_ended() -> Self {
        Self::covered()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The edge reveals it**, from anywhere, which is how a person asks.
    #[test]
    fn the_edge_reveals_it() {
        let covered = Revealing::covered();
        assert!(!covered.is_revealed());
        assert!(covered.the_pointer_is(ThePointer::AtTheEdge).is_revealed());
    }

    /// **The pointer can travel onto it without it vanishing on the way.** The
    /// original reason this is a state machine: the obvious version hides the
    /// surface the moment the pointer leaves the strip, which is the moment it
    /// reaches the surface.
    #[test]
    fn the_pointer_can_move_from_the_edge_onto_the_surface_and_it_stays() {
        let revealed = Revealing::covered().the_pointer_is(ThePointer::AtTheEdge);
        let still = revealed.the_pointer_is(ThePointer::OnTheSurface);
        assert!(still.is_revealed(), "it vanished from under the pointer");

        assert!(
            still
                .the_pointer_is(ThePointer::OnTheSurface)
                .the_pointer_is(ThePointer::OnTheSurface)
                .is_revealed(),
            "moving about on it dismissed it"
        );
    }

    /// **Leaving every region hides it**, which is the only thing that does.
    #[test]
    fn leaving_every_region_hides_it() {
        let revealed = Revealing::covered().the_pointer_is(ThePointer::AtTheEdge);
        assert!(!revealed.the_pointer_is(ThePointer::Elsewhere).is_revealed());

        let from_the_surface = revealed.the_pointer_is(ThePointer::OnTheSurface);
        assert!(
            !from_the_surface
                .the_pointer_is(ThePointer::Elsewhere)
                .is_revealed()
        );
    }

    /// **The pointer leaving cannot dismiss it while the keyboard is on it**,
    /// and the keyboard leaving cannot dismiss it while the pointer is.
    ///
    /// *This is the rule the machine got wrong until 2026-09-30, in both
    /// directions: each road concealed regardless of the other, so tabbing to
    /// the surface and then moving the mouse away lost it.*
    #[test]
    fn one_region_leaving_does_not_dismiss_it_while_another_holds_it() {
        let both = Revealing::covered()
            .the_keyboard(TheKeyboard::IsOnIt)
            .the_pointer_is(ThePointer::OnTheSurface);
        assert!(both.is_revealed());

        let pointer_gone = both.the_pointer_is(ThePointer::Elsewhere);
        assert!(
            pointer_gone.is_revealed(),
            "the pointer leaving dismissed it while the keyboard was still on it"
        );

        let keyboard_gone = both.the_keyboard(TheKeyboard::IsElsewhere);
        assert!(
            keyboard_gone.is_revealed(),
            "the keyboard leaving dismissed it while the pointer was still on it"
        );

        assert!(
            !pointer_gone
                .the_keyboard(TheKeyboard::IsElsewhere)
                .is_revealed(),
            "both gone should conceal it"
        );
    }

    /// **A drag holds it open by itself**, because letting go of a window over
    /// a panel that vanished mid-drag is a person losing what they carried.
    #[test]
    fn a_drag_holds_it_open_by_itself() {
        let dragging = Revealing::covered().a_drag(ADrag::IsHappening);
        assert!(dragging.is_revealed());
        assert!(
            dragging.the_pointer_is(ThePointer::Elsewhere).is_revealed(),
            "the pointer leaving dismissed it mid-drag"
        );
        assert!(!dragging.a_drag(ADrag::IsNot).is_revealed());
    }

    /// **An open menu holds it open by itself**, because the pointer is usually
    /// off the surface while using one.
    #[test]
    fn an_open_menu_holds_it_open_by_itself() {
        let menu = Revealing::covered().a_menu(AMenu::IsOpen);
        assert!(menu.is_revealed());
        assert!(
            menu.the_pointer_is(ThePointer::Elsewhere).is_revealed(),
            "the pointer leaving dismissed it while its own menu was open"
        );
        assert!(!menu.a_menu(AMenu::IsNot).is_revealed());
    }

    /// **Dismissing from the keyboard closes it whatever else holds it**, and
    /// gives the keyboard back to the window.
    #[test]
    fn dismissing_from_the_keyboard_closes_it_and_returns_focus() {
        let held = Revealing::covered()
            .the_keyboard(TheKeyboard::IsOnIt)
            .the_pointer_is(ThePointer::OnTheSurface)
            .a_menu(AMenu::IsOpen);
        let after = held.dismissed_by_the_keyboard();
        assert!(
            !after.revealing.is_revealed(),
            "a dismissal a person asked for was refused because the pointer was over it"
        );
        assert_eq!(after.focus, FocusGoes::BackToTheWindow);
    }

    /// **A surface that never held the keyboard sends it nowhere.** Answering
    /// *back to the window* would be moving focus on a guess.
    #[test]
    fn dismissing_something_that_never_had_focus_moves_no_focus() {
        let pointed_at = Revealing::covered().the_pointer_is(ThePointer::AtTheEdge);
        let after = pointed_at.dismissed_by_the_keyboard();
        assert!(!after.revealing.is_revealed());
        assert_eq!(after.focus, FocusGoes::Nowhere);
    }

    /// **Coming back to the edge reveals it again**, however many times.
    #[test]
    fn it_can_be_asked_for_again_and_again() {
        let mut state = Revealing::covered();
        for _ in 0..3 {
            state = state.the_pointer_is(ThePointer::AtTheEdge);
            assert!(state.is_revealed());
            state = state.the_pointer_is(ThePointer::Elsewhere);
            assert!(!state.is_revealed());
        }
    }

    /// Being told the pointer is on a surface nothing holds open answers
    /// *covered* rather than inventing a reveal: there was nothing to be on.
    #[test]
    fn being_on_a_surface_that_is_not_there_reveals_nothing() {
        assert!(
            !Revealing::covered()
                .the_pointer_is(ThePointer::OnTheSurface)
                .is_revealed()
        );
    }

    /// **Two surfaces are two values.** The Dock's reveal and the panel's
    /// cannot move each other, which is what having nothing shared means —
    /// asserted rather than left to be obvious.
    #[test]
    fn two_surfaces_do_not_move_each_other() {
        let dock = Revealing::covered().the_pointer_is(ThePointer::AtTheEdge);
        let panel = Revealing::covered();
        assert!(dock.is_revealed());
        assert!(!panel.is_revealed());

        let panel = panel.the_pointer_is(ThePointer::AtTheEdge);
        let dock = dock.the_pointer_is(ThePointer::Elsewhere);
        assert!(panel.is_revealed(), "dismissing one dismissed the other");
        assert!(!dock.is_revealed());
    }

    /// **Nothing in this file consults a clock.** Held by reading the source
    /// above the tests, the way this repository's other source checks do — a
    /// grace period is the usual patch for the bug this machine does not have,
    /// and it would arrive as an import.
    #[test]
    fn nothing_here_is_a_timer() {
        let source = include_str!("revealing.rs");
        let code = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(above, _)| above);
        assert!(
            code.contains("pub const fn the_pointer_is"),
            "the split took the code away with the tests"
        );
        for forbidden in ["Instant", "Duration", "sleep(", "elapsed(", "timeout"] {
            assert!(
                !code.contains(forbidden),
                "a clock reached this file: {forbidden}"
            );
        }
    }
}
