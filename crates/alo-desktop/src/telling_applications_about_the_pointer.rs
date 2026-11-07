//! **Telling applications which pointer the person chose.**
//!
//! An application that draws its own pointer reads the cursor theme from
//! `XCURSOR_THEME` when it starts. The image names alo's theme as the default
//! (`/usr/lib/environment.d/60-alo-cursor.conf`); a person who chooses the
//! familiar arrow in Settings is given the theme with that arrow, so the pointer
//! is the same arrow over every window rather than only over alo's own.
//!
//! # Only once the person has chosen
//!
//! **Nothing is told at sign-in unless the person chose in Settings.** A person
//! who set `XCURSOR_THEME` themselves, in their own `~/.config/environment.d`,
//! keeps it until they choose here — at which point what they chose here is the
//! latest thing they said. Telling the session at every sign-in would overwrite
//! their own setting with alo's default, which is a choice taken away
//! (`CLAUDE.md`, law 5).
//!
//! # What it does not reach
//!
//! **An application already running keeps the pointer it started with**: the
//! environment is read once, at start. An application that asks alo for a shape
//! by name (`wp_cursor_shape_v1`) is drawn by alo and changes at once.

use alo_shell::ThePointersLook;

/// The variables an application is told for `look`.
#[must_use]
pub fn what_applications_are_told(look: ThePointersLook) -> Vec<String> {
    vec![format!("XCURSOR_THEME={}", look.the_theme())]
}

/// The pointer as `appearance` holds it, and whether the person chose it —
/// which is whether the session is told at sign-in.
#[must_use]
pub fn as_they_left_it(appearance: &alo_appearance::Appearance) -> (ThePointersLook, bool) {
    let look = ThePointersLook {
        shape: appearance.pointer_shape(),
    };
    (look, appearance.changes().pointer_shape().is_some())
}

/// Tell the person's session `look`, over a connection of its own, and say on
/// the log what could not be told.
///
/// **A connection of its own** because this is rare — a person changing their
/// pointer — and holding one open for it would be a second bus connection for
/// the whole session to save a handshake once in a while.
pub fn tell(look: ThePointersLook) {
    match zbus::blocking::Connection::session() {
        Ok(session) => crate::telling_the_session_where_alo_draws::told_to_the_session(
            session.inner(),
            &what_applications_are_told(look),
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

    /// **Each shape names its own theme**, so an application drawing its own
    /// pointer draws the arrow the person chose.
    #[test]
    fn each_shape_is_told_as_its_theme() {
        assert_eq!(
            what_applications_are_told(ThePointersLook::default()),
            ["XCURSOR_THEME=alo"]
        );
        assert_eq!(
            what_applications_are_told(ThePointersLook {
                shape: alo_appearance::PointerShape::Classic,
            }),
            ["XCURSOR_THEME=alo-classic"]
        );
    }

    /// **A person who chose nothing is told nothing**, so a theme they set
    /// themselves outside Settings is left as they set it; one who chose is
    /// told, the shipped arrow included.
    #[test]
    fn only_a_choice_is_told_at_sign_in() {
        let untouched = alo_appearance::Appearance::shipped();
        assert_eq!(
            as_they_left_it(&untouched),
            (ThePointersLook::default(), false)
        );
        let mut chose = alo_appearance::Appearance::shipped();
        chose.set_pointer_shape(alo_appearance::PointerShape::Classic);
        assert_eq!(
            as_they_left_it(&chose),
            (
                ThePointersLook {
                    shape: alo_appearance::PointerShape::Classic
                },
                true
            )
        );
        let mut chose_the_kite = alo_appearance::Appearance::shipped();
        chose_the_kite.set_pointer_shape(alo_appearance::PointerShape::Kite);
        assert!(as_they_left_it(&chose_the_kite).1);
    }
}
