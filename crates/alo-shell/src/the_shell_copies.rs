//! **alo putting words of its own on the clipboard** — the first thing the
//! compositor itself has ever copied: the GitHub sign-in sheet's *Copy*
//! (`crate::the_github_sheet`).
//!
//! Until now this shell only carried a selection from one client to another,
//! and read one into its own fields (`crate::what_is_on_the_clipboard`). A
//! selection the compositor owns is `smithay`'s `set_data_device_selection`:
//! the seat offers the text under the mime types below, and when an
//! application pastes, `SelectionHandler::send_selection` hands over a pipe to
//! write the text into ([`written`]).
//!
//! # Nothing here waits
//!
//! The application at the other end of the pipe reads when it gets round to
//! it, and a compositor that blocked writing would stop every window on the
//! machine — `crate::what_is_on_the_clipboard`'s rule, the other way round. So
//! the text is written **on a thread of its own**, which ends when it has
//! written or the reader has gone.
//!
//! # And alo's own fields can paste it back
//!
//! A selection the compositor owns is one `request_data_device_client_selection`
//! refuses — it asks a client, and there is none. So the paste road asks here
//! first ([`what_the_shell_copied`]) and writes the text down the same pipe a
//! client would have, so a code copied on the sheet pastes into alo's own
//! fields as it does into any application.

use std::io::Write as _;
use std::os::unix::io::OwnedFd;
use std::sync::Arc;

/// The mime types the text is offered as: UTF-8 named explicitly, plain text,
/// and the two names X applications ask for under `Xwayland`.
const OFFERED_AS: [&str; 4] = [
    "text/plain;charset=utf-8",
    "text/plain",
    "UTF8_STRING",
    "STRING",
];

/// Text alo put on the clipboard itself: the seat's selection while it is the
/// latest thing copied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopiedByTheShell {
    /// The text, shared with each thread writing it.
    text: Arc<str>,
}

impl CopiedByTheShell {
    /// `text`, to be copied.
    pub(crate) fn of(text: &str) -> Self {
        Self {
            text: Arc::from(text),
        }
    }
}

/// **Write what was copied down `to`, on a thread of its own**, and close it,
/// which is how the reader learns it has everything.
pub(crate) fn written(to: OwnedFd, copied: &CopiedByTheShell) {
    let text = Arc::clone(&copied.text);
    let named = std::thread::Builder::new().name("alo-copying".to_owned());
    // A thread that will not spawn is a paste that brings nothing, which is
    // what the person sees anyway if the reader has gone; there is nothing
    // better to do and nobody to tell.
    let _ = named.spawn(move || {
        let mut file = std::fs::File::from(to);
        let _ = file.write_all(text.as_bytes());
    });
}

impl crate::Server {
    /// **Put `text` on the clipboard**, as the seat's selection. Nothing where
    /// there is no seat yet, which is a machine with no keyboard to paste with.
    pub(crate) fn the_shell_copies(&mut self, text: &str) {
        let Some(keyboard) = self.surfaces.keyboard.as_ref() else {
            return;
        };
        let seat = keyboard.seat.clone();
        smithay::wayland::selection::data_device::set_data_device_selection(
            &self.surfaces.display,
            &seat,
            OFFERED_AS.iter().map(|it| (*it).to_owned()).collect(),
            CopiedByTheShell::of(text),
        );
    }
}

/// **What alo itself copied, while it is still the latest thing copied** on
/// `seat` — for the paste road to write straight back.
pub(crate) fn what_the_shell_copied(
    seat: &smithay::input::Seat<crate::surfaces::Surfaces>,
) -> Option<CopiedByTheShell> {
    smithay::wayland::selection::data_device::current_data_device_selection_userdata(seat)
        .map(|copied| copied.clone())
}

#[cfg(test)]
mod tests {
    use std::io::Read as _;

    use super::*;

    /// **What was copied arrives whole down the pipe**, and the pipe is closed
    /// after it, so a reader reading to the end is not left waiting.
    #[test]
    fn what_was_copied_arrives_whole_and_the_pipe_closes() {
        let Ok((mut reading, writing)) = std::io::pipe() else {
            unreachable!("no pipe");
        };
        written(OwnedFd::from(writing), &CopiedByTheShell::of("4F2A-9C1D"));
        let mut came = String::new();
        assert!(reading.read_to_string(&mut came).is_ok());
        assert_eq!(came, "4F2A-9C1D");
    }

    /// **The text is offered as UTF-8 text first**, the type alo's own paste
    /// road asks for (`crate::what_is_on_the_clipboard`).
    #[test]
    fn it_is_offered_as_the_text_alos_own_paste_asks_for() {
        assert_eq!(
            OFFERED_AS.first().copied(),
            Some(crate::what_is_on_the_clipboard::TheClipboard::as_text())
        );
    }
}
