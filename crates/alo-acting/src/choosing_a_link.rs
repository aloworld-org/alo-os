//! The words on the compositor's door for **the person choosing what opens a
//! link** — the *Open with* sheet the owner approved on 2026-11-12 (frame
//! `25:52`), shown when an application sets `ask` on `OpenURI`.
//!
//! ```text
//! → choose-link org.gnome.Papers https://example.com/invoice org.mozilla.firefox io.gitlab.librewolf-community
//! ← chosen io.gitlab.librewolf-community     (or: cancelled, unanswered, cannot-show)
//! ```
//!
//! `docs/contracts/application-door.md`, *The person chooses what opens a
//! link*, is the published surface; this is it as working code.
//!
//! # The portal decides what is offered, and the desktop only asks
//!
//! The applications offered are the portal's answer — the person's default
//! first, then every other installed handler of that kind — and the desktop
//! draws exactly those. An answer naming one that was not offered is not an
//! answer ([`Chose::read`] is given the offer to hold it to).
//!
//! # It starts nothing
//!
//! `chosen` is the person's pick, carried back. The portal records it and then
//! writes `open-link` for it ([`crate::ToOpenALink`]), so every link is started
//! by one road.

use alo_applications::{Application, Link};

/// The word that begins this act.
pub const CHOOSE_LINK: &str = "choose-link";

/// The most a line of this act may be, in bytes.
pub const THE_MOST: usize = 4096;

/// The most applications one sheet may offer.
pub const THE_MOST_OFFERED: usize = 16;

/// A request for the person to choose what opens a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToChooseALink {
    /// The application that asked, as its sandbox named it.
    asking: Application,
    /// The link.
    link: Link,
    /// What is offered, the person's default first.
    offered: Vec<Application>,
}

impl ToChooseALink {
    /// Ask the person to choose among `offered` — the default first — what
    /// opens `link`, which `asking` asked to open.
    ///
    /// # Errors
    ///
    /// [`NotALinkToChoose::NothingOffered`] for an empty offer,
    /// [`NotALinkToChoose::TooMany`] for more than [`THE_MOST_OFFERED`], and
    /// [`NotALinkToChoose::OfferedTwice`] for one application offered twice.
    pub fn of(
        asking: Application,
        link: Link,
        offered: Vec<Application>,
    ) -> Result<Self, NotALinkToChoose> {
        if offered.is_empty() {
            return Err(NotALinkToChoose::NothingOffered);
        }
        if offered.len() > THE_MOST_OFFERED {
            return Err(NotALinkToChoose::TooMany);
        }
        for (at, one) in offered.iter().enumerate() {
            if offered
                .iter()
                .skip(at + 1)
                .any(|other| other.identifier() == one.identifier())
            {
                return Err(NotALinkToChoose::OfferedTwice);
            }
        }
        let asked = Self {
            asking,
            link,
            offered,
        };
        if asked.written().len() > THE_MOST {
            return Err(NotALinkToChoose::TooLong);
        }
        Ok(asked)
    }

    /// The application that asked.
    #[must_use]
    pub const fn asking(&self) -> &Application {
        &self.asking
    }

    /// The link.
    #[must_use]
    pub const fn link(&self) -> &Link {
        &self.link
    }

    /// What is offered, the person's default first.
    #[must_use]
    pub fn offered(&self) -> &[Application] {
        &self.offered
    }

    /// The person's default, which the sheet starts on.
    #[must_use]
    pub fn the_default(&self) -> Option<&Application> {
        self.offered.first()
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        let mut line = format!(
            "{CHOOSE_LINK} {} {}",
            self.asking.identifier(),
            self.link.as_str()
        );
        for one in &self.offered {
            line.push(' ');
            line.push_str(one.identifier());
        }
        line
    }

    /// One line, read as this act — [`None`] when it is not this act at all.
    ///
    /// A line that begins with [`CHOOSE_LINK`] and is not exactly this act's
    /// words is `Some(Err(…))`: a caller with a bug.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotALinkToChoose>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        if words.next() != Some(CHOOSE_LINK) {
            return None;
        }
        if line.len() > THE_MOST {
            return Some(Err(NotALinkToChoose::TooLong));
        }
        let (Some(asking), Some(link)) = (words.next(), words.next()) else {
            return Some(Err(NotALinkToChoose::NotEnoughWords));
        };
        let Some(asking) = an_identifier(asking) else {
            return Some(Err(NotALinkToChoose::NotAnApplication));
        };
        let Ok(link) = Link::checked(link) else {
            return Some(Err(NotALinkToChoose::NotALink));
        };
        let mut offered = Vec::new();
        for word in words {
            let Some(one) = an_identifier(word) else {
                return Some(Err(NotALinkToChoose::NotAnApplication));
            };
            offered.push(one);
        }
        Some(Self::of(asking, link, offered))
    }
}

/// An identifier exactly as written, or [`None`].
fn an_identifier(word: &str) -> Option<Application> {
    let application = Application::identified(word).ok()?;
    (application.identifier() == word).then_some(application)
}

/// Why a line that began as this act is not one, or why an offer cannot be
/// made. English, for whoever is fixing the caller; a person never reads one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotALinkToChoose {
    /// Longer than [`THE_MOST`].
    #[error("a choose-link line is at most {THE_MOST} bytes")]
    TooLong,
    /// No asking application, or no link.
    #[error("choose-link is followed by the asking application, a link and what is offered")]
    NotEnoughWords,
    /// A word that should be an application's identifier is not one.
    #[error("choose-link names applications by their identifiers")]
    NotAnApplication,
    /// The link is not one this machine opens.
    #[error("choose-link carries an http, https or mailto link")]
    NotALink,
    /// Nothing is offered.
    #[error("choose-link offers at least one application")]
    NothingOffered,
    /// More than [`THE_MOST_OFFERED`] are offered.
    #[error("choose-link offers at most {THE_MOST_OFFERED} applications")]
    TooMany,
    /// One application is offered twice.
    #[error("choose-link offers each application once")]
    OfferedTwice,
}

/// What the person did with the sheet, as the desktop answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chose {
    /// The person opened the link in this application, which was offered.
    Chosen(Application),
    /// The person pressed *Cancel* or Escape.
    Cancelled,
    /// Nobody answered while the sheet was up, and the desktop took it down.
    Unanswered,
    /// This desktop cannot ask the person now.
    CannotShow,
}

/// The word for [`Chose::Chosen`].
const CHOSEN: &str = "chosen";
/// The word for [`Chose::Cancelled`].
const CANCELLED: &str = "cancelled";
/// The word for [`Chose::Unanswered`].
const UNANSWERED: &str = "unanswered";
/// The word for [`Chose::CannotShow`].
const CANNOT_SHOW: &str = "cannot-show";

impl Chose {
    /// The answer, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Chosen(application) => format!("{CHOSEN} {}", application.identifier()),
            Self::Cancelled => CANCELLED.to_owned(),
            Self::Unanswered => UNANSWERED.to_owned(),
            Self::CannotShow => CANNOT_SHOW.to_owned(),
        }
    }

    /// An answer heard for `asked`, or [`None`] when it is not one — an
    /// unknown word, or `chosen` of an application that was not offered.
    /// **An unknown answer is never read as a choice.**
    #[must_use]
    pub fn read(heard: &str, asked: &ToChooseALink) -> Option<Self> {
        let heard = heard.trim_end_matches(['\r', '\n']);
        match heard {
            CANCELLED => return Some(Self::Cancelled),
            UNANSWERED => return Some(Self::Unanswered),
            CANNOT_SHOW => return Some(Self::CannotShow),
            _ => {}
        }
        let named = heard.strip_prefix(CHOSEN)?.strip_prefix(' ')?;
        asked
            .offered()
            .iter()
            .find(|one| one.identifier() == named)
            .cloned()
            .map(Self::Chosen)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn called(identifier: &str) -> Application {
        Application::identified(identifier).unwrap()
    }

    fn asked() -> ToChooseALink {
        ToChooseALink::of(
            called("org.gnome.Papers"),
            Link::checked("https://example.com/invoice").unwrap(),
            vec![
                called("org.mozilla.firefox"),
                called("io.gitlab.librewolf-community"),
            ],
        )
        .unwrap()
    }

    /// **What is written is what is read**, the default first.
    #[test]
    fn a_line_written_reads_back_as_itself() {
        let asked = asked();
        let line = asked.written();
        assert_eq!(
            line,
            "choose-link org.gnome.Papers https://example.com/invoice org.mozilla.firefox \
             io.gitlab.librewolf-community"
        );
        assert_eq!(ToChooseALink::read(&line), Some(Ok(asked.clone())));
        assert_eq!(
            asked.the_default().map(Application::identifier),
            Some("org.mozilla.firefox")
        );
    }

    /// **Another act is not this one.**
    #[test]
    fn another_act_is_not_this_one() {
        for line in [
            "open-link org.mozilla.firefox https://a.example/",
            "open org.mozilla.firefox",
            "front",
        ] {
            assert_eq!(ToChooseALink::read(line), None, "{line}");
        }
    }

    /// **Anything else after the act is refused**, and never read as a
    /// shorter request.
    #[test]
    fn what_is_not_this_act_is_refused() {
        for (line, refused) in [
            ("choose-link", NotALinkToChoose::NotEnoughWords),
            (
                "choose-link org.gnome.Papers",
                NotALinkToChoose::NotEnoughWords,
            ),
            (
                "choose-link org.gnome.Papers https://a.example/",
                NotALinkToChoose::NothingOffered,
            ),
            (
                "choose-link org.gnome.Papers file:///etc/shadow org.mozilla.firefox",
                NotALinkToChoose::NotALink,
            ),
            (
                "choose-link org.gnome.Papers https://a.example/ /usr/bin/firefox",
                NotALinkToChoose::NotAnApplication,
            ),
            (
                "choose-link org.gnome.Papers https://a.example/  org.mozilla.firefox",
                NotALinkToChoose::NotAnApplication,
            ),
            (
                "choose-link org.gnome.Papers https://a.example/ org.mozilla.firefox org.mozilla.firefox",
                NotALinkToChoose::OfferedTwice,
            ),
        ] {
            assert_eq!(ToChooseALink::read(line), Some(Err(refused)), "{line}");
        }
        let many: Vec<String> = (0..=THE_MOST_OFFERED)
            .map(|n| format!("com.example.B{n}"))
            .collect();
        let line = format!(
            "choose-link org.gnome.Papers https://a.example/ {}",
            many.join(" ")
        );
        assert_eq!(
            ToChooseALink::read(&line),
            Some(Err(NotALinkToChoose::TooMany))
        );
        let long = format!(
            "choose-link org.gnome.Papers https://a.example/{} org.mozilla.firefox",
            "a".repeat(THE_MOST)
        );
        assert_eq!(
            ToChooseALink::read(&long),
            Some(Err(NotALinkToChoose::TooLong))
        );
    }

    /// **The longest link, with as many applications as may be offered under
    /// ordinary names, fits on one line.**
    #[test]
    fn the_longest_link_and_a_full_offer_fit() {
        let link = format!(
            "https://example.com/{}",
            "a".repeat(alo_applications::web_address::LONGEST - 20)
        );
        let offered: Vec<Application> = (0..THE_MOST_OFFERED)
            .map(|n| called(&format!("org.example.Browser{n:02}")))
            .collect();
        let asked = ToChooseALink::of(
            called("org.gnome.Papers"),
            Link::checked(&link).unwrap(),
            offered,
        )
        .unwrap();
        assert!(ToChooseALink::read(&asked.written()).unwrap().is_ok());
    }

    /// **Every answer reads back, and `chosen` of something not offered, or an
    /// unknown word, is no answer at all.**
    #[test]
    fn an_answer_is_one_of_the_offer_or_nothing() {
        let asked = asked();
        for chose in [
            Chose::Chosen(called("io.gitlab.librewolf-community")),
            Chose::Cancelled,
            Chose::Unanswered,
            Chose::CannotShow,
        ] {
            assert_eq!(Chose::read(&chose.written(), &asked), Some(chose.clone()));
            assert_eq!(
                Chose::read(&format!("{}\n", chose.written()), &asked),
                Some(chose)
            );
        }
        for heard in [
            "chosen org.example.Evil",
            "chosen",
            "chosen  org.mozilla.firefox",
            "opened",
            "",
        ] {
            assert_eq!(Chose::read(heard, &asked), None, "{heard}");
        }
    }
}
