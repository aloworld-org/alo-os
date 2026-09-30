//! Looking at a window that is put aside, without bringing it back.
//!
//! Task 5 of `docs/autonomy/putting-a-window-aside.md`. A larger readable view over the
//! current canvas; releasing or Escape removes it and **the window stays minimised**.
//!
//! # The state machine is `alo-dock`'s, and that was checked rather than assumed
//!
//! [`alo_dock::Peeking`] already is this: a window looked at while the canvas stays exactly
//! where it is, ended by letting go or by choosing it. Writing a second one would be the
//! fault the plan names — *writing the second one first is how there come to be two*.
//!
//! It was read for **specificity on an axis its header does not advertise**, because that is
//! what went wrong with `revealing`: that module was general about geometry, said so, and was
//! specific about how many things could hold it open without mentioning it. `Peeking` holds
//! one window, which is right for a peek rather than a limit; does not ask how the peek began,
//! and says so loudly, because the keyboard road has to end somewhere identical; and **holds
//! no view at all**, which is how *the canvas stays exactly where it is* is kept.
//!
//! # The camera is unchanged because there is nothing here to change it with
//!
//! The acceptance asks for the window's state and the camera to be unchanged, **asserted
//! separately**. The first is a fact about this crate's own data and is tested.
//!
//! The second is held by the shape: [`peek_at`] and [`stop_peeking`] take no view and return
//! none, so no caller can be handed a reason to move. A test asserting *the view I held before
//! equals the view I hold after* could not fail, and this crate spent today learning what a
//! check that cannot fail costs — it lends unearned credit to the checks beside it. So the
//! promise is stated here and guarded by
//! `tests/a_peek_leaves_the_window_and_the_canvas_alone.rs`, which reads this file's source
//! for a [`crate::restoring::Travel`] it must never build. **That can fail.**
//!
//! # Choosing a peeked preview is a restore, not a travel
//!
//! [`alo_dock::PeekEnded::ByTravellingThere`] names the ordinary road for a window on the
//! canvas, and for one that is put aside the ordinary road is a **restore** — which since task
//! 4 can come back with a proposal, because the saved place may be taken.
//!
//! So a reader who follows that variant's name and performs a travel **skips the collision
//! proposal**, putting the window back invisibly behind whatever is there: exactly what task 4
//! exists to prevent, reachable through a variant name. Not a complaint about the name, whose
//! meaning — *the person chose it, do the ordinary thing* — is general. It is the reason
//! [`chosen`] exists and is tested: the wrong reading fails here rather than in the shell.

use alo_dock::on_the_canvas::{Patch, TheView};
use alo_dock::window::WindowId;
use alo_dock::windows::Windows;
use alo_dock::{PeekEnded, Peeking};

use crate::panel::{NotPutAside, Panel};
use crate::restoring_into_a_taken_place::{Restored, ask_for};

/// Begin peeking at a window the panel holds.
///
/// **Changes nothing.** Not the window, not the panel, not the view. A peek is a way of
/// looking, and a look that altered what it looked at would be the thing this surface is for
/// avoiding.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] if the panel does not hold that window. A peek at nothing is
/// refused rather than begun-and-empty, because [`Peeking::is_peeking`] would then answer
/// *yes* about a window nobody can see.
pub fn peek_at(panel: &Panel, id: WindowId) -> Result<Peeking, NotPutAside> {
    if panel.holds(id) {
        Ok(Peeking::begun(id))
    } else {
        Err(NotPutAside::ItIsNotThere)
    }
}

/// Let go, or Escape: the peek ends and **the window stays minimised**.
///
/// The owner's sentence, and it is kept by this function having no way to do anything else —
/// it takes no [`Windows`] and no [`Panel`], so it cannot move a window or empty the panel
/// even by mistake.
#[must_use]
pub fn stop_peeking(peeking: Peeking) -> Peeking {
    let (PeekEnded::AndNothingMoved | PeekEnded::ByTravellingThere(_), after) = peeking.let_go();
    after
}

/// The person chose the preview they were peeking at, so it comes back properly.
///
/// **Through the restore road, collision proposal and all** — not through a travel, however the
/// variant that reports the choice is spelled. See this module's header: following that name
/// would skip the proposal and put the window back invisibly behind whatever now occupies its
/// place.
///
/// Returns the peek afterwards — always [`Peeking::at_nothing`] — beside what the restore
/// decided, so a caller cannot forget to end the peek it just acted on.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] if nothing was being peeked at, or if the panel no longer
/// holds it. Both leave the desk untouched.
pub fn chosen(
    windows: &mut Windows,
    panel: &mut Panel,
    peeking: Peeking,
    showing: TheView,
    occupied_by: Option<WindowId>,
    offer: Patch,
) -> Result<(Restored, Peeking), NotPutAside> {
    let (ended, after) = peeking.choose_it();
    let Some(PeekEnded::ByTravellingThere(id)) = ended else {
        return Err(NotPutAside::ItIsNotThere);
    };
    let what = ask_for(windows, panel, id, showing, occupied_by, offer)?;
    Ok((what, after))
}
