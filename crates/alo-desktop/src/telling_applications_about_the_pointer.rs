//! **Telling applications which pointer the person chose**: its arrow and its
//! size.
//!
//! An application that draws its own pointer reads `XCURSOR_THEME` and
//! `XCURSOR_SIZE` when it starts. The image names alo's theme at 24 pixels as
//! the default (`/usr/lib/environment.d/60-alo-cursor.conf`); a person who
//! chooses the familiar arrow, or a larger pointer, in Settings is given the
//! theme with that arrow and that size, so the pointer is the same over every
//! window rather than only over alo's own.
//!
//! # Only what the person chose
//!
//! **Nothing is told unless the person chose it in Settings, and only that.** A
//! person who set `XCURSOR_THEME` or `XCURSOR_SIZE` themselves, in their own
//! `~/.config/environment.d`, keeps it until they choose that same thing here —
//! at which point what they chose here is the latest thing they said. Telling the
//! session everything at every sign-in would overwrite their own setting with
//! alo's default, which is a choice taken away (`CLAUDE.md`, law 5). So making
//! the pointer larger tells the size and leaves the theme as it was.
//!
//! # What it does not reach
//!
//! **An application already running keeps the pointer it started with**: the
//! environment is read once, at start. An application that asks alo for a shape
//! by name (`wp_cursor_shape_v1`) is drawn by alo and changes at once.

use alo_shell::ThePointersLook;

/// Which of the pointer's two variables to tell: the theme, which is the
/// arrow and the colour together, and the size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WhatToTell {
    /// The cursor theme, which is the arrow.
    pub theme: bool,
    /// The cursor size.
    pub size: bool,
}

impl WhatToTell {
    /// What differs between `before` and `now`.
    #[must_use]
    pub fn between(before: ThePointersLook, now: ThePointersLook) -> Self {
        Self {
            theme: before.shape != now.shape || before.colour != now.colour,
            size: before.size != now.size,
        }
    }
}

/// The variables an application is told for `look`, of those `what` names.
#[must_use]
pub fn what_applications_are_told(look: ThePointersLook, what: WhatToTell) -> Vec<String> {
    let mut told = Vec::new();
    if what.theme {
        told.push(format!("XCURSOR_THEME={}", look.the_theme()));
    }
    if what.size {
        told.push(format!("XCURSOR_SIZE={}", look.logical_pixels()));
    }
    told
}

/// The pointer as `appearance` holds it, and which of it the person chose —
/// which is what the session is told at sign-in.
#[must_use]
pub fn as_they_left_it(appearance: &alo_appearance::Appearance) -> (ThePointersLook, WhatToTell) {
    let look = ThePointersLook {
        shape: appearance.pointer_shape(),
        size: appearance.pointer_size(),
        colour: appearance.pointer_colour(),
    };
    // **The theme is the arrow and the colour together**, so a choice of
    // either is a choice of theme.
    let chose = WhatToTell {
        theme: appearance.changes().pointer_shape().is_some()
            || appearance.changes().pointer_colour().is_some(),
        size: appearance.changes().pointer_size().is_some(),
    };
    (look, chose)
}

/// Tell the person's session `look`, as much of it as `what` names, over a
/// connection of its own, and say on the log what could not be told.
///
/// **A connection of its own** because this is rare — a person changing their
/// pointer — and holding one open for it would be a second bus connection for
/// the whole session to save a handshake once in a while. Nothing to tell opens
/// nothing.
pub fn tell(look: ThePointersLook, what: WhatToTell) {
    let told = what_applications_are_told(look, what);
    if told.is_empty() {
        return;
    }
    match zbus::blocking::Connection::session() {
        Ok(session) => crate::telling_the_session_where_alo_draws::told_to_the_session(
            session.inner(),
            &told,
            "which pointer the person chose, so an application it starts draws the one it had",
        ),
        Err(why) => eprintln!(
            "alo-desktop: the session bus could not be reached, so applications started from \
             now on draw the pointer they had rather than the one just chosen: {why}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_appearance::{PointerShape, PointerSize};

    /// Both variables.
    const BOTH: WhatToTell = WhatToTell {
        theme: true,
        size: true,
    };

    /// **Each choice is told as the variable an application reads**: the arrow
    /// as its theme, the size in logical pixels.
    #[test]
    fn each_choice_is_told_as_what_an_application_reads() {
        assert_eq!(
            what_applications_are_told(ThePointersLook::SHIPPED, BOTH),
            ["XCURSOR_THEME=alo", "XCURSOR_SIZE=24"]
        );
        assert_eq!(
            what_applications_are_told(
                ThePointersLook {
                    shape: PointerShape::Classic,
                    size: PointerSize::Larger,
                    ..ThePointersLook::SHIPPED
                },
                BOTH
            ),
            ["XCURSOR_THEME=alo-classic", "XCURSOR_SIZE=48"]
        );
        assert!(
            what_applications_are_told(ThePointersLook::SHIPPED, WhatToTell::default()).is_empty()
        );
    }

    /// **A person who chose nothing is told nothing**, and one who chose only
    /// the size is told only the size, so a theme they set themselves outside
    /// Settings is left as they set it.
    #[test]
    fn only_what_was_chosen_is_told_at_sign_in() {
        let untouched = alo_appearance::Appearance::shipped();
        assert_eq!(
            as_they_left_it(&untouched),
            (ThePointersLook::SHIPPED, WhatToTell::default())
        );
        let mut larger = alo_appearance::Appearance::shipped();
        larger.set_pointer_size(PointerSize::Large);
        let (look, chose) = as_they_left_it(&larger);
        assert_eq!(
            chose,
            WhatToTell {
                theme: false,
                size: true
            }
        );
        assert_eq!(what_applications_are_told(look, chose), ["XCURSOR_SIZE=36"]);
        let mut classic = alo_appearance::Appearance::shipped();
        classic.set_pointer_shape(PointerShape::Classic);
        assert_eq!(
            as_they_left_it(&classic).1,
            WhatToTell {
                theme: true,
                size: false
            }
        );
    }

    /// **A new colour is a new theme**: the white kite is `alo-white`, told as
    /// the theme and not as a size.
    #[test]
    fn a_colour_is_told_as_its_theme() {
        let white = ThePointersLook {
            colour: alo_appearance::PointerColour::White,
            ..ThePointersLook::SHIPPED
        };
        let what = WhatToTell::between(ThePointersLook::SHIPPED, white);
        assert_eq!(
            what,
            WhatToTell {
                theme: true,
                size: false
            }
        );
        assert_eq!(
            what_applications_are_told(white, what),
            ["XCURSOR_THEME=alo-white"]
        );
        let mut chose = alo_appearance::Appearance::shipped();
        chose.set_pointer_colour(alo_appearance::PointerColour::Black);
        let (look, told) = as_they_left_it(&chose);
        assert_eq!(
            what_applications_are_told(look, told),
            ["XCURSOR_THEME=alo-black"]
        );
    }

    /// **A change tells what changed and nothing else.**
    #[test]
    fn a_change_tells_only_what_changed() {
        let larger = ThePointersLook {
            size: PointerSize::Larger,
            ..ThePointersLook::SHIPPED
        };
        assert_eq!(
            WhatToTell::between(ThePointersLook::SHIPPED, larger),
            WhatToTell {
                theme: false,
                size: true
            }
        );
        assert_eq!(
            WhatToTell::between(larger, larger),
            WhatToTell::default(),
            "nothing changed, nothing told"
        );
    }
}
