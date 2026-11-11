//! **The person's answer to *Apps describe themselves*, carried from Settings
//! to the desktop** — the owner's decision of 2026-11-11, on as a machine
//! comes.
//!
//! Settings keeps the answer in the person's `access.toml`. This compositor
//! does nothing with it itself: what an application is told is the desktop's
//! to tell the person's session, so a written answer is held here until the
//! desktop takes it on the next frame, as a chosen pointer is
//! (`crate::the_pointers_look`). The desktop tells the session only when the
//! answer differs from what it last told, so turning another accessibility
//! switch over tells applications nothing new.

impl crate::Server {
    /// The answer as Settings just wrote it, held for the desktop.
    pub(crate) const fn apps_describing_themselves_is_now(&mut self, describe: bool) {
        self.describing_to_tell = Some(describe);
    }

    /// The answer the desktop has not been handed yet, taken: [`None`] when
    /// nothing was written since it last asked.
    pub(crate) const fn the_describing_to_tell(&mut self) -> Option<bool> {
        self.describing_to_tell.take()
    }
}

#[cfg(test)]
mod tests {
    /// **An answer written is handed over once**, and nothing is handed where
    /// nothing was written.
    #[cfg(target_os = "linux")]
    #[test]
    fn an_answer_is_handed_to_the_desktop_once() {
        use std::os::unix::fs::PermissionsExt as _;
        let Ok(directory) = tempfile::tempdir() else {
            unreachable!("no private directory");
        };
        assert!(
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
                .is_ok()
        );
        let Ok(mut server) = crate::Server::bind(directory.path(), "describing") else {
            unreachable!("no bound socket");
        };
        assert_eq!(server.the_describing_to_tell(), None);
        server.apps_describing_themselves_is_now(false);
        assert_eq!(server.the_describing_to_tell(), Some(false));
        assert_eq!(server.the_describing_to_tell(), None, "handed once");
    }
}
