//! The move itself: a window leaving the canvas for the panel, and coming back.
//!
//! Task 1 built the panel. This is the **transition** — the part where a window stops
//! being on the canvas and starts being in the panel, and the part a person actually
//! performs.
//!
//! # Two things change and nothing else does
//!
//! The window's state becomes [`HowItSits::PutAside`], and the panel gains a preview.
//! **Every other window is untouched**, which is the owner's *the other windows do not
//! move to fill the gap*: a canvas that closed up after a minimise would move things a
//! person had placed, and placing is the whole of what a canvas is for.
//!
//! That is held by this file taking the window it was asked for and no other — and
//! tested by comparing every other window before and after, which is a test whose
//! subject is an absence and the only kind that can hold *nothing else happened*.
//!
//! # Nothing is discarded, because nothing is thrown away to begin with
//!
//! *Minimising never discards work* is not a promise this module keeps by being careful.
//! **The window is not destroyed and its patch is not recomputed** — it changes one
//! field and is handed a preview that already holds where it was. A round trip returns
//! the same patch because it is the same patch, not because two calculations agreed.
//!
//! # What this cannot do yet, and will not pretend to
//!
//! **The Place.** A patch alone is ambiguous: `(4200, 0)` exists on every surface, so
//! *which surface* is part of what putting a window aside has to save. There is no
//! canvas Place to save. That clause stays in task 2 of
//! `docs/autonomy/putting-a-window-aside.md` and the task does not close without it.
//!
//! That is the only clause still owed. **The zoom is done**, and how it got done is worth
//! keeping because the first answer written here was wrong.
//!
//! It said the zoom could not be taken: `alo_canvas::Zoom` is the right type, holding a
//! zoom a caller hands in is not reading the camera, but `Zoom` is exported beside
//! `Camera`, so depending on `alo-canvas` would put a `Camera` in reach — and it proposed
//! moving `Zoom` to a module of its own so that the dependency could be taken safely.
//! **Rust's privacy boundary is the crate, not the module.** `alo-canvas` declares `pub
//! mod camera`, so `alo_canvas::camera::Camera` is nameable here the moment the
//! dependency exists, whatever path `Zoom` comes by. The split would have bought a tidier
//! import and no boundary at all, which is worse than no boundary because it is the one
//! nobody keeps checking. Two other lanes measured that crate's `lib.rs` rather than
//! reasoning about it, and the crate's owner then decided: take the dependency, hold the
//! rule with a test.
//!
//! So the rule lives in `tests/the_panel_never_reaches_the_camera.rs`, which reads this
//! crate's own source — the idiom `alo-adapting` already uses to forbid itself a network.
//! Storing a zoom as raw thousandths and depending on nothing was considered and
//! rejected: `Zoom::of` is what refuses 0 and 50_000, and **a panel holding a number
//! nobody validated is a worse trade than a panel holding a checked type beside a check
//! that it holds nothing else.**

use alo_canvas::Zoom;
use alo_dock::window::{HowItSits, WindowId};
use alo_dock::windows::Windows;

use crate::panel::{NotPutAside, Panel};
use crate::where_it_goes_back::WhereItGoesBack;

/// Put a window aside: it leaves the canvas and the panel holds its preview.
///
/// The `zoom` is the one the person was at when they performed the gesture. It is handed
/// in because this crate cannot ask a camera, and because the person who was looking is
/// the only one who knows.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] if no window has that id — **a different refusal from
/// the panel's own**, which is about the panel not holding it. Asking to minimise a
/// window that does not exist and asking to restore one the panel does not have are
/// different mistakes, and a caller that could not tell them apart would say the wrong
/// one to a person.
///
/// [`NotPutAside::ItIsAlreadyThere`] if it is already in the panel.
pub fn put_aside(
    windows: &mut Windows,
    panel: &mut Panel,
    id: WindowId,
    zoom: Zoom,
    place: alo_canvas::Place,
    privacy: crate::whether_it_is_private::Privacy,
) -> Result<(), NotPutAside> {
    let window = windows.window(id).ok_or(NotPutAside::ItIsNotThere)?;
    panel.put_aside(window, zoom, place, privacy)?;
    // **After the panel has accepted it, never before.** If the state changed first and
    // the panel then refused, the window would be put aside with nothing holding its
    // preview — a window a person cannot see and cannot get back, which is the one
    // outcome this whole surface exists to prevent.
    windows.now_sits(id, HowItSits::PutAside);
    Ok(())
}

/// Bring a window back: the panel gives up its preview and says which view it goes to.
///
/// The view returned is **the one saved when it was put aside** — the same patch and the
/// same zoom, handed back rather than recalculated. That is what makes *restore it where
/// it was* a sentence rather than a wish.
///
/// It returns the whole view and not the patch alone, so that a caller cannot restore a
/// window to its saved rectangle at whatever zoom the person has since wandered to. That
/// failure would look like a bug in the window and be a bug in what was saved.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] if the panel does not hold it.
pub fn bring_back(
    windows: &mut Windows,
    panel: &mut Panel,
    id: WindowId,
) -> Result<WhereItGoesBack, NotPutAside> {
    let preview = panel.bring_back(id)?;
    windows.now_sits(id, HowItSits::OnTheCanvas);
    Ok(preview.where_it_goes_back())
}
