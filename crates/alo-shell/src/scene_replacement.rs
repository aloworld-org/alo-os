//! Blocking scene replacement and explicit post-commit cleanup outcomes.

use crate::{ResourceError, ResourceFailure, ScanoutPixels};
use crate::{
    display_resources::Allocation,
    scanout::{Scanout, ScanoutDevice},
    scene_scanout::Submitted,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use std::io;

/// The new scene is active, independently of old-resource cleanup success.
#[derive(Debug)]
#[must_use = "inspect retirement errors before continuing presentation"]
pub struct SceneReplacement {
    /// Failed old-resource release. Disable the new scene and retire the device;
    /// further replacement is refused. New scene identities remain authoritative.
    pub retirement_error: Option<ResourceError>,
}

impl<D: ScanoutDevice + Clone> Submitted<D> {
    /// Shared production/test path; no externally supplied fd or route can drift.
    pub(crate) fn replace(
        &mut self,
        (pixels, surfaces): (ScanoutPixels, Vec<WlSurface>),
    ) -> Result<SceneReplacement, ResourceError> {
        let invalid = |stage| ResourceError {
            failure: ResourceFailure {
                stage,
                source: io::ErrorKind::InvalidInput.into(),
            },
            cleanup: Vec::new(),
        };
        if !self.replacement_allowed || !self.active.enabled {
            return Err(invalid("scene requires session retirement"));
        }
        let (w, h) = self.mode.size();
        if pixels.size() != (u32::from(w), u32::from(h)) {
            return Err(invalid("validate replacement scene"));
        }
        let candidate = (|| {
            let frame = pixels.frame().map_err(|source| ResourceError {
                failure: ResourceFailure {
                    stage: "validate replacement scene",
                    source,
                },
                cleanup: Vec::new(),
            })?;
            let (mut owned, fb, blob) =
                Allocation::allocate(self.active.owned.device.clone(), &self.mode, &self.formats)?;
            owned.write_frame(&frame)?;
            Scanout::activate(owned, self.active.plan.clone(), fb, blob)
        })();
        let candidate = match candidate {
            Ok(candidate) => candidate,
            Err(error) => {
                if !error.cleanup.is_empty() {
                    self.replacement_allowed = false;
                }
                return Err(error);
            }
        };
        // Blocking success means the old framebuffer is no longer scanned out.
        // Disarm its disable before dropping it: disabling would blank the new scene.
        let mut old = std::mem::replace(&mut self.active, candidate);
        old.enabled = false;
        self.surfaces = surfaces;
        let retirement_error = old.owned.release().err();
        self.replacement_allowed = retirement_error.is_none();
        Ok(SceneReplacement { retirement_error })
    }
}
