//! Shared native scene selection, validation and painting between clients and cursors.
use crate::{RenderError, WindowControlReaderScene};
use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

/// Shared painter input keeps the existing complete-label public API additive.
#[derive(Clone, Copy)]
pub(crate) enum NativeScene<'a> {
    /// Existing strip with an optional complete label.
    Controls(crate::WindowControlScene<'a>),
    /// Complete paged reader and its original strip.
    Reader(&'a WindowControlReaderScene<'a>),
    /// The sign-in screen, which is the whole output.
    SignIn(&'a crate::sign_in_raster::SignInPicture),
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
}

impl NativeScene<'_> {
    /// Refuse mismatched target geometry before importing clients.
    pub(crate) fn validate(self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        match self {
            Self::Controls(scene) => scene.validate(size),
            Self::Reader(scene) => scene.validate(size),
            Self::SignIn(picture) => picture.validate(size),
        }
    }

    /// Paint the selected native content between clients and cursors.
    pub(crate) fn paint(self, frame: &mut impl Frame) -> Result<(), RenderError> {
        match self {
            Self::Reader(scene) => scene.paint(frame),
            Self::SignIn(picture) => picture.paint(frame),
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
