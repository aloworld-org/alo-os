//! **What the person's pointer looks like**, as they last said in Settings, and
//! whether applications have been told yet.
//!
//! Two audiences, as with every setting a person changes here. **This
//! compositor** draws its own pointer (`crate::the_pointer_drawn`) and draws it
//! the new way on the next frame. **Applications that draw their own pointer**
//! read the cursor theme from their environment when they start, so the desktop
//! has to tell the person's session the theme's new name
//! (`TheDesktop::the_pointer_changed`). That is why a change is held here until
//! the desktop has taken it: a choice made in Settings and never told to the
//! session would change alo's pointer and leave every older application on the
//! old one, which is the pointer changing look at a window's edge again.

use alo_appearance::PointerShape;

/// The pointer, as the person last said.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ThePointersLook {
    /// Which arrow it is.
    pub shape: PointerShape,
}

impl ThePointersLook {
    /// The arrow `alo_cursors` draws for this look.
    #[must_use]
    pub const fn silhouette(self) -> alo_cursors::Silhouette {
        match self.shape {
            PointerShape::Kite => alo_cursors::Silhouette::Kite,
            PointerShape::Classic => alo_cursors::Silhouette::Classic,
        }
    }

    /// The cursor theme an application that draws its own pointer is told to
    /// draw from.
    #[must_use]
    pub const fn the_theme(self) -> &'static str {
        alo_cursors::theme::the_name_of(self.silhouette())
    }
}

impl crate::Server {
    /// The pointer as the person chose it, told at stand-up.
    ///
    /// **Told rather than read**, for the reason `Server::the_window_titles_are`
    /// gives. Not a change to pass on: the desktop read it from the same file and
    /// told the session itself at sign-in.
    pub fn the_pointer_is(&mut self, look: ThePointersLook) {
        self.pointers_look = look;
    }

    /// The pointer as the person just changed it in Settings: drawn so from the
    /// next frame, and held until the desktop takes it for the session
    /// ([`Self::the_pointer_change_to_tell`]).
    pub(crate) fn the_pointer_changed_to(&mut self, look: ThePointersLook) {
        if look != self.pointers_look {
            self.pointers_look = look;
            self.pointer_change_to_tell = Some(look);
        }
    }

    /// How the pointer looks.
    #[must_use]
    pub const fn the_pointers_look(&self) -> ThePointersLook {
        self.pointers_look
    }

    /// The change applications have not been told about yet, taken: [`None`]
    /// when nothing changed since the desktop last asked.
    pub(crate) const fn the_pointer_change_to_tell(&mut self) -> Option<ThePointersLook> {
        self.pointer_change_to_tell.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The look names the arrow and the theme that go with it**, so the
    /// pointer alo draws and the one an application draws from its theme are
    /// the same arrow.
    #[test]
    fn each_shape_has_its_arrow_and_its_theme() {
        let kite = ThePointersLook::default();
        assert_eq!(kite.silhouette(), alo_cursors::Silhouette::Kite);
        assert_eq!(kite.the_theme(), "alo");
        let classic = ThePointersLook {
            shape: PointerShape::Classic,
        };
        assert_eq!(classic.silhouette(), alo_cursors::Silhouette::Classic);
        assert_eq!(classic.the_theme(), "alo-classic");
    }

    /// **The session is told once per change, and never about nothing**: the
    /// look a desktop reads at stand-up is not a change, choosing what is
    /// already chosen is not one, and a change is told once and then gone.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_change_is_told_once_and_nothing_else_is() {
        use std::os::unix::fs::PermissionsExt as _;
        let Ok(directory) = tempfile::tempdir() else {
            unreachable!("no private directory");
        };
        assert!(
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
                .is_ok()
        );
        let Ok(mut server) = crate::Server::bind(directory.path(), "the-pointer") else {
            unreachable!("no bound socket");
        };
        let classic = ThePointersLook {
            shape: PointerShape::Classic,
        };
        assert_eq!(server.the_pointer_change_to_tell(), None);
        server.the_pointer_is(classic);
        assert_eq!(server.the_pointers_look(), classic);
        assert_eq!(server.the_pointer_change_to_tell(), None, "stand-up is not a change");
        server.the_pointer_changed_to(classic);
        assert_eq!(server.the_pointer_change_to_tell(), None, "the same choice again");
        server.the_pointer_changed_to(ThePointersLook::default());
        assert_eq!(
            server.the_pointer_change_to_tell(),
            Some(ThePointersLook::default())
        );
        assert_eq!(server.the_pointer_change_to_tell(), None, "told once");
        assert_eq!(server.the_pointers_look(), ThePointersLook::default());
    }
}
