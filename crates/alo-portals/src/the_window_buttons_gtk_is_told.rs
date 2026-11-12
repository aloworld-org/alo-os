//! Which window buttons a sandboxed GTK application is told to draw.
//!
//! # Why alo says anything about it at all
//!
//! The owner, 2026-11-12: *No need to add controls if the app has, we can only
//! add the ones that are not there.* So the shell must know which of Minimise
//! and Close a GTK application draws in its own header. It may not look to
//! find out. GTK draws whatever `gtk-decoration-layout` says, and a sandboxed
//! GTK application reads that from this portal, as `button-layout` in
//! `org.gnome.desktop.wm.preferences`. **When alo answers that key, the answer
//! is a fact alo controls rather than one it has to guess.**
//!
//! # Why this value
//!
//! Measured once, by hand, on 2026-11-12, against GNOME Text Editor, Papers and
//! Loupe from Flathub (GNOME runtime 51) and Ptyxis (runtime 50), each in all
//! four states below except `appmenu:close`, which was Text Editor's and
//! a libadwaita window of our own (runtime 51). The method is in
//! `docs/quirks/an-application-that-draws-its-own-header-is-not-asked-which-buttons.md`.
//!
//! | what this portal answered | what each one drew |
//! |---|---|
//! | every key refused, as for an application without the grant | Minimise, Maximise, Close |
//! | appearance and pointer keys, no `button-layout` | Close alone |
//! | `button-layout` = `appmenu:minimize,maximize,close` | Minimise, Maximise, Close |
//! | `button-layout` = `appmenu:close` | Close alone |
//!
//! **So without this key, the buttons depended on whether the person had
//! granted the application their appearance settings**, which is a grant about
//! colours, not about buttons. Granting it took Minimise and Maximise away.
//! Answering the layout GTK draws anyway, ungranted, makes both cases the same:
//! the application shows its three, and alo adds Window actions beside them.
//! Nothing a person had is taken away by it. It is what an ungranted
//! application already shows.
//!
//! **What this does not reach**: an application that is not GTK, and a GTK
//! application outside a sandbox, which reads its own settings and not this
//! portal. Neither is shipped. Both are in the quirk.

/// The layout every granted GTK application is told: a menu on the leading
/// side, then Minimise, Maximise and Close on the trailing side, in GNOME's own
/// spelling.
pub const THE_LAYOUT: &str = "appmenu:minimize,maximize,close";

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Minimise and Close are both in it**, which is what the shell's table
    /// of GTK applications (`alo_shell`'s measured rows) rests on. If either
    /// left, every GTK row there would be wrong.
    #[test]
    fn the_layout_keeps_the_applications_own_minimise_and_close() {
        let (_, trailing) = THE_LAYOUT
            .split_once(':')
            .expect("a leading and a trailing side");
        let buttons: Vec<&str> = trailing.split(',').collect();
        assert_eq!(buttons, ["minimize", "maximize", "close"]);
    }
}
