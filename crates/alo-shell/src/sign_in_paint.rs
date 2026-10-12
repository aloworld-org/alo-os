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

    /// Draw the surface, then the shapes, then the text.
    ///
    /// **The surface is under everything**, because it is the ground the rest
    /// is drawn on: the warm off-white, the monogram's rounded square and the
    /// entry field with its corners. `washed` is that raster already uploaded
    /// — a frame borrows the renderer, so nothing can upload inside one, which
    /// is why the caller is asked for it.
    ///
    /// A picture with no surface still paints: the sentence a machine with no
    /// accounts shows keeps its flat ground until that screen is built.
    pub(crate) fn paint<F: Frame>(
        &self,
        frame: &mut F,
        washed: Option<&(F::TextureId, smithay::utils::Rectangle<i32, Physical>)>,
    ) -> Result<(), RenderError> {
        if let Some((texture, area)) = washed {
            crate::painted::wash(frame, texture, *area)?;
        }
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
