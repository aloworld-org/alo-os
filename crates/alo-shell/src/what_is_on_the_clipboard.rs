//! **What a person copied somewhere else, read into a field of alo's own.**
//!
//! Nothing in alo OS had ever read a selection. The Bar takes typing, the
//! terminal takes typing, Settings now takes typing — and none of the three
//! could be pasted into, which for one of them is not an inconvenience but a
//! wall: an API key is around 164 characters, and **nobody types that.** So a
//! person could reach the screen that adds a provider and still not be able to
//! add one.
//!
//! # Why this is work rather than a line
//!
//! A selection is not held by the compositor. It belongs to whichever client
//! last copied something, and reading it is a conversation: the compositor
//! hands that client a pipe and asks it to write, and the client writes when
//! it gets round to it. `smithay`'s `request_data_device_client_selection` is
//! that ask, it is vendored in this tree, and **it had no callers.**
//!
//! # Nothing here waits, which is this repository's own rule
//!
//! `crate::what_a_person_asked_goes_to_the_agent` states it for the agent and
//! it holds just as hard here: *this is read from a compositor's frame loop,
//! so writing and waiting here would stop every window on the machine from
//! drawing.* A client that is busy, swapped out, or malicious can take as long
//! as it likes to write — and a compositor that blocked on the pipe would hang
//! the whole machine on a press of Control+V.
//!
//! So the read is **handed to a thread of its own**, which reads until the
//! client closes the pipe and posts what came back down a channel. The loop
//! asks [`TheClipboard::whatever_was_pasted`] once a frame, which never blocks
//! and usually answers with nothing.
//!
//! # What is refused, and why each
//!
//! - **More than [`AT_MOST`] bytes.** A client may write as much as it likes
//!   down that pipe, and a field that accepted it would let one press turn
//!   into a compositor holding a gigabyte. The limit is far above any
//!   credential and far below anything that hurts.
//! - **Anything that is not text.** The mime type asked for is `text/plain`
//!   with a UTF-8 charset, and bytes that are not UTF-8 are dropped rather
//!   than repaired: a key is typed by a person and mending one would be
//!   silently changing a credential, which `alo_models::Secret` refuses for
//!   the same reason.
//! - **Control characters**, which the form drops on the way in. A key copied
//!   out of a web page arrives with a newline on the end, and a field holding
//!   one looks right and does not work.

use std::io::Read;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};

/// The most a paste may bring, in bytes.
///
/// **Chosen against what a credential is, not against what a clipboard can
/// hold.** The longest key any provider in `shipped.toml` issues is a few
/// hundred characters; a person pasting an essay into a provider's address has
/// made a mistake this will not improve. Sixty-four kilobytes is far above the
/// first and far below anything that costs a machine memory it notices.
pub const AT_MOST: usize = 64 * 1024;

/// What the mime type asked for is.
///
/// **UTF-8 named explicitly.** `text/plain` alone is defined as ASCII by some
/// readings, and a provider name in Polish or a key a person keeps in a note
/// beside Cyrillic text would come back mangled or refused.
const AS_TEXT: &str = "text/plain;charset=utf-8";

/// One paste, as it came back from whichever client owns the selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatWasPasted {
    /// The text, with control characters already gone.
    text: String,
}

impl WhatWasPasted {
    /// The characters to put in the field, in order.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// The road from a press of Control+V to characters in a field.
///
/// **Held by the `Server` and asked once a frame**, which is
/// `crate::what_a_person_asked_goes_to_the_agent::TheAgentsDoor`'s shape and
/// for its reason.
#[derive(Debug)]
pub struct TheClipboard {
    /// Where a thread posts what it read.
    posts: Sender<WhatWasPasted>,
    /// Where the loop collects it.
    comes_back: Receiver<WhatWasPasted>,
}

impl Default for TheClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl TheClipboard {
    /// A clipboard nothing has been asked of.
    #[must_use]
    pub fn new() -> Self {
        let (posts, comes_back) = channel();
        Self { posts, comes_back }
    }

    /// Read this pipe on a thread, and post what it holds.
    ///
    /// The caller has already asked the selection's owner to write into the
    /// other end. **Taking the pipe rather than making it** is what lets this
    /// be tested without a client, a seat or a bus: a test writes into a pipe
    /// and reads the answer back, which is the whole of what this does.
    pub fn reading(&self, from: std::os::unix::io::OwnedFd) {
        let posts = self.posts.clone();
        // **Named, like the agent's thread is.** A thread with no name is a
        // line in a backtrace that says nothing about which part of a
        // compositor was waiting on what.
        let named = std::thread::Builder::new().name("alo-clipboard".to_owned());
        // A thread that will not spawn is a paste that does not happen, which
        // is what the person sees anyway if the client never writes. There is
        // nothing better to do with it and nothing to tell them that they
        // cannot already see.
        let _ = named.spawn(move || {
            let mut file = std::fs::File::from(from);
            let mut held = Vec::new();
            // **Bounded as it reads, not after.** Reading it all and then
            // measuring is how a client that writes without stopping takes the
            // machine's memory before anybody checks.
            let read = std::io::Read::by_ref(&mut file)
                .take(AT_MOST as u64)
                .read_to_end(&mut held);
            if read.is_err() {
                return;
            }
            let Ok(text) = String::from_utf8(held) else {
                // Not text. Dropped rather than repaired: mending a credential
                // is silently changing one.
                return;
            };
            let text: String = text.chars().filter(|letter| !letter.is_control()).collect();
            if text.is_empty() {
                return;
            }
            let _ = posts.send(WhatWasPasted { text });
        });
    }

    /// Everything that came back since this was last asked.
    ///
    /// **Never blocks**, and usually answers with nothing, which is what lets
    /// a frame loop call it every time round.
    pub fn whatever_was_pasted(&self) -> Vec<WhatWasPasted> {
        let mut came = Vec::new();
        loop {
            match self.comes_back.try_recv() {
                Ok(pasted) => came.push(pasted),
                // **Disconnected is treated as empty rather than as a fault.**
                // This holds its own sender, so it cannot happen while the
                // clipboard is alive; if it ever does, a paste that does not
                // arrive is what a person sees either way.
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return came,
            }
        }
    }

    /// The mime type a selection is asked for as.
    #[must_use]
    pub const fn as_text() -> &'static str {
        AS_TEXT
    }
}

#[cfg(test)]
#[path = "what_is_on_the_clipboard_tests.rs"]
mod tests;
