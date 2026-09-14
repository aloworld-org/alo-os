//! Paint a laid-out sign-in screen into a frame, and nothing else with it.
//!
//! The picture is the whole output: there is no client beneath it and no
//! control strip beside it, because nobody is signed in to own a window.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::sign_in_raster::SignInPicture;

impl SignInPicture {
    /// Refuse a frame the picture was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::SignInScene)
        }
    }

    /// Draw the shapes, then the text, one run of equal pixels at a time.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
