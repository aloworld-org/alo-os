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

use crate::window_edge::{Decorations, HowItFits, OnTheEdge};

/// What pressing this does, as an action the rest of the machine already has.
///
/// [`None`] for the window menu, which has no [`Action`] yet — recorded rather
/// than given a fabricated one. A reader that announced a menu it could not
/// name, or named it in English only, would be worse than one that does not
/// announce it, and the honest fix is an action with a word like every other.
#[must_use]
pub fn what_it_does(control: OnTheEdge) -> Option<Action> {
    match control {
        OnTheEdge::Minimise => Some(Action::MinimiseWindow),
        OnTheEdge::Maximise => Some(Action::MaximiseWindow),
        OnTheEdge::Close => Some(Action::CloseWindow),
        OnTheEdge::Menu => None,
        // **Not an action, and for a different reason than Menu's.** Menu has
        // none yet and should; Move is a *drag*, and a drag is an interaction
        // rather than one of `alo_shortcuts`' discrete actions. Pressing it
        // begins something that ends when the pointer is let go.
        //
        // The design also forbids inventing one: *do not guess a keyboard
        // chord: use the project's assigned window-menu action* — and this
        // project has no window-menu, move-window or resize-window action at
        // all, measured 2026-10-11 against `alo_shortcuts::Action`. Three of
        // the menu's five rows therefore have no action behind them yet, which
        // is recorded in `docs/autonomy/the-external-window-edge-plan.md`
        // rather than papered over here.
        OnTheEdge::Move => None,
    }
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
    // **Width-independent, and that is the design's answer rather than a
    // simplification.** `the_controls_of` below takes a `HowItFits` because
    // which controls are *drawn on the edge* depends on how much room there
    // is. What a window can *do* does not: the compact layout moves Minimise
    // and Maximise into the window menu and keeps Close on the edge, so the
    // contract's *no action is silently dropped* and *pointer, touch, keyboard
    // and screen-reader users must reach the same actions* both hold at every
    // width.
    //
    // **So a reader hears the same list at 80 pixels as at 800.** This was
    // argued the other way first - that a reader announcing Close on a narrow
    // window names something that is not there - and it is the right argument
    // about the wrong layout: Close is the one control the design guarantees
    // stays on the edge at every width, and the two that move are reachable
    // through the menu. Announcing fewer actions on a narrow window would be
    // the silent drop the contract forbids.
    //
    // The caller has no width to pass in any case: `crate::access_nodes`
    // walks `&[FrameName]`, which carries names and no geometry. That is a
    // consequence of this being the right answer and not the reason for it.
    let mut told: Vec<Control> = the_controls_of(decorations, HowItFits::Full)
        .into_iter()
        .filter_map(what_it_does)
        .map(Control::for_action)
        .collect();

    // **The two the window menu offers and the edge never did.** Until the
    // menu existed a reader was told three things a window could do, and a
    // window could do five — moving and resizing were reachable by dragging a
    // border and by nothing a reader could find.
    //
    // Named from `alo_access`' own words rather than from an action's, because
    // there is no action: `alo_shortcuts` has neither, and the design forbids
    // inventing a chord for one. `Control::for_action` is therefore not the
    // road, and these are built with their name directly.
    //
    // **Appended rather than interleaved**, so the three a person can press on
    // the edge come first in the order they are drawn and the two that are only
    // in the menu follow. One order, as this file's header says, and this is
    // still it: what the edge draws, then what the menu adds.
    // **Only where the shell draws the decorations.** An application with its
    // own header gets movement and a menu from alo and keeps its own buttons,
    // and what that menu holds is a question the compact design does not
    // answer: it draws one menu, with five rows, beside a shell-drawn edge.
    // Announcing Move and Resize over an app-drawn window would be deciding
    // that its menu offers them, which is the owner's to decide and not
    // mine - so this leaves that case exactly as it was, announcing nothing,
    // and the question is recorded rather than answered.
    let also = if decorations == Decorations::TheShellDraws {
        &[
            alo_access::words::MOVE_THE_WINDOW,
            alo_access::words::RESIZE_THE_WINDOW,
        ][..]
    } else {
        &[][..]
    };
    for name in also.iter().copied() {
        told.push(Control {
            role: alo_access::Role::Button,
            name,
            state: alo_access::State::CanBeUsed,
            setting: None,
            does: None,
        });
    }
    told
}
/// Which controls a window with these decorations has at all.
///
/// The same answer `crate::window_edge` lays out when the edge is revealed,
/// and the reason it is here rather than read off an `WindowEdge`: a concealed edge
/// draws none of them and a person can still reach every one.
#[must_use]
pub fn the_controls_of(decorations: Decorations, fits: HowItFits) -> Vec<OnTheEdge> {
    match (decorations, fits) {
        (Decorations::TheShellDraws, HowItFits::Full) => vec![
            OnTheEdge::Move,
            OnTheEdge::Minimise,
            OnTheEdge::Maximise,
            OnTheEdge::Close,
        ],
        // **Close stays, and the two that leave are not lost.** The contract:
        // *keep Close directly available whenever the edge's controls are
        // revealed*, and *put the remaining actions in the window menu*. So a
        // narrow window has Minimise and Maximise through Menu rather than not
        // at all, which is why this is a different layout and not a shorter
        // list of what a window can do.
        (Decorations::TheShellDraws, HowItFits::Compact) => {
            vec![OnTheEdge::Move, OnTheEdge::Menu, OnTheEdge::Close]
        }
        // **No Close, at either width.** The owner's ruling, which the compact
        // contract repeats: *application-owned title bars retain their own
        // controls; do not add duplicate Close buttons to them.* alo adds
        // movement and a menu and nothing else, so the width makes no
        // difference here — there is nothing to drop.
        (Decorations::TheApplicationDraws, _) => vec![OnTheEdge::Move, OnTheEdge::Menu],
        // **Close only**, at either width: minimise and maximise wait for the
        // right-hand panel to hold a task, and a task window does not move.
        (Decorations::ATaskWindow, _) => vec![OnTheEdge::Close],
    }
}

#[cfg(test)]
#[path = "window_edge_reading_tests.rs"]
mod tests;
