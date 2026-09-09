//! Borrowed native scene content, validated before importing any client buffer.

use alo_appearance::Scheme;
use smithay::utils::{Physical, Size};

use crate::{RenderError, WindowControlLabel, WindowControlLayout};

/// One frame's native strip and optional complete label, with no input authority.
///
/// Refresh the layout from the live mapping, publish presentation only after
/// successful submission, and retire on removal/failure. This value neither
/// selects a target nor authorizes input. Opaque labels require a host overlay
/// input policy before interactive use. Clipped labels refuse: use a larger box
/// or an alternate full-text surface, never silently omit words.
#[derive(Clone, Copy)]
pub struct WindowControlScene<'a> {
    /// Fresh scale-one strip geometry and feedback.
    pub layout: &'a WindowControlLayout,
    /// Optional prepared label; must fit and leave all controls unobscured.
    pub label: Option<&'a WindowControlLabel>,
    /// Existing appearance tokens, without introducing another palette.
    pub scheme: Scheme,
}

impl WindowControlScene<'_> {
    /// Refuse stale output geometry or inaccessible label placement before paint.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if self.layout.viewport.size != size {
            return Err(RenderError::ControlScene);
        }
        if let Some(label) = self.label
            && (label.viewport != self.layout.viewport
                || label.clipped()
                || self
                    .layout
                    .controls()
                    .iter()
                    .any(|control| control.bounds().intersection(label.bounds()).is_some()))
        {
            return Err(RenderError::ControlScene);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LabelGeometry, WindowControlLabels};
    use alo_appearance::TextScale;
    use alo_strings::Strings;

    #[test]
    fn control_scene_accepts_complete_labels_and_refuses_stale_or_obscured_content()
    -> Result<(), Box<dyn std::error::Error>> {
        let layout = WindowControlLayout::new((320, 180), (0, 0), [false; 3], false)?;
        let strings = Strings::of(alo_shortcuts::shortcut_words()?);
        let mut labels = WindowControlLabels::new()?;
        let control = layout.controls().first().ok_or("missing control")?;
        for (viewport, origin, size, accepted) in [
            ((320, 180), (0, 40), (240, 80), true),
            ((321, 180), (0, 40), (240, 80), false),
            ((320, 180), (0, 0), (240, 80), false),
            ((320, 180), (0, 40), (9, 9), false),
            ((320, 180), (300, 170), (240, 80), false),
        ] {
            let label = labels.prepare(
                control,
                &strings,
                LabelGeometry {
                    viewport,
                    origin,
                    size,
                },
                Scheme::Light,
                TextScale::ordinary(),
            )?;
            let scene = WindowControlScene {
                layout: &layout,
                label: Some(&label),
                scheme: Scheme::Light,
            };
            assert_eq!(scene.validate((320, 180).into()).is_ok(), accepted);
        }
        let scene = WindowControlScene {
            layout: &layout,
            label: None,
            scheme: Scheme::Dark,
        };
        assert!(scene.validate((320, 180).into()).is_ok());
        assert!(matches!(
            scene.validate((319, 180).into()),
            Err(RenderError::ControlScene)
        ));
        Ok(())
    }
}
