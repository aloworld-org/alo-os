//! Bringing a window back, and the travel that goes with it.
//!
//! One click returns a window to the view it was put aside from, and **the canvas travels
//! to show it** — not to wherever the person happens to be looking now. Task 3 of
//! `docs/autonomy/putting-a-window-aside.md`.
//!
//! # The restore names the saved view, and the test proves it by disagreeing
//!
//! *Restore it where it was* and *restore it where I am* give the same answer whenever the
//! window happens to be on screen already, which is most of the time and is why this is
//! worth a file. The tests pull the two apart on purpose: they restore from a view that
//! does not contain the window at all, so an implementation that returned the current view
//! fails rather than passing by luck.
//!
//! # Nothing here touches a camera, and that is on purpose
//!
//! This file decides **whether a travel is needed and what it would be**. It does not
//! perform one. The caller — which has a camera — moves it.
//!
//! That is not squeamishness about a dependency. `alo-canvas`'s rule is that nothing in the
//! viewport layer may read the camera to correct itself, and the panel is a viewport
//! surface; `tests/the_panel_never_reaches_the_camera.rs` is what holds it. A version of
//! this file that took a `Camera` and moved it would be a viewport surface driving the
//! plane, and it would pass any test that only checked where the window ended up.
//!
//! What it takes instead is an [`alo_dock::on_the_canvas::TheView`] — the part of the plane
//! being shown. A view is not a camera: it says what is visible, not who decided.
//!
//! # Already shown is strict, and stays strict
//!
//! [`TheView::already_shows`] is *wholly inside*, not *touching*. Its own file gives the
//! reason: a window with one corner on screen is a window a person cannot read, so *travel
//! only if needed* means needed to see the thing rather than needed to see evidence of it.
//!
//! The cost of that reading is a travel for a window that is nearly there. The cost of the
//! loose one is a click that appears to do nothing, which is worse — a person who clicks a
//! preview and sees no change does not conclude that the window was already visible, they
//! conclude the click was lost.

use alo_canvas::Place;
use alo_dock::on_the_canvas::TheView;
use alo_dock::window::WindowId;
use alo_dock::windows::Windows;

use crate::panel::{NotPutAside, Panel};
use crate::putting_aside::bring_back;
use crate::where_it_goes_back::WhereItGoesBack;

/// What the canvas has to do for a restored window to be readable.
///
/// **A named case rather than an [`Option`].** *No travel is needed* and *no answer* are
/// different things, and an `Option<WhereItGoesBack>` spells them the same way — so a
/// caller that forgot to handle `None` would silently do nothing in the one case where
/// doing nothing is correct, and be right by accident until the day it was not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Travel {
    /// The view already shows the window whole. Moving would move it for nothing.
    NotNeeded,
    /// The canvas must go here: the patch the window occupies, at the zoom it was saved at.
    To(WhereItGoesBack),
    /// **The window is on another Place**, so the canvas has to go to that Place first.
    ///
    /// A third case rather than a flag on [`Self::To`], because what a caller does about it is
    /// a different act: `To` is a pan and a zoom on the surface already shown, and this is
    /// leaving that surface. A caller that treated them alike would pan to `(4200, 0)` on the
    /// wrong Place — which is not a near miss, it is a window somewhere else, and the view
    /// would look correct.
    ///
    /// The saved view travels with it, so the caller does not have to ask twice: go to that
    /// Place, then to this patch at this zoom.
    ToAnotherPlace(WhereItGoesBack),
}

impl Travel {
    /// Where the canvas has to go, if anywhere.
    #[must_use]
    pub const fn to(self) -> Option<WhereItGoesBack> {
        match self {
            Self::NotNeeded => None,
            Self::To(there) | Self::ToAnotherPlace(there) => Some(there),
        }
    }

    /// Whether the canvas has to move at all.
    #[must_use]
    pub const fn is_needed(self) -> bool {
        matches!(self, Self::To(_) | Self::ToAnotherPlace(_))
    }

    /// Whether the canvas has to leave the Place it is on.
    ///
    /// Asked separately because it is a different act from panning, and a caller that only
    /// asked `is_needed` would pan on the surface it was already looking at.
    #[must_use]
    pub const fn leaves_this_place(self) -> bool {
        matches!(self, Self::ToAnotherPlace(_))
    }
}

/// Restore a window from the panel, and say what travel that needs.
///
/// The window comes back to the view it was put aside from. The travel is decided against
/// **that saved view**, never against `showing` — which is the whole of this task's
/// acceptance, and the reason `showing` is taken as an argument rather than consulted for
/// an answer.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] if the panel does not hold that window. **Nothing changes
/// on a refusal**, including the answer about travelling: a refusal that also reported
/// [`Travel::NotNeeded`] would hand a caller something it could act on.
pub fn restore(
    windows: &mut Windows,
    panel: &mut Panel,
    id: WindowId,
    showing: TheView,
    looking_at: Place,
) -> Result<Travel, NotPutAside> {
    let goes_back_to = bring_back(windows, panel, id)?;
    Ok(travel_for(goes_back_to, showing, looking_at))
}

/// The travel a saved view needs, given what is on screen and which Place that is.
///
/// Its own function because it is the decision, and because it is worth being able to ask
/// it without moving a window — a caller drawing a preview may want to say *this one is
/// over there* before anybody clicks.
///
/// # The Place is asked first, and the order is the correctness
///
/// `already_shows` compares rectangles, and **a rectangle does not know which surface it is
/// on**. `(4200, 0)` exists on every Place, so a view showing that patch on Place 2 answers
/// *yes, already shown* about a window saved on Place 7 — and the caller then does nothing,
/// having been told the window is visible when it is not even on this surface.
///
/// That is the worst shape of wrong answer available here: it is not a travel to the wrong
/// place, it is **a confident report that no travel is needed**, and the screen agrees
/// because something else is in that rectangle. So the Place is settled before the geometry
/// is consulted at all.
#[must_use]
pub fn travel_for(goes_back_to: WhereItGoesBack, showing: TheView, looking_at: Place) -> Travel {
    if !goes_back_to.is_on(looking_at) {
        return Travel::ToAnotherPlace(goes_back_to);
    }
    if showing.already_shows(goes_back_to.at()) {
        Travel::NotNeeded
    } else {
        Travel::To(goes_back_to)
    }
}
