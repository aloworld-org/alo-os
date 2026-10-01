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
use alo_dock::{AppId, HowItSits, NotAnApp, Patch, Window, WindowId};
use alo_put_aside::panel::NotPutAside;
use alo_put_aside::whether_it_is_private::Privacy;
use alo_put_aside::{Panel, Preview};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// Why a window could not be put aside, or brought back.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum NotAside {
    /// The panel refused it — it is already there, or it is not.
    #[error(transparent)]
    ThePanel(#[from] NotPutAside),
    /// The window has no name this machine can use as an application's.
    ///
    /// **Not a fallback.** A window whose class is blank is a window whose
    /// preview would be headed with nothing, and a preview a person cannot
    /// read is a window they cannot find — which is the outcome the whole
    /// surface exists to prevent.
    ///
    /// The refusal is carried by value rather than as a source, because
    /// `alo_dock::NotAnApp` does not implement `Error` — it is a refusal this
    /// repository words for a person rather than a chain a programmer walks.
    #[error("this window has no application name to head its preview with")]
    Unnamed(NotAnApp),
}

impl From<NotAnApp> for NotAside {
    fn from(why: NotAnApp) -> Self {
        Self::Unnamed(why)
    }
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
    /// [`NotAside::ThePanel`] when the panel will not take it, and
    /// [`NotAside::Unnamed`] when the window has no usable application name.
    pub fn put_this_window_aside(
        &mut self,
        panel: &mut Panel,
        frame: &WlSurface,
        at: Patch,
        zoom: Zoom,
        privacy: Privacy,
    ) -> Result<WindowId, NotAside> {
        let window = self.as_a_model_window(frame, at)?;
        let id = window.id();
        panel.put_aside(&window, zoom, privacy)?;
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
    fn as_a_model_window(&self, frame: &WlSurface, at: Patch) -> Result<Window, NotAnApp> {
        let id = WindowId::numbered(crate::window_number::Numbers::of(frame));
        // **Two different facts, read separately on purpose.** `the_name_of`
        // answers one question — *what is this frame called* — by falling back
        // from the title to the class, which is right for a label and wrong
        // here: a preview is headed with the application and says the window's
        // own name beneath it. Collapsing them would head every preview of a
        // titled window with its title and leave the application unsaid.
        let called = self.the_name_of(frame);
        // **Class first, then the window's own title, then a refusal.** The
        // class is the application; the title is this window. Falling back is
        // not conflating them — a client that named its window and not its
        // class has told us what it is by the only name it gave, and the
        // alternative is refusing to put aside a window that is plainly
        // identifiable.
        //
        // **A window with neither is refused, and that is a hole rather than a
        // policy.** `docs/design` has an answer for an unnamed window —
        // `FrameName::AnApplication`, said with the machine's own translated
        // word — and `alo_dock::Window` cannot carry it, because `AppId` is an
        // identity and a translated word is not one: two unnamed windows would
        // become one application. Recorded here rather than papered over with
        // the number, which would be an identity nobody could read.
        let name = the_class_of(frame)
            .or_else(|| called.its_own_words().map(str::to_owned))
            .unwrap_or_default();
        let app = AppId::named(&name)?;
        Ok(Window::of(
            id,
            app,
            called.its_own_words().unwrap_or_default(),
            at,
            HowItSits::OnTheCanvas,
        ))
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
