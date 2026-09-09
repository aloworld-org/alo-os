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
}

impl NativeScene<'_> {
    /// Refuse mismatched target geometry before importing clients.
    pub(crate) fn validate(self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        match self {
            Self::Controls(scene) => scene.validate(size),
            Self::Reader(scene) => scene.validate(size),
        }
    }

    /// Paint the selected native content between clients and cursors.
    pub(crate) fn paint(self, frame: &mut impl Frame) -> Result<(), RenderError> {
        match self {
            Self::Reader(scene) => scene.paint(frame),
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
