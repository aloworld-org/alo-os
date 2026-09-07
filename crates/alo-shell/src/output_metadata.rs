//! Validated, scale-one backend output descriptions, independent of presentation.

use crate::RenderError;
use smithay::output::{PhysicalProperties, Subpixel};

/// One output's immutable identity and current refresh rate.
///
/// Zero millimetres and zero refresh explicitly mean unknown. Make/model are
/// descriptive data, not a claim of EDID discovery. Identity and physical size
/// must stay fixed for the lifetime of a Server; hotplug is not implemented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputMetadata {
    /// Session-unique ASCII connector or virtual-output name.
    pub name: String,
    /// Manufacturer, or "unknown" when unavailable.
    pub make: String,
    /// Model, or "unknown" when unavailable.
    pub model: String,
    /// Physical width and height in millimetres; (0, 0) means unknown.
    pub physical_size: (i32, i32),
    /// Current mode refresh in millihertz, or zero when unknown.
    pub refresh: i32,
}

impl OutputMetadata {
    /// Honest fallback for older custom targets with no hardware description.
    pub fn virtual_output() -> Self {
        Self {
            name: "alo-virtual".into(),
            make: "unknown".into(),
            model: "virtual".into(),
            physical_size: (0, 0),
            refresh: 0,
        }
    }

    /// Refuse malformed wire strings, dimensions and refresh before backend I/O.
    pub(crate) fn validate(&self) -> Result<(), RenderError> {
        let text = |s: &str| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control);
        let (w, h) = self.physical_size;
        if !text(&self.name)
            || !self
                .name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
            || !text(&self.make)
            || !text(&self.model)
            || !((w == 0 && h == 0) || (w > 0 && h > 0))
            || self.refresh < 0
        {
            return Err(RenderError::InvalidOutputMetadata);
        }
        Ok(())
    }

    /// Refresh may vary; replacing the output identity requires a new server.
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        self.name == other.name
            && self.make == other.make
            && self.model == other.model
            && self.physical_size == other.physical_size
    }

    /// Subpixel order is unknown until a backend supplies validated information.
    pub(crate) fn properties(&self) -> PhysicalProperties {
        PhysicalProperties {
            size: self.physical_size.into(),
            subpixel: Subpixel::Unknown,
            make: self.make.clone(),
            model: self.model.clone(),
        }
    }
}

/// Derive millihertz from progressive kernel timings, avoiding integer-Hz loss.
pub(crate) fn direct_metadata(output: &crate::DirectOutput) -> Result<OutputMetadata, RenderError> {
    let mode = output.mode;
    let total = u64::from(mode.hsync().2) * u64::from(mode.vsync().2);
    if !crate::direct_output::supported(&mode) {
        return Err(RenderError::InvalidOutputMetadata);
    }
    let refresh = (u64::from(mode.clock()) * 1_000_000 + total / 2) / total;
    if refresh == 0 {
        return Err(RenderError::InvalidOutputMetadata);
    }
    let physical_size = match output.physical_size {
        Some((w, h)) if w > 0 && h > 0 => (
            i32::try_from(w).map_err(|_| RenderError::InvalidOutputMetadata)?,
            i32::try_from(h).map_err(|_| RenderError::InvalidOutputMetadata)?,
        ),
        _ => (0, 0),
    };
    let metadata = OutputMetadata {
        name: format!("alo-drm-{}", u32::from(output.connector)),
        make: "unknown".into(),
        model: "unknown".into(),
        physical_size,
        refresh: i32::try_from(refresh).map_err(|_| RenderError::InvalidOutputMetadata)?,
    };
    metadata.validate()?;
    Ok(metadata)
}

#[cfg(test)]
#[path = "output_metadata_tests.rs"]
mod tests;
