//! Restoring when the saved place may be taken, and placing once the person has answered.
//!
//! [`crate::restoring`] answers *where does the canvas go*. This answers the question that
//! comes first: **is the place still free**, and if it is not, what is put in front of the
//! person instead.
//!
//! # Why this is not a branch inside `restore`
//!
//! Because the two outcomes differ in what they have already done. A free place means the
//! window is back: the panel has given it up and its state has changed. A taken place means
//! **nothing has happened yet** — the panel still holds the preview, the window still says
//! it is put aside, and the desk is exactly as it was.
//!
//! *Nothing is rearranged without the proposal being visible first* cannot be kept by a
//! function that has already restored the window and is deciding where to put it. By then
//! the rearranging has begun; what is left is choosing how much. So the collision is
//! checked **before** the panel is touched, and this file is where that ordering lives —
//! the same discipline as `putting_aside`, where the panel accepts a preview before the
//! window's state changes, and for the same reason: the order is the guarantee.
//!
//! # It is told what is in the way
//!
//! `occupied_by` is passed in. This crate cannot ask which windows overlap a patch —
//! `alo-dock`'s `Patch::wholly_inside` is private, *overlaps* is a different question, and
//! computing geometry about another crate's type here would be a second opinion that agrees
//! today and drifts later. Asked of the lane that owns that crate.
//!
//! The caller that holds the [`Windows`] is also the one that knows what is on screen, so
//! it is the right one to ask anyway — exactly as `restoring` is handed a view rather than
//! reaching for a camera.

use alo_dock::on_the_canvas::{Patch, TheView};
use alo_dock::window::{HowItSits, WindowId};
use alo_dock::windows::Windows;

use crate::panel::{NotPutAside, Panel};
use crate::proposing::Proposal;
use crate::restoring::{Travel, travel_for};
use crate::shown::Shown;

/// What happened when a window was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restored {
    /// The place was free. **The window is back**, and this is the travel it needs.
    Travelled(Travel),
    /// The place was taken. **Nothing has moved**, and this is what to show the person.
    Proposed(Proposal),
}

/// Ask for a window back, knowing what now occupies its saved place.
///
/// `occupied_by` is the window in the way, or [`None`] if the place is free. A caller that
/// found several passes the one it means to name — **a person shown *something is here*
/// learns less than a person shown which window**, and this surface names one thing so that
/// it can be pointed at.
///
/// # What is true afterwards
///
/// On [`Restored::Travelled`], the window is on the canvas and the panel has let it go.
/// On [`Restored::Proposed`], **the desk is untouched**: the panel still holds the preview
/// and the window still says it is put aside. That is the half of *nothing is rearranged
/// without the proposal being visible first* which a crate can hold.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] if the panel does not hold that window, and then nothing
/// changes and **nothing is proposed either**. A refusal that also handed back a proposal
/// would be a refusal a caller could act on.
pub fn ask_for(
    windows: &mut Windows,
    panel: &mut Panel,
    id: WindowId,
    showing: TheView,
    occupied_by: Option<WindowId>,
    offer: Patch,
) -> Result<Restored, NotPutAside> {
    // **Read before writing.** The preview is looked at rather than taken, so that the
    // taken-place road can return having changed nothing at all.
    let preview = panel
        .previews()
        .iter()
        .find(|preview| preview.window() == id)
        .ok_or(NotPutAside::ItIsNotThere)?;
    let goes_back_to = preview.where_it_goes_back();

    if let Some(in_the_way) = occupied_by {
        return Ok(Restored::Proposed(Proposal::of(
            id,
            goes_back_to.at(),
            offer,
            in_the_way,
        )));
    }

    // The place is free, so this is an ordinary restore and the window comes back.
    panel.bring_back(id)?;
    windows.now_sits(id, HowItSits::OnTheCanvas);
    Ok(Restored::Travelled(travel_for(goes_back_to, showing)))
}

/// Accept the offered position: the window comes back there.
///
/// Takes the [`Shown`] token **by value**, so one showing authorises one placement. A
/// caller cannot accept a proposal it never displayed, because there is no way to make a
/// token except by saying which view it was drawn in — which is what stops *nothing is
/// rearranged without the proposal being visible first* from being a convention.
///
/// The saved position is returned alongside the placement, because it is what History is
/// owed and **accepting must not be what loses it**.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] if the panel no longer holds that window — which can
/// happen honestly, between showing a proposal and a person answering it.
pub fn accept(
    windows: &mut Windows,
    panel: &mut Panel,
    proposal: Proposal,
    shown: Shown,
) -> Result<Placed, NotPutAside> {
    place(windows, panel, proposal, shown, proposal.offered())
}

/// Put the window where the person dragged it instead.
///
/// The other road the specification offers, and it takes the same token: a drag is still an
/// answer to something that was shown. The saved position survives this road too.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] if the panel no longer holds that window.
pub fn dragged_to(
    windows: &mut Windows,
    panel: &mut Panel,
    proposal: Proposal,
    shown: Shown,
    where_they_put_it: Patch,
) -> Result<Placed, NotPutAside> {
    place(windows, panel, proposal, shown, where_they_put_it)
}

/// Where a window ended up, and where it used to be.
///
/// Both, always. *Keep the original position in History* is part of this task's acceptance
/// and History is a `[v1]` feature while this panel is `[v0.01]`, so the surface is not
/// built here — but **the value is carried**, so that when History exists it has something
/// true to read rather than a position somebody would have to reconstruct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    /// Where the window is now.
    at: Patch,
    /// Where it was put aside from.
    was_at: Patch,
}

impl Placed {
    /// What happened, from both values at once.
    ///
    /// **Deliberately not `pub`.** A `Placed` is evidence of a placement that took place, not a
    /// claim a caller can make: the only way to obtain one is to have placed a window through
    /// [`accept`] or [`dragged_to`], each of which spends a [`Shown`]. A public constructor
    /// would let a caller assemble the report without the event, which is the fault *a proposal
    /// accepted without having been shown* one step further along.
    ///
    /// It is a function rather than a struct literal because the check in
    /// `tests/the_types_that_carry_a_guarantee_have_one_way_in.rs` counts ways in, and a type
    /// built by a literal has none to count. That check reported **zero** for this type, which
    /// was right about the code and wrong about the guarantee — so the code moved to match what
    /// the other two types do, rather than the check being taught a second shape to accept. A
    /// harness that grows a special case per subject stops being one rule.
    const fn of(at: Patch, was_at: Patch) -> Self {
        Self { at, was_at }
    }

    /// Where the window is now.
    #[must_use]
    pub const fn at(self) -> Patch {
        self.at
    }

    /// Where it was put aside from — the value History is owed.
    #[must_use]
    pub const fn was_at(self) -> Patch {
        self.was_at
    }

    /// Whether it ended up somewhere other than where it was.
    #[must_use]
    pub fn moved(self) -> bool {
        self.at != self.was_at
    }
}

/// The one road both answers take, so that accepting and dragging cannot drift apart.
///
/// `_shown` is consumed and not read. **That is the whole of its job**: its existence is
/// the evidence, and taking it by value spends it. A version of this that inspected it
/// would be looking for something to check, when what is being checked is that a caller had
/// to have one.
fn place(
    windows: &mut Windows,
    panel: &mut Panel,
    proposal: Proposal,
    _shown: Shown,
    at: Patch,
) -> Result<Placed, NotPutAside> {
    let id = proposal.window();
    panel.bring_back(id)?;
    windows.now_sits(id, HowItSits::OnTheCanvas);
    Ok(Placed::of(at, proposal.saved()))
}
