//! The words on the compositor's door for **starting an application with a link
//! a person followed** — added 2026-11-12 by the owner's decision, *"Yes, add it
//! (Recommended)"*: a narrow connection by which the portal service asks the
//! desktop to start the chosen application exactly as a person's press does.
//!
//! ```text
//! → open-link org.mozilla.firefox https://example.com/invoice
//! ← acted                      (or: not-installed, could-not-ask)
//! ```
//!
//! # Who writes it, and why it is not an agent's verb
//!
//! The open-with portal writes it, for a link an application asked to have
//! opened — a person following a link in their mail or a document. **No agent
//! verb reaches this act**: `alo-agentd` writes only `open`, `focus` and
//! `close`, and the door admits only the person's own uid, which the agent is
//! not (`docs/contracts/application-door.md`). The authority is the person's
//! own act of following the link, and the decision of what opens it was the
//! portal's — the desktop is the hand, never a second judge.
//!
//! # Nothing in it can be a command
//!
//! Three words, and the third is an [`alo_applications::Link`], held to being
//! an `http`, `https` or `mailto` address on one line: it begins with a letter,
//! so it can never be read as an option, and it has no space, so it is exactly
//! one argument. The desktop hands it to the application as that one argument,
//! or as the one URI of a bus `Open` — never to a shell.

use alo_applications::{Application, Link};

/// The word that begins this act.
pub const OPEN_LINK: &str = "open-link";

/// The most a line of this act may be, in bytes: the act, an identifier and
/// the longest link. Longer than [`crate::Request::THE_MOST`], which bounds
/// every other line, because a link may be up to
/// `alo_applications::web_address::LONGEST` bytes on its own.
pub const THE_MOST: usize = 2560;

/// A request to start an application with a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToOpenALink {
    /// The application that opens it, by its identifier.
    application: Application,
    /// The link.
    link: Link,
}

impl ToOpenALink {
    /// Start `application` with `link`.
    #[must_use]
    pub const fn with(application: Application, link: Link) -> Self {
        Self { application, link }
    }

    /// The application to start.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// The link to start it with.
    #[must_use]
    pub const fn link(&self) -> &Link {
        &self.link
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        format!(
            "{OPEN_LINK} {} {}",
            self.application.identifier(),
            self.link.as_str()
        )
    }

    /// One line, read as this act — [`None`] when it is not this act at all,
    /// so the door can read it as one of the others.
    ///
    /// A line that begins with [`OPEN_LINK`] and is not exactly an identifier
    /// and a link after it is `Some(Err(…))`: it was meant as this act and is a
    /// caller with a bug.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotALinkToOpen>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        if words.next() != Some(OPEN_LINK) {
            return None;
        }
        if line.len() > THE_MOST {
            return Some(Err(NotALinkToOpen::TooLong));
        }
        let (Some(named), Some(link), None) = (words.next(), words.next(), words.next()) else {
            return Some(Err(NotALinkToOpen::NotTwoWords));
        };
        let Ok(application) = Application::identified(named) else {
            return Some(Err(NotALinkToOpen::NotAnApplication));
        };
        if application.identifier() != named {
            return Some(Err(NotALinkToOpen::NotAnApplication));
        }
        let Ok(link) = Link::checked(link) else {
            return Some(Err(NotALinkToOpen::NotALink));
        };
        Some(Ok(Self { application, link }))
    }
}

/// Why a line that began as this act is not one.
///
/// English, read by whoever is fixing the caller. A person never reads one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotALinkToOpen {
    /// Longer than [`THE_MOST`].
    #[error("an open-link line is at most {THE_MOST} bytes")]
    TooLong,
    /// Not exactly an identifier and a link after the act.
    #[error("open-link is followed by an application and a link, each once")]
    NotTwoWords,
    /// The second word is not an application's identifier.
    #[error("open-link names an application by its identifier")]
    NotAnApplication,
    /// The third word is not a link this machine opens.
    #[error("open-link carries an http, https or mailto link")]
    NotALink,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn firefox() -> Application {
        Application::identified("org.mozilla.firefox").unwrap()
    }

    /// **What is written is what is read**, for a web link and a mail link.
    #[test]
    fn a_line_written_reads_back_as_itself() {
        for address in [
            "https://example.com/a?b=c#d",
            "mailto:anna@example.com?subject=March",
        ] {
            let asked = ToOpenALink::with(firefox(), Link::checked(address).unwrap());
            let line = asked.written();
            assert_eq!(ToOpenALink::read(&line), Some(Ok(asked.clone())));
            assert_eq!(ToOpenALink::read(&format!("{line}\n")), Some(Ok(asked)));
        }
    }

    /// **Another act is not this one**, so the door reads it as the other.
    #[test]
    fn another_act_is_not_this_one() {
        for line in [
            "open org.mozilla.firefox",
            "count org.mozilla.firefox",
            "front",
        ] {
            assert_eq!(ToOpenALink::read(line), None, "{line}");
        }
    }

    /// **Anything else after the act is refused**, and a refusal is never a
    /// shorter, different request.
    #[test]
    fn what_is_not_exactly_an_application_and_a_link_is_refused() {
        for (line, refused) in [
            ("open-link", NotALinkToOpen::NotTwoWords),
            ("open-link org.mozilla.firefox", NotALinkToOpen::NotTwoWords),
            (
                "open-link org.mozilla.firefox https://a.example/ https://b.example/",
                NotALinkToOpen::NotTwoWords,
            ),
            (
                "open-link  https://a.example/",
                NotALinkToOpen::NotAnApplication,
            ),
            (
                "open-link org.mozilla.firefox file:///etc/shadow",
                NotALinkToOpen::NotALink,
            ),
            (
                "open-link org.mozilla.firefox --profile=/tmp/x",
                NotALinkToOpen::NotALink,
            ),
            (
                "open-link /usr/bin/firefox https://a.example/",
                NotALinkToOpen::NotAnApplication,
            ),
        ] {
            assert_eq!(ToOpenALink::read(line), Some(Err(refused)), "{line}");
        }
        let long = format!(
            "open-link org.mozilla.firefox https://example.com/{}",
            "a".repeat(THE_MOST)
        );
        assert_eq!(ToOpenALink::read(&long), Some(Err(NotALinkToOpen::TooLong)));
    }

    /// **The longest link a link may be fits on one line of this act.**
    #[test]
    fn the_longest_link_fits() {
        let link = format!(
            "https://example.com/{}",
            "a".repeat(alo_applications::web_address::LONGEST - 20)
        );
        let asked = ToOpenALink::with(firefox(), Link::checked(&link).unwrap());
        assert!(asked.written().len() <= THE_MOST);
        assert!(ToOpenALink::read(&asked.written()).unwrap().is_ok());
    }
}
