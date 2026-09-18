//! Exclusive nested lock frame submission and fail-closed blanking.
use crate::{
    Cursor, FrameTarget, LockBackground, LockLook, LockSurface, Nested, RenderError,
    WindowControlLabels,
};
use alo_locking::LockScreen;

impl Nested {
    /// Draw only the lock policy's view and, after Enter, the reused sign-in fields.
    /// The caller refreshes `screen` from the same surface at each tick and prepares
    /// its background again on output/appearance/rotation changes. No client is read.
    /// # Errors
    /// Refuses stale background, invalid geometry or any backend failure. The caller
    /// must retain lock ownership on failure and must never fall back to a desktop.
    pub fn submit_lock<N>(
        &mut self,
        surface: &LockSurface<N>,
        screen: &LockScreen,
        background: &LockBackground,
        labels: &mut WindowControlLabels,
        look: &LockLook<'_>,
    ) -> Result<(), RenderError> {
        let result = self.try_submit_lock(surface, screen, background, labels, look);
        if result.is_err() {
            // Replace an old desktop with an opaque empty frame on layout failure.
            // A failed backend is still returned; the session must retire that output.
            self.submit_native_scene(&[], &[], &Cursor::Hidden, None, None)?;
        }
        result
    }

    /// Prepare the exclusive frame before any private surface is imported.
    fn try_submit_lock<N>(
        &mut self,
        surface: &LockSurface<N>,
        screen: &LockScreen,
        background: &LockBackground,
        labels: &mut WindowControlLabels,
        look: &LockLook<'_>,
    ) -> Result<(), RenderError> {
        let size = self.size();
        if background.size != (size.w, size.h) {
            return Err(RenderError::LockScene);
        }
        let fields = surface.is_asking().then(|| surface.shows());
        let drawn = crate::lock_raster::picture(screen, background, fields, labels, look)?;
        self.submit_native_scene(
            &[],
            &[],
            &Cursor::Hidden,
            Some(crate::scene_native::NativeScene::Lock(&drawn)),
            None,
        )
        .map(|_| ())
    }
}
