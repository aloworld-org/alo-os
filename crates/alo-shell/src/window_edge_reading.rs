//! What a screen reader is told about a window's edge, and what a keyboard
//! reaches on it.
//!
//! # Ported before the old implementation is deleted, which was the order
//!
//! The owner's direction of 2026-10-09: *port accessibility before deleting
//! its implementation. Preserve accessible names, roles, available actions,
//! focus behaviour and state announcements.* So this exists while
//! `window_control_reader_*` still does, and that one goes only once this
//! answers for the edge.
//!
//! # The names are not written here, and that is the point
//!
//! `alo_access::Control::for_action` builds a control whose name **is** the
//! action's own word. Its own note says why that matters: *the words a reader
//! says are the words drawn on the control, in every language, because there
//! is one string rather than two that must agree* — which satisfies EN 301 549
//! clause 11.2.5.3 by construction rather than by review.
//!
//! `alo_shortcuts::Action` already carries `MinimiseWindow`, `MaximiseWindow`
//! and `CloseWindow` with their words, and the owner asked to *reuse shared
//! window actions — close, minimise, maximise and restore — where they remain
//! valid*. They remain valid: the edge moved where a control is, not what
//! pressing it does.
//!
//! # Reachable without hovering
//!
//! > Screen-reader and keyboard users must reach the new edge **without first
//! > hovering**.
//!
//! That clause is the one a mouse-shaped implementation fails quietly, and
//! this file failed it on the first attempt: [`what_a_reader_is_told`] read
//! the drawn controls off an [`WindowEdge`], and a concealed edge has none —
//! correctly, because nothing is drawn at rest. A keyboard user would have
//! been unable to reach a control until a pointer had been near it.
//!
//! **So it asks what a window has, not what is drawn.** It takes
//! [`Decorations`] and no geometry at all, because where a control sits is a
//! drawing question and whether it exists is not. **Revealing is what a
//! person sees; it is never what a person can reach.**

use alo_access::tree::Control;
use alo_shortcuts::Action;

use crate::window_edge::{Decorations, OnTheEdge};

/// What pressing this does, as an action the rest of the machine already has.
///
/// **Every control has one, from 2026-10-10.** The window menu answered [`None`]
/// until then — recorded rather than given a fabricated name, because a reader
/// announcing a menu it could only say in English would be worse than one that
/// does not announce it. The owner's ruling of that date made the honest fix
/// necessary rather than merely owed: the menu is where a long title is
/// selectable, so a control nobody can reach is a title nobody can read.
///
/// `Action::WindowOptions` is named for what it does and not for the box it
/// opens (ADR 0089), its chord is `Alt+Space` because that is what every
/// desktop a person arrives from already spells it, and its sentence is in the
/// shared vocabulary, so a reader says it in the person's own language like
/// every other control here.
#[must_use]
pub const fn what_it_does(control: OnTheEdge) -> Option<Action> {
    Some(match control {
        OnTheEdge::Minimise => Action::MinimiseWindow,
        OnTheEdge::Maximise => Action::MaximiseWindow,
        OnTheEdge::Close => Action::CloseWindow,
        OnTheEdge::Menu => Action::WindowOptions,
    })
}

/// Every control a window with these decorations has, in the order a reader
/// meets them.
///
/// Left to right, which is the order they are drawn in and the order a
/// keyboard walks them — one order, so what a person hears and what the
/// keyboard does cannot disagree.
///
/// **Takes no geometry, deliberately.** Where a control sits is a drawing
/// question; whether it exists is not. An earlier version read the drawn
/// controls off an `WindowEdge` and returned nothing for a concealed one, which is
/// the *without first hovering* clause failing — see this file's header.
#[must_use]
pub fn what_a_reader_is_told(decorations: Decorations) -> Vec<Control> {
    the_controls_of(decorations)
        .into_iter()
        .filter_map(what_it_does)
        .map(Control::for_action)
        .collect()
}

/// Which controls a window with these decorations has at all.
///
/// The same answer `crate::window_edge` lays out when the edge is revealed,
/// and the reason it is here rather than read off an `WindowEdge`: a concealed edge
/// draws none of them and a person can still reach every one.
#[must_use]
pub fn the_controls_of(decorations: Decorations) -> Vec<OnTheEdge> {
    match decorations {
        Decorations::TheShellDraws => {
            vec![OnTheEdge::Minimise, OnTheEdge::Maximise, OnTheEdge::Close]
        }
        Decorations::TheApplicationDraws => vec![OnTheEdge::Menu],
    }
}

#[cfg(test)]
#[path = "window_edge_reading_tests.rs"]
mod tests;
