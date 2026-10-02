//! A click on a preview brings that window back, and the click reaches no client.
//!
//! The panel has been drawable since `#367`, pointable since `#373`, revealable since
//! `#397` and concealable since `#409`, and until this file **nothing could take a
//! window out of it.** `Server::bring_this_window_back` had no production caller at
//! all: the road was built and the only thing that ever drove it was a test.
//!
//! # Why the decision is in two halves
//!
//! Deciding *which* preview was clicked needs the live [`Panel`], because
//! `crate::which_preview_the_pointer_is_on` checks the identity of the window in that
//! slot against the draw. A panel reordered since the draw must name **no** window
//! rather than the one now occupying the slot — and for a click that matters far more
//! than for a peek: a peek shows the wrong preview for one frame, a click brings back
//! the wrong window and a person's arrangement changes under them.
//!
//! Input has no desktop in scope, so it cannot ask that question. What it *can* decide
//! alone is whether the panel claims the click at all, because that is geometry and the
//! geometry is stored on the server by the draw.
//!
//! So the click is **claimed** where input happens and **met** in
//! `crate::direct_desktop`, which is the one place holding both. Exactly the shape
//! `Server::asked_to_put_aside` already uses, in the same direction.
//!
//! # The point of the press, not the pointer afterwards
//!
//! The press records where it happened. A peek asks where the pointer *is*; a click
//! asks where it **was when the button went down**. Reading the live pointer when the
//! ask is met would act on whichever preview the pointer had reached by then, which is
//! a different window from the one the person chose — and a person who clicks and
//! immediately moves their hand is ordinary rather than unusual.
//!
//! # A concealed panel claims nothing, and that falls out rather than being written
//!
//! The claim is *on the rail*, and `crate::panel_raster` gives a concealed panel a rail
//! of no height. So a panel nobody has reached for cannot swallow a click, with no
//! condition here mentioning concealment. A panel that claimed clicks while invisible
//! would be the worst kind of bug in this surface: a person losing clicks to something
//! they cannot see.
//!
//! A click in the reserved column but **off** the rail is deliberately not claimed.
//! There is no preview there to bring back, and swallowing a click to perform nothing
//! would be worse than letting it pass — nothing is drawn under the reserved column
//! anyway, because reserving it is what keeps windows out.

use smithay::backend::input::ButtonState;
use smithay::utils::{Physical, Point};

use alo_dock::WindowId;
use alo_put_aside::Panel;

use crate::which_preview_the_pointer_is_on::OnThePanel;
use alo_dock::revealing::ThePointer;

/// What is to be done with one pointer button, as far as the panel is concerned.
///
/// Three outcomes rather than a `bool`, because taking a press and taking a release are
/// different acts: one records where the person clicked and the other only keeps a
/// client's buttons paired. A `bool` would have let the caller forget which it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheClick {
    /// The panel takes it, and where it happened is to be remembered.
    TakeItAndRememberWhere,
    /// The panel takes it because it took the matching press. Nothing to remember.
    TakeItToKeepTheButtonsPaired,
    /// Not the panel's. Route it on.
    PassItOn,
}

/// Whether the panel takes this button, decided without a seat or a pointer.
///
/// Separate from the event for the same reason `crate::desktop_swipes`' `carry_out` is:
/// extracting a button from libinput needs a device, and deciding what this compositor
/// does about one is a decision of its own that should be answerable without plugging a
/// mouse in. Every one of the four cases below is a test.
///
/// `on` is `None` before the first draw or before the seat has a pointer — both mean *no
/// classification exists*, and a click that cannot be placed is not the panel's.
pub(crate) const fn whether_the_panel_takes_this_click(
    on: Option<ThePointer>,
    state: ButtonState,
    the_press_was_taken: bool,
) -> TheClick {
    match state {
        // **A release is taken on the bookkeeping and never on the geometry.** Asking
        // where the pointer is now would take the release of a press that happened over a
        // window, if the person had moved onto the panel in between — and leave that
        // window's client holding a button for ever.
        ButtonState::Released => {
            if the_press_was_taken {
                TheClick::TakeItToKeepTheButtonsPaired
            } else {
                TheClick::PassItOn
            }
        }
        // **On the rail, and nothing else.** `AtTheEdge` is the reserved column off the
        // rail, where there is no preview to bring back: swallowing a click to perform
        // nothing is worse than letting it pass. And a concealed panel's rail has no
        // height, so it answers `AtTheEdge` for its whole column and takes no clicks —
        // which is why no case here mentions concealment.
        ButtonState::Pressed => match on {
            Some(ThePointer::OnTheSurface) => TheClick::TakeItAndRememberWhere,
            Some(ThePointer::AtTheEdge | ThePointer::Elsewhere) | None => TheClick::PassItOn,
        },
    }
}

impl crate::Server {
    /// Whether the panel took this pointer button, so the caller does not route it on.
    ///
    /// A thin translation of [`whether_the_panel_takes_this_click`] into the two pieces of
    /// state it implies: where the person clicked, and which buttons are the panel's until
    /// they come up. The decision itself holds no state and is tested without a seat.
    ///
    /// Asked from `crate::direct_seat` **after** that function has validated the button and
    /// **before** it routes anything — so a malformed event is still refused by the one
    /// place that refuses them, and a claimed click never reaches a client.
    pub(crate) fn the_panel_took_this_click(&mut self, code: u32, state: ButtonState) -> bool {
        let on = self.what_the_pointer_is_to_the_panel();
        match whether_the_panel_takes_this_click(
            on,
            state,
            self.clicks_the_panel_took.contains(&code),
        ) {
            TheClick::TakeItAndRememberWhere => {
                // **Where it happened, asked once, here.** If the point cannot be had the
                // click is not taken — a claim with nowhere to act is a swallowed click.
                let Some(at) = self.where_the_pointer_is_in_pixels() else {
                    return false;
                };
                self.asked_to_bring_back.push(at);
                self.clicks_the_panel_took.insert(code);
                true
            }
            TheClick::TakeItToKeepTheButtonsPaired => {
                self.clicks_the_panel_took.remove(&code);
                true
            }
            TheClick::PassItOn => false,
        }
    }

    /// Bring back whatever was clicked, and answer how many windows came back.
    ///
    /// Takes the panel because the identity check needs it. `None` — a desktop with no
    /// panel — **drops the asks** rather than keeping them: a click belongs to the panel
    /// that was drawn when it happened, and holding it for a panel that may arrive later
    /// would bring a window back long after the person had moved on.
    pub fn bring_back_what_was_clicked(&mut self, panel: Option<&mut Panel>) -> usize {
        // Taken before anything is attempted, so a click that names no window cannot be
        // retried for ever against a slot that will never answer.
        let asked = std::mem::take(&mut self.asked_to_bring_back);
        if asked.is_empty() {
            return 0;
        }
        let Some(panel) = panel else {
            return 0;
        };

        let chosen: Vec<WindowId> = asked
            .iter()
            .filter_map(|at| self.which_window_was_clicked(panel, *at))
            .collect();

        let mut brought_back = 0;
        for id in chosen {
            // Found and cloned before anything changes: finding borrows the server, and
            // bringing the window back needs it mutably.
            let surface = self.the_put_aside_window_numbered(id);
            if let Some(surface) = surface
                && self.bring_this_window_back(panel, &surface).is_ok()
            {
                brought_back += 1;
            }
        }
        brought_back
    }

    /// The hidden window carrying this number, or `None`.
    ///
    /// **Searched among the hidden windows, not the mapped ones.** A window in the panel
    /// has been hidden, so it is in `minimized_surfaces`; looking among the mapped ones
    /// would answer `None` for every preview and the panel would be a place windows went
    /// to and never came back from.
    ///
    /// **`given_to` rather than `of`.** `Numbers::of` gives a window a number when it has
    /// none, so asking it inside a search would mint one for every surface tested before
    /// the match — a lookup with a side effect on everything it looked at, and on a
    /// machine where the search fails, on everything full stop. `given_to` asks without
    /// answering for anything.
    ///
    /// Public so a test can reach it with a real client's window: the decisions here are
    /// about which collection and which accessor, and neither can be shown against a
    /// surface somebody constructed.
    #[must_use]
    pub fn the_put_aside_window_numbered(
        &self,
        id: WindowId,
    ) -> Option<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface> {
        self.minimized_surfaces()
            .find(|surface| crate::window_number::Numbers::given_to(surface) == Some(id.number()))
            .cloned()
    }

    /// Which window a click at `at` chose, identity-checked against the draw.
    ///
    /// `None` where the click named no preview, or named a slot whose window is no
    /// longer the one drawn there.
    fn which_window_was_clicked(
        &self,
        panel: &Panel,
        at: Point<i32, Physical>,
    ) -> Option<WindowId> {
        let drawn = self.panel_as_drawn.as_ref()?;
        match crate::which_preview_the_pointer_is_on::which_preview_the_pointer_is_on(
            panel, drawn, at,
        ) {
            OnThePanel::APreview(id) => Some(id),
            OnThePanel::TheRegionButNoPreview | OnThePanel::Elsewhere => None,
        }
    }
}

#[cfg(test)]
#[path = "a_click_brings_a_window_back_tests.rs"]
mod tests;
