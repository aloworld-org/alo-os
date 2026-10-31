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
    /// The cloud before the processing destination, while a provider answers.
    Cloud,
}

impl AnIcon {
    /// Every icon, in the order the row draws them.
    pub(crate) const ALL: [Self; 6] = [
        Self::Attach,
        Self::TheModelsChevron,
        Self::Permissions,
        Self::Send,
        Self::Stop,
        Self::Cloud,
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
            Self::Cloud => include_str!("../icons/the-reply-box/cloud.svg"),
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
            Self::Cloud => "cloud.svg",
        }
    }

    /// How large it is drawn, in logical pixels.
    pub(crate) const fn is(self) -> u32 {
        match self {
            Self::TheModelsChevron => alo_tasks::measures::A_CHEVRON,
            Self::Cloud => alo_tasks::measures::THE_DESTINATIONS_ICON,
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
            Self::Cloud => 5,
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
/// **At the file's own size**: its `viewBox`, in logical pixels, through the
/// display's scale. For the square icons that is the icon frame itself; the
/// cloud's is 14.5 × 10.5 in the middle of a 16 frame, which the design draws
/// and which stretching it to the frame would distort. Never larger than the
/// frame ([`AnIcon::is`]).
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
    let (wide, tall) = shape.its_box();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "an icon's box is a few dozen pixels at any display's scale"
    )]
    let at_this_scale = |logical: f32| (f64::from(logical) * scale.factor()).round() as i32;
    // **Never bigger than its icon frame**, whatever a file's `viewBox` says:
    // the frame is the room the design gives it, and a box that overran it
    // would draw over the control beside it.
    let frame = scale.px(icon.is());
    let (wide, tall) = (at_this_scale(wide).min(frame), at_this_scale(tall).min(frame));
    let left = centred_in.loc.x + (centred_in.size.w - wide) / 2;
    let top = centred_in.loc.y + (centred_in.size.h - tall) / 2;
    crate::inking_a_shape::ink_a_shape(
        panel,
        shape,
        Rectangle::new((left, top).into(), (wide, tall).into()),
        ink,
    );
}

#[cfg(test)]
#[path = "the_reply_boxs_icons_tests.rs"]
mod tests;
