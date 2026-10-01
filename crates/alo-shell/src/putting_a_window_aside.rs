//! Putting a window aside, and bringing it back.
//!
//! **This makes an existing promise true rather than adding one**, which is the
//! judgement `CLAUDE.md` asks a change to state. `docs/features.md` has carried
//! *minimising puts a preview in the panel at the edge of the screen* at
//! `[v0.01]` since the owner moved it there; `alo-put-aside` decided every part
//! of what that means and has been complete for a day; and nothing could reach
//! it, because **no road led here**. Nothing promised to a person is new below.
//!
//! # The road that existed went somewhere else
//!
//! `crate::window_minimize` answers `xdg_toplevel.set_minimized` and its own
//! header says what it is: *trusted visibility transitions*. It **hides** a
//! window. Hidden roots keep their buffers, their placement and their cycling
//! position, and revealing neither activates nor raises.
//!
//! That is what minimising meant before this product had a panel, and it is
//! still exactly right for a client that asks. But `features.md` distinguishes
//! two things and says the person picks: *compacting* leaves a live tile on the
//! canvas, *minimising* puts a **preview in the panel**. A compositor answering
//! the XDG request and a plan promising a preview can both be true sentences
//! **about different mechanisms** — which is how this went a day without being
//! noticed, and why the two roads are named apart here rather than merged.
//!
//! So: the client-driven request keeps hiding. **A person putting a window
//! aside comes through this file**, and gets a preview.
//!
//! # Told where it is, never asking
//!
//! The patch — where the window sits on the canvas — is handed in. This file
//! does not reach into a placement for it, because only the caller knows which
//! display's canvas this is and what scale puts the window in the plane's own
//! units. The same seam `crate::dock_room` takes, and for the same reason.
//!
//! **That patch is what a person gets back.** `Preview` carries it, and
//! bringing a window back returns it to its own place rather than to wherever
//! the canvas happens to be — which is the clause `alo-put-aside`'s restoring
//! module exists for and the reason the patch is not optional here.

use alo_canvas::Zoom;
use alo_dock::{AppId, HowItSits, Patch, Window, WindowId};
use alo_put_aside::panel::NotPutAside;
use alo_put_aside::whether_it_is_private::Privacy;
use alo_put_aside::{Panel, Preview};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// Why a window could not be put aside, or brought back.
///
/// **Two variants, and which two changed twice on 2026-10-01.** `Unnamed` refused a window
/// whose client had set neither a class nor a title, on the reasoning that its
/// preview would be headed with nothing and a preview a person cannot read is a
/// window they cannot find. The owner's direction replaced that: *a missing
/// application identity must not prevent minimization.* The premise was also
/// wrong — the preview is headed with the **window's** own name, and only the
/// application line is lost, so such a preview is one line shorter rather than
/// blank.
///
/// The variant is removed rather than kept and never produced. A refusal nothing
/// can cause is a refusal every caller must still handle and no test can reach,
/// which is the shape this repository spent 2026-09-30 removing elsewhere.
///
/// **Then [`NotAside::NoPlace`] arrived the same day, and it is not that shape
/// wearing a new name.** The two were compared on exactly the question that
/// retired the first: can a test cause it? `Unnamed` could not, once the preview
/// was headed with the window's own name. `NoPlace` can — a surface really is on
/// no Place while it maps, and `the_place_of_the_window` answers `Option` because
/// of that and not as a courtesy. So the count went back to two on purpose, and
/// this paragraph exists because a reader who saw only the sentence above would
/// reasonably think the lesson had been forgotten within the hour.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum NotAside {
    /// The panel refused it — it is already there, or it is not.
    #[error(transparent)]
    ThePanel(#[from] NotPutAside),
    /// The compositor has no Place for this surface, so it cannot say which
    /// surface the window is on. Saving a Place nobody chose would send it back
    /// to the wrong one.
    ///
    /// **Not a fallback, and the reason is sharper than the removed variant's.**
    /// A patch alone is ambiguous: `(4200, 0)` exists on every Place, so a
    /// preview saved without one restores the window to whichever surface the
    /// person happens to be looking at. Defaulting to `Place::FIRST` would make
    /// that a window quietly coming back somewhere else, which looks like a bug
    /// in the panel while the panel does exactly what it was told — and every
    /// test would stay green, because every other fixture has one Place.
    ///
    /// `alo_canvas::Place` has no `Default` precisely so this case has to be
    /// written rather than skipped: the type refuses to let a call site not
    /// answer *which Place*. This refusal is that answer being **unavailable**
    /// rather than unasked.
    ///
    /// **And unlike `Unnamed`, this one is reachable**, which is the whole of why
    /// one went and this arrives in the same change.
    /// [`crate::Server::the_place_of_the_window`] answers `Option` because a real surface
    /// can really not be on a Place: while it is mapping, or on a display that
    /// has gone away. A refusal a test can cause is a guarantee; a refusal
    /// nothing can cause is a cost every caller pays for nothing.
    #[error("this window is not on a Place this compositor knows")]
    NoPlace,
}

impl crate::Server {
    /// Put this window aside: it leaves the canvas and becomes a preview.
    ///
    /// The window is hidden **after** the panel has accepted it, never before.
    /// `alo_put_aside::putting_aside` gives the reason and it is the same one:
    /// if the window were hidden first and the panel then refused, there would
    /// be a window a person cannot see and cannot get back.
    ///
    /// # Errors
    /// [`NotAside::ThePanel`] when the panel will not take it — **the only
    /// refusal left.** There was a second until 2026-10-01: a window whose
    /// client had set neither a class nor a title was declined, so a person
    /// could not minimise it at all. The owner directed that a missing
    /// application identity must not prevent minimisation, and such a window is
    /// now put aside carrying no application rather than refused.
    pub fn put_this_window_aside(
        &mut self,
        panel: &mut Panel,
        frame: &WlSurface,
        at: Patch,
        zoom: Zoom,
        privacy: Privacy,
    ) -> Result<WindowId, NotAside> {
        let window = self.as_a_model_window(frame, at);
        let id = window.id();
        // **Which Place, asked of the display rather than assumed.** This is the
        // only side that knows: the panel cannot ask a canvas, so the Place
        // arrives as an argument the same way the zoom does. `Option` is refused
        // rather than defaulted — see `NotAside::NoPlace`.
        let place = self
            .the_place_of_the_window(frame)
            .ok_or(NotAside::NoPlace)?;
        panel.put_aside(&window, zoom, place, privacy)?;
        // Only now. See this method's own note, and `putting_aside`'s.
        let _ = self.set_window_minimized(frame, true);
        Ok(id)
    }

    /// Bring a window back from the panel to the place it left.
    ///
    /// Answers the preview it held, whose patch is where the window was — the
    /// caller puts it there, because the caller owns the canvas.
    ///
    /// # Errors
    /// [`NotAside::ThePanel`] when the panel is not holding this window.
    pub fn bring_this_window_back(
        &mut self,
        panel: &mut Panel,
        frame: &WlSurface,
    ) -> Result<Preview, NotAside> {
        let id = WindowId::numbered(crate::window_number::Numbers::of(frame));
        let preview = panel.bring_back(id)?;
        let _ = self.set_window_minimized(frame, false);
        Ok(preview)
    }

    /// This compositor window as the model's, so the panel can be given one.
    ///
    /// **The identity is the compositor's own number.** `window_number::of` is
    /// unique by construction and stable for a surface's life, and
    /// `alo_dividing` already bridges to its own window id the same way — so a
    /// window put aside and a window a division moved are the same window,
    /// rather than two ids that agree by luck.
    fn as_a_model_window(&self, frame: &WlSurface, at: Patch) -> Window {
        let id = WindowId::numbered(crate::window_number::Numbers::of(frame));
        // **Two different facts, read separately on purpose.** `the_name_of`
        // answers one question — *what is this frame called* — by falling back
        // from the title to the class, which is right for a label and wrong
        // here: a preview is headed with the application and says the window's
        // own name beneath it. Collapsing them would head every preview of a
        // titled window with its title and leave the application unsaid.
        let called = self.the_name_of(frame);
        // **Class first, then the window's own title, then no application at
        // all.** The class is the application; the title is this window. Falling
        // back is not conflating them — a client that named its window and not
        // its class has told us what it is by the only name it gave, and the
        // alternative would be declining a window that is plainly identifiable.
        //
        // **A window with neither is put aside with no application, and that is
        // the policy rather than a hole.** It was a refusal until 2026-10-01: a
        // client that mapped a toplevel having set no class and no title could
        // not be minimised at all. The owner's direction is that *a missing
        // application identity must not prevent minimization* and that no
        // identity is to be fabricated — so the absence is carried.
        //
        // The two rejected answers are worth keeping named, because both look
        // like fixes. `FrameName::AnApplication` is a **translated word**, and
        // two unnamed windows sharing it would become one application in every
        // place that groups by `AppId`. The window's own number is unique and is
        // an identity **nobody can read**. An absence is neither: it cannot be
        // grouped wrongly and it cannot be shown as a name.
        let app = the_class_of(frame)
            .or_else(|| called.its_own_words().map(str::to_owned))
            .and_then(|name| AppId::named(&name).ok());
        Window::of(
            id,
            app,
            called.its_own_words().unwrap_or_default(),
            at,
            HowItSits::OnTheCanvas,
        )
    }
}

/// The application's own class for this frame, where it set one.
///
/// Read here rather than taken from [`FrameName`] because that type answers a
/// different question and deliberately loses this one — see its own header.
fn the_class_of(frame: &WlSurface) -> Option<String> {
    smithay::wayland::compositor::with_states(frame, |states| {
        states
            .data_map
            .get::<smithay::wayland::shell::xdg::XdgToplevelSurfaceData>()
            .and_then(|data| data.lock().ok())
            .and_then(|state| state.app_id.clone())
    })
    .filter(|it| !it.trim().is_empty())
}
