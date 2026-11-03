//! Shared native scene selection, validation and painting between clients and cursors.
use crate::{RenderError, WindowControlReaderScene};
use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Rectangle, Size},
};

/// Shared painter input keeps the existing complete-label public API additive.
#[derive(Clone, Copy)]
pub(crate) enum NativeScene<'a> {
    /// An exclusive opaque lock texture; no other layer may be imported.
    Lock(&'a crate::lock_raster::LockPicture),
    /// Existing strip with an optional complete label.
    ///
    /// **Superseded.** `docs/design/the-external-window-edge.md` replaces this
    /// with [`NativeLayers::edges`] for ordinary shell-decorated windows, by
    /// the owner's ruling of 2026-10-09. It remains only while the reader and
    /// label paths that share its layout are moved across.
    Controls(crate::WindowControlScene<'a>),
    /// Complete paged reader and its original strip.
    Reader(&'a WindowControlReaderScene<'a>),
    /// The sign-in screen, which is the whole output.
    SignIn(&'a crate::sign_in_raster::SignInPicture),
    /// The recovery screen, which is the whole output: a machine whose desktop
    /// will not start has no client to draw under it.
    Recovery(&'a crate::recovery_raster::RecoveryPicture),
}

/// Everything native painted over clients in one frame: the selected scene, the
/// desktop above it, the record window above that, Settings above the record,
/// the approval surface above Settings, and the egress indicator above all of
/// them.
#[derive(Clone, Copy)]
pub(crate) struct NativeLayers<'a> {
    /// Controls, a reader or the sign-in screen, when one is selected.
    pub(crate) scene: Option<NativeScene<'a>>,
    /// Every open window's external edge, in the order the windows are drawn.
    ///
    /// **A slice and not a scene, because an edge is not one of a set of
    /// alternatives.** It lived in [`Self::scene`] until 2026-10-10, as a
    /// `TheWindowEdge` variant beside the lock screen, the reader and the
    /// sign-in screen — a field that holds *one of* those. So a machine with
    /// three windows open could draw exactly one edge, and its only caller
    /// built every window's edge and then passed `edges.first()`. Nothing was
    /// wrong with any line of it; the shape could not express what a person
    /// sees, which is every window wearing its own.
    ///
    /// **Above the clients and below the desktop**, with the other layers: an
    /// edge sits on its window and the Dock sits over both.
    pub(crate) edges: &'a [crate::window_edge_picture::EdgePicture],
    /// Every task window's edge, painted with the task windows wherever they
    /// are in [`Self::in_front`]'s order.
    pub(crate) task_edges: &'a [crate::window_edge_picture::EdgePicture],
    /// alo's terminal's edge — none or one — painted with the terminal
    /// wherever it is in [`Self::in_front`]'s order (`crate::the_terminals_edge`).
    pub(crate) terminal_edge: &'a [crate::window_edge_picture::EdgePicture],
    /// Which kind of window is in front of which — the applications, the
    /// terminal, the task windows — and so the order they are painted in
    /// (`crate::which_window_is_in_front`, the owner's decision of 2026-10-31).
    pub(crate) in_front: crate::which_window_is_in_front::WhichIsInFront,
    /// The dock and the desktop windows, when the frame carries the desktop.
    pub(crate) desktop: Option<&'a crate::desktop_raster::DesktopPicture>,
    /// The record window, when the frame carries one.
    pub(crate) record: Option<&'a crate::record_raster::RecordPicture>,
    /// Settings, when the frame carries it.
    pub(crate) settings: Option<&'a crate::settings_raster::SettingsPicture>,
    /// The approval surface, when the frame carries one.
    pub(crate) approval: Option<&'a crate::approval_raster::ApprovalPicture>,
    /// The egress indicator, when the frame carries a status area.
    pub(crate) status: Option<&'a crate::egress_status_raster::EgressStatusPicture>,
    /// What on this machine is watching or listening, beside the egress
    /// indicator in the same status area and never drawn as it.
    pub(crate) in_use: Option<&'a crate::in_use_raster::InUsePicture>,
    /// The notifications `alo-notifying` handed over, at the other end of the
    /// dock from both indicators.
    pub(crate) notifications: Option<&'a crate::notification_raster::NotificationPicture>,
    /// The capture tools, while somebody is choosing what to capture or
    /// marking what they captured. Above everything, because what a person is
    /// drawing on must not be covered by what arrives while they draw.
    pub(crate) capturing: Option<&'a crate::capture_raster::CapturePicture>,
    /// alo's own terminal, when one is open and was drawn.
    ///
    /// **A window rather than furniture**, which is why it is its own layer:
    /// the Dock, the status area and a notification each sit above whatever a
    /// client maps, and a terminal does not — it is a window among windows.
    pub(crate) terminal: Option<&'a crate::the_terminal_on_the_plane::TheTerminalOnTheScreen>,
}

impl NativeLayers<'_> {
    /// A frame with none of this shell's own surfaces on it.
    ///
    /// The base every caller builds from, so a layer added to this struct
    /// arrives as *absent* at every call site rather than as a compile error
    /// each one answers its own way.
    pub(crate) const fn nothing() -> Self {
        Self {
            scene: None,
            edges: &[],
            task_edges: &[],
            terminal_edge: &[],
            in_front: crate::which_window_is_in_front::WhichIsInFront::AS_IT_WAS,
            desktop: None,
            record: None,
            settings: None,
            approval: None,
            status: None,
            in_use: None,
            notifications: None,
            capturing: None,
            terminal: None,
        }
    }

    /// Whether there is anything of this shell's own on this frame at all.
    pub(crate) const fn is_empty(&self) -> bool {
        self.scene.is_none()
            && self.edges.is_empty()
            && self.task_edges.is_empty()
            && self.terminal_edge.is_empty()
            && self.desktop.is_none()
            && self.record.is_none()
            && self.settings.is_none()
            && self.approval.is_none()
            && self.status.is_none()
            && self.in_use.is_none()
            && self.notifications.is_none()
            && self.capturing.is_none()
            // **And the terminal**, or a frame holding nothing but alo's own
            // terminal would read as empty and a caller that skips empty
            // frames would skip it.
            && self.terminal.is_none()
    }
}

impl<'a> NativeScene<'a> {
    /// Refuse mismatched target geometry before importing clients.
    pub(crate) fn validate(self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        match self {
            Self::Lock(picture) => {
                if picture.size == (size.w, size.h) {
                    Ok(())
                } else {
                    Err(RenderError::LockScene)
                }
            }
            Self::Controls(scene) => scene.validate(size),
            Self::Reader(scene) => scene.validate(size),
            Self::SignIn(picture) => picture.validate(size),
            Self::Recovery(picture) => picture.validate(size),
        }
    }

    /// The rasters this scene needs uploaded before a frame opens.
    ///
    /// **A scene cannot upload one itself**, because painting happens inside a
    /// frame and the frame borrows the renderer. `crate::desktop_paint` meets
    /// the same wall and answers it the same way: the caller asks what is
    /// needed, uploads it, and hands the textures back to `paint`.
    pub(crate) fn washed(self) -> Option<&'a crate::painted::Washed> {
        match self {
            Self::SignIn(picture) => picture.surface.as_ref(),
            Self::Lock(_) | Self::Controls(_) | Self::Reader(_) | Self::Recovery(_) => None,
        }
    }

    /// Paint the selected native content between clients and cursors.
    pub(crate) fn paint<F: Frame>(
        self,
        frame: &mut F,
        washed: Option<&(F::TextureId, Rectangle<i32, Physical>)>,
    ) -> Result<(), RenderError> {
        match self {
            Self::Lock(_) => Err(RenderError::LockScene),
            Self::Reader(scene) => scene.paint(frame),
            // **The surface first, then the flats and the words on it.** The
            // same order the Dock's own surface is drawn in, and for the same
            // reason: it is what they sit on.
            Self::SignIn(picture) => picture.paint(frame, washed),
            Self::Recovery(picture) => picture.paint(frame),
            Self::Controls(scene) => {
                scene.layout.paint(frame, scene.scheme)?;
                if let Some(label) = scene.label {
                    label.paint(frame)?;
                }
                Ok(())
            }
        }
    }
}
