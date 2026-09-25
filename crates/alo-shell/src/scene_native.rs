//! Shared native scene selection, validation and painting between clients and cursors.
use crate::{RenderError, WindowControlReaderScene};
use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

/// Shared painter input keeps the existing complete-label public API additive.
#[derive(Clone, Copy)]
pub(crate) enum NativeScene<'a> {
    /// An exclusive opaque lock texture; no other layer may be imported.
    Lock(&'a crate::lock_raster::LockPicture),
    /// Existing strip with an optional complete label.
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
            desktop: None,
            record: None,
            settings: None,
            approval: None,
            status: None,
            in_use: None,
            notifications: None,
        }
    }

    /// Whether there is anything of this shell's own on this frame at all.
    pub(crate) const fn is_empty(&self) -> bool {
        self.scene.is_none()
            && self.desktop.is_none()
            && self.record.is_none()
            && self.settings.is_none()
            && self.approval.is_none()
            && self.status.is_none()
            && self.in_use.is_none()
            && self.notifications.is_none()
    }
}

impl NativeScene<'_> {
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

    /// Paint the selected native content between clients and cursors.
    pub(crate) fn paint(self, frame: &mut impl Frame) -> Result<(), RenderError> {
        match self {
            Self::Lock(_) => Err(RenderError::LockScene),
            Self::Reader(scene) => scene.paint(frame),
            Self::SignIn(picture) => picture.paint(frame),
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
