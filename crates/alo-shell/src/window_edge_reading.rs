//! What a screen reader is told about a window's band, and what a keyboard
//! reaches on it.
//!
//! # The names are the specification's
//!
//! `docs/design/the-window-controls.md`: the three controls are called
//! **Window actions**, **Minimise** and **Close** — as a tooltip, as the name a
//! reader speaks, everywhere. The words are `alo-access`'s, because
//! `alo-saying` collects that crate and not this one: a word declared here would
//! be a string a person sees and no translator can reach.
//!
//! # Reachable without hovering
//!
//! > Screen-reader and keyboard users must reach the new edge **without first
//! > hovering**.
//!
//! So this asks what a window **has**, not what is drawn. It takes
//! [`Decorations`] and no geometry: where a control sits is a drawing question,
//! whether it exists is not. **Revealing is what a person sees; it is never what
//! a person can reach.**

use alo_access::tree::Control;
use alo_shortcuts::Action;

use crate::window_edge::{Decorations, OnTheEdge};

/// What pressing this does, as an action the rest of the machine already has.
///
/// Window actions is [`Action::WindowOptions`] — *Show the window's options*,
/// `Alt+F3` — because pressing it and pressing that chord open the same panel.
#[must_use]
pub const fn what_it_does(control: OnTheEdge) -> Action {
    match control {
        OnTheEdge::WindowActions => Action::WindowOptions,
        OnTheEdge::Minimise => Action::MinimiseWindow,
        OnTheEdge::Close => Action::CloseWindow,
    }
}

/// What this control is called, for a tooltip and for a reader.
#[must_use]
pub const fn what_it_is_called(control: OnTheEdge) -> alo_strings::Word {
    match control {
        OnTheEdge::WindowActions => alo_access::words::WINDOW_ACTIONS,
        OnTheEdge::Minimise => alo_access::words::MINIMISE,
        OnTheEdge::Close => alo_access::words::CLOSE,
    }
}

/// Every control a window with these decorations has, in the order a reader
/// meets them — left to right, the order they are drawn and the order the
/// keyboard walks them. One order, so what a person hears and what the
/// keyboard does cannot disagree.
///
/// **Window actions opens something of its own**, so it is told as a control
/// that opens; whether its panel is open is the caller's to say, because it is
/// a fact about this moment and not about the window.
#[must_use]
pub fn what_a_reader_is_told(decorations: Decorations) -> Vec<Control> {
    the_controls_of(decorations)
        .into_iter()
        .map(|control| Control {
            role: alo_access::Role::Button,
            name: what_it_is_called(control),
            state: if control == OnTheEdge::WindowActions {
                alo_access::State::OpensSomething
            } else {
                alo_access::State::CanBeUsed
            },
            setting: None,
            // **What pressing it does**, so a reader's press reaches the same
            // action a pointer's does — the task window's Close among them.
            does: Some(what_it_does(control)),
        })
        .collect()
}

/// Which controls a window with these decorations has at all.
///
/// The same answer `crate::window_edge` lays out when the band is revealed,
/// and the reason it is here rather than read off a `WindowEdge`: a concealed
/// band draws none of them and a person can still reach every one.
#[must_use]
pub fn the_controls_of(decorations: Decorations) -> Vec<OnTheEdge> {
    match decorations {
        Decorations::TheShellDraws => vec![
            OnTheEdge::WindowActions,
            OnTheEdge::Minimise,
            OnTheEdge::Close,
        ],
        // **Window actions, and only what the application does not show.**
        // The owner, 2026-11-12: *No need to add controls if the app has, we
        // can only add the ones that are not there.* No application draws
        // alo's panel, so Window actions is always added; Minimise and Close
        // are added where the application has none of its own, and never a
        // second one beside its own.
        Decorations::TheApplicationDraws(its_own) => [
            Some(OnTheEdge::WindowActions),
            (!its_own.minimise).then_some(OnTheEdge::Minimise),
            (!its_own.close).then_some(OnTheEdge::Close),
        ]
        .into_iter()
        .flatten()
        .collect(),
        // **Minimise and Close** on a task window: minimising puts the task in
        // the right-hand panel, still running (`crate::a_task_goes_aside`), the
        // owner's *minimise later with the right-hand panel* of 2026-10-28.
        // It does not move, so there are no window actions.
        Decorations::ATaskWindow => vec![OnTheEdge::Minimise, OnTheEdge::Close],
        // **Minimise and Close** on alo's own windows: minimising puts one in
        // the right-hand panel like any window (design `27:2`,
        // `crate::alos_own_window_goes_aside`). They do not maximise yet, and a
        // control that reaches nothing would look live.
        Decorations::AlosOwnWindow => vec![OnTheEdge::Minimise, OnTheEdge::Close],
    }
}

/// Every word a reader is told on a band, for the vocabulary's count.
#[must_use]
pub fn every_word_this_band_says() -> Vec<alo_strings::Word> {
    [
        OnTheEdge::WindowActions,
        OnTheEdge::Minimise,
        OnTheEdge::Close,
    ]
    .into_iter()
    .map(what_it_is_called)
    .collect()
}

#[cfg(test)]
#[path = "window_edge_reading_tests.rs"]
mod tests;
