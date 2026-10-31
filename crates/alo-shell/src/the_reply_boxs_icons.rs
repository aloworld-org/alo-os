//! **The reply box's five icons**: Attach, the arrow beside the model's name,
//! Permissions, and the one button's Send arrow or Stop square.
//!
//! Until 2026-10-30 the composer row drew three empty grey boxes and an empty
//! navy circle: the artwork had never been committed, and the shell could draw
//! no curve that was not a letter. The owner's decision of that day: take the
//! icons from the design file as SVG, keep them in the repository, and draw
//! them as vectors. The files are in `crates/alo-shell/icons/the-reply-box/`,
//! byte for byte as the design file exported them, with the node each came from
//! in `where-they-came-from.toml` beside them. `alo_vector_shapes` reads and
//! draws them (ADR 0142).
//!
//! # Drawn at the display's scale
//!
//! The artwork is **28 logical pixels** (`alo_tasks::measures::ARTWORK`) and the
//! chevron **24** (`alo_tasks::measures::A_CHEVRON`), each converted through the
//! display's scale: on a screen at 200 per cent an icon is drawn into 56 pixels
//! from the same paths, so it is sharper and the same size to the eye.
//!
//! # Inked in the palette's colour
//!
//! The file's own colours are the design's light tokens — `text/primary` for
//! the three on a cool ground, white on the navy button — and a test holds them
//! to that. What is drawn is the palette's ink, so the icons follow the person's
//! contrast setting as the words beside them do.

use std::sync::OnceLock;

use alo_tasks::TheOneButton;
use alo_vector_shapes::AShape;
use smithay::utils::{Physical, Rectangle};

use crate::display_scale::DisplayScale;
use crate::painted::Washed;

/// One of the reply box's icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnIcon {
    /// The paperclip on Attach.
    Attach,
    /// The arrow beside the model's name.
    TheModelsChevron,
    /// The shield on Permissions.
    Permissions,
    /// The one button while alo is not working: an arrow up.
    Send,
    /// The one button while alo is working: a solid square.
    Stop,
}

impl AnIcon {
    /// Every icon, in the order the row draws them.
    pub(crate) const ALL: [Self; 5] = [
        Self::Attach,
        Self::TheModelsChevron,
        Self::Permissions,
        Self::Send,
        Self::Stop,
    ];

    /// The icon the one button shows. **Stop while alo is working, Send
    /// otherwise**: the design draws Stop in every state where alo is in the
    /// middle of something, including waiting for an answer, and Send once the
    /// work is over. A greyed Send or Stop is drawn as itself, because how a
    /// disabled button looks is not in the design.
    pub(crate) const fn on_the_one_button(button: TheOneButton) -> Self {
        match button {
            TheOneButton::Send | TheOneButton::NothingToSend => Self::Send,
            TheOneButton::Stop | TheOneButton::Stopping => Self::Stop,
        }
    }

    /// The file, as the design exported it.
    pub(crate) const fn svg(self) -> &'static str {
        match self {
            Self::Attach => include_str!("../icons/the-reply-box/attach.svg"),
            Self::TheModelsChevron => include_str!("../icons/the-reply-box/model-chevron.svg"),
            Self::Permissions => include_str!("../icons/the-reply-box/permissions.svg"),
            Self::Send => include_str!("../icons/the-reply-box/send.svg"),
            Self::Stop => include_str!("../icons/the-reply-box/stop.svg"),
        }
    }

    /// The file's name, as `where-they-came-from.toml` records it.
    pub(crate) const fn file_name(self) -> &'static str {
        match self {
            Self::Attach => "attach.svg",
            Self::TheModelsChevron => "model-chevron.svg",
            Self::Permissions => "permissions.svg",
            Self::Send => "send.svg",
            Self::Stop => "stop.svg",
        }
    }

    /// How large it is drawn, in logical pixels.
    pub(crate) const fn is(self) -> u32 {
        match self {
            Self::TheModelsChevron => alo_tasks::measures::A_CHEVRON,
            Self::Attach | Self::Permissions | Self::Send | Self::Stop => {
                alo_tasks::measures::ARTWORK
            }
        }
    }

    /// Its place in [`Self::ALL`].
    const fn at(self) -> usize {
        match self {
            Self::Attach => 0,
            Self::TheModelsChevron => 1,
            Self::Permissions => 2,
            Self::Send => 3,
            Self::Stop => 4,
        }
    }

    /// The artwork, read once for the life of the shell.
    ///
    /// [`None`] for a file `alo_vector_shapes` refused, which is said once on
    /// the error stream when it is first asked for, and which
    /// `the_reply_boxs_icons_tests` refuses on every build.
    pub(crate) fn shape(self) -> Option<&'static AShape> {
        static READ: OnceLock<Vec<Option<AShape>>> = OnceLock::new();
        READ.get_or_init(|| {
            AnIcon::ALL
                .iter()
                .map(|icon| match AShape::read(icon.svg()) {
                    Ok(shape) => Some(shape),
                    Err(why) => {
                        eprintln!(
                            "alo-shell: the reply box's {} could not be read and is not drawn: {why}",
                            icon.file_name()
                        );
                        None
                    }
                })
                .collect()
        })
        .get(self.at())
        .and_then(Option::as_ref)
    }
}

/// Ink `icon` into the panel's raster, centred in `centred_in` (in the
/// display's pixels), at the display's `scale`, in `ink`.
///
/// Pixels outside the raster are left alone, so an icon on a window running off
/// the screen's raster is cut rather than refused.
pub(crate) fn ink_an_icon(
    panel: &mut Washed,
    icon: AnIcon,
    centred_in: Rectangle<i32, Physical>,
    scale: DisplayScale,
    ink: [u8; 3],
) {
    let Some(shape) = icon.shape() else {
        return;
    };
    let side = scale.px(icon.is());
    let Ok(side_is) = u32::try_from(side) else {
        return;
    };
    let left = centred_in.loc.x + (centred_in.size.w - side) / 2;
    let top = centred_in.loc.y + (centred_in.size.h - side) / 2;
    let covered = shape.coverage(side_is, side_is);
    let wide = panel.area.size.w;
    for (at, coverage) in covered.into_iter().enumerate() {
        if coverage == 0 {
            continue;
        }
        let Ok(at) = i32::try_from(at) else {
            return;
        };
        let (x, y) = (
            left + at % side - panel.area.loc.x,
            top + at / side - panel.area.loc.y,
        );
        if x < 0 || y < 0 || x >= wide || y >= panel.area.size.h {
            continue;
        }
        let Ok(index) = usize::try_from((y * wide + x) * 4) else {
            continue;
        };
        if let Some(pixel) = panel.pixels.get_mut(index..index + 3) {
            for (channel, inked) in pixel.iter_mut().zip(ink) {
                *channel = blended(*channel, inked, coverage);
            }
        }
    }
}

/// `under` with `over` painted on it at this coverage.
fn blended(under: u8, over: u8, coverage: u8) -> u8 {
    let (under, over, coverage) = (u32::from(under), u32::from(over), u32::from(coverage));
    let mixed = (over * coverage + under * (255 - coverage) + 127) / 255;
    u8::try_from(mixed).unwrap_or(u8::MAX)
}

#[cfg(test)]
#[path = "the_reply_boxs_icons_tests.rs"]
mod tests;
